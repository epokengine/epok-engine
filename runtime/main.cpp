#include "psyqo/application.hh"
#include "psyqo/fragments.hh"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/triangles.hh"
#include "ordered_draw.hpp"
#include "ordered_projection.hpp"
#include "baked_packet.hpp"
#include "psyqo/scene.hh"
#include "psyqo/advancedpad.hh"
#include "memory_card_backend.hpp"
#include "fixed_trigonometry.hpp"
#include "track_camera_pose.hpp"
#include "scene.hh"
#include "texture.hpp"
#include "texture_upload.hpp"
#include "palette.hpp"
#include "skeletal.hpp"
#include "audio-bank.hh"
#ifdef EPOK_HAS_SEQUENCES
#include "sequence_clock.hpp"
#endif
#include "music.hpp"
#include "streaming.hpp"
#include "video_service.hpp"
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
#include "baked_selection.hpp"
#include "polygon.hpp"
#include "retained.hpp"
#include "hud.hpp"
#include "dungeon_minimap.hpp"
#ifdef EPOK_WORLD2D
#include "world2d_render.hpp"
#endif
#include "hud_focus.hpp"
#include "loading_renderer.hpp"
#include "sky_gradient.hpp"
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
const Texture* pending_texture_bank=nullptr;
bool load_pending_texture_bank(psyqo::GPU& gpu) {
  if(!pending_texture_bank)return true;
  gpu.waitChainIdle();texture_assets=nullptr;
  const bool ready=upload_texture_bank(pending_texture_bank,texture_count,stream_page_count,stream_page_bytes,
    [&](uint32_t page){return streaming_acquire(page,gpu);},
    [](uint32_t page){streaming_release(page);},
    [&](const uint16_t* data,uint16_t x,uint16_t y,uint16_t width,uint16_t height){
      gpu.uploadToVRAM(data,{{{.x=int16_t(x),.y=int16_t(y)}},{{.w=int16_t(width),.h=int16_t(height)}}});
    });
  if(ready){texture_assets=pending_texture_bank;pending_texture_bank=nullptr;}
  else stream_failed=true; // Never publish/render a partially uploaded bank.
  return ready;
}
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
union ProjectionScratch {
  epok::ProjectedVertex full[scratch_vertex_capacity];
  epok::CompactProjectedVertex compact[1024/sizeof(epok::CompactProjectedVertex)];
  ProjectionScratch() {}
};
__attribute__((section(".scratchpad"))) ProjectionScratch projection_scratch;
constexpr size_t capacity = epok::render_capacity;
// If the entire archive fits, loaded pages cannot be evicted by another valid
// page. Retain whole objects eagerly, like resident geometry, without testing
// per-chunk packet validity on every subsequent frame.
constexpr bool demand_streaming = epok::stream_page_count > epok::stream_pool_pages;
static_assert(
    capacity <= 8192,
    "Scene exceeds the initial renderer's 8192-triangle buffer budget");
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
epok::SpriteRenderer<epok::sprite_triangle_budget> sprites;
epok::ParticlePool particles;
epok::PaletteRenderer palettes;
epok::HudRenderer hud;
epok::dungeon::MinimapRenderer<epok::dungeon_minimap_capacity,epok::display_interlaced> minimap;
// The player's world point for the minimap: Q12 x, y, z in 3D; pixels x, y in 2D.
[[maybe_unused]] bool minimap_player(int32_t* raw) {
  const auto& m=epok::dungeon::minimap;
  if(m.player<0||size_t(m.player)>=epok::object_count||!epok::is_active_slot(size_t(m.player)))return false;
  const auto& data=epok::objects[size_t(m.player)];
#ifdef EPOK_WORLD2D
  if(epok::dungeon::map.two_d) {
    const auto* root=data.owner&&epok::active_object_registry?epok::active_object_registry->resolve<epok::SceneComponent2D>(data.owner->root_id()):nullptr;
    epok::Affine2D matrix;
    if(!root||!epok::world_matrix_2d(*root,[](epok::ObjectId id){return epok::world2d::resolve_scene2d(id);},matrix))return false;
    raw[0]=matrix.t[0].raw();raw[1]=matrix.t[1].raw();raw[2]=0;
    return true;
  }
#endif
  const auto affine=epok::gameplay_world_affine(&data);
  if(!affine.success)return false;
  for(int k=0;k<3;++k)raw[k]=affine.position[k].raw();
  return true;
}
epok::LoadingRenderer loading_renderer;
psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> fade_pages[2];
psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> fade_restore_pages[2];
psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> fade_rectangles[2];
epok::BlobRenderer blobs;
epok::LightingRenderer<epok::objects.size()> lighting;
// Retained GPU packets for static meshes, one record per scene quad.
epok::RetainedGeometry<epok::objects.size(), epok::retained_geometry ? epok::retained_quad_capacity : 1> retained;
epok::FramePacketSlots<epok::retained_geometry ? capacity : 1> frame_packet_slots;
std::array<epok::MeshMaterialCache,epok::objects.size()> mesh_material_cache;
std::array<epok::StreamObjectBinding, epok::stream_archive_fits ? epok::objects.size() : 0> stream_bindings{};
epok::DataHandle camera_override;

// Degrees in the editor and scripts; fixed point throughout the PSX runtime.
bool rotation_terms(Fixed degrees,Fixed& sine,Fixed& cosine) {
    int32_t wrapped = degrees.raw();
    if (!wrapped)return false;
    // Angles rarely exceed a turn: subtract instead of a hardware division.
    constexpr int32_t turn = 360 * 4096;
    if (wrapped >= turn || wrapped <= -turn) wrapped %= turn;
    if (wrapped < 0)
      wrapped += turn;
    if (!wrapped)return false;
    // A read-only Q12 quarter table provides 4096 angular steps per turn.
    // Unsigned division by a constant lowers to multiply/shifts on MIPS.
    const uint32_t phase=uint32_t(wrapped)/360u;
    sine=Fixed(epok::sine_turn12(phase),Fixed::RAW);
    cosine=Fixed(epok::cosine_turn12(phase),Fixed::RAW);return true;
}
// Mesh vertices are projected by the GTE in Q8 camera units (see gte_geometry.hpp).
using RenderFrustum=epok::ScreenFrustum<epok::display_width,epok::display_height>;
// Q8 inputs (Q12 CPU coordinates shifted by four, or checked GTE matrices)
// stay within 2^27. Keep the per-vertex ordinary-mode outcode in 32 bits.
using Units = epok::CameraUnits<64,128*256,6,RenderFrustum,true>;
using OrderedUnits = epok::CameraUnits<64,1<<27,6,RenderFrustum,false>;
using epok::ProjectedVertex;
using epok::QuadPacket;
#ifdef EPOK_VALIDATE_GTE
// Cumulative failures: compact XY/depth/frustum, rigid/mesh triples, camera,
// screen, reserved; followed by the last camera XYZ and flags/outcode.
uint32_t gte_validation_diagnostic[12]={};
#endif
template<class Number> __attribute__((noinline,optimize("O3"))) bool project_compact_vertices(const Number (*vertices)[3],size_t count,
    const int32_t* origin,epok::CompactProjectedVertex* out) {
  size_t v=0;
  for(;v+2<count;v+=3){
    uint32_t screens[3];int32_t depths[3];
    if(!epok::project_geometry_triple(vertices+v,origin,screens,depths))return false;
    for(unsigned i=0;i<3;++i){out[v+i].screen.packed=screens[i];out[v+i].camera.value=uint16_t(depths[i]);}
  }
  for(;v<count;++v){
    int32_t camera[3];uint32_t flags;
    epok::project_geometry_vertex(vertices[v],origin,camera,out[v].screen.packed,flags);
    if(flags&epok::gte_projection_flags)return false;
    out[v].camera.value=uint16_t(camera[2]);
  }
#ifdef EPOK_VALIDATE_GTE
  for(size_t i=0;i<count;++i){
    ProjectedVertex reference{};uint32_t flags;
    epok::project_geometry_vertex(vertices[i],origin,reference.camera,reference.screen.packed,flags);
    const auto code=Units::outcode(reference.camera[0],reference.camera[1],reference.camera[2]);
    if(reference.screen.packed!=out[i].screen.packed||reference.camera[2]!=out[i].camera[2]||code){
      ++performance_work.gte_validation_errors;
      ++gte_validation_diagnostic[reference.screen.packed!=out[i].screen.packed?0:reference.camera[2]!=out[i].camera[2]?1:2];
      for(int c=0;c<3;++c)gte_validation_diagnostic[8+c]=uint32_t(reference.camera[c]);gte_validation_diagnostic[11]=code;
    }
  }
#endif
  return true;
}
inline bool project_compact_vertices(epok::MeshVertices vertices,size_t count,
    const int32_t* origin,epok::CompactProjectedVertex* out) {
  return vertices.wide() ? project_compact_vertices(vertices.wide_data(),count,origin,out)
                         : project_compact_vertices(vertices.narrow_data(),count,origin,out);
}
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
  int32_t scale_u = 0, scale_v = 0, base_u = 0, base_v = 0;
  void update(const epok::Material& material) {
    if (material.texture == texture && material.blend == blend) return;
    texture = material.texture; blend = material.blend; tex = epok::texture(material.texture);
    if (!tex) { command = 0x30000000u; return; }
    const auto clut = epok::texture_clut(*tex);
    const auto page = epok::texture_page(*tex, material.blend, epok::dither_3d);
    uint16_t clut_bits, page_bits;
    __builtin_memcpy(&clut_bits, &clut, 2); __builtin_memcpy(&page_bits, &page, 2);
    clut16 = uint32_t(clut_bits) << 16; tpage16 = uint32_t(page_bits) << 16;
    command = 0x34000000u | (material.blend == epok::BlendMode::Cutout ? 0u : 0x02000000u);
    scale_u = tex->width - 1; scale_v = tex->height - 1; base_u = epok::texture_base_u(*tex); base_v = tex->y % 256;
  }
  // Same mapping as texture_uv: Q12 face coordinates to 8-bit page pixels.
  uint32_t uv(int32_t u, int32_t v) const {
    u = u < 0 ? 0 : u > 4096 ? 4096 : u; v = v < 0 ? 0 : v > 4096 ? 4096 : v;
    return (uint32_t(base_u + (u * scale_u >> 12)) & 255u) | ((uint32_t(base_v + (v * scale_v >> 12)) & 255u) << 8);
  }
};
inline uint32_t color_bits(psyqo::Color c) { return c.packed & 0x00ffffffu; }
struct PolygonEmitter {
  psyqo::OrderingTable<buckets>* table;
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTriangle>* gouraud;
  psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTexturedTriangle>* textured;
  // Dynamic packets descend through gaps not claimed by visible retained chunks.
  uint32_t next_dynamic = capacity;
  uint32_t emitted = 0, retained_emitted = 0;
  bool editable = false;
  bool double_sided = false;
  bool mirror_x = false;
  int fixed_depth = -1; // Prepared packet fragments share one monotone OT bucket.
  // 0: rejected, 1: direct GPU submission, 2: software clipping required.
  inline int classify(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c) {
    if (a.outcode & b.outcode & c.outcode) return 0;
    if (!(a.visible && b.visible && c.visible)) return 2;
    // Inside all six planes, projected spans are bounded by the viewport.
    // Keep wide/near-plane cases on the checked path below.
    if (epok::display_width <= 1023 && epok::display_height <= 511 && !(a.outcode | b.outcode | c.outcode)) {
      const int32_t area = epok::screen_area(a, b, c);
      if (area == 0) return 0;
      if (editable && !double_sided && epok::screen_backface(area, mirror_x)) { ++epok::mesh_stats.backfaces; return 0; }
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
    if (editable && !double_sided && epok::screen_backface(area, mirror_x)) { ++epok::mesh_stats.backfaces; return 0; }
    return 1;
  }
  // Three visible vertices within the hardware span limits, already checked for
  // degeneracy and facing. Colors are final packet values, UVs are u | v << 8.
  template<class Vertex> inline void direct(const Vertex& a, const Vertex& b, const Vertex& c,
                     uint32_t ca, uint32_t cb, uint32_t cc, uint32_t uva, uint32_t uvb, uint32_t uvc,
                     const QuadPacket& q) {
    const int depth = fixed_depth>=0?fixed_depth:Units::bucket(a.camera[2], b.camera[2], c.camera[2]) + q.depth_bias;
    if (depth < 0 || depth >= buckets) return;
    uint32_t slot;
    if constexpr(epok::retained_geometry) slot=frame_packet_slots.allocate_dynamic();
    else slot=next_dynamic?--next_dynamic:capacity;
    if (slot>=capacity) { ++epok::lighting_work.dropped_triangles; return; }
    ++emitted;
    if (q.textured) {
      auto& f = textured[slot];
      uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
      w[0] = q.command | ca; w[1] = a.screen.packed; w[2] = uva | q.clut16;
      w[3] = cb; w[4] = b.screen.packed; w[5] = uvb | q.tpage16;
      w[6] = cc; w[7] = c.screen.packed; w[8] = uvc;
      table->insert(f, depth);
      return;
    }
    auto& f = gouraud[slot];
    uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
    w[0] = 0x30000000u | ca; w[1] = a.screen.packed; w[2] = cb; w[3] = b.screen.packed; w[4] = cc; w[5] = c.screen.packed;
    table->insert(f, depth);
  }
  // Retained slot: screen words every frame, colour words only when they changed.
  template<class Vertex> inline void retained_emit(uint32_t slot, const Vertex& a, const Vertex& b, const Vertex& c,
                            const epok::RetainedQuad& rec, const uint32_t* colors, int corner_a, int ib, int ic) {
    const int depth = Units::bucket(a.camera[2], b.camera[2], c.camera[2]) + rec.depth_bias;
    if (depth < 0 || depth >= buckets) return;
    ++emitted; ++retained_emitted;
    if (rec.textured) {
      auto& f = textured[slot];
      uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
      w[1] = a.screen.packed; w[4] = b.screen.packed; w[7] = c.screen.packed;
      if (colors) { w[0] = rec.command | colors[corner_a]; w[3] = colors[ib]; w[6] = colors[ic]; }
      table->insert(f, depth);
      return;
    }
    auto& f = gouraud[slot];
    uint32_t* w = reinterpret_cast<uint32_t*>(&f.primitive);
    w[1] = a.screen.packed; w[3] = b.screen.packed; w[5] = c.screen.packed;
    if (colors) { w[0] = 0x30000000u | colors[corner_a]; w[2] = colors[ib]; w[4] = colors[ic]; }
    table->insert(f, depth);
  }
  // Keep the common interior/no-fog loop out of the large general renderer.
  // Besides removing repeated clip tests, this bounds register pressure and
  // instruction-cache traffic on the R3000A. Packet colours/UVs stay retained.
  __attribute__((noinline)) void restore_retained_colors(epok::RetainedQuad& rec,uint32_t slot,uint8_t parity_bit) {
    static constexpr int corners[2][3]={{0,1,2},{0,2,3}};
    for(int t=0;t<2;++t){
      if(rec.textured){
        auto* w=reinterpret_cast<uint32_t*>(&textured[slot+t].primitive);
        w[0]=rec.command|epok::packet_color(rec.shaded[corners[t][0]],true,rec.gpu_colors);
        w[3]=epok::packet_color(rec.shaded[corners[t][1]],true,rec.gpu_colors);w[6]=epok::packet_color(rec.shaded[corners[t][2]],true,rec.gpu_colors);
      }else{
        auto* w=reinterpret_cast<uint32_t*>(&gouraud[slot+t].primitive);
        w[0]=0x30000000u|rec.shaded[corners[t][0]];w[2]=rec.shaded[corners[t][1]];w[4]=rec.shaded[corners[t][2]];
      }
    }
    rec.fogged&=uint8_t(~parity_bit);
  }
  template<class Vertex> __attribute__((noinline,optimize("O3"))) void retained_interior(const epok::MeshQuad* quads,
      epok::RetainedQuad* records,size_t count,uint32_t slot_base,
      const Vertex* vertices,uint8_t parity_bit,const uint32_t* visibility=nullptr) {
    uint32_t drawn=0,rejected=0;
    for(size_t q=0;q<count;++q){
      if(!epok::visibility_face(visibility,q))continue;
      auto& rec=records[q];const auto* indices=quads[q].indices;
      if(rec.fogged&parity_bit)restore_retained_colors(rec,slot_base+uint32_t(q)*2,parity_bit);
      auto triangle=[&](uint16_t ia,uint16_t ib,uint16_t ic,uint32_t slot){
        if(ia==ib||ib==ic||ia==ic)return;
        const auto& a=vertices[ia];const auto& b=vertices[ib];const auto& c=vertices[ic];
        const int32_t area=epok::screen_area(a,b,c);
        if(!area)return;
        if(editable&&!quads[q].material.double_sided&&epok::screen_backface(area,mirror_x)){++rejected;return;}
        const int depth=Units::bucket(a.camera[2],b.camera[2],c.camera[2])+rec.depth_bias;
        if(depth<0||depth>=buckets)return;
        if(rec.textured){
          auto& f=textured[slot];auto* w=reinterpret_cast<uint32_t*>(&f.primitive);
          w[1]=a.screen.packed;w[4]=b.screen.packed;w[7]=c.screen.packed;
          table->insert(f,depth);
        }else{
          auto& f=gouraud[slot];auto* w=reinterpret_cast<uint32_t*>(&f.primitive);
          w[1]=a.screen.packed;w[3]=b.screen.packed;w[5]=c.screen.packed;
          table->insert(f,depth);
        }
        ++drawn;
      };
      triangle(indices[0],indices[1],indices[2],slot_base+uint32_t(q)*2);
      triangle(indices[0],indices[2],indices[3],slot_base+uint32_t(q)*2+1);
    }
    emitted+=drawn;retained_emitted+=drawn;epok::mesh_stats.backfaces+=rejected;
  }
  // Boundary chunks still contain mostly ordinary, unclipped triangles. Keep
  // that loop compact as well; resolve UVs/materials only for an actual clip.
  __attribute__((noinline,optimize("O3"))) void retained_boundary(const epok::MeshQuad* quads,
      epok::RetainedQuad* records,size_t count,uint32_t slot_base,
      const ProjectedVertex* vertices,uint8_t parity_bit,const epok::Material& fallback,const uint32_t* visibility=nullptr) {
    MaterialState material;
    for(size_t q=0;q<count;++q){
      if(!epok::visibility_face(visibility,q))continue;
      auto& rec=records[q];if(rec.dynamic)continue;
      const auto& quad=quads[q];const auto* ix=quad.indices;
      double_sided=quad.material.double_sided;
      const auto& a=vertices[ix[0]];const auto& b=vertices[ix[1]];
      const auto& c=vertices[ix[2]];const auto& d=vertices[ix[3]];
      if(a.outcode&b.outcode&c.outcode&d.outcode)continue;
      const int first=(ix[0]==ix[1]||ix[1]==ix[2]||ix[0]==ix[2])?0:classify(a,b,c);
      const int second=(ix[0]==ix[2]||ix[2]==ix[3]||ix[0]==ix[3])?0:classify(a,c,d);
      if(!first&&!second)continue;
      const uint32_t slot=slot_base+uint32_t(q)*2;
      if(rec.fogged&parity_bit)restore_retained_colors(rec,slot,parity_bit);
      if(first!=2&&second!=2){
        if(first)retained_emit(slot,a,b,c,rec,nullptr,0,1,2);
        if(second)retained_emit(slot+1,a,c,d,rec,nullptr,0,2,3);
        continue;
      }
      material.update(editable?quad.material:fallback);
      QuadPacket packet;packet.textured=rec.textured;packet.command=rec.command;
      packet.gpu_colors=rec.gpu_colors;
      packet.clut16=material.clut16;packet.tpage16=material.tpage16;packet.depth_bias=rec.depth_bias;
      uint32_t uv[4]={};
      if(rec.textured)for(int v=0;v<4;++v)uv[v]=quad.packed_uv?quad.uvw[v]:material.uv(quad.uv[v][0],quad.uv[v][1]);
      if(first==1)retained_emit(slot,a,b,c,rec,nullptr,0,1,2);
      else if(first==2)clipped(a,b,c,rec.shaded[0],rec.shaded[1],rec.shaded[2],uv[0],uv[1],uv[2],packet);
      if(second==1)retained_emit(slot+1,a,c,d,rec,nullptr,0,2,3);
      else if(second==2)clipped(a,c,d,rec.shaded[0],rec.shaded[2],rec.shaded[3],uv[0],uv[2],uv[3],packet);
    }
  }
  // Software clipping stays out of line so the per-quad loop remains compact.
  // Colors here are the fogged 0..255 values; modulation happens after clipping.
  // KeepScreens: visible input corners keep the screen position they arrived
  // with (the ordered pass's GTE-exact ones), so only new intersections are
  // projected here and edges shared with unclipped neighbours stay sealed.
  template<class ClipUnits=Units,bool KeepScreens=false> __attribute__((noinline,optimize("O3"))) void clipped(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c,
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
      if constexpr (KeepScreens) { buffers[0][i].screen = points[i]->screen.packed; buffers[0][i].has_screen = points[i]->visible; }
    }
    int from = 0;
    const int count = epok::clip_polygon<ClipUnits>(buffers, 3, planes, from);
    if (count < 3) return;
    ++epok::mesh_stats.clipped;
    ProjectedVertex p[12]; uint32_t col[12], tex[12];
    for (int i = 0; i < count; ++i) {
      const auto& v = buffers[from][i];
      for (int d = 0; d < 3; ++d) p[i].camera[d] = v.p[d];
      if (KeepScreens && v.has_screen) { p[i].screen.packed = v.screen; p[i].visible = true; }
      else epok::project_cpu<ClipUnits, epok::display_width, epok::display_height>(p[i]);
      const uint32_t packed = epok::pack_color(uint8_t(v.color[0]), uint8_t(v.color[1]), uint8_t(v.color[2]));
      col[i] = epok::packet_color(packed, q.textured, q.gpu_colors);
      tex[i] = uint32_t(uint8_t(v.uv[0] >> 16)) | (uint32_t(uint8_t(v.uv[1] >> 16)) << 8);
    }
    for (int i = 1; i + 1 < count; ++i) {
      const int32_t area = epok::screen_area(p[0], p[i], p[i + 1]);
      if (area == 0) continue;
      if (editable && !double_sided && epok::screen_backface(area, mirror_x)) { ++epok::mesh_stats.backfaces; continue; }
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
      final[i] = epok::packet_color(shaded[i], q.textured, q.gpu_colors);
      uv[i] = q.textured ? material.uv(v[i]->uv[0], v[i]->uv[1]) : 0;
    }
    switch (classify(p[0], p[1], p[2])) {
    case 1: direct(p[0], p[1], p[2], final[0], final[1], final[2], uv[0], uv[1], uv[2], q); break;
    case 2: clipped(p[0], p[1], p[2], shaded[0], shaded[1], shaded[2], uv[0], uv[1], uv[2], q); break;
    default: break;
    }
  }
};
#include "ordered_renderer.hpp"
epok::BakedSelectionCache<epok::baked_draw_cache_slots> baked_selection_cache;
static_assert(epok::baked_draw_abi == 3, "Re-cook geometry with the current baked packet ABI");
// Small static kernel: cooked packet attributes and selected faces only. It is
// intentionally outside frame(), so dynamic material/animation paths do not
// compete with this loop's register allocation and instruction working set.
__attribute__((noinline,optimize("O3"))) void draw_baked_complex(
    PolygonEmitter& emitter,const epok::BakedDrawFace* faces,size_t count,
    const uint32_t* selected,const ProjectedVertex* vertices,const uint8_t* tint) {
  QuadPacket packet;
  const bool tinted=tint[0]!=255||tint[1]!=255||tint[2]!=255;
  for(size_t q=0;q<count;++q){
    if(!epok::visibility_face(selected,q))continue;
    const auto& face=faces[q];const auto* ix=face.indices;
    const auto& a=vertices[ix[0]];const auto& b=vertices[ix[1]];
    const auto& c=vertices[ix[2]];const auto& d=vertices[ix[3]];
    if(a.outcode&b.outcode&c.outcode&d.outcode)continue;
    emitter.double_sided=epok::baked_double_sided(face);
    const int first=(ix[0]==ix[1]||ix[1]==ix[2]||ix[0]==ix[2])?0:emitter.classify(a,b,c);
    const int second=(ix[0]==ix[2]||ix[2]==ix[3]||ix[0]==ix[3])?0:emitter.classify(a,c,d);
    if(!(first|second))continue;
    // Cooked texture words (ABI 3); untextured without a resident bank.
    packet.textured=epok::baked_textured(face,epok::texture_assets!=nullptr);
    if(packet.textured){
      const auto words=epok::baked_texture_words<epok::display_interlaced,epok::dither_3d>(face);
      packet.clut16=words.uv_clut&0xffff0000u;packet.tpage16=words.uv_tpage&0xffff0000u;
    }
    packet.command=epok::baked_command(face);packet.depth_bias=face.depth_bias;
    packet.gpu_colors=epok::baked_indexed_color(face);
    uint32_t final[4],shaded[4];
    for(unsigned v=0;v<4;++v){
      shaded[v]=packet.gpu_colors?epok::baked_corner_color(face,v,epok::vertex_color_palette):face.shaded;
      if(tinted)shaded[v]=epok::tint_color(shaded[v],epok::pack_color(tint[0],tint[1],tint[2]));
      final[v]=epok::packet_color(shaded[v],packet.textured,packet.gpu_colors);
    }
    const uint16_t uv[4]={epok::baked_uv(face,0),epok::baked_uv(face,1),epok::baked_uv(face,2),epok::baked_uv(face,3)};
    if(first==1)emitter.direct(a,b,c,final[0],final[1],final[2],uv[0],uv[1],uv[2],packet);
    // Corners in front of the near plane keep the screen position every
    // neighbouring face uses (GTE, or the CPU for a GTE-flagged corner); only
    // new intersections are projected by the clipper. No seam at clip edges.
    else if(first==2)emitter.clipped<Units,true>(a,b,c,shaded[0],shaded[1],shaded[2],uv[0],uv[1],uv[2],packet);
    if(second==1)emitter.direct(a,c,d,final[0],final[2],final[3],uv[0],uv[2],uv[3],packet);
    else if(second==2)emitter.clipped<Units,true>(a,c,d,shaded[0],shaded[2],shaded[3],uv[0],uv[2],uv[3],packet);
  }
}
// Triangles are the common compiled primitive. Keep their loop small enough
// for the R3000 instruction cache; actual quads and clipping share the checked
// out-of-line path above instead of duplicating it in every iteration.
__attribute__((noinline,optimize("O3"))) void draw_baked_faces(
    PolygonEmitter& emitter,const epok::BakedDrawFace* faces,size_t count,
    const uint32_t* selected,const ProjectedVertex* vertices,const uint8_t* tint) {
  QuadPacket packet;
  // Instance tint is dynamic, but uncommon for baked scenery. Keep its colour
  // arithmetic in the checked kernel, outside the triangle instruction loop.
  if(tint[0]!=255||tint[1]!=255||tint[2]!=255){
    draw_baked_complex(emitter,faces,count,selected,vertices,tint);return;
  }
  uint32_t mask=0;
  for(size_t q=0;q<count;++q){
    if(!(q&31))mask=selected[q>>5];else mask>>=1;
    if(!(mask&1))continue;
    const auto& face=faces[q];const auto* ix=face.indices;
    if(face.topology!=1){
      if(face.topology==0)draw_baked_complex(emitter,&face,1,nullptr,vertices,tint);
      continue;
    }
    const auto& a=vertices[ix[0]];const auto& b=vertices[ix[1]];const auto& c=vertices[ix[2]];
    emitter.double_sided=epok::baked_double_sided(face);
    const int kind=emitter.classify(a,b,c);
    if(!kind)continue;
    if(kind==2){draw_baked_complex(emitter,&face,1,nullptr,vertices,tint);continue;}
    // Cooked texture words (ABI 3); untextured without a resident bank.
    packet.textured=epok::baked_textured(face,epok::texture_assets!=nullptr);
    if(packet.textured){
      const auto words=epok::baked_texture_words<epok::display_interlaced,epok::dither_3d>(face);
      packet.clut16=words.uv_clut&0xffff0000u;packet.tpage16=words.uv_tpage&0xffff0000u;
    }
    const uint32_t ca=epok::baked_corner_color(face,0,epok::vertex_color_palette);
    const uint32_t cb=epok::baked_corner_color(face,1,epok::vertex_color_palette);
    const uint32_t cc=epok::baked_corner_color(face,2,epok::vertex_color_palette);
    packet.command=epok::baked_command(face);packet.depth_bias=face.depth_bias;
    emitter.direct(a,b,c,ca,cb,cc,epok::baked_uv(face,0),epok::baked_uv(face,1),epok::baked_uv(face,2),packet);
  }
}
// Camera matrix for static-world chunks, built as the ordered pass builds it:
// GTE inputs are world Q8 relative to the camera's Q8 lattice point and the
// translation carries the exact residual. Every chunk of every static-world
// actor then shares one matrix, so their common corners project identically.
struct StaticWorldView {
  Matrix matrix;int32_t base_offset[3];int32_t base_translation[3];bool valid=false;
};
StaticWorldView static_world_view(const epok::CameraSpace<Fixed>& view,const Fixed* row_scale) {
  StaticWorldView out;out.matrix=view.basis;
  int32_t origin[3];Fixed residual[3],translation[3];
  for(unsigned c=0;c<3;++c){
    const auto q8=view.origin[c].raw()>>4;origin[c]=q8*16;out.base_offset[c]=-q8;
    residual[c]=Fixed(origin[c]-view.origin[c].raw(),Fixed::RAW);
  }
  out.matrix.point(residual,translation);
  for(unsigned r=0;r<3;++r)out.matrix.values[r][3]=translation[r];
  for(unsigned r=0;r<2;++r)for(unsigned c=0;c<4;++c)out.matrix.values[r][c]*=row_scale[r];
  for(unsigned r=0;r<3;++r)out.base_translation[r]=out.matrix.values[r][3].raw()>>4;
  out.valid=true;
  return out;
}
template<class WorkingSet>
__attribute__((noinline,optimize("O3"))) void draw_baked_object(
    const epok::ActorData& object,const Matrix& object_view,PolygonEmitter& emitter,
    epok::StreamPageCursor& cursor,const WorkingSet& pages,psyqo::GPU& gpu,
    ProjectedVertex* ram,const StaticWorldView* world=nullptr) {
  bool matrix_loaded=epok::load_projection_matrix(object_view);
  bool world_translation_moved=false;
  const bool object_gte=matrix_loaded;
  emitter.editable=epok::active_visibility_track->cull_backfaces;
  for(auto* mesh=object.geometry;mesh;mesh=mesh->next){
    if(!epok::visibility_track_started||!epok::visibility_chunk(mesh->visibility_faces,mesh->quad_count)){
      ++performance_work.visibility_skipped_chunks;continue;
    }
    ++epok::mesh_stats.tested_chunks;++epok::mesh_stats.visible_chunks;
    if(mesh->vertex_count>512||mesh->quad_count>96||!mesh->visibility_faces||
        (demand_streaming&&!epok::streaming_working_page_ready(pages,mesh->stream_page))){
      ++performance_work.stream_budget_skipped_chunks;continue;
    }
    const auto payload=cursor.resolve(*mesh,gpu);
    if(!payload.valid){++performance_work.stream_failed_chunks;continue;}
    const auto* faces=reinterpret_cast<const epok::BakedDrawFace*>(payload.quads);
    const auto& selection=baked_selection_cache.get(mesh,
        size_t(mesh->visibility_faces-epok::active_visibility_track->current),
        mesh->visibility_faces,faces,mesh->quad_count);
    const auto* selected=selection.vertices;const auto count=selection.count;
    auto* projected=mesh->vertex_count<=scratch_vertex_capacity?projection_scratch.full:ram;
    int32_t origin[3]={mesh->origin[0]>>4,mesh->origin[1]>>4,mesh->origin[2]>>4};
    Matrix chunk_view;
#ifdef EPOK_VALIDATE_GTE
    Matrix reference_view=object_view;
#endif
    bool gte=false;
    if(world&&(mesh->editable&epok::mesh_static_world)){
      // Cooked origins are Q8-aligned, so origin>>4 + local>>4 is exactly the
      // world Q8 point; offsetting by the camera's lattice point keeps it exact.
      for(int c=0;c<3;++c)origin[c]+=world->base_offset[c];
      if(object_gte&&epok::gte_vertex_bounds_fit(origin,mesh->center,mesh->extent)){
        if(world_translation_moved){epok::load_projection_translation(world->base_translation);world_translation_moved=false;}
        gte=true;
      }else if(object_gte){
        // Far from the camera: move the lattice by whole 4096-Q8 steps with
        // the exact integer translation, as the ordered pass re-anchors.
        int32_t center[3],offset[3],translation[3];
        for(int c=0;c<3;++c)center[c]=(mesh->origin[c]>>4)+(mesh->center[c]>>4);
        if(epok::ordered_projection_anchor(world->matrix,world->base_offset,center,offset,translation)){
          for(int c=0;c<3;++c)origin[c]=(mesh->origin[c]>>4)+offset[c];
          if(epok::gte_vertex_bounds_fit(origin,mesh->center,mesh->extent)){
            epok::load_projection_translation(translation);world_translation_moved=true;gte=true;
          }
        }
      }
      if(!gte){
        chunk_view=world->matrix;
        for(int r=0;r<3;++r)for(int c=0;c<3;++c)
          chunk_view.values[r][3]+=chunk_view.values[r][c]*Fixed(mesh->origin[c]+world->base_offset[c]*16,Fixed::RAW);
        origin[0]=origin[1]=origin[2]=0;
      }
#ifdef EPOK_VALIDATE_GTE
      reference_view=world->matrix;
      for(int r=0;r<3;++r)for(int c=0;c<3;++c)
        reference_view.values[r][3]+=reference_view.values[r][c]*Fixed(mesh->origin[c]+world->base_offset[c]*16,Fixed::RAW);
#endif
    }else{
      bool shared=object_gte;
      for(int c=0;c<3;++c)if(origin[c]>epok::gte_shared_origin_limit||origin[c]<-epok::gte_shared_origin_limit)shared=false;
      if(payload.vertices.wide()&&!epok::gte_vertex_bounds_fit(origin,mesh->center,mesh->extent))shared=false;
#ifdef EPOK_VALIDATE_GTE
      for(int r=0;r<3;++r)for(int c=0;c<3;++c)
        reference_view.values[r][3]+=reference_view.values[r][c]*Fixed(mesh->origin[c],Fixed::RAW);
#endif
      if(shared){if(!matrix_loaded)matrix_loaded=epok::load_projection_matrix(object_view);gte=matrix_loaded;}
      else {
        chunk_view=object_view;
        for(int r=0;r<3;++r)for(int c=0;c<3;++c)chunk_view.values[r][3]+=chunk_view.values[r][c]*Fixed(mesh->origin[c],Fixed::RAW);
        origin[0]=origin[1]=origin[2]=0;gte=epok::load_projection_matrix(chunk_view);matrix_loaded=false;
        if(payload.vertices.wide()&&!epok::gte_vertex_bounds_fit(origin,mesh->center,mesh->extent))gte=false;
      }
    }
    const uint16_t begin=COUNTERS[1].value;
    uint32_t vertex_mask=0;
    // Choose width once per chunk, preserving the narrow hot loop.
    auto project_selected=[&](const auto* vertices){
    for(size_t v=0;v<mesh->vertex_count;++v){
      if(!(v&31))vertex_mask=selected[v>>5];else vertex_mask>>=1;
      if(!(vertex_mask&1))continue;
      auto& p=projected[v];uint32_t flags=epok::gte_projection_flags;
      if(gte)epok::project_geometry_vertex(vertices[v],origin,p.camera,p.screen.packed,flags);
      else{Fixed local[3],camera[3];for(int c=0;c<3;++c)local[c]=Fixed(vertices[v][c],Fixed::RAW);
        chunk_view.point(local,camera);for(int c=0;c<3;++c)p.camera[c]=camera[c].raw()>>4;}
#ifdef EPOK_VALIDATE_GTE
      if(gte){
        Fixed local[3],camera[3];for(int c=0;c<3;++c)local[c]=Fixed(vertices[v][c],Fixed::RAW);
        reference_view.point(local,camera);
        for(int c=0;c<3;++c){
          const int64_t difference=int64_t(p.camera[c])*16-camera[c].raw();
          const uint32_t delta=uint32_t(difference<0?-difference:difference);
          if(delta>performance_work.gte_max_delta)performance_work.gte_max_delta=delta;
          if(delta>128){++performance_work.gte_validation_errors;++gte_validation_diagnostic[5];}
        }
        if(!(flags&epok::gte_projection_flags)&&p.camera[2]>=Units::near&&p.camera[2]<Units::far){
          auto reference=p;epok::project_cpu<Units,epok::display_width,epok::display_height>(reference);
          const int dx=reference.screen.x-p.screen.x,dy=reference.screen.y-p.screen.y;
          const uint32_t delta=uint32_t((dx<0?-dx:dx)>(dy<0?-dy:dy)?(dx<0?-dx:dx):(dy<0?-dy:dy));
          if(delta>performance_work.gte_screen_max_delta)performance_work.gte_screen_max_delta=delta;
          if(delta>2){++performance_work.gte_validation_errors;++gte_validation_diagnostic[6];}
        }
      }
#endif
      p.outcode=Units::outcode(p.camera[0],p.camera[1],p.camera[2]);
      p.fog=0;p.visible=p.camera[2]>=Units::near&&p.camera[2]<Units::far;
      if(p.visible&&(flags&epok::gte_projection_flags))epok::project_cpu<Units,epok::display_width,epok::display_height>(p);
    }
    };
    if(payload.vertices.wide())project_selected(payload.vertices.wide_data());
    else project_selected(payload.vertices.narrow_data());
    performance_work.vertex_scanlines+=uint16_t(COUNTERS[1].value-begin);
    if(gte)performance_work.gte_vertices+=count;else performance_work.software_vertices+=count;
    epok::mesh_stats.transformed_vertices+=count;
    const uint16_t polygons=COUNTERS[1].value;
    draw_baked_faces(emitter,faces,mesh->quad_count,mesh->visibility_faces,projected,object.material.color);
    performance_work.polygon_scanlines+=uint16_t(COUNTERS[1].value-polygons);
  }
}
Matrix local_matrix(const epok::Transform &transform) {
  ++performance_work.local_matrices;
  Matrix out;
  for(int c=0;c<3;++c)out.values[c][c]=transform.scale[c];
  for(int axis=0;axis<3;++axis){
    Fixed sine,cosine;
    if(rotation_terms(transform.rotation[axis],sine,cosine))out.rotate_rows(axis,sine,cosine);
  }
  for (int r = 0; r < 3; ++r)
    out.values[r][3] = transform.position[r];
  return out;
}
Matrix inverse_local(const epok::Transform &transform,bool translation=true) {
  Matrix out=Matrix::identity();
  if(translation)for(int r=0;r<3;++r)out.values[r][3]=-transform.position[r];
  for(int axis=2;axis>=0;--axis){
    Fixed sine,cosine;
    if(rotation_terms(-transform.rotation[axis],sine,cosine))out.rotate_rows(axis,sine,cosine,translation?4:3);
  }
  for(int r=0;r<3;++r){
    // Unit scale is exact and avoids twelve software 64-bit divisions for a
    // normal camera. Preserve the existing zero-scale fallback for scripts.
    if(transform.scale[r]==Fixed(1.0))continue;
    for(int c=0;c<4;++c)out.values[r][c]=transform.scale[r].raw()==0?Fixed(0,Fixed::RAW):out.values[r][c]/transform.scale[r];
  }
  return out;
}
epok::ActorData* camera_entity() {
  auto valid=[](epok::ActorData* e){return e&&epok::is_active(e)&&(e->camera||e->camera_settings.enabled);};
  if(auto* e=camera_override.get();valid(e))return e;
  for(size_t i=0;i<epok::object_count;++i)if(valid(&epok::objects[i]))return &epok::objects[i];
  return nullptr;
}
void refresh_world_for(size_t index);
epok::CameraSpace<Fixed> camera_view(bool interpolated=false,const Matrix* rendered=nullptr) {
  epok::Transform fallback={{0.0,3.0,-6.0},{18.0,0.0,0.0},{1.0,1.0,1.0}};
  auto* camera=camera_entity();
  epok::projection_focal=1.0;
  epok::projection_focal_x=1.0;
  epok::CameraSpace<Fixed> frame;
  if(!camera){
    frame.basis=inverse_local(fallback,false);
    for(int r=0;r<3;++r)frame.origin[r]=fallback.position[r];
    return frame;
  }
  int32_t degrees=camera->camera_settings.field_of_view.raw();
  if(degrees<25*4096)degrees=25*4096;if(degrees>120*4096)degrees=120*4096;
  if(degrees!=90*4096){const uint32_t half=uint32_t(degrees)/720u;
    epok::projection_focal=Fixed(epok::cosine_turn12(half),Fixed::RAW)/Fixed(epok::sine_turn12(half),Fixed::RAW);}
  epok::projection_focal_x=camera->camera_settings.mirror_x?-epok::projection_focal:epok::projection_focal;
  auto view=Matrix::identity();
  const int camera_index=epok::entity_index(camera);
  if(!rendered)refresh_world_for(size_t(camera_index));
  const auto& camera_world=rendered?rendered[camera_index]:world[camera_index];
  for(int r=0;r<3;++r)frame.origin[r]=camera_world.values[r][3];
  int i=camera_index;unsigned depth=0;
  for(;i>=0&&size_t(i)<epok::object_count&&depth<33;++depth){
    const bool tracked=epok::active_visibility_track&&size_t(i)==epok::active_visibility_track->camera;
    const auto transform=interpolated&&!tracked?motion.local(size_t(i),epok::objects[i].transform):epok::objects[i].transform;
    const auto inverse=inverse_local(transform,false);
    view=depth?view.compose(inverse):inverse;
    i=epok::objects[i].parent;
  }
  frame.basis=view;return frame;
}
void refresh_world() {
  ScanlineScope timer{performance_work.world_scanlines};
  ++performance_work.world_syncs;
  transform_cache.sync(epok::objects,world,epok::object_count,local_matrix);
  performance_work.world_matrices+=transform_cache.world_rebuilds;
}
void refresh_world_for(size_t index) {
  ScanlineScope timer{performance_work.world_scanlines};++performance_work.world_syncs;
  bool selected[epok::objects.size()]={};int current=int(index);unsigned depth=0;
  while(current>=0&&size_t(current)<epok::object_count&&depth++<33){
    if(selected[current])break;selected[current]=true;current=epok::objects[size_t(current)].parent;
  }
  transform_cache.sync(epok::objects,world,epok::object_count,local_matrix,selected);
  performance_work.world_matrices+=transform_cache.world_rebuilds;
}
__attribute__((noinline,optimize("O3"))) void refresh_collisions() {
  ScanlineScope timer{performance_work.collision_scanlines};
  // Scripts can write transforms directly. Still inspect every active collider
  // and all its parents, but do not inspect nine scalars of unrelated render,
  // audio, camera, VFX or reserved slots at every physics query.
  bool selected[epok::objects.size()]={};
  uint16_t colliders[epok::objects.size()];size_t collider_count=0;bool roots_only=true;
  for(size_t index=0;index<epok::object_count;++index) {
    const auto& object=epok::objects[index];
    if(!object.collider.enabled||!epok::is_active_slot(index))continue;
    colliders[collider_count++]=uint16_t(index);roots_only&=object.parent<0;
  }
  if(!roots_only)for(size_t n=0;n<collider_count;++n){
    const size_t index=colliders[n];
    int ancestor=int(index);unsigned depth=0;
    while(ancestor>=0&&size_t(ancestor)<epok::object_count&&depth++<33){
      if(selected[ancestor])break;
      selected[ancestor]=true;ancestor=epok::objects[ancestor].parent;
    }
  }
  {
    ScanlineScope world_timer{performance_work.world_scanlines};
    ++performance_work.world_syncs;
    if(roots_only){
      transform_cache.local_rebuilds=transform_cache.world_rebuilds=0;
      for(size_t n=0;n<collider_count;++n){const size_t index=colliders[n];
        transform_cache.sync_root(epok::objects[index],world[index],index,local_matrix);}
    }else transform_cache.sync(epok::objects,world,epok::object_count,local_matrix,selected);
    performance_work.world_matrices+=transform_cache.world_rebuilds;
  }
  collision_world.begin_sync();
  for(size_t n=0;n<collider_count;++n) {
    const size_t index=colliders[n];
    const auto& object=epok::objects[index];
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
    epok::nav::move_actor=&epok::move_and_slide;
    epok::nav::bounds_actor=&epok::collider_aabb;
#ifdef EPOK_WORLD2D
    epok::world2d::install();
#endif
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
      if (!epok::pending_texture_bank && epok::is_active(&object) && object.audio.enabled && object.audio.play_on_start &&
          !object.audio.is_playing())
        object.audio.play();
#endif
    syscall_puts("EPOK: runtime ready\n");
  }
  void frame() override;
  psyqo::OrderingTable<buckets> ordering[2];
  // Separate packets, not a depth bias: base surfaces cannot overwrite the world.
  psyqo::OrderingTable<buckets> background_ordering[2];
  psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> geometry_pages[2], hud_pages[2];
  epok::FrameClear<epok::display_width,epok::display_height> clear;
  epok::SkyGradient<epok::display_width,epok::display_height> sky_gradient;
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
    // Videos share the BGM drive and its ISO9660 directory.
    if constexpr (epok::video_clip_count > 0) epok::music_prepare(true);
    // PsyQo callback trampolines live in writable RAM. GPU, pad, memory-card
    // and sequence setup assign their lambdas after PsyQo's early cache flush;
    // flush again before interrupts can dispatch one of those RAM stubs.
    syscall_flushCache();
  }
  void createScene() override { pushScene(&scene); }
  GameScene scene;
};
Game game;

void GameScene::frame() {
#ifdef EPOK_HAS_VIDEO
  // A playing video owns the screen, the CD and the pad; the game stands still.
  if(epok::fmv::frame(gpu(),pad)){epok::time.synchronize(gpu().now());return;}
#endif
  const uint16_t frame_begin = COUNTERS[1].value;
  epok::debug_hud::begin(gpu());
  performance_work={};
  if constexpr(epok::ordered_draw_enabled)ordered_work={};
  performance_work.frame=epok::performance_stats.frame+1;
  const bool scene_changed=epok::scene_tick(gpu());
  if(scene_changed||epok::scene_loading())motion.clear();
  epok::input.poll_multitap(pad); // AdvancedPad's Pad1a..Pad1d are 0..3.
#ifdef EPOK_TRANSITIONS
  if(epok::stream_failed&&!epok::transition.loading())epok::transition.fail();
  if(epok::transition.phase==epok::TransitionPhase::Failed&&epok::input.frame_pressed(epok::Button::Start)){
    if(epok::streaming_retry()){
      epok::transition.phase=epok::TransitionPhase::Loading;epok::transition.boot=true;epok::transition.presented=false;
    }
  }
  if(scene_changed||epok::transition.boot_ready()){
    bool ok=epok::load_pending_texture_bank(gpu());
    if constexpr(epok::stream_page_count>0)if(ok)ok=epok::streaming_warmup_scene(epok::objects.data(),epok::object_count,[](size_t i){return epok::is_active_slot(i);},gpu());
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
  if(epok::pending_texture_bank&&epok::input.frame_pressed(epok::Button::Start)&&epok::streaming_retry()){
    if(epok::load_pending_texture_bank(gpu())){
      if constexpr(epok::stream_page_count>0)epok::streaming_warmup_scene(epok::objects.data(),epok::object_count,[](size_t i){return epok::is_active_slot(i);},gpu());
      for(auto& object:epok::objects)if(epok::is_active(&object)&&object.audio.enabled&&object.audio.play_on_start&&!object.audio.is_playing())object.audio.play();
    }
  }
#endif
  if(epok::pending_texture_bank){
    epok::time.synchronize(gpu().now());epok::music_tick();epok::streaming_tick();
    clear.draw(gpu(),psyqo::Color{{.r=0,.g=0,.b=0}});loading_renderer.draw(gpu());
    epok::performance_stats=performance_work;return;
  }
  const unsigned steps=epok::scene_loading()?(epok::time.synchronize(gpu().now()),0u):epok::time.advance(gpu().now());
  performance_work.steps=steps;
  if(!epok::scene_loading())epok::level.frame_update(epok::time.frame_microseconds);
  if(epok::time.paused()||epok::scene_loading())epok::input.discard_edges();
  motion.select(epok::objects,epok::object_count,[](size_t i){
    const auto& o=epok::objects[i];
    return (o.mesh||o.sprite.enabled||o.camera||o.camera_settings.enabled||o.light.enabled||o.blob_shadow.enabled)&&epok::is_active_slot(i);
  });
  bool advanced_motion=false;
  if(!epok::time.paused()&&!epok::scene_loading())epok::nav::world.tick();
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
#ifdef EPOK_WORLD2D
    // World2D after the actors moved: animation, colliders, triggers, cameras.
    epok::world2d::service.step(dt,&epok::level);
#endif
    bool needs_triggers=collision_world.has_trigger_pairs();
    for(size_t i=0;i<epok::object_count&&!needs_triggers;++i)
      needs_triggers=epok::objects[i].collider.enabled&&epok::objects[i].collider.trigger&&epok::is_active_slot(i);
    if(needs_triggers){refresh_collisions();collision_world.update_triggers(dispatch_trigger);}
    bool has_emitters=false;
    for(size_t i=0;i<epok::object_count;++i)if(
        (epok::objects[i].particle_emitter.enabled||epok::objects[i].animator.enabled||
         epok::objects[i].sprite_animator.enabled||epok::objects[i].palette_animator.enabled)&&epok::is_active_slot(i)) {
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

  bool demand_verification_serviced=false;
  auto required_page_ready=[&](uint32_t page){
    return epok::streaming_required_page_ready(page,demand_verification_serviced);
  };
  auto track_pages_ready=[this,&required_page_ready](bool next){
        if(epok::visibility_has_page_sets()){
          if(next&&epok::rail_pending_cut()){
            uint32_t pages[decltype(epok::stream_pool)::capacity+1];size_t count=0;
            epok::visibility_visit_pages(true,[&](uint32_t page){
              if(count<=decltype(epok::stream_pool)::capacity)pages[count++]=page;
            });
            if(count>decltype(epok::stream_pool)::capacity)return false;
            const bool loaded=epok::streaming_warmup(pages,count,gpu());
            epok::time.synchronize(gpu().now());
            return loaded;
          }
          bool ready=true;
          epok::visibility_visit_pages(next,[&](uint32_t page){ready&=required_page_ready(page);});
          return ready;
        }
        if(next&&epok::rail_pending_cut()){
          // A camera cut has no intermediate view. Finish all destination I/O
          // before returning to the render pass: the last complete framebuffer
          // stays on screen while old, unpinned geometry pages are replaced.
          uint32_t pages[decltype(epok::stream_pool)::capacity+1];size_t count=0;
          for(size_t i=0;i<epok::object_count;++i)for(auto* mesh=epok::objects[i].geometry;mesh;mesh=mesh->next){
            if(!epok::is_active_slot(i)||!mesh->visibility_faces||mesh->stream_page==epok::stream_invalid_page)continue;
            if(!epok::visibility_chunk(epok::visibility_next_mask(mesh->visibility_faces),mesh->quad_count))continue;
            bool seen=false;for(size_t p=0;p<count;++p)seen|=pages[p]==mesh->stream_page;
            if(seen)continue;
            pages[count++]=mesh->stream_page;
            if(count>decltype(epok::stream_pool)::capacity)return false; // Cook rejects this layout.
          }
          const bool loaded=epok::streaming_warmup(pages,count,gpu());
          epok::time.synchronize(gpu().now()); // No gameplay catch-up across storage waits.
          return loaded;
        }
        for(size_t i=0;i<epok::object_count;++i)for(auto* mesh=epok::objects[i].geometry;mesh;mesh=mesh->next){
          if(!epok::is_active_slot(i)||!mesh->visibility_faces||mesh->stream_page==epok::stream_invalid_page)continue;
          const auto* mask=next?epok::visibility_next_mask(mesh->visibility_faces):mesh->visibility_faces;
          if(epok::visibility_chunk(mask,mesh->quad_count)&&!required_page_ready(mesh->stream_page))return false;
        }
        return true;
      };
  auto track_ready=[&](bool next){
    if(!track_pages_ready(next))return false;
    if constexpr(epok::ordered_draw_enabled){
      const auto* data=epok::active_visibility_track->ordered;
      if(data){
        const auto sample=next?epok::visibility_page_selection.next:epok::visibility_page_selection.current;
        int pinned=-1;uint32_t pinned_page=epok::stream_invalid_page;
        const bool prepared=ordered_cursor.prepare(data,sample,[&](uint32_t page)->const uint8_t*{
          pinned_page=page;pinned=epok::stream_pool.pin_slot(page);
          return epok::stream_pool.destination(pinned);
        });
        if(pinned>=0)epok::stream_pool.unpin_slot(pinned,pinned_page);
        if(!prepared){epok::stream_failed=true;++performance_work.stream_failed_chunks;return false;}
        // Establish the current checkpoint before a lookahead prepare borrows
        // scratch buffers. Camera publication occurs later in this same frame.
        if(!next&&!ordered_cursor.publish(data,sample)){epok::stream_failed=true;return false;}
      }
    }
    return true;
  };
  const auto* track=epok::active_visibility_track;
  const auto elapsed=epok::time.paused()?0:epok::time.frame_microseconds;
  const auto previous_transitions=epok::rail_navigation_state.transitions;
  auto track_camera=[&](const epok::VisibilityTrackSample& a,const epok::VisibilityTrackSample& b,uint32_t fraction){
        auto pose=epok::interpolate_track_camera(a,b,fraction);
        if(track->navigation&&track->navigation->aim){
          const auto target=track->navigation->target;
          const auto& pos=epok::objects[target].transform.position;
          const int32_t player[]={pos[0].raw(),pos[1].raw(),pos[2].raw()};
          epok::aim_track_camera(pose,epok::is_active_slot(target)?elapsed:0,player,previous_transitions);
        }
        auto& actor=epok::objects[track->camera];
        actor.camera_settings.field_of_view=Fixed(pose.fov,Fixed::RAW);
        for(unsigned c=0;c<3;++c){actor.transform.position[c]=Fixed(pose.position[c],Fixed::RAW);actor.transform.rotation[c]=Fixed(pose.rotation[c],Fixed::RAW);}
      };
#ifdef EPOK_VISIBILITY_SWEEP
  if(track&&track->navigation){
    epok::advance_rail_sweep(track_ready,[&](const epok::VisibilityTrackSample& a,const epok::VisibilityTrackSample& b,uint32_t fraction){
      const auto pose=epok::interpolate_track_camera(a,b,fraction);
      auto& actor=epok::objects[track->camera];
      actor.camera_settings.field_of_view=Fixed(pose.fov,Fixed::RAW);
      for(unsigned c=0;c<3;++c){actor.transform.position[c]=Fixed(pose.position[c],Fixed::RAW);actor.transform.rotation[c]=Fixed(pose.rotation[c],Fixed::RAW);}
    });
  }else
#else
  if(track&&track->navigation){
    const auto target=track->navigation->target;
    const auto& pos=epok::objects[target].transform.position;
    const int32_t player[]={pos[0].raw(),pos[1].raw(),pos[2].raw()};
    epok::advance_rail_navigation(epok::is_active_slot(target)?elapsed:0,player,track_ready,track_camera);
  }else
#endif
  epok::advance_visibility_track(elapsed,track_ready,track_camera);
  if constexpr(epok::ordered_draw_enabled)if(track&&track->ordered&&epok::visibility_track_started){
    if(!ordered_cursor.publish(track->ordered,epok::visibility_page_selection.current)){
      epok::stream_failed=true;++performance_work.stream_failed_chunks;
    }
  }
  if(epok::stream_failed&&(epok::rail_pending_cut()||(track&&track->ordered))){
    // The source pages may already have been replaced. Never render their old
    // masks after a failed cut; retain the normal loading/error presentation.
    clear.draw(gpu(),psyqo::Color{{.r=0,.g=0,.b=0}});
    loading_renderer.draw(gpu());epok::performance_stats=performance_work;return;
  }

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
  const auto* clear_camera=camera_entity();
#ifdef EPOK_WORLD2D
  // With no 3D camera the active Camera2D paints the background.
  const auto* sky_color=clear_camera?clear_camera->camera_settings.sky_color:epok::world2d::service.background();
#else
  const auto* sky_color=clear_camera?clear_camera->camera_settings.sky_color:nullptr;
#endif
  const uint8_t sky_r=sky_color?sky_color[0]:33;
  const uint8_t sky_g=sky_color?sky_color[1]:40;
  const uint8_t sky_b=sky_color?sky_color[2]:52;
  // A gradient background is drawn once the projection is known, below.
  const bool gradient_background=clear_camera&&clear_camera->camera_settings.sky_gradient;
  if(!gradient_background)clear.draw(gpu(),psyqo::Color{{.r = sky_r, .g = sky_g, .b = sky_b}});
  const auto view = camera_view(true,render_world.data());
  EPOK_DETAIL_END(camera, camera_scanlines);
  static constexpr int16_t vertices[8][3] = {
      {-2048, -2048, -2048}, {2048, -2048, -2048}, {2048, 2048, -2048},
      {-2048, 2048, -2048},  {-2048, -2048, 2048}, {2048, -2048, 2048},
      {2048, 2048, 2048},    {-2048, 2048, 2048}};
  static constexpr epok::MeshQuad faces[6] = {
      {{0, 1, 2, 3}, 0}, {{5, 4, 7, 6}, 1}, {{4, 0, 3, 7}, 2},
      {{1, 5, 6, 2}, 3}, {{3, 2, 6, 7}, 4}, {{4, 5, 1, 0}, 5}};
  static constexpr epok::MeshGeometry default_cube = {vertices, 8, faces, 6};
  union ProjectionRam {
    ProjectedVertex full[512];epok::CompactProjectedVertex compact[512];
    ProjectionRam() {}
  } projection_ram;
  auto* projected_vertices_ram=projection_ram.full;
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
      horizontal_pixels == projection_focal_pixels ? epok::projection_focal_x : epok::projection_focal_x * Fixed(horizontal_pixels) / Fixed(projection_focal_pixels),
      epok::projection_focal};
  epok::load_projection_screen(projection_focal_pixels, epok::display_width / 2, epok::display_height / 2);
  if(gradient_background){
    const auto& settings=clear_camera->camera_settings;
    epok::SkyGradientColors colors;
    for(int c=0;c<3;++c){colors.sky[c]=settings.sky_color[c];colors.horizon[c]=settings.horizon_color[c];colors.ground[c]=settings.ground_color[c];}
    epok::SkyGradientView sky_view;
    if(!epok::sky_gradient_view(view.basis,row_scale[0].raw(),row_scale[1].raw(),projection_focal_pixels,epok::display_width/2,epok::display_height/2,sky_view)
       ||!sky_gradient.draw(gpu(),sky_view,colors))
      clear.draw(gpu(),psyqo::Color{{.r = sky_r, .g = sky_g, .b = sky_b}});
  }
  PolygonEmitter emitter{&table, triangles[parity], textured_triangles[parity]};
  emitter.mirror_x=epok::projection_focal_x.raw()<0;
  if constexpr(epok::retained_geometry)frame_packet_slots.begin_frame();
  bool background_active=false;
  constexpr size_t working_slots = decltype(epok::stream_pool)::capacity;
  epok::StreamPageWorkingSet<working_slots> working_pages;
  int working_pins[working_slots];
  if constexpr (demand_streaming) {
    const bool baked_pages=epok::visibility_has_page_sets();
    if(baked_pages){
      epok::visibility_visit_pages(false,[&](uint32_t page){working_pages.consider(page,0);});
      if(epok::visibility_prefetch_next())
        epok::visibility_visit_pages(true,[&](uint32_t page){working_pages.consider(page,1);});
    }
    // Select camera-near pages from permanent bounds before reading payloads.
    for (size_t index = 0; index < epok::object_count; ++index) {
      const auto& object = epok::objects[index];
      if (!object.mesh || !epok::is_active_slot(index)) continue;
      const bool skeletal = object.animator.enabled && object.animator.model;
      const auto* root = skeletal ? object.animator.model->geometry : object.geometry;
      if (!root) continue;
      if(baked_pages&&root->visibility_faces)continue;
      // The prepared packet order draws these actors from its own pages; their
      // ordinary chunks are query copies, loaded on request, not by distance.
      if constexpr(epok::ordered_draw_enabled)if(track&&track->ordered
          &&index<track->ordered->actor_count&&track->ordered->actors[index])continue;
      const auto object_view = root->visibility_faces?Matrix::identity():view.compose(render_world[index]);
      for (auto* mesh = root; mesh; mesh = mesh->next) {
        if (mesh->stream_page == epok::stream_invalid_page || !mesh->vertex_count) continue;
        if(mesh->visibility_faces){
          const bool current=epok::visibility_chunk(mesh->visibility_faces,mesh->quad_count);
          const bool next=epok::visibility_prefetch_next()&&
              epok::visibility_chunk(epok::visibility_next_mask(mesh->visibility_faces),mesh->quad_count);
          if(current||next)working_pages.consider(mesh->stream_page,current?0:1);
          continue;
        }
        Fixed local[3], camera[3];
        int64_t radius = 0;
        for (int c = 0; c < 3; ++c) {
          local[c] = Fixed(mesh->origin[c] + mesh->center[c], Fixed::RAW);
          const int64_t extent = mesh->extent[c] < 0 ? -int64_t(mesh->extent[c]) : int64_t(mesh->extent[c]);
          if (extent > radius) radius = extent;
        }
        object_view.point(local, camera);
        // A doubled sphere is conservative for rotated chunk bounds.
        radius = radius * 2 + 4096;
        const int64_t x = camera[0].raw(), y = camera[1].raw(), z = camera[2].raw();
        const int64_t far = z + radius;
        if (far < 1024 || z - radius >= int64_t(epok::geometry_draw_distance) * 4096 ||
            x - radius > far || x + radius < -far ||
            (y - radius) * 4 > far * 3 || (y + radius) * 4 < -far * 3) continue;
        const int64_t ax = x < 0 ? -x : x, ay = y < 0 ? -y : y;
        int64_t score = (z > 0 ? z : 0) + (ax + ay) / 2;
        if(epok::active_visibility_track&&score<2)score=2; // Baked current/next pages have priorities 0/1.
        if (score > UINT32_MAX) score = UINT32_MAX;
        working_pages.consider(mesh->stream_page, uint32_t(score));
      }
    }
#if !EPOK_HOST_DATA
    if constexpr(epok::streaming_prefetch_enabled) {
      // Spare-slot lookahead never displaces current/next geometry or an
      // untracked actor selected above. CD + sliced integrity verification
      // needs more lead than one 100 ms camera sample. No rail branch guesses.
      epok::visibility_visit_lookahead_pages(1500000,[&](uint32_t page,uint32_t distance){
        if(!working_pages.contains(page)&&working_pages.count()<working_slots)
          working_pages.consider(page,distance);
      });
    }
#endif
    epok::streaming_begin_working_set(working_pages, working_pins);
  }
  epok::StreamPageCursor stream_cursor;
  if constexpr(epok::ordered_draw_enabled)if(track&&track->ordered){
#ifdef EPOK_SWEEP_CAMERAS
    // Diagnostic: the ordered pass of a listed sample uses that table camera.
    if(const auto* table=epok::sweep_camera(epok::visibility_page_selection.current)){
      epok::CameraSpace<Fixed> sweep_view;
      for(unsigned r=0;r<3;++r){
        for(unsigned c=0;c<3;++c)sweep_view.basis.values[r][c]=Fixed(int32_t(table->basis[r*3+c]),Fixed::RAW);
        sweep_view.basis.values[r][3]=Fixed(0,Fixed::RAW);
        sweep_view.origin[r]=Fixed(table->origin[r],Fixed::RAW);
      }
      const Fixed sweep_scale[2]={Fixed(table->row_scale[0],Fixed::RAW),Fixed(table->row_scale[1],Fixed::RAW)};
      draw_ordered_world(emitter,sweep_view,sweep_scale);
    }else
#endif
    draw_ordered_world(emitter,view,row_scale);
  }
  StaticWorldView static_world;
  for (size_t index = 0; index < epok::object_count; ++index) {
    const auto &object = epok::objects[index];
    if (!object.mesh || !epok::is_active_slot(index))
      continue;
    // Query copies remain available, but may never be accidentally rendered
    // while their prepared program is waiting for a checkpoint/page barrier.
    if constexpr(epok::ordered_draw_enabled)if(track&&track->ordered
        &&index<track->ordered->actor_count&&track->ordered->actors[index])continue;
    emitter.table=object.lighting.background_pass?&background_ordering[parity]:&table;
    background_active|=object.lighting.background_pass;
    if constexpr(epok::baked_draw_enabled) if(object.geometry&&(object.geometry->editable&epok::mesh_baked_draw)){
      bool visible=false;uint32_t chunks=0;
      if(object.geometry->visibility_words){
        chunks=object.geometry->visibility_chunks;
        visible=epok::visibility_chunk(object.geometry->visibility_faces,size_t(object.geometry->visibility_words)*32);
      }else for(auto* mesh=object.geometry;mesh;mesh=mesh->next){
        ++chunks;if(epok::visibility_chunk(mesh->visibility_faces,mesh->quad_count)){visible=true;break;}
      }
      if(!visible||!epok::visibility_track_started){performance_work.visibility_skipped_chunks+=chunks;continue;}
      if(object.geometry->editable&epok::mesh_static_world){
        // Cooked in world space: the actor's runtime transform is not applied.
        if(!static_world.valid)static_world=static_world_view(view,row_scale);
        draw_baked_object(object,static_world.matrix,emitter,stream_cursor,working_pages,gpu(),projected_vertices_ram,&static_world);
        continue;
      }
      auto object_view=view.compose(render_world[index]);
      for(int r=0;r<2;++r)for(int c=0;c<4;++c)object_view.values[r][c]*=row_scale[r];
      draw_baked_object(object,object_view,emitter,stream_cursor,working_pages,gpu(),projected_vertices_ram);
      continue;
    }
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
    const epok::MeshGeometry* const geometry_root = skeletal ? object.animator.model->geometry : (object.geometry ? object.geometry : &default_cube);
    if(geometry_root->visibility_faces){
      bool visible=false;uint32_t chunks=0;
      for(auto* mesh=geometry_root;mesh;mesh=mesh->next){++chunks;visible|=epok::visibility_chunk(mesh->visibility_faces,mesh->quad_count);}
      if(!visible||!epok::visibility_track_started){performance_work.visibility_skipped_chunks+=chunks;continue;}
    }
    auto object_view=view.compose(render_world[index]);
    for(int r=0;r<2;++r)for(int c=0;c<4;++c)object_view.values[r][c]*=row_scale[r];
    MaterialState material_state;
    const bool object_lit = !object.material.unlit && object.lighting.enabled &&
        !(skeletal && mesh_material_cache[index].faces_unlit(geometry_root));
    if constexpr (epok::stream_archive_fits) if (!skeletal &&
        !epok::streaming_bind_object(geometry_root, stream_bindings[index], gpu())) {
      // Binding is atomic at object granularity: never draw a partial chain.
      for (auto* mesh = geometry_root; mesh; mesh = mesh->next) {
        ++performance_work.stream_failed_chunks;
        epok::lighting_work.dropped_triangles += uint32_t(mesh->quad_count) * 2;
      }
      continue;
    }
    // Unlit animated meshes also have immutable UV/material packet words.
    // Only projected coordinates change with a pose. Lit/baked-colour skins
    // stay on the dynamic path because their corner colours may change.
    bool object_retained = false, retained_checked = false, object_any_dynamic = false, stream_metadata_validated = false;
    epok::RetainedQuad* object_records = nullptr;
    uint32_t object_first_slot = 0, quad_base = 0;
    if constexpr (epok::retained_geometry) {
      // Sparse baked selections allocate packets only for selected faces.
      // Whole-object retained slots would reserve invisible faces and starve
      // later chunks even though the visible triangle count fits the budget.
      if (!geometry_root->visibility_faces && epok::retain_mesh_packets(skeletal,object_lit,use_bake)) {
        const auto& state = retained.state(index);
        bool allocated = state.allocated && state.geometry == geometry_root;
        if (!allocated) {
          size_t total_quads = 0;
          for (auto* m = geometry_root; m; m = m->next) total_quads += m->quad_count;
          allocated = retained.allocate(index, geometry_root, total_quads, capacity);
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
            const auto normal_vertices=epok::skeletal_detail::scratch.geometry.vertices;
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
      if constexpr (epok::vertex_colors_enabled) if (quad.vertex_colors) {
        for(int v=0;v<4;++v)base[v]=epok::tint_color(epok::vertex_color_palette[quad.vertex_color_indices[v]],base[v]);
        uniform=false;
      }
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
        rec.gpu_colors = epok::vertex_colors_enabled && quad.vertex_colors;
        rec.dynamic = textured && (draw_material.uv_scroll[0] || draw_material.uv_scroll[1]);
        rec.fogged &= uint8_t(~(1u << build_parity));
        for (int v = 0; v < 4; ++v) rec.shaded[v] = base[v];
        if (rec.dynamic) { state.any_dynamic = true; chunk_dynamic = true; continue; }
        const uint32_t slot = object_first_slot + global_quad * 2;
        uint32_t final[4], uv[4] = {0, 0, 0, 0};
        for (int v = 0; v < 4; ++v) final[v] = epok::packet_color(base[v], textured, rec.gpu_colors);
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
      if (mesh.quad_count) {
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
        // lookup already adds 256. Scale that per-axis rounding allowance
        // with the plane, including non-4:3 and custom drawing areas.
        const int64_t rounding_guard=int64_t(256)*(z_scale+a_scale-1);
        visibility.add_plane(int64_t(view_rows[2][0])*z_scale + int64_t(view_rows[a][0])*sign*a_scale,
                             int64_t(view_rows[2][1])*z_scale + int64_t(view_rows[a][1])*sign*a_scale,
                             int64_t(view_rows[2][2])*z_scale + int64_t(view_rows[a][2])*sign*a_scale,
                             int64_t(view_t[2])*z_scale + int64_t(view_t[a])*sign*a_scale + offset + rounding_guard);
      };
      // Conservative planes of the actual chunk. Keep near at
      // zero and omit far; exact near/far and hardware clipping still run below.
      add_plane(0, 0, 1, 0, 0);
      add_plane(0, 1, RenderFrustum::xz, RenderFrustum::xx, 0); add_plane(0, -1, RenderFrustum::xz, RenderFrustum::xx, 0);
      add_plane(1, 1, RenderFrustum::yz, RenderFrustum::yy, 0); add_plane(1, -1, RenderFrustum::yz, RenderFrustum::yy, 0);
    }
    const auto visibility_mask = visibility.merged();
    const bool bounds_cache_active = visibility.bounds_cache_active();
    const bool basis_cache_active = visibility.basis_cache_active();
    size_t chunk_ordinal = 0;
    const bool object_cached = object_retained;
    uint32_t prefetch_source = 0xffffffffu;
    bool skeletal_pose_ready=false,baked_vertices_ready=false;
    for (auto *mesh_ptr = geometry_root;
         mesh_ptr; mesh_ptr = mesh_ptr->next, ++chunk_ordinal) {
      const auto &bounds_mesh = *mesh_ptr;
      const uint32_t chunk_quad_base = quad_base;
      quad_base += uint32_t(bounds_mesh.quad_count);
      if (!visibility_mask.candidate(chunk_ordinal) ||
          !epok::visibility_chunk(bounds_mesh.visibility_faces,bounds_mesh.quad_count) ||
          (bounds_mesh.visibility_faces&&!epok::visibility_track_started)) { ++performance_work.visibility_skipped_chunks; continue; }
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
        if (far_z < 1024 || center[2] - extent[2] >= epok::geometry_draw_distance * 4096 ||
            (int64_t(center[0])-extent[0])*RenderFrustum::xx>int64_t(far_z)*RenderFrustum::xz ||
            (int64_t(center[0])+extent[0])*RenderFrustum::xx<-int64_t(far_z)*RenderFrustum::xz ||
            (int64_t(center[1])-extent[1])*RenderFrustum::yy>int64_t(far_z)*RenderFrustum::yz ||
            (int64_t(center[1])+extent[1])*RenderFrustum::yy<-int64_t(far_z)*RenderFrustum::yz) {
          if (bounds_cache_active) visibility.remember_bounds(chunk_ordinal, false);
          EPOK_DETAIL_END(setup, setup_scanlines);
          continue;
        }
        if (bounds_cache_active) visibility.remember_bounds(chunk_ordinal, true);
        fully_inside_chunk = narrow_view && epok::chunk_fully_inside<RenderFrustum>(center, extent, projection_focal_pixels);
      }
      ++epok::mesh_stats.visible_chunks;
      if constexpr (demand_streaming) if (!epok::streaming_working_page_ready(
              working_pages, bounds_mesh.stream_page)) {
        ++performance_work.stream_budget_skipped_chunks;
        epok::lighting_work.dropped_triangles += uint32_t(bounds_mesh.quad_count) * 2;
        EPOK_DETAIL_END(setup, setup_scanlines);
        continue;
      }
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
      // A cache address is claimed only after visibility/residency succeeds.
      // If dynamic work already borrowed it, use the dynamic path for this
      // chunk; never overwrite a packet linked into the current ordering table.
      const bool object_retained = object_cached && frame_packet_slots.claim(
          object_first_slot + chunk_quad_base * 2, uint32_t(mesh.quad_count) * 2);
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
      if(cpu_rigid&&!skeletal_pose_ready){const uint16_t begin=COUNTERS[1].value;epok::skeletal_detail::scratch.pose(*object.animator.model,object.animator,object.aim_bone,object.aim_stop_bones,object.aim_pitch);performance_work.skeletal_scanlines+=uint16_t(COUNTERS[1].value-begin);performance_work.skeletal_bone_matrices+=object.animator.model->bone_count;performance_work.skeletal_cpu_vertices+=mesh.vertex_count;skeletal_pose_ready=true;}
      const auto mesh_vertices = (baked_vertices||cpu_rigid)?epok::skeletal_detail::scratch.geometry.vertices:mesh_view.vertices;
      const auto* mesh_quads = mesh_view.quads;
      uint32_t selected_vertices[16];
      const size_t selected_vertex_count=mesh.visibility_faces?
          epok::visibility_vertices(mesh.visibility_faces,mesh_quads,mesh.quad_count,selected_vertices):mesh.vertex_count;
      if constexpr (epok::stream_page_count > 0) {
#ifdef EPOK_PROFILE_DETAIL
        if (bounds_mesh.stream_page == 0xffffffffu) ++non_streamed_visible;
#endif
        // Whole-archive pools warm the active scene. Packet rebuilds remain
        // lazy so they never overwrite unclaimed, borrowed GPU slots.
        if constexpr (epok::streaming_prefetch_enabled && !demand_streaming)
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
      // Editable unlit faces do not consume the GTE light registers or the cube
      // face colours. Do not select lights and build matrices just to discard
      // them (especially for imported unlit characters with a default actor tint).
      if (!use_bake && !shaded && (object_lit || !mesh.editable)) {
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
        for (int r = 0; r < 3; ++r) for (int c = 0; c < 3; ++c) key.basis[r * 3 + c] = skeletal ? 0 : world[index].values[r][c].raw();
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
          // Invalidate without loading/writing invisible chunks. Resident
          // geometry must also rebuild lazily: its slots may be borrowed.
          const uint8_t keep = uint8_t(~(1u << parity));
          for (uint32_t q = 0; q < state.quad_count; ++q) object_records[q].valid_parities &= keep;
        }
        object_any_dynamic = retained.state(index).any_dynamic;
        ++epok::retained_stats.objects;
      }
      if (object_retained && mesh.quad_count) {
        if (!(object_records[chunk_quad_base].valid_parities & (1u << parity)))
          build_retained_chunk(parity, mesh, mesh_quads, chunk_quad_base);
        object_any_dynamic = object_records[chunk_quad_base].chunk_dynamic;
      }
      EPOK_DETAIL_END(setup, setup_scanlines);
      const uint16_t vertex_begin=COUNTERS[1].value;
      ProjectedVertex* const projected_vertices =
          mesh.vertex_count <= scratch_vertex_capacity ? projection_scratch.full : projected_vertices_ram;
      auto finish_projection=[&](ProjectedVertex& out,uint32_t flags){
        const int32_t z = out.camera[2];
        out.outcode = Units::outcode(out.camera[0], out.camera[1], z);
        out.fog = fog_enabled ? epok::fog_amount(z, fog_start, fog_end) : 0;
        out.visible = z >= Units::near && z < Units::far;
        if (out.visible && (flags & epok::gte_projection_flags))
          epok::project_cpu<Units, epok::display_width, epok::display_height>(out);
      };
      bool gte_geometry=true;
      bool skeletal_triples=skeletal && fully_inside_chunk && epok::display_width<=1023 && epok::display_height<=511;
      if(skeletal_triples&&object_retained)skeletal_triples=!object_any_dynamic;
      else if(skeletal_triples)for(size_t q=0;q<mesh.quad_count;++q)
        if((mesh.editable?mesh_quads[q].material:object.material).uv_scroll[0]||(mesh.editable?mesh_quads[q].material:object.material).uv_scroll[1]){skeletal_triples=false;break;}
      const bool compact_eligible=fully_inside_chunk&&object_retained&&!object_any_dynamic&&!fog_enabled&&
          epok::display_width<=1023&&epok::display_height<=511;
      auto* compact=mesh.vertex_count<=128?projection_scratch.compact:projection_ram.compact;
      auto draw_compact=[&](){
        performance_work.gte_vertices+=mesh.vertex_count;epok::mesh_stats.transformed_vertices+=mesh.vertex_count;
        performance_work.vertex_scanlines+=uint16_t(COUNTERS[1].value-vertex_begin);
        epok::debug_hud::geometry(vertex_begin,true);
        const uint16_t begin=COUNTERS[1].value;emitter.editable=mesh.editable&&(!mesh.visibility_faces||epok::active_visibility_track->cull_backfaces);
        emitter.retained_interior(mesh_quads,object_records+chunk_quad_base,mesh.quad_count,
            object_first_slot+chunk_quad_base*2,compact,uint8_t(1u<<parity),mesh.visibility_faces);
        performance_work.polygon_scanlines+=uint16_t(COUNTERS[1].value-begin);
      };
      if(rigid_gte){
        if(!skeletal_pose_ready){const uint16_t begin=COUNTERS[1].value;epok::skeletal_detail::scratch.pose_bones(*object.animator.model,object.animator,object.aim_bone,object.aim_stop_bones,object.aim_pitch);performance_work.skeletal_scanlines+=uint16_t(COUNTERS[1].value-begin);performance_work.skeletal_bone_matrices+=object.animator.model->bone_count;skeletal_pose_ready=true;}
        const int32_t origin8[3]={0,0,0};
        if(compact_eligible){
          bool valid=true;
          for(size_t bone=0;bone<object.animator.model->bone_count;++bone){
            const size_t first=object.animator.model->bone_vertices[bone],last=object.animator.model->bone_vertices[bone+1];
            if(first==last)continue;
            const auto bone_view=object_view.compose(epok::skeletal_detail::scratch.bones[bone]);
            if(!epok::load_projection_matrix(bone_view)||!project_compact_vertices(mesh_vertices+first,last-first,origin8,compact+first)){valid=false;break;}
          }
          object_matrix_loaded=false;
          if(valid){draw_compact();continue;}
        }
        size_t gte_count=0,software_count=0;
        // Interior non-scrolling faces consume screen/depth only. Boundary,
        // overflow and UV-scroll paths retain full camera coordinates.
        for(size_t bone=0;bone<object.animator.model->bone_count;++bone){
          const size_t first=object.animator.model->bone_vertices[bone];
          const size_t last=object.animator.model->bone_vertices[bone+1];
          if(first==last)continue;
          Matrix bone_view=object_view.compose(epok::skeletal_detail::scratch.bones[bone]);
          const bool bone_gte=epok::load_projection_matrix(bone_view);
          size_t v=first;
          if(bone_gte && skeletal_triples)for(;v+2<last;v+=3){
            uint32_t screens[3];int32_t depths[3];
            if(epok::project_geometry_triple(mesh_vertices+v,origin8,screens,depths)){
              for(unsigned i=0;i<3;++i){
                auto& out=projected_vertices[v+i];
                out.camera[0]=out.camera[1]=0;out.camera[2]=depths[i];out.screen.packed=screens[i];
                out.outcode=0;out.visible=true;out.fog=fog_enabled?epok::fog_amount(depths[i],fog_start,fog_end):0;
#ifdef EPOK_VALIDATE_GTE
                ProjectedVertex reference{};uint32_t flags;
                epok::project_geometry_vertex(mesh_vertices[v+i],origin8,reference.camera,reference.screen.packed,flags);
                finish_projection(reference,flags);
                if(reference.screen.packed!=out.screen.packed||reference.camera[2]!=out.camera[2]||reference.outcode||!reference.visible||reference.fog!=out.fog)
                  {++performance_work.gte_validation_errors;++gte_validation_diagnostic[3];}
#endif
              }
            }else for(unsigned i=0;i<3;++i){
              auto& out=projected_vertices[v+i];uint32_t flags;
              epok::project_geometry_vertex(mesh_vertices[v+i],origin8,out.camera,out.screen.packed,flags);finish_projection(out,flags);
            }
          }
          for(;v<last;++v){
            auto& out=projected_vertices[v];uint32_t flags=epok::gte_projection_flags;
            if(bone_gte)epok::project_geometry_vertex(mesh_vertices[v],origin8,out.camera,out.screen.packed,flags);
            else{Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);bone_view.point(local,p);for(int c=0;c<3;++c)out.camera[c]=p[c].raw()>>4;}
#ifdef EPOK_VALIDATE_GTE
            if(bone_gte){
              Fixed local[3],p[3];for(int c=0;c<3;++c)local[c]=Fixed(mesh_vertices[v][c],Fixed::RAW);bone_view.point(local,p);
              for(int c=0;c<3;++c){const int64_t difference=int64_t(out.camera[c])*16-p[c].raw();const uint32_t delta=uint32_t(difference<0?-difference:difference);if(delta>performance_work.gte_max_delta)performance_work.gte_max_delta=delta;if(delta>128){++performance_work.gte_validation_errors;++gte_validation_diagnostic[5];}}
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
        if(mesh_vertices.wide()&&!epok::gte_vertex_bounds_fit(origin8,mesh.center,mesh.extent))shared_matrix=false;
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
          if(mesh_vertices.wide()&&!epok::gte_vertex_bounds_fit(origin8,mesh.center,mesh.extent))gte_geometry=false;
          object_matrix_loaded = false;
        }
        if(gte_geometry&&compact_eligible&&project_compact_vertices(mesh_vertices,mesh.vertex_count,origin8,compact)){
          draw_compact();continue;
        }
        size_t v = 0;
        if (gte_geometry && fully_inside_chunk && ((object_retained && !object_any_dynamic) || skeletal_triples)) {
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
                  {++performance_work.gte_validation_errors;++gte_validation_diagnostic[4];}
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
          if(mesh.visibility_faces&&!epok::visibility_face(selected_vertices,v))continue;
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
            if(delta>128){++performance_work.gte_validation_errors;++gte_validation_diagnostic[5];}
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
            // The CPU fallback's conservative guard band excludes -1024;
            // the GTE can produce -1024 without a saturation flag. For these
            // already depth-valid, flag-free points, compare coordinates, not
            // the fallback's guard-band policy. Frustum/triangle clipping is
            // still performed by the normal renderer below.
            if (delta > 2) {
              ++performance_work.gte_validation_errors;++gte_validation_diagnostic[6];
              for(int c=0;c<3;++c)gte_validation_diagnostic[8+c]=uint32_t(out.camera[c]);gte_validation_diagnostic[11]=flags;
            }
          }
          }
#endif
          finish_projection(out,flags);
        }
        if(gte_geometry)performance_work.gte_vertices+=selected_vertex_count;else performance_work.software_vertices+=selected_vertex_count;
      }
      epok::mesh_stats.transformed_vertices += selected_vertex_count;
      performance_work.vertex_scanlines+=uint16_t(COUNTERS[1].value-vertex_begin);
      epok::debug_hud::geometry(vertex_begin, gte_geometry);
      const uint16_t polygon_begin=COUNTERS[1].value;
      emitter.editable = mesh.editable&&(!mesh.visibility_faces||epok::active_visibility_track->cull_backfaces);
      if (object_retained && fully_inside_chunk && !object_any_dynamic && !fog_enabled && epok::display_width<=1023 && epok::display_height<=511) {
        emitter.retained_interior(mesh_quads,object_records+chunk_quad_base,mesh.quad_count,
            object_first_slot+chunk_quad_base*2,projected_vertices,uint8_t(1u<<parity),mesh.visibility_faces);
      } else if (object_retained && !fog_enabled) {
        emitter.retained_boundary(mesh_quads,object_records+chunk_quad_base,mesh.quad_count,
            object_first_slot+chunk_quad_base*2,projected_vertices,uint8_t(1u<<parity),object.material,mesh.visibility_faces);
      } else if (object_retained) {
        epok::RetainedQuad* records = object_records + chunk_quad_base;
        const uint32_t slot_base = object_first_slot + chunk_quad_base * 2;
        const uint8_t parity_bit = uint8_t(1u << parity);
        for (size_t q = 0; q < mesh.quad_count; ++q) {
          if(!epok::visibility_face(mesh.visibility_faces,q))continue;
          auto& rec = records[q];
          if (rec.dynamic) continue;
          const auto& quad = mesh_quads[q];
          emitter.double_sided = quad.material.double_sided;
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
            const bool gpu_fog=rec.textured&&rec.gpu_colors;
            shaded_now[0] = epok::blend_vertex_fog(rec.shaded[0], p0.fog, fog_rgb,gpu_fog); shaded_now[1] = epok::blend_vertex_fog(rec.shaded[1], p1.fog, fog_rgb,gpu_fog);
            shaded_now[2] = epok::blend_vertex_fog(rec.shaded[2], p2.fog, fog_rgb,gpu_fog); shaded_now[3] = epok::blend_vertex_fog(rec.shaded[3], p3.fog, fog_rgb,gpu_fog);
            shaded = shaded_now;
            for (int v = 0; v < 4; ++v) colors[v] = epok::packet_color(shaded_now[v], rec.textured, rec.gpu_colors);
            recolor = colors; rec.fogged |= parity_bit;
          } else if (rec.fogged & parity_bit) {
            for (int v = 0; v < 4; ++v) colors[v] = epok::packet_color(rec.shaded[v], rec.textured, rec.gpu_colors);
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
            packet.gpu_colors = rec.gpu_colors;
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
        emitter.double_sided = quad.material.double_sided;
        if(!epok::visibility_face(mesh.visibility_faces,q)){color_offset+=4;continue;}
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
        packet.gpu_colors = epok::vertex_colors_enabled && quad.vertex_colors;
        packet.command = material_state.command; packet.clut16 = material_state.clut16; packet.tpage16 = material_state.tpage16;
        packet.depth_bias = draw_material.depth_bias;
        uint32_t base[4];
        // Final packet colour shared by the four corners when nothing varies.
        const bool uniform = quad_colors(mesh, quad, color_offset, packet.textured, base);
        const uint32_t packet_flat = flat_final;
        EPOK_DETAIL_END(shade, shade_scanlines);
        EPOK_DETAIL_BEGIN(fog);
        if (fog_enabled && (p0.fog | p1.fog | p2.fog | p3.fog)) {
          const bool gpu_fog=packet.textured&&packet.gpu_colors;
          base[0] = epok::blend_vertex_fog(base[0], p0.fog, fog_rgb,gpu_fog); base[1] = epok::blend_vertex_fog(base[1], p1.fog, fog_rgb,gpu_fog);
          base[2] = epok::blend_vertex_fog(base[2], p2.fog, fog_rgb,gpu_fog); base[3] = epok::blend_vertex_fog(base[3], p3.fog, fog_rgb,gpu_fog);
        }
        EPOK_DETAIL_END(fog, fog_scanlines);
        EPOK_DETAIL_BEGIN(emit);
        for (int v = 0; v < 4; ++v) packet.shaded[v] = base[v];
        if (packet.textured) {
          if (uniform) {
            packet.final_color[0] = packet.final_color[1] = packet.final_color[2] = packet.final_color[3] = packet_flat;
          } else {
            for (int v = 0; v < 4; ++v) packet.final_color[v] = epok::packet_color(base[v], true, packet.gpu_colors);
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
  // The ordered path otherwise reads the hardware timer four times per face.
  // Mark its uncollected breakdown unavailable only after all mesh passes have
  // accumulated, so an ordinary pass cannot wrap this sentinel back to a count.
  if constexpr(epok::ordered_draw_enabled)if(ordered_work.candidates){
    performance_work.vertex_scanlines = UINT32_MAX;
    performance_work.polygon_scanlines = UINT32_MAX;
  }
#endif
  performance_work.backfaces = epok::mesh_stats.backfaces;
  performance_work.clipped = epok::mesh_stats.clipped;
  performance_work.triangles = used;
  performance_work.retained_triangles = emitter.retained_emitted;
  if constexpr(epok::retained_geometry)
    retained.invalidate_borrowed(parity,frame_packet_slots.dynamic_low());
  performance_work.retained_rebuilds = epok::retained_stats.rebuilds;
  EPOK_DETAIL_BEGIN(sprite);
  stream_cursor.release();
  if constexpr (demand_streaming)
    epok::streaming_end_working_set(working_pages, working_pins);
  blobs.draw(parity, table, epok::objects, render_world, epok::object_count, view);
  sprites.begin(epok::dither_3d);
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
  if constexpr(epok::dither_3d) {
    auto& page=geometry_pages[parity];page.primitive.attr.setDithering(true);
    epok::configure_display_field<epok::display_interlaced>(page.primitive.attr);
    gpu().chain(page);
  }
  if(background_active)gpu().chain(background_ordering[parity]);
  gpu().chain(table);
  if constexpr(epok::dither_3d) {
    auto& page=hud_pages[parity];page.primitive.attr.setDithering(false);
    epok::configure_display_field<epok::display_interlaced>(page.primitive.attr);
    gpu().chain(page);
  }
#ifdef EPOK_WORLD2D
  // The 2D world draws over the 3D world and under the HUD.
  epok::world2d::renderer.draw(gpu());
#endif
  EPOK_DETAIL_END(sprite, sprite_scanlines);
  EPOK_DETAIL_BEGIN(hud);
  // D-pad focus moves before the HUD is compiled, so the highlighted element is
  // the one this frame's press edge selected.
  epok::hud_focus_update(epok::objects.data(), epok::object_count, epok::hud_focus_edges());
  // The dungeon minimap explores from the player and draws under the HUD.
  if constexpr(epok::dungeon_minimap_capacity>0) {
    int32_t point[3];
    minimap.draw(gpu(), epok::display_width, epok::display_height, minimap_player(point)?point:nullptr,
                 gpu().getFrameCount(),[](size_t index){return index<epok::object_count&&epok::is_active_slot(index);});
  }
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
  for(size_t i=0;i<epok::texture_count;++i)if(const auto* t=epok::texture(int(i))){++usage.textures;usage.vram_texture_words+=uint32_t(t->word_width)*t->height;usage.vram_palette_words+=epok::texture_colors(*t);}
  usage.active_texture_bytes=2*(usage.vram_texture_words+usage.vram_palette_words);usage.resident_texture_bytes=epok::resident_texture_bytes;
  usage.particles=epok::particle_stats.alive;usage.particle_peak=epok::particle_stats.peak;usage.particle_dropped=epok::particle_stats.dropped;
  usage.mesh_triangles=epok::lighting_stats.triangles;usage.sprite_triangles=epok::sprite_stats.triangles;
  usage.dropped_primitives=epok::lighting_stats.dropped_triangles+epok::sprite_stats.dropped+epok::hud_stats.dropped;
  usage.sprite_estimated_pixels=epok::sprite_stats.estimated_pixels;
  usage.sprite_overdraw_per_mille=uint64_t(usage.sprite_estimated_pixels)*1000/(epok::display_width*epok::display_height);
  usage.frame_scanlines=epok::lighting_stats.frame_scanlines;
  performance_work.frame_scanlines=uint16_t(COUNTERS[1].value-frame_begin);
  performance_work.finish_scanlines=uint16_t(COUNTERS[1].value-finish_begin);
  if constexpr(epok::ordered_draw_enabled){
    // Invalidate first, publish the frame identity LAST. An emulator snapshot
    // stopped between individual MIPS stores must not join torn counters.
    epok::ordered_draw_stats.frame=0;
    asm volatile("":::"memory");
    ordered_work.frame=0;epok::ordered_draw_stats=ordered_work;
    epok::performance_stats=performance_work;
    asm volatile("":::"memory");
    epok::ordered_draw_stats.frame=performance_work.frame;
  }else epok::performance_stats=performance_work;
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
  Fixed p[3];camera_view().point(world_point,p);p[0]*=projection_focal_x;p[1]*=projection_focal;
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
  for(auto& cache:mesh_material_cache)cache={};
  if constexpr (epok::stream_archive_fits) for (auto &binding : stream_bindings) binding = {};
  hud.invalidate();
  palettes.clear();
  lighting.clear();
  camera_override={};projection_focal=1.0;projection_focal_x=1.0;
  collision_world.clear();particles.clear();input.discard_edges();time=Time{};
#ifdef EPOK_WORLD2D
  world2d::service.reset();
#endif
}
void activate_texture_bank(const Texture* textures) {
  if(!textures){game.gpu().waitChainIdle();texture_assets=nullptr;pending_texture_bank=nullptr;return;}
  pending_texture_bank=textures;load_pending_texture_bank(game.gpu());
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
void raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask,const ActorData* ignore,bool triggers) {
  if(!queries||!results||!count)return;
  refresh_collisions();collision_world.raycast_batch(queries,results,count,mask,entity_index(ignore),triggers);
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
  const int index=entity_index(&entity);
  if(index<0||size_t(index)>=object_count||!model||!output)return false;
  refresh_world_for(size_t(index));
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
  if(entity.parent<0){
    for(int r=0;r<3;++r)entity.transform.position[r]+=result.displacement[r];
    return result;
  }
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
int main() {
  epok::serial_debug::capture();
  return game.run();
}
