// Real Level dispatch: inputs and actor rules precede physics, then presentation.
#include <cassert>
#include <cstdio>
#include <cstring>
#include "../../runtime/epok.hpp"
using namespace epok;
static char trace[64];static unsigned trace_size=0,command=0,position=0,observed=0,physics_steps=0,visual_steps=0,frame_callbacks=0;
static void note(char c){trace[trace_size++]=c;trace[trace_size]=0;}
struct PhysicsProbe:ActorComponent {
 static constexpr uint64_t static_class_id=0x7001;
 static constexpr TickGroup tick_group=TickGroup::Physics;
 uint64_t class_id()const override{return static_class_id;}
 void tick(Fixed)override{note('P');position+=command;++physics_steps;}
};
struct InheritedPhysics:PhysicsProbe {
 static constexpr uint64_t static_class_id=0x7002;
 uint64_t class_id()const override{return static_class_id;}
};
struct InputProbe:ActorComponent {
 static constexpr uint64_t static_class_id=0x7003;
 static constexpr TickGroup tick_group=TickGroup::Input;
 uint64_t class_id()const override{return static_class_id;}
 void tick(Fixed)override{note('I');command=1;}
};
struct VisualProbe:ActorComponent {
 static constexpr uint64_t static_class_id=0x7004;
 static constexpr TickGroup tick_group=TickGroup::PostPhysics;
 uint64_t class_id()const override{return static_class_id;}
 void tick(Fixed)override{note('V');assert(position==observed);++visual_steps;}
};
struct PlayerProbe:Actor3D {
 static constexpr uint64_t static_class_id=0x7010;
 bool destroy_on_tick=false;
 uint64_t class_id()const override{return static_class_id;}
 void tick(Fixed)override{note('A');command=2;if(destroy_on_tick)destroy();}
 void on_frame(uint32_t)override{++frame_callbacks;}
};
struct CameraProbe:Actor3D {
 static constexpr uint64_t static_class_id=0x7011;
 static constexpr TickGroup tick_group=TickGroup::PostPhysics;
 uint64_t class_id()const override{return static_class_id;}
 void tick(Fixed)override{note('C');observed=position;}
};
template<class T,class Base> static ClassDescriptor descriptor(ObjectFamily family){
 auto value=ClassDescriptor{T::static_class_id,Base::static_class_id,family,ObjectDomain::World3D,1,0,
  &object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,4>::acquire,&ObjectPool<T,4>::release};
 if constexpr(std::is_base_of_v<ActorComponent,T>){value.component_tick=ComponentCallbacks<T>::tick;value.component_frame=ComponentCallbacks<T>::frame;}
 else{value.component_tick=ActorCallbacks<T>::tick;value.component_frame=ActorCallbacks<T>::frame;}
 value.tick_group=TickGroupOf<T>::value;return value;
}
namespace epok {
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
int entity_index(const ActorData*){return -1;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 descriptor<Actor3D,Actor>(ObjectFamily::Actor),
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
 descriptor<PhysicsProbe,ActorComponent>(ObjectFamily::Component),
 descriptor<InheritedPhysics,PhysicsProbe>(ObjectFamily::Component),
 descriptor<InputProbe,ActorComponent>(ObjectFamily::Component),
 descriptor<VisualProbe,ActorComponent>(ObjectFamily::Component),
 descriptor<PlayerProbe,Actor3D>(ObjectFamily::Actor),
 descriptor<CameraProbe,Actor3D>(ObjectFamily::Actor),
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
static void reset_trace(){trace_size=0;trace[0]=0;}
struct OrdinaryProperty {unsigned tick_group=12;};
struct OrdinaryConstant {static constexpr unsigned tick_group=2;};
struct OrdinaryTypedProperty {const TickGroup tick_group=TickGroup::PostPhysics;};
// A conservative callback prefilter may see a stale list entry. Only the
// generation-checked dispatcher may authorize the actual callback.
struct DispatchListProbe:Level {
 void append_probe(ObjectId id){assert(m_actor_count<level_actor_capacity);m_actors[m_actor_count++]=id;}
};
int main(){
 static_assert(TickGroupOf<OrdinaryProperty>::value==TickGroup::Gameplay);
 static_assert(TickGroupOf<OrdinaryConstant>::value==TickGroup::Gameplay);
 static_assert(TickGroupOf<OrdinaryTypedProperty>::value==TickGroup::Gameplay);
 static_assert(TickGroupOf<Actor3D>::value==TickGroup::Gameplay);
 static_assert(TickGroupOf<InputProbe>::value==TickGroup::Input);
 static_assert(TickGroupOf<InheritedPhysics>::value==TickGroup::Physics);
 ObjectRegistryStorage<32> registry;active_object_registry=&registry;DispatchListProbe level;assert(level.bind(registry));
 // Camera precedes the player; its presentation must still see this step's motion.
 auto camera=level.spawn_actor(*find_object_class(CameraProbe::static_class_id),"Camera");assert(camera.valid());
 auto player_id=level.spawn_actor(*find_object_class(PlayerProbe::static_class_id),"Player");
 auto* player=registry.resolve<PlayerProbe>(player_id);assert(player);
 // Presentation and physics are deliberately placed before the input component.
 assert(level.add_component<VisualProbe>(*player,"Animation"));
 assert(level.add_component<InheritedPhysics>(*player,"Motor"));
 auto* input=level.add_component<InputProbe>(*player,"Input");assert(input);
 level.append_probe({player_id.index,0});
 level.append_probe({player_id.index,uint16_t(player_id.generation+1)});
 level.append_probe({uint16_t(registry.capacity+1),1});
 level.append_probe({});
 const Fixed dt(1.0/60.0);
 for(unsigned i=0;i<3;++i){reset_trace();level.tick(dt);assert(std::strcmp(trace,"IAPCV")==0);assert(position==2*(i+1));assert(physics_steps==i+1&&visual_steps==i+1);level.frame_update(1000);assert(frame_callbacks==i+1);}
 auto carrier_id=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Input after player");auto* carrier=registry.resolve<Actor3D>(carrier_id);assert(carrier);
 assert(level.remove_component(*player,input->id()));assert(level.add_component<InputProbe>(*carrier,"Input"));
 reset_trace();level.tick(dt);assert(std::strcmp(trace,"IAPCV")==0&&position==8&&physics_steps==4);
 assert(level.set_active(player_id,false)&&level.set_active(carrier_id,false));reset_trace();level.tick(dt);assert(std::strcmp(trace,"C")==0&&physics_steps==4);
 assert(level.set_active(player_id,true)&&level.set_active(carrier_id,true));player->set_wants_tick(false);reset_trace();level.tick(dt);
 assert(std::strcmp(trace,"IPCV")==0&&position==9&&physics_steps==5); // Component physics continues without actor rules.
 player->set_wants_tick(true);player->destroy_on_tick=true;reset_trace();level.tick(dt);
 assert(std::strcmp(trace,"IAC")==0&&physics_steps==5&&!registry.get(player_id));
 level.end_play_all(EndPlayReason::LevelUnloaded);
 std::printf("Tick groups: same-step actor/component commands, inherited physics, camera-before-player, one simulation, activation, destruction and stale/out-of-range callback candidates passed; Actor=%zu ClassDescriptor=%zu Level=%zu\n",sizeof(Actor),sizeof(ClassDescriptor),sizeof(Level));
}
