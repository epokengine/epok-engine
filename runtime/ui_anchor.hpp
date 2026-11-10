#pragma once
#include "object_model.hpp"

namespace epok {
// Values are stable; an unavailable world projection keeps a readable fallback.
enum class ScreenAnchorStatus : uint32_t { Idle,Following,Fallback,InvalidContent,ContentTooLarge };
class ScreenAnchorComponent;
namespace ui_anchor {
bool valid_content(const ScreenAnchorComponent& anchor);
ScreenAnchorStatus update(ScreenAnchorComponent& anchor);
}
class EPOK_CLASS(Blueprintable,Domain=UI,Owners=UI,Capability=canvas,Id="066be6a2-ec4e-50fd-a0bd-3dae2a7b8d73") ScreenAnchorComponent : public UIComponent {
    ScreenAnchorStatus state_=ScreenAnchorStatus::Idle;
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("066be6a2-ec4e-50fd-a0bd-3dae2a7b8d73");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::RectTransformComponent",Id="cabc30ca-49e1-5f7f-a3bb-d44ff0f91994") ObjectId content{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor",Id="ed58dbec-39f7-5f84-962d-3e00a89b611e") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere,Id="5eaefb07-2d43-5422-beff-6474e12692c5") bool enabled=true;
    EPOK_PROPERTY(EditAnywhere,Id="36fc4259-63de-5d8b-afcc-a06dc214896f") Fixed offset_x=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="7a0b324c-3a3d-5b73-b5e5-c9e2bd68b679") Fixed offset_y=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="2c80fee3-cbae-5d99-b3b5-f826b485d2a2") Fixed offset_z=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="b17b49e5-34d1-5ad3-9ff7-7144c91804ed") Fixed pixel_x=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="c663fbfe-ce06-58f1-891b-f3350864a978") Fixed pixel_y=12.0;
    EPOK_PROPERTY(EditAnywhere,Id="a42ef7d0-5889-5090-a676-d12a967d59d3") Fixed margin=8.0;
    EPOK_FUNCTION(BlueprintPure,Id="e7893aa0-fc76-589a-9dde-f3a2711a61cc") bool valid_content() const{return ui_anchor::valid_content(*this);}
    EPOK_FUNCTION(BlueprintCallable,Id="66e04031-b246-5051-9064-fe3fb10bfdee") ScreenAnchorStatus refresh(){state_=enabled&&target.valid()?ui_anchor::update(*this):ScreenAnchorStatus::Idle;return state_;}
    EPOK_FUNCTION(BlueprintCallable,Id="0e67fb1d-ace8-58b5-be6c-5bcf61eb5466") void set_target(ObjectId value){if(target!=value){target=value;refresh();}}
    EPOK_FUNCTION(BlueprintPure,Id="97dbbb6f-31b5-5d7d-9ce2-fc57e48bfe67") ScreenAnchorStatus status() const{return state_;}
    void on_frame(uint32_t) override{refresh();}
    void on_disable() override{state_=ScreenAnchorStatus::Idle;}
    void end_play(EndPlayReason) override{target={};state_=ScreenAnchorStatus::Idle;}
};
}
