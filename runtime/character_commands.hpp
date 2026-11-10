#pragma once
// Command policies share motor APIs; they do not implement another movement solver.
#define EPOK_INCLUDE_FROM_CHARACTER_COMMANDS 1
#include "gameplay_api.hpp"
#include "character3d.hpp"
#include "character2d.hpp"
#undef EPOK_INCLUDE_FROM_CHARACTER_COMMANDS

namespace epok {
enum class CharacterCommandStatus:uint8_t {Applied,NoStep,InvalidOwner,InvalidMotor,InvalidTarget,InvalidConfiguration,Busy};
struct EPOK_VALUE(Id="30f44db2-32a8-5dff-8fe1-cd9cfbc9a33e") CharacterCommandResult3D {
    CharacterCommandStatus status=CharacterCommandStatus::Applied;
    CharacterCommand3D command{};
};
struct EPOK_VALUE(Id="e421c2ee-3e19-55d3-b9b4-7f9c3b15ef8b") CharacterCommandResult2D {
    CharacterCommandStatus status=CharacterCommandStatus::Applied;
    CharacterCommand2D command{};
};
namespace detail {
template<class Motor> struct CharacterCommandLease {
    ObjectId motor{};ObjectRegistry* registry=nullptr;uint32_t revision=0;bool updating=false;
    bool release(){
        auto* value=registry?registry->resolve<Motor>(motor):nullptr;
        const bool owned=value&&value->command_revision_==revision;
        if(owned)value->stop();motor={};registry=nullptr;revision=0;return owned;
    }
    static Motor* resolve(ActorComponent& source,ObjectId binding,ObjectRegistry* registry){
        auto* owner=registry?registry->resolve<Actor>(source.owner_id()):nullptr;
        if(!owner||!is_active(owner))return nullptr;
        auto* target=binding.valid()?registry->resolve<Motor>(binding):nullptr;
        if(!binding.valid())if(auto* level=registry->resolve<Level>(owner->level_id()))target=level->get_component<Motor>(*owner);
        auto* target_owner=target?registry->resolve<Actor>(target->owner_id()):nullptr;
        return target_owner&&target_owner->level_id()==owner->level_id()&&is_active(target_owner)?target:nullptr;
    }
    template<class Command> void submit(Motor& value,Command command,ObjectRegistry* current){
        if(registry!=current||motor!=value.id()||revision!=value.command_revision_){release();value.stop();}
        value.submit(command);motor=value.id();registry=current;revision=value.command_revision_;
    }
};
struct CharacterCommandUpdateScope {
    bool& updating;explicit CharacterCommandUpdateScope(bool& flag):updating(flag){updating=true;}
    ~CharacterCommandUpdateScope(){updating=false;}
};
inline bool valid_character_command(CharacterCommand3D& command){
    command.heading=MathLibrary::wrap_degrees(command.heading);
    return character3d::bounded(command.velocity.x,4096)&&character3d::bounded(command.velocity.y,4096);
}
inline bool valid_character_command(CharacterCommand2D& command){return character2d::bounded(command.velocity.x)&&character2d::bounded(command.velocity.y);}
template<class Source,class Motor,class Result> Result prepare_character_command(Source& source,CharacterCommandLease<Motor>& lease,Fixed dt,ObjectRegistry* registry){
    Result result;result.status=CharacterCommandStatus::NoStep;
    if(!source.enabled){lease.release();return result;}
    if(dt.raw()==0)return result;
    if(dt<0.0||dt>1.0){lease.release();result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
    auto* owner=registry?registry->resolve<Actor>(source.owner_id()):nullptr;
    if(!owner||!is_active(owner)||registry->resolve<Source>(source.id())!=&source){lease.release();result.status=CharacterCommandStatus::InvalidOwner;return result;}
    auto* motor=lease.resolve(source,source.motor,registry);
    if(!motor){lease.release();result.status=CharacterCommandStatus::InvalidMotor;return result;}
    const auto self=source.id(),target=motor->id();
    result=source.calculate_command(dt);
    if(active_object_registry!=registry||registry->resolve<Source>(self)!=&source||!is_active(owner)){
        lease.release();result.status=CharacterCommandStatus::InvalidOwner;return result;
    }
    if(!source.enabled){lease.release();result.status=CharacterCommandStatus::NoStep;return result;}
    if(registry->resolve<Motor>(target)!=motor||lease.resolve(source,source.motor,registry)!=motor){
        lease.release();result.status=CharacterCommandStatus::InvalidMotor;return result;
    }
    if(result.status!=CharacterCommandStatus::Applied){lease.release();return result;}
    if(!valid_character_command(result.command)){lease.release();result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
    lease.submit(*motor,result.command,registry);return result;
}
inline bool character_position_2d(Actor& actor,GameplayVector2& result){
    auto* root=active_object_registry?active_object_registry->resolve<SceneComponent2D>(actor.root_id()):nullptr;
    Affine2D world;if(!root||!world2d::world_of(*root,world))return false;
    result={world.t[0],world.t[1]};return true;
}
inline bool character_planar_velocity(Fixed from_x,Fixed from_y,Fixed to_x,Fixed to_y,Fixed speed,Fixed radius,Fixed dt,GameplayVector2& velocity){
    const int64_t dx=int64_t(to_x.raw())-from_x.raw(),dy=int64_t(to_y.raw())-from_y.raw();
    if(dx<-INT32_MAX/2||dx>INT32_MAX/2||dy<-INT32_MAX/2||dy>INT32_MAX/2||dt.raw()<=0)return false;
    const int64_t distance=fixed_math::sqrt64(uint64_t(dx*dx+dy*dy));velocity={};
    if(distance<=radius.raw())return true;
    int64_t amount=(distance-radius.raw())*4096/dt.raw();if(amount>speed.raw())amount=speed.raw();
    velocity={Fixed(int32_t(dx*amount/distance),Fixed::RAW),Fixed(int32_t(dy*amount/distance),Fixed::RAW)};return true;
}
}
class EPOK_CLASS(Abstract,Blueprintable,Domain=World3D,Owners=World3D,Id="319e0e29-420a-5a7c-a115-d6eb2e6459ff") CharacterCommandSource3DComponent:public ActorComponent {
    detail::CharacterCommandLease<CharacterMotor3DComponent> lease_;
    CharacterCommandResult3D sample_{CharacterCommandStatus::NoStep,{}};
protected:
    CharacterMotor3DComponent* target_motor(){return lease_.resolve(*this,motor,active_object_registry);}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("319e0e29-420a-5a7c-a115-d6eb2e6459ff");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::CharacterMotor3DComponent") ObjectId motor{};
    EPOK_PROPERTY(EditAnywhere) bool enabled=true;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintEvent) virtual CharacterCommandResult3D calculate_command(Fixed delta_seconds){(void)delta_seconds;return {};}
    EPOK_FUNCTION(BlueprintEvent) virtual void command_submitted(CharacterCommandResult3D result){(void)result;}
    EPOK_FUNCTION(BlueprintCallable) bool release_command(){return lease_.release();}
    EPOK_FUNCTION(BlueprintCallable) void set_enabled(bool value){enabled=value;if(!value)lease_.release();}
    EPOK_FUNCTION(BlueprintPure) CharacterCommandResult3D sample()const{return sample_;}
    EPOK_FUNCTION(BlueprintCallable) CharacterCommandResult3D update(Fixed delta_seconds){
        if(lease_.updating)return {CharacterCommandStatus::Busy,{}};
        auto* registry=active_object_registry;if(!registry){sample_={CharacterCommandStatus::InvalidOwner,{}};return sample_;}
        ObjectDispatchScope scope(*registry);detail::CharacterCommandUpdateScope updating(lease_.updating);
        const auto result=detail::prepare_character_command<CharacterCommandSource3DComponent,CharacterMotor3DComponent,CharacterCommandResult3D>(*this,lease_,delta_seconds,registry);
        sample_=result;if(result.status==CharacterCommandStatus::Applied)command_submitted(result);return result;
    }
    void begin_play()override{lease_.release();sample_={CharacterCommandStatus::NoStep,{}};}
    void tick(Fixed dt)override{if(automatic_tick)update(dt);}
    void on_disable()override{lease_.release();}
    void end_play(EndPlayReason)override{lease_.release();sample_={CharacterCommandStatus::NoStep,{}};}
};
class EPOK_CLASS(Abstract,Blueprintable,Domain=World2D,Owners=World2D,Id="f0fcc71d-d59a-5292-b3b7-e0d1f2881d6d") CharacterCommandSource2DComponent:public ActorComponent {
    detail::CharacterCommandLease<CharacterMotor2DComponent> lease_;
    CharacterCommandResult2D sample_{CharacterCommandStatus::NoStep,{}};
protected:
    CharacterMotor2DComponent* target_motor(){return lease_.resolve(*this,motor,active_object_registry);}
    CharacterBody2DComponent* target_body(){auto* value=target_motor();return value?value->body_value():nullptr;}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("f0fcc71d-d59a-5292-b3b7-e0d1f2881d6d");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::CharacterMotor2DComponent") ObjectId motor{};
    EPOK_PROPERTY(EditAnywhere) bool enabled=true;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintEvent) virtual CharacterCommandResult2D calculate_command(Fixed delta_seconds){(void)delta_seconds;return {};}
    EPOK_FUNCTION(BlueprintEvent) virtual void command_submitted(CharacterCommandResult2D result){(void)result;}
    EPOK_FUNCTION(BlueprintCallable) bool release_command(){return lease_.release();}
    EPOK_FUNCTION(BlueprintCallable) void set_enabled(bool value){enabled=value;if(!value)lease_.release();}
    EPOK_FUNCTION(BlueprintPure) CharacterCommandResult2D sample()const{return sample_;}
    EPOK_FUNCTION(BlueprintCallable) CharacterCommandResult2D update(Fixed delta_seconds){
        if(lease_.updating)return {CharacterCommandStatus::Busy,{}};
        auto* registry=active_object_registry;if(!registry){sample_={CharacterCommandStatus::InvalidOwner,{}};return sample_;}
        ObjectDispatchScope scope(*registry);detail::CharacterCommandUpdateScope updating(lease_.updating);
        const auto result=detail::prepare_character_command<CharacterCommandSource2DComponent,CharacterMotor2DComponent,CharacterCommandResult2D>(*this,lease_,delta_seconds,registry);
        sample_=result;if(result.status==CharacterCommandStatus::Applied)command_submitted(result);return result;
    }
    void begin_play()override{lease_.release();sample_={CharacterCommandStatus::NoStep,{}};}
    void tick(Fixed dt)override{if(automatic_tick)update(dt);}
    void on_disable()override{lease_.release();}
    void end_play(EndPlayReason)override{lease_.release();sample_={CharacterCommandStatus::NoStep,{}};}
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="76c91654-0c63-5474-b44f-08019e831fa9") ScriptedCommand3DComponent:public CharacterCommandSource3DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("76c91654-0c63-5474-b44f-08019e831fa9");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) CharacterCommand3D desired{{0.0,0.0},0.0,false,false};
    CharacterCommandResult3D calculate_command(Fixed)override{return {CharacterCommandStatus::Applied,desired};}
    void command_submitted(CharacterCommandResult3D)override{desired.jump=false;}
};
class EPOK_CLASS(Blueprintable,Domain=World2D,Owners=World2D,Id="1eb12c94-7e09-55fd-801f-1955f5e5cc2a") ScriptedCommand2DComponent:public CharacterCommandSource2DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("1eb12c94-7e09-55fd-801f-1955f5e5cc2a");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) CharacterCommand2D desired{{0.0,0.0},false};
    CharacterCommandResult2D calculate_command(Fixed)override{return {CharacterCommandStatus::Applied,desired};}
    void command_submitted(CharacterCommandResult2D)override{desired.jump=false;}
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="3393c8d2-55a0-5e23-999c-5160f5a1014c") MoveTo3DComponent:public CharacterCommandSource3DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("3393c8d2-55a0-5e23-999c-5160f5a1014c");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere) GameplayVector3 goal{0.0,0.0,0.0};
    EPOK_PROPERTY(EditAnywhere) bool use_point=false;
    EPOK_PROPERTY(EditAnywhere) Fixed speed=3.0;
    EPOK_PROPERTY(EditAnywhere) Fixed stop_distance=0.2;
    EPOK_PROPERTY(EditAnywhere) bool turn_to_movement=true;
    CharacterCommandResult3D calculate_command(Fixed dt)override{
        CharacterCommandResult3D result;
        if(speed<0.0||stop_distance<0.0||!character3d::bounded(speed,4096)||!character3d::bounded(stop_distance,4096)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
        if(!target.valid()&&!use_point)return result;
        auto* value=target_motor();auto* owner=value?value->get_owner():nullptr;
        const auto from=gameplay_world_affine(owner?owner->data():nullptr);if(!from.success){result.status=CharacterCommandStatus::InvalidMotor;return result;}
        GameplayVector3 destination=goal;
        if(target.valid()){
            auto* actor=active_object_registry->resolve<Actor3D>(target);
            if(!actor||actor->level_id()!=owner->level_id()||!is_active(actor)){result.status=CharacterCommandStatus::InvalidTarget;return result;}
            const auto world=gameplay_world_affine(actor->data());if(!world.success){result.status=CharacterCommandStatus::InvalidTarget;return result;}
            destination={world.position[0],world.position[1],world.position[2]};
        }
        if(!detail::character_planar_velocity(from.position[0],from.position[2],destination.x,destination.z,speed,stop_distance,dt,result.command.velocity)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
        result.command.turn=turn_to_movement&&(result.command.velocity.x.raw()||result.command.velocity.y.raw());
        if(result.command.turn)result.command.heading=MathLibrary::heading_degrees(result.command.velocity.x,result.command.velocity.y);return result;
    }
};
class EPOK_CLASS(Blueprintable,Domain=World2D,Owners=World2D,Id="8c499172-ee35-5727-bc12-7f77b8a65165") MoveTo2DComponent:public CharacterCommandSource2DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("8c499172-ee35-5727-bc12-7f77b8a65165");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor2D") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere) GameplayVector2 goal{0.0,0.0};
    EPOK_PROPERTY(EditAnywhere) bool use_point=false;
    EPOK_PROPERTY(EditAnywhere) Fixed speed=64.0;
    EPOK_PROPERTY(EditAnywhere) Fixed stop_distance=2.0;
    CharacterCommandResult2D calculate_command(Fixed dt)override{
        CharacterCommandResult2D result;
        if(speed<0.0||stop_distance<0.0||!character2d::bounded(speed)||!character2d::bounded(stop_distance)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
        if(!target.valid()&&!use_point)return result;
        auto* value=target_motor();auto* owner=value?value->get_owner():nullptr;auto* body=target_body();GameplayVector2 from;
        if(!owner||!body||!detail::character_position_2d(*owner,from)){result.status=CharacterCommandStatus::InvalidMotor;return result;}
        auto destination=goal;
        if(target.valid()){
            auto* actor=active_object_registry->resolve<Actor2D>(target);
            if(!actor||actor->level_id()!=owner->level_id()||!is_active(actor)||!detail::character_position_2d(*actor,destination)){result.status=CharacterCommandStatus::InvalidTarget;return result;}
        }
        if(body->floating){
            if(!detail::character_planar_velocity(from.x,from.y,destination.x,destination.y,speed,stop_distance,dt,result.command.velocity))result.status=CharacterCommandStatus::InvalidConfiguration;
        }else{
            const auto x=body->up_direction[0],y=body->up_direction[1];
            if(!character2d::normalized_up(x,y)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
            const int64_t dx=int64_t(destination.x.raw())-from.x.raw(),dy=int64_t(destination.y.raw())-from.y.raw();
            const int64_t distance=(dx*(-int64_t(y.raw()))+dy*x.raw())/4096;
            if(distance<INT32_MIN||distance>INT32_MAX){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
            GameplayVector2 along;
            if(!detail::character_planar_velocity(0.0,0.0,Fixed(int32_t(distance),Fixed::RAW),0.0,speed,stop_distance,dt,along)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
            result.command.velocity={-y*along.x,x*along.x};
        }return result;
    }
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="308ddb5e-632e-5266-901f-a5ce234e481c") NPC3D:public Character3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("308ddb5e-632e-5266-901f-a5ce234e481c");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Command Source") MoveTo3DComponent command_source;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="c25d7cb9-0c02-5130-8735-1a4864928ff5") NPC2D:public Character2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("c25d7cb9-0c02-5130-8735-1a4864928ff5");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Command Source") MoveTo2DComponent command_source;
};
}
