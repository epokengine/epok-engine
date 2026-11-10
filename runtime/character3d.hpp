#pragma once
// World-unit 3D locomotion. Command production, animation and cameras are separate.
#define EPOK_INCLUDE_FROM_CHARACTER3D 1
#include "gameplay_api.hpp"
#undef EPOK_INCLUDE_FROM_CHARACTER3D

namespace epok {
namespace detail {template<class Motor> struct CharacterCommandLease;}
enum class CharacterMotionStatus:uint8_t {Applied,NoStep,InvalidOwner,InvalidConfiguration,InvalidContext};
struct EPOK_VALUE(Id="47b81129-2766-5c7b-854b-2f71ed567cbe") CharacterCommand3D {
    GameplayVector2 velocity{}; // Desired world X/Z velocity, in world units/second.
    Fixed heading=0.0;
    bool turn=false,jump=false;
};
struct EPOK_VALUE(Id="e53f95c1-6315-50c3-8f77-1c0dd6e26353") CharacterMotionSample3D {
    CharacterMotionStatus status=CharacterMotionStatus::NoStep;
    GameplayVector3 velocity{},displacement{};
    Fixed facing=0.0,speed=0.0;
    bool grounded=false,jumped=false,blocked=false,unresolved_overlap=false;
};
namespace character3d {
struct Tuning {
    Fixed gravity=24.0,jump_speed=9.0,ground_stick=-0.05,step_height=0.6;
    Fixed ground_response=12.0,air_response=3.0,turn_rate=540.0;
    uint32_t collision_mask=0xffffffffu;
};
struct State {
    Fixed velocity[3]={},facing=0.0;
    bool grounded=false;
};
inline bool bounded(Fixed value,int32_t maximum){const int64_t raw=value.raw();return raw>=-int64_t(maximum)*4096&&raw<=int64_t(maximum)*4096;}
inline bool valid(const Tuning& t,const State& state,const CharacterCommand3D& command,Fixed dt){
    return dt>0.0&&dt<=1.0&&t.gravity>=0.0&&t.jump_speed>=0.0&&t.step_height>=0.0
        &&t.ground_response>=0.0&&t.air_response>=0.0&&t.turn_rate>=0.0
        &&t.ground_stick<=0.0&&t.ground_stick>=-8.0
        &&bounded(t.gravity,4096)&&bounded(t.jump_speed,4096)&&bounded(t.step_height,64)
        &&bounded(t.ground_response,4096)&&bounded(t.air_response,4096)&&bounded(t.turn_rate,360000)
        &&bounded(command.velocity.x,4096)&&bounded(command.velocity.y,4096)
        &&bounded(state.velocity[0],8192)&&bounded(state.velocity[1],8192)&&bounded(state.velocity[2],8192)
        &&int64_t(command.heading.raw())-state.facing.raw()>=INT32_MIN
        &&int64_t(command.heading.raw())-state.facing.raw()<=INT32_MAX;
}
// The operation order intentionally matches the existing third-person starter.
// Native collision queries own contact/overlap behavior; this adds no physics world.
struct WorldQueries {
    MoveResult move(ActorData& body,const Fixed* displacement,uint32_t mask){return move_and_slide(body,displacement,mask);}
    SpatialHit ground(const ActorData& body,Fixed distance,uint32_t mask){return query_ground(body,distance,mask);}
};
// A query adapter can reuse an already synchronized world without changing the
// motor's integration/contact rules. Callers may defer the optional speed work
// when another project action owns animation for this step.
template<class Queries>
inline CharacterMotionSample3D advance(ActorData& body,State& state,const Tuning& t,const CharacterCommand3D& command,Fixed dt,Queries& queries,bool measure_speed=true){
    CharacterMotionSample3D sample;
    sample.velocity={state.velocity[0],state.velocity[1],state.velocity[2]};sample.facing=state.facing;sample.grounded=state.grounded;
    if(dt.raw()==0)return sample;
    if(!valid(t,state,command,dt)){sample.status=CharacterMotionStatus::InvalidConfiguration;return sample;}
    if(command.turn)state.facing=MathLibrary::move_toward_degrees(state.facing,command.heading,t.turn_rate*dt);
    Fixed response=(state.grounded?t.ground_response:t.air_response)*dt;
    if(response>Fixed(1.0))response=1.0;
    state.velocity[0]+=(command.velocity.x-state.velocity[0])*response;
    state.velocity[2]+=(command.velocity.y-state.velocity[2])*response;
    const bool jumped=state.grounded&&command.jump;
    if(state.grounded)state.velocity[1]=jumped?t.jump_speed:Fixed(0.0);
    else state.velocity[1]-=t.gravity*dt;
    const Fixed fall=state.grounded&&!jumped?t.ground_stick:state.velocity[1]*dt;
    const Fixed movement[3]={state.velocity[0]*dt,fall,state.velocity[2]*dt};
    const Fixed before[3]={body.transform.position[0],body.transform.position[1],body.transform.position[2]};
    const bool was_grounded=state.grounded;
    const auto result=queries.move(body,movement,t.collision_mask);
    state.grounded=result.grounded;
    if(state.velocity[1]>Fixed(0.0)&&result.displacement[1]+Fixed(0.001)<fall)state.velocity[1]=0.0;
    if(t.step_height>0.0&&result.blocked&&state.grounded&&!jumped){
        const Fixed remaining[3]={movement[0]-result.displacement[0],0.0,movement[2]-result.displacement[2]};
        if(remaining[0].raw()||remaining[2].raw()){
            const Fixed lift[3]={0.0,t.step_height,0.0};
            const auto lifted=queries.move(body,lift,t.collision_mask);
            queries.move(body,remaining,t.collision_mask);
            const Fixed settle[3]={0.0,-lifted.displacement[1]+t.ground_stick,0.0};
            state.grounded=queries.move(body,settle,t.collision_mask).grounded;
        }
    }else if(t.step_height>0.0&&was_grounded&&!state.grounded&&!jumped){
        const auto below=queries.ground(body,t.step_height,t.collision_mask);
        if(below){const Fixed drop[3]={0.0,-(t.step_height*below.fraction)+t.ground_stick,0.0};state.grounded=queries.move(body,drop,t.collision_mask).grounded;}
    }
    body.transform.rotation[1]=state.facing;
    sample.status=CharacterMotionStatus::Applied;sample.velocity={state.velocity[0],state.velocity[1],state.velocity[2]};
    sample.displacement={body.transform.position[0]-before[0],body.transform.position[1]-before[1],body.transform.position[2]-before[2]};
    sample.facing=state.facing;if(measure_speed)sample.speed=MathLibrary::length2(sample.displacement.x/dt,sample.displacement.z/dt);
    sample.grounded=state.grounded;sample.jumped=jumped;sample.blocked=result.blocked;sample.unresolved_overlap=result.unresolved_overlap;
    return sample;
}
inline CharacterMotionSample3D advance(ActorData& body,State& state,const Tuning& t,const CharacterCommand3D& command,Fixed dt){
    WorldQueries queries;return advance(body,state,t,command,dt,queries);
}
}

class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="34a2d5d2-b0bb-5faf-8dc4-7f34f07f9c95") CharacterMotor3DComponent:public ActorComponent {
    friend struct detail::CharacterCommandLease<CharacterMotor3DComponent>;
    uint32_t command_revision_=0;
    void command_changed(){if(!++command_revision_)++command_revision_;}
    character3d::State state_{};
    CharacterCommand3D command_{};
    CharacterMotionSample3D sample_{};
    struct SupportCache {
        Fixed position[3]{},rotation[3]{},scale[3]{};
        character3d::Tuning tuning{};
        const ObjectRegistry* registry=nullptr;
        const ActorData* body=nullptr;
        ObjectId context{},owner{};
        uint32_t revision=0,epoch=0,generation=0;
        int32_t parent=-1;
        bool valid=false;
    } support_{};
    static bool same_tuning(const character3d::Tuning& a,const character3d::Tuning& b){
        return a.gravity==b.gravity&&a.jump_speed==b.jump_speed&&a.ground_stick==b.ground_stick
            &&a.step_height==b.step_height&&a.ground_response==b.ground_response
            &&a.air_response==b.air_response&&a.turn_rate==b.turn_rate&&a.collision_mask==b.collision_mask;
    }
    ActorData* available_body(){
        auto* actor=get_owner();auto* data=actor?actor->data():nullptr;
        return !m_ended&&active_object_registry&&active_object_registry->get(id())==this
            &&actor&&!actor->pending_destruction()&&actor->is_a(Actor3D::static_class_id)
            &&is_active(actor)&&data&&data->collider.enabled?data:nullptr;
    }
    CharacterMotionSample3D fail(CharacterMotionStatus status){
        support_.valid=false;sample_={};sample_.status=status;
        sample_.velocity={state_.velocity[0],state_.velocity[1],state_.velocity[2]};
        sample_.facing=state_.facing;sample_.grounded=state_.grounded;return sample_;
    }
    struct Queries {
        SpatialQueryBatch* batch=nullptr;
        MoveResult(*sweep)(SpatialQueryBatch*,ActorData&,const Fixed*,uint32_t)=nullptr;
        bool stable=false;Fixed stick=0.0;
        MoveResult move(ActorData& body,const Fixed* delta,uint32_t mask){
            if(stable&&delta[0].raw()==0&&delta[2].raw()==0&&delta[1]==stick){MoveResult result;result.grounded=true;return result;}
            return sweep(batch,body,delta,mask);
        }
        SpatialHit ground(const ActorData& body,Fixed distance,uint32_t mask){return query_ground(body,distance,mask);}
    };
    CharacterMotionSample3D simulate_body(ActorData& body,Fixed dt,Queries queries,ObjectId context,uint32_t epoch,uint32_t revision){
        const character3d::Tuning tuning{gravity,jump_speed,ground_stick,step_height,ground_response,air_response,turn_rate,collision_mask};
        bool stable=context.valid()&&revision&&support_.valid&&state_.grounded&&ground_stick<0.0
            &&support_.registry==active_object_registry&&support_.context==context
            &&support_.owner==owner_id()&&support_.body==&body&&support_.generation==body.generation&&support_.parent==body.parent
            &&support_.revision==revision&&support_.epoch==epoch&&same_tuning(support_.tuning,tuning);
        Fixed prior_rotation[3];
        for(unsigned i=0;i<3;++i){
            stable&=support_.position[i]==body.transform.position[i]&&support_.rotation[i]==body.transform.rotation[i]&&support_.scale[i]==body.transform.scale[i];
            prior_rotation[i]=body.transform.rotation[i];
        }
        queries.stable=stable;queries.stick=ground_stick;
        sample_=character3d::advance(body,state_,tuning,command_,dt,queries,measure_speed);
        if(sample_.status==CharacterMotionStatus::Applied){
            command_.jump=false;
            support_.valid=context.valid()&&revision&&sample_.grounded;
            if(support_.valid){
                for(unsigned i=0;i<3;++i){support_.position[i]=body.transform.position[i];support_.rotation[i]=prior_rotation[i];support_.scale[i]=body.transform.scale[i];}
                support_.tuning=tuning;support_.registry=active_object_registry;support_.context=context;
                support_.body=&body;support_.generation=body.generation;support_.owner=owner_id();support_.parent=body.parent;
                support_.revision=revision;support_.epoch=epoch;
            }
        }
        return sample_;
    }
public:
    static constexpr TickGroup tick_group=TickGroup::Physics;
    static constexpr uint64_t static_class_id=detail::compact_class_id("34a2d5d2-b0bb-5faf-8dc4-7f34f07f9c95");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,Id="88198dfe-90ea-527a-9cb3-f2dccd13a0df") Fixed gravity=24.0;
    EPOK_PROPERTY(EditAnywhere,Id="8292bcae-0dbe-5eaf-96a1-4a61c5eedbf8") Fixed jump_speed=9.0;
    EPOK_PROPERTY(EditAnywhere,Id="6ed73ac2-8083-5ec3-9460-233538dc1730") Fixed ground_stick=-0.05;
    EPOK_PROPERTY(EditAnywhere,Id="f91e23f9-00a1-5f32-b908-cbd2675b1e0b") Fixed step_height=0.6;
    EPOK_PROPERTY(EditAnywhere,Id="892bd984-3d6a-5c5c-b1ef-a50cb91784f9") Fixed ground_response=12.0;
    EPOK_PROPERTY(EditAnywhere,Id="c2690c7b-9849-5757-bce0-e66228ed432a") Fixed air_response=3.0;
    EPOK_PROPERTY(EditAnywhere,Id="3b8f674f-aedf-56ce-b119-3209c65cde3b") Fixed turn_rate=540.0;
    EPOK_PROPERTY(EditAnywhere,Id="9b750a45-0706-53fc-bd48-1cfd9462ab21") uint32_t collision_mask=0xffffffffu;
    EPOK_PROPERTY(EditAnywhere,Id="352309c6-51ea-52a7-bae5-410b8b2c9903") bool automatic_tick=true;
    EPOK_PROPERTY(EditAnywhere,Id="2386ce16-414c-52c1-907b-a98c08482406") bool measure_speed=true;
    EPOK_FUNCTION(BlueprintCallable,Id="8bd58843-38df-570d-98c2-a03f6bdd6f3b") void submit(CharacterCommand3D command){command.jump=command.jump||command_.jump;command_=command;command_changed();}
    EPOK_FUNCTION(BlueprintCallable,Id="c4705ac0-2d5c-5d21-a0c7-2d2152967863") void stop(){command_={};command_changed();}
    EPOK_FUNCTION(BlueprintPure,Id="a65a9197-8e23-5c1f-a468-d08c47ebda31") CharacterMotionSample3D motion()const{return sample_;}
    // Assign project-controlled facing without applying a second turn policy.
    // The next successful step applies it to the body, as command turning does.
    EPOK_FUNCTION(BlueprintCallable,Id="680bf821-bffc-55e2-a1a8-70aff698acdb") bool set_facing(Fixed degrees){
        if(!character3d::bounded(degrees,360000))return false;state_.facing=degrees;return true;
    }
    EPOK_FUNCTION(BlueprintCallable,Id="0b4c19f0-e4ce-5e4e-b788-2b66346de485") void clear_velocity(){
        for(auto& value:state_.velocity)value=0.0;command_={};command_changed();sample_={};
        sample_.facing=state_.facing;sample_.grounded=state_.grounded;
    }
    EPOK_FUNCTION(BlueprintCallable,Id="cc26eb82-78fb-55d3-a2cf-7bc23bc2a739") void reset_motion(){
        support_.valid=false;state_={};command_={};command_changed();sample_={};if(auto* actor=get_owner())if(auto* data=actor->data())state_.facing=data->transform.rotation[1];sample_.facing=state_.facing;
    }
    EPOK_FUNCTION(BlueprintCallable,Id="e9534340-c95c-5f68-ac5e-00299c16b7de") CharacterMotionSample3D simulate(Fixed delta_seconds){
        auto* data=available_body();if(!data)return fail(CharacterMotionStatus::InvalidOwner);
        Queries queries;queries.sweep=+[](SpatialQueryBatch*,ActorData& body,const Fixed* delta,uint32_t mask){return move_and_slide(body,delta,mask);};
        return simulate_body(*data,delta_seconds,queries,{},0,0);
    }
    EPOK_FUNCTION(BlueprintCallable,Id="b118871d-7889-58e5-906d-e521b8d4dbb9") CharacterMotionSample3D simulate_shared(Fixed delta_seconds,ObjectId query_context,uint32_t support_revision){
        auto* data=available_body();if(!data)return fail(CharacterMotionStatus::InvalidOwner);
        auto* owner=get_owner();
        auto* context=active_object_registry?active_object_registry->resolve<SpatialQueryContextComponent>(query_context):nullptr;
        auto* context_owner=context?context->get_owner():nullptr;
        if(!context_owner||context_owner->level_id()!=owner->level_id()||!context->live_batch())return fail(CharacterMotionStatus::InvalidContext);
        Queries queries;queries.batch=context->live_batch();queries.sweep=+[](SpatialQueryBatch* batch,ActorData& body,const Fixed* delta,uint32_t mask){return batch->move_and_slide(body,delta,mask);};
        return simulate_body(*data,delta_seconds,queries,context->id(),context->reset_epoch_,support_revision);
    }
    void begin_play()override{reset_motion();}
    void tick(Fixed dt)override{if(automatic_tick)simulate(dt);}
    void on_disable()override{support_.valid=false;stop();}
    void end_play(EndPlayReason)override{reset_motion();}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="a9264178-1960-5ec3-8407-2ef64bbdd9dd") Character3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("a9264178-1960-5ec3-8407-2ef64bbdd9dd");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Collider") Collider3DComponent collider;
    EPOK_COMPONENT(Name="Motor") CharacterMotor3DComponent motor;
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Mesh3DComponent") ObjectId visual{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") ObjectId camera{};
    EPOK_FUNCTION(BlueprintCallable) void submit_command(CharacterCommand3D command){motor.submit(command);}
    EPOK_FUNCTION(BlueprintCallable) void stop_movement(){motor.stop();}
    EPOK_FUNCTION(BlueprintPure) CharacterMotionSample3D motion()const{return motor.motion();}
    EPOK_FUNCTION(BlueprintPure) ObjectId motor_id()const{return motor.id();}
};
}
