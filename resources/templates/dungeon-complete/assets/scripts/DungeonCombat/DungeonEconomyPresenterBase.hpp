#pragma once
#include "DungeonEconomyWidgetBase.hpp"
#include "blueprint_api.hpp"
struct EPOK_VALUE(Id="e2340c95-d6d6-5787-a411-2717c21f9e02") DungeonEconomyUIContext {bool controls=false;bool can_heal=false;uint32_t station=0;epok::Fixed dt=0.0;};
struct EPOK_VALUE(Id="2171567b-b197-5e01-8963-34970dfd4921") DungeonEconomyUIOutput {
    bool blocked=false;
    bool world_visible=false;
    uint32_t heal=0;
    uint32_t cue=0;
    uint32_t menu=0;
    uint32_t selected=0;
};
struct EPOK_VALUE(Configuration,Id="65a64bbd-e3f2-511e-9af1-9856f936b2b5") DungeonEconomyUIStats {
    uint32_t transfers=0;
    uint32_t saves=0;
    uint32_t error=0;
};
struct EPOK_VALUE(Configuration,Id="54fc7e85-255d-58b3-b0db-595497e3240c") DungeonEconomyUIRuntime {
    bool started=false;
    bool spawn_pending=false;
    uint32_t spawn_wait=0;
    uint32_t transfers=0;
    uint32_t saves=0;
    uint32_t error=0;
    bool controls=false;
    uint32_t station=0;
    uint32_t input_port=0;
    bool open_cue=false;
};
struct EPOK_VALUE(Configuration,Id="df4a5a9b-72aa-54b2-a316-3f8b850ef03d") DungeonHUDStyle {
    uint32_t normal_red=0;
    uint32_t normal_green=0;
    uint32_t normal_blue=0;
    uint32_t warning_red=0;
    uint32_t warning_green=0;
    uint32_t warning_blue=0;
    uint32_t critical_red=0;
    uint32_t critical_green=0;
    uint32_t critical_blue=0;
};
struct EPOK_VALUE(Id="09071477-fa9a-5a20-9644-fd64f800ac13") DungeonHUDSnapshot {
    uint32_t health=0;
    uint32_t maximum=0;
    uint32_t boss_health=0;
    uint32_t boss_maximum=0;
    uint32_t state=0;
    uint32_t phase=0;
    bool has_key=false;
    bool aiming=false;
    bool alive=false;
    bool damage=false;
};
struct EPOK_VALUE(Id="93e636ad-a97e-5427-b21d-1f6fa484a9ef") DungeonHUDRatio {
    uint32_t remainder=0;
    uint32_t quotient=0;
};
struct EPOK_VALUE(Id="84c010fa-ab0c-59f5-b319-cc7ac1493552") DungeonHUDDamage {
    bool enabled=false;
    uint32_t border=0;
    uint32_t red=0;
    uint32_t green=0;
    uint32_t blue=0;
};

class EPOK_CLASS(Abstract,Blueprintable,Placeable,PlacedOnly,Domain=UI,Id="c2107d00-9d72-5baa-bf79-631479772dff") DungeonEconomyPresenterBase:public DungeonEconomyPresenterEndpoint {
    epok::InputCapture capture_{};uint32_t heal_=0;epok::ObjectId bound_{};
    DungeonHUDSnapshot hud_{};DungeonHUDDamage damage_{};
    DungeonEconomyIntent temporary_intent_{};DungeonHUDRatio temporary_ratio_{};
    bool hud_valid_=false,hud_modal_=false;uint32_t old_station_=0;DungeonHUDStyle old_style_{};
    struct Authored {epok::ObjectId target{};epok::Fixed width{};uint32_t source=0;} authored_[11]{};
    char hud_buffer_[64]{};
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("c2107d00-9d72-5baa-bf79-631479772dff");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEconomyPolicyBase",Id="c8b96a2f-16f4-5a90-93c8-49b4665fc44f") epok::ObjectId policy{};
    EPOK_PROPERTY(ActorClass="DungeonEconomyWidgetBase",Id="10ce20d8-9398-51ae-8286-4aaf69ff5ed3") epok::ObjectId view{};
    EPOK_PROPERTY(Id="91219944-1e5a-5e8c-86c6-148c3675d7d6") DungeonEconomyScreen screen{0,0,0,0,0,0,0,0,false,false};
    EPOK_PROPERTY(EditAnywhere,Id="7e2c620d-9f10-5fb9-8562-a46e1956500f") DungeonEconomyUIRuntime runtime{false,false,0,0,0,0,false,0,0,false};
    EPOK_PROPERTY(EditAnywhere,Id="06356824-d09d-532f-903a-ec4026f0670c") DungeonHUDStyle style{148,255,102,255,205,75,255,85,65};
    DungeonEconomyPolicyBase* policy_value()const{auto* p=epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyPolicyBase>(policy):nullptr;auto* owner=p?p->get_owner():nullptr;return owner&&owner->level_id()==level_id()?p:nullptr;}
    DungeonEconomyWidgetBase* view_value()const{return epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyWidgetBase>(view):nullptr;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="bbfdb35e-e4fc-56a4-b100-1202634cb07f") epok::ObjectId energy_label{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="22e84edf-3a74-5609-8443-8693a6ec9922") epok::ObjectId energy_bar{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="95627b0a-38d3-52d0-9ac5-50077acfd9d8") epok::ObjectId energy_panel{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="d18cdb6b-804d-5e2d-8bc1-d34e5034453d") epok::ObjectId boss_bar{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="217f7063-7d96-57e1-9ec7-c11131146f44") epok::ObjectId reticle{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="700963ba-4592-5eda-b0df-7259c01dc15d") epok::ObjectId message_panel{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="2ed63aa7-b454-5313-9f18-601c80437a03") epok::ObjectId objective{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="cfe835e0-db7f-5674-8b48-74cbacd6191b") epok::ObjectId key_status{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="66b130c5-922d-5e67-bf4c-91a3c7784b48") epok::ObjectId game_message{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="460a01fa-b042-507b-98a5-c3574a6cee50") epok::ObjectId boss_label{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="402ee2de-55c1-54a8-8340-bb818d5a2e09") epok::ObjectId controls_hint{};
    epok::ActorData* hud_data(epok::ObjectId target)const{auto* a=epok::active_object_registry?epok::active_object_registry->resolve<epok::UIActor>(target):nullptr;return a&&a->level_id()==level_id()?a->data():nullptr;}
    Authored* authored(epok::ObjectId target){for(auto& v:authored_)if(v.target==target)return &v;auto* d=hud_data(target);if(!d)return nullptr;for(auto& v:authored_)if(!v.target.valid()){v={target,d->rect.size[0],d->image.region[2]};return &v;}return nullptr;}
    EPOK_FUNCTION(BlueprintPure,Id="0f6fa63a-b4cf-5ec0-a68b-93138cd6fff4") bool policy_live()const{return policy_value()!=nullptr;}
    EPOK_FUNCTION(BlueprintPure,Id="de7b3008-a588-533f-9dd7-935ec6f2b1a4") bool same_policy()const{return bound_==policy;}
    EPOK_FUNCTION(BlueprintCallable,Id="f8deb1dc-c1df-5b96-92be-2fb7b6aa5681") void assign_bound(epok::ObjectId value){bound_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="8e48a83b-7022-591f-a0ba-6eedb60791f8") DungeonSupplyTransaction read_tx()const{auto* p=policy_value();return p?p->tx:DungeonSupplyTransaction{};}
    EPOK_FUNCTION(BlueprintPure,Id="d23e4264-8333-5d91-8549-9bb30f15e847") bool read_loaded()const{auto* p=policy_value();return p&&p->loaded;}
    EPOK_FUNCTION(BlueprintPure,Id="bf16c344-0240-536d-8f55-c9ec95cc1491") DungeonPocketFlow read_pocket()const{auto* p=policy_value();return p?p->pocket:DungeonPocketFlow{};}
    EPOK_FUNCTION(BlueprintCallable,Id="fad5f694-d220-5769-9ac9-fdb1073c16d0") void assign_screen(DungeonEconomyScreen value){screen=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="63b8a914-86ec-50ad-8c94-ead317533273") void assign_stats(DungeonEconomyUIStats value){runtime.transfers=value.transfers;runtime.saves=value.saves;runtime.error=value.error;}
    EPOK_FUNCTION(BlueprintPure,Id="e5164848-05ab-5c10-bc0e-69148fc12ca0") DungeonEconomyUIStats make_stats(uint32_t transfers,uint32_t saves,uint32_t error)const{return {transfers,saves,error};}
    EPOK_FUNCTION(BlueprintPure,Id="5e2163cc-7f59-59cc-bec0-8f776e440617") DungeonEconomyScreen make_screen(uint32_t menu,uint32_t selected,uint32_t port,uint32_t card_phase,uint32_t request,uint32_t notice,uint32_t progress,uint32_t cue,bool visible,bool busy)const{return {menu,selected,port,card_phase,request,notice,progress,cue,visible,busy};}
    EPOK_FUNCTION(BlueprintPure,Id="73c0e873-75d8-59d2-ad18-4f05214f49ea") DungeonEconomyButtons make_buttons(bool up,bool down,bool left,bool right,bool confirm,bool cancel)const{return {up,down,left,right,confirm,cancel};}
    EPOK_FUNCTION(BlueprintPure,Id="577d9855-e896-53df-aa67-59d2a224a1d7") DungeonEconomyUIOutput make_output(bool blocked,bool world_visible,uint32_t heal,uint32_t cue,uint32_t menu,uint32_t selected)const{return {blocked,world_visible,heal,cue,menu,selected};}
    EPOK_FUNCTION(BlueprintPure,Id="bca7b195-bd59-59c5-af56-25307408ef1a") epok::InputCapture capture_token()const{return capture_;}
    EPOK_FUNCTION(BlueprintCallable,Id="c80ec40f-d7cd-599e-95fc-994e7e3d917f") void assign_capture(epok::InputCapture value){capture_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="89953dcc-ec47-5279-b84b-21e8e61ed5f3") epok::InputCapture empty_capture()const{return {};}
    EPOK_FUNCTION(BlueprintPure,Id="5fa8f1ff-fed5-51e8-be38-5eea2f619e25") epok::ObjectId empty_reference()const{return {};}
    EPOK_FUNCTION(BlueprintPure,Id="1d1180db-2dec-538b-ba02-5ab04d6272b2") epok::ObjectId self_reference()const{return id();}
    EPOK_FUNCTION(BlueprintPure,Id="43934072-4f61-599d-9444-c4a0f98aca5c") epok::ObjectId parent_of(epok::ObjectId value)const{auto* a=epok::active_object_registry?epok::active_object_registry->resolve<epok::Actor>(value):nullptr;return a?a->logical_parent():epok::ObjectId{};}
    EPOK_FUNCTION(BlueprintCallable,Id="eda8c525-0aa2-5987-a001-8126bbef3501") void assign_view(epok::ObjectId value){view=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="781cce1c-9f4c-558d-811e-97fc12fa901c") void assign_started(bool value){runtime.started=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="47d1cb56-b4c2-5c0f-a69c-b3ffab7e0c14") void assign_pending(bool value){runtime.spawn_pending=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="ecd8c0d3-867e-5468-87ea-a92eb8bb8c79") void assign_wait(uint32_t value){runtime.spawn_wait=value;}
    EPOK_FUNCTION(BlueprintPure,Id="a9b3966d-4880-5a43-93e2-b38cf67aeba4") uint32_t pending_heal()const{return heal_;}
    EPOK_FUNCTION(BlueprintCallable,Id="2d2fe5dd-21fb-5d11-8544-62ff84c875b7") void assign_heal(uint32_t value){heal_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="61d4bd22-c018-5e39-a2a7-46c3b25e83b7") epok::GameplayAction input_action(uint32_t value)const{return epok::GameplayAction(value);}
    EPOK_FUNCTION(BlueprintPure,Id="9a425ec5-3768-551f-a222-aecf73dcc7d9") uint32_t card_state(epok::CardState value)const{return uint32_t(value);}
    EPOK_FUNCTION(BlueprintPure,Id="752ffc5e-0f66-5243-b30a-326bab11e9f7") uint32_t card_error(epok::CardError value)const{return uint32_t(value);}
    EPOK_FUNCTION(BlueprintCallable,Id="f4f773c5-9359-5127-82f1-b2485ca79ddd") void assign_controls(bool value){runtime.controls=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="4b7ca323-bacd-5078-a2b1-449e360223f3") void assign_station(uint32_t value){runtime.station=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="4e7a86e6-8d96-5af8-9a5d-69d10bc19714") void assign_open_cue(bool value){runtime.open_cue=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="096a03a7-219f-5a93-b233-2a906465f6d0") void assign_modal(bool value){hud_modal_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="1c3ada52-da51-5307-8a1d-37e3d627307f") DungeonHUDSnapshot read_hud()const{return hud_;}
    EPOK_FUNCTION(BlueprintCallable,Id="e7010294-bdd5-577c-889b-e478b56e81c8") void assign_hud(DungeonHUDSnapshot value,bool visible){hud_=value;hud_modal_=visible;old_style_=style;old_station_=runtime.station;hud_valid_=true;}
    EPOK_FUNCTION(BlueprintPure,Id="7efc7d66-9824-5b3d-8a59-42a4b4b85d1a") bool hud_same(DungeonHUDSnapshot value,bool visible)const{return hud_valid_&&hud_modal_==visible&&old_station_==runtime.station&&hud_.health==value.health&&hud_.maximum==value.maximum&&hud_.boss_health==value.boss_health&&hud_.boss_maximum==value.boss_maximum&&hud_.state==value.state&&hud_.phase==value.phase&&hud_.has_key==value.has_key&&hud_.aiming==value.aiming&&hud_.alive==value.alive&&hud_.damage==value.damage&&old_style_.normal_red==style.normal_red&&old_style_.normal_green==style.normal_green&&old_style_.normal_blue==style.normal_blue&&old_style_.warning_red==style.warning_red&&old_style_.warning_green==style.warning_green&&old_style_.warning_blue==style.warning_blue&&old_style_.critical_red==style.critical_red&&old_style_.critical_green==style.critical_green&&old_style_.critical_blue==style.critical_blue;}
    EPOK_FUNCTION(BlueprintCallable,Id="6e320b4b-93af-5ad8-9ad8-a6dd121168b4") void assign_damage(DungeonHUDDamage value){damage_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="791812b9-06a7-5802-8e6f-c399c9533311") DungeonHUDDamage read_damage()const{return damage_;}
    EPOK_FUNCTION(BlueprintCallable,Id="6e250af1-36eb-5d11-8faf-6fee20ceb337") void hud_damage_copy(DungeonHUDDamage value){if(auto* d=data()){d->image.enabled=value.enabled;for(auto& edge:d->image.borders)edge=uint16_t(value.border);d->image.color[0]=uint8_t(value.red);d->image.color[1]=uint8_t(value.green);d->image.color[2]=uint8_t(value.blue);}}
    EPOK_FUNCTION(BlueprintPure,Id="ae819adc-a4d8-5701-beb0-5acdbbfc0898") DungeonHUDRatio make_ratio(uint32_t remainder,uint32_t quotient)const{return {remainder,quotient};}
    EPOK_FUNCTION(BlueprintCallable,Id="3eee8ed7-3818-5110-b8ad-0b96678bd587") void hud_flags(epok::ObjectId target,bool text,bool image,bool progress){if(auto* d=hud_data(target)){d->text.enabled=text;d->image.enabled=image;d->progress.enabled=progress;}}
    EPOK_FUNCTION(BlueprintCallable,Id="ec40d620-2240-5fc1-922e-adb12f74f413") void hud_border(bool value){if(auto* d=data())d->image.enabled=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="d3fb9d18-bf9a-5ff0-a284-5ffa7b6b9509") void hud_key(epok::ObjectId target,uint32_t key){if(auto* d=hud_data(target)){d->text.key=-1;d->text.set_text(epok::localization.get(int(key)));}}
    EPOK_FUNCTION(BlueprintCallable,Id="8616b528-2ea2-56fb-bdc2-4eaea15c377d") void hud_number(epok::ObjectId target,uint32_t key,uint32_t value){uint32_t at=0;const char* s=epok::localization.get(int(key));while(*s&&at<50)hud_buffer_[at++]=*s++;char digits[10];uint32_t n=0;do{digits[n++]=char(48+value%10);value/=10;}while(value);while(n&&at<63)hud_buffer_[at++]=digits[--n];hud_buffer_[at]=0;if(auto* d=hud_data(target)){d->text.key=-1;d->text.set_text(hud_buffer_);}}
    EPOK_FUNCTION(BlueprintPure,Id="cf869ce1-9b7b-582a-98d5-6ff23311b56a") uint32_t hud_source_width(epok::ObjectId target)const{auto* v=const_cast<DungeonEconomyPresenterBase*>(this)->authored(target);return v?v->source:0;}
    EPOK_FUNCTION(BlueprintPure,Id="d6f3da49-8f81-5326-b704-6718c54bc024") epok::Fixed hud_authored_width(epok::ObjectId target)const{auto* v=const_cast<DungeonEconomyPresenterBase*>(this)->authored(target);return v?v->width:epok::Fixed{};}
    EPOK_FUNCTION(BlueprintPure,Id="75d948e2-1d83-5e6f-b25b-58b7b1de35f2") epok::Fixed hud_fixed(uint32_t raw)const{return epok::Fixed(int32_t(raw),epok::Fixed::RAW);}
    EPOK_FUNCTION(BlueprintCallable,Id="a071a2f7-ae88-51d1-b4c3-ba834420933e") void hud_crop(epok::ObjectId target,epok::Fixed width,uint32_t source){if(auto* d=hud_data(target)){d->rect.size[0]=width;d->image.region[2]=uint16_t(source);}}
    EPOK_FUNCTION(BlueprintCallable,Id="f983803e-e675-58f8-aff4-e13c70cf0323") void hud_progress(epok::ObjectId target,epok::Fixed value){if(auto* d=hud_data(target))d->progress.value=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="f2765b03-f19e-5415-af96-a82a92c8238a") void hud_tint(epok::ObjectId target,uint32_t red,uint32_t green,uint32_t blue){if(auto* d=hud_data(target)){d->image.color[0]=uint8_t(red);d->image.color[1]=uint8_t(green);d->image.color[2]=uint8_t(blue);}}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="f1c7bcd1-fe17-5df6-8554-ceac88f60fab") virtual DungeonEconomyUIOutput frame(DungeonEconomyUIContext context,uint32_t microseconds)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="85d76444-ad6c-50c0-b17e-ebad5337c019") virtual void shutdown()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="9a5cc066-392b-5387-8a5b-02161a21d9e3") virtual void take(DungeonSupplyDecision decision)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="7ebd4eb5-ab90-53a7-99c7-da7d0cf51ad9") virtual void load()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="b0844c45-c529-52e1-96e1-6f21086a4994") virtual void save()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="8ae4965d-7ad1-5b24-9ce0-2d095d7f9415") virtual void dispatch(DungeonSupplyDecision decision)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="4161ffd8-848a-56f5-ab79-5d4b860771e1") virtual void poll_card()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="5d1716ed-fd2f-5c47-8b0e-9e29f912431f") virtual void close_view()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="8336be8f-10ca-5899-bb50-7edc875b7fb8") virtual void prepare_view()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="06172711-ddfc-5466-a74f-a9c655751f62") virtual DungeonHUDRatio ratio_double(DungeonHUDRatio value,uint32_t denominator)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="737ca074-5c8c-5ff4-b1c9-e85dcf4c4a68") virtual DungeonHUDRatio ratio_add(DungeonHUDRatio value,uint32_t numerator,uint32_t denominator,uint32_t bit)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="f434c12e-b15b-5c7a-80c7-d74d82f672df") virtual DungeonHUDRatio ratio_scale(uint32_t numerator,uint32_t denominator,uint32_t scale)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="7d3100ab-510f-54ac-b9e2-fd7adb65b13a") virtual void hide_hud()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="5269cbad-ecb1-5ad6-93b7-27d1910af32c") virtual void damage_hud(DungeonHUDDamage value)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="74f720c3-4eeb-5e99-99b8-e49ed79eaee4") virtual void render_hud(DungeonHUDSnapshot state)=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="9dba0ffe-856f-5fa2-80f8-4e0021444666") virtual bool station_available(uint32_t port) const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="4b7b674e-7eb7-54f5-8b59-576cbeafa177") virtual bool open_station(uint32_t menu,uint32_t port)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="5257e846-3f11-5c39-a567-72be03fb3bed") virtual void station_focus(uint32_t menu,bool focused)=0;
    EPOK_FUNCTION(BlueprintCallable,Id="07b83324-d250-53a6-9c5b-a567808aaac6") void temporary_intent_set(DungeonEconomyIntent value){temporary_intent_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="2c97368c-50ba-5456-8f55-80e191b82ec5") DungeonEconomyIntent temporary_intent_get()const{return temporary_intent_;}
    EPOK_FUNCTION(BlueprintCallable,Id="74c6ea9d-141f-5405-adcf-50d0e0bb7730") void temporary_ratio_set(DungeonHUDRatio value){temporary_ratio_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="7342a8a6-7049-5f81-9003-609080dc05ab") DungeonHUDRatio temporary_ratio_get()const{return temporary_ratio_;}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="df926440-a22d-5006-abfc-53f90e066480") virtual bool frame_initialize(DungeonEconomyUIContext context,uint32_t microseconds)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="d655e3a4-5d64-589f-80d5-f84d4854cc8c") virtual DungeonEconomyUIOutput frame_controls(DungeonEconomyUIContext context)=0;
};
