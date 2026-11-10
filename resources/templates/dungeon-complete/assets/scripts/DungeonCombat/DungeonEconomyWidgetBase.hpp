#pragma once
#include "DungeonEconomyPolicyBase.hpp"
#include "DungeonEconomyPresenterEndpoint.hpp"
#include "blueprint_api.hpp"
struct EPOK_VALUE(Configuration,Id="f9fa8093-7d6c-59a3-aefd-4c83e517810a") DungeonEconomyScreen {
    uint32_t menu=0;
    uint32_t selected=0;
    uint32_t port=0;
    uint32_t card_phase=0;
    uint32_t request=0;
    uint32_t notice=0;
    uint32_t progress=0;
    uint32_t cue=0;
    bool visible=false;
    bool busy=false;
};
struct EPOK_VALUE(Id="9c0d0f55-dbc1-52c6-bc87-ea27f8799701") DungeonEconomyButtons {
    bool up=false;
    bool down=false;
    bool left=false;
    bool right=false;
    bool confirm=false;
    bool cancel=false;
};
struct EPOK_VALUE(Id="5ca569a9-d62c-57f7-be46-67361753313a") DungeonEconomyIntent {
    uint32_t command=0;
    uint32_t selected=0;
    uint32_t port=0;
};
class EPOK_CLASS(Abstract,Blueprintable,Placeable,Spawnable,Domain=UI,Id="cd4bf230-fb5d-5200-b004-1fa3c3132bfb") DungeonEconomyWidgetBase:public epok::UIActor {
    char buffer_[512]{};uint32_t cursor_=0;
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("cd4bf230-fb5d-5200-b004-1fa3c3132bfb");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Canvas",Id="cde964b9-f692-503b-8432-0d7b529d68af") epok::CanvasComponent canvas;
    EPOK_COMPONENT(Name="Panel",Id="f6358d36-4e58-576f-9dcf-c39468a69b52") epok::ImageComponent panel;
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEconomyPolicyBase",Id="878408b5-1538-5ea8-a70b-06a8ee81236e") epok::ObjectId policy{};
    EPOK_PROPERTY(Id="07bacf16-dcd3-50b7-9484-5eb49a58a831") DungeonEconomyScreen screen{0,0,0,0,0,0,0,0,false,false};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="592dd3cb-8ffb-5091-acd7-38e0391f654e") epok::ObjectId title{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="ae07322c-fb68-506c-a024-338c55edda4c") epok::ObjectId summary{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="c950c2bc-5b18-5e14-a7ba-d10610c4a928") epok::ObjectId body{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="fe694b08-2c6e-5b4a-b396-4fae95b84ee0") epok::ObjectId status{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="e840a9b3-99c0-5f9b-9fa5-c42f8ad5a816") epok::ObjectId footer{};
    DungeonEconomyPolicyBase* policy_value()const{auto* p=epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyPolicyBase>(policy):nullptr;auto* owner=p?p->get_owner():nullptr;return owner&&owner->level_id()==level_id()?p:nullptr;}
    EPOK_FUNCTION(BlueprintPure,Id="f6747fe6-2212-56ee-908c-4b2146fde64c") bool policy_live()const{return policy_value()!=nullptr;}
    EPOK_FUNCTION(BlueprintPure,Id="c8ac3f8a-d4b3-5ab1-ad27-2d9043ba4e2f") DungeonEconomySave read_saved()const{auto* p=policy_value();return p?p->saved:DungeonEconomySave{};}
    EPOK_FUNCTION(BlueprintPure,Id="c200141c-7f9f-5938-a009-6add47fe257d") DungeonSupplyTuning read_tuning()const{auto* p=policy_value();return p?p->tuning:DungeonSupplyTuning{};}
    EPOK_FUNCTION(BlueprintPure,Id="6406da8c-7331-5cb9-98ef-0b11293d1664") DungeonPocketFlow read_pocket()const{auto* p=policy_value();return p?p->pocket:DungeonPocketFlow{};}
    EPOK_FUNCTION(BlueprintPure,Id="f353ac5d-504b-520d-b0cf-588808024360") epok::ObjectId self_reference()const{return id();}
    EPOK_FUNCTION(BlueprintPure,Id="e103261c-e598-51f0-a766-84d6f2fa627e") epok::ObjectId parent_reference()const{return logical_parent();}
    EPOK_FUNCTION(BlueprintPure,Id="6c1ae93e-b65b-5397-97d4-c76d4025f73d") bool has_parent()const{return logical_parent().valid();}
    EPOK_FUNCTION(BlueprintCallable,Id="dd802efe-9401-51ea-9891-dce2453f2035") void assign_screen(DungeonEconomyScreen value){screen=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="d8d7801c-d790-5f22-89e7-ce82e52d367e") void assign_selected(uint32_t value){screen.selected=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="78c21e41-bdb4-55a0-87c3-ef8e5221de26") void assign_port(uint32_t value){screen.port=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="effa4e45-0ccd-51e2-b33a-967449a3b323") void assign_policy(epok::ObjectId value){policy=value;}
    EPOK_FUNCTION(BlueprintPure,Id="0f388f79-60a3-5006-a8eb-c320a56d584a") DungeonEconomyIntent make_intent(uint32_t command,uint32_t selected,uint32_t port)const{return {command,selected,port};}
    EPOK_FUNCTION(BlueprintCallable,Id="8b6708b1-8a2d-565b-b39a-1b1044e93129") void clear_buffer(){cursor_=0;buffer_[0]=0;}
    EPOK_FUNCTION(BlueprintCallable,Id="9de9ad62-c0c3-5f49-9f17-036eb3aecaaa") void append_text(uint32_t key){const char* value=epok::localization.get(int(key));for(uint32_t n=0;value[n]&&cursor_<511;++n)buffer_[cursor_++]=value[n];buffer_[cursor_]=0;}
    EPOK_FUNCTION(BlueprintCallable,Id="94bf9012-074f-5380-980b-f9845b15c49d") void append_unsigned(uint32_t value){char digits[10];uint32_t count=0;do{digits[count++]=char(48+value%10);value/=10;}while(value);while(count&&cursor_<511)buffer_[cursor_++]=digits[--count];buffer_[cursor_]=0;}
    EPOK_FUNCTION(BlueprintCallable,Id="9eb27bdb-7a1f-5c44-8fc0-0d6b1542976f") void copy_text(epok::ObjectId target){auto* text=epok::active_object_registry?epok::active_object_registry->resolve<epok::TextComponent>(target):nullptr;auto* owner=text?text->get_owner():nullptr;if(!owner||owner->level_id()!=level_id())return;if(auto* data=owner->data()){data->text.key=-1;data->text.set_text(buffer_);}}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="afae03f4-7579-5338-84a9-651fb0116d8b") virtual uint32_t menu_kind()const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="b6d64911-ca78-5b42-8edc-409d1a1b9ffb") virtual DungeonEconomyIntent input(DungeonEconomyButtons buttons)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="a9e154fc-dc48-5161-9000-50a0bdb6a912") virtual void refresh(DungeonEconomyScreen state)=0;
};
