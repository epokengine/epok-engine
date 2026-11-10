#include "ThirdPersonController.hpp"

void ThirdPersonController::begin_play(){
    epok::LocalPlayer3DInputComponent::begin_play();
    auto* registry=epok::active_object_registry;
    auto* rig=registry?registry->resolve<epok::CameraRig3D>(camera):nullptr;
    if(rig)rig->make_active();
}
