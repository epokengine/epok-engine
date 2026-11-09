#pragma once
// Cooked conversation execution. Tables and receiver dispatch are supplied by the
// host compiler; one bounded session owns the current modal control capture.
#define EPOK_INCLUDE_FROM_DIALOGUE 1
#include "interaction.hpp"
#undef EPOK_INCLUDE_FROM_DIALOGUE

namespace epok {
enum class DialoguePhase : uint8_t { Idle,Revealing,Waiting,Selecting,Executing,Completed,Cancelled,Failed };
enum class DialogueResult : uint8_t {
    Accepted,Busy,InvalidOwner,InvalidBinding,InvalidData,MissingPresenter,
    CaptureExhausted,NoSession,Completed,UserCancelled,SourceUnavailable,
    PresenterUnavailable,LevelUnloaded,TransitionBudget,NoChoices,CaptureLost,GenerationExhausted,
    Unavailable,OutOfRange,DomainMismatch
};
}
namespace epok::dialogue {
using Phase=DialoguePhase;
using Result=DialogueResult;
inline Phase terminal_phase(Result result) {
    if(result==Result::Completed)return Phase::Completed;
    return result==Result::UserCancelled||result==Result::SourceUnavailable||result==Result::PresenterUnavailable||result==Result::LevelUnloaded||result==Result::CaptureLost?Phase::Cancelled:Phase::Failed;
}
inline constexpr uint16_t missing=0xffffu;
inline constexpr uint8_t no_role=0xffu;
inline constexpr unsigned max_roles=8,max_options=64,visible_options=4,max_transitions=32;
enum class Kind : uint8_t { Line,Choice,Branch,Event,End };
struct Page {const char* text=nullptr;uint16_t glyphs=0;};
struct Text {
    const Page* pages=nullptr;
    // language-major page ranges; language_count + 1 offsets.
    const uint16_t* language_offsets=nullptr;
    uint16_t page_count=0,language_count=0;
};
struct Role {uint32_t id=0;uint16_t name=missing;int16_t portrait=-1;};
struct Assignment {uint32_t role=0;ObjectId receiver{};};
using Condition=bool(*)(ObjectId,ObjectId);
using Action=void(*)(ObjectId,ObjectId,uint32_t);
struct Binding {
    uint8_t role=no_role;
    uint64_t receiver_class=0;
    Condition condition=nullptr;
    Action action=nullptr;
};
struct Option {uint32_t id=0;uint16_t text=missing,next=missing,binding=missing;};
struct Node {
    uint32_t id=0;
    Kind kind=Kind::End;
    uint8_t speaker=no_role;
    uint16_t text=missing,next=missing,otherwise=missing,binding=missing;
    const Option* options=nullptr;
    uint16_t option_count=0;
    // Sentinels inherit style timing. Zero reveal is instant; zero auto is manual.
    uint16_t reveal_ms=missing;
    uint32_t auto_ms=0xffffffffu;
};
struct Conversation {
    uint64_t id=0;
    const Role* roles=nullptr;uint16_t role_count=0;
    const Binding* bindings=nullptr;uint16_t binding_count=0;
    const Text* texts=nullptr;uint16_t text_count=0;
    const Node* nodes=nullptr;uint16_t node_count=0,entry=0;
};
struct Timing {uint16_t reveal_ms=30;uint32_t auto_ms=0;};
// Session policy only. All visual data belongs to the widget controls.
struct Style {
    Timing timing{};
    int16_t open_sound=-1,advance_sound=-1,close_sound=-1;
};
struct Resource {uint64_t conversation=0,style=0;const Conversation* table=nullptr;const Style* presentation=nullptr;uint64_t widget=0;};
// Emitted once by the host cook beside scene data, as with object_classes.
extern const Resource resources[];
extern const size_t resource_count;
inline const Resource* resource(uint64_t conversation,uint64_t style=0,uint64_t widget=0) {
    for(size_t i=0;i<resource_count;++i)if(resources[i].conversation==conversation&&resources[i].style==style&&resources[i].widget==widget)return resources+i;
    return nullptr;
}
using Entered=void(*)(ObjectId,uint32_t);
// Finish notifications also identify the presenter. The source may already be
// disabled/destroyed; adapters revalidate it before invoking project callbacks.
using Finished=void(*)(ObjectId,ObjectId,ObjectId,uint32_t,Result);
using Started=void(*)(ObjectId,ObjectId,uint32_t,uint32_t);
struct Start {
    ObjectId owner{},initiator{},presenter{};
    uint32_t port=0;
    const Assignment* roles=nullptr;uint16_t role_count=0;
    Entered entered=nullptr;Finished finished=nullptr;
    Started started=nullptr;
    Timing timing{};
    bool automatic_input=true;
};
struct Snapshot {
    Phase phase=Phase::Idle;
    Result result=Result::NoSession;
    uint32_t generation=0,node=0,revealed=0,glyphs=0,page=0,pages=0;
    uint32_t choices=0,choice_page=0,selected=0,port=0;
    uint64_t conversation=0;
};

class Session {
    const Conversation* table_=nullptr;
    ObjectRegistry* registry_=nullptr;
    ObjectId owner_{},initiator_{},presenter_{},level_{};
    ObjectId roles_[max_roles]{};
    uint8_t available_[max_options]{};
    uint16_t node_=0,page_=0,choice_count_=0,selected_=0;
    uint16_t language_=0,reveal_ms_=0;
    uint32_t elapsed_=0,auto_ms_=0,generation_=0,microseconds_=0;
    int8_t nav_=0;uint32_t nav_elapsed_=0;
    bool nav_repeating_=false,dispatching_=false,finishing_=false,automatic_input_=true;
    Timing timing_{};Snapshot state_{};
    InputCapture capture_{};
    Entered entered_=nullptr;Finished finished_=nullptr;

    static bool running(Phase phase){return phase==Phase::Revealing||phase==Phase::Waiting||phase==Phase::Selecting||phase==Phase::Executing;}
    bool current(uint32_t generation) const{return active()&&generation_==generation;}
    const Node* node() const{return table_&&node_<table_->node_count?table_->nodes+node_:nullptr;}
    const Page* page_for(uint16_t text,uint16_t page,uint16_t* count=nullptr) const {
        if(!table_||text>=table_->text_count)return nullptr;
        const auto& value=table_->texts[text];
        if(!value.pages||!value.language_offsets||!value.language_count)return nullptr;
        const unsigned language=language_<value.language_count?language_:0;
        const auto first=value.language_offsets[language],end=value.language_offsets[language+1];
        if(end<=first||end>value.page_count)return nullptr;
        if(count)*count=uint16_t(end-first);
        return page<end-first?value.pages+first+page:nullptr;
    }
    void consume_edges() {
        const bool context=GameplayInputLibrary::valid(capture_);
        for(unsigned action=0;action<unsigned(GameplayAction::Count);++action) {
            if(context)GameplayInputLibrary::context_take_pressed(capture_,GameplayAction(action),true);
            else GameplayInputLibrary::take_frame_pressed(GameplayAction(action),state_.port);
        }
    }
    void finish(Result result) {
        if(!active()||finishing_)return;
        const auto owner=owner_,initiator=initiator_,presenter=presenter_;const auto port=state_.port;const auto callback=finished_;
        const bool notify=active_object_registry==registry_;
        finishing_=true;
        consume_edges();GameplayInputLibrary::release(capture_);capture_={};
        state_.phase=terminal_phase(result);
        state_.result=result;state_.revealed=state_.glyphs=state_.pages=state_.choices=0;
        // No query or callback can retain a pointer into a bank being unloaded.
        table_=nullptr;owner_={};initiator_={};presenter_={};level_={};registry_=nullptr;
        for(auto& role:roles_)role={};
        entered_=nullptr;finished_=nullptr;choice_count_=0;
        if(notify&&callback){ObjectDispatchScope scope(*active_object_registry);callback(owner,initiator,presenter,port,result);}
        finishing_=false;
    }
    bool lifetime() {
        if(!active())return false;
        if(active_object_registry!=registry_){finish(Result::LevelUnloaded);return false;}
        const auto* level=registry_->resolve<Level>(level_);
        if(!level||level->state()==ObjectState::EndingPlay||level->state()==ObjectState::Destroyed){finish(Result::LevelUnloaded);return false;}
        if(!interaction::live(owner_)||!interaction::live(initiator_)){finish(Result::SourceUnavailable);return false;}
        const auto* source=registry_->resolve<InteractableComponent>(owner_);
        if(source&&!source->enabled){finish(Result::SourceUnavailable);return false;}
        if(!interaction::live(presenter_)){finish(Result::PresenterUnavailable);return false;}
        for(unsigned i=0;table_&&i<table_->role_count;++i)if(!interaction::live(roles_[i])){finish(Result::SourceUnavailable);return false;}
        if(!GameplayInputLibrary::valid(capture_)){finish(Result::CaptureLost);return false;}
        return true;
    }
    bool condition(uint16_t index,bool& value) {
        if(!table_||index>=table_->binding_count){finish(Result::InvalidBinding);return false;}
        const auto binding=table_->bindings[index];
        if(binding.role>=table_->role_count||!binding.condition){finish(Result::InvalidBinding);return false;}
        const auto receiver=roles_[binding.role];
        if(!receiver.get()||!receiver.get()->is_a(binding.receiver_class)){finish(Result::InvalidBinding);return false;}
        const auto generation=generation_;
        value=binding.condition(receiver,initiator_);
        return current(generation)&&lifetime();
    }
    bool action(uint16_t index) {
        if(!table_||index>=table_->binding_count){finish(Result::InvalidBinding);return false;}
        const auto binding=table_->bindings[index];
        if(binding.role>=table_->role_count||!binding.action){finish(Result::InvalidBinding);return false;}
        const auto receiver=roles_[binding.role];
        if(!receiver.get()||!receiver.get()->is_a(binding.receiver_class)){finish(Result::InvalidBinding);return false;}
        const auto generation=generation_;
        binding.action(receiver,initiator_,state_.port);
        return current(generation)&&lifetime();
    }
    bool reveal_page() {
        const auto* value=node();uint16_t count=0;
        const auto* page=value?page_for(value->text,page_,&count):nullptr;
        if(!page||!page->text){finish(Result::InvalidData);return false;}
        reveal_ms_=value->reveal_ms==missing?timing_.reveal_ms:value->reveal_ms;
        auto_ms_=value->auto_ms==0xffffffffu?timing_.auto_ms:value->auto_ms;
        state_.page=page_;state_.pages=count;state_.glyphs=page->glyphs;
        state_.revealed=reveal_ms_?0:page->glyphs;elapsed_=0;
        state_.phase=state_.revealed<state_.glyphs?Phase::Revealing:Phase::Waiting;
        return true;
    }
    bool choices(const Node& value) {
        if(!value.options||!value.option_count||value.option_count>max_options){finish(Result::InvalidData);return false;}
        choice_count_=0;
        for(unsigned i=0;i<value.option_count;++i) {
            bool allowed=true;
            if(value.options[i].binding!=missing&&!condition(value.options[i].binding,allowed))return false;
            if(allowed)available_[choice_count_++]=uint8_t(i);
        }
        if(!choice_count_){finish(Result::NoChoices);return false;}
        if(selected_>=choice_count_)selected_=uint16_t(choice_count_-1);
        state_.choices=choice_count_;state_.selected=selected_;state_.choice_page=selected_/visible_options;
        return true;
    }
    void walk(bool notify=true) {
        for(unsigned transitions=0;transitions<max_transitions&&lifetime();++transitions) {
            const auto* value=node();if(!value){finish(Result::InvalidData);return;}
            state_.phase=Phase::Executing;state_.node=value->id;state_.choices=0;
            const auto generation=generation_;
            if(notify&&entered_)entered_(owner_,value->id);
            notify=true;
            if(!current(generation)||!lifetime())return;
            switch(value->kind) {
                case Kind::Line:page_=0;reveal_page();return;
                case Kind::Choice:
                    selected_=0;if(!choices(*value))return;
                    page_=0;
                    if(value->text!=missing){reveal_page();return;}
                    state_.pages=state_.glyphs=state_.revealed=0;state_.phase=Phase::Selecting;return;
                case Kind::Branch:{bool yes=false;if(!condition(value->binding,yes))return;node_=yes?value->next:value->otherwise;break;}
                case Kind::Event:if(!action(value->binding))return;node_=value->next;break;
                case Kind::End:finish(Result::Completed);return;
                default:finish(Result::InvalidData);return;
            }
        }
        if(active())finish(Result::TransitionBudget);
    }
    Result advance_inner() {
        const auto* value=node();if(!value){finish(Result::InvalidData);return Result::InvalidData;}
        if(state_.phase==Phase::Revealing){state_.revealed=state_.glyphs;state_.phase=Phase::Waiting;elapsed_=0;return Result::Accepted;}
        if(state_.phase==Phase::Selecting) {
            if(!choice_count_||selected_>=choice_count_){finish(Result::NoChoices);return Result::NoChoices;}
            const auto option=value->options[available_[selected_]];
            bool allowed=true;
            if(option.binding!=missing&&!condition(option.binding,allowed))return state_.result;
            if(!allowed){choices(*value);return active()?Result::Accepted:state_.result;}
            node_=option.next;walk();return active()?Result::Accepted:state_.result;
        }
        if(state_.phase!=Phase::Waiting)return Result::Busy;
        if(uint32_t(page_)+1<state_.pages){++page_;reveal_page();return Result::Accepted;}
        if(value->kind==Kind::Choice){state_.phase=Phase::Selecting;return Result::Accepted;}
        node_=value->next;walk();return active()?Result::Accepted:state_.result;
    }
public:
    bool active() const{return table_&&running(state_.phase);}
    Snapshot snapshot() const{return state_;}
    ObjectId source() const{return active()?owner_:ObjectId{};}
    ObjectId initiator() const{return active()?initiator_:ObjectId{};}
    ObjectId presenter() const{return active()?presenter_:ObjectId{};}
    ObjectId speaker() const {const auto* value=node();return value&&value->speaker<table_->role_count?roles_[value->speaker]:ObjectId{};}
    const char* speaker_name() const {
        const auto* value=node();
        if(!value||value->speaker>=table_->role_count)return "";
        const auto* name=page_for(table_->roles[value->speaker].name,0);
        return name&&name->text?name->text:"";
    }
    int32_t speaker_portrait() const {const auto* value=node();return value&&value->speaker<table_->role_count?table_->roles[value->speaker].portrait:-1;}
    const char* text() const {const auto* value=node();const auto* page=value?page_for(value->text,page_):nullptr;return page&&page->text?page->text:"";}
    uint32_t choice_id(unsigned visible) const {
        const auto* value=node();const unsigned index=state_.choice_page*visible_options+visible;
        return value&&state_.phase==Phase::Selecting&&visible<visible_options&&index<choice_count_?value->options[available_[index]].id:0;
    }
    const char* choice_text(unsigned visible) const {
        const auto* value=node();const unsigned index=state_.choice_page*visible_options+visible;
        if(!value||state_.phase!=Phase::Selecting||visible>=visible_options||index>=choice_count_)return "";
        const auto* page=page_for(value->options[available_[index]].text,0);
        return page&&page->text?page->text:"";
    }
    Result start(const Conversation& table,const Start& options) {
        if(active()||dispatching_||finishing_)return Result::Busy;
        if(scene_loading())return Result::Busy;
        if(generation_==0xffffffffu)return Result::GenerationExhausted;
        if(!active_object_registry||options.port>=4||!interaction::live(options.owner)||!interaction::live(options.initiator))return Result::InvalidOwner;
        auto* owner=interaction::actor(options.owner);auto* initiator=interaction::actor(options.initiator);
        auto* presenter=interaction::actor(options.presenter);
        if(!owner||!initiator||owner->level_id()!=initiator->level_id())return Result::InvalidOwner;
        if(!presenter||!interaction::live(options.presenter)||interaction::domain(presenter)!=ObjectDomain::UI||presenter->level_id()!=owner->level_id())return Result::MissingPresenter;
        if(!table.id||!table.nodes||!table.node_count||table.node_count>1024||table.entry>=table.node_count||table.role_count>max_roles||table.binding_count>64||
           (table.role_count&&!table.roles)||(table.binding_count&&!table.bindings)||(table.text_count&&!table.texts))return Result::InvalidData;
        if(options.role_count!=table.role_count||(options.role_count&&!options.roles))return Result::InvalidBinding;
        ObjectId resolved[max_roles]{};
        for(unsigned i=0;i<table.role_count;++i) {
            if(!table.roles[i].id)return Result::InvalidData;
            for(unsigned j=0;j<i;++j)if(table.roles[j].id==table.roles[i].id)return Result::InvalidData;
            unsigned matches=0;
            for(unsigned j=0;j<options.role_count;++j)if(options.roles[j].role==table.roles[i].id){resolved[i]=options.roles[j].receiver;++matches;}
            auto* receiver=interaction::actor(resolved[i]);
            if(matches!=1||!interaction::live(resolved[i])||!receiver||receiver->level_id()!=owner->level_id())return Result::InvalidBinding;
        }
        for(unsigned i=0;i<table.binding_count;++i) {
            const auto& binding=table.bindings[i];
            if(binding.role>=table.role_count||!binding.receiver_class||bool(binding.condition)==bool(binding.action))return Result::InvalidData;
            const auto* receiver=resolved[binding.role].get();
            if(!receiver||!receiver->is_a(binding.receiver_class))return Result::InvalidBinding;
        }
        const auto token=GameplayInputLibrary::capture(options.owner,GameplayInputLibrary::all_actions(),100,options.port);
        if(!GameplayInputLibrary::valid(token))return Result::CaptureExhausted;
        ObjectDispatchScope scope(*active_object_registry);
        registry_=active_object_registry;owner_=options.owner;initiator_=options.initiator;presenter_=options.presenter;level_=owner->level_id();
        table_=&table;capture_=token;timing_=options.timing;entered_=options.entered;finished_=options.finished;automatic_input_=options.automatic_input;
        for(unsigned i=0;i<max_roles;++i)roles_[i]=resolved[i];
        ++generation_;
        state_={};state_.generation=generation_;state_.conversation=table.id;state_.port=options.port;
        state_.result=Result::Accepted;state_.phase=Phase::Executing;node_=table.entry;language_=localization.language;
        microseconds_=0;
        nav_=0;nav_elapsed_=0;nav_repeating_=false;dispatching_=true;
        consume_edges();
        if(options.started)options.started(owner_,initiator_,state_.port,generation_);
        if(active()&&lifetime())walk();dispatching_=false;
        return active()?Result::Accepted:state_.result;
    }
    Result advance() {
        if(dispatching_||finishing_)return Result::Busy;
        if(!lifetime())return Result::NoSession;
        ObjectDispatchScope scope(*active_object_registry);dispatching_=true;
        consume_edges();const auto result=advance_inner();dispatching_=false;return result;
    }
    Result cancel() {
        if(dispatching_||finishing_)return Result::Busy;
        if(!lifetime())return Result::NoSession;
        ObjectDispatchScope scope(*active_object_registry);finish(Result::UserCancelled);return Result::Accepted;
    }
    Result move_selection(int32_t delta) {
        if(dispatching_||finishing_)return Result::Busy;
        if(!lifetime())return Result::NoSession;
        if(state_.phase!=Phase::Selecting||!choice_count_)return Result::Busy;
        const int64_t count=choice_count_;int64_t next=(int64_t(selected_)+delta)%count;if(next<0)next+=count;
        selected_=uint16_t(next);state_.selected=selected_;state_.choice_page=selected_/visible_options;return Result::Accepted;
    }
    void prune(){if(active())lifetime();}
    void invalidate_presenter(ObjectId presenter){if(active()&&presenter_==presenter)finish(Result::PresenterUnavailable);}
    void reset(){if(active())finish(Result::LevelUnloaded);if(!dispatching_&&!finishing_){state_={};table_=nullptr;}}
    void update_microseconds(uint32_t elapsed) {
        if(!active()||dispatching_||finishing_)return;
        const uint64_t total=uint64_t(microseconds_)+elapsed;
        microseconds_=uint32_t(total%1000);update(uint32_t(total/1000));
    }
    void update(uint32_t milliseconds) {
        if(dispatching_||finishing_||!lifetime())return;
        ObjectDispatchScope scope(*active_object_registry);
        if(language_!=localization.language) {
            language_=localization.language;
            if(state_.phase==Phase::Revealing||state_.phase==Phase::Waiting) {
                uint16_t count=0;const auto* value=node();
                if(!value||!page_for(value->text,0,&count)){finish(Result::InvalidData);return;}
                if(page_>=count)page_=uint16_t(count-1);reveal_page();
            }
        }
        bool confirm=false,cancelled=false;
        if(automatic_input_) {
            cancelled=GameplayInputLibrary::context_take_pressed(capture_,GameplayAction::Cancel,true);
            confirm=GameplayInputLibrary::context_take_pressed(capture_,GameplayAction::Confirm,true);
        }
        if(cancelled){cancel();return;}
        if(confirm){advance();return;}
        if(state_.phase==Phase::Revealing) {
            const uint64_t elapsed=uint64_t(elapsed_)+milliseconds;
            const uint64_t added=reveal_ms_?elapsed/reveal_ms_:state_.glyphs;
            const uint64_t revealed=uint64_t(state_.revealed)+added;
            state_.revealed=uint32_t(revealed>state_.glyphs?state_.glyphs:revealed);
            elapsed_=reveal_ms_?uint32_t(elapsed%reveal_ms_):0;
            if(state_.revealed==state_.glyphs){state_.phase=Phase::Waiting;elapsed_=0;}
        } else if(state_.phase==Phase::Waiting&&auto_ms_) {
            const uint64_t elapsed=uint64_t(elapsed_)+milliseconds;elapsed_=uint32_t(elapsed>0xffffffffu?0xffffffffu:elapsed);
            if(elapsed_>=auto_ms_)advance();
        } else if(state_.phase==Phase::Selecting&&automatic_input_) {
            const auto direction=GameplayInputLibrary::context_sample(capture_,GameplayAction::MoveY,true).value.raw();
            const int8_t nav=direction>2048?-1:direction<-2048?1:0;
            if(!nav){nav_=0;nav_elapsed_=0;nav_repeating_=false;}
            else if(nav!=nav_){nav_=nav;nav_elapsed_=0;nav_repeating_=false;move_selection(nav);}
            else {
                const uint64_t elapsed=uint64_t(nav_elapsed_)+milliseconds;nav_elapsed_=uint32_t(elapsed>1000?1000:elapsed);
                const unsigned delay=nav_repeating_?100:400;
                if(nav_elapsed_>=delay){nav_elapsed_=0;nav_repeating_=true;move_selection(nav);}
            }
        }
    }
};
inline Session& session(){static Session instance;return instance;}
inline void activation_changed(){session().prune();}
}
#include "dialogue_helpers.hpp"
