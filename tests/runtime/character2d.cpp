// Compare the shared motor with the original platformer sequence on the real solver.
#include <cassert>
#include <cstdio>
#include "../../runtime/character2d.hpp"
#include "../../runtime/world2d_service.hpp"
using namespace epok;
template<class T,class Base> static ClassDescriptor descriptor(ObjectFamily family,unsigned count=0,NativeComponentDefault(*defaults)(Object&,size_t)=nullptr){
 auto value=ClassDescriptor{T::static_class_id,Base::static_class_id,family,ObjectDomain::World2D,2,0,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,16>::acquire,&ObjectPool<T,16>::release,count,defaults};
 value.tick_group=TickGroupOf<T>::value;
 if constexpr(std::is_same_v<T,Collider2DComponent>)value.flags|=ObjectClassMultiple;
 if constexpr(std::is_same_v<T,SceneComponent2D>)value.flags|=ObjectClassRoot;
 if constexpr(std::is_base_of_v<ActorComponent,T>){value.component_tick=ComponentCallbacks<T>::tick;value.component_frame=ComponentCallbacks<T>::frame;}
 else{value.component_tick=ActorCallbacks<T>::tick;value.component_frame=ActorCallbacks<T>::frame;}
 return value;
}
static NativeComponentDefault defaults(Object& object,size_t index){
 auto& actor=static_cast<Character2D&>(object);
 switch(index){
 case 0:return {&actor.root,"Root",true,-1,SceneComponent2D::static_class_id};
 case 1:return {&actor.collider,"Collider",false,-1,Collider2DComponent::static_class_id};
 case 2:return {&actor.body,"Body",false,-1,CharacterBody2DComponent::static_class_id};
 default:return {&actor.motor,"Motor",false,-1,CharacterMotor2DComponent::static_class_id};
 }
}
namespace epok {
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
ActorData* DataHandle::get()const{return nullptr;}
int entity_index(const ActorData*){return -1;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
 descriptor<Actor2D,Actor>(ObjectFamily::Actor),
 descriptor<SceneComponent2D,ActorComponent>(ObjectFamily::Component),
 descriptor<Collider2DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<CharacterBody2DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<CharacterMotor2DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<Path2DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<PathFollow2DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<Character2D,Actor2D>(ObjectFamily::Actor,4,&defaults),
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
int main(){
 ObjectRegistryStorage<128> registry;active_object_registry=&registry;Level level;assert(level.bind(registry));world2d::service.reset();world2d::install();
 auto legacy_id=level.spawn_actor(*find_object_class(Actor2D::static_class_id),"Same name");auto* legacy=registry.resolve<Actor2D>(legacy_id);assert(legacy);
 auto* legacy_shape=level.add_component<Collider2DComponent>(*legacy,"Collider");auto* legacy_body=level.add_component<CharacterBody2DComponent>(*legacy,"Body");assert(legacy_shape&&legacy_body);
 legacy_shape->set_box_size(16.0,16.0);legacy_shape->shape.layer=2;legacy_body->collision_mask=2;
 auto id=level.spawn_actor(*find_object_class(Character2D::static_class_id),"Same name");auto* actor=registry.resolve<Character2D>(id);assert(actor&&actor->component_count()==4);
 actor->collider.set_box_size(16.0,16.0);actor->collider.shape.layer=1;actor->body.collision_mask=1;actor->motor.automatic_tick=false;
 Collider2DComponent* moving[2]{};PathFollow2DComponent* followers[2]{};
 for(unsigned layer=1;layer<=2;++layer){
  for(unsigned part=0;part<5;++part){
   auto geometry_id=level.spawn_actor(*find_object_class(Actor2D::static_class_id),"Geometry");auto* geometry=registry.resolve<Actor2D>(geometry_id);assert(geometry);
   auto* shape=level.add_component<Collider2DComponent>(*geometry,"Collider");assert(shape);shape->shape.layer=layer;
   if(part==0){shape->set_box_size(1000.0,16.0);geometry->root.transform.position[1]=16.0;}
   else if(part<3){shape->set_box_size(16.0,80.0);geometry->root.transform.position[0]=part==1?64.0:-64.0;geometry->root.transform.position[1]=-24.0;}
   else if(part==3){shape->set_box_size(160.0,8.0);geometry->root.transform.position[1]=-56.0;}
   else{shape->set_box_size(48.0,4.0);geometry->root.transform.position[1]=-24.0;shape->shape.one_way=true;moving[layer-1]=shape;}
  }
 }
 const Fixed dt(1.0/60.0);unsigned grounded=0,walls=0,ceilings=0,jumps=0,platform_contacts=0,carried=0;
 for(unsigned frame=0;frame<720;++frame){
  time.delta_raw=dt.raw();const bool riding=frame>=360&&frame<480;
  if(frame==360){
   actor->root.transform.position[0]=legacy->root.transform.position[0]=0.0;
   actor->root.transform.position[1]=legacy->root.transform.position[1]=-34.0;
   actor->motor.reset_motion();legacy_body->set_velocity(0.0,0.0);legacy_body->on_floor=legacy_body->on_wall=legacy_body->on_ceiling=false;
   static const Fixed path_points[]={Fixed(0.0),Fixed(0.0),Fixed(100.0),Fixed(0.0)};
   static const Fixed path_lengths[]={Fixed(0.0),Fixed(100.0)};
   for(unsigned i=0;i<2;++i){
    auto rail_id=level.spawn_actor(*find_object_class(Actor2D::static_class_id),"Rail");auto* rail=registry.resolve<Actor2D>(rail_id);assert(rail);
    rail->root.transform.position[1]=-24.0;auto* path=level.add_component<Path2DComponent>(*rail,"Path");assert(path);
    path->points=path_points;path->lengths=path_lengths;path->point_count=2;
    auto* platform=static_cast<Actor2D*>(moving[i]->get_owner());platform->root.transform.position[0]=0.0;
    followers[i]=level.add_component<PathFollow2DComponent>(*platform,"Follow");assert(followers[i]);followers[i]->path=rail_id;followers[i]->speed=8.0;
   }
  }
  const Fixed speed=riding?Fixed(0.0):(frame%160<80?Fixed(96.0):Fixed(-96.0));
  const bool jump=!riding&&frame%90==4;
  if(frame<360){for(auto* platform:moving){auto* owner=static_cast<Actor2D*>(platform->get_owner());owner->root.transform.position[0]=Fixed(int(frame%64)-32,0)/Fixed(2.0);}}
  else for(auto* follower:followers)follower->moving=riding;
  Fixed fall=legacy_body->velocity_y()+Fixed(720.0)*dt;
  if(legacy_body->is_on_floor()&&jump)fall=-264.0;
  legacy_body->set_velocity(speed,fall);legacy_body->move_and_slide();
  actor->submit_command({{speed,0.0},jump});const auto sample=actor->motor.simulate(dt);assert(sample.status==CharacterMotionStatus::Applied);
  for(unsigned axis=0;axis<2;++axis){
   assert(actor->root.transform.position[axis]==legacy->root.transform.position[axis]);
   assert(actor->body.velocity[axis]==legacy_body->velocity[axis]);
   assert(actor->body.last_motion[axis]==legacy_body->last_motion[axis]);
  }
  assert(actor->body.on_floor==legacy_body->on_floor&&actor->body.on_wall==legacy_body->on_wall&&actor->body.on_ceiling==legacy_body->on_ceiling);
  grounded+=sample.grounded;walls+=sample.on_wall;ceilings+=sample.on_ceiling;jumps+=sample.jumped;
  platform_contacts+=sample.grounded&&actor->body.last_collider==moving[0]->owner_id();
  const auto before_carry=actor->root.transform.position[0];
  world2d::service.step(dt,&level);
  carried+=riding&&actor->root.transform.position[0]!=before_carry;
  for(unsigned axis=0;axis<2;++axis)assert(actor->root.transform.position[axis]==legacy->root.transform.position[axis]);
 }
 assert(grounded&&walls&&ceilings&&jumps&&platform_contacts&&carried);
 // A sensor before a solid shape must not invalidate the body.
 auto sensor_owner_id=level.spawn_actor(*find_object_class(Actor2D::static_class_id),"Sensor first");auto* sensor_owner=registry.resolve<Actor2D>(sensor_owner_id);assert(sensor_owner);
 auto* sensor=level.add_component<Collider2DComponent>(*sensor_owner,"Sensor");assert(sensor);sensor->shape.trigger=true;
 auto* solid=level.add_component<Collider2DComponent>(*sensor_owner,"Solid");assert(solid);
 auto* body=level.add_component<CharacterBody2DComponent>(*sensor_owner,"Body");auto* motor=level.add_component<CharacterMotor2DComponent>(*sensor_owner,"Motor");assert(body&&motor);motor->automatic_tick=false;
 sensor_owner->root.transform.position[0]=300.0;body->on_floor=true;
 motor->submit({{},true});motor->submit({});const auto y=sensor_owner->root.transform.position[1];
 assert(motor->simulate(0.0).status==CharacterMotionStatus::NoStep&&sensor_owner->root.transform.position[1]==y);
 motor->gravity=-1.0;assert(motor->simulate(dt).status==CharacterMotionStatus::InvalidConfiguration&&sensor_owner->root.transform.position[1]==y);
 motor->gravity=720.0;assert(motor->simulate(dt).jumped);
 motor->body=actor->body.id();assert(motor->simulate(dt).status==CharacterMotionStatus::InvalidOwner);motor->body={};
 body->up_direction[0]=Fixed(INT32_MIN,Fixed::RAW);assert(motor->simulate(dt).status==CharacterMotionStatus::InvalidConfiguration);body->up_direction[0]=0.0;
 body->floating=true;motor->submit({{40.0,60.0},true});const auto floating=motor->simulate(dt);assert(floating.status==CharacterMotionStatus::Applied&&!floating.jumped&&floating.velocity.x==40.0&&floating.velocity.y==60.0);
 level.end_play_all(EndPlayReason::LevelUnloaded);assert(world2d::bodies.count==0&&world2d::colliders.count==0);
 std::printf("Character2D: 720 Q12 legacy comparisons with real floor/wall/ceiling, one-way/moving platform solver; grounded=%u walls=%u ceilings=%u jumps=%u platform=%u carried=%u; latching, invalid body, sensors and floating motion passed\n",grounded,walls,ceilings,jumps,platform_contacts,carried);
}
