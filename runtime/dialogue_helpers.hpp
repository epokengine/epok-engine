#pragma once
#include "dialogue.hpp"

namespace epok {
namespace ui_anchor {
bool valid_binding(ObjectId reference,const Actor& owner);
void set_target(ObjectId reference,Actor& owner,ObjectId target);
}
struct EPOK_VALUE(Id="790ab107-6935-55ee-97fb-dbb11a13c467") DialogueTiming {
    uint32_t reveal_ms=30,auto_advance_ms=0;
};
struct EPOK_VALUE(Id="453acec4-0727-5ee5-9ed5-3e5994ccb8a8") DialogueHandle {
    uint32_t generation=0;
};
struct EPOK_VALUE(Id="d7ce88f3-ab19-568a-9fd5-6d6538b08648") DialogueSnapshot {
    DialoguePhase phase=DialoguePhase::Idle;
    DialogueResult result=DialogueResult::NoSession;
    uint32_t generation=0,node=0,revealed=0,glyphs=0,page=0,pages=0;
    uint32_t choices=0,choice_page=0,selected=0,port=0;
    ObjectId source{},initiator{},presenter{},speaker{};
};
struct EPOK_VALUE(Id="407a968a-dd70-5629-91fe-b2b1c283115a") DialogueStartResult {
    DialogueHandle session{};
    DialogueResult result=DialogueResult::NoSession;
};
namespace dialogue {
// Compare before pruning: a stale handle cannot touch the current capture.
inline bool matches_handle(DialogueHandle handle) {return handle.generation&&session().snapshot().generation==handle.generation;}
inline DialogueSnapshot snapshot_value() {
    DialogueSnapshot result;const auto& value=session();const auto state=value.snapshot();
    result.phase=state.phase;result.result=state.result;result.generation=state.generation;result.node=state.node;
    result.revealed=state.revealed;result.glyphs=state.glyphs;result.page=state.page;result.pages=state.pages;
    result.choices=state.choices;result.choice_page=state.choice_page;result.selected=state.selected;result.port=state.port;
    result.source=value.source();result.initiator=value.initiator();result.presenter=value.presenter();result.speaker=value.speaker();
    return result;
}
}
struct EPOK_FUNCTION_LIBRARY(Category="Dialogue", Capability=dialogue, Id="57ae5208-13d3-5702-98d8-dcb2c5faa5e5") DialogueSessionLibrary {
    EPOK_FUNCTION(BlueprintCallable, ResourceAcquisition, Id="6071f61f-ece2-51e3-a8b1-b69c3d67430b") static DialogueStartResult start(ObjectId source,ObjectId initiator,uint32_t port);
    EPOK_FUNCTION(BlueprintPure, Id="54f41b9b-223a-588b-8ddd-56b057cd259a") static bool valid(DialogueHandle handle) {
        if(!dialogue::matches_handle(handle))return false;
        auto& value=dialogue::session();value.prune();return value.active();
    }
    EPOK_FUNCTION(BlueprintPure, Id="08500ccf-42ec-5977-8ffd-cc3548c3abd2") static DialogueHandle owner_session(ObjectId source) {
        auto& value=dialogue::session();
        if(value.source()!=source)return {};
        value.prune();return value.active()?DialogueHandle{value.snapshot().generation}:DialogueHandle{};
    }
    EPOK_FUNCTION(BlueprintPure, Id="3fa15b2e-32f8-5c5d-9d87-2687d6d939d0") static DialogueSnapshot snapshot(DialogueHandle handle) {
        if(!dialogue::matches_handle(handle))return {};
        dialogue::session().prune();return dialogue::snapshot_value();
    }
    EPOK_FUNCTION(BlueprintCallable, Id="eec69158-55a7-54a0-a6de-7192d5ebfcdc") static DialogueResult advance(DialogueHandle handle) {
        return dialogue::matches_handle(handle)&&dialogue::session().active()?dialogue::session().advance():DialogueResult::NoSession;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="c6fd86e4-7fb2-5aad-b93c-160431c37355") static DialogueResult cancel(DialogueHandle handle) {
        return dialogue::matches_handle(handle)&&dialogue::session().active()?dialogue::session().cancel():DialogueResult::NoSession;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="9a3bf56f-9d15-5922-a448-63dfb2676375") static DialogueResult move_selection(DialogueHandle handle,int32_t delta) {
        return dialogue::matches_handle(handle)&&dialogue::session().active()?dialogue::session().move_selection(delta):DialogueResult::NoSession;
    }
    EPOK_FUNCTION(BlueprintPure, Id="4d28d6d9-d21a-5821-b6d2-a84f917b6f80") static uint32_t choice_id(DialogueHandle handle,uint32_t visible) {
        return valid(handle)?dialogue::session().choice_id(visible):0;
    }
};

class DialoguePresenter;
namespace dialogue {
bool prepare_presenter(ObjectId presenter,ObjectId source,const Resource& resource);
const Resource* presenter_resource(ObjectId presenter,uint64_t conversation,uint64_t style);
Timing presenter_timing(ObjectId presenter,const Resource& resource);
void bind_widget_layout(Actor& presenter,uint64_t layout);
void begin_presenter(ObjectId presenter,ObjectId source);
void release_presenter(ObjectId presenter,ObjectId source,DialogueResult result);
}
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D|UI, Cardinality=Multiple, Capability=dialogue, Id="bdd1ba5d-d32f-57b8-83f3-7b2f6cd37060") DialogueRoleBindingsComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("bdd1ba5d-d32f-57b8-83f3-7b2f6cd37060");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere, Id="96365767-6b59-5b2a-81f8-422b8521d440") uint32_t role0=0;
    EPOK_PROPERTY(EditAnywhere, Id="95eb7436-9b76-54bc-bfbb-32566f9c1aa8") ObjectId target0{};
    EPOK_PROPERTY(EditAnywhere, Id="c99eb337-5459-57b5-999d-c9888278c305") uint32_t role1=0;
    EPOK_PROPERTY(EditAnywhere, Id="3127c31e-d9df-5163-be5b-8252804651ef") ObjectId target1{};
    EPOK_PROPERTY(EditAnywhere, Id="61192b0f-9a48-5dbf-aa9f-38ebce88108c") uint32_t role2=0;
    EPOK_PROPERTY(EditAnywhere, Id="df7497d1-e024-596d-b49f-70335747e89f") ObjectId target2{};
    EPOK_PROPERTY(EditAnywhere, Id="2d7de752-4482-54da-96d6-cefdb9a0fe98") uint32_t role3=0;
    EPOK_PROPERTY(EditAnywhere, Id="594c6145-3d4c-5f3e-873d-465d559c4626") ObjectId target3{};
    EPOK_PROPERTY(EditAnywhere, Id="32505643-e9ae-5979-8019-1dedafc3a77f") uint32_t role4=0;
    EPOK_PROPERTY(EditAnywhere, Id="84780c9b-ec67-531d-9262-d28a0a070584") ObjectId target4{};
    EPOK_PROPERTY(EditAnywhere, Id="97dc03e0-8d3c-5c86-a8d1-085484a67115") uint32_t role5=0;
    EPOK_PROPERTY(EditAnywhere, Id="a407e00c-40f9-58af-ae78-8da2078a8761") ObjectId target5{};
    EPOK_PROPERTY(EditAnywhere, Id="5314d501-7091-5f87-ad0c-7d89691c5271") uint32_t role6=0;
    EPOK_PROPERTY(EditAnywhere, Id="9dadfe9c-17cc-57ef-ac87-00a4bf5715ca") ObjectId target6{};
    EPOK_PROPERTY(EditAnywhere, Id="a0f61f72-3311-5f9e-8fed-583ee813e284") uint32_t role7=0;
    EPOK_PROPERTY(EditAnywhere, Id="19921d56-f168-5885-9c0c-280bfdf29cc9") ObjectId target7{};
    EPOK_FUNCTION(BlueprintPure, Id="20dd5e25-8a8d-5cca-b2ae-61c2e32c73eb") uint32_t role_id(uint32_t slot) const {switch(slot){case 0:return role0;case 1:return role1;case 2:return role2;case 3:return role3;case 4:return role4;case 5:return role5;case 6:return role6;case 7:return role7;default:return 0;}}
    EPOK_FUNCTION(BlueprintPure, Id="a8c192c8-920d-571c-ba75-30bfea52f5e2") ObjectId receiver(uint32_t slot) const {switch(slot){case 0:return target0;case 1:return target1;case 2:return target2;case 3:return target3;case 4:return target4;case 5:return target5;case 6:return target6;case 7:return target7;default:return {};}}
    EPOK_FUNCTION(BlueprintCallable, Id="93ccfd88-1cda-5212-b86a-ff4a7789dc32") bool set_binding(uint32_t slot,uint32_t role,ObjectId target) {switch(slot){case 0:role0=role;target0=target;return true;case 1:role1=role;target1=target;return true;case 2:role2=role;target2=target;return true;case 3:role3=role;target3=target;return true;case 4:role4=role;target4=target;return true;case 5:role5=role;target5=target;return true;case 6:role6=role;target6=target;return true;case 7:role7=role;target7=target;return true;default:return false;}}
};
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Capability=dialogue, Id="aa343ddf-5098-5b32-b993-d8b9b619d455") DialogueInteractionComponent : public InteractableComponent {
    bool starting_=false,notified_=false;
    DialogueResult last_=DialogueResult::NoSession;
    ObjectId prepared_presenter_{};
    static void started(ObjectId source,ObjectId initiator,uint32_t port,uint32_t generation) {
        auto* value=active_object_registry?active_object_registry->resolve<DialogueInteractionComponent>(source):nullptr;
        if(value&&interaction::live(source)){
            dialogue::begin_presenter(value->prepared_presenter_,source);
            if(interaction::live(source)&&interaction::live(initiator)&&dialogue::session().active())value->dialogue_started(initiator,port,{generation});
        }
    }
    static void entered(ObjectId source,uint32_t node) {
        auto* value=active_object_registry?active_object_registry->resolve<DialogueInteractionComponent>(source):nullptr;
        if(value&&interaction::live(source))value->dialogue_node_entered(node);
    }
    static InteractionResult interaction_result(DialogueResult result) {
        if(result==DialogueResult::Completed)return InteractionResult::Accepted;
        if(result==DialogueResult::Busy||result==DialogueResult::CaptureExhausted)return InteractionResult::Busy;
        if(result==DialogueResult::InvalidBinding||result==DialogueResult::InvalidData||result==DialogueResult::MissingPresenter)return InteractionResult::MissingBinding;
        return InteractionResult::Unavailable;
    }
    void finish_attempt(ObjectId initiator,uint32_t port,DialogueResult result) {
        last_=result;notified_=true;
        const auto view=prepared_presenter_;prepared_presenter_={};
        dialogue::release_presenter(view,id(),result);
        complete(interaction_result(result));
        if(interaction::live(id()))dialogue_finished(initiator,port,result);
    }
    static void finished(ObjectId source,ObjectId initiator,ObjectId presenter,uint32_t port,DialogueResult result) {
        dialogue::release_presenter(presenter,source,result);
        auto* value=active_object_registry?active_object_registry->resolve<DialogueInteractionComponent>(source):nullptr;
        if(value&&interaction::live(source))value->finish_attempt(initiator,port,result);
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("aa343ddf-5098-5b32-b993-d8b9b619d455");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere, AssetKind=Dialogue, Id="42b86ee7-d1f5-506d-8b98-7db1d092c77f") uint64_t conversation=0;
    EPOK_PROPERTY(EditAnywhere, AssetKind=DialogueStyle, Id="6c5c9e2c-c8fd-53e6-88b7-7fefcf322ab9") uint64_t style=0;
    EPOK_PROPERTY(EditAnywhere, ActorClass="epok::DialoguePresenter", Id="aba4b044-0cf8-53ab-9b78-3e9f1b22073c") ObjectId presenter{};
    EPOK_PROPERTY(EditAnywhere, Id="4607ac90-3ea4-5437-b564-f84e5e36582e") bool automatic_input=true;
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::DialogueRoleBindingsComponent", Id="1927a293-81b2-5332-af6e-6e76ff19becc") ObjectId bindings{};
    EPOK_FUNCTION(BlueprintEvent, Id="d8a0e23c-a846-56b7-9d9f-7f99864492b8") virtual void dialogue_started(ObjectId initiator,uint32_t port,DialogueHandle session){(void)initiator;(void)port;(void)session;}
    EPOK_FUNCTION(BlueprintEvent, Id="bb4c2b7e-dde9-56b2-9a77-89001f49d2a6") virtual void dialogue_node_entered(uint32_t node){(void)node;}
    EPOK_FUNCTION(BlueprintEvent, Id="fcc225e4-a304-5142-83c5-77b59c3c2911") virtual void dialogue_finished(ObjectId initiator,uint32_t port,DialogueResult result){(void)initiator;(void)port;(void)result;}
    EPOK_FUNCTION(BlueprintPure, Id="7887bc2a-a11e-5779-9960-ed2fe0b13dee") DialogueResult last_result() const{return last_;}
    EPOK_FUNCTION(BlueprintPure, Id="4e4dea1b-68b3-5d50-b0ff-58189bd7f1b9") DialogueHandle session() const{return DialogueSessionLibrary::owner_session(id());}
    EPOK_FUNCTION(BlueprintCallable, ResourceAcquisition, Id="e3b0bdf8-ced8-57a7-88a3-5841224e2457") DialogueStartResult start_dialogue(ObjectId initiator,uint32_t port) {
        if(!active_object_registry)return {{},DialogueResult::InvalidOwner};
        ObjectDispatchScope dispatch(*active_object_registry);
        if(starting_||dialogue::session().active())return {{},DialogueResult::Busy};
        last_=DialogueResult::NoSession;
        const auto result=request(initiator,port);
        if(result!=InteractionResult::Accepted) {
            const auto reason=result==InteractionResult::Busy?DialogueResult::Busy:
                result==InteractionResult::Unavailable?DialogueResult::Unavailable:
                result==InteractionResult::OutOfRange?DialogueResult::OutOfRange:
                result==InteractionResult::DomainMismatch?DialogueResult::DomainMismatch:
                result==InteractionResult::MissingBinding?DialogueResult::InvalidBinding:DialogueResult::InvalidOwner;
            last_=reason;
            return {{},reason};
        }
        return {{last_==DialogueResult::Accepted||last_==DialogueResult::Completed?dialogue::session().snapshot().generation:0},last_};
    }
    void interaction_started(ObjectId initiator,uint32_t port) override {
        InteractableComponent::interaction_started(initiator,port);
        if(starting_||!is_busy())return;
        starting_=true;notified_=false;
        DialogueResult result=DialogueResult::InvalidData;
        const auto* resource=dialogue::presenter_resource(presenter,conversation,style);
        if(dialogue::session().active())result=DialogueResult::Busy;
        else if(resource&&resource->table&&resource->presentation) {
            ObjectDispatchScope scope(*active_object_registry);
            auto* binding=active_object_registry->resolve<DialogueRoleBindingsComponent>(bindings);
            auto* binding_owner=binding?binding->get_owner():nullptr;
            if((bindings.valid()&&(!binding||!interaction::live(bindings)||!binding_owner||binding_owner->level_id()!=get_owner()->level_id()))||(resource->table->role_count&&!binding))result=DialogueResult::InvalidBinding;
            else if(dialogue::prepare_presenter(presenter,id(),*resource)&&interaction::live(id())&&interaction::live(initiator)) {
                prepared_presenter_=presenter;hold_open();
                dialogue::Assignment roles[8]{};uint16_t count=0;
                if(binding)for(unsigned i=0;i<8;++i){const auto role=binding->role_id(i);const auto receiver=binding->receiver(i);if(role||receiver.valid())roles[count++]={role,receiver};}
                dialogue::Start options;options.owner=id();options.initiator=initiator;options.presenter=presenter;options.port=port;
                options.roles=roles;options.role_count=count;options.timing=dialogue::presenter_timing(presenter,*resource);options.automatic_input=automatic_input;
                options.started=&started;options.entered=&entered;options.finished=&finished;
                result=dialogue::session().start(*resource->table,options);
            } else result=DialogueResult::MissingPresenter;
        }
        last_=result;
        if(result!=DialogueResult::Accepted&&!notified_)finish_attempt(initiator,port,result);
        starting_=false;
    }
};

class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="3a5fa04e-d6e9-59f7-80b1-666f5eaf94af") DialogueNPC3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("3a5fa04e-d6e9-59f7-80b1-666f5eaf94af");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Dialogue") DialogueInteractionComponent dialogue;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="9c3b4da8-a056-5016-b106-c58e1e057fb5") DialogueNPC2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("9c3b4da8-a056-5016-b106-c58e1e057fb5");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Dialogue") DialogueInteractionComponent dialogue;
};

// A single reusable UI composition per level. The editor places ordinary UI
// children and binds these component identities; projects can replace each one.
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=UI, Id="d40dc688-f7c0-50a3-9af8-7beaf1e3eea3") DialoguePresenter : public UIActor {
    uint64_t widget_layout_=0;
    friend const dialogue::Resource* dialogue::presenter_resource(ObjectId,uint64_t,uint64_t);
    friend void dialogue::bind_widget_layout(Actor&,uint64_t);
    const dialogue::Resource* resource_=nullptr;
    ObjectId source_{};
    uint32_t last_node_=0,last_page_=0,last_generation_=0;
    DialogueSnapshot state_{};
    bool opened_=false;
    bool registered() const{return id().get()==this;}
    static ActorData* label(ObjectId reference,uint64_t type,ObjectId level,bool active) {
        auto* object=reference.get();auto* actor=interaction::actor(reference);
        return object&&object->is_a(type)&&actor&&actor->level_id()==level&&(!active||interaction::live(reference))?actor->data():nullptr;
    }
    ActorData* text(ObjectId reference,bool active=true) const{return label(reference,TextComponent::static_class_id,level_id(),active);}
    ActorData* image(ObjectId reference,bool active=true) const{return label(reference,ImageComponent::static_class_id,level_id(),active);}
    void anchor_target(ObjectId value) {
#ifdef EPOK_UI_ANCHOR
        ui_anchor::set_target(anchor,*this,value);
#else
        (void)value;
#endif
    }
    void sound(int32_t clip){if(clip>=0){audio.set_enabled(true);audio.set_clip(clip);audio.play();}}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("d40dc688-f7c0-50a3-9af8-7beaf1e3eea3");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Canvas",Id="0762122d-cb6b-52f6-8fb1-3287494801fc") CanvasComponent canvas;
    EPOK_COMPONENT(Name="Audio",Id="b0e900ca-43e6-5bdf-bdbd-a52f0416b36b") AudioComponent audio;
    EPOK_PROPERTY(EditAnywhere, Id="75fc7c53-401d-54df-bc77-51f69ce4e15b") DialogueTiming default_timing{30,0};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::ImageComponent", Id="fb0a97ba-26a6-5e12-9f7a-312baecbce5b") ObjectId panel{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::ImageComponent", Id="83d56c12-f3dc-50ac-8753-d16bff5fbf56") ObjectId choice_panel{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="3a81bfbc-c49f-5066-9d04-582793f43f33") ObjectId body{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="d4189c64-a79f-5f0c-a50e-05c045244280") ObjectId speaker_name{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::ImageComponent", Id="b0fe3d0d-ba49-5663-846e-b86bb077c17e") ObjectId portrait{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="ba7920de-c81e-56b7-bddc-76e1b844ec21") ObjectId choice0{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="65547fdf-00d4-579d-808e-ec7ebae086d7") ObjectId choice1{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="661bafa1-85a3-50a7-8740-3a69014cfe35") ObjectId choice2{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="dddc9965-aab2-51b6-9a9b-587f459411c7") ObjectId choice3{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="f870bc78-bdde-4026-a2aa-008d7ad0e8bf") ObjectId prompt{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::TextComponent", Id="33137a53-a969-4423-81f9-860ea127f351") ObjectId continuation{};
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::ScreenAnchorComponent", Id="1cec1157-8892-5392-bbf0-6ddc7e9990d4") ObjectId anchor{};
    EPOK_FUNCTION(BlueprintEvent, Capability=dialogue, Id="1fa4077f-5f91-4937-9fe1-6db1e61aa24c") virtual void dialogue_opened(DialogueSnapshot state){(void)state;}
    EPOK_FUNCTION(BlueprintEvent, Capability=dialogue, Id="1267adf5-fd83-49b6-9c89-fe32c803e146") virtual void dialogue_closed(DialogueSnapshot state){(void)state;}
    EPOK_FUNCTION(BlueprintEvent, BlueprintPure, Capability=dialogue, Id="cb0d4973-882d-5378-b965-7fffc7e5847d") virtual bool can_present(ObjectId source) const {
        auto* value=active_object_registry?active_object_registry->resolve<DialogueInteractionComponent>(source):nullptr;
        const auto* resource=value?dialogue::resource(value->conversation,value->style,widget_layout_):nullptr;
        if(!registered()||!resource||!resource->table||!resource->presentation||!text(body))return false;
        if(!value->style&&(default_timing.reveal_ms>1000||default_timing.auto_advance_ms>3600000))return false;
        if(anchor.valid()) {
#ifdef EPOK_UI_ANCHOR
            if(!ui_anchor::valid_binding(anchor,*this))return false;
#else
            return false;
#endif
        }
        bool choices=false;
        for(unsigned i=0;i<resource->table->node_count;++i)choices|=resource->table->nodes[i].kind==dialogue::Kind::Choice;
        return !choices||(text(choice0)&&text(choice1)&&text(choice2)&&text(choice3));
    }
    EPOK_FUNCTION(BlueprintEvent, Capability=dialogue, Id="4ce39cc0-2020-5725-93f5-33c418934a60") virtual void present(DialogueSnapshot state) {
        if(!resource_||state.source!=source_||state.generation!=dialogue::session().snapshot().generation||dialogue::session().presenter()!=id())return;
        if(auto* actor=interaction::actor(state.speaker.valid()?state.speaker:state.source))anchor_target(actor->id());
        const auto& style=*resource_->presentation;
        const bool selecting=state.phase==DialoguePhase::Selecting;
        const bool prompting=state.choices!=0;
        auto* prompt_control=text(prompt);
        // The widget owns its hierarchy, geometry, fonts, colors and materials.
        // Binding updates content and transient dialogue state only.
        if(auto* data=image(panel))data->image.enabled=true;
        if(auto* data=image(choice_panel))data->image.enabled=selecting;
        if(auto* data=text(body)){data->text.enabled=!selecting&&(!prompting||!prompt.valid());data->text.key=-1;data->text.set_text(dialogue::session().text());data->text.visible_glyphs=uint16_t(state.revealed);}
        if(prompt_control){auto& text=prompt_control->text;text.key=-1;text.set_text(dialogue::session().text());text.enabled=prompting&&text.value[0];text.visible_glyphs=selecting?0xffff:uint16_t(state.revealed);}
        if(auto* data=text(continuation))data->text.enabled=state.phase==DialoguePhase::Waiting;
        const ObjectId choices[]={choice0,choice1,choice2,choice3};
        for(unsigned i=0;i<4;++i)if(auto* data=text(choices[i])) {
            data->text.enabled=selecting&&dialogue::session().choice_id(i);data->text.key=-1;
            data->text.set_text(dialogue::session().choice_text(i));data->text.visible_glyphs=0xffff;
            if(selecting&&state.selected%4==i)canvas.set_focused(entity_index(data));
        }
        if(!selecting)canvas.set_focused(-1);
        if(auto* data=text(speaker_name)){data->text.key=-1;data->text.set_text(dialogue::session().speaker_name());data->text.enabled=data->text.value[0];data->text.visible_glyphs=0xffff;}
        if(auto* data=image(portrait)){data->image.texture=dialogue::session().speaker_portrait();data->image.enabled=data->image.texture>=0;}
        if(last_generation_==state.generation&&(last_node_!=state.node||last_page_!=state.page))sound(style.advance_sound);
        last_generation_=state.generation;last_node_=state.node;last_page_=state.page;
    }
    void clear(bool closing=false,DialogueResult result=DialogueResult::NoSession) {
        if(!resource_)return;
        const auto* resource=resource_;const bool opened=opened_;
        auto ending=state_;const auto terminal=dialogue::snapshot_value();
        if(terminal.generation==ending.generation&&!dialogue::session().active()) {
            const auto source=ending.source,initiator=ending.initiator,presenter=ending.presenter,speaker=ending.speaker;
            ending=terminal;ending.source=source;ending.initiator=initiator;ending.presenter=presenter;ending.speaker=speaker;
            if(result==DialogueResult::NoSession)result=terminal.result;
        }
        if(result==DialogueResult::NoSession)result=DialogueResult::PresenterUnavailable;
        ending.phase=dialogue::terminal_phase(result);ending.result=result;ending.revealed=ending.glyphs=ending.pages=ending.choices=0;
        state_=ending;resource_=nullptr;source_={};last_generation_=0;opened_=false;
        if(!registered())return;
        anchor_target({});
        const ObjectId labels[]={body,prompt,continuation,speaker_name,choice0,choice1,choice2,choice3};
        // Inactive controls remain valid objects. Hide them too so re-enabling a
        // child or the widget cannot revive the previous session's text.
        for(auto id:labels)if(auto* data=text(id,false))data->text.enabled=false;
        for(auto id:{panel,choice_panel,portrait})if(auto* data=image(id,false))data->image.enabled=false;
        canvas.set_focused(-1);
        if(closing&&opened){
            sound(resource->presentation->close_sound);
            ObjectDispatchScope scope(*active_object_registry);dialogue_closed(ending);
        }
    }
    bool prepare(ObjectId source,const dialogue::Resource& resource) {
        if(!interaction::live(id())||!can_present(source)||!interaction::live(source)||!interaction::live(id()))return false;
        clear();resource_=&resource;source_=source;return true;
    }
    bool owns(ObjectId source) const{return resource_&&source_==source;}
    void begin(ObjectId source){if(owns(source)&&!opened_){opened_=true;state_=dialogue::snapshot_value();if(auto* actor=interaction::actor(source))anchor_target(actor->id());sound(resource_->presentation->open_sound);dialogue_opened(state_);}}
    void on_frame(uint32_t elapsed) override {
        UIActor::on_frame(elapsed);
        if(!registered()){clear();return;}
        if(dialogue::session().active()&&dialogue::session().presenter()==id()&&source_!=dialogue::session().source()){dialogue::session().invalidate_presenter(id());clear(true);return;}
        const auto handle=DialogueSessionLibrary::owner_session(source_);
        if(!handle.generation||dialogue::session().presenter()!=id()){clear(true);return;}
        if(!can_present(source_)||!interaction::live(id())){dialogue::session().invalidate_presenter(id());clear(true);return;}
        state_=DialogueSessionLibrary::snapshot(handle);
        if(resource_)present(state_);
    }
    void on_disable() override {if(registered())dialogue::session().invalidate_presenter(id());clear(true,DialogueResult::PresenterUnavailable);UIActor::on_disable();}
    void end_play(EndPlayReason reason) override {
        const auto result=reason==EndPlayReason::Destroyed?DialogueResult::PresenterUnavailable:DialogueResult::LevelUnloaded;
        if(registered()&&dialogue::session().presenter()==id()){
            if(result==DialogueResult::LevelUnloaded)dialogue::session().reset();else dialogue::session().invalidate_presenter(id());
        }
        clear(true,result);UIActor::end_play(reason);
    }
};
namespace dialogue {
inline const Resource* presenter_resource(ObjectId presenter,uint64_t conversation,uint64_t style) {
    auto* view=active_object_registry?active_object_registry->resolve<DialoguePresenter>(presenter):nullptr;
    return view?resource(conversation,style,view->widget_layout_):nullptr;
}
inline Timing presenter_timing(ObjectId presenter,const Resource& resource) {
    const auto* view=active_object_registry?active_object_registry->resolve<DialoguePresenter>(presenter):nullptr;
    return view&&!resource.style?Timing{uint16_t(view->default_timing.reveal_ms),view->default_timing.auto_advance_ms}:resource.presentation->timing;
}
inline void bind_widget_layout(Actor& presenter,uint64_t layout) {
    if(presenter.is_a(DialoguePresenter::static_class_id))static_cast<DialoguePresenter&>(presenter).widget_layout_=layout;
}
inline bool prepare_presenter(ObjectId presenter,ObjectId source,const Resource& resource) {
    auto* view=active_object_registry?active_object_registry->resolve<DialoguePresenter>(presenter):nullptr;
    auto* owner=interaction::actor(source);
    return view&&owner&&owner->level_id()==view->level_id()&&view->prepare(source,resource);
}
inline void begin_presenter(ObjectId presenter,ObjectId source) {
    auto* view=active_object_registry?active_object_registry->resolve<DialoguePresenter>(presenter):nullptr;
    if(view&&interaction::live(presenter))view->begin(source);
}
inline void release_presenter(ObjectId presenter,ObjectId source,DialogueResult result) {
    auto* view=active_object_registry?active_object_registry->resolve<DialoguePresenter>(presenter):nullptr;
    if(view&&view->owns(source))view->clear(true,result);
}
}
inline DialogueStartResult DialogueSessionLibrary::start(ObjectId source,ObjectId initiator,uint32_t port) {
    auto* value=active_object_registry?active_object_registry->resolve<DialogueInteractionComponent>(source):nullptr;
    return value?value->start_dialogue(initiator,port):DialogueStartResult{{},DialogueResult::InvalidOwner};
}
}
