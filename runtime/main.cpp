#include "psyqo/application.hh"
#include "psyqo/fragments.hh"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/triangles.hh"
#include "psyqo/scene.hh"
#include "psyqo/advancedpad.hh"
#include "memory_card_backend.hpp"
#include "psyqo/trigonometry.hh"
#include "scene.hh"
#include "texture.hpp"
#include "palette.hpp"
#include "skeletal.hpp"
#include "audio-bank.hh"
#ifdef EPOK_HAS_SEQUENCES
#include "sequence_clock.hpp"
#endif
#include "music.hpp"
#include "streaming.hpp"
#include "lifecycle.hpp"
#include "scene_service.hpp"
#include "sprites.hpp"
#include "particles.hpp"
#ifdef EPOK_BLUEPRINTS
#include "blueprint_playback_service.hpp"
#endif
#ifdef EPOK_EFFECTS
#include "particle_effect_service.hpp"
#endif
#include "affine.hpp"
#include "transform_cache.hpp"
#include "motion_interpolation.hpp"
#include "frustum.hpp"
#include "visibility.hpp"
#include "gte_geometry.hpp"
#include "polygon.hpp"
#include "retained.hpp"
#include "hud.hpp"
#include "loading_renderer.hpp"
#include "debug_hud.hpp"
#include "serial_debug.hpp"
#include "lua_runtime.hpp"
#include "common/syscalls/syscalls.h"
#include "lighting.hpp"
#include "shadows.hpp"
#include "common/hardware/counters.h"

namespace epok {
LightingEnvironment lighting_environment;
LightingStats lighting_work;
LightingStats lighting_stats;
MeshStats mesh_stats;
PerformanceStats performance_stats;
} // namespace epok

namespace {
#ifdef EPOK_PLAYBACK_WAITS
void observe_blueprint_playback(){
  // Capture outcomes even for inactive consumers; their continuation dispatch
  // remains paused. No gameplay runs during observation or playback-slot reuse.
  for(size_t i=0;i<epok::level.actor_count();++i) {
    if(auto* actor=epok::object_registry.resolve<epok::Actor>(epok::level.actor_at(i))) {
      actor->blueprint_observe();
      for(size_t c=0;c<actor->component_count();++c)
        if(auto* component=epok::object_registry.resolve<epok::ActorComponent>(actor->component_id(c))) component->blueprint_observe();
    }
  }
}
#endif
using epok::Fixed;
constexpr int buckets = 512;
constexpr size_t scratch_vertex_capacity = 1024 / sizeof(epok::ProjectedVertex);
// The R3000 has no data cache, so every read of a projected vertex is a main-memory
// access, and the classifier and the emitter re-read each one for every quad that
// shares it. The 1 KiB scratchpad answers in a single cycle and a chunk's vertices
// fit in it; larger chunks keep the RAM array. File scope avoids a static guard.
__attribute__((section(".scratchpad"))) epok::ProjectedVertex scratch_vertices[scratch_vertex_capacity];
constexpr size_t capacity = epok::render_capacity;
// If the entire archive fits, loaded pages cannot be evicted by another valid
// page. Retain whole objects eagerly, like resident geometry, without testing
// per-chunk packet validity on every subsequent frame.
constexpr bool demand_streaming = epok::stream_page_count > epok::stream_pool_pages;
static_assert(
    capacity <= 8192,
    "Scene exceeds the initial renderer's 8192-triangle buffer budget");
psyqo::Trig<> trig;
using Matrix = epok::Affine<Fixed>;
epok::PerformanceStats performance_work;
#ifdef EPOK_PROFILE_DETAIL
uint32_t non_streamed_visible = 0;
#endif
struct ScanlineScope {
  uint32_t& total;
  uint16_t begin=COUNTERS[1].value;
  ~ScanlineScope(){total+=uint16_t(COUNTERS[1].value-begin);}
};
std::array<Matrix, epok::objects.size()> world;
epok::TransformCache<Fixed,epok::objects.size()> transform_cache;
epok::MotionInterpolation<Fixed,epok::objects.size(),epok::motion_interpolation> motion;
psyqo::AdvancedPad pad;
epok::PsyqoMemoryCardDriver card_driver;
epok::CollisionWorld<Fixed,epok::objects.size()> collision_world;
epok::SpriteRenderer<> sprites;
epok::ParticlePool particles;
epok::PaletteRenderer palettes;
epok::HudRenderer hud;
epok::LoadingRenderer loading_renderer;
psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> fade_pages[2];
psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> fade_restore_pages[2];
psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> fade_rectangles[2];
epok::BlobRenderer blobs;
epok::LightingRenderer<epok::objects.size()> lighting;
// Retained GPU packets for static meshes, one record per scene quad.
epok::RetainedGeometry<epok::objects.size(), epok::retained_geometry ? epok::retained_quad_capacity : 1> retained;
std::array<epok::StreamObjectBinding, epok::stream_archive_fits ? epok::objects.size() : 0> stream_bindings{};
epok::DataHandle camera_override;

// Degrees in the editor and scripts; fixed point throughout the PSX runtime.
void rotate(Fixed *p, const Fixed *degrees) {
  for (int axis = 0; axis < 3; ++axis) {
    int32_t wrapped = degrees[axis].raw();
    if (!wrapped)continue;
    // Angles rarely exceed a turn: subtract instead of a hardware division.
    constexpr int32_t turn = 360 * 4096;
    if (wrapped >= turn || wrapped <= -turn) wrapped %= turn;
    if (wrapped < 0)
      wrapped += turn;
    if (!wrapped)continue;
    // wrapped / 720 exactly, via a rounded-up 32-bit reciprocal.
    psyqo::Angle angle(int32_t((uint64_t(uint32_t(wrapped)) * 5965233u) >> 32), psyqo::Angle::RAW);
    auto s = trig.sin(angle), c = trig.cos(angle);
    int a = (axis + 1) % 3, b = (axis + 2) % 3;
    auto pa = p[a] * c - p[b] * s;
    p[b] = p[a] * s + p[b] * c;
    p[a] = pa;
  }
}
// Mesh vertices are projected by the GTE in Q8 camera units (see gte_geometry.hpp).
using Units = epok::CameraQ8;
using epok::ProjectedVertex;
using epok::QuadPacket;
#ifdef EPOK_PROFILE_DETAIL
#define EPOK_DETAIL_BEGIN(name) const uint16_t name##_begin = COUNTERS[1].value
#define EPOK_DETAIL_END(name, field) performance_work.field += uint16_t(COUNTERS[1].value - name##_begin)
#else
#define EPOK_DETAIL_BEGIN(name) (void)0
#define EPOK_DETAIL_END(name, field) (void)0
#endif
// Texture packet words and UV scaling resolved once per material change, not
// per triangle. Keyed on the texture index and blend mode a material exposes.
struct MaterialState {
  int texture = -2;
  epok::BlendMode blend = epok::BlendMode::Cutout;
  const epok::Texture* tex = nullptr;
  uint32_t clut16 = 0, tpage16 = 0, command = 0x30000000u;
  int32_t scale_u = 0, scale_v = 0, base_v = 0;
  void update(const epok::Material& material) {
    if (material.texture == texture && material.blend == blend) return;
    texture = material.texture; blend = material.blend; tex = epok::texture(material.texture);
    if (!tex) { command = 0x30000000u; return; }
    const auto clut = epok::texture_clut(*tex);
    const auto page = epok::texture_page(*tex, material.blend);
    uint16_t clut_bits, page_bits;
    __builtin_memcpy(&clut_bits, &clut, 2); __builtin_memcpy(&page_bits, &page, 2);
    clut16 = uint32_t(clut_bits) << 16; tpage16 = uint32_t(page_bits) << 16;
    command = 0x34000000u | (material.blend == epok::BlendMode::Cutout ? 0u : 0x02000000u);
    scale_u = tex->width - 1; scale_v = tex->height - 1; base_v = tex->y % 256;
  }
  // Same mapping as texture_uv: Q12 face coordinates to 8-bit page pixels.
  uint32_t uv(int32_t u, int32_t v) const {
    u = u < 0 ? 0 : u > 4096 ? 4096 : u; v = v < 0 ? 0 : v > 4096 ? 4096 : v;
    return (uint32_t(u * scale_u >> 12) & 255u) | ((uint32_t(base_v + (v * scale_v >> 12)) & 255u) << 8);
  }
};
inline uint32_t color_bits(psyqo::Color c) { return c.packed & 0x00ffffffu; }
struct PolygonEmitter {
  psyqo::OrderingTable<buckets>& table;
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTriangle>* gouraud;
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTexturedTriangle>* textured;
  // Per-frame packets take slots from the top; retained packets own the bottom.
  uint32_t next_dynamic = capacity;
  uint32_t emitted = 0, retained_emitted = 0;
  bool editable = false;
  // 0: rejected, 1: direct GPU submission, 2: software clipping required.
  inline int classify(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c) {
    if (a.outcode & b.outcode & c.outcode) return 0;
    if (!(a.visible && b.visible && c.visible)) return 2;
    // Inside all six planes, projected spans are bounded by the viewport.
    // Keep wide/near-plane cases on the checked path below.
    if (epok::display_width <= 1023 && epok::display_height <= 511 && !(a.outcode | b.outcode | c.outcode)) {
      const int32_t area = epok::screen_area(a, b, c);
      if (area == 0) return 0;
      if (editable && area < 0) { ++epok::mesh_stats.backfaces; return 0; }
      return 1;
    }
    int min_x = a.screen.x, max_x = min_x, min_y = a.screen.y, max_y = min_y;
    if (b.screen.x < min_x) min_x = b.screen.x; if (b.screen.x > max_x) max_x = b.screen.x;
    if (b.screen.y < min_y) min_y = b.screen.y; if (b.screen.y > max_y) max_y = b.screen.y;
    if (c.screen.x < min_x) min_x = c.screen.x; if (c.screen.x > max_x) max_x = c.screen.x;
    if (c.screen.y < min_y) min_y = c.screen.y; if (c.screen.y > max_y) max_y = c.screen.y;
    if (max_x - min_x > 1023 || max_y - min_y > 511) return 2;
    const int32_t area = epok::screen_area(a, b, c);
    if (area == 0) return 0;
    if (editable && area < 0) { ++epok::mesh_stats.backfaces; return 0; }
    return 1;
  }
  // Three visible vertices within the hardware span limits, already checked for
  // degeneracy and facing. Colors are final packet values, UVs are u | v << 8.
  inline void direct(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c,
                     uint32_t ca, uint32_t cb, uint32_t cc, uint32_t uva, uint32_t uvb, uint32_t uvc,
                     const QuadPacket& q) {
    const int depth = Units::bucket(a.camera[2], b.camera[2], c.camera[2]) + q.depth_bias;
    if (depth < 0 || depth >= buckets) return;
    if (next_dynamic <= retained.slot_top()) { ++epok::lighting_work.dropped_triangles; return; }
    ++emitted;
    if (q.textured) {
      auto& f = textured[--next_dynamic];
      uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
      w[0] = q.command | ca; w[1] = a.screen.packed; w[2] = uva | q.clut16;
      w[3] = cb; w[4] = b.screen.packed; w[5] = uvb | q.tpage16;
      w[6] = cc; w[7] = c.screen.packed; w[8] = uvc;
      table.insert(f, depth);
      return;
    }
    auto& f = gouraud[--next_dynamic];
    uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
    w[0] = 0x30000000u | ca; w[1] = a.screen.packed; w[2] = cb; w[3] = b.screen.packed; w[4] = cc; w[5] = c.screen.packed;
    table.insert(f, depth);
  }
  // Retained slot: screen words every frame, colour words only when they changed.
  inline void retained_emit(uint32_t slot, const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c,
                            const epok::RetainedQuad& rec, const uint32_t* colors, int corner_a, int ib, int ic) {
    const int depth = Units::bucket(a.camera[2], b.camera[2], c.camera[2]) + rec.depth_bias;
    if (depth < 0 || depth >= buckets) return;
    ++emitted; ++retained_emitted;
    if (rec.textured) {
      auto& f = textured[slot];
      uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
      w[1] = a.screen.packed; w[4] = b.screen.packed; w[7] = c.screen.packed;
      if (colors) { w[0] = rec.command | colors[corner_a]; w[3] = colors[ib]; w[6] = colors[ic]; }
      table.insert(f, depth);
      return;
    }
    auto& f = gouraud[slot];
    uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
    w[1] = a.screen.packed; w[3] = b.screen.packed; w[5] = c.screen.packed;
    if (colors) { w[0] = 0x30000000u | colors[corner_a]; w[2] = colors[ib]; w[4] = colors[ic]; }
    table.insert(f, depth);
  }
  // Software clipping stays out of line so the per-quad loop remains compact.
  // Colors here are the fogged 0..255 values; modulation happens after clipping.
  __attribute__((noinline)) void clipped(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c,
                                         uint32_t ca, uint32_t cb, uint32_t cc, uint32_t uva, uint32_t uvb, uint32_t uvc,
                                         const QuadPacket& q) {
    const uint8_t planes = a.outcode | b.outcode | c.outcode;
    epok::ClipVertex buffers[2][12];
    const ProjectedVertex* points[3] = {&a, &b, &c};
    const uint32_t colors[3] = {ca, cb, cc}, uvs[3] = {uva, uvb, uvc};
    for (int i = 0; i < 3; ++i) {
      for (int d = 0; d < 3; ++d) buffers[0][i].p[d] = points[i]->camera[d];
      buffers[0][i].color[0] = colors[i] & 255; buffers[0][i].color[1] = (colors[i] >> 8) & 255; buffers[0][i].color[2] = (colors[i] >> 16) & 255;
      buffers[0][i].uv[0] = int32_t(uvs[i] & 255) << 16; buffers[0][i].uv[1] = int32_t((uvs[i] >> 8) & 255) << 16;
    }
    int from = 0;
    const int count = epok::clip_polygon<Units>(buffers, 3, planes, from);
    if (count < 3) return;
    ++epok::mesh_stats.clipped;
    ProjectedVertex p[12]; uint32_t col[12], tex[12];
    for (int i = 0; i < count; ++i) {
      const auto& v = buffers[from][i];
      for (int d = 0; d < 3; ++d) p[i].camera[d] = v.p[d];
      epok::project_cpu<Units, epok::display_width, epok::display_height>(p[i]);
      const uint32_t packed = epok::pack_color(uint8_t(v.color[0]), uint8_t(v.color[1]), uint8_t(v.color[2]));
      col[i] = q.textured ? epok::modulate_color(packed) : packed;
      tex[i] = uint32_t(uint8_t(v.uv[0] >> 16)) | (uint32_t(uint8_t(v.uv[1] >> 16)) << 8);
    }
    for (int i = 1; i + 1 < count; ++i) {
      const int32_t area = epok::screen_area(p[0], p[i], p[i + 1]);
      if (area == 0) continue;
      if (editable && area < 0) { ++epok::mesh_stats.backfaces; continue; }
      direct(p[0], p[i], p[i + 1], col[0], col[i], col[i + 1], tex[0], tex[i], tex[i + 1], q);
    }
  }
  // Camera-space triangles produced by UV scrolling take the generic route.
  __attribute__((noinline)) void unprojected(const epok::UvVertex& a, const epok::UvVertex& b, const epok::UvVertex& c,
                                             const MaterialState& material, const QuadPacket& q) {
    const epok::UvVertex* v[3] = {&a, &b, &c};
    ProjectedVertex p[3]; uint32_t shaded[3], final[3], uv[3];
    for (int i = 0; i < 3; ++i) {
      for (int d = 0; d < 3; ++d) p[i].camera[d] = v[i]->camera[d];
      p[i].fog = 0;
      epok::project_cpu<Units, epok::display_width, epok::display_height>(p[i]);
      shaded[i] = epok::pack_color(uint8_t(v[i]->color[0]), uint8_t(v[i]->color[1]), uint8_t(v[i]->color[2]));
      final[i] = q.textured ? epok::modulate_color(shaded[i]) : shaded[i];
      uv[i] = q.textured ? material.uv(v[i]->uv[0], v[i]->uv[1]) : 0;
    }
    switch (classify(p[0], p[1], p[2])) {
    case 1: direct(p[0], p[1], p[2], final[0], final[1], final[2], uv[0], uv[1], uv[2], q); break;
    case 2: clipped(p[0], p[1], p[2], shaded[0], shaded[1], shaded[2], uv[0], uv[1], uv[2], q); break;
    default: break;
    }
  }
};
Matrix local_matrix(const epok::Transform &transform) {
  ++performance_work.local_matrices;
  Matrix out;
  for (int c = 0; c < 3; ++c) {
    Fixed axis[3] = {};
    axis[c] = transform.scale[c];
    rotate(axis, transform.rotation);
    for (int r = 0; r < 3; ++r)
      out.values[r][c] = axis[r];
  }
  for (int r = 0; r < 3; ++r)
    out.values[r][3] = transform.position[r];
  return out;
}
Matrix inverse_local(const epok::Transform &transform) {
  Matrix out;
  for (int c = 0; c < 4; ++c) {
    Fixed p[3] = {};
    if (c < 3)
      p[c] = 1.0;
    else
      for (int r = 0; r < 3; ++r)
        p[r] = -transform.position[r];
    for (int axis = 2; axis >= 0; --axis) {
      Fixed inverse[3] = {};
      inverse[axis] = -transform.rotation[axis];
      rotate(p, inverse);
    }
    for (int r = 0; r < 3; ++r) {
      // Editor validates positive scale. Avoid division by zero if a script
      // sets zero.
      out.values[r][c] = transform.scale[r].raw() == 0
                             ? Fixed(0, Fixed::RAW)
                             : p[r] / transform.scale[r];
    }
  }
  return out;
}
epok::ActorData* camera_entity() {
  auto valid=[](epok::ActorData* e){return e&&epok::is_active(e)&&(e->camera||e->camera_settings.enabled);};
  if(auto* e=camera_override.get();valid(e))return e;
  for(size_t i=0;i<epok::object_count;++i)if(valid(&epok::objects[i]))return &epok::objects[i];
  return nullptr;
}
Matrix camera_view(bool interpolated=false) {
  epok::Transform fallback={{0.0,3.0,-6.0},{18.0,0.0,0.0},{1.0,1.0,1.0}};
  auto* camera=camera_entity();
  epok::projection_focal=1.0;
  if(!camera)return inverse_local(fallback);
  int32_t degrees=camera->camera_settings.field_of_view.raw();
  if(degrees<25*4096)degrees=25*4096;if(degrees>120*4096)degrees=120*4096;
  if(degrees!=90*4096){psyqo::Angle half(degrees/1440,psyqo::Angle::RAW);epok::projection_focal=trig.cos(half)/trig.sin(half);}
  auto view=Matrix::identity();
  int i=epok::entity_index(camera);unsigned depth=0;
  for(;i>=0&&size_t(i)<epok::object_count&&depth<33;++depth){
    const auto transform=interpolated?motion.local(size_t(i),epok::objects[i].transform):epok::objects[i].transform;
    const auto inverse=inverse_local(transform);
    view=depth?view.compose(inverse):inverse;
    i=epok::objects[i].parent;
  }
  return view;
}
void refresh_world() {
  ScanlineScope timer{performance_work.world_scanlines};
  ++performance_work.world_syncs;
  transform_cache.sync(epok::objects,world,epok::object_count,local_matrix);
  performance_work.world_matrices+=transform_cache.world_rebuilds;
}
void refresh_collisions() {
  ScanlineScope timer{performance_work.collision_scanlines};
  refresh_world();collision_world.begin_sync();
  for(size_t index=0;index<epok::object_count;++index) {
    const auto& object=epok::objects[index];
    if(object.collider.enabled&&epok::is_active_slot(index))
      performance_work.collider_bounds+=collision_world.set_cached(index,object.collider,world[index],transform_cache.revision(index),true,object.generation);
  }
}
void dispatch_trigger(const epok::TriggerEvent& event) {
  const epok::DataHandle first{event.first,event.first_generation},second{event.second,event.second_generation};
  if(first.get()&&epok::is_active(first.get()))epok::dispatch_slot_trigger(first,second,event.phase);
  if(second.get()&&epok::is_active(second.get()))epok::dispatch_slot_trigger(second,first,event.phase);
}
class GameScene final : public psyqo::Scene {
  void start(StartReason) override {
    // Audio quarantine and trigger-delivery predicates for the object model. Installed
    // before the first bank load, so no component-owned AudioSource can be recycled
    // while the XA consumer still points at it.
    epok::install_actor_service_hooks();
#if defined(EPOK_LUA_MODE) && EPOK_LUA_MODE != 0
    // The VM opens its arena, loads every class chunk and caches the method
    // functions before any script can be constructed.
    epok::lua::initialize();
#endif
    // GPU DMA completion needs interrupts, which Application::prepare disables.
    hud.initialize(gpu());
    epok::debug_hud::initialize(gpu());
      epok::input.reset();
#ifdef EPOK_PLAYBACK_WAITS
      epok::bp::playback_observer=&observe_blueprint_playback;
#endif
#ifdef EPOK_TRANSITIONS
    epok::transition.begin(epok::TransitionOptions{},gpu().now(),true);
    epok::initialize_scripts();
#else
    epok::initialize_scripts();
    if constexpr (epok::stream_page_count > 0)
      epok::streaming_warmup_scene(epok::objects.data(), epok::object_count,
                                  [](size_t i) { return epok::is_active_slot(i); }, gpu());
    // Initial CD loading completes before the simulation clock starts.
    epok::time.reset(gpu().now());
    for (auto &object : epok::objects)
      if (epok::is_active(&object) && object.audio.enabled && object.audio.play_on_start &&
          !object.audio.is_playing())
        object.audio.play();
#endif
    syscall_puts("EPOK: runtime ready\n");
  }
  void frame() override;
  psyqo::OrderingTable<buckets> ordering[2];
  epok::FrameClear<epok::display_width,epok::display_height> clear;
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTriangle>
      triangles[2][capacity];
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTexturedTriangle> textured_triangles[2][capacity];
};
class Game final : public psyqo::Application {
  void prepare() override {
    epok::serial_debug::restore();
    psyqo::GPU::Configuration config;
    config.set(epok::display_width == 256 ? psyqo::GPU::Resolution::W256 :
               epok::display_width == 320 ? psyqo::GPU::Resolution::W320 :
               epok::display_width == 368 ? psyqo::GPU::Resolution::W368 :
               epok::display_width == 512 ? psyqo::GPU::Resolution::W512 : psyqo::GPU::Resolution::W640)
        .set(psyqo::GPU::VideoMode::NTSC)
        .set(psyqo::GPU::ColorMode::C15BITS)
        .set(epok::display_interlaced ? psyqo::GPU::Interlace::INTERLACED : psyqo::GPU::Interlace::PROGRESSIVE);
    gpu().initialize(config);
    pad.initialize(psyqo::AdvancedPad::PollingMode::Fast);
    card_driver.prepare();
    // Expose all 240 lines / 480 interlaced rows after PsyQo initializes the range.
    psyqo::Hardware::GPU::Ctrl = 0x07000000 | 16 | (256 << 10);
    epok::audio_initialize(epok::audio_clips, epok::audio_clip_count);
#ifdef EPOK_HAS_SEQUENCES
    if(!epok::sequence_clock_start())psyqo::Kernel::abort("PSX sequence bank/clock initialization failed; inspect music_sequence_stats.error");
#endif
    epok::music_prepare();
    epok::streaming_prepare();
  }
  void createScene() override { pushScene(&scene); }
  GameScene scene;
};
Game game;

void GameScene::frame() {
  const uint16_t frame_begin = COUNTERS[1].value;
  epok::debug_hud::begin(gpu());
  performance_work={};
  performance_work.frame=epok::performance_stats.frame+1;
  const bool scene_changed=epok::scene_tick(gpu());
  if(scene_changed||epok::scene_loading())motion.clear();
  epok::input.poll(pad,4); // AdvancedPad's physical ports are Pad1a=0, Pad2a=4.
#ifdef EPOK_TRANSITIONS
  if(epok::stream_failed&&!epok::transition.loading())epok::transition.fail();
  if(epok::transition.phase==epok::TransitionPhase::Failed&&epok::input.frame_pressed(epok::Button::Start)){
    if(!epok::stream_read_pending&&!epok::stream_lookup_pending&&!epok::music_lookup&&epok::music_drive.isIdle()){
      epok::stream_failed=false;epok::stream_lookup_started=false;epok::stream_ready=false;
      if(epok::music_boot_failed){
        epok::music_boot_failed=false;epok::music_booting=false;epok::music_ready=false;
      }
#if EPOK_HOST_DATA
      if(epok::stream_host_file>=0)PCclose(epok::stream_host_file);
      epok::stream_host_file=-1;
#endif
      epok::transition.phase=epok::TransitionPhase::Loading;epok::transition.boot=true;epok::transition.presented=false;
    }
  }
  if(scene_changed||(epok::transition.boot&&epok::transition.presented)){
    bool ok=true;
    if constexpr(epok::stream_page_count>0)ok=epok::streaming_warmup_scene(epok::objects.data(),epok::object_count,[](size_t i){return epok::is_active_slot(i);},gpu());
    // An oversized working set uses ordinary demand streaming, not an error.
    if(!ok&&!epok::stream_failed)ok=true;
    if(ok){
      epok::time.synchronize(gpu().now());
      for(auto& object:epok::objects)if(epok::is_active(&object)&&object.audio.enabled&&object.audio.play_on_start&&!object.audio.is_playing())object.audio.play();
      epok::transition.loaded(gpu().now());
    }else epok::transition.fail();
  }
#else
  (void)scene_changed;
#endif
  const unsigned steps=epok::scene_loading()?(epok::time.synchronize(gpu().now()),0u):epok::time.advance(gpu().now());
  performance_work.steps=steps;
  if(!epok::scene_loading())epok::level.frame_update(epok::time.frame_microseconds);
  if(epok::time.paused()||epok::scene_loading())epok::input.discard_edges();
  motion.select(epok::objects,epok::object_count,[](size_t i){
    const auto& o=epok::objects[i];
    return (o.mesh||o.sprite.enabled||o.camera||o.camera_settings.enabled||o.light.enabled||o.blob_shadow.enabled)&&epok::is_active_slot(i);
  });
  bool advanced_motion=false;
  for(unsigned step=0;step<steps&&!epok::time.paused()&&!epok::scene_loading();++step) {
    motion.before_tick(epok::objects,epok::object_count);advanced_motion=true;
    epok::time.begin_tick();epok::input.begin_tick();
    const Fixed dt(epok::time.delta_raw,Fixed::RAW);
#ifdef EPOK_EFFECTS
    epok::effects::prepare();
#endif
#ifdef EPOK_TIMELINES
    epok::timeline::advance(dt);
#endif
#ifdef EPOK_EFFECTS
      epok::effects::after_timeline();
  #endif
  #ifdef EPOK_PLAYBACK_WAITS
      epok::bp::observe_playback();
  #endif
    epok::level.tick(dt);
    refresh_collisions();collision_world.update_triggers(dispatch_trigger);
    bool has_emitters=false;
    for(size_t i=0;i<epok::object_count;++i)if(epok::is_active_slot(i)) {
      has_emitters|=epok::objects[i].particle_emitter.enabled;
      epok::objects[i].animator.advance();
      epok::objects[i].sprite_animator.advance(dt,epok::objects[i].sprite);
      epok::objects[i].palette_animator.advance(dt);
    }
#ifdef EPOK_EFFECTS
    epok::effects::advance(dt);
    has_emitters|=epok::effect_stats.active!=0;
#endif
    if(has_emitters||epok::particle_stats.alive){
      refresh_world();particles.begin(dt);particles.scene_emitters(epok::objects,world,epok::object_count);
#ifdef EPOK_EFFECTS
      epok::effects::pool.emitters([&](epok::EffectLayerHandle owner,epok::ParticleEmitter& emitter,const Matrix& matrix){particles.emitter(owner,emitter,matrix);});
#endif
      particles.advance();
    }
    epok::input.end_tick();
  }
  epok::audio_tick();
  // Storage a service still points at was skipped by the last release pass; retry it
  // here, next to the point where the legacy path re-checks music_active.
  epok::collect_object_quarantine();
  epok::level.refresh_stats();
  if(advanced_motion)motion.after_tick(epok::objects,epok::object_count);
  if(epok::time.paused()||epok::scene_loading()||steps==epok::Time::max_steps)motion.clear();
  epok::streaming_tick();
  epok::streaming_service_gameplay_requests();
  epok::music_tick();
  if(epok::transition.loading()){
    clear.draw(gpu(),psyqo::Color{{.r=0,.g=0,.b=0}});
    loading_renderer.draw(gpu());
    epok::performance_stats=performance_work;
    epok::debug_hud::draw(gpu());
    return;
  }
  performance_work.simulation_scanlines=uint16_t(COUNTERS[1].value-frame_begin);
  const uint16_t prepare_begin=COUNTERS[1].value;

  refresh_world();

  const auto& render_world=motion.prepare(epok::objects,world,epok::object_count,epok::time.interpolation_raw());
  EPOK_DETAIL_BEGIN(camera);
  palettes.upload(gpu(),epok::objects,epok::object_count);
  const uint16_t lights_begin = COUNTERS[1].value;
  lighting.prepare(epok::objects, render_world, epok::object_count);
  epok::lighting_work.lighting_scanlines =
      uint16_t(COUNTERS[1].value - lights_begin);
  int parity = gpu().getParity();
  auto &table = ordering[parity];
  clear.draw(gpu(),psyqo::Color{{.r = 33, .g = 40, .b = 52}});
  Matrix view = camera_view(true);
  EPOK_DETAIL_END(camera, camera_scanlines);
  static constexpr int16_t vertices[8][3] = {
      {-2048, -2048, -2048}, {2048, -2048, -2048}, {2048, 2048, -2048},
      {-2048, 2048, -2048},  {-2048, -2048, 2048}, {2048, -2048, 2048},
      {2048, 2048, 2048},    {-2048, 2048, 2048}};
  static constexpr epok::MeshQuad faces[6] = {
      {{0, 1, 2, 3}, 0}, {{5, 4, 7, 6}, 1}, {{4, 0, 3, 7}, 2},
      {{1, 5, 6, 2}, 3}, {{3, 2, 6, 7}, 4}, {{4, 5, 1, 0}, 5}};
  static constexpr epok::MeshGeometry default_cube = {vertices, 8, faces, 6};
  ProjectedVertex projected_vertices_ram[512];
  const uint16_t render_begin=COUNTERS[1].value;
  performance_work.prepare_scanlines=uint16_t(render_begin-prepare_begin);
  epok::mesh_stats = {};
#ifdef EPOK_PROFILE_DETAIL
  if constexpr (epok::stream_page_count > 0) non_streamed_visible = 0;
#endif
  epok::retained_stats = {};
  // Fog thresholds resolved once per frame in camera units; an empty range
  // behaves as a step.
  const bool fog_enabled = epok::fog_environment.enabled;
  const int32_t fog_start = epok::fog_environment.start >> 4;
  const int32_t fog_end = (epok::fog_environment.end >> 4) > fog_start ? (epok::fog_environment.end >> 4) : fog_start + 1;
  const uint8_t* const fog_rgb = epok::fog_environment.color;
  // One pixel focal length for both axes: display_height*2/3 keeps the vertical
  // projection; the horizontal one folds the remaining aspect into matrix row 0
  // (a 1:1 ratio for square-pixel modes). Lighting never touches these registers.
  constexpr int32_t projection_focal_pixels = epok::display_height * 2 / 3;
  constexpr int32_t horizontal_pixels = epok::display_width / 2;
  const Fixed row_scale[2] = {
      horizontal_pixels == projection_focal_pixels ? epok::projection_focal : epok::projection_focal * Fixed(horizontal_pixels) / Fixed(projection_focal_pixels),
      epok::projection_focal};
  epok::load_projection_screen(projection_focal_pixels, epok::display_width / 2, epok::display_height / 2);
  PolygonEmitter emitter{table, triangles[parity], textured_triangles[parity]};
  epok::StreamPageCursor stream_cursor;
  for (size_t index = 0; index < epok::object_count; ++index) {
    const auto &object = epok::objects[index];
    if (!object.mesh || !epok::is_active_slot(index))
      continue;
    const bool use_bake =
        !object.material.unlit && object.lighting.enabled &&
        object.lighting.receive == epok::ReceiveLighting::Baked &&
        object.baked_colors;
    std::array<psyqo::Color, 6> face_colors{};
    bool shaded = false, local_normals = false;
    epok::lighting_detail::MeshNormalTransform normal_transform;
    const bool skeletal=object.animator.enabled && object.animator.model;
    const auto skeletal_storage=skeletal?object.animator.model->storage:epok::SkeletalStorage::CpuRigid;
    const bool rigid_gte=skeletal_storage==epok::SkeletalStorage::RigidGte;
    const bool baked_vertices=skeletal_storage==epok::SkeletalStorage::BakedVertices;
    const bool cpu_rigid=skeletal&&!rigid_gte&&!baked_vertices;
    auto object_view=view.compose(render_world[index]);
    for(int r=0;r<2;++r)for(int c=0;c<4;++c)object_view.values[r][c]*=row_scale[r];
    MaterialState material_state;
    const bool object_lit = !object.material.unlit && object.lighting.enabled;
    const epok::MeshGeometry* const geometry_root = skeletal ? object.animator.model->geometry : (object.geometry ? object.geometry : &default_cube);
    if constexpr (epok::stream_archive_fits) if (!skeletal &&
        !epok::streaming_bind_object(geometry_root, stream_bindings[index], gpu())) {
      // Binding is atomic at object granularity: never draw a partial chain.
      for (auto* mesh = geometry_root; mesh; mesh = mesh->next) {
        ++performance_work.stream_failed_chunks;
        epok::lighting_work.dropped_triangles += uint32_t(mesh->quad_count) * 2;
      }
      continue;
    }
    // Retained packets: static geometry whose packet words survive across frames.
    bool object_retained = false, retained_checked = false, object_any_dynamic = false, stream_metadata_validated = false;
    epok::RetainedQuad* object_records = nullptr;
    uint32_t object_first_slot = 0, quad_base = 0;
    if constexpr (epok::retained_geometry) {
      if (!skeletal) {
        const auto& state = retained.state(index);
        bool allocated = state.allocated && state.geometry == geometry_root;
        if (!allocated) {
          size_t total_quads = 0;
          for (auto* m = geometry_root; m; m = m->next) total_quads += m->quad_count;
          allocated = retained.allocate(index, geometry_root, total_quads, emitter.next_dynamic);
          // Whole-archive bindings already validate this entire chain before
          // entering the renderer. Only demand cursors consume this flag.
          if constexpr (demand_streaming) if (allocated) {
            bool valid = true;
            for (auto* m = geometry_root; m; m = m->next) valid &= epok::streaming_descriptor_valid(*m);
            // Generated topology is immutable; replacing geometry allocates a
            // new state and validates its archive ranges before trusting them.
            retained.state(index).stream_validated = valid;
          }
        }
        if (allocated) {
          object_retained = true;
          object_records = retained.quads(index);
          object_first_slot = retained.first_slot(index);
          stream_metadata_validated = retained.state(index).stream_validated;
        }
      }
    }
    uint32_t flat_key = 0xffffffffu, flat_color = 0, flat_final = 0;
    // Corner colours of one quad before fog: baked per vertex, flat (cached per
    // face colour) or lit through the GTE. Returns true when the four corners
    // share flat_final, the modulated cache value.
    auto quad_colors = [&](const epok::MeshGeometry& mesh, const epok::MeshQuad& quad, size_t& color_offset, bool textured, uint32_t* base) __attribute__((always_inline)) -> bool {
      const auto* indices = quad.indices;
      if (mesh.editable) color_offset = quad.color_offset;
      bool uniform = false;
      if (use_bake) {
        const uint32_t fallback = mesh.editable ? 0x00ffffffu : color_bits(face_colors[quad.face]);
        for (int v = 0; v < 4; ++v) {
          base[v] = fallback;
          if (color_offset + v < object.baked_color_count) {
            const auto *light = object.baked_colors[color_offset + v];
            uint32_t rgb[3];
            for (int c = 0; c < 3; ++c) {
              const uint32_t level = (mesh.editable && quad.material.unlit) ? 255 : light[c];
              uint32_t tint = object.material.color[c];
              if (mesh.editable) tint = epok::scale_channel(tint, quad.material.color[c]);
              rgb[c] = epok::scale_channel(level, tint);
            }
            base[v] = epok::pack_color(rgb[0], rgb[1], rgb[2]);
            ++epok::lighting_work.baked_vertices;
          }
        }
      } else if (mesh.editable) {
        uint32_t flat;
        if (!object_lit || quad.material.unlit) {
          const uint32_t key = epok::pack_color(quad.material.color[0], quad.material.color[1], quad.material.color[2]);
          if (key != flat_key) {
            flat_key = key;
            flat_color = epok::pack_color(epok::scale_channel(quad.material.color[0], object.material.color[0]),
                                           epok::scale_channel(quad.material.color[1], object.material.color[1]),
                                           epok::scale_channel(quad.material.color[2], object.material.color[2]));
            flat_final = textured ? epok::modulate_color(flat_color) : flat_color;
          }
          flat = flat_color;
          uniform = flat_final != 0;
        } else {
          const int16_t* normal = quad.normal;
          int16_t skeletal_normal[3];
          if (skeletal) {
            const auto* normal_vertices=epok::skeletal_detail::scratch.geometry.vertices;
            int32_t a[3],b[3];for(int c=0;c<3;++c){a[c]=int32_t(normal_vertices[indices[1]][c])-normal_vertices[indices[0]][c];b[c]=int32_t(normal_vertices[indices[2]][c])-normal_vertices[indices[0]][c];}
            epok::lighting_detail::Vector n;for(int c=0;c<3;++c)n.v[c]=int32_t((int64_t(a[(c+1)%3])*b[(c+2)%3]-int64_t(a[(c+2)%3])*b[(c+1)%3])/4096);
            n=epok::lighting_detail::normalize(n);for(int c=0;c<3;++c)skeletal_normal[c]=int16_t(n.v[c]);
            normal = skeletal_normal;
          }
          flat = local_normals ? color_bits(epok::lighting_detail::mesh_shade_local(normal, quad.material, object.material))
                               : color_bits(epok::lighting_detail::mesh_shade_normal(world[index], normal, quad.material, object.material, true, &normal_transform));
        }
        base[0] = base[1] = base[2] = base[3] = flat;
      } else {
        base[0] = base[1] = base[2] = base[3] = color_bits(face_colors[quad.face]);
      }
      color_offset += 4;
      return uniform;
    };
    // Streamed objects rebuild only the visible chunk, while its page is pinned.
    // Packet storage and keys contain no pointers into the evictable page pool.
    auto build_retained_chunk = [&](int build_parity, const epok::MeshGeometry& mesh,
                                    const epok::MeshQuad* mesh_quads, uint32_t global_quad) {
      auto& state = retained.state(index);
      ++epok::retained_stats.rebuilds;
      MaterialState build_material;
      auto* tex_frags = textured_triangles[build_parity]; auto* gou_frags = triangles[build_parity];
      size_t color_offset = 0;
      bool chunk_dynamic = false;
      const uint32_t chunk_first = global_quad;
      for (size_t q = 0; q < mesh.quad_count; ++q, ++global_quad) {
        const auto& quad = mesh_quads[q];
        auto& rec = object_records[global_quad];
        const epok::Material& draw_material = mesh.editable ? quad.material : object.material;
        build_material.update(draw_material);
        const bool textured = build_material.tex != nullptr;
        uint32_t base[4];
        quad_colors(mesh, quad, color_offset, textured, base);
        rec.textured = textured; rec.command = build_material.command; rec.depth_bias = draw_material.depth_bias;
        rec.dynamic = textured && (draw_material.uv_scroll[0] || draw_material.uv_scroll[1]);
        rec.fogged &= uint8_t(~(1u << build_parity));
        for (int v = 0; v < 4; ++v) rec.shaded[v] = base[v];
        if (rec.dynamic) { state.any_dynamic = true; chunk_dynamic = true; continue; }
        const uint32_t slot = object_first_slot + global_quad * 2;
        uint32_t final[4], uv[4] = {0, 0, 0, 0};
        for (int v = 0; v < 4; ++v) final[v] = textured ? epok::modulate_color(base[v]) : base[v];
        if (textured) {
          if (quad.packed_uv) { for (int v = 0; v < 4; ++v) uv[v] = quad.uvw[v]; }
          else { for (int v = 0; v < 4; ++v) uv[v] = build_material.uv(quad.uv[v][0], quad.uv[v][1]); }
        }
        static constexpr int corners[2][3] = {{0, 1, 2}, {0, 2, 3}};
        for (int t = 0; t < 2; ++t) {
          const int corner_a = corners[t][0], ib = corners[t][1], ic = corners[t][2];
          if (textured) {
            uint32_t* w = reinterpret_cast<uint32_t*>(&tex_frags[slot + t].primitive);
            w[0] = rec.command | final[corner_a]; w[2] = uv[corner_a] | build_material.clut16; w[3] = final[ib];
            w[5] = uv[ib] | build_material.tpage16; w[6] = final[ic]; w[8] = uv[ic];
          } else {
            uint32_t* w = reinterpret_cast<uint32_t*>(&gou_frags[slot + t].primitive);
            w[0] = 0x30000000u | final[corner_a]; w[2] = final[ib]; w[4] = final[ic];
          }
        }
      }
      if constexpr (demand_streaming) if (mesh.quad_count) {
        object_records[chunk_first].valid_parities |= uint8_t(1u << build_parity);
        object_records[chunk_first].chunk_dynamic = chunk_dynamic;
      }
    };
    // One GTE matrix per object: chunks add their origin to the vertices, so a
    // vertex shared by two chunks produces identical GTE inputs and no seam.
    const bool object_gte = !rigid_gte&&epok::load_projection_matrix(object_view);
    bool object_matrix_loaded = object_gte;
    // Raw view coefficients for the per-chunk bounds test: widening multiplies
    // and shifts only, no Fixed arithmetic or matrix copies per chunk.
    int32_t view_rows[3][3], view_abs[3][3], view_t[3];
    int32_t view_peak = 0;
    for (int r = 0; r < 3; ++r) {
      view_t[r] = object_view.values[r][3].raw();
      for (int c = 0; c < 3; ++c) {
        const int32_t v = object_view.values[r][c].raw();
        view_rows[r][c] = v; view_abs[r][c] = v < 0 ? -v : v;
        if (view_abs[r][c] > view_peak) view_peak = view_abs[r][c];
      }
    }
    // Products stay below 2^31 when coefficients are under 2^14 and chunk
    // vectors under 2^17 (32 units), which covers ordinary scenes.
    const bool narrow_view = view_peak < 16384;
    const bool isolated_x = epok::chunk_bounds_isolated_x(view_rows);
    epok::ChunkVisibilityQuery visibility(epok::precomputed_visibility && !skeletal ? geometry_root->visibility : nullptr);
    if constexpr (epok::precomputed_visibility) if (!visibility.reuse_matrix(view_rows, view_t, true)) {
      const auto add_plane = [&](int a, int sign, int z_scale, int a_scale, int32_t offset) {
        visibility.add_plane(int64_t(view_rows[2][0])*z_scale + int64_t(view_rows[a][0])*sign*a_scale,
                             int64_t(view_rows[2][1])*z_scale + int64_t(view_rows[a][1])*sign*a_scale,
                             int64_t(view_rows[2][2])*z_scale + int64_t(view_rows[a][2])*sign*a_scale,
                             int64_t(view_t[2])*z_scale + int64_t(view_t[a])*sign*a_scale + offset);
      };
      // Conservative planes of the actual chunk. Keep near at
      // zero and omit far; exact near/far and hardware clipping still run below.
      add_plane(0, 0, 1, 0, 0);
      add_plane(0, 1, 1, 1, 0); add_plane(0, -1, 1, 1, 0);
      add_plane(1, 1, 3, 4, 0); add_plane(1, -1, 3, 4, 0);
    }
    const auto visibility_mask = visibility.merged();
    const bool bounds_cache_active = visibility.bounds_cache_active();
    const bool basis_cache_active = visibility.basis_cache_active();
    size_t chunk_ordinal = 0;
    uint32_t prefetch_source = 0xffffffffu;
    bool skeletal_pose_ready=false,baked_vertices_ready=false;
    for (auto *mesh_ptr = geometry_root;
         mesh_ptr; mesh_ptr = mesh_ptr->next, ++chunk_ordinal) {
      const auto &bounds_mesh = *mesh_ptr;
      const uint32_t chunk_quad_base = quad_base;
      quad_base += uint32_t(bounds_mesh.quad_count);
      if (!visibility_mask.candidate(chunk_ordinal)) { ++performance_work.visibility_skipped_chunks; continue; }
      if (bounds_mesh.vertex_count == 0 || bounds_mesh.vertex_count > 512)
        continue;
      // Culling uses the permanent descriptor and never needs a CD read.
      EPOK_DETAIL_BEGIN(setup);
      ++epok::mesh_stats.tested_chunks;
      const int cached_bounds = bounds_cache_active ? visibility.cached_bounds(chunk_ordinal) : -1;
      bool fully_inside_chunk = false;
      if (cached_bounds == 0) {
        EPOK_DETAIL_END(setup, setup_scanlines);
        continue;
      }
      if (cached_bounds < 0) {
        // Conservative camera-space box of the chunk from the object matrix.
        int32_t center[3], extent[3];
        const auto* basis_bounds = basis_cache_active ? visibility.cached_basis_bounds(chunk_ordinal) : nullptr;
        if (basis_bounds) {
          for (int r = 0; r < 3; ++r) {
            center[r] = view_t[r] + basis_bounds->center[r];
            extent[r] = basis_bounds->extent[r];
          }
        } else {
        const int32_t chunk_vector[3] = {bounds_mesh.origin[0] + bounds_mesh.center[0], bounds_mesh.origin[1] + bounds_mesh.center[1], bounds_mesh.origin[2] + bounds_mesh.center[2]};
        const bool narrow_chunk = narrow_view &&
            chunk_vector[0] < 131072 && chunk_vector[0] > -131072 && chunk_vector[1] < 131072 && chunk_vector[1] > -131072 &&
            chunk_vector[2] < 131072 && chunk_vector[2] > -131072 && bounds_mesh.extent[0] < 131072 && bounds_mesh.extent[1] < 131072 && bounds_mesh.extent[2] < 131072;
        if (narrow_chunk) {
          if constexpr (!epok::precomputed_visibility) {
            epok::narrow_chunk_bounds(view_rows, view_abs, view_t, chunk_vector,
                                       bounds_mesh.extent, center, extent, isolated_x);
          } else if (basis_cache_active) {
          constexpr int32_t zero_translation[3] = {};
          epok::narrow_chunk_bounds(view_rows, view_abs, zero_translation, chunk_vector,
                                     bounds_mesh.extent, center, extent, isolated_x);
          visibility.remember_basis_bounds(chunk_ordinal, center, extent);
          for (int r = 0; r < 3; ++r) center[r] += view_t[r];
          } else {
            epok::narrow_chunk_bounds(view_rows, view_abs, view_t, chunk_vector,
                                       bounds_mesh.extent, center, extent, isolated_x);
          }
        } else {
          for (int r = 0; r < 3; ++r) {
            int64_t sum = 0, span = 0;
            for (int c = 0; c < 3; ++c) {
              sum += int64_t(view_rows[r][c]) * chunk_vector[c];
              span += int64_t(view_abs[r][c]) * bounds_mesh.extent[c];
            }
            center[r] = view_t[r] + int32_t(sum >> 12);
            extent[r] = int32_t(span >> 12) + 1;
          }
        }
        }
        const int far_z = center[2] + extent[2];
        if (far_z < 1024 || center[2] - extent[2] >= 128 * 4096 ||
            center[0] - extent[0] > far_z || center[0] + extent[0] < -far_z ||
            (center[1] - extent[1]) * 4 > far_z * 3 ||
            (center[1] + extent[1]) * 4 < -far_z * 3) {
          if (bounds_cache_active) visibility.remember_bounds(chunk_ordinal, false);
          EPOK_DETAIL_END(setup, setup_scanlines);
          continue;
        }
        if (bounds_cache_active) visibility.remember_bounds(chunk_ordinal, true);
        fully_inside_chunk = narrow_view && epok::chunk_fully_inside(center, extent, projection_focal_pixels);
      }
      ++epok::mesh_stats.visible_chunks;
      // This chunk consumes its raw view before the cursor can advance. The
      // cursor keeps the page pinned across consecutive chunks of that page.
      epok::StreamMeshView mesh_view;
      if constexpr (!demand_streaming) mesh_view = {bounds_mesh.vertices, bounds_mesh.quads, true};
      else mesh_view = stream_cursor.resolve(bounds_mesh, gpu(), stream_metadata_validated);
      if (!mesh_view.valid) {
#ifdef EPOK_PROFILE_DETAIL
        if constexpr (epok::stream_page_count > 0) ++non_streamed_visible;
#endif
        ++performance_work.stream_failed_chunks;
        epok::lighting_work.dropped_triangles += uint32_t(bounds_mesh.quad_count) * 2;
        EPOK_DETAIL_END(setup, setup_scanlines);
        continue;
      }
      const auto& mesh = bounds_mesh;
      if(baked_vertices&&!baked_vertices_ready){
        // decode_vertices() returns the bind positions untouched when the animator
        // has no valid clip or the clip carries no encoded vertex frames (see
        // runtime/skeletal.hpp). Mirror that condition here so the counter reports
        // coordinates that were actually decoded from a clip frame, not the
        // per-vertex cost of a model that is only showing its bind pose.
        const auto& animator=object.animator;const auto& model=*animator.model;
        const epok::AnimationClip* const decoded_clip=
            animator.clip>=0&&size_t(animator.clip)<model.clip_count?&model.clips[animator.clip]:nullptr;
        const bool frame_decoded=decoded_clip&&decoded_clip->vertex_frames&&decoded_clip->vertex_data;
        const uint16_t begin=COUNTERS[1].value;
        epok::skeletal_detail::scratch.decode_vertices(model,animator);
        performance_work.skeletal_scanlines+=uint16_t(COUNTERS[1].value-begin);
        if(frame_decoded)performance_work.skeletal_decoded_vertices+=mesh.vertex_count;
        baked_vertices_ready=true;
      }
      if(cpu_rigid&&!skeletal_pose_ready){const uint16_t begin=COUNTERS[1].value;epok::skeletal_detail::scratch.pose(*object.animator.model,object.animator);performance_work.skeletal_scanlines+=uint16_t(COUNTERS[1].value-begin);performance_work.skeletal_bone_matrices+=object.animator.model->bone_count;performance_work.skeletal_cpu_vertices+=mesh.vertex_count;skeletal_pose_ready=true;}
      const auto* mesh_vertices = (baked_vertices||cpu_rigid)?epok::skeletal_detail::scratch.geometry.vertices:mesh_view.vertices;
      const auto* mesh_quads = mesh_view.quads;
      if constexpr (epok::stream_page_count > 0) {
#ifdef EPOK_PROFILE_DETAIL
        if (bounds_mesh.stream_page == 0xffffffffu) ++non_streamed_visible;
#endif
        // Whole-archive pools warm the active scene and rebuild retained
        // objects eagerly. Lookahead is useful only on the eviction path.
        if constexpr (epok::streaming_prefetch_enabled && demand_streaming)
          if (bounds_mesh.stream_page != 0xffffffffu && epok::streaming_prefetch_needed()) {
          // Spatially adjacent chunks are exported together. Speculation uses
          // the next candidate page, only free slots, and never interrupts XA.
          if (prefetch_source != bounds_mesh.stream_page) {
            prefetch_source = bounds_mesh.stream_page;
            size_t next_ordinal = chunk_ordinal + 1;
            for (auto* next = mesh_ptr->next; next; next = next->next, ++next_ordinal) {
              if (next->stream_page != bounds_mesh.stream_page && next->stream_page != 0xffffffffu && visibility_mask.candidate(next_ordinal)) {
                epok::streaming_prefetch(next->stream_page);
                break;
              }
            }
          }
        }
      }
      if (!use_bake && !shaded) {
        const uint16_t begin = COUNTERS[1].value;
        face_colors =
            lighting.shade(index, epok::objects, render_world, mesh.editable);
        shaded = true;
        local_normals = mesh.editable && object_lit && lighting.localize(world[index]);
        epok::lighting_work.lighting_scanlines +=
            uint16_t(COUNTERS[1].value - begin);
      }
      if (object_retained && !retained_checked) {
        retained_checked = true;
        epok::RetainedKey key;
        key.valid = true; key.geometry = geometry_root; key.bank = epok::texture_assets;
        key.baked = object.baked_colors; key.baked_count = object.baked_color_count;
        key.generation = object.generation;
        key.light_signature = (object_lit && !use_bake) ? lighting.signature : 0;
        for (int r = 0; r < 3; ++r) for (int c = 0; c < 3; ++c) key.basis[r * 3 + c] = world[index].values[r][c].raw();
        for (int c = 0; c < 3; ++c) key.color[c] = object.material.color[c];
        key.unlit = object.material.unlit; key.texture = object.material.texture; key.blend = object.material.blend;
        key.depth_bias = object.material.depth_bias; key.uv_scroll[0] = object.material.uv_scroll[0]; key.uv_scroll[1] = object.material.uv_scroll[1];
        key.lighting_enabled = object.lighting.enabled; key.receive = object.lighting.receive;
        auto& state = retained.state(index);
        if (!(state.key[parity] == key)) {
          // Quad colours/flags are shared between parities, unlike GPU packet
          // storage. A different material must invalidate the other key too.
          if (state.key[1 - parity].valid && !(state.key[1 - parity] == key))
            state.key[1 - parity].valid = false;
          state.key[parity] = key; state.any_dynamic = false;
          if constexpr (demand_streaming) {
            // Invalidate packet contents without acquiring invisible geometry.
            const uint8_t keep = uint8_t(~(1u << parity));
            for (uint32_t q = 0; q < state.quad_count; ++q) object_records[q].valid_parities &= keep;
          } else {
            uint32_t first = 0;
            for (auto* m = geometry_root; m; m = m->next) {
              build_retained_chunk(parity, *m, m->quads, first);
              first += uint32_t(m->quad_count);
            }
          }
        }
        object_any_dynamic = retained.state(index).any_dynamic;
        ++epok::retained_stats.objects;
      }
      if constexpr (demand_streaming) if (object_retained && mesh.quad_count) {
        if (!(object_records[chunk_quad_base].valid_parities & (1u << parity)))
          build_retained_chunk(parity, mesh, mesh_quads, chunk_quad_base);
        object_any_dynamic = object_records[chunk_quad_base].chunk_dynamic;
      }
      EPOK_DETAIL_END(setup, setup_scanlines);
      const uint16_t vertex_begin=COUNTERS[1].value;
      ProjectedVertex* const projected_vertices =
          mesh.vertex_count <= scratch_vertex_capacity ? scratch_vertices : projected_vertices_ram;
      auto finish_projection=[&](ProjectedVertex& out,uint32_t flags){
        const int32_t z = out.camera[2];
        out.outcode = epok::frustum_outcode32<Units::near, Units::far>(out.camera[0], out.camera[1], z);
        out.fog = fog_enabled ? epok::fog_amount(z, fog_start, fog_end) : 0;
        out.visible = z >= Units::near && z < Units::far;
        if (out.visible && (flags & epok::gte_projection_flags))
          epok::project_cpu<Units, epok::display_width, epok::display_height>(out);
      };
      bool gte_geometry=true;
      if(rigid_gte){
        if(!skeletal_pose_ready){const uint16_t begin=COUNTERS[1].value;epok::skeletal_detail::scratch.pose_bones(*object.animator.model,object.animator);performance_work.skeletal_scanlines+=uint16_t(COUNTERS[1].value-begin);performance_work.skeletal_bone_matrices+=object.animator.model->bone_count;skeletal_pose_ready=true;}
        const int32_t origin8[3]={0,0,0};
        size_t gte_count=0,software_count=0;
        for(size_t bone=0;bone<object.animator.model->bone_count;++bone){
          const size_t first=object.animator.model->bone_vertices[bone];
          const size_t last=object.animator.model->bone_vertices[bone+1];
          if(first==last)continue;
          Matrix bone_view=object_view.compose(epok::skeletal_detail::scratch.bones[bone]);
          const bool bone_gte=epok::load_projection_matrix(bone_view);
          for(size_t v=first;v<last;++v){
            auto& out=projected_vertices[v];uint32_t flags=epok::gte_projection_flags;
            if(bone_gte)epok::project_geometry_vertex(mesh_vertices[v],origin8,out.camera,out.screen.packed,flags);
            else{Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);bone_view.point(local,p);for(int c=0;c<3;++c)out.camera[c]=p[c].raw()>>4;}
#ifdef EPOK_VALIDATE_GTE
            if(bone_gte){
              Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);bone_view.point(local,p);
              for(int c=0;c<3;++c){const int64_t difference=int64_t(out.camera[c])*16-p[c].raw();const uint32_t delta=uint32_t(difference<0?-difference:difference);if(delta>performance_work.gte_max_delta)performance_work.gte_max_delta=delta;if(delta>128)++performance_work.gte_validation_errors;}
            }
#endif
            finish_projection(out,flags);
          }
          if(bone_gte)gte_count+=last-first;else software_count+=last-first;
        }
        performance_work.gte_vertices+=gte_count;performance_work.software_vertices+=software_count;
        gte_geometry=software_count==0;object_matrix_loaded=false;
      }else{
        int32_t origin8[3] = {mesh.origin[0] >> 4, mesh.origin[1] >> 4, mesh.origin[2] >> 4};
        bool shared_matrix = object_gte;
        for (int c = 0; c < 3; ++c)
          if (origin8[c] > epok::gte_shared_origin_limit || origin8[c] < -epok::gte_shared_origin_limit) shared_matrix = false;
        Matrix model_view;
#ifdef EPOK_VALIDATE_GTE
        const bool need_chunk_matrix = true;
#else
        const bool need_chunk_matrix = !shared_matrix;
#endif
        if (need_chunk_matrix) {
          model_view = object_view;
          for (int r = 0; r < 3; ++r)
            for (int c = 0; c < 3; ++c)
              model_view.values[r][3] += model_view.values[r][c] * Fixed(mesh.origin[c], Fixed::RAW);
        }
        if (shared_matrix) {
          if (!object_matrix_loaded) object_matrix_loaded = epok::load_projection_matrix(object_view);
          gte_geometry = true;
        } else {
          origin8[0] = origin8[1] = origin8[2] = 0;
          gte_geometry = epok::load_projection_matrix(model_view);
          object_matrix_loaded = false;
        }
        size_t v = 0;
        if (gte_geometry && fully_inside_chunk && object_retained && !object_any_dynamic) {
          for (; v + 2 < mesh.vertex_count; v += 3) {
            uint32_t screens[3];int32_t depths[3];
            if (epok::project_geometry_triple(mesh_vertices+v,origin8,screens,depths)) {
              for (unsigned i=0;i<3;++i) {
                auto& out=projected_vertices[v+i];
                out.camera[0]=out.camera[1]=0;out.camera[2]=depths[i];out.screen.packed=screens[i];
                out.outcode=0;out.visible=true;out.fog=fog_enabled?epok::fog_amount(depths[i],fog_start,fog_end):0;
#ifdef EPOK_VALIDATE_GTE
                ProjectedVertex reference{};uint32_t flags;
                epok::project_geometry_vertex(mesh_vertices[v+i],origin8,reference.camera,reference.screen.packed,flags);
                finish_projection(reference,flags);
                if(reference.screen.packed!=out.screen.packed || reference.camera[2]!=out.camera[2] ||
                   reference.outcode || !reference.visible || reference.fog!=out.fog)
                  ++performance_work.gte_validation_errors;
#endif
              }
            } else {
              for (unsigned i=0;i<3;++i) {
                auto& out=projected_vertices[v+i];uint32_t flags;
                epok::project_geometry_vertex(mesh_vertices[v+i],origin8,out.camera,out.screen.packed,flags);
                finish_projection(out,flags);
              }
            }
          }
        }
        for (; v < mesh.vertex_count; ++v) {
          auto &out = projected_vertices[v];
          uint32_t flags = epok::gte_projection_flags;
          if (gte_geometry) {
            epok::project_geometry_vertex(mesh_vertices[v], origin8, out.camera, out.screen.packed, flags);
          } else {
            Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);
            model_view.point(local,p);
            for(int c=0;c<3;++c)out.camera[c]=p[c].raw()>>4;
          }
#ifdef EPOK_VALIDATE_GTE
          if (gte_geometry) {
          Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);
          model_view.point(local,p);
          for(int c=0;c<3;++c){
            // Q8 output against the exact Q12 transform: the dropped vertex
            // fraction bounds the difference to |R| * 15 / 4096 per axis.
            const int64_t difference=int64_t(out.camera[c])*16-p[c].raw();
            const uint32_t delta=uint32_t(difference<0?-difference:difference);
            if(delta>performance_work.gte_max_delta)performance_work.gte_max_delta=delta;
            if(delta>128)++performance_work.gte_validation_errors;
          }
          if (!(flags & epok::gte_projection_flags) && out.camera[2] >= Units::near && out.camera[2] < Units::far) {
            // The GTE's Newton-Raphson reciprocal carries about 16 bits, so its
            // screen coordinates stay within two pixels of the exact floored
            // projection computed here in 64-bit arithmetic.
            epok::ProjectedVertex reference = out;
            epok::project_cpu<Units, epok::display_width, epok::display_height>(reference);
            const int dx = reference.screen.x - out.screen.x, dy = reference.screen.y - out.screen.y;
            const uint32_t delta = uint32_t((dx < 0 ? -dx : dx) > (dy < 0 ? -dy : dy) ? (dx < 0 ? -dx : dx) : (dy < 0 ? -dy : dy));
            if (delta > performance_work.gte_screen_max_delta) performance_work.gte_screen_max_delta = delta;
            if (delta > 2 || !reference.visible) ++performance_work.gte_validation_errors;
          }
          }
#endif
          finish_projection(out,flags);
        }
        if(gte_geometry)performance_work.gte_vertices+=mesh.vertex_count;else performance_work.software_vertices+=mesh.vertex_count;
      }
      epok::mesh_stats.transformed_vertices += mesh.vertex_count;
      performance_work.vertex_scanlines+=uint16_t(COUNTERS[1].value-vertex_begin);
      epok::debug_hud::geometry(vertex_begin, gte_geometry);
      const uint16_t polygon_begin=COUNTERS[1].value;
      emitter.editable = mesh.editable;
      if (object_retained) {
        epok::RetainedQuad* records = object_records + chunk_quad_base;
        const uint32_t slot_base = object_first_slot + chunk_quad_base * 2;
        const uint8_t parity_bit = uint8_t(1u << parity);
        for (size_t q = 0; q < mesh.quad_count; ++q) {
          auto& rec = records[q];
          if (rec.dynamic) continue;
          const auto& quad = mesh_quads[q];
          const auto* indices = quad.indices;
          const ProjectedVertex& p0 = projected_vertices[indices[0]];
          const ProjectedVertex& p1 = projected_vertices[indices[1]];
          const ProjectedVertex& p2 = projected_vertices[indices[2]];
          const ProjectedVertex& p3 = projected_vertices[indices[3]];
          if (p0.outcode & p1.outcode & p2.outcode & p3.outcode) continue;
          const int first = (indices[0] == indices[1] || indices[1] == indices[2] || indices[0] == indices[2]) ? 0 : emitter.classify(p0, p1, p2);
          const int second = (indices[0] == indices[2] || indices[2] == indices[3] || indices[0] == indices[3]) ? 0 : emitter.classify(p0, p2, p3);
          if (!first && !second) continue;
          EPOK_DETAIL_BEGIN(fog);
          // Colour words are rewritten while fog touches the quad and once more
          // when it stops, so the packet returns to its retained colours.
          uint32_t colors[4], shaded_now[4];
          const uint32_t* recolor = nullptr;
          const uint32_t* shaded = rec.shaded;
          if (fog_enabled && (p0.fog | p1.fog | p2.fog | p3.fog)) {
            shaded_now[0] = epok::blend_fog(rec.shaded[0], p0.fog, fog_rgb); shaded_now[1] = epok::blend_fog(rec.shaded[1], p1.fog, fog_rgb);
            shaded_now[2] = epok::blend_fog(rec.shaded[2], p2.fog, fog_rgb); shaded_now[3] = epok::blend_fog(rec.shaded[3], p3.fog, fog_rgb);
            shaded = shaded_now;
            for (int v = 0; v < 4; ++v) colors[v] = rec.textured ? epok::modulate_color(shaded_now[v]) : shaded_now[v];
            recolor = colors; rec.fogged |= parity_bit;
          } else if (rec.fogged & parity_bit) {
            for (int v = 0; v < 4; ++v) colors[v] = rec.textured ? epok::modulate_color(rec.shaded[v]) : rec.shaded[v];
            recolor = colors; rec.fogged &= uint8_t(~parity_bit);
          }
          EPOK_DETAIL_END(fog, fog_scanlines);
          EPOK_DETAIL_BEGIN(emit);
          const uint32_t slot = slot_base + uint32_t(q) * 2;
          if (first == 2 || second == 2) {
            // Software clipping needs a packet; rebuilt here because it is rare.
            const epok::Material& draw_material = mesh.editable ? quad.material : object.material;
            material_state.update(draw_material);
            QuadPacket packet;
            packet.textured = rec.textured; packet.command = rec.command; packet.clut16 = material_state.clut16; packet.tpage16 = material_state.tpage16;
            packet.depth_bias = rec.depth_bias;
            uint32_t uv[4] = {0, 0, 0, 0};
            if (rec.textured) {
              if (quad.packed_uv) { for (int v = 0; v < 4; ++v) uv[v] = quad.uvw[v]; }
              else { for (int v = 0; v < 4; ++v) uv[v] = material_state.uv(quad.uv[v][0], quad.uv[v][1]); }
            }
            if (first == 1) emitter.retained_emit(slot, p0, p1, p2, rec, recolor, 0, 1, 2);
            else if (first == 2) emitter.clipped(p0, p1, p2, shaded[0], shaded[1], shaded[2], uv[0], uv[1], uv[2], packet);
            if (second == 1) emitter.retained_emit(slot + 1, p0, p2, p3, rec, recolor, 0, 2, 3);
            else if (second == 2) emitter.clipped(p0, p2, p3, shaded[0], shaded[2], shaded[3], uv[0], uv[2], uv[3], packet);
            EPOK_DETAIL_END(emit, emit_scanlines);
            continue;
          }
          if (first) emitter.retained_emit(slot, p0, p1, p2, rec, recolor, 0, 1, 2);
          if (second) emitter.retained_emit(slot + 1, p0, p2, p3, rec, recolor, 0, 2, 3);
          EPOK_DETAIL_END(emit, emit_scanlines);
        }
      }
      if (!object_retained || object_any_dynamic) {
      size_t color_offset = 0;
      QuadPacket packet;
      for (size_t q = 0; q < mesh.quad_count; ++q) {
        const auto& quad = mesh_quads[q];
        // Retained objects only reach this loop for their scrolling quads.
        if (object_retained && !object_records[chunk_quad_base + q].dynamic) { color_offset += 4; continue; }
        const auto* indices = quad.indices;
        const ProjectedVertex& p0 = projected_vertices[indices[0]];
        const ProjectedVertex& p1 = projected_vertices[indices[1]];
        const ProjectedVertex& p2 = projected_vertices[indices[2]];
        const ProjectedVertex& p3 = projected_vertices[indices[3]];
        if (p0.outcode & p1.outcode & p2.outcode & p3.outcode) { color_offset += 4; continue; }
        const int first = (indices[0] == indices[1] || indices[1] == indices[2] || indices[0] == indices[2]) ? 0 : emitter.classify(p0, p1, p2);
        const int second = (indices[0] == indices[2] || indices[2] == indices[3] || indices[0] == indices[3]) ? 0 : emitter.classify(p0, p2, p3);
        if (!first && !second) { color_offset += 4; continue; }
        EPOK_DETAIL_BEGIN(shade);
        const epok::Material& draw_material = mesh.editable ? quad.material : object.material;
        material_state.update(draw_material);
        packet.textured = material_state.tex != nullptr;
        packet.command = material_state.command; packet.clut16 = material_state.clut16; packet.tpage16 = material_state.tpage16;
        packet.depth_bias = draw_material.depth_bias;
        uint32_t base[4];
        // Final packet colour shared by the four corners when nothing varies.
        const bool uniform = quad_colors(mesh, quad, color_offset, packet.textured, base);
        const uint32_t packet_flat = flat_final;
        EPOK_DETAIL_END(shade, shade_scanlines);
        EPOK_DETAIL_BEGIN(fog);
        if (fog_enabled && (p0.fog | p1.fog | p2.fog | p3.fog)) {
          base[0] = epok::blend_fog(base[0], p0.fog, fog_rgb); base[1] = epok::blend_fog(base[1], p1.fog, fog_rgb);
          base[2] = epok::blend_fog(base[2], p2.fog, fog_rgb); base[3] = epok::blend_fog(base[3], p3.fog, fog_rgb);
        }
        EPOK_DETAIL_END(fog, fog_scanlines);
        EPOK_DETAIL_BEGIN(emit);
        for (int v = 0; v < 4; ++v) packet.shaded[v] = base[v];
        if (packet.textured) {
          if (uniform) {
            packet.final_color[0] = packet.final_color[1] = packet.final_color[2] = packet.final_color[3] = packet_flat;
          } else {
            for (int v = 0; v < 4; ++v) packet.final_color[v] = epok::modulate_color(base[v]);
          }
          if (quad.packed_uv) {
            for (int v = 0; v < 4; ++v) packet.uv[v] = quad.uvw[v];
          } else {
            for (int v = 0; v < 4; ++v) packet.uv[v] = material_state.uv(quad.uv[v][0], quad.uv[v][1]);
          }
          if (draw_material.uv_scroll[0] || draw_material.uv_scroll[1]) {
            static constexpr int triples[2][3]={{0,1,2},{0,2,3}};
            const ProjectedVertex* corners[4] = {&p0, &p1, &p2, &p3};
            for (int t = 0; t < 2; ++t) {
              if (!(t == 0 ? first : second)) continue;
              epok::UvVertex input[3];
              for(int i=0;i<3;++i){const int v=triples[t][i];const auto& p=*corners[v];for(int c=0;c<3;++c)input[i].camera[c]=p.camera[c];input[i].color[0]=base[v]&255;input[i].color[1]=(base[v]>>8)&255;input[i].color[2]=(base[v]>>16)&255;for(int c=0;c<2;++c)input[i].uv[c]=quad.uv[v][c];}
              epok::scroll_triangle(input,draw_material.uv_scroll,epok::time.ticks,[&](const epok::UvVertex& a,const epok::UvVertex& b,const epok::UvVertex& c){
                emitter.unprojected(a, b, c, material_state, packet);
              });
            }
            EPOK_DETAIL_END(emit, emit_scanlines);
            continue;
          }
        } else {
          for (int v = 0; v < 4; ++v) { packet.final_color[v] = base[v]; packet.uv[v] = 0; }
        }
        if (first == 1) emitter.direct(p0, p1, p2, packet.final_color[0], packet.final_color[1], packet.final_color[2], packet.uv[0], packet.uv[1], packet.uv[2], packet);
        else if (first == 2) emitter.clipped(p0, p1, p2, packet.shaded[0], packet.shaded[1], packet.shaded[2], packet.uv[0], packet.uv[1], packet.uv[2], packet);
        if (second == 1) emitter.direct(p0, p2, p3, packet.final_color[0], packet.final_color[2], packet.final_color[3], packet.uv[0], packet.uv[2], packet.uv[3], packet);
        else if (second == 2) emitter.clipped(p0, p2, p3, packet.shaded[0], packet.shaded[2], packet.shaded[3], packet.uv[0], packet.uv[2], packet.uv[3], packet);
        EPOK_DETAIL_END(emit, emit_scanlines);
      }
      }
      performance_work.polygon_scanlines+=uint16_t(COUNTERS[1].value-polygon_begin);
    }
  }
  const size_t used = emitter.emitted;
  performance_work.render_scanlines=uint16_t(COUNTERS[1].value-render_begin);
  const uint16_t finish_begin=COUNTERS[1].value;
  epok::lighting_work.triangles = used;
  performance_work.tested_chunks = epok::mesh_stats.tested_chunks;
  performance_work.visible_chunks = epok::mesh_stats.visible_chunks;
#ifdef EPOK_PROFILE_DETAIL
  if constexpr (epok::stream_page_count > 0)
    performance_work.streamed_chunks = epok::mesh_stats.visible_chunks - non_streamed_visible;
#else
  // Descriptive per-chunk diagnostics are not collected in production builds.
  // Preserve the public layout while distinguishing unavailable from zero.
  performance_work.streamed_chunks = UINT32_MAX;
#endif
  performance_work.backfaces = epok::mesh_stats.backfaces;
  performance_work.clipped = epok::mesh_stats.clipped;
  performance_work.triangles = used;
  performance_work.retained_triangles = emitter.retained_emitted;
  performance_work.retained_rebuilds = epok::retained_stats.rebuilds;
  EPOK_DETAIL_BEGIN(sprite);
  stream_cursor.release();
  blobs.draw(parity, table, epok::objects, render_world, epok::object_count, view);
  sprites.begin();
  for(size_t i=0;i<epok::object_count;++i)if(epok::objects[i].sprite.enabled&&epok::is_active_slot(i)) {
    const auto& object=epok::objects[i];
    if(!object.sprite.unlit&&object.lighting.enabled&&!object.material.unlit)lighting.shade(i,epok::objects,render_world,true);
    sprites.draw(parity,table,object.sprite,render_world[i],view,object.lighting.enabled&&!object.material.unlit);
  }
#ifdef EPOK_EFFECTS
  auto draw_effect=[&](epok::EffectLayerHandle layer,const epok::Sprite& sprite,const Matrix& matrix){
    const auto owner=epok::effects::pool.lighting_owner(layer);
    const bool lit=owner.get()&&owner.get()->lighting.enabled&&!owner.get()->material.unlit;
    if(lit&&!sprite.unlit)lighting.shade(owner.index,epok::objects,render_world,true);
    sprites.draw(parity,table,sprite,matrix,view,lit);
  };
  epok::effects::pool.sprites(draw_effect);
#endif
  particles.each_all([&](epok::timeline::BoundTarget owner,const epok::Sprite& source,const Matrix& matrix) {
    if(owner.internal){
#ifdef EPOK_EFFECTS
      if(const auto* layer=owner.effect_layer()){auto sprite=source;epok::effects::Pool::tint(*layer,sprite);draw_effect(owner.layer,sprite,matrix);}
#endif
      return;
    }
    const size_t i=owner.data_slot().index;const auto& sprite=source;
    if(!epok::is_active_slot(i))return;
    if(!sprite.unlit&&epok::objects[i].lighting.enabled&&!epok::objects[i].material.unlit)lighting.shade(i,epok::objects,render_world,true);
    sprites.draw(parity,table,sprite,matrix,view,epok::objects[i].lighting.enabled&&!epok::objects[i].material.unlit);
  });
  gpu().chain(table);
  EPOK_DETAIL_END(sprite, sprite_scanlines);
  EPOK_DETAIL_BEGIN(hud);
  hud.draw(gpu(), epok::objects, epok::object_count);
  if (const auto amount=epok::transition.opacity>epok::screen_fade?epok::transition.opacity:epok::screen_fade) {
    auto& page=fade_pages[parity];
    page.primitive.attr.set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(2)).setDithering(false);
    epok::configure_display_field<epok::display_interlaced>(page.primitive.attr);
    gpu().chain(page);
    auto& fragment=fade_rectangles[parity];auto& rectangle=fragment.primitive;
    rectangle.position={{.x=0,.y=0}};
    rectangle.size={{.w=epok::display_width,.h=epok::display_height}};
    if(amount==255)rectangle.setColor(psyqo::Color{{.r=0,.g=0,.b=0}}).setOpaque();
    else rectangle.setColor(psyqo::Color{{.r=amount,.g=amount,.b=amount}}).setSemiTrans();
    gpu().chain(fragment);
    auto& restore=fade_restore_pages[parity];
    restore.primitive.attr.set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(0)).setDithering(false);
    epok::configure_display_field<epok::display_interlaced>(restore.primitive.attr);
    gpu().chain(restore);
  }
  loading_renderer.draw(gpu());
  EPOK_DETAIL_END(hud, hud_scanlines);
  epok::lighting_work.frame_scanlines =
      uint16_t(COUNTERS[1].value - frame_begin);
  epok::lighting_stats = epok::lighting_work;
  auto& usage=epok::resource_usage;usage={};
  usage.slot_capacity=epok::objects.size();usage.scene_banks=epok::scene_bank_count;
  for(size_t i=0;i<epok::object_count;++i){if(epok::objects[i].alive)++usage.alive_slots;if(epok::is_active_slot(i))++usage.active_slots;}
  for(size_t i=0;i<epok::texture_count;++i)if(const auto* t=epok::texture(int(i))){++usage.textures;usage.vram_texture_words+=uint32_t(t->word_width)*t->height;usage.vram_palette_words+=256;}
  usage.active_texture_bytes=2*(usage.vram_texture_words+usage.vram_palette_words);usage.resident_texture_bytes=epok::resident_texture_bytes;
  usage.particles=epok::particle_stats.alive;usage.particle_peak=epok::particle_stats.peak;usage.particle_dropped=epok::particle_stats.dropped;
  usage.mesh_triangles=epok::lighting_stats.triangles;usage.sprite_triangles=epok::sprite_stats.triangles;
  usage.dropped_primitives=epok::lighting_stats.dropped_triangles+epok::sprite_stats.dropped+epok::hud_stats.dropped;
  usage.sprite_estimated_pixels=epok::sprite_stats.estimated_pixels;
  usage.sprite_overdraw_per_mille=uint64_t(usage.sprite_estimated_pixels)*1000/(epok::display_width*epok::display_height);
  usage.frame_scanlines=epok::lighting_stats.frame_scanlines;
  performance_work.frame_scanlines=uint16_t(COUNTERS[1].value-frame_begin);
  performance_work.finish_scanlines=uint16_t(COUNTERS[1].value-finish_begin);
  epok::performance_stats=performance_work;
  // Last in the chain: visible above game HUD, fades and loading screen.
  epok::debug_hud::draw(gpu());
}
} // namespace
namespace epok {
void reset_motion_interpolation(){motion.clear();}
bool set_active_camera(ActorData* camera) {
  if(!camera){camera_override={};motion.clear();return true;}
  if(entity_index(camera)<0||!is_active(camera)||(!camera->camera&&!camera->camera_settings.enabled))return false;
  if(camera_override.index!=handle(camera).index||camera_override.generation!=handle(camera).generation)motion.clear();
  camera_override=handle(camera);return true;
}
DataHandle active_camera(){return handle(camera_entity());}
bool camera_project(const Fixed* world_point,Fixed* screen_xy) {
  if(!world_point||!screen_xy)return false;
  Fixed p[3];camera_view().point(world_point,p);p[0]*=projection_focal;p[1]*=projection_focal;
  const int64_t x=p[0].raw(),y=p[1].raw(),z=p[2].raw();
  if(z<1024||z>=128*4096)return false;
  const int64_t sx=int64_t(display_width/2)*4096+x*(display_width/2)*4096/z;
  const int64_t sy=int64_t(display_height/2)*4096-y*(display_height*2/3)*4096/z;
  if(sx<0||sy<0||sx>=int64_t(display_width)*4096||sy>=int64_t(display_height)*4096)return false;
  screen_xy[0]=Fixed(int32_t(sx),Fixed::RAW);screen_xy[1]=Fixed(int32_t(sy),Fixed::RAW);return true;
}
void reset_runtime_services() {
#ifdef EPOK_EFFECTS
  epok::effects::reset_scene();
#endif
#ifdef EPOK_TIMELINES
  epok::timeline::reset_scene();
#endif
#ifdef EPOK_BLUEPRINTS
  ++blueprint_scene_generation;
  if(!blueprint_scene_generation)blueprint_scene_generation=1;
#endif
  epok::streaming_scene_changed();
  transform_cache.clear();
  retained.reset();
  if constexpr (epok::stream_archive_fits) for (auto &binding : stream_bindings) binding = {};
  hud.invalidate();
  palettes.clear();
  lighting.clear();
  camera_override={};projection_focal=1.0;
  collision_world.clear();particles.clear();input.discard_edges();time=Time{};
}
void activate_texture_bank(const Texture* textures) {
  game.gpu().waitChainIdle();texture_assets=textures;textures_initialize(game.gpu());
}
void remove_runtime_owner(size_t index) {
#ifdef EPOK_EFFECTS
  epok::effects::remove_owner(handle(&objects[index]));
#endif
#ifdef EPOK_TIMELINES
  epok::timeline::remove_owner(handle(&objects[index]));
#endif
  particles.remove_owner(index);lighting.reset_owner(index);retained.forget(index);
}
#ifdef EPOK_EFFECTS
Affine<Fixed> effect_world(DataHandle owner){refresh_world();return owner.get()?world[owner.index]:Affine<Fixed>::identity();}
// The 3D helper lives in this translation unit's anonymous namespace. Qualify
// it explicitly: `epok::local_matrix` is the unrelated 2D overload exported by
// world2d.hpp and otherwise wins lookup from inside namespace epok.
Affine<Fixed> effect_matrix(const Transform& transform){return ::local_matrix(transform);}
void remove_effect_particles(EffectLayerHandle owner){particles.remove_layer(owner);}
#endif
SpatialHit raycast(const Fixed* origin,const Fixed* displacement,uint32_t mask,const ActorData* ignore,bool triggers) {
  refresh_collisions();return collision_world.raycast(origin,displacement,mask,entity_index(ignore),triggers);
}
size_t overlap(const Aabb& box,DataHandle* output,size_t capacity,uint32_t mask,const ActorData* ignore,bool triggers) {
  refresh_collisions();uint16_t indices[objects.size()];
  size_t count=collision_world.overlap(box,indices,objects.size(),mask,entity_index(ignore),triggers);
  if(output)for(size_t i=0;i<count&&i<capacity;++i)output[i]=handle(&objects[indices[i]]);
  return count;
}
bool collider_aabb(const ActorData& entity,Aabb& output) {
  refresh_collisions();int index=entity_index(&entity);
  auto box=index<0?nullptr:collision_world.bounds(size_t(index));if(!box)return false;
  output=*box;return true;
}
bool skeletal_world_point(const ActorData& entity,const Fixed* model,Fixed* output) {
  refresh_world();const int index=entity_index(&entity);
  if(index<0||size_t(index)>=object_count||!model||!output)return false;
  world[size_t(index)].point(model,output);return true;
}
WorldAffineSample gameplay_world_affine(const ActorData* entity) {
  WorldAffineSample result;if(!entity)return result;refresh_world();const int index=entity_index(entity);
  if(index<0||size_t(index)>=object_count)return result;const auto& value=world[size_t(index)];result.success=true;
  for(int row=0;row<3;++row){result.basis_x[row]=value.values[row][0];result.basis_y[row]=value.values[row][1];result.basis_z[row]=value.values[row][2];result.position[row]=value.values[row][3];}
  return result;
}
SpatialHit query_ground(const ActorData& entity,Fixed distance,uint32_t mask) {
  refresh_collisions();int index=entity_index(&entity);
  auto box=index<0?nullptr:collision_world.bounds(size_t(index));
  return box?collision_world.ground(*box,distance,mask&entity.collider.mask,index):SpatialHit{};
}
MoveResult move_and_slide(ActorData& entity,const Fixed* displacement,uint32_t mask) {
  refresh_collisions();int index=entity_index(&entity);
  auto box=index<0?nullptr:collision_world.bounds(size_t(index));
  if(!box) { MoveResult result;result.unresolved_overlap=true;return result; }
  auto result=collision_world.move_and_slide(*box,displacement,mask&entity.collider.mask,index);
  // A world displacement is a vector: transform only the inverse parent basis.
  Matrix inverse=Matrix::identity();int parent=entity.parent;unsigned depth=0;
  while(parent>=0&&size_t(parent)<object_count&&depth++<32) {
    inverse=inverse.compose(inverse_local(objects[parent].transform));parent=objects[parent].parent;
  }
  if(parent>=0) { result=MoveResult{};result.unresolved_overlap=true;return result; }
  for(int r=0;r<3;++r)for(int c=0;c<3;++c)entity.transform.position[r]+=inverse.values[r][c]*result.displacement[c];
  return result;
}
}
int main() { epok::serial_debug::capture(); return game.run(); }
