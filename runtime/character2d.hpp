#pragma once
// Pixel-unit command motor over the existing World2D character body and solver.
#define EPOK_INCLUDE_FROM_CHARACTER2D 1
#include "gameplay_api.hpp"
#include "character3d.hpp"
#include "world2d_components.hpp"
#undef EPOK_INCLUDE_FROM_CHARACTER2D

namespace epok {
struct EPOK_VALUE(Id="1585f564-18c2-51c7-8834-bbc3dc60d96c") CharacterCommand2D {
    GameplayVector2 velocity{}; // Pixels/second; non-floating bodies project onto their floor tangent.
    bool jump=false;
};
struct EPOK_VALUE(Id="a971f14a-bbaa-57ab-99af-61836524f283") CharacterMotionSample2D {
    CharacterMotionStatus status=CharacterMotionStatus::NoStep;
    GameplayVector2 velocity{},displacement{};
    bool grounded=false,jumped=false,on_wall=false,on_ceiling=false,blocked=false;
};
namespace character2d {
inline bool bounded(Fixed value,int32_t limit=32768){return int64_t(value.raw())>=-int64_t(limit)*4096&&int64_t(value.raw())<=int64_t(limit)*4096;}
inline bool normalized_up(Fixed x,Fixed y){
    if(!bounded(x,1)||!bounded(y,1))return false;
    const int64_t length=int64_t(x.raw())*x.raw()+int64_t(y.raw())*y.raw();
    return length>=int64_t(4088)*4088&&length<=int64_t(4104)*4104;
}
}
class EPOK_CLASS(Blueprintable,Domain=World2D,Owners=World2D,Id="705deefe-c507-5369-bcaf-c1b3d53f9aa9") CharacterMotor2DComponent:public ActorComponent {
    friend struct detail::CharacterCommandLease<CharacterMotor2DComponent>;
    friend class CharacterCommandSource2DComponent;
    uint32_t command_revision_=0;
    void command_changed(){if(!++command_revision_)++command_revision_;}
    CharacterCommand2D command_{};
    CharacterMotionSample2D sample_{};
    CharacterBody2DComponent* body_value(){
        auto* owner=get_owner();
        auto* value=body.valid()&&active_object_registry?active_object_registry->resolve<CharacterBody2DComponent>(body):world2d::find<CharacterBody2DComponent>(owner);
        return value&&owner&&value->owner_id()==owner->id()?value:nullptr;
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("705deefe-c507-5369-bcaf-c1b3d53f9aa9");
    static constexpr TickGroup tick_group=TickGroup::Physics;
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::CharacterBody2DComponent") ObjectId body{};
    EPOK_PROPERTY(EditAnywhere) Fixed gravity=720.0;
    EPOK_PROPERTY(EditAnywhere) Fixed jump_speed=264.0;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintCallable) void submit(CharacterCommand2D command){command.jump=command.jump||command_.jump;command_=command;command_changed();}
    EPOK_FUNCTION(BlueprintCallable) void stop(){command_={};command_changed();}
    EPOK_FUNCTION(BlueprintPure) CharacterMotionSample2D motion()const{return sample_;}
    EPOK_FUNCTION(BlueprintCallable) void reset_motion(){
        command_={};command_changed();sample_={};
        if(auto* value=body_value()){
            value->set_velocity(0.0,0.0);value->on_floor=value->on_wall=value->on_ceiling=false;
            value->last_motion[0]=value->last_motion[1]=value->blocked_velocity[0]=value->blocked_velocity[1]=0.0;
            value->last_collider={};value->wall_collider={};value->last_hit_tile=false;
        }
    }
    EPOK_FUNCTION(BlueprintCallable) CharacterMotionSample2D simulate(Fixed delta_seconds){
        auto* owner=get_owner();auto* value=body_value();
        auto* collider=value?world2d::solid_collider_of(*value):nullptr;
        CharacterMotionSample2D result;
        if(!owner||!owner->is_a(Actor2D::static_class_id)||!is_active(owner)||!value||!collider||!collider->is_enabled()||collider->is_trigger()||!world2d::hooks.move){
            result.status=CharacterMotionStatus::InvalidOwner;sample_=result;return sample_;
        }
        result.velocity={value->velocity[0],value->velocity[1]};result.grounded=value->on_floor;
        result.on_wall=value->on_wall;result.on_ceiling=value->on_ceiling;
        if(delta_seconds.raw()==0){sample_=result;return sample_;}
        if(delta_seconds<0.0||delta_seconds>1.0||gravity<0.0||jump_speed<0.0
            ||!character2d::bounded(gravity)||!character2d::bounded(jump_speed)
            ||!character2d::bounded(command_.velocity.x)||!character2d::bounded(command_.velocity.y)
            ||!character2d::bounded(value->velocity[0])||!character2d::bounded(value->velocity[1])){
            result.status=CharacterMotionStatus::InvalidConfiguration;sample_=result;return sample_;
        }
        GameplayVector2 velocity=command_.velocity;
        if(!value->floating){
            const auto x=value->up_direction[0],y=value->up_direction[1];
            if(!character2d::normalized_up(x,y)){result.status=CharacterMotionStatus::InvalidConfiguration;sample_=result;return sample_;}
            const Fixed requested=velocity.x*x+velocity.y*y;
            Fixed normal=value->velocity[0]*x+value->velocity[1]*y-gravity*delta_seconds;
            result.jumped=command_.jump&&value->on_floor;
            if(result.jumped)normal=jump_speed;
            velocity={velocity.x-x*requested+x*normal,velocity.y-y*requested+y*normal};
        }
        if(!character2d::bounded(velocity.x)||!character2d::bounded(velocity.y)){
            result.status=CharacterMotionStatus::InvalidConfiguration;result.jumped=false;sample_=result;return sample_;
        }
        value->set_velocity(velocity.x,velocity.y);value->move_and_slide_step(delta_seconds);
        result.status=CharacterMotionStatus::Applied;
        result.velocity={value->velocity[0],value->velocity[1]};result.displacement={value->last_motion[0],value->last_motion[1]};
        result.grounded=value->on_floor;result.on_wall=value->on_wall;result.on_ceiling=value->on_ceiling;
        result.blocked=result.on_wall||result.on_ceiling;sample_=result;command_.jump=false;return sample_;
    }
    void begin_play()override{reset_motion();}
    void tick(Fixed dt)override{if(automatic_tick)simulate(dt);}
    void on_disable()override{stop();}
    void end_play(EndPlayReason)override{reset_motion();}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="059ce53e-b93c-5f7c-9446-af075cf9898b") Character2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("059ce53e-b93c-5f7c-9446-af075cf9898b");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Collider") Collider2DComponent collider;
    EPOK_COMPONENT(Name="Body") CharacterBody2DComponent body;
    EPOK_COMPONENT(Name="Motor") CharacterMotor2DComponent motor;
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Sprite2DComponent") ObjectId visual{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Camera2DComponent") ObjectId camera{};
    EPOK_FUNCTION(BlueprintCallable) void submit_command(CharacterCommand2D command){motor.submit(command);}
    EPOK_FUNCTION(BlueprintCallable) void stop_movement(){motor.stop();}
    EPOK_FUNCTION(BlueprintPure) CharacterMotionSample2D motion()const{return motor.motion();}
    EPOK_FUNCTION(BlueprintPure) ObjectId motor_id()const{return motor.id();}
};
}
