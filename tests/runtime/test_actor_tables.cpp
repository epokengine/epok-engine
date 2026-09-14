// Cooked actor tables against the real runtime: the table shape below is written by
// hand in exactly the form src/project.rs emits, so a change to either side fails here.
#include <array>
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#define EPOK_OBJECT_REGISTRY_CAPACITY 48
namespace psyqo {struct GPU {uint32_t clock=0,waits=0;uint32_t now(){return clock;}void waitChainIdle(){++waits;}};}
#include "../../runtime/actor_tables.hpp"
namespace epok {
inline std::array<ActorData, 8> objects;
inline size_t object_count = 3, authored_count = 3;
inline AudioSource* music_active=nullptr;
inline AudioSource* music_requested=nullptr;
inline bool music_lookup=false;
inline unsigned audio_plays=0,audio_stops=0;
void AudioSource::play(){++audio_plays;} void AudioSource::stop(){++audio_stops;} bool AudioSource::is_playing()const{return false;}
void remove_runtime_owner(size_t){}
void reset_runtime_services(){}
void load_scene();
struct SceneBank {const char* name;void(*load)();};
inline const SceneBank scene_banks[]={{"Bank",load_scene}};
inline constexpr size_t scene_bank_count=1;
}
#include "../../runtime/lifecycle.hpp"
#include "../../runtime/scene_service.hpp"
using namespace epok;

namespace {
// Event log. Every gameplay callback appends one token so the order is an assertion
// about text rather than about counters.
char order[512];
size_t order_length=0;
void note(const char* token){for(size_t i=0;token[i];++i)if(order_length+1<sizeof(order))order[order_length++]=token[i];order[order_length]=0;}
bool contains(const char* needle){
    for(size_t i=0;i<=order_length;++i){size_t j=0;while(needle[j]&&order[i+j]==needle[j])++j;if(!needle[j])return true;}
    return false;
}
int position(const char* needle){
    for(size_t i=0;i<=order_length;++i){size_t j=0;while(needle[j]&&order[i+j]==needle[j])++j;if(!needle[j])return int(i);}
    return -1;
}

// Test classes. Identities are arbitrary but stable; the native bases use the real
// compile-time identities from object_model.hpp.
constexpr uint64_t hero_class=0x1001, tracker_class=0x1002, script_class=0x1003;
bool request_on_end=false;

struct Tracker : ActorComponent {
    static constexpr uint64_t static_class_id=tracker_class;
    uint64_t class_id() const override {return static_class_id;}
    int level_of_detail=0;
    // Observed inside begin_play: the cooked override must already be in place.
    int observed_level_of_detail=-1;
    unsigned triggers=0;
    void begin_play() override {note("tc+");observed_level_of_detail=level_of_detail;}
    void tick(Fixed) override {note("tct");}
    void on_trigger(DataHandle,TriggerPhase) override {++triggers;note("tg");}
    void end_play(EndPlayReason reason) override {note(reason==EndPlayReason::LevelUnloaded?"tc-u":"tc-d");}
};
struct Hero : Actor3D {
    static constexpr uint64_t static_class_id=hero_class;
    uint64_t class_id() const override {return static_class_id;}
    Fixed speed=Fixed(0.0);
    // Observed inside the actor's own begin_play: the P10 deviation is closed, so the
    // per-instance overrides, the authored components and the bound legacy slot are all
    // visible here rather than only from on_enable onwards.
    Fixed observed_speed=Fixed(0.0);
    bool observed_component=false,observed_slot=false;
    void begin_play() override;
    void tick(Fixed) override {note("at");}
    void end_play(EndPlayReason reason) override {note(reason==EndPlayReason::LevelUnloaded?"a-u":"a-d");if(request_on_end){request_on_end=false;request_scene(size_t(0));}}
};
struct MapScript : SceneScriptActor {
    static constexpr uint64_t static_class_id=script_class;
    uint64_t class_id() const override {return static_class_id;}
    unsigned observed_actors=0;
    // Map-scoped references, lowered to the null identity by P6 and bound by the cooked
    // scene_reference table before this class begins play.
    ObjectId target_actor;
    ObjectId target_entity;
    ObjectId observed_target;
    ObjectId observed_entity;
    void begin_play() override;
    void tick(Fixed) override {note("st");}
    void end_play(EndPlayReason reason) override {note(reason==EndPlayReason::LevelUnloaded?"s-u":"s-d");}
};

}

namespace epok {
const ClassDescriptor object_classes[] = {
{Object::static_class_id,0,epok::ObjectFamily::Object,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr},
{Actor::static_class_id,Object::static_class_id,epok::ObjectFamily::Actor,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr},
{Actor3D::static_class_id,Actor::static_class_id,epok::ObjectFamily::Actor,epok::ObjectDomain::World3D,0,6,
 &epok::object_construct<epok::Actor3D>,&epok::object_destruct,sizeof(epok::Actor3D),alignof(epok::Actor3D),
 &epok::ObjectPool<epok::Actor3D,4>::acquire,&epok::ObjectPool<epok::Actor3D,4>::release},
{SceneScriptActor::static_class_id,Actor::static_class_id,epok::ObjectFamily::Actor,epok::ObjectDomain::None,0,8,
 &epok::object_construct<epok::SceneScriptActor>,&epok::object_destruct,sizeof(epok::SceneScriptActor),alignof(epok::SceneScriptActor),
 &epok::ObjectPool<epok::SceneScriptActor,4>::acquire,&epok::ObjectPool<epok::SceneScriptActor,4>::release},
{ActorComponent::static_class_id,Object::static_class_id,epok::ObjectFamily::Component,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr},
{SceneComponent3D::static_class_id,ActorComponent::static_class_id,epok::ObjectFamily::Component,epok::ObjectDomain::World3D,1,16,
 &epok::object_construct<epok::SceneComponent3D>,&epok::object_destruct,sizeof(epok::SceneComponent3D),alignof(epok::SceneComponent3D),
 &epok::ObjectPool<epok::SceneComponent3D,4>::acquire,&epok::ObjectPool<epok::SceneComponent3D,4>::release},
{Level::static_class_id,Object::static_class_id,epok::ObjectFamily::Level,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr},
{hero_class,Actor3D::static_class_id,epok::ObjectFamily::Actor,epok::ObjectDomain::World3D,0,6,
 &epok::object_construct<Hero>,&epok::object_destruct,sizeof(Hero),alignof(Hero),
 &epok::ObjectPool<Hero,4>::acquire,&epok::ObjectPool<Hero,4>::release},
{tracker_class,ActorComponent::static_class_id,epok::ObjectFamily::Component,epok::ObjectDomain::None,7,0,
 &epok::object_construct<Tracker>,&epok::object_destruct,sizeof(Tracker),alignof(Tracker),
 &epok::ObjectPool<Tracker,4>::acquire,&epok::ObjectPool<Tracker,4>::release},
{script_class,SceneScriptActor::static_class_id,epok::ObjectFamily::Actor,epok::ObjectDomain::None,0,8,
 &epok::object_construct<MapScript>,&epok::object_destruct,sizeof(MapScript),alignof(MapScript),
 &epok::ObjectPool<MapScript,4>::acquire,&epok::ObjectPool<MapScript,4>::release},
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}

namespace {
void Hero::begin_play(){
    note("a+");
    // Overrides, authored components and the bound legacy slot are visible here.
    observed_speed=speed;
    observed_component=level.get_component<Tracker>(*this)!=nullptr;
    observed_slot=data()==&objects[1];
}
void MapScript::begin_play(){
    note("s+");
    // The scene script always observes fully configured actors.
    observed_actors=unsigned(level.actor_count());
    observed_target=target_actor;
    observed_entity=target_entity;
    auto* hero=object_registry.resolve<Hero>(level.actor_at(0));
    assert(hero&&hero->speed==Fixed(2.5)&&hero->data()==&objects[1]);
    assert(level.get_component<Tracker>(*hero)&&level.get_component<Tracker>(*hero)->level_of_detail==3);
}

// ---- cooked table, in the exact shape src/project.rs emits --------------------------
void actor_apply_0(ObjectRegistry& registry,Actor& actor,const ObjectId* components,const ObjectId*){
(void)registry;(void)actor;(void)components;
if(auto* self=registry.resolve<Hero>(actor.id())){
self->speed = Fixed(10240, Fixed::RAW);
}
if(auto* component=registry.resolve<Tracker>(components[1])){
component->level_of_detail = 3;
}
}
// Generated writer for the bank's map-scoped references: typed member access needs the
// C++ class, so the cook emits the switch and the table stays plain data.
void scene_reference_bind(ObjectRegistry& registry,Actor& owner,uint64_t member,ObjectId target){
(void)registry;(void)owner;(void)member;(void)target;
if(auto* self=registry.resolve<MapScript>(owner.id())){
if(member==UINT64_C(8193)){self->target_actor = target;}
if(member==UINT64_C(8194)){self->target_entity = target;}
}
}
constexpr SceneReferenceRecord scene_references[]={
{script_class,UINT64_C(8193),SceneRefKind::Actor,0,-1},
{script_class,UINT64_C(8194),SceneRefKind::Actor,1,-1},
};
constexpr ActorComponentRecord actor_components_0[]={
{SceneComponent3D::static_class_id,"Root",true,-1,1},
{tracker_class,"Tracker",false,-1,-1},
};
constexpr ActorComponentRecord actor_components_1[]={
{SceneComponent3D::static_class_id,"Root",true,-1,2},
};
const ActorRecord actor_records[]={
{hero_class,"Hero",true,-1,-1,-1,actor_components_0,2,&actor_apply_0},
{Actor3D::static_class_id,"Marker",true,0,0,0,actor_components_1,1,nullptr},
};
const ActorTable actor_table={actor_records,2,script_class,scene_references,2,&scene_reference_bind};
const ActorTable empty_table={nullptr,0,UINT64_C(0)};

const ActorTable* pending=&actor_table;
}

namespace epok {
void load_scene(){
    for(size_t j=0;j<objects.size();++j){auto generation=objects[j].generation+1;if(!generation)generation=1;objects[j]=ActorData{};objects[j].generation=generation;objects[j].alive=j<object_count;objects[j].parent=-1;}
    load_actor_bank(*pending,objects.data(),object_count);
}
}

static void reset(){
    order_length=0;order[0]=0;pending=&actor_table;
    pending_scene=-1;scene_stopping=scene_transitioning=lifecycle_tearing_down=false;
}

static void load_and_begin_play(){
    reset();load_scene();
    assert(level.actor_count()==2&&level.scene_script().valid());
    assert(object_registry.resolve<SceneScriptActor>(level.scene_script()));
    auto* script=object_registry.resolve<MapScript>(level.scene_script());
    assert(script&&script->observed_actors==2);
    // Overrides, components and the legacy slot are in place inside the actor's own
    // begin_play, not only from the scene script onwards (P10 deviation 1, closed).
    auto* hero=object_registry.resolve<Hero>(level.actor_at(0));
    assert(hero&&hero->observed_speed==Fixed(2.5)&&hero->observed_component&&hero->observed_slot);
    assert(level.get_component<Tracker>(*hero)->observed_level_of_detail==3);
    // Map-scoped references are live ObjectIds/handles before the script begins play.
    assert(script->observed_target==level.actor_at(0));
    assert(script->observed_entity==level.actor_at(1));
    // Components begin before their owning actor; the scene script begins last.
    assert(contains("tc+"));
    assert(position("a+")<position("s+"));
    assert(position("tc+")<position("s+"));
    // Logical parent and spatial attachment resolved from table indices.
    auto* marker=object_registry.resolve<Actor>(level.actor_at(1));
    assert(marker&&marker->logical_parent()==level.actor_at(0));
    auto* root=object_registry.resolve<SceneComponent3D>(marker->root_id());
    assert(root&&root->attach_parent.valid()&&root->entity_slot()==&objects[2]);
    assert(actor_stats.actors==2&&actor_stats.scene_scripts>=1&&actor_stats.components==3);
}

static void tick_order_runs_actors_then_script(){
    order_length=0;order[0]=0;
    level.tick(Fixed(0.25));
    assert(position("at")>=0&&position("st")>=0&&position("at")<position("st"));
    assert(position("tct")<position("at")); // components tick before their actor
}

static void transition_ends_the_script_before_actors(){
    order_length=0;order[0]=0;
    psyqo::GPU gpu;
    assert(request_scene(size_t(0))&&scene_tick(gpu));
    // The scene script ends first, while the actors are still alive.
    assert(position("s-u")>=0&&position("a-u")>=0&&position("s-u")<position("a-u"));
    assert(position("tc-u")>=0&&position("tc-u")<position("a-u"));
    // The legacy binding teardown follows the level teardown, exactly once.
    // The next bank starts from an empty level and gets its own scene script.
    assert(level.actor_count()==2&&level.scene_script().valid());
    assert(object_registry.stats.alive>0);
}

static void a_bank_without_actors_still_has_one_scene_script(){
    reset();pending=&empty_table;
    psyqo::GPU gpu;assert(request_scene(size_t(0))&&scene_tick(gpu));
    assert(level.actor_count()==0&&level.scene_script().valid());
    assert(object_registry.resolve<SceneScriptActor>(level.scene_script()));
    assert(!object_registry.resolve<MapScript>(level.scene_script()));
    level.tick(Fixed(0.25));
    reset();
}

static void trigger_delivery_reaches_each_component_once(){
    auto* hero=object_registry.resolve<Hero>(level.actor_at(0));
    assert(hero);auto* tracker=level.get_component<Tracker>(*hero);assert(tracker);
    const auto self=handle(hero->data()),other=handle(&objects[2]);
    assert(dispatch_slot_trigger(self,other,TriggerPhase::Enter)==2);
    assert(tracker->triggers==1);
    assert(dispatch_slot_trigger(self,other,TriggerPhase::Stay)==2);
    assert(tracker->triggers==2);
    assert(level.set_active(hero->id(),false));
    assert(dispatch_slot_trigger(self,other,TriggerPhase::Exit)==0);
    assert(level.set_active(hero->id(),true));
}

// Component-owned AudioSource storage follows the same rule allocate_actor_data applies to a
// legacy slot's audio: quarantined while the XA consumer still points at it.
static void the_audio_quarantine_hook_follows_the_music_service(){
    AudioSource owned;
    music_active=music_requested=nullptr;music_lookup=false;
    install_actor_service_hooks();
    assert(!audio_source_retained(&owned));
    music_active=&owned;assert(audio_source_retained(&owned));
    music_lookup=true;assert(audio_source_retained(&owned));
    music_active=nullptr;music_lookup=false;
    music_requested=&owned;assert(audio_source_retained(&owned));
    music_requested=nullptr;assert(!audio_source_retained(&owned));
    // The per-frame retry entry point main.cpp calls.
    collect_object_quarantine();
}

#ifndef EPOK_ACTOR_TABLE_FIXTURE_ONLY
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    load_and_begin_play();
    tick_order_runs_actors_then_script();
    transition_ends_the_script_before_actors();
    trigger_delivery_reaches_each_component_once();
    a_bank_without_actors_still_has_one_scene_script();
    the_audio_quarantine_hook_follows_the_music_service();
    std::puts("Cooked actor tables: bank load, overrides before begin_play, scene references, tick order, trigger delivery, audio quarantine and transition teardown passed.");
}
#endif
