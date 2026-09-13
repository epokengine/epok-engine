#include <array>
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#define EPOK_BLUEPRINTS 1
namespace psyqo {struct GPU {void waitChainIdle(){}};}
#include "../../runtime/blueprint_spawn.hpp"
#include "../../runtime/blueprint_template.hpp"
namespace epok {
inline std::array<Entity, 36> objects;
inline size_t object_count = 1, authored_count = 1;
struct View {Binding* data=nullptr;size_t count=0;Binding* begin()const{return data;}Binding* end()const{return data+count;}};
inline View bindings;
inline AudioSource* music_active=nullptr;
inline AudioSource* music_requested=nullptr;
inline bool music_lookup=false;
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing()const{return false;}
void remove_runtime_owner(size_t){}
void reset_runtime_services(){++blueprint_scene_generation;}
void load_scene();
struct SceneBank {const char* name;void(*load)();};
inline const SceneBank scene_banks[]={{"Scene",load_scene}};
inline constexpr size_t scene_bank_count=1;
}
#include "../../runtime/lifecycle.hpp"
#include "../../runtime/scene_service.hpp"
using namespace epok;
namespace {
unsigned starts=0,enables=0,disables=0,destroys=0,cancels=0,updates=0,frames=0,triggers=0;
EntityHandle nested, victim;
EntityHandle template_child;
bool components_configured=false, reject_components=false;
struct Enemy:Behaviour {
    int value=7;
    bool kill_self=false,replace_other=false,spawn_during_destroy=false;
    uint64_t blueprint_class_id()const override{return 2;}
    void start(Transform&)override{++starts;assert(&entity()!=nullptr&&entity().alive);}
    void on_enable()override{++enables;}
    void on_disable()override{++disables;}
    void on_destroy()override{++destroys;if(spawn_during_destroy)nested=bp::spawn(2,"Teardown");}
    void blueprint_cancel()override{++cancels;}
    void frame_update(Transform&,uint32_t)override{++frames;}
    void on_trigger(EntityHandle,TriggerPhase)override{++triggers;}
    void update(Transform&,Fixed)override{
        ++updates;
        if(kill_self){
            value=99;auto old=handle(&entity());assert(destroy_entity(&entity()));
            nested=bp::spawn(2,"Nested");
            assert(value==99&&!old.get()); // Reset is deferred until callback exit.
            assert(!nested.get()||nested.index!=old.index);
        }
        if(replace_other){assert(destroy_entity(victim.get()));nested=bp::spawn(2,"Replacement");assert(nested.get());}
    }
};
struct Many:Enemy {uint64_t blueprint_class_id()const override{return 3;}};
struct TemplateEnemy:Enemy {
    uint64_t blueprint_class_id()const override{return 4;}
    void start(Transform& transform)override {
        Enemy::start(transform);assert(components_configured&&template_child.get()&&bp::behaviour(template_child));
        assert(static_cast<Enemy*>(bp::behaviour(template_child))->value==88);
        assert(value==77&&transform.position[0]==Fixed(3.0));
    }
};
bool configure_template(EntityHandle owner);
bool configure_invalid(EntityHandle owner);
}
namespace epok::bp {
inline const ClassInfo classes[]={
    {1,0,nullptr,nullptr},{2,1,&TypedPool<Enemy,2>::acquire,&TypedPool<Enemy,2>::release},
    {3,2,&TypedPool<Many,32>::acquire,&TypedPool<Many,32>::release},
    {4,2,&TypedPool<TemplateEnemy,2>::acquire,&TypedPool<TemplateEnemy,2>::release,&configure_template},
    {5,2,&TypedPool<TemplateEnemy,2>::acquire,&TypedPool<TemplateEnemy,2>::release,&configure_invalid},
};
inline const size_t class_count=5;
}
namespace {
void root_values(Behaviour* behaviour,const EntityHandle* owners,size_t count){assert(count==3);static_cast<TemplateEnemy*>(behaviour)->value=77;template_child=owners[2];}
void child_values(Behaviour* behaviour,const EntityHandle*,size_t){static_cast<Enemy*>(behaviour)->value=88;}
bool configure_components(const EntityHandle* owners,size_t count){
    assert(count==3&&owners[0].get()&&owners[1].get()&&owners[2].get());
    assert(!bp::find_binding(owners[0])->started&&!bp::find_binding(owners[2])->started);
    assert(static_cast<Enemy*>(bp::behaviour(owners[2]))->value==88);
    components_configured=true;return !reject_components;
}
bool configure_template(EntityHandle owner){
    Entity prototypes[3]={};
    for(auto& prototype:prototypes){prototype.transform.scale[0]=prototype.transform.scale[1]=prototype.transform.scale[2]=1.0;prototype.set_name("Prototype");}
    prototypes[0].parent=-1;prototypes[0].transform.position[0]=3.0;
    prototypes[1].parent=0;prototypes[1].active=false;prototypes[1].collider.enabled=true;
    prototypes[2].parent=1;
    bp::TemplateBinding bindings[]={{0,4,&root_values},{2,2,&child_values}};
    return bp::instantiate_template(owner,prototypes,3,0,bindings,2,&configure_components);
}
bool configure_invalid(EntityHandle owner){
    Entity prototypes[2]={};prototypes[0].parent=-1;prototypes[1].parent=99;
    return bp::instantiate_template(owner,prototypes,2,0,nullptr,0);
}
}
namespace epok {
void load_scene(){
    for(auto& entity:objects){auto generation=next_generation(entity.generation);entity={};entity.generation=generation;entity.alive=false;entity.parent=-1;}
    objects[0].alive=true;object_count=authored_count=1;bindings={};
}
}
static void reset(){
    // Tear down every dynamic object through production lifecycle code.
    for(size_t i=authored_count;i<object_count;++i)if(objects[i].alive)destroy_entity(&objects[i]);
    assert(bp::spawn_stats.alive==0);
    load_scene();bp::spawn_stats={};starts=enables=disables=destroys=cancels=updates=frames=triggers=0;
    pending_scene=-1;scene_stopping=scene_transitioning=lifecycle_tearing_down=false;nested=victim={};
    components_configured=reject_components=false;
}
static void update_all(){bp::visit([](Binding& binding,EntityHandle owner){if(is_active(owner.get()))binding.behaviour->update(owner.get()->transform,0.25);});}
static void pools_and_lifecycle(){
    reset();assert(!bp::spawn(0,"Invalid").get()&&!bp::spawn(1,"Abstract").get());
    auto first=bp::spawn(2,"First"),second=bp::spawn(2,"Second");
    assert(first.get()&&second.get()&&starts==2&&enables==2&&bp::spawn_stats.alive==2);
    assert(!bp::spawn(2,"Full").get()&&bp::spawn_stats.rejected==3);
    assert(bp::is_a(first,2)&&bp::is_a(first,1)&&!bp::is_a(first,3));
    assert(!bp::class_is_a(999,999)&&!bp::class_is_a(2,0));
    auto* instance=static_cast<Enemy*>(bp::behaviour(first));instance->value=42;
    assert(set_active(first.get(),false)&&disables==1);update_all();assert(updates==1);
    assert(set_active(first.get(),true)&&enables==3);
    bp::visit([](Binding& binding,EntityHandle owner){binding.behaviour->frame_update(owner.get()->transform,1);binding.behaviour->on_trigger(owner,TriggerPhase::Enter);});
    assert(frames==2&&triggers==2);
    assert(destroy_entity(first.get())&&!first.get()&&!bp::behaviour(first)&&destroys==1&&cancels==1);
    auto replacement=bp::spawn(2,"Replacement");assert(replacement.get()&&replacement.index==first.index);
    assert(static_cast<Enemy*>(bp::behaviour(replacement))->value==7);
    for(unsigned i=0;i<100;++i){auto old=replacement;assert(destroy_entity(old.get()));replacement=bp::spawn(2,"Again");assert(replacement.get()&&!old.get());}
    assert(bp::spawn_stats.alive==2);
}
static void mutation_and_scene(){
    reset();auto first=bp::spawn(2,"Self");static_cast<Enemy*>(bp::behaviour(first))->kill_self=true;
    update_all();assert(!first.get()&&nested.get()&&updates==1&&bp::spawn_stats.alive==1);
    assert(static_cast<Enemy*>(bp::behaviour(nested))->value==7);
    reset();first=bp::spawn(2,"First");victim=bp::spawn(2,"Victim");
    static_cast<Enemy*>(bp::behaviour(first))->replace_other=true;
    update_all();assert(updates==1&&!victim.get()&&nested.get());
    static_cast<Enemy*>(bp::behaviour(first))->replace_other=false;
    static_cast<Enemy*>(bp::behaviour(first))->spawn_during_destroy=true;
    auto before=first;psyqo::GPU gpu;assert(request_scene(size_t(0))&&scene_tick(gpu));
    assert(!before.get()&&!nested.get()&&bp::spawn_stats.alive==0);
    assert(bp::spawn(2,"After scene").get());
}
static void global_capacity_and_authored(){
    reset();EntityHandle handles[32];
    for(auto& owner:handles){owner=bp::spawn(3,"Many");assert(owner.get());}
    assert(!bp::spawn(2,"Global full").get()&&bp::spawn_stats.alive==32);
    for(auto owner:handles)assert(destroy_entity(owner.get()));
    assert(bp::spawn_stats.alive==0&&bp::spawn(3,"Capacity recovered").get());
    reset();Enemy authored;authored.bind(objects[0]);Binding record{&authored,0};bindings={&record,1};
    const auto owner=handle(&objects[0]);assert(bp::behaviour(owner)==&authored&&bp::is_a(owner,1));
    assert(destroy_entity(owner.get())&&!bp::behaviour(owner));bindings={};
}
static void templates_bind_before_start_and_roll_back(){
    reset();const auto root=bp::spawn(4,"Caller Name",&objects[0]);
    assert(root.get()&&root.get()->parent==0&&root.get()->name[0]=='C');
    assert(starts==2&&enables==1&&bp::spawn_stats.alive==2&&template_child.get());
    const auto parent=template_child.get()->parent;assert(parent>=0&&objects[size_t(parent)].collider.enabled&&!objects[size_t(parent)].active);
    assert(destroy_entity(root.get())&&!template_child.get()&&bp::spawn_stats.alive==0);
    reset();const auto first=bp::spawn(2,"Pool 1"),second=bp::spawn(2,"Pool 2");assert(first.get()&&second.get());
    const auto before_starts=starts,before_destroy=destroys;
    assert(!bp::spawn(4,"Cannot reserve child").get());
    assert(starts==before_starts&&destroys==before_destroy&&bp::spawn_stats.alive==2);
    assert(!bp::spawn(5,"Bad prototype").get()&&starts==before_starts&&destroys==before_destroy);
    assert(destroy_entity(first.get())&&destroy_entity(second.get()));
    assert(bp::spawn(4,"Retry after rollback").get());
    reset();reject_components=true;
    assert(!bp::spawn(4,"Component capacity exhausted").get());
    assert(components_configured&&!template_child.get()&&starts==0&&destroys==0&&bp::spawn_stats.alive==0);
    for(size_t i=authored_count;i<object_count;++i)assert(!objects[i].alive);
    reject_components=false;assert(bp::spawn(4,"Component capacity recovered").get());
}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    pools_and_lifecycle();mutation_and_scene();global_capacity_and_authored();templates_bind_before_start_and_roll_back();reset();
    std::puts("Blueprint typed spawning, lifecycle, scene reset, capacity recovery and reentrant callback quarantine passed.");
}
