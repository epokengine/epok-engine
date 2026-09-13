#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstring>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
namespace psyqo { struct GPU {int waits=0;uint32_t clock=0;uint32_t now(){return clock;}void waitChainIdle(){++waits;}}; }
namespace epok {
struct Audio { bool enabled=false,play_on_start=false;int stops=0,plays=0;void stop(){++stops;}void play(){++plays;} };
struct Transform { double scale[3]={}; };
struct Material {uint8_t color[3]={};};
struct Entity {bool alive=true,active=true;uint32_t generation=1;int parent=-1;Audio audio;Transform transform;Material material;char name[129]={};void set_name(const char* s){if(s)std::strncpy(name,s,128);}};
struct EntityHandle {uint16_t index=0xffff;uint32_t generation=0;Entity* get()const;};
struct Behaviour {virtual void on_enable(){}virtual void on_disable(){}virtual void on_destroy(){}};
struct Binding {Behaviour* behaviour;size_t entity;};
inline std::array<Entity,12> objects;
inline size_t object_count=3,authored_count=3;
struct View {Binding* data=nullptr;size_t count=0;Binding* begin(){return data;}Binding* end(){return data+count;}};
inline View bindings;
inline Audio *music_active=nullptr,*music_requested=nullptr;
inline bool music_lookup=false;
inline unsigned removals=0,resets=0;
void remove_runtime_owner(size_t){++removals;}
void reset_runtime_services(){++resets;}
Entity* create_entity(const char*,Entity* parent=nullptr);
bool request_scene(size_t);
bool request_scene(const char*);
void load0();void load1();
struct SceneBank {const char* name;void(*load)();};
inline const SceneBank scene_banks[]={{"First",load0},{"Second",load1}};
inline constexpr size_t scene_bank_count=2;
}
#include "../../runtime/lifecycle.hpp"
#include "../../runtime/scene_service.hpp"
namespace epok {
static unsigned loaded=99;
void load(unsigned which){loaded=which;for(auto& e:objects){auto g=next_generation(e.generation);e=Entity{};e.generation=g;e.alive=false;}object_count=authored_count=1;objects[0].alive=true;objects[0].audio.enabled=true;objects[0].audio.play_on_start=true;bindings={};}
void load0(){load(0);}void load1(){load(1);}
}
using namespace epok;
struct Observer:Behaviour {
    unsigned enabled=0,disabled=0,destroyed=0;Entity* owner=nullptr;bool teardown_test=false;
    void on_enable()override{++enabled;}
    void on_disable()override{++disabled;}
    void on_destroy()override{
        ++destroyed;
        if(teardown_test){assert(!destroy_entity(owner));assert(!create_entity("During teardown"));request_scene(size_t(0));}
    }
};
void reset(){objects={};object_count=authored_count=3;objects[1].parent=0;objects[2].parent=1;bindings={};music_active=music_requested=nullptr;music_lookup=false;removals=resets=0;pending_scene=-1;scene_stopping=scene_transitioning=lifecycle_tearing_down=false;scene_stats={};}
void activation_destroy_and_reuse(){
    reset();Observer a,b;a.owner=&objects[0];b.owner=&objects[1];Binding list[]={{&a,0},{&b,1}};bindings={list,2};
    objects[2].audio.enabled=true;auto child_handle=handle(&objects[2]);
    assert(set_active(&objects[0],false));assert(!is_active(&objects[2]));assert(a.disabled==1&&b.disabled==1);assert(objects[2].audio.stops==1);
    assert(set_active(&objects[0],false));assert(a.disabled==1);
    assert(set_active(&objects[0],true));assert(a.enabled==1&&b.enabled==1);
    assert(set_active(&objects[0],false));assert(destroy_entity(&objects[0]));assert(a.disabled==2&&a.destroyed==1&&b.destroyed==1);assert(!child_handle.get());assert(removals==3);
    assert(!destroy_entity(&objects[0]));
    Entity* first=create_entity("First");assert(first&&entity_index(first)==3);auto old=handle(first);assert(destroy_entity(first));auto reused=create_entity("Second");assert(reused==first&&!old.get());assert(find_entity("Second")==reused&&!find_entity("First"));
    reused->parent=999;assert(!create_entity("Invalid child",reused));reused->parent=-1;
    music_active=&reused->audio;assert(destroy_entity(reused));auto replacement=create_entity("Quarantine");assert(replacement!=reused);music_active=nullptr;assert(create_entity("Released")==reused);
    objects[0].generation=0xffffffff;objects[0].alive=true;objects[0].parent=-1;assert(destroy_entity(&objects[0]));assert(objects[0].generation==1);
}
void scene_transition(){
    reset();Observer a;a.owner=&objects[0];a.teardown_test=true;Binding list[]={{&a,0}};bindings={list,1};auto old=handle(&objects[0]);psyqo::GPU gpu;
    assert(!request_scene(size_t(99)));assert(!request_scene("Missing"));assert(scene_stats.rejected==2);
    assert(request_scene("Second"));music_active=&objects[0].audio;music_lookup=true;
    assert(!scene_tick(gpu));assert(objects[0].audio.stops==1);assert(!scene_tick(gpu));assert(objects[0].audio.stops==1);
    music_active=nullptr;music_lookup=false;assert(scene_tick(gpu));assert(loaded==1&&current_scene()==1);assert(!old.get());assert(a.destroyed==1&&a.disabled==1);assert(gpu.waits==1&&resets==1);assert(objects[0].audio.plays==1);
    assert(scene_loading());assert(scene_tick(gpu));assert(loaded==0&&scene_stats.transitions==2&&!scene_loading());
    for(unsigned i=0;i<100;++i){auto before=handle(&objects[0]);assert(request_scene(size_t(i%2)));assert(scene_tick(gpu));assert(!before.get());assert(object_count==1);}
}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    activation_destroy_and_reuse();scene_transition();std::puts("Runtime lifecycle, audio quarantine and scene transition tests passed.");
    return 0;
}
