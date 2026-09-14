#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="a997b0f2-b3ac-45c2-9d75-9ec11dc890af") Spinner : public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("a997b0f2-b3ac-45c2-9d75-9ec11dc890af");
    uint64_t class_id() const override {return static_class_id;}
    EPOK_PROPERTY(Editable) epok::Fixed speed=45.0;
    void tick(epok::Fixed dt) override {
        auto* owner=get_owner();
        auto* root=owner && epok::active_object_registry ? epok::active_object_registry->resolve<epok::SceneComponent3D>(owner->root_id()) : nullptr;
        if(root && root->transform) root->transform->rotation[1]+=speed*dt;
    }
};
