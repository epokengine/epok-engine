#pragma once
#define EPOK_INCLUDE_FROM_CHARACTER_INPUT 1
#include "character_commands.hpp"
#include "gameplay_control.hpp"
#include "camera_rig.hpp"
#undef EPOK_INCLUDE_FROM_CHARACTER_INPUT

namespace epok {
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Capability=gameplay-control,Id="73925989-7e27-5509-a493-9e6c1b71939d") LocalPlayer3DInputComponent:public CharacterCommandSource3DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("73925989-7e27-5509-a493-9e6c1b71939d");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) uint32_t port=0;
    EPOK_PROPERTY(EditAnywhere) Fixed max_speed=6.0;
    EPOK_PROPERTY(EditAnywhere) bool allow_jump=true;
    EPOK_PROPERTY(EditAnywhere) bool camera_relative=true;
    EPOK_PROPERTY(EditAnywhere) bool turn_to_movement=true;
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") ObjectId camera{};
    CharacterCommandResult3D calculate_command(Fixed)override{
        CharacterCommandResult3D result;
        if(port>=4||max_speed<0.0||!character3d::bounded(max_speed,4096)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
        auto* value=target_motor();auto* controlled=value?value->get_owner():nullptr;
        if(!controlled){result.status=CharacterCommandStatus::InvalidMotor;return result;}
        const auto x=GameplayInputLibrary::sample(GameplayAction::MoveX,port),y=GameplayInputLibrary::sample(GameplayAction::MoveY,port);
        if(!x.connected||!y.connected){result.status=CharacterCommandStatus::NoStep;return result;}
        Fixed yaw=0.0;
        if(camera_relative){
            ObjectId selected=camera;
            if(!selected.valid())if(auto* character=active_object_registry->resolve<Character3D>(controlled->id()))selected=character->camera;
            if(selected.valid()){
                auto* rig=active_object_registry->resolve<CameraRig3D>(selected);
                if(!rig||!is_active(rig)||rig->level_id()!=controlled->level_id()){result.status=CharacterCommandStatus::InvalidTarget;return result;}
                yaw=rig->yaw;
            }else if(auto* current=active_camera().get()){
                const auto world=gameplay_world_affine(current);if(world.success)yaw=MathLibrary::heading_degrees(world.basis_z[0],world.basis_z[2]);
            }
        }
        if(GameplayInputLibrary::captured(GameplayAction::Jump,port))release_command();
        const auto intent=MathLibrary::stick_intent(x.value,y.value);
        const Fixed sine=MathLibrary::sine_degrees(yaw),cosine=MathLibrary::cosine_degrees(yaw);
        result.command.velocity={(sine*intent.y+cosine*intent.x)*max_speed,(cosine*intent.y-sine*intent.x)*max_speed};
        result.command.turn=turn_to_movement&&intent.strength.raw()!=0;
        if(result.command.turn)result.command.heading=yaw+MathLibrary::heading_degrees(intent.x,intent.y);
        result.command.jump=allow_jump&&GameplayInputLibrary::sample(GameplayAction::Jump,port).pressed;return result;
    }
    void command_submitted(CharacterCommandResult3D result)override{
        if(result.command.jump)GameplayInputLibrary::take_pressed(GameplayAction::Jump,port);
    }
};
class EPOK_CLASS(Blueprintable,Domain=World2D,Owners=World2D,Capability=gameplay-control,Id="26900842-e053-5f2c-9057-250242310406") LocalPlayer2DInputComponent:public CharacterCommandSource2DComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("26900842-e053-5f2c-9057-250242310406");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) uint32_t port=0;
    EPOK_PROPERTY(EditAnywhere) Fixed max_speed=96.0;
    EPOK_PROPERTY(EditAnywhere) bool allow_jump=true;
    CharacterCommandResult2D calculate_command(Fixed)override{
        CharacterCommandResult2D result;
        if(port>=4||max_speed<0.0||!character2d::bounded(max_speed)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
        auto* body=target_body();if(!body){result.status=CharacterCommandStatus::InvalidMotor;return result;}
        const auto x=GameplayInputLibrary::sample(GameplayAction::MoveX,port),y=GameplayInputLibrary::sample(GameplayAction::MoveY,port);
        if(!x.connected||!y.connected){result.status=CharacterCommandStatus::NoStep;return result;}
        if(GameplayInputLibrary::captured(GameplayAction::Jump,port))release_command();
        if(body->floating){
            const auto intent=MathLibrary::stick_intent(x.value,y.value);result.command.velocity={intent.x*max_speed,-intent.y*max_speed};
        }else{
            const auto up_x=body->up_direction[0],up_y=body->up_direction[1];
            if(!character2d::normalized_up(up_x,up_y)){result.status=CharacterCommandStatus::InvalidConfiguration;return result;}
            const auto intent=MathLibrary::stick_intent(x.value,0.0);result.command.velocity={-up_y*intent.x*max_speed,up_x*intent.x*max_speed};
            result.command.jump=allow_jump&&GameplayInputLibrary::sample(GameplayAction::Jump,port).pressed;
        }return result;
    }
    void command_submitted(CharacterCommandResult2D result)override{
        if(result.command.jump)GameplayInputLibrary::take_pressed(GameplayAction::Jump,port);
    }
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="e3832599-5fa1-5d20-9714-1d32934f63ef") Player3D:public Character3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e3832599-5fa1-5d20-9714-1d32934f63ef");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Player Input") LocalPlayer3DInputComponent input_source;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="6551d454-ed4c-579f-9a85-95cae7eb3a19") Player2D:public Character2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("6551d454-ed4c-579f-9a85-95cae7eb3a19");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Player Input") LocalPlayer2DInputComponent input_source;
};
}
