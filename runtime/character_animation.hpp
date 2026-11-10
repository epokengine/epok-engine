#pragma once
#define EPOK_INCLUDE_FROM_CHARACTER_ANIMATION 1
#include "character3d.hpp"
#include "character2d.hpp"
#undef EPOK_INCLUDE_FROM_CHARACTER_ANIMATION

namespace epok {
enum class LocomotionState:uint8_t {Idle,Walk,Run,JumpUp,JumpDown,Land};
enum class LocomotionStatus:uint8_t {Applied,NoStep,InvalidOwner,InvalidMotor,InvalidVisual,InvalidConfiguration,MissingClip,Busy};
struct EPOK_VALUE(Id="1831665a-8aea-56cd-8b41-6deca23e181f") LocomotionClips {
    int32_t idle=-1,walk=-1,run=-1,jump_up=-1,jump_down=-1,land=-1;
};
struct EPOK_VALUE(Id="d99f567b-d5de-5d2f-9ef4-f5ad32cf2069") LocomotionTuning {
    Fixed idle_speed=0.12,run_enter=3.6,run_exit=3.0;
    Fixed walk_cycle_speed=2.6,run_cycle_speed=6.0,min_rate=0.2,max_rate=1.6;
};
struct EPOK_VALUE(Id="32cb8a8e-5e0e-5474-9999-38d7c3012930") LocomotionSelection {
    LocomotionState state=LocomotionState::Idle;
    int32_t clip=-1;
    bool looping=true;
    Fixed rate=1.0;
};
struct EPOK_VALUE(Id="0688fc12-6235-5af1-ae20-aa5b0fdaf142") LocomotionSample {
    LocomotionStatus status=LocomotionStatus::NoStep;
    LocomotionState state=LocomotionState::Idle;
    int32_t clip=-1;
    Fixed rate=1.0;
    bool changed=false;
};
namespace locomotion {
inline bool moving(LocomotionState state){return state==LocomotionState::Walk||state==LocomotionState::Run;}
inline bool valid(const LocomotionTuning& tuning){
    return tuning.idle_speed>=0.0&&tuning.run_enter>=tuning.run_exit&&tuning.run_exit>=tuning.idle_speed
        &&tuning.walk_cycle_speed>0.0&&tuning.run_cycle_speed>0.0&&tuning.min_rate>=0.0&&tuning.max_rate>=tuning.min_rate
        &&character2d::bounded(tuning.idle_speed)&&character2d::bounded(tuning.run_enter)&&character2d::bounded(tuning.run_exit)
        &&character2d::bounded(tuning.walk_cycle_speed)&&character2d::bounded(tuning.run_cycle_speed)&&tuning.max_rate<=16.0;
}
inline LocomotionState choose(LocomotionState state,bool grounded,Fixed vertical,Fixed speed,bool finished,bool jumped,const LocomotionTuning& tuning){
    if(jumped)return LocomotionState::JumpUp;
    if(!grounded)return vertical>0.0?LocomotionState::JumpUp:LocomotionState::JumpDown;
    if(state==LocomotionState::JumpUp||state==LocomotionState::JumpDown)return LocomotionState::Land;
    if(state==LocomotionState::Land&&!finished)return state;
    if(speed<tuning.idle_speed)return LocomotionState::Idle;
    if(speed>tuning.run_enter||(state==LocomotionState::Run&&speed>tuning.run_exit))return LocomotionState::Run;
    return LocomotionState::Walk;
}
inline LocomotionSelection selection(LocomotionState state,Fixed speed,const LocomotionClips& clips,const LocomotionTuning& tuning,bool distance_driven){
    const int32_t values[]={clips.idle,clips.walk,clips.run,clips.jump_up,clips.jump_down,clips.land};
    LocomotionSelection result;result.state=state;result.clip=values[unsigned(state)];
    result.looping=state==LocomotionState::Idle||moving(state);
    if(distance_driven&&moving(state)){
        const auto cycle=state==LocomotionState::Walk?tuning.walk_cycle_speed:tuning.run_cycle_speed;
        int64_t rate=cycle.raw()>0?int64_t(speed.raw())*4096/cycle.raw():0;
        if(rate<tuning.min_rate.raw())rate=tuning.min_rate.raw();if(rate>tuning.max_rate.raw())rate=tuning.max_rate.raw();
        result.rate=Fixed(int32_t(rate),Fixed::RAW);
    }
    return result;
}
inline bool valid(LocomotionSelection value){return unsigned(value.state)<=unsigned(LocomotionState::Land)&&value.clip>=-1&&value.rate>=0.0&&value.rate<=16.0;}
struct Runtime {
    LocomotionSample sample{};Fixed phase=0.0;ObjectId motor{},visual{};const void* bank=nullptr;bool updating=false,manual=false;
    void reset(){sample={};phase=0.0;motor={};visual={};bank=nullptr;manual=false;}
    LocomotionSample report(LocomotionStatus status){sample.status=status;sample.changed=false;return sample;}
    bool fresh(ObjectId new_motor,ObjectId new_visual,const void* new_bank)const{return motor!=new_motor||visual!=new_visual||bank!=new_bank;}
    LocomotionSample applied(LocomotionSelection value,bool changed,ObjectId new_motor,ObjectId new_visual,const void* new_bank){
        sample={LocomotionStatus::Applied,value.state,value.clip,value.rate,changed};motor=new_motor;visual=new_visual;bank=new_bank;return sample;
    }
};
struct UpdateScope {bool& value;explicit UpdateScope(bool& flag):value(flag){value=true;}~UpdateScope(){value=false;}};
template<class T> T* component(ActorComponent& source,ObjectId reference,Actor& fallback){
    auto* registry=active_object_registry;auto* owner=source.get_owner();if(!registry||!owner)return nullptr;
    auto* value=reference.valid()?registry->resolve<T>(reference):world2d::find<T>(&fallback);
    auto* target_owner=value?value->get_owner():nullptr;
    return target_owner&&is_active(target_owner)&&target_owner->level_id()==owner->level_id()?value:nullptr;
}
inline bool alive(ActorComponent& source,ObjectRegistry* registry,ObjectId identity){
    auto* owner=source.get_owner();return active_object_registry==registry&&registry->get(identity)==&source&&owner&&is_active(owner);
}
}
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="af114d97-d97c-5aca-8c0e-c0bf179273d8") SkeletalLocomotionComponent:public ActorComponent {
    locomotion::Runtime runtime_;
    CharacterMotor3DComponent* motor_value(){auto* owner=get_owner();return owner?locomotion::component<CharacterMotor3DComponent>(*this,motor,*owner):nullptr;}
    Mesh3DComponent* visual_value(CharacterMotor3DComponent& value){
        auto* owner=value.get_owner();if(!owner)return nullptr;ObjectId reference=visual;
        if(!reference.valid())if(auto* character=active_object_registry->resolve<Character3D>(owner->id()))reference=character->visual;
        return locomotion::component<Mesh3DComponent>(*this,reference,*owner);
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("af114d97-d97c-5aca-8c0e-c0bf179273d8");
    static constexpr TickGroup tick_group=TickGroup::PostPhysics;
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::CharacterMotor3DComponent") ObjectId motor{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Mesh3DComponent") ObjectId visual{};
    EPOK_PROPERTY(EditAnywhere) LocomotionClips clips{-1,-1,-1,-1,-1,-1};
    EPOK_PROPERTY(EditAnywhere) LocomotionTuning tuning{0.12,3.6,3.0,2.6,6.0,0.2,1.6};
    EPOK_PROPERTY(EditAnywhere) bool distance_driven=true;
    EPOK_PROPERTY(EditAnywhere) bool enabled=true;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintEvent) virtual LocomotionSelection select_animation(CharacterMotionSample3D motion,LocomotionState previous,bool finished){
        return locomotion::selection(locomotion::choose(previous,motion.grounded,motion.velocity.y,motion.speed,finished,motion.jumped,tuning),motion.speed,clips,tuning,distance_driven);
    }
    EPOK_FUNCTION(BlueprintEvent) virtual void animation_applied(LocomotionSample value){(void)value;}
    EPOK_FUNCTION(BlueprintPure) LocomotionSample sample()const{return runtime_.sample;}
    EPOK_FUNCTION(BlueprintCallable) void reset(){runtime_.reset();}
    EPOK_FUNCTION(BlueprintCallable) void set_enabled(bool value){enabled=value;if(!value)reset();}
    EPOK_FUNCTION(BlueprintCallable) LocomotionSample update(Fixed dt){
        if(runtime_.updating)return {LocomotionStatus::Busy};
        auto* registry=active_object_registry;if(!registry)return runtime_.report(LocomotionStatus::InvalidOwner);
        ObjectDispatchScope scope(*registry);locomotion::UpdateScope updating(runtime_.updating);const auto self=id();
        if(!enabled||dt.raw()==0)return runtime_.report(LocomotionStatus::NoStep);
        if(!locomotion::alive(*this,registry,self))return runtime_.report(LocomotionStatus::InvalidOwner);
        if(dt<0.0||dt>1.0||!locomotion::valid(tuning))return runtime_.report(LocomotionStatus::InvalidConfiguration);
        auto* body=motor_value();if(!body)return runtime_.report(LocomotionStatus::InvalidMotor);
        const auto motion=body->motion();if(motion.status!=CharacterMotionStatus::Applied)return runtime_.report(LocomotionStatus::NoStep);
        auto* mesh=visual_value(*body);auto* data=mesh?mesh->entity_slot():nullptr;
        if(!data||!data->animator.enabled||!data->animator.model||!data->animator.model->clips)return runtime_.report(LocomotionStatus::InvalidVisual);
        auto& animator=data->animator;const auto* model=animator.model;const auto body_id=body->id(),mesh_id=mesh->id();
        const bool fresh=runtime_.fresh(body_id,mesh_id,model);
        const uint32_t frames=animator.clip>=0&&size_t(animator.clip)<model->clip_count?model->clips[animator.clip].frames:0;
        const bool finished=frames==0||animator.ticks>=(frames-1)*2;
        const auto selected=select_animation(motion,fresh?LocomotionState::Idle:runtime_.sample.state,finished);
        if(!locomotion::alive(*this,registry,self))return runtime_.report(LocomotionStatus::InvalidOwner);
        if(!enabled)return runtime_.report(LocomotionStatus::NoStep);
        if(registry->resolve<CharacterMotor3DComponent>(body_id)!=body||motor_value()!=body)return runtime_.report(LocomotionStatus::InvalidMotor);
        if(registry->resolve<Mesh3DComponent>(mesh_id)!=mesh||visual_value(*body)!=mesh||mesh->entity_slot()!=data||animator.model!=model||!animator.enabled)return runtime_.report(LocomotionStatus::InvalidVisual);
        if(!locomotion::valid(selected))return runtime_.report(LocomotionStatus::InvalidConfiguration);
        if(selected.clip<0||size_t(selected.clip)>=model->clip_count||!model->clips[selected.clip].frames)return runtime_.report(LocomotionStatus::MissingClip);
        const bool changed=fresh||runtime_.sample.state!=selected.state||animator.clip!=selected.clip||animator.looping!=selected.looping;
        if(changed){animator.play(selected.clip,selected.looping);runtime_.phase=0.0;}
        const bool manual=(distance_driven&&locomotion::moving(selected.state))||selected.rate!=1.0;
        if(manual){
            if(!changed&&!runtime_.manual)runtime_.phase=Fixed(int32_t(animator.ticks),0);
            animator.pause();const auto length=model->clips[selected.clip].frames;
            int64_t phase=int64_t(runtime_.phase.raw())+int64_t((selected.rate*dt).raw())*60;
            if(selected.looping)phase%=int64_t(length>1?length-1:1)*2*4096;
            else {const int64_t end=int64_t(length-1)*2*4096;if(phase>end)phase=end;}
            runtime_.phase=Fixed(int32_t(phase),Fixed::RAW);
            animator.ticks=uint32_t(runtime_.phase.raw()/4096);
        }else if(selected.looping||animator.ticks<uint32_t(model->clips[selected.clip].frames-1)*2)animator.resume();
        runtime_.manual=manual;
        const auto result=runtime_.applied(selected,changed,body_id,mesh_id,model);animation_applied(result);return result;
    }
    void tick(Fixed dt)override{if(automatic_tick)update(dt);}
    void begin_play()override{reset();}
    void on_disable()override{reset();}
    void end_play(EndPlayReason)override{reset();}
};
class EPOK_CLASS(Blueprintable,Domain=World2D,Owners=World2D,Id="a55c1719-65fb-52f1-addf-7b397da706fd") SpriteLocomotion2DComponent:public ActorComponent {
    locomotion::Runtime runtime_;
    CharacterMotor2DComponent* motor_value(){auto* owner=get_owner();return owner?locomotion::component<CharacterMotor2DComponent>(*this,motor,*owner):nullptr;}
    Sprite2DComponent* visual_value(CharacterMotor2DComponent& value){
        auto* owner=value.get_owner();if(!owner)return nullptr;ObjectId reference=visual;
        if(!reference.valid())if(auto* character=active_object_registry->resolve<Character2D>(owner->id()))reference=character->visual;
        return locomotion::component<Sprite2DComponent>(*this,reference,*owner);
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("a55c1719-65fb-52f1-addf-7b397da706fd");
    static constexpr TickGroup tick_group=TickGroup::PostPhysics;
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::CharacterMotor2DComponent") ObjectId motor{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Sprite2DComponent") ObjectId visual{};
    EPOK_PROPERTY(EditAnywhere) LocomotionClips clips{0,1,1,-1,-1,-1};
    EPOK_PROPERTY(EditAnywhere) LocomotionTuning tuning{0.12,128.0,112.0,96.0,192.0,0.2,1.6};
    EPOK_PROPERTY(EditAnywhere) bool distance_driven=false;
    EPOK_PROPERTY(EditAnywhere) bool animate_air=false;
    EPOK_PROPERTY(EditAnywhere) bool face_motion=true;
    EPOK_PROPERTY(EditAnywhere) bool enabled=true;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintEvent) virtual LocomotionSelection select_animation(CharacterMotionSample2D motion,LocomotionState previous,bool finished){
        auto* motor=motor_value();auto* owner=motor?motor->get_owner():nullptr;
        auto* body=owner?locomotion::component<CharacterBody2DComponent>(*this,motor->body,*owner):nullptr;
        const auto x=body?body->up_direction[0]:Fixed(0.0),y=body?body->up_direction[1]:Fixed(-1.0);
        const Fixed tangent=-y*motion.velocity.x+x*motion.velocity.y;
        const Fixed speed=tangent<0.0?-tangent:tangent,vertical=x*motion.velocity.x+y*motion.velocity.y;
        const auto state=locomotion::choose(previous,!animate_air||motion.grounded,vertical,speed,finished,animate_air&&motion.jumped,tuning);
        return locomotion::selection(state,speed,clips,tuning,distance_driven);
    }
    EPOK_FUNCTION(BlueprintEvent) virtual void animation_applied(LocomotionSample value){(void)value;}
    EPOK_FUNCTION(BlueprintPure) LocomotionSample sample()const{return runtime_.sample;}
    EPOK_FUNCTION(BlueprintCallable) void reset(){runtime_.reset();}
    EPOK_FUNCTION(BlueprintCallable) void set_enabled(bool value){enabled=value;if(!value)reset();}
    EPOK_FUNCTION(BlueprintCallable) LocomotionSample update(Fixed dt){
        if(runtime_.updating)return {LocomotionStatus::Busy};
        auto* registry=active_object_registry;if(!registry)return runtime_.report(LocomotionStatus::InvalidOwner);
        ObjectDispatchScope scope(*registry);locomotion::UpdateScope updating(runtime_.updating);const auto self=id();
        if(!enabled||dt.raw()==0)return runtime_.report(LocomotionStatus::NoStep);
        if(!locomotion::alive(*this,registry,self))return runtime_.report(LocomotionStatus::InvalidOwner);
        if(dt<0.0||dt>1.0||!locomotion::valid(tuning))return runtime_.report(LocomotionStatus::InvalidConfiguration);
        auto* motor=motor_value();if(!motor)return runtime_.report(LocomotionStatus::InvalidMotor);
        const auto motion=motor->motion();if(motion.status!=CharacterMotionStatus::Applied)return runtime_.report(LocomotionStatus::NoStep);
        auto* sprite=visual_value(*motor);if(!sprite||!sprite->clips)return runtime_.report(LocomotionStatus::InvalidVisual);
        const auto motor_id=motor->id(),sprite_id=sprite->id();const auto* bank=sprite->clips;
        const bool fresh=runtime_.fresh(motor_id,sprite_id,bank);
        const auto selected=select_animation(motion,fresh?LocomotionState::Idle:runtime_.sample.state,sprite->finished);
        if(!locomotion::alive(*this,registry,self))return runtime_.report(LocomotionStatus::InvalidOwner);
        if(!enabled)return runtime_.report(LocomotionStatus::NoStep);
        if(registry->resolve<CharacterMotor2DComponent>(motor_id)!=motor||motor_value()!=motor)return runtime_.report(LocomotionStatus::InvalidMotor);
        if(registry->resolve<Sprite2DComponent>(sprite_id)!=sprite||visual_value(*motor)!=sprite||sprite->clips!=bank)return runtime_.report(LocomotionStatus::InvalidVisual);
        if(!locomotion::valid(selected))return runtime_.report(LocomotionStatus::InvalidConfiguration);
        if(selected.clip<0||uint32_t(selected.clip)>=sprite->clip_count||!bank[selected.clip].frames||!bank[selected.clip].frame_count)return runtime_.report(LocomotionStatus::MissingClip);
        if(selected.looping!=bank[selected.clip].looping)return runtime_.report(LocomotionStatus::InvalidConfiguration);
        const bool changed=fresh||runtime_.sample.state!=selected.state||sprite->animation!=selected.clip;
        if(changed){sprite->play(uint32_t(selected.clip));sprite->frame=0;sprite->elapsed=0.0;}
        sprite->set_speed_scale(selected.rate);
        if(!sprite->finished)sprite->playing=true;
        if(face_motion){
            auto* owner=motor->get_owner();auto* body=owner?locomotion::component<CharacterBody2DComponent>(*this,motor->body,*owner):nullptr;
            if(body){const auto tangent=-body->up_direction[1]*motion.velocity.x+body->up_direction[0]*motion.velocity.y;if(tangent.raw())sprite->set_flip_h(tangent<0.0);}
        }
        const auto result=runtime_.applied(selected,changed,motor_id,sprite_id,bank);animation_applied(result);return result;
    }
    void tick(Fixed dt)override{if(automatic_tick)update(dt);}
    void begin_play()override{reset();}
    void on_disable()override{reset();}
    void end_play(EndPlayReason)override{reset();}
};
}
