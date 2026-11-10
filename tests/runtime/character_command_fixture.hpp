#pragma once
// Producers use the real registry, routing, tick stages and both movement solvers.
#define EPOK_GAMEPLAY_CONTROL 1
#include <cassert>
#include <cstdio>
#include "../../runtime/character_input.hpp"
#ifdef EPOK_TEST_CHARACTER_ANIMATION
#include "../../runtime/character_animation.hpp"
#endif
#include "../../runtime/world2d_service.hpp"
using namespace epok;
static ActorData slots[24];static unsigned used=0;
static CollisionWorld<Fixed,32,4> world;
static ActorData* current=nullptr;
class CallbackSource:public ScriptedCommand3DComponent {
public:
 static constexpr uint64_t static_class_id=0x2ac121;
 uint64_t class_id()const override{return static_class_id;}
 unsigned action=0,submitted=0;CharacterCommandStatus nested=CharacterCommandStatus::NoStep;
 CharacterCommandResult3D calculate_command(Fixed dt)override{
  nested=update(dt).status;
  if(action==1)active_object_registry->resolve<Level>(get_owner()->level_id())->remove_component(*get_owner(),id());
  if(action==2)get_owner()->destroy();
  if(action==3)active_object_registry->resolve<Level>(get_owner()->level_id())->remove_component(*get_owner(),motor);
  if(action==4)enabled=false;
  return ScriptedCommand3DComponent::calculate_command(dt);
 }
 void command_submitted(CharacterCommandResult3D result)override{
  ++submitted;nested=update(Fixed(1.0/60.0)).status;
  ScriptedCommand3DComponent::command_submitted(result);
  if(action==5)active_object_registry->resolve<Level>(get_owner()->level_id())->remove_component(*get_owner(),id());
 }
};
#ifdef EPOK_TEST_CHARACTER_ANIMATION
class CallbackLocomotion:public SkeletalLocomotionComponent {
public:
 static constexpr uint64_t static_class_id=0x2ac122;
 uint64_t class_id()const override{return static_class_id;}
 unsigned action=0,calls=0;LocomotionStatus nested=LocomotionStatus::NoStep;
 LocomotionSelection select_animation(CharacterMotionSample3D motion,LocomotionState previous,bool finished)override{
  nested=update(Fixed(1.0/60.0)).status;
  if(action==1)get_owner()->destroy();
  if(action==2)enabled=false;
  auto selected=SkeletalLocomotionComponent::select_animation(motion,previous,finished);
  if(action==3)selected.rate=-1.0;
  if(action==4)selected.clip=99;
  if(action==6)get_owner()->data()->animator.model=nullptr;
  return selected;
 }
 void animation_applied(LocomotionSample)override{++calls;if(action==5)get_owner()->destroy();}
};
class CallbackSpriteLocomotion:public SpriteLocomotion2DComponent {
public:
 static constexpr uint64_t static_class_id=0x2ac123;
 uint64_t class_id()const override{return static_class_id;}
 unsigned action=0,calls=0;LocomotionStatus nested=LocomotionStatus::NoStep;
 LocomotionSelection select_animation(CharacterMotionSample2D motion,LocomotionState previous,bool finished)override{
  nested=update(Fixed(1.0/60.0)).status;
  auto selected=SpriteLocomotion2DComponent::select_animation(motion,previous,finished);
  if(action==1)get_owner()->destroy();
  if(action==2)active_object_registry->resolve<Sprite2DComponent>(visual)->clips=nullptr;
  return selected;
 }
 void animation_applied(LocomotionSample)override{++calls;if(action==3)get_owner()->destroy();}
};
#endif
template<class T>static NativeComponentDefault defaults(Object& object,size_t index){
 auto& actor=static_cast<T&>(object);
 if(index==0)return {&actor.root,"Root",true,-1,std::is_base_of_v<Actor2D,T>?SceneComponent2D::static_class_id:SceneComponent3D::static_class_id};
 if constexpr(std::is_base_of_v<Character2D,T>){
  if(index==1)return {&actor.collider,"Collider",false,-1,Collider2DComponent::static_class_id};
  if(index==2)return {&actor.body,"Body",false,-1,CharacterBody2DComponent::static_class_id};
  if(index==3)return {&actor.motor,"Motor",false,-1,CharacterMotor2DComponent::static_class_id};
 }else if constexpr(std::is_base_of_v<Character3D,T>){
  if(index==1)return {&actor.collider,"Collider",false,-1,Collider3DComponent::static_class_id};
  if(index==2)return {&actor.motor,"Motor",false,-1,CharacterMotor3DComponent::static_class_id};
 }else if constexpr(std::is_base_of_v<CameraRig3D,T>){
  if(index==1)return {&actor.camera,"Camera",false,-1,Camera3DComponent::static_class_id};
 }
 if constexpr(std::is_same_v<T,Player3D>)return {&actor.input_source,"Input",false,-1,LocalPlayer3DInputComponent::static_class_id};
 if constexpr(std::is_same_v<T,Player2D>)return {&actor.input_source,"Input",false,-1,LocalPlayer2DInputComponent::static_class_id};
 if constexpr(std::is_same_v<T,NPC3D>)return {&actor.command_source,"AI",false,-1,MoveTo3DComponent::static_class_id};
 if constexpr(std::is_same_v<T,NPC2D>)return {&actor.command_source,"AI",false,-1,MoveTo2DComponent::static_class_id};
 return {};
}
template<class T,class Base>static ClassDescriptor descriptor(ObjectDomain domain,unsigned count=0,NativeComponentDefault(*init)(Object&,size_t)=nullptr){
 constexpr bool component=std::is_base_of_v<ActorComponent,T>;
 auto value=ClassDescriptor{T::static_class_id,Base::static_class_id,component?ObjectFamily::Component:ObjectFamily::Actor,domain,object_domain_bit(domain),uint8_t(component?0:6),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,16>::acquire,&ObjectPool<T,16>::release,count,init};
 value.tick_group=TickGroupOf<T>::value;
 if constexpr(std::is_same_v<T,SceneComponent3D>||std::is_same_v<T,SceneComponent2D>)value.flags|=ObjectClassRoot;
 if constexpr(component){value.component_tick=ComponentCallbacks<T>::tick;value.component_frame=ComponentCallbacks<T>::frame;}
 else{value.component_tick=ActorCallbacks<T>::tick;value.component_frame=ActorCallbacks<T>::frame;}
 return value;
}
namespace epok {
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
 descriptor<Actor3D,Actor>(ObjectDomain::World3D),descriptor<Actor2D,Actor>(ObjectDomain::World2D),
 descriptor<SceneComponent3D,ActorComponent>(ObjectDomain::World3D),descriptor<SceneComponent2D,ActorComponent>(ObjectDomain::World2D),
 descriptor<Collider3DComponent,ActorComponent>(ObjectDomain::World3D),descriptor<Collider2DComponent,ActorComponent>(ObjectDomain::World2D),
 descriptor<CharacterBody2DComponent,ActorComponent>(ObjectDomain::World2D),
 descriptor<CharacterMotor3DComponent,ActorComponent>(ObjectDomain::World3D),descriptor<CharacterMotor2DComponent,ActorComponent>(ObjectDomain::World2D),
 descriptor<Character3D,Actor3D>(ObjectDomain::World3D,3,&defaults<Character3D>),descriptor<Character2D,Actor2D>(ObjectDomain::World2D,4,&defaults<Character2D>),
 {CharacterCommandSource3DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassAbstract},
 {CharacterCommandSource2DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World2D,2,ObjectClassAbstract},
 descriptor<ScriptedCommand3DComponent,CharacterCommandSource3DComponent>(ObjectDomain::World3D),
 descriptor<ScriptedCommand2DComponent,CharacterCommandSource2DComponent>(ObjectDomain::World2D),
 descriptor<MoveTo3DComponent,CharacterCommandSource3DComponent>(ObjectDomain::World3D),
 descriptor<MoveTo2DComponent,CharacterCommandSource2DComponent>(ObjectDomain::World2D),
 descriptor<LocalPlayer3DInputComponent,CharacterCommandSource3DComponent>(ObjectDomain::World3D),
 descriptor<LocalPlayer2DInputComponent,CharacterCommandSource2DComponent>(ObjectDomain::World2D),
 descriptor<CallbackSource,ScriptedCommand3DComponent>(ObjectDomain::World3D),
 descriptor<Player3D,Character3D>(ObjectDomain::World3D,4,&defaults<Player3D>),descriptor<Player2D,Character2D>(ObjectDomain::World2D,5,&defaults<Player2D>),
 descriptor<NPC3D,Character3D>(ObjectDomain::World3D,4,&defaults<NPC3D>),descriptor<NPC2D,Character2D>(ObjectDomain::World2D,5,&defaults<NPC2D>),
 descriptor<Camera3DComponent,ActorComponent>(ObjectDomain::World3D),
 descriptor<CameraRig3D,Actor3D>(ObjectDomain::World3D,2,&defaults<CameraRig3D>),
#ifdef EPOK_TEST_CHARACTER_ANIMATION
 descriptor<Mesh3DComponent,ActorComponent>(ObjectDomain::World3D),
 descriptor<Sprite2DComponent,ActorComponent>(ObjectDomain::World2D),
 descriptor<SkeletalLocomotionComponent,ActorComponent>(ObjectDomain::World3D),
 descriptor<SpriteLocomotion2DComponent,ActorComponent>(ObjectDomain::World2D),
 descriptor<CallbackLocomotion,SkeletalLocomotionComponent>(ObjectDomain::World3D),
 descriptor<CallbackSpriteLocomotion,SpriteLocomotion2DComponent>(ObjectDomain::World2D),
#endif
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
int entity_index(const ActorData* data){for(unsigned i=0;i<used;++i)if(slots+i==data)return int(i);return -1;}
ActorData* DataHandle::get()const{return index<used&&slots[index].alive&&slots[index].generation==generation?slots+index:nullptr;}
DataHandle active_camera(){return current?DataHandle{uint16_t(entity_index(current)),current->generation}:DataHandle{};}
bool set_active_camera(ActorData* data){current=data;return data!=nullptr;}
WorldAffineSample gameplay_world_affine(const ActorData* data){WorldAffineSample value;if(data&&data->alive){value.success=true;for(unsigned i=0;i<3;++i)value.position[i]=data->transform.position[i];const auto yaw=data->transform.rotation[1];value.basis_z[0]=MathLibrary::sine_degrees(yaw);value.basis_z[2]=MathLibrary::cosine_degrees(yaw);value.basis_y[1]=1.0;}return value;}
SpatialHit raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers){return world.raycast(origin,delta,mask,entity_index(ignore),triggers);}
static Aabb bounds(const ActorData& data){Aabb box;for(unsigned i=0;i<3;++i){box.min[i]=data.transform.position[i]-data.collider.half_extents[i];box.max[i]=data.transform.position[i]+data.collider.half_extents[i];}return box;}
MoveResult move_and_slide(ActorData& data,const Fixed* delta,uint32_t mask){auto result=world.move_and_slide(bounds(data),delta,mask);for(unsigned i=0;i<3;++i)data.transform.position[i]+=result.displacement[i];return result;}
SpatialHit query_ground(const ActorData& data,Fixed distance,uint32_t mask){return world.ground(bounds(data),distance,mask);}
}
template<class T>static T* spawn(Level& level){
 auto id=level.spawn_actor(*find_object_class(T::static_class_id),"Repeated name");auto* actor=active_object_registry->resolve<T>(id);assert(actor);
 if constexpr(std::is_base_of_v<Actor3D,T>){assert(used<24);auto& data=slots[used++];data=ActorData{};data.alive=data.active=true;data.transform.scale[0]=data.transform.scale[1]=data.transform.scale[2]=1.0;actor->bind_data(data);actor->root.bind_slot(data);data.collider.enabled=true;data.collider.half_extents[0]=data.collider.half_extents[2]=0.4;data.collider.half_extents[1]=1.0;data.transform.position[1]=1.0;}
 return actor;
}
static void frame(uint16_t buttons,unsigned port=0,bool analog=false,uint8_t x=128,uint8_t y=128){
 input.sample(port,true,buttons,analog,x,y);gameplay_controls().begin_frame();input.begin_tick();gameplay_controls().begin_tick();
}
