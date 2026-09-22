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
#include "scene.hh"
#include "hud_commands.hpp"
#include "hud-config.hh"
#include "hud_focus.hpp"
#include "native_play_protocol.h"
#include "transform_cache.hpp"

namespace epok {
MusicStats music_stats; PerformanceStats performance_stats; LightingEnvironment lighting_environment;
LightingStats lighting_stats; MeshStats mesh_stats;
AudioSource* music_active=nullptr; AudioSource* music_requested=nullptr;
static std::string requested_scene;
static std::map<const AudioSource*,bool> audio_playing;
struct HostAudioEvent{uintptr_t source;uint32_t action;int32_t clip,volume,pitch;};
static std::vector<HostAudioEvent> audio_events;
static void audio_event(const AudioSource* source,uint32_t action){
    if(audio_events.size()<EPOK_NATIVE_PLAY_AUDIO_LIMIT)audio_events.push_back({uintptr_t(source),action,source->clip,source->volume.raw(),source->pitch.raw()});
}
void AudioSource::play(){if(!enabled||clip<0)return;audio_playing[this]=true;audio_event(this,1);}
void AudioSource::stop(){auto i=audio_playing.find(this);if(i!=audio_playing.end()&&i->second)audio_event(this,0);audio_playing[this]=false;}
bool AudioSource::is_playing() const {auto i=audio_playing.find(this);return i!=audio_playing.end()&&i->second;}
void remove_runtime_owner(size_t){}
void reset_runtime_services(){}
bool request_scene(const char* name){if(!name||!*name||!requested_scene.empty())return false;requested_scene=name;return true;}
bool request_scene(size_t index){return request_scene(std::to_string(index).c_str());}
bool request_scene(const char* name,const TransitionOptions&){return request_scene(name);}
bool request_scene(size_t index,const TransitionOptions&){return request_scene(index);}
bool scene_loading(){return !requested_scene.empty();}
size_t current_scene(){return 0;}
uint32_t scene_transition_count(){return requested_scene.empty()?0u:1u;}
uint32_t scene_rejected_count(){return 0;}
bool scene_waiting(){return false;}
}
#include "lifecycle.hpp"

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
void reset_motion_interpolation(){}
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
    const int64_t x=int64_t(p[0].raw()*focal),y=int64_t(p[1].raw()*focal),z=p[2].raw();
    screen[0]=Fixed(int32_t(int64_t(display_width/2)*4096+x*(display_width/2)*4096/z),Fixed::RAW);
    screen[1]=Fixed(int32_t(int64_t(display_height/2)*4096-y*(display_height*2/3)*4096/z),Fixed::RAW);return true;
}
SpatialHit raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers){refresh_collisions();return collision_world.raycast(origin,delta,mask,entity_index(ignore),triggers);}
void raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask,const ActorData* ignore,bool triggers){if(!queries||!results)return;refresh_collisions();collision_world.raycast_batch(queries,results,count,mask,entity_index(ignore),triggers);}
size_t overlap(const Aabb& box,DataHandle* output,size_t capacity,uint32_t mask,const ActorData* ignore,bool triggers){refresh_collisions();uint16_t found[objects.size()];auto count=collision_world.overlap(box,found,objects.size(),mask,entity_index(ignore),triggers);if(output)for(size_t i=0;i<count&&i<capacity;++i)output[i]=handle(&objects[found[i]]);return count;}
bool collider_aabb(const ActorData& entity,Aabb& output){refresh_collisions();const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));if(!box)return false;output=*box;return true;}
SpatialHit query_ground(const ActorData& entity,Fixed distance,uint32_t mask){refresh_collisions();const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));return box?collision_world.ground(*box,distance,mask&entity.collider.mask,i):SpatialHit{};}
MoveResult move_and_slide(ActorData& entity,const Fixed* delta,uint32_t mask){
    refresh_collisions();const int i=entity_index(&entity);auto* box=i<0?nullptr:collision_world.bounds(size_t(i));
    if(!box){MoveResult result;result.unresolved_overlap=true;return result;}
    auto result=collision_world.move_and_slide(*box,delta,mask&entity.collider.mask,i);
    if(entity.parent<0){for(int r=0;r<3;++r)entity.transform.position[r]+=result.displacement[r];return result;}
    Matrix inverse=Matrix::identity();int parent=entity.parent;unsigned depth=0;
    while(parent>=0&&size_t(parent)<object_count&&depth++<32){inverse=inverse.compose(inverse_local(objects[parent].transform));parent=objects[parent].parent;}
    if(parent>=0){result=MoveResult{};result.unresolved_overlap=true;return result;}
    for(int r=0;r<3;++r)for(int c=0;c<3;++c)entity.transform.position[r]+=inverse.values[r][c]*result.displacement[c];return result;
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
static void frame(){
    EpokHudSink sink;for(size_t i=0;i<epok::texture_count;++i){sink.dimensions.push_back(epok::texture_assets[i].width);sink.dimensions.push_back(epok::texture_assets[i].height);}
    epok::hud_focus_update(epok::objects.data(),epok::object_count,epok::hud_focus_edges());
    epok::hud_core::Compiler compiler(sink,epok::display_width,epok::display_height,{epok::hud_layout_budget,epok::hud_rectangle_budget,epok::hud_text_budget,epok::hud_glyph_budget,epok::hud_rotated_budget});
    int first[epok::objects.size()],next[epok::objects.size()];epok::Fixed measured[epok::objects.size()][2];epok::hud_core::Rect rects[epok::objects.size()];epok::hud_core::Affine2 transforms[epok::objects.size()];compiler.draw(epok::objects.data(),epok::object_count,first,next,measured,rects,transforms);auto s=compiler.stats;
    write32(EPOK_NATIVE_PLAY_MAGIC);write32(epok::performance_stats.frame);write32(epok::screen_fade);
    write32(uint32_t(epok::object_count));write32(uint32_t(epok::authored_count));const auto camera=epok::active_camera();write32(camera?camera.index:0xffffffffu);
    write32(uint32_t(sink.commands.size()));write32(uint32_t(epok::requested_scene.size()));write32(uint32_t(epok::audio_events.size()));
    write32(s.rectangles);write32(s.glyphs);write32(s.texts);write32(s.images);write32(s.dropped);
    for(size_t i=0;i<epok::object_count;++i){const auto& o=epok::objects[i];write32(o.alive);write32(o.active);write32(uint32_t(o.parent));for(auto v:o.transform.position)write32(uint32_t(v.raw()));for(auto v:o.transform.rotation)write32(uint32_t(v.raw()));for(auto v:o.transform.scale)write32(uint32_t(v.raw()));write32(uint32_t(o.camera_settings.field_of_view.raw()));write32(uint32_t(o.animator.ticks));write32(uint32_t(o.animator.clip));write32((o.animator.enabled?1u:0u)|(o.animator.playing?2u:0u)|(o.animator.looping?4u:0u));}
    for(const auto& command:sink.commands)for(auto value:command.v)write32(uint32_t(value));
    for(const auto& event:epok::audio_events){write32(uint32_t(event.source));write32(uint32_t(uint64_t(event.source)>>32));write32(event.action);write32(uint32_t(event.clip));write32(uint32_t(event.volume));write32(uint32_t(event.pitch));}
    fwrite(epok::requested_scene.data(),1,epok::requested_scene.size(),frames);fflush(frames);epok::audio_events.clear();
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
    epok::initialize_components();if(epok::load_actor_bank(epok::actor_table,epok::objects.data(),epok::object_count)!=epok::actor_table.count)return 4;
    for(auto& object:epok::objects)if(epok::is_active(&object)&&object.audio.enabled&&object.audio.play_on_start&&!object.audio.is_playing())object.audio.play();frame();
    uint32_t now=0,elapsed=0;uint32_t buttons[4]={},analog[4]={},axes[4]={};
    while(read32(elapsed)){for(unsigned p=0;p<4;++p)if(!read32(buttons[p])||!read32(analog[p])||!read32(axes[p])||buttons[p]>65535||analog[p]>1)return 3;uint32_t completed=0;if(!read32(completed)||elapsed>100000||completed>EPOK_NATIVE_PLAY_AUDIO_LIMIT)return 3;
        for(uint32_t i=0;i<completed;++i){uint32_t low=0,high=0;if(!read32(low)||!read32(high))return 3;const auto* source=reinterpret_cast<const epok::AudioSource*>(uintptr_t(uint64_t(low)|(uint64_t(high)<<32)));auto found=epok::audio_playing.find(source);if(found!=epok::audio_playing.end())found->second=false;}
        if(epok::requested_scene.empty()){now+=elapsed;for(unsigned p=0;p<4;++p)epok::input.sample(p,true,uint16_t(buttons[p]),analog[p],uint8_t(axes[p]),uint8_t(axes[p]>>8),uint8_t(axes[p]>>16),uint8_t(axes[p]>>24));const auto steps=epok::time.advance(now);epok::level.frame_update(epok::time.frame_microseconds);
            if(epok::time.paused())epok::input.discard_edges();if(!epok::time.paused())epok::nav::world.tick();
            for(unsigned step=0;step<steps&&!epok::time.paused();++step){epok::time.begin_tick();epok::input.begin_tick();const Fixed dt(epok::time.delta_raw,Fixed::RAW);epok::level.tick(dt);
                bool triggers=collision_world.has_trigger_pairs();for(size_t i=0;i<epok::object_count&&!triggers;++i)triggers=epok::objects[i].collider.enabled&&epok::objects[i].collider.trigger&&epok::is_active_slot(i);
                if(triggers){refresh_collisions();collision_world.update_triggers(dispatch_trigger);}for(size_t i=0;i<epok::object_count;++i)if(epok::is_active_slot(i)){epok::objects[i].animator.advance();epok::objects[i].sprite_animator.advance(dt,epok::objects[i].sprite);epok::objects[i].palette_animator.advance(dt);}epok::input.end_tick();}
            cards.pump();epok::level.refresh_stats();++epok::performance_stats.frame;
        }frame();
    }
}
