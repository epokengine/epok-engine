#pragma once
#include "epok.hpp"

// Project extension point. Movement, jumps and turning belong to the shared
// motor; camera orbit and locomotion playback are separate scene components.
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="9233f481-d27e-4765-a2d3-4dcfcb0cc910") ThirdPersonController : public epok::LocalPlayer3DInputComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("9233f481-d27e-4765-a2d3-4dcfcb0cc910");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    void begin_play() override;
};
