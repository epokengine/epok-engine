#pragma once
#ifndef EPOK_NATIVE_REQUIREMENTS
#define EPOK_NATIVE_REQUIREMENTS(...)
#endif
#include "effects.hpp"
#include "transition.hpp"
#include "sprite_types.hpp"
#include "particle_types.hpp"
#include "playback_types.hpp"
#include "palette_types.hpp"
#include "display.hh"
#include "text.hpp"
#include "input.hpp"
#include "memory_card.hpp"
#include "time.hpp"
#include "collision.hpp"
#include "texture_types.hpp"
#include "resources.hpp"
#include "visibility.hpp"
#include "psyqo/fixed-point.hh"
#include <stddef.h>
#include <stdint.h>

// Host-only annotations consumed by Epok's Clang extractor. They are intentionally
// empty for the MIPS compiler, so reflection metadata never requires RTTI/exceptions.
#if defined(__clang__) && defined(EPOK_REFLECTION)
#define EPOK_CLASS(...) __attribute__((annotate("EPOK_CLASS:" #__VA_ARGS__)))
#define EPOK_PROPERTY(...) __attribute__((annotate("EPOK_PROPERTY:" #__VA_ARGS__)))
#define EPOK_FUNCTION(...) __attribute__((annotate("EPOK_FUNCTION:" #__VA_ARGS__)))
#define EPOK_FUNCTION_LIBRARY(...) __attribute__((annotate("EPOK_FUNCTION_LIBRARY:" #__VA_ARGS__)))
#define EPOK_VALUE(...) __attribute__((annotate("EPOK_VALUE:" #__VA_ARGS__)))
#else
#define EPOK_CLASS(...)
#define EPOK_PROPERTY(...)
#define EPOK_FUNCTION(...)
#define EPOK_FUNCTION_LIBRARY(...)
#define EPOK_VALUE(...)
#endif

namespace epok {
using Fixed = psyqo::FixedPoint<12>;
using Collider=ColliderT<Fixed>;
using Aabb=AabbT<Fixed>;
using SpatialHit=SpatialHitT<Fixed>;
using RaycastQuery=RaycastQueryT<Fixed>;
using MoveResult=MoveResultT<Fixed>;
// All script transforms are local to the object's parent.
struct Transform { Fixed position[3]; Fixed rotation[3]; Fixed scale[3]; };
struct CameraSettings {bool enabled=false;Fixed field_of_view=90.0;uint8_t sky_color[3]={33,40,52};};
inline Fixed projection_focal=1.0;
struct AudioSource {
    bool enabled=false; int clip=-1; Fixed volume=1.0,pitch=1.0; bool play_on_start=true;uint8_t priority=128;
    void play(); void stop(); bool is_playing() const;
};
// state: 0 unused, 1 initializing, 2 idle, 3 seeking, 4 playing, 5 stopping, 6 error.
struct MusicStats {uint32_t state=0,starts=0,ends=0,loops=0,errors=0;int clip=-1;uint32_t error_code=0;};
extern MusicStats music_stats;
struct Material { uint8_t color[3]; bool unlit; int texture=-1; BlendMode blend=BlendMode::Cutout; int16_t depth_bias=0;int32_t uv_scroll[2]={}; };
enum class LightType { Directional, Point };
enum class LightMode { Baked, Realtime, Mixed };
enum class ReceiveLighting { Baked, Realtime };
struct Light { bool enabled=false;LightType type=LightType::Directional;LightMode mode=LightMode::Realtime;uint8_t color[3]={255,255,255};Fixed intensity=0.8,range=8.0;int priority=0; };
struct MeshLighting { bool enabled=true;ReceiveLighting receive=ReceiveLighting::Realtime;bool static_geometry=false,cast_shadows=true;uint8_t subdivisions=1;bool background_pass=false; };
struct BlobShadow {bool enabled=false;Fixed radius=0.6,strength=0.25,distance=3.0;};
struct LightingEnvironment { Fixed ambient[3]={0.3,0.3,0.3};bool point_lights=true; };
extern LightingEnvironment lighting_environment;
// Scanline counters read PsyQo's existing timer; about 64 us resolution, CPU work only.
struct LightingStats { uint32_t active_lights=0,lit_objects=0,gte_normals=0,baked_vertices=0,triangles=0,dropped_triangles=0,blob_triangles=0,lighting_scanlines=0,frame_scanlines=0; };
extern LightingStats lighting_stats;
// Last completed frame, in the existing ~64 us scanline timer units. Simulation
// includes world/collision work; collision includes world synchronization.
struct PerformanceStats {
  uint32_t frame=0,frame_scanlines=0,simulation_scanlines=0,world_scanlines=0,
           collision_scanlines=0,render_scanlines=0,vertex_scanlines=0,
           polygon_scanlines=0,steps=0,world_syncs=0,local_matrices=0,
           world_matrices=0,collider_bounds=0,fog_scanlines=0,
           shade_scanlines=0,emit_scanlines=0,gte_vertices=0,
           software_vertices=0,gte_validation_errors=0,gte_max_delta=0,
           // Appended after the original layout: completed-frame mesh counters.
           tested_chunks=0,visible_chunks=0,backfaces=0,clipped=0,triangles=0,
           gte_screen_max_delta=0,
           // Frame work outside simulation and mesh rendering, plus the per-chunk
           // setup (detail builds only).
           prepare_scanlines=0,finish_scanlines=0,setup_scanlines=0,
           // Detail builds only: camera/palette/light preparation, sprites,
           // blobs and particles, and the HUD.
           camera_scanlines=0,sprite_scanlines=0,hud_scanlines=0,
           // Retained packets: triangles drawn from retained slots and rebuilds this frame.
           retained_triangles=0,retained_rebuilds=0,
           visibility_skipped_chunks=0,streamed_chunks=0,stream_failed_chunks=0,
           // Skeletal work occurs only after the animation envelope is visible.
           skeletal_scanlines=0,skeletal_bone_matrices=0,
           skeletal_cpu_vertices=0,skeletal_decoded_vertices=0;
};
extern PerformanceStats performance_stats;
struct SkeletalQueryStats {
    uint32_t calls=0,vertices=0,bones=0,decoded_bytes=0,failures=0;
};
struct Canvas { bool enabled=false; };
struct RectTransform {
    bool enabled=false;
    Fixed anchor_min[2]={0.5,0.5},anchor_max[2]={0.5,0.5},pivot[2]={0.5,0.5};
    Fixed position[2]={0.0,0.0},size[2]={100.0,32.0};
};
struct Image { bool enabled=false; uint8_t color[3]={51,102,166}; int texture=-1; uint16_t region[4]={}; uint16_t borders[4]={}; };
struct ProgressBar { bool enabled=false;Fixed value=0.75;uint8_t color[3]={64,217,77},background[3]={31,31,31}; };
struct MeshQuad {
  uint16_t indices[4];
  uint8_t face;
  int16_t normal[3] = {};
  Material material = {{255, 255, 255}, false};
  uint32_t color_offset = 0;
  int16_t uv[4][2]={{0,0},{4096,0},{4096,4096},{0,4096}};
  // Exporter-baked 8-bit page coordinates (u | v << 8) for the face texture.
  bool packed_uv=false;
  uint16_t uvw[4]={};
};
struct ChunkVisibility;
struct MeshGeometry {
  mutable const int16_t (*vertices)[3];
  size_t vertex_count;
  mutable const MeshQuad *quads;
  size_t quad_count;
  bool editable = false;
  int32_t origin[3] = {};
  int32_t center[3] = {};
  int32_t extent[3] = {2048, 2048, 2048};
  const MeshGeometry *next = nullptr;
  const ChunkVisibility* visibility = nullptr;
  // Bounds and topology counts stay resident. Payload offsets address one
  // immutable CD page; the renderer resolves and pins it only after culling.
  uint32_t stream_page = 0xffffffffu;
  uint16_t stream_vertex_offset = 0, stream_quad_offset = 0;
};
struct MeshStats {
  uint32_t tested_chunks = 0, visible_chunks = 0, transformed_vertices = 0,
           backfaces = 0, clipped = 0;
};
extern MeshStats mesh_stats;
struct BonePose {int16_t translation[3],rotation[4],scale[3];};
struct Bone {int16_t parent;BonePose bind;};
struct BoneTrack {const BonePose* poses;bool constant;};
struct VertexFrame {uint32_t offset;const uint32_t* seek;bool raw;};
struct AnimationClip {const BoneTrack* tracks;const VertexFrame* vertex_frames;const uint8_t* vertex_data;uint16_t frames;const char* name;};
enum class SkeletalStorage:uint8_t {CpuRigid,RigidGte,BakedVertices};
struct SkeletalMesh {const MeshGeometry* geometry;const uint8_t* vertex_bones;const uint16_t* bone_vertices;const uint16_t* portable_to_cooked;const Bone* bones;size_t bone_count;const AnimationClip* clips;size_t clip_count;SkeletalStorage storage;size_t portable_vertex_count=0;};

enum class PoseKind:uint32_t { Bind=0, Current=1 };
enum class CoordinateSpace:uint32_t { Model=0, World=1 };
enum class MeshDataState:uint32_t { Unavailable=0, Pending=1, Ready=2, Failed=3 };
enum class MeshVertexError:uint32_t {
    None=0,MissingGeometry=1,InvalidVertex=2,Pending=3,
    StreamFailed=4,InvalidCoordinateSpace=5,WorldUnavailable=6
};
enum class SkeletalError:uint32_t {
    None=0, MissingModel=1, InvalidVertex=2, InvalidBone=3,
    MissingPoseData=4, InvalidCoordinateSpace=5, WorldUnavailable=6
};
struct EPOK_VALUE(Id="85ef1110-f14f-4fe0-a02e-e92c895c29bd") VertexSample {
    bool success=false;
    SkeletalError error=SkeletalError::MissingModel;
    Fixed position[3]={};
    uint32_t sampled_frame=0;
};
struct EPOK_VALUE(Id="55740603-1504-4684-a984-eb168601f871") MeshVertexSample {
    bool success=false;
    MeshVertexError error=MeshVertexError::MissingGeometry;
    MeshDataState data_state=MeshDataState::Unavailable;
    Fixed position[3]={};
};
struct EPOK_VALUE(Id="c95e7523-ed82-4f5c-8700-e38573d25ac0") BoneSample {
    bool success=false;
    SkeletalError error=SkeletalError::MissingModel;
    Fixed basis_x[3]={},basis_y[3]={},basis_z[3]={},position[3]={};
    uint32_t sampled_frame=0;
    int32_t parent=-1;
};
struct EPOK_VALUE(Id="6ac7a61e-2b83-4264-b720-2e5a09cac329") VertexIndexBatch4 {
    uint32_t count=0,index0=0,index1=0,index2=0,index3=0;
};
struct EPOK_VALUE(Id="9d3678b0-a22e-4e30-9d65-68615882e39c") VertexSamples4 {
    uint32_t count=0,total=0;
    VertexSample sample0{},sample1{},sample2{},sample3{};
};
struct EPOK_VALUE(Id="34b2149d-75f4-4652-8c1b-d3cb9f4ce11c") SkeletalPlaybackState {
    bool valid=false,enabled=false,playing=false,looping=false;
    int32_t clip=-1;
    uint32_t ticks=0,sampled_frame=0;
};
struct EPOK_VALUE(Id="dd720731-b284-4889-bb65-ccf49db8733e") SpritePlaybackState {
    bool valid=false,enabled=false,playing=false,completed=false;
    uint32_t clip=0,frame=0,clip_count=0,pending_events=0,dropped_events=0;
};
struct EPOK_VALUE(Id="1266de44-df43-4b72-af35-2acbb1142d9f") ParticleEmitterState {
    bool valid=false,enabled=false,playing=false,continuous=false;
    uint32_t pending=0,max_particles=0;
};
struct EPOK_VALUE(Id="2aee8bea-55df-4392-8c65-32ac4353befa") PaletteAnimationState {
    bool valid=false,enabled=false,reverse=false;
    uint32_t texture=0,first=0,last=0,offset=0;
    Fixed speed=0.0;
};
struct EPOK_VALUE(Id="82cfcc57-9cff-4e54-b6e7-c75c7f34f070") MaterialSnapshot {
    bool valid=false,unlit=false;
    uint32_t red=0,green=0,blue=0,texture=0,blend=0;
    int32_t depth_bias=0;
    Fixed uv_x=0.0,uv_y=0.0;
};
struct EPOK_VALUE(Id="57fed44f-f01b-4b2d-a9ec-12ec21cfb96d") WorldAffineSample {
    bool success=false;
    Fixed basis_x[3]={},basis_y[3]={},basis_z[3]={},position[3]={};
};
struct Animator {
    bool enabled=false;const SkeletalMesh* model=nullptr;int clip=-1;uint32_t ticks=0;bool playing=true,looping=true;
    bool play(int index,bool loop=true){if(!model||index<0||size_t(index)>=model->clip_count)return false;clip=index;ticks=0;playing=true;looping=loop;return true;}
    void pause(){playing=false;}void resume(){playing=true;}void stop(){playing=false;ticks=0;}
    void advance(){if(!enabled||!playing||!model||clip<0||size_t(clip)>=model->clip_count)return;auto frames=model->clips[clip].frames;if(looping){ticks=(ticks+1)%(uint32_t(frames>1?frames-1:1)*2);}else if(ticks<uint32_t(frames-1)*2)++ticks;else playing=false;}
};
class Actor;
struct ActorData {
    bool camera; bool mesh; bool tiled; int parent; Transform transform; Material material;
    Canvas canvas;RectTransform rect;Image image;Text text;ProgressBar progress;char name[129]={};
    const MeshGeometry* geometry=nullptr;
    Animator animator;
    // Optional additive pitch applied at one skeletal branch. Stop bones cancel
    // the inherited pitch before branches such as the legs. This belongs to the
    // visual instance rather than Animator so animation playback ABI stays stable.
    int16_t aim_bone=-1;int16_t aim_stop_bones[2]={-1,-1};Fixed aim_pitch=0.0;
    Sprite sprite;SpriteAnimator sprite_animator;ParticleEmitter particle_emitter;
    PaletteAnimator palette_animator;
    Light light;MeshLighting lighting;BlobShadow blob_shadow;AudioSource audio;const uint8_t (*baked_colors)[3]=nullptr;size_t baked_color_count=0;
    bool alive=true,active=true;
    Collider collider;
    CameraSettings camera_settings;
    uint32_t generation=1;
    Actor* owner=nullptr;
    void set_name(const char* value){size_t i=0;if(value)for(;i<128 && value[i];++i)name[i]=value[i];name[i]=0;}
    template<class T>T& component();
    template<class T>T* get(){auto& c=component<T>();return c.enabled?&c:nullptr;}
    template<class T>T& add(){auto& c=component<T>();c.enabled=true;return c;}
    template<class T>void remove(){component<T>().enabled=false;}
};
template<>inline Transform& ActorData::component<Transform>(){return transform;}
template<>inline Collider& ActorData::component<Collider>(){return collider;}
template<>inline CameraSettings& ActorData::component<CameraSettings>(){return camera_settings;}
template<>inline BlobShadow& ActorData::component<BlobShadow>(){return blob_shadow;}
template<>inline Light& ActorData::component<Light>(){return light;}
template<>inline AudioSource& ActorData::component<AudioSource>(){return audio;}
template<>inline Animator& ActorData::component<Animator>(){return animator;}
template<>inline Sprite& ActorData::component<Sprite>(){return sprite;}
template<>inline SpriteAnimator& ActorData::component<SpriteAnimator>(){return sprite_animator;}
template<>inline PaletteAnimator& ActorData::component<PaletteAnimator>(){return palette_animator;}
template<>inline ParticleEmitter& ActorData::component<ParticleEmitter>(){return particle_emitter;}
template<>inline MeshLighting& ActorData::component<MeshLighting>(){return lighting;}
template<>inline Canvas& ActorData::component<Canvas>(){return canvas;}
template<>inline RectTransform& ActorData::component<RectTransform>(){return rect;}
template<>inline Image& ActorData::component<Image>(){return image;}
template<>inline Text& ActorData::component<Text>(){return text;}
template<>inline ProgressBar& ActorData::component<ProgressBar>(){return progress;}
template<>inline Transform* ActorData::get<Transform>(){return &transform;}
template<>inline Transform& ActorData::add<Transform>(){return transform;}
ActorData* find_actor_data(const char* name);
ActorData* allocate_actor_data(const char* name,ActorData* parent=nullptr);
struct DataHandle {
    uint16_t index=0xffff; uint32_t generation=0;
    ActorData* get() const;
    explicit operator bool() const {return get()!=nullptr;}
};
DataHandle handle(const ActorData* entity);
bool destroy_actor_data(ActorData* entity);
bool set_active(ActorData* entity,bool active);
bool is_active(const ActorData* entity);
// Same test for a slot index; skips the pointer validation of is_active.
bool is_active_slot(size_t index);
bool request_scene(size_t index);
bool request_scene(const char* name);
bool request_scene(size_t index,const TransitionOptions& options);
bool request_scene(const char* name,const TransitionOptions& options);
size_t current_scene();
bool scene_loading();
uint32_t scene_transition_count();
uint32_t scene_rejected_count();
bool scene_waiting();
bool set_active_camera(ActorData* camera);
DataHandle active_camera();
bool camera_project(const Fixed* world_point,Fixed* screen_xy);
void activate_texture_bank(const Texture* textures);
// Spatial queries use world coordinates. Ray displacement defines a finite segment.
SpatialHit raycast(const Fixed* origin,const Fixed* displacement,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=false);
// One synchronized world snapshot for independent queries. Apply gameplay
// mutations AFTER this call; results[i] belongs to queries[i]. No allocations.
void raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=false);
size_t overlap(const Aabb& box,DataHandle* output,size_t capacity,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=true);
SpatialHit query_ground(const ActorData& entity,Fixed distance,uint32_t mask=0xffffffffu);
inline DataHandle hit_entity(const SpatialHit& hit) { return hit.entity<0?DataHandle{}:DataHandle{uint16_t(hit.entity),hit.generation}; }
MoveResult move_and_slide(ActorData& entity,const Fixed* world_displacement,uint32_t mask=0xffffffffu);
// Call after an intentional teleport/cut to snap visual position history.
void reset_motion_interpolation();
bool collider_aabb(const ActorData& entity,Aabb& output);
VertexSample skeletal_sample_vertex(const ActorData* entity,uint32_t vertex,PoseKind pose,CoordinateSpace space);
VertexSamples4 skeletal_sample_vertices(const ActorData* entity,VertexIndexBatch4 indices,PoseKind pose,CoordinateSpace space);
BoneSample skeletal_sample_bone(const ActorData* entity,uint32_t bone,PoseKind pose,CoordinateSpace space);
bool skeletal_world_point(const ActorData& entity,const Fixed* model,Fixed* world);
MeshDataState mesh_geometry_state(const MeshGeometry* geometry);
bool request_mesh_geometry(const MeshGeometry* geometry);
MeshVertexSample sample_mesh_vertex(const ActorData* entity,uint32_t vertex,CoordinateSpace space);
WorldAffineSample gameplay_world_affine(const ActorData* entity);
void reset_runtime_services();
void remove_runtime_owner(size_t index);
#ifdef EPOK_EDITOR_PREVIEW
inline bool editor_preview_active=false;
#endif
#ifdef EPOK_BLUEPRINTS
inline uint32_t blueprint_scene_generation=1;
#endif

}
#include "effect_types.hpp"
#ifndef EPOK_INCLUDE_FROM_OBJECT_MODEL
#include "object_model.hpp"
#include "navigation_components.hpp"
#ifndef EPOK_INCLUDE_FROM_UTILITY
#include "utility.hpp"
#endif
#if !defined(EPOK_INCLUDE_FROM_GAMEPLAY_API) && !defined(EPOK_INCLUDE_FROM_SKELETAL) && !defined(EPOK_INCLUDE_FROM_BLUEPRINT_API) && !defined(EPOK_INCLUDE_FROM_BLUEPRINT_RUNTIME) && !defined(EPOK_INCLUDE_FROM_UTILITY)
#include "gameplay_api.hpp"
#endif
#endif
