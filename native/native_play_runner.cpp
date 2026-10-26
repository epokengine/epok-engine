// Native PC gameplay process. It executes the generated C++ scene and the same
// fixed-step lifecycle as the console runtime; hardware-owned services are
// replaced here by bounded host implementations. Binary stdout is protocol-only.
#include <cmath>
#include <cstdio>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
extern "C" __declspec(dllimport) unsigned long __stdcall SetErrorMode(unsigned long);
#else
#include <unistd.h>
#endif
#include "transition-config.hh"
#include "scene.hh"
#include "loading-image.hh"
#include "data_table_service.hpp"
#include "hud_commands.hpp"
#include "fonts.hh"
#include "hud-config.hh"
#include "hud_focus.hpp"
#include "native_play_protocol.h"
#include "transform_cache.hpp"
#include "track_camera_pose.hpp"
static uint32_t rumble_identities[4]={};

namespace epok {
MusicStats music_stats; PerformanceStats performance_stats; LightingEnvironment lighting_environment;
LightingStats lighting_stats; MeshStats mesh_stats;
AudioSource* music_active=nullptr; AudioSource* music_requested=nullptr;
struct AudioPlayback {bool playing=false;uint64_t token=0;};
static std::map<const AudioSource*,AudioPlayback> audio_playing;
static uint64_t next_audio_token=0;
struct HostAudioEvent{uint64_t source;uint32_t action;int32_t clip,volume,pitch;};
static std::vector<HostAudioEvent> audio_events;
static void audio_event(const AudioSource* source,uint32_t action){
    if(audio_events.size()<EPOK_NATIVE_PLAY_AUDIO_LIMIT)audio_events.push_back({audio_playing[source].token,action,source->clip,source->volume.raw(),source->pitch.raw()});
}
void AudioSource::play(){if(!enabled||clip<0)return;stop();audio_playing[this]={true,++next_audio_token};audio_event(this,1);}
void AudioSource::stop(){auto i=audio_playing.find(this);if(i!=audio_playing.end()&&i->second.playing){audio_event(this,0);i->second.playing=false;}}
bool AudioSource::is_playing() const {auto i=audio_playing.find(this);return i!=audio_playing.end()&&i->second.playing;}
void remove_runtime_owner(size_t index){objects[index].audio.stop();}
void activate_texture_bank(const Texture* textures){texture_assets=textures;}

}
#include "lifecycle.hpp"
#include "scene_lifecycle.hpp"
#ifdef EPOK_WORLD2D
#include "world2d_service.hpp"
#endif

namespace {
using epok::Fixed;
using Matrix=epok::Affine<Fixed>;
// MSVC otherwise omits these inline generated definitions when only project
// translation units reference their extern declarations.
const epok::ClassDescriptor* volatile native_class_table=epok::object_classes;
const size_t* volatile native_class_count=&epok::object_class_count;
epok::TransformCache<Fixed,epok::objects.size()> transform_cache;
Matrix world[epok::objects.size()];
epok::CollisionWorld<Fixed,epok::objects.size()> collision_world;
epok::DataHandle camera_override;

Fixed host_fixed(double value){return Fixed(int32_t(std::llround(value*4096.0)),Fixed::RAW);}
Matrix local_matrix(const epok::Transform& transform){
    Matrix out;
    for(int c=0;c<3;++c)out.values[c][c]=transform.scale[c];
    for(int axis=0;axis<3;++axis){
        const double radians=double(transform.rotation[axis].raw())*3.14159265358979323846/(180.0*4096.0);
        if(radians!=0.0)out.rotate_rows(axis,host_fixed(std::sin(radians)),host_fixed(std::cos(radians)));
    }
    for(int r=0;r<3;++r)out.values[r][3]=transform.position[r];
    return out;
}
Matrix inverse_local(const epok::Transform& transform){
    Matrix out=Matrix::identity();
    for(int r=0;r<3;++r)out.values[r][3]=-transform.position[r];
    for(int axis=2;axis>=0;--axis){
        const double radians=-double(transform.rotation[axis].raw())*3.14159265358979323846/(180.0*4096.0);
        if(radians!=0.0)out.rotate_rows(axis,host_fixed(std::sin(radians)),host_fixed(std::cos(radians)),4);
    }
    for(int r=0;r<3;++r)for(int c=0;c<4;++c)
        out.values[r][c]=transform.scale[r].raw()==0?Fixed(0,Fixed::RAW):out.values[r][c]/transform.scale[r];
    return out;
}
void refresh_world(){transform_cache.sync(epok::objects,world,epok::object_count,local_matrix);}
void refresh_collisions(){
    refresh_world();collision_world.begin_sync();
    for(size_t i=0;i<epok::object_count;++i){auto& object=epok::objects[i];
        if(object.collider.enabled&&epok::is_active_slot(i))
            collision_world.set(i,object.collider,world[i],true,object.generation);
    }
}
epok::ActorData* camera_entity(){
    auto valid=[](epok::ActorData* e){return e&&epok::is_active(e)&&(e->camera||e->camera_settings.enabled);};
    if(auto* e=camera_override.get();valid(e))return e;
    for(size_t i=0;i<epok::object_count;++i)if(valid(&epok::objects[i]))return &epok::objects[i];
    return nullptr;
}
void dispatch_trigger(const epok::TriggerEvent& event){
    const epok::DataHandle a{event.first,event.first_generation},b{event.second,event.second_generation};
    if(a.get()&&epok::is_active(a.get()))epok::dispatch_slot_trigger(a,b,event.phase);
    if(b.get()&&epok::is_active(b.get()))epok::dispatch_slot_trigger(b,a,event.phase);
}
}

namespace epok {
bool scene_loading(){return pending_scene>=0||transition.busy();}
void reset_runtime_services(){
    rumble.suspend();
    transform_cache.clear();collision_world.clear();camera_override={};
    input.discard_edges();time=Time{};world3d_visible=true;
    audio_playing.clear();audio_events.clear();music_active=music_requested=nullptr;
#ifdef EPOK_BLUEPRINTS
    ++blueprint_scene_generation;if(!blueprint_scene_generation)blueprint_scene_generation=1;
#endif
#ifdef EPOK_WORLD2D
    world2d::service.reset();
#endif
}
// Resident banks share the console lifecycle. The host must present Loading
// before retiring the old scene, even when both fade durations are zero.
static void scene_tick(uint32_t now){
    transition.advance(now);
    if(transition.phase==TransitionPhase::Idle&&pending_scene>=0)
        transition.begin(pending_transition.options,now);
    if(transition.phase==TransitionPhase::Loading&&transition.presented&&pending_scene>=0){
        scene_stop_audio();const auto next=scene_retire_bank();scene_start_bank(next);
        time.synchronize(now);transition.loaded(now);
    }
    scene_stats.waiting=scene_loading();
}
void reset_motion_interpolation(){}
// Native Play renders through the editor, without the PSX retained-packet cache.
// Report that no warmup was queued; normal drawing remains available.
bool prewarm_geometry(ActorData*){return false;}
bool set_active_camera(ActorData* camera){
    if(!camera){camera_override={};return true;}
    if(entity_index(camera)<0||!is_active(camera)||(!camera->camera&&!camera->camera_settings.enabled))return false;
    camera_override=handle(camera);return true;
}
DataHandle active_camera(){return handle(camera_entity());}
bool camera_project(const Fixed* point,Fixed* screen){
    if(!point||!screen)return false;auto* camera=camera_entity();if(!camera)return false;
    refresh_world();const int index=entity_index(camera);if(index<0)return false;
    auto view=Matrix::identity();int current=index;unsigned depth=0;
    while(current>=0&&size_t(current)<object_count&&depth++<33){view=view.compose(inverse_local(objects[current].transform));current=objects[current].parent;}
    Fixed p[3];view.point(point,p);if(p[2]<=Fixed(0.25))return false;
    int32_t fov=camera->camera_settings.field_of_view.raw();if(fov<25*4096)fov=25*4096;if(fov>120*4096)fov=120*4096;
    const double focal=1.0/std::tan(double(fov)*3.14159265358979323846/(2.0*180.0*4096.0));
    const int64_t x=int64_t(p[0].raw()*focal)*(camera->camera_settings.mirror_x?-1:1),y=int64_t(p[1].raw()*focal),z=p[2].raw();
    screen[0]=Fixed(int32_t(int64_t(display_width/2)*4096+x*(display_width/2)*4096/z),Fixed::RAW);
    screen[1]=Fixed(int32_t(int64_t(display_height/2)*4096-y*(display_height*2/3)*4096/z),Fixed::RAW);return true;
}
SpatialHit raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers){refresh_collisions();return collision_world.raycast(origin,delta,mask,entity_index(ignore),triggers);}
void raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask,const ActorData* ignore,bool triggers){if(!queries||!results)return;refresh_collisions();collision_world.raycast_batch(queries,results,count,mask,entity_index(ignore),triggers);}
size_t overlap(const Aabb& box,DataHandle* output,size_t capacity,uint32_t mask,const ActorData* ignore,bool triggers){refresh_collisions();uint16_t found[objects.size()];auto count=collision_world.overlap(box,found,objects.size(),mask,entity_index(ignore),triggers);if(output)for(size_t i=0;i<count&&i<capacity;++i)output[i]=handle(&objects[found[i]]);return count;}
bool collider_aabb(const ActorData& entity,Aabb& output){refresh_collisions();const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));if(!box)return false;output=*box;return true;}
SpatialHit query_ground(const ActorData& entity,Fixed distance,uint32_t mask){refresh_collisions();const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));return box?collision_world.ground(*box,distance,mask&entity.collider.mask,i):SpatialHit{};}
static MoveResult move_actor_synced(ActorData& entity,const Fixed* delta,uint32_t mask){
    const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));
    if(!box){MoveResult result;result.unresolved_overlap=true;return result;}
    auto result=collision_world.move_and_slide(*box,delta,mask&entity.collider.mask,i);
    if(entity.parent<0){for(int r=0;r<3;++r)entity.transform.position[r]+=result.displacement[r];return result;}
    Matrix inverse=Matrix::identity();int parent=entity.parent;unsigned depth=0;
    while(parent>=0&&size_t(parent)<object_count&&depth++<32){inverse=inverse.compose(inverse_local(objects[parent].transform));parent=objects[parent].parent;}
    if(parent>=0){result=MoveResult{};result.unresolved_overlap=true;return result;}
    for(int r=0;r<3;++r)for(int c=0;c<3;++c)entity.transform.position[r]+=inverse.values[r][c]*result.displacement[c];return result;
}
MoveResult move_and_slide(ActorData& entity,const Fixed* delta,uint32_t mask){
    refresh_collisions();return move_actor_synced(entity,delta,mask);
}
void SpatialQueryBatch::ensure() const {if(!ready){refresh_collisions();ready=true;}}
void SpatialQueryBatch::sync(ActorData& entity){
    if(!ready){ensure();return;}
    const int index=entity_index(&entity);if(index<0)return;
    if(entity.parent<0)transform_cache.sync_root(entity,world[index],size_t(index),::local_matrix);
    else if(!transform_cache.sync_path(objects,world,object_count,size_t(index),::local_matrix))refresh_world();
    collision_world.set(size_t(index),entity.collider,world[index],is_active(&entity),entity.generation);
}
SpatialHit SpatialQueryBatch::raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers) const {
    ensure();return collision_world.raycast(origin,delta,mask,entity_index(ignore),triggers);
}
void SpatialQueryBatch::raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask,const ActorData* ignore,bool triggers) const {
    if(!queries||!results||!count)return;ensure();
    collision_world.raycast_batch(queries,results,count,mask,entity_index(ignore),triggers);
}
MoveResult SpatialQueryBatch::move_and_slide(ActorData& entity,const Fixed* delta,uint32_t mask){
    sync(entity);auto result=move_actor_synced(entity,delta,mask);
    if(entity.parent<0){const int index=entity_index(&entity);if(index>=0)collision_world.translate(size_t(index),result.displacement);}
    else sync(entity);
    return result;
}
bool skeletal_world_point(const ActorData& entity,const Fixed* model,Fixed* output){const int i=entity_index(&entity);if(i<0||!model||!output)return false;refresh_world();world[i].point(model,output);return true;}
WorldAffineSample gameplay_world_affine(const ActorData* entity){WorldAffineSample result;const int i=entity_index(entity);if(i<0)return result;refresh_world();result.success=true;for(int r=0;r<3;++r){result.basis_x[r]=world[i].values[r][0];result.basis_y[r]=world[i].values[r][1];result.basis_z[r]=world[i].values[r][2];result.position[r]=world[i].values[r][3];}return result;}
}

class NativeCards final:public epok::MemoryCardDriver{
    std::function<void()> pending;std::map<std::string,std::vector<uint8_t>> files;
public:
    bool idle()const override{return !pending;}void pump(){if(pending){auto next=std::move(pending);pending={};next();}}
    void probe(unsigned,void* owner,Completion cb)override{pending=[=]{cb(owner,epok::CardError::OK);};}
    void read(unsigned port,const char* name,void* bytes,uint32_t capacity,uint32_t* size,void* owner,Completion cb)override{std::string key=std::to_string(port)+":"+name;pending=[=,this]{auto i=files.find(key);if(i==files.end()){cb(owner,epok::CardError::FileNotFound);return;}if(i->second.size()>capacity){cb(owner,epok::CardError::FileTooLarge);return;}*size=uint32_t(i->second.size());memcpy(bytes,i->second.data(),*size);cb(owner,epok::CardError::OK);};}
    void write(unsigned port,const char* name,const char*,const epok::CardIcon&,const void* bytes,uint32_t size,void* owner,Completion cb)override{std::string key=std::to_string(port)+":"+name;auto p=static_cast<const uint8_t*>(bytes);std::vector<uint8_t> copy(p,p+size);pending=[=,this]{files[key]=copy;cb(owner,epok::CardError::OK);};}
    void list(unsigned,epok::CardFile*,uint32_t* count,void* owner,Completion cb)override{pending=[=]{*count=0;cb(owner,epok::CardError::OK);};}
};
static FILE* frames=nullptr;
static bool write32(uint32_t value){uint8_t b[4];for(unsigned i=0;i<4;++i)b[i]=uint8_t(value>>(i*8));return fwrite(b,1,4,frames)==4;}
static bool read32(uint32_t& value){uint8_t b[4];if(fread(b,1,4,stdin)!=4)return false;value=0;for(unsigned i=0;i<4;++i)value|=uint32_t(b[i])<<(i*8);return true;}
static void advance_camera_track(uint32_t elapsed){
    const auto* track=epok::active_visibility_track;if(!track)return;
    const auto transitions=epok::rail_navigation_state.transitions;
    const bool active=!track->navigation||epok::is_active_slot(track->navigation->target);
    if(!active||epok::time.paused())elapsed=0;
    int32_t target[3]{};
    if(track->navigation)for(unsigned c=0;c<3;++c)target[c]=epok::objects[track->navigation->target].transform.position[c].raw();
    auto camera=[&](const auto& a,const auto& b,uint32_t fraction){
        auto pose=epok::interpolate_track_camera(a,b,fraction);
        epok::aim_track_camera(pose,elapsed,target,transitions);
        auto& actor=epok::objects[track->camera];actor.camera_settings.field_of_view=Fixed(pose.fov,Fixed::RAW);
        for(unsigned c=0;c<3;++c){actor.transform.position[c]=Fixed(pose.position[c],Fixed::RAW);actor.transform.rotation[c]=Fixed(pose.rotation[c],Fixed::RAW);}
    };
    // Native gameplay uses the same rail/aim controller with all pages ready.
    // This does not enable streaming or visibility filtering in Scene View.
    auto ready=[](bool){return true;};
    if(track->navigation)epok::advance_rail_navigation(elapsed,target,ready,camera);
    else epok::advance_visibility_track(elapsed,ready,camera);
}
static void frame(){
    EpokHudSink sink;for(size_t i=0;i<epok::texture_count;++i){sink.dimensions.push_back(epok::texture_assets[i].width);sink.dimensions.push_back(epok::texture_assets[i].height);}
    // Cooked fonts are global and already carry their VRAM placement; the
    // layout core only reads their metrics, exactly as the console does.
    sink.fonts.assign(epok::font_assets,epok::font_assets+epok::font_count);
#ifdef EPOK_WORLD2D
    // The 2D world draws under the HUD, through the same command rasterizer: the
    // background of the active Camera2D when no 3D camera paints one, then every
    // resolved sprite as a textured quad (texel corners are inclusive).
    {
        static epok::world2d::SpriteDraw draws[EPOK_WORLD2D_DRAW_ITEMS];
        static epok::world2d::DrawItem items[EPOK_WORLD2D_DRAW_ITEMS],scratch[EPOK_WORLD2D_DRAW_ITEMS];
        if(!camera_entity())if(const uint8_t* background=epok::world2d::service.background())sink.rectangle(-1,0,0,epok::display_width,epok::display_height,background);
        // Corners A, B, C, D; a transpose swaps the texture axes (B and C exchange texels).
        auto quad=[&](int texture,const int16_t* sx,const int16_t* sy,int su,int sv,int sw,int sh,bool flip_h,bool flip_v,bool transpose,const uint8_t* color){
            int x[4],y[4];for(int k=0;k<4;++k){x[k]=sx[k];y[k]=sy[k];}
            const int u0=su,u1=su+sw-1,v0=sv,v1=sv+sh-1;
            const int ul=flip_h?u1:u0,ur=flip_h?u0:u1,vt=flip_v?v1:v0,vb=flip_v?v0:v1;
            const int u[4]={ul,transpose?ul:ur,transpose?ur:ul,ur},v[4]={vt,transpose?vb:vt,transpose?vt:vb,vb};
            sink.textured_quad(-1,texture,x,y,u,v,color);
        };
        // One view: collect through the current render pass and emit its commands.
        auto view=[&]{
            const size_t count=epok::world2d::collect_sprites(draws,items,scratch,EPOK_WORLD2D_DRAW_ITEMS,[](int index,uint16_t& w,uint16_t& h){
                if(index<0||size_t(index)>=epok::texture_count)return false;w=epok::texture_assets[index].width;h=epok::texture_assets[index].height;return w>0&&h>0;});
            for(size_t i=0;i<count;++i){
                const auto& d=draws[items[i].index];
                using epok::world2d::DrawKind;
                if(d.kind==DrawKind::Line){
                    // Flat quads: each segment takes the colour at its middle.
                    epok::world2d::each_line_quad(d,[&](const epok::world2d::ShapeQuad& q){int x[4],y[4];uint8_t c[3];for(int k=0;k<4;++k){x[k]=q.x[k];y[k]=q.y[k];}for(int k=0;k<3;++k)c[k]=uint8_t((q.color[0][k]+q.color[1][k]+1)/2);sink.quad(-1,x,y,c);});
                    continue;
                }
                if(d.kind==DrawKind::Polygon){
                    const auto& shape=*static_cast<const epok::Polygon2DComponent*>(d.component);uint8_t c[3];for(int k=0;k<3;++k)c[k]=uint8_t((unsigned(shape.color[k])*d.color[k]+127)/255);
                    const bool textured=shape.texels&&d.texture>=0&&size_t(d.texture)<epok::texture_count;
                    // A triangle is a quad whose last corner repeats the third.
                    epok::world2d::each_polygon_triangle(d,[&](const epok::world2d::ShapeTriangle& t){int x[4]={t.x[0],t.x[1],t.x[2],t.x[2]},y[4]={t.y[0],t.y[1],t.y[2],t.y[2]};
                        if(textured){int u[4]={t.u[0],t.u[1],t.u[2],t.u[2]},v[4]={t.v[0],t.v[1],t.v[2],t.v[2]};sink.textured_quad(-1,int(d.texture),x,y,u,v,c);}else sink.quad(-1,x,y,c);});
                    continue;
                }
                if(d.texture==-2){
                    // A SubViewport2D's picture, drawn earlier this frame.
                    const auto* picture=static_cast<const epok::SubViewport2DComponent*>(d.component);
                    if(picture)quad(0x10000+int(picture->native_index),d.x,d.y,d.source[0],d.source[1],d.source[2],d.source[3],d.flip_h,d.flip_v,false,d.color);
                    continue;
                }
                if(d.texture<0||size_t(d.texture)>=epok::texture_count)continue;
                const auto& t=epok::texture_assets[d.texture];
                if(d.kind==DrawKind::Tiles){
                    epok::world2d::each_tile(d,t.width,t.height,[&](const epok::world2d::TileDraw& tile){quad(int(d.texture),tile.x,tile.y,tile.u,tile.v,tile.w,tile.h,tile.flip_h,tile.flip_v,tile.transpose,d.color);});
                    continue;
                }
                if(d.kind==DrawKind::Particles){
                    epok::world2d::each_particle(d,t.width,t.height,[&](const epok::world2d::SpriteDraw& p){quad(int(d.texture),p.x,p.y,p.source[0],p.source[1],p.source[2],p.source[3],false,false,false,p.color);});
                    continue;
                }
                quad(int(d.texture),d.x,d.y,d.source[0],d.source[1],d.source[2],d.source[3],d.flip_h,d.flip_v,false,d.color);
            }
        };
        // Pictures first, as on the console; then the views.
        using epok::world2d::RenderPass;
        RenderPass main;uint16_t pictures=0;
        epok::world2d::subviewports.each([&](epok::SubViewport2DComponent& picture){
            if(picture.hide_layer&&picture.layer!=epok::world2d::every_layer)main.hide(int8_t(picture.layer));
            picture.native_index=pictures++;
            if(!picture.enabled||!epok::is_active(&picture)||!picture.width||!picture.height)return;
            epok::Camera2D through;if(!picture.view(through))return;
            sink.begin_picture(picture.native_index,picture.width,picture.height,picture.background);
            RenderPass pass;pass.camera=&through;pass.screen=false;pass.only_layer=picture.layer;pass.clip[2]=int16_t(picture.width);pass.clip[3]=int16_t(picture.height);
            epok::world2d::render_pass=pass;view();
            sink.end_picture();
        });
        if(auto* split=epok::world2d::active_split()){
            for(int i=0;i<2;++i){
                epok::Camera2D through;if(!split->view(i,through))continue;
                const int16_t* r=through.viewport;
                sink.clip(r[0],r[1],r[0]+r[2],r[1]+r[3]);
                if(auto* camera=epok::world2d::camera_on(split->cameras[i]))sink.rectangle(-1,r[0],r[1],r[0]+r[2],r[1]+r[3],camera->background);
                RenderPass pass=main;pass.camera=&through;pass.screen=false;for(int k=0;k<4;++k)pass.clip[k]=r[k];
                epok::world2d::render_pass=pass;view();
            }
            sink.clip(0,0,epok::display_width,epok::display_height);
            if(split->divider){int16_t first[4];split->half(0,first);
                if(split->side_by_side)sink.rectangle(-1,first[2],0,first[2]+split->divider,epok::display_height,split->divider_color);
                else sink.rectangle(-1,0,first[3],epok::display_width,first[3]+split->divider,split->divider_color);}
            main.world=false;
        }
        epok::world2d::render_pass=main;view();
        epok::world2d::render_pass=RenderPass{};
    }
#endif
    if(!epok::scene_loading())epok::hud_focus_update(epok::objects.data(),epok::object_count,epok::hud_focus_edges());
    epok::hud_core::Compiler compiler(sink,epok::display_width,epok::display_height,{epok::hud_layout_budget,epok::hud_rectangle_budget,epok::hud_text_budget,epok::hud_glyph_budget,epok::hud_rotated_budget});
    int first[epok::objects.size()],next[epok::objects.size()];epok::Fixed measured[epok::objects.size()][2];epok::hud_core::Rect rects[epok::objects.size()];epok::hud_core::Affine2 transforms[epok::objects.size()];compiler.draw(epok::objects.data(),epok::object_count,first,next,measured,rects,transforms);auto s=compiler.stats;
    write32(EPOK_NATIVE_PLAY_MAGIC);write32(epok::performance_stats.frame);write32(std::max(epok::screen_fade,epok::transition.opacity));
    write32(uint32_t(epok::object_count));write32(uint32_t(epok::authored_count));const auto camera=epok::active_camera();write32(camera?camera.index:0xffffffffu);
    const bool loading=epok::transition.loading();
    const char* text=loading?epok::transition.text:"";
    const auto* picture=loading?(epok::transition.options.loading.image?epok::transition.options.loading.image:&epok::default_loading_image):nullptr;
    const bool valid_picture=picture&&picture->pixels&&picture->width&&picture->height&&picture->width<=64&&picture->height<=64&&!(picture->width&1);
    write32(uint32_t(sink.commands.size()));write32(uint32_t(strlen(text)));write32(uint32_t(epok::audio_events.size()));
    write32(s.rectangles);write32(s.glyphs);write32(s.texts);write32(s.images);write32(s.dropped);
    // The visibility track sample whose list the console would draw this frame.
    write32(epok::active_visibility_track&&epok::visibility_track_started?uint32_t(epok::visibility_page_selection.current):0xffffffffu);
    write32(uint32_t(epok::current_scene()));write32(epok::scene_transition_count());
    write32(uint32_t(epok::transition.phase));write32(epok::transition.audio_gain);
    const auto* color=epok::transition.options.loading.color;
    write32(uint32_t(color[0])|(uint32_t(color[1])<<8)|(uint32_t(color[2])<<16));
    write32(valid_picture?picture->width:0);write32(valid_picture?picture->height:0);
    for(unsigned p=0;p<4;++p){const auto out=epok::rumble.output(p);write32(rumble_identities[p]);write32(out.revision);write32(out.large);write32(out.small);write32(out.remaining_us);}
    for(size_t i=0;i<epok::object_count;++i){const auto& o=epok::objects[i];write32(o.alive);write32(o.active);write32(uint32_t(o.parent));for(auto v:o.transform.position)write32(uint32_t(v.raw()));for(auto v:o.transform.rotation)write32(uint32_t(v.raw()));for(auto v:o.transform.scale)write32(uint32_t(v.raw()));write32(uint32_t(o.camera_settings.field_of_view.raw()));write32(uint32_t(o.animator.ticks));write32(uint32_t(o.animator.clip));write32((o.animator.enabled?1u:0u)|(o.animator.playing?2u:0u)|(o.animator.looping?4u:0u)|(o.camera_settings.mirror_x?8u:0u));}
    for(const auto& command:sink.commands)for(auto value:command.v)write32(uint32_t(value));
    for(const auto& event:epok::audio_events){write32(uint32_t(event.source));write32(uint32_t(uint64_t(event.source)>>32));write32(event.action);write32(uint32_t(event.clip));write32(uint32_t(event.volume));write32(uint32_t(event.pitch));}
    fwrite(text,1,strlen(text),frames);
    if(valid_picture)for(size_t i=0;i<size_t(picture->width)*picture->height;++i){const uint16_t pixel=picture->pixels[i];fputc(pixel&255,frames);fputc(pixel>>8,frames);}
    fflush(frames);epok::audio_events.clear();
    if(loading)epok::transition.presented=true;
}
int main(){
#ifdef _WIN32
    SetErrorMode(0x0001|0x0002);_setmode(_fileno(stdin),_O_BINARY);_setmode(_fileno(stdout),_O_BINARY);frames=_fdopen(_dup(_fileno(stdout)),"wb");_dup2(_fileno(stderr),_fileno(stdout));
#else
    frames=fdopen(dup(fileno(stdout)),"wb");dup2(fileno(stderr),fileno(stdout));
#endif
    if(!frames)return 2;NativeCards cards;epok::memory_card.attach(cards);epok::time.reset(0);epok::input.reset();
    epok::audio_source_retained=[](const epok::AudioSource* source){return epok::music_active==source||epok::music_requested==source;};
    epok::nav::move_actor=&epok::move_and_slide;epok::nav::bounds_actor=&epok::collider_aabb;
#ifdef EPOK_WORLD2D
    epok::world2d::install();
#endif
    // Startup calls may queue one finite pulse before the first device sample.
    for(unsigned p=0;p<4;++p)epok::rumble.device(p,true,false,true);
    epok::scene_start_bank(0);
    for(auto& object:epok::objects)if(epok::is_active(&object)&&object.audio.enabled&&object.audio.play_on_start&&!object.audio.is_playing())object.audio.play();advance_camera_track(0);frame();
    uint32_t now=0,elapsed=0,magic=0;uint32_t buttons[4]={},analog[4]={},axes[4]={},flags[4]={};
    while(read32(magic)){if(magic!=EPOK_NATIVE_PLAY_MAGIC||!read32(elapsed))return 3;for(unsigned p=0;p<4;++p)if(!read32(buttons[p])||!read32(analog[p])||!read32(axes[p])||!read32(flags[p])||!read32(rumble_identities[p])||buttons[p]>65535||analog[p]>1||flags[p]>7)return 3;uint32_t completed=0;if(!read32(completed)||elapsed>100000||completed>EPOK_NATIVE_PLAY_AUDIO_LIMIT)return 3;
        for(uint32_t i=0;i<completed;++i){uint32_t low=0,high=0;if(!read32(low)||!read32(high))return 3;const uint64_t token=uint64_t(low)|(uint64_t(high)<<32);for(auto& entry:epok::audio_playing)if(entry.second.token==token)entry.second.playing=false;}
        now+=elapsed;
        epok::rumble.begin_frame(now,!elapsed||epok::time.paused()||epok::scene_loading());
        for(unsigned p=0;p<4;++p)epok::rumble.device(p,flags[p]&1,flags[p]&2,false,flags[p]&4,rumble_identities[p]);
        if(elapsed){epok::scene_tick(now);}
        if(elapsed&&!epok::scene_loading()){for(unsigned p=0;p<4;++p)epok::input.sample(p,flags[p]&1,uint16_t(buttons[p]),analog[p],uint8_t(axes[p]),uint8_t(axes[p]>>8),uint8_t(axes[p]>>16),uint8_t(axes[p]>>24));const auto steps=epok::time.advance(now);epok::level.frame_update(epok::time.frame_microseconds);
            if(epok::time.paused())epok::input.discard_edges();if(!epok::time.paused())epok::nav::world.tick();
            for(unsigned step=0;step<steps&&!epok::time.paused();++step){epok::time.begin_tick();epok::input.begin_tick();const Fixed dt(epok::time.delta_raw,Fixed::RAW);epok::level.tick(dt);
#ifdef EPOK_WORLD2D
                epok::world2d::service.step(dt,&epok::level);
#endif

                bool triggers=collision_world.has_trigger_pairs();for(size_t i=0;i<epok::object_count&&!triggers;++i)triggers=epok::objects[i].collider.enabled&&epok::objects[i].collider.trigger&&epok::is_active_slot(i);
                if(triggers){refresh_collisions();collision_world.update_triggers(dispatch_trigger);}for(size_t i=0;i<epok::object_count;++i)if(epok::is_active_slot(i)){epok::objects[i].animator.advance();epok::objects[i].sprite_animator.advance(dt,epok::objects[i].sprite);epok::objects[i].palette_animator.advance(dt);}epok::input.end_tick();}
            advance_camera_track(epok::time.frame_microseconds);epok::level.refresh_stats();
        }else {epok::time.synchronize(now);epok::input.discard_edges();}
        if(epok::time.paused()||epok::scene_loading()||!elapsed)epok::rumble.suspend();
        if(elapsed){cards.pump();++epok::performance_stats.frame;}frame();
    }
}
