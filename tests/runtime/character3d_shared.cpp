// Exercise the reflected motor boundary against the frozen complete-game motor.
#include <cassert>
#include <cstdio>
#include "../../runtime/character3d.hpp"
#include "character3d_complete_reference.hpp"
using namespace epok;
static CollisionWorld<Fixed,8,4> world;
static ActorData slots[4];
static unsigned sweeps=0,grounds=0;
static Aabb bounds(const ActorData& actor){
    Aabb box;for(unsigned i=0;i<3;++i){const auto center=actor.transform.position[i]+actor.collider.center[i];box.min[i]=center-actor.collider.half_extents[i];box.max[i]=center+actor.collider.half_extents[i];}return box;
}
namespace epok {
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {Actor3D::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::World3D,0,6,&object_construct<Actor3D>,&object_destruct,sizeof(Actor3D),alignof(Actor3D),&ObjectPool<Actor3D,8>::acquire,&ObjectPool<Actor3D,8>::release},
 {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot},
 {CharacterMotor3DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<CharacterMotor3DComponent>,&object_destruct,sizeof(CharacterMotor3DComponent),alignof(CharacterMotor3DComponent),&ObjectPool<CharacterMotor3DComponent,4>::acquire,&ObjectPool<CharacterMotor3DComponent,4>::release},
 {SpatialQueryContextComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<SpatialQueryContextComponent>,&object_destruct,sizeof(SpatialQueryContextComponent),alignof(SpatialQueryContextComponent),&ObjectPool<SpatialQueryContextComponent,4>::acquire,&ObjectPool<SpatialQueryContextComponent,4>::release},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
int entity_index(const ActorData* data){for(unsigned i=0;i<4;++i)if(data==slots+i)return int(i);return -1;}
MoveResult move_and_slide(ActorData& actor,const Fixed* delta,uint32_t mask){
    ++sweeps;const auto result=world.move_and_slide(bounds(actor),delta,mask);
    for(unsigned i=0;i<3;++i)actor.transform.position[i]+=result.displacement[i];return result;
}
SpatialHit query_ground(const ActorData& actor,Fixed distance,uint32_t mask){++grounds;return world.ground(bounds(actor),distance,mask);}
void SpatialQueryBatch::ensure()const{ready=true;}
void SpatialQueryBatch::sync(ActorData&){ensure();}
MoveResult SpatialQueryBatch::move_and_slide(ActorData& actor,const Fixed* delta,uint32_t mask){sync(actor);return epok::move_and_slide(actor,delta,mask);}
}
static void slab(unsigned slot,Fixed x,Fixed y,Fixed z,Fixed hx,Fixed hy,Fixed hz){
    Collider collider;collider.enabled=true;collider.half_extents[0]=hx;collider.half_extents[1]=hy;collider.half_extents[2]=hz;
    auto transform=Affine<Fixed>::identity();transform.values[0][3]=x;transform.values[1][3]=y;transform.values[2][3]=z;world.set(slot,collider,transform);
}
static void body_defaults(ActorData& body){
    body.transform={};body.alive=body.active=true;body.collider.enabled=true;
    body.collider.center[1]=0.92;body.collider.half_extents[0]=body.collider.half_extents[2]=0.28;body.collider.half_extents[1]=0.92;
}
int main(){
    ObjectRegistryStorage<96> registry;active_object_registry=&registry;Level level,other;assert(level.bind(registry)&&other.bind(registry));
    auto spawn=[&](Level& owner){const auto id=owner.spawn_actor(*find_object_class(Actor3D::static_class_id),"Same name");auto* actor=registry.resolve<Actor3D>(id);assert(actor);return actor;};
    auto* actor=spawn(level);auto& body=slots[0];body_defaults(body);actor->bind_data(body);actor->root.bind_slot(body);
    auto* motor=level.add_component<CharacterMotor3DComponent>(*actor,"Motor");assert(motor);motor->automatic_tick=false;motor->step_height=0.0;motor->measure_speed=false;
    auto* context_owner=spawn(level);auto* context=level.add_component<SpatialQueryContextComponent>(*context_owner,"Context");assert(context);context->automatic_reset=false;
    const Fixed dt(1.0/60.0);unsigned comparisons=0,skipped=0;
    for(unsigned scenario=0;scenario<4;++scenario){
        world.clear();slab(0,0.0,-0.25,0.0,10.0,0.25,10.0);
        if(scenario==1)slab(1,2.0,2.0,0.0,0.1,2.0,4.0);
        if(scenario==2)slab(1,0.0,2.0,0.0,8.0,0.1,4.0);
        body_defaults(body);motor->reset_motion();context->reset_queries();auto old=body;
        character3d::State legacy{};complete_reference::Cache cache{};uint32_t revision=1;
        for(unsigned frame=0;frame<360;++frame){
            GameplayVector2 desired{};if(frame>=30&&frame<120)desired={6.0,0.0};if(frame>=180&&frame<240)desired={-3.0,2.0};
            if(frame==150){body.transform.position[0]=old.transform.position[0]=0.0;++revision;}
            if(frame==270){body.transform.rotation[2]=old.transform.rotation[2]=10.0;++revision;}
            if(scenario==3&&frame==300){world.clear();++revision;}
            const Fixed facing=frame<180?Fixed(90.0):Fixed(300.0);assert(motor->set_facing(facing));
            CharacterCommand3D command;command.velocity=desired;command.jump=frame==60||frame==210;motor->submit(command);
            sweeps=grounds=0;const auto sample=motor->simulate_shared(dt,context->id(),revision);const unsigned actual=sweeps;skipped+=actual==0;
            assert(grounds==0&&sample.speed==0.0);
            sweeps=grounds=0;const bool jumped=complete_reference::original(old,legacy,cache,desired,facing,command.jump,dt,revision);
            assert(sample.status==CharacterMotionStatus::Applied&&sample.jumped==jumped&&sweeps==actual&&grounds==0);
            const Fixed velocity[]={sample.velocity.x,sample.velocity.y,sample.velocity.z};
            for(unsigned i=0;i<3;++i){assert(body.transform.position[i]==old.transform.position[i]);assert(body.transform.rotation[i]==old.transform.rotation[i]);assert(velocity[i]==legacy.velocity[i]);}
            assert(sample.grounded==legacy.grounded&&sample.facing==legacy.facing);++comparisons;
        }
    }
    assert(comparisons==1440&&skipped==524);
    auto reset=[&](){world.clear();slab(0,0.0,-0.25,0.0,10.0,0.25,10.0);body_defaults(body);motor->gravity=24.0;motor->collision_mask=0xffffffffu;motor->reset_motion();context->reset_queries();};
    auto step=[&](uint32_t revision){sweeps=0;const auto result=motor->simulate_shared(dt,context->id(),revision);assert(result.status==CharacterMotionStatus::Applied);return result;};
    reset();assert(step(7).grounded&&sweeps==1);step(7);assert(sweeps==0);
    context->reset_queries();step(7);assert(sweeps==1);step(7);assert(sweeps==0);
    motor->gravity=25.0;step(7);assert(sweeps==1);step(7);assert(sweeps==0);
    step(0);assert(sweeps==1);step(0);assert(sweeps==1);
    step(8);assert(sweeps==1);body.transform.position[0]=1.0;step(8);assert(sweeps==1);
    body.transform.rotation[2]=10.0;step(8);assert(sweeps==1);
    reset();step(9);CharacterCommand3D jump;jump.jump=true;motor->submit(jump);assert(motor->set_facing(45.0));
    const auto before=body.transform;assert(motor->simulate_shared(0.0,context->id(),9).status==CharacterMotionStatus::NoStep);
    assert(motor->simulate_shared(dt,{},9).status==CharacterMotionStatus::InvalidContext);
    motor->gravity=-1.0;assert(motor->simulate_shared(dt,context->id(),9).status==CharacterMotionStatus::InvalidConfiguration);
    assert(body.transform.position[1]==before.position[1]&&body.transform.rotation[1]==before.rotation[1]);
    motor->gravity=24.0;assert(step(9).jumped&&body.transform.rotation[1]==Fixed(45.0));
    motor->clear_velocity();assert(!motor->motion().grounded&&motor->motion().velocity.y==0.0);assert(step(9).velocity.y<0.0);
    assert(!motor->set_facing(Fixed(400000)));
    reset();step(11);motor->clear_velocity();assert(motor->motion().grounded);step(11);assert(sweeps==0);
    motor->submit(jump);assert(level.set_active(context_owner->id(),false));assert(motor->simulate_shared(dt,context->id(),11).status==CharacterMotionStatus::InvalidContext);
    assert(level.set_active(context_owner->id(),true));assert(step(11).jumped);
    auto* foreign_owner=spawn(other);auto* foreign=other.add_component<SpatialQueryContextComponent>(*foreign_owner,"Foreign");assert(foreign);
    assert(motor->simulate_shared(dt,foreign->id(),11).status==CharacterMotionStatus::InvalidContext);
    reset();step(12);step(12);assert(sweeps==0);motor->simulate(dt);step(12);assert(sweeps==1);
    const auto stale=context->id();assert(level.destroy_actor(context_owner->id()));assert(motor->simulate_shared(dt,stale,12).status==CharacterMotionStatus::InvalidContext);
    context_owner=spawn(level);context=level.add_component<SpatialQueryContextComponent>(*context_owner,"Reused");assert(context&&context->id()!=stale);context->automatic_reset=false;step(12);assert(sweeps==1);
    auto* second_actor=spawn(level);auto& second_body=slots[1];body_defaults(second_body);second_body.transform.position[0]=5.0;second_actor->bind_data(second_body);second_actor->root.bind_slot(second_body);
    auto* second=level.add_component<CharacterMotor3DComponent>(*second_actor,"Motor");assert(second);second->automatic_tick=false;second->step_height=0.0;
    assert(second->simulate_shared(dt,context->id(),12).grounded);sweeps=0;assert(second->simulate_shared(dt,context->id(),12).grounded&&sweeps==0);
    context->reset_queries();step(12);assert(sweeps==1);sweeps=0;second->simulate_shared(dt,context->id(),12);assert(sweeps==1);
    CharacterCommand3D run;run.velocity.x=6.0;second->submit(run);assert(second->simulate_shared(dt,context->id(),12).speed>0.0);second->measure_speed=false;assert(second->simulate_shared(dt,context->id(),12).speed==0.0);
    reset();step(20);step(20);assert(sweeps==0);
    slots[2]=body;actor->bind_data(slots[2]);actor->root.bind_slot(slots[2]);
    sweeps=0;assert(motor->simulate_shared(dt,context->id(),20).status==CharacterMotionStatus::Applied);assert(sweeps==1);
    ++slots[2].generation;actor->bind_data(slots[2]);actor->root.bind_slot(slots[2]);
    sweeps=0;assert(motor->simulate_shared(dt,context->id(),20).status==CharacterMotionStatus::Applied);assert(sweeps==1);
    slots[2].transform.scale[0]=2.0;sweeps=0;motor->simulate_shared(dt,context->id(),20);assert(sweeps==1);
    slots[3].alive=slots[3].active=true;context_owner->bind_data(slots[3]);context_owner->root.bind_slot(slots[3]);
    slots[2].parent=3;sweeps=0;motor->simulate_shared(dt,context->id(),20);assert(sweeps==1);
    const auto dead_motor=motor->id();assert(level.destroy_actor(actor->id())&&!registry.get(dead_motor));
    level.end_play_all(EndPlayReason::LevelUnloaded);other.end_play_all(EndPlayReason::LevelUnloaded);
    std::printf("Shared motor: %u exact complete-game steps / %u idle skips, context epochs, tuning, poses, jumps, clearing, identity reuse and independent consumers passed\n",comparisons,skipped);
}
