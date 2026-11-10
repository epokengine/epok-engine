// Exercise the reusable motor through the real Level and component lifecycle.
#include <cassert>
#include <cstdio>
#include "../../runtime/character3d.hpp"
using namespace epok;
static CollisionWorld<Fixed,4,4> world;
static ActorData slots[2];
namespace epok {
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing()const{return false;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {Actor3D::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::World3D,0,6,&object_construct<Actor3D>,&object_destruct,sizeof(Actor3D),alignof(Actor3D),&ObjectPool<Actor3D,3>::acquire,&ObjectPool<Actor3D,3>::release},
 {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot},
 {UIActor::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::UI,0,6,&object_construct<UIActor>,&object_destruct,sizeof(UIActor),alignof(UIActor),&ObjectPool<UIActor,1>::acquire,&ObjectPool<UIActor,1>::release},
 {UIComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassAbstract},
 {RectTransformComponent::static_class_id,UIComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassRoot},
 {CharacterMotor3DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<CharacterMotor3DComponent>,&object_destruct,sizeof(CharacterMotor3DComponent),alignof(CharacterMotor3DComponent),&ObjectPool<CharacterMotor3DComponent,3>::acquire,&ObjectPool<CharacterMotor3DComponent,3>::release},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
int entity_index(const ActorData* data){for(unsigned i=0;i<2;++i)if(data==slots+i)return int(i);return -1;}
static Aabb bounds(const ActorData& actor){Aabb box;for(unsigned i=0;i<3;++i){box.min[i]=actor.transform.position[i]-actor.collider.half_extents[i];box.max[i]=actor.transform.position[i]+actor.collider.half_extents[i];}return box;}
MoveResult move_and_slide(ActorData& actor,const Fixed* movement,uint32_t mask){auto result=world.move_and_slide(bounds(actor),movement,mask);for(unsigned i=0;i<3;++i)actor.transform.position[i]+=result.displacement[i];return result;}
SpatialHit query_ground(const ActorData& actor,Fixed distance,uint32_t mask){return world.ground(bounds(actor),distance,mask);}
}
int main(){
 ObjectRegistryStorage<32> registry;active_object_registry=&registry;Level level;assert(level.bind(registry));
 Collider floor;floor.enabled=true;floor.half_extents[0]=floor.half_extents[2]=40.0;floor.half_extents[1]=0.25;
 auto transform=Affine<Fixed>::identity();transform.values[1][3]=-0.25;world.set(0,floor,transform);
 CharacterMotor3DComponent* motors[2];ObjectId actors[2];const Fixed dt(1.0/60.0);
 for(unsigned i=0;i<2;++i){
  actors[i]=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Same name");auto* actor=registry.resolve<Actor3D>(actors[i]);assert(actor);
  auto& data=slots[i];data.alive=data.active=true;data.collider.enabled=true;
  data.collider.half_extents[0]=data.collider.half_extents[2]=0.4;data.collider.half_extents[1]=1.0;
  data.transform.position[0]=Fixed(int(i)*10);data.transform.position[1]=1.0;data.transform.rotation[1]=Fixed(int(i)*90);
  actor->bind_data(data);actor->root.bind_slot(data);motors[i]=level.add_component<CharacterMotor3DComponent>(*actor,"Motor");assert(motors[i]);
  assert(motors[i]->motion().facing==data.transform.rotation[1]);motors[i]->automatic_tick=false;
  assert(!level.add_component<CharacterMotor3DComponent>(*actor,"Duplicate"));
  assert(motors[i]->simulate(dt).grounded);
 }
 const Fixed rest[2]={slots[0].transform.position[1],slots[1].transform.position[1]};
 auto* first=motors[0];CharacterCommand3D jump;jump.jump=true;first->submit(jump);first->submit({});
 assert(first->simulate(0.0).status==CharacterMotionStatus::NoStep);
 first->gravity=-1.0;assert(first->simulate(dt).status==CharacterMotionStatus::InvalidConfiguration);assert(slots[0].transform.position[1]==rest[0]);
 first->gravity=24.0;assert(first->simulate(dt).jumped); // Neither a zero step nor invalid tuning consumes the edge.
 const auto position=slots[0].transform.position[1];assert(!first->simulate(dt).jumped&&slots[0].transform.position[1]>position);
 assert(motors[1]->motion().grounded&&slots[1].transform.position[1]==rest[1]); // Per-instance state, independent of display name.
 registry.resolve<Actor>(actors[0])->set_name("Renamed");registry.resolve<Actor>(actors[1])->set_name("Renamed");
 first->stop();const Fixed vertical=first->motion().velocity.y;assert(first->simulate(dt).velocity.y<vertical); // Stop commands, retain physics.
 const auto before=slots[0].transform.position[1];level.tick(dt);assert(slots[0].transform.position[1]==before);
 first->automatic_tick=true;level.tick(dt);assert(slots[0].transform.position[1]!=before);
 assert(level.set_active(actors[0],false));const auto disabled=slots[0].transform.position[1];
 assert(first->simulate(dt).status==CharacterMotionStatus::InvalidOwner);level.tick(dt);assert(slots[0].transform.position[1]==disabled);
 assert(level.set_active(actors[0],true));first->automatic_tick=false;
 for(unsigned i=0;i<180;++i)first->simulate(dt);assert(first->motion().grounded);
 first->submit(jump);assert(level.set_active(actors[0],false)&&level.set_active(actors[0],true));assert(!first->simulate(dt).jumped);
 slots[0].collider.enabled=false;assert(first->simulate(dt).status==CharacterMotionStatus::InvalidOwner);slots[0].collider.enabled=true;
 const ObjectId old_motor=first->id();assert(level.destroy_actor(actors[0])&&!registry.get(old_motor));
 const auto ui=level.spawn_actor(*find_object_class(UIActor::static_class_id),"UI");assert(ui.valid());
 assert(!level.add_component<CharacterMotor3DComponent>(*registry.resolve<UIActor>(ui),"Wrong domain"));
 level.end_play_all(EndPlayReason::LevelUnloaded);
 std::puts("Character motor lifecycle: independent identities, jump latching, pause, invalid tuning, automatic/manual tick, disable, collider and owner-domain checks passed");
}
