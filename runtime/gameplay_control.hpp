#pragma once
// Routed gameplay input. Raw Input remains independent. State and frame hooks
// are selected only by the gameplay-control cook capability.
#define EPOK_INCLUDE_FROM_GAMEPLAY_CONTROL 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_GAMEPLAY_CONTROL

namespace epok {
enum class GameplayAction : uint8_t {
    MoveX, MoveY, LookX, LookY, Jump, Interact, Confirm, Cancel, Pause,
    Attack, Aim, CycleTarget, Inventory, Sprint,
    Custom0, Custom1, Custom2, Custom3, Custom4, Custom5, Custom6, Custom7, Count
};
struct EPOK_VALUE(Id="69d4b10c-6e3e-5349-b0fe-162c43e3edce") InputCapture {
    uint32_t slot=0xffffffffu, generation=0;
};
struct EPOK_VALUE(Id="2f40cdc2-816a-5492-b7f4-3d880d295da8") GameplayActionSample {
    Fixed value=0.0;
    bool held=false, pressed=false, released=false, connected=false;
};

class GameplayControls {
    friend void gameplay_controls_activation_changed();
    static constexpr unsigned ports=4, action_count=unsigned(GameplayAction::Count), captures=16;
    struct Binding {
        uint16_t buttons=0;
        int8_t axis=-1, negative=-1, positive=-1;
    };
    struct Sources { uint16_t buttons=0; uint8_t axes=0; };
    struct Edges {
        uint16_t frame_pressed=0, frame_released=0, pending_pressed=0,
                 pending_released=0, tick_pressed=0, tick_released=0;
    };
    struct Capture {
        ObjectRegistry* registry=nullptr;
        ObjectId owner{}, level{};
        uint32_t actions=0, generation=0;
        int32_t priority=0;
        uint8_t port=0;
    };
    Input* m_input;
    Binding m_bindings[ports][action_count]{};
    Edges m_edges[ports]{};
    Capture m_captures[captures]{};
    uint32_t m_generation=0;
    uint16_t m_live=0;

    static constexpr uint16_t button_bit(unsigned button) {
        return button<16 ? uint16_t(1u<<button) : 0;
    }
    static constexpr Binding button_binding(Button button) {
        return {button_bit(unsigned(button)),-1,-1,-1};
    }
    constexpr void defaults() {
        for(auto& bindings:m_bindings) {
            for(auto& binding:bindings)binding={};
            bindings[0]={uint16_t(button_bit(unsigned(Button::Left))|button_bit(unsigned(Button::Right))),int8_t(Axis::LeftX),int8_t(Button::Left),int8_t(Button::Right)};
            bindings[1]={uint16_t(button_bit(unsigned(Button::Down))|button_bit(unsigned(Button::Up))),int8_t(Axis::LeftY),int8_t(Button::Down),int8_t(Button::Up)};
            bindings[2]={0,int8_t(Axis::RightX),-1,-1};
            bindings[3]={0,int8_t(Axis::RightY),-1,-1};
            const Button buttons[]={Button::Cross,Button::Cross,Button::Cross,Button::Circle,
                                    Button::Start,Button::Square,Button::L1,Button::R1,Button::Triangle,Button::L3};
            for(unsigned i=0;i<10;++i)bindings[i+4]=button_binding(buttons[i]);
        }
    }
    Sources sources(uint32_t actions,unsigned port) const {
        Sources result;
        for(unsigned a=0;a<action_count;++a)if(actions&(1u<<a)) {
            const auto& b=m_bindings[port][a];
            result.buttons|=b.buttons;
            if(b.axis>=0)result.axes|=uint8_t(1u<<unsigned(b.axis));
        }
        return result;
    }
    static bool intersects(Sources first,Sources second) {
        return (first.buttons&second.buttons)||(first.axes&second.axes);
    }
    static Actor* capture_actor(ObjectId owner) {
        auto* registry=active_object_registry;
        auto* object=registry?registry->get(owner):nullptr;
        if(!object||object->state()==ObjectState::EndingPlay||object->state()==ObjectState::Destroyed)return nullptr;
        if(auto* actor=registry->resolve<Actor>(owner))return actor;
        if(auto* component=registry->resolve<ActorComponent>(owner))return component->get_owner();
        return nullptr;
    }
    static bool available(ObjectId owner,ObjectId level) {
        auto* actor=capture_actor(owner);
        auto* scope=active_object_registry?active_object_registry->resolve<Level>(level):nullptr;
        return actor&&scope&&actor->level_id()==level&&actor->state()!=ObjectState::EndingPlay&&
               actor->state()!=ObjectState::Destroyed&&scope->actor_active(*actor);
    }
    void discard_sources(Sources affected,unsigned port) {
        auto& e=m_edges[port];
        const uint16_t keep=uint16_t(~affected.buttons);
        e.frame_pressed&=keep;e.frame_released&=keep;
        e.pending_pressed&=keep;e.pending_released&=keep;
        e.tick_pressed&=keep;e.tick_released&=keep;
    }
    void erase(unsigned slot) {
        auto& capture=m_captures[slot];
        discard_sources(owned_sources(capture),capture.port);
        capture={};--m_live;
    }
    void prune() {
        if(!m_live)return;
        for(unsigned i=0;i<captures;++i) {
            const auto& c=m_captures[i];
            if(c.generation&&(c.registry!=active_object_registry||!available(c.owner,c.level)))erase(i);
        }
    }
    const Capture* resolve(InputCapture token) const {
        if(token.slot>=captures||!token.generation)return nullptr;
        const auto& c=m_captures[token.slot];
        return c.generation==token.generation?&c:nullptr;
    }
    const Capture* winner(Sources requested,unsigned port) const {
        const Capture* result=nullptr;
        if(!m_live)return result;
        for(const auto& c:m_captures)if(c.generation&&c.port==port&&intersects(requested,sources(c.actions,port))) {
            if(!result||c.priority>result->priority||(c.priority==result->priority&&c.generation>result->generation))result=&c;
        }
        return result;
    }
    Sources owned_sources(const Capture& context) const {
        Sources result=sources(context.actions,context.port);
        for(unsigned b=0;b<16;++b)if((result.buttons&button_bit(b))&&winner({button_bit(b),0},context.port)!=&context)
            result.buttons&=uint16_t(~button_bit(b));
        return result;
    }
    uint16_t held_buttons(unsigned port) const {
        uint16_t result=0;
        for(unsigned b=0;b<16;++b)if(m_input->held(Button(b),port))result|=button_bit(b);
        return result;
    }
    GameplayActionSample read(GameplayAction action,unsigned port,bool frame,const Capture* context) const {
        const unsigned a=unsigned(action);
        if(a>=action_count||port>=ports)return {};
        GameplayActionSample result;result.connected=m_input->connected(port);
        const auto requested=sources(1u<<a,port);
        const auto* selected=winner(requested,port);
        if(selected!=context||(context&&!(context->actions&(1u<<a))))return result;
        const auto& b=m_bindings[port][a];
        const uint16_t held=held_buttons(port);
        int32_t raw=b.axis>=0?m_input->axis_raw(Axis(b.axis),port):(held&b.buttons?4096:0);
        if(b.negative>=0&&b.positive>=0) {
            const bool negative=(held&button_bit(unsigned(b.negative)))!=0;
            const bool positive=(held&button_bit(unsigned(b.positive)))!=0;
            if(negative||positive)raw=(int(positive)-int(negative))*4096;
        }
        result.value=Fixed(raw,Fixed::RAW);result.held=raw!=0;
        const auto& e=m_edges[port];
        result.pressed=((frame?e.frame_pressed:e.tick_pressed)&b.buttons)!=0;
        result.released=((frame?e.frame_released:e.tick_released)&b.buttons)!=0;
        return result;
    }
    bool take(GameplayAction action,unsigned port,bool frame,const Capture* context) {
        const auto value=read(action,port,frame,context);
        if(!value.pressed)return false;
        discard_sources(sources(mask(action),port),port);
        return true;
    }
public:
    constexpr explicit GameplayControls(Input& source=input):m_input(&source) {defaults();}
    static constexpr uint32_t mask(GameplayAction action) {
        return unsigned(action)<action_count?1u<<unsigned(action):0;
    }
    static constexpr uint32_t all_actions(){return (1u<<action_count)-1u;}
    bool captured(GameplayAction action,unsigned port){
        prune();
        return port<ports&&unsigned(action)<action_count&&winner(sources(mask(action),port),port)!=nullptr;
    }
    void reset() {
        for(auto& c:m_captures)c={};
        for(auto& e:m_edges)e={};
        m_live=0;defaults(); // Never reset token generations.
    }
    void begin_frame() {
        prune();
        for(unsigned p=0;p<ports;++p) {
            auto& e=m_edges[p];e.frame_pressed=e.frame_released=0;
            for(unsigned b=0;b<16;++b) {
                if(m_input->frame_pressed(Button(b),p))e.frame_pressed|=button_bit(b);
                if(m_input->frame_released(Button(b),p))e.frame_released|=button_bit(b);
            }
            e.pending_pressed|=e.frame_pressed;e.pending_released|=e.frame_released;
        }
    }
    void begin_tick() {
        prune();
        for(auto& e:m_edges) {
            e.tick_pressed=e.pending_pressed;e.tick_released=e.pending_released;
            e.pending_pressed=e.pending_released=0;
        }
    }
    void discard_tick_edges() {
        for(auto& e:m_edges)e.pending_pressed=e.pending_released=e.tick_pressed=e.tick_released=0;
    }
    void end_tick() {for(auto& e:m_edges)e.tick_pressed=e.tick_released=0;}
    InputCapture capture(ObjectId owner,uint32_t actions,int32_t priority,unsigned port) {
        prune();
        auto* actor=capture_actor(owner);
        if(port>=ports||!actions||(actions&~all_actions())||!actor||
           !available(owner,actor->level_id())||m_live==captures||m_generation==0xffffffffu)return {};
        for(unsigned i=0;i<captures;++i)if(!m_captures[i].generation) {
            auto& c=m_captures[i];c.registry=active_object_registry;c.owner=owner;
            c.level=actor->level_id();c.actions=actions;c.priority=priority;c.port=uint8_t(port);
            c.generation=++m_generation;++m_live;
            discard_sources(owned_sources(c),port);
            return {i,c.generation};
        }
        return {};
    }
    bool valid(InputCapture token) {prune();return resolve(token)!=nullptr;}
    InputCapture owner_capture(ObjectId owner,uint32_t actions,unsigned port) {
        prune();InputCapture result;
        if(port>=ports||!actions||(actions&~all_actions()))return result;
        for(unsigned i=0;i<captures;++i) {
            const auto& c=m_captures[i];
            if(c.generation&&c.owner==owner&&c.port==port&&c.actions==actions&&c.generation>result.generation)
                result={i,c.generation};
        }
        return result;
    }
    bool release(InputCapture token) {
        prune();if(!resolve(token))return false;erase(token.slot);return true;
    }
    bool bind_button(GameplayAction action,Button button,unsigned port) {
        if(port>=ports||!mask(action)||unsigned(button)>=16)return false;
        m_edges[port]={};m_bindings[port][unsigned(action)]=button_binding(button);return true;
    }
    bool bind_axis(GameplayAction action,Axis axis,Button negative,Button positive,unsigned port) {
        if(port>=ports||!mask(action)||unsigned(axis)>=4||unsigned(negative)>=16||unsigned(positive)>=16||negative==positive)return false;
        m_edges[port]={};m_bindings[port][unsigned(action)]={uint16_t(button_bit(unsigned(negative))|button_bit(unsigned(positive))),int8_t(axis),int8_t(negative),int8_t(positive)};
        return true;
    }
    GameplayActionSample sample(GameplayAction action,unsigned port,bool frame=false) {
        prune();return read(action,port,frame,nullptr);
    }
    GameplayActionSample context_sample(InputCapture token,GameplayAction action,bool frame) {
        prune();const auto* c=resolve(token);return c?read(action,c->port,frame,c):GameplayActionSample{};
    }
    bool take_pressed(GameplayAction action,unsigned port,bool frame=false) {
        prune();return take(action,port,frame,nullptr);
    }
    bool context_take_pressed(InputCapture token,GameplayAction action,bool frame) {
        prune();const auto* c=resolve(token);return c&&take(action,c->port,frame,c);
    }
    // Temporary physical-source adapters for the existing controller templates.
    bool button_held(Button button,unsigned port) {
        prune();const uint16_t bit=button_bit(unsigned(button));
        return port<ports&&bit&&!winner({bit,0},port)&&m_input->held(button,port);
    }
    bool button_pressed(Button button,unsigned port,bool frame=false) {
        prune();const uint16_t bit=button_bit(unsigned(button));
        return port<ports&&bit&&!winner({bit,0},port)&&((frame?m_edges[port].frame_pressed:m_edges[port].tick_pressed)&bit);
    }
    int32_t axis_raw(Axis axis,unsigned port) {
        prune();if(port>=ports||unsigned(axis)>=4||winner({0,uint8_t(1u<<unsigned(axis))},port))return 0;
        return m_input->axis_raw(axis,port);
    }
};

inline GameplayControls& gameplay_controls() {static constinit GameplayControls value;return value;}
// Observe the inactive state before a nested callback can enable the owner again.
inline void gameplay_controls_activation_changed(){gameplay_controls().prune();}

struct EPOK_FUNCTION_LIBRARY(Category="Gameplay Input", Capability=gameplay-control, Id="4ccee5f3-89cf-58d1-ab6b-2293318caf49") GameplayInputLibrary {
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="f145e05d-fdc1-5ab6-9e69-c06c66cd79af") static uint32_t mask(GameplayAction action){return GameplayControls::mask(action);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="be0ca548-7917-5ca4-8d9b-55f71a39246b") static uint32_t movement_actions(){return mask(GameplayAction::MoveX)|mask(GameplayAction::MoveY);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="081250d0-93e4-5512-9efa-89ab539b6e37") static uint32_t look_actions(){return mask(GameplayAction::LookX)|mask(GameplayAction::LookY);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="1ba01b3e-7d63-5123-aae6-d8b422cc0886") static uint32_t all_actions(){return GameplayControls::all_actions();}
    EPOK_FUNCTION(BlueprintCallable, ResourceAcquisition, Id="7ab66eb1-358e-5e15-8ea2-b93b5a4e7667") static InputCapture capture(ObjectId owner,uint32_t actions,int32_t priority,uint32_t port){return gameplay_controls().capture(owner,actions,priority,port);}
    EPOK_FUNCTION(BlueprintCallable, Id="447cf5d1-b270-533f-90a4-2a83830974a9") static bool release(InputCapture token){return gameplay_controls().release(token);}
    EPOK_FUNCTION(BlueprintPure, Id="b473bc23-7423-5664-a367-2ee27a9469b5") static bool valid(InputCapture token){return gameplay_controls().valid(token);}
    EPOK_FUNCTION(BlueprintPure, Id="3b7a1661-439f-57ad-a8bf-7c9067d1b472") static InputCapture owner_capture(ObjectId owner,uint32_t actions,uint32_t port){return gameplay_controls().owner_capture(owner,actions,port);}
    EPOK_FUNCTION(BlueprintPure, Id="4fb26875-f383-5dc8-ac56-65eb53b78319") static GameplayActionSample sample(GameplayAction action,uint32_t port){return gameplay_controls().sample(action,port);}
    EPOK_FUNCTION(BlueprintPure, Id="2f1f6db7-661b-5ae9-bf59-4f923e380050") static bool captured(GameplayAction action,uint32_t port){return gameplay_controls().captured(action,port);}
    EPOK_FUNCTION(BlueprintPure, Id="081aed88-9978-554d-ace4-17bfd5fbb3aa") static GameplayActionSample frame_sample(GameplayAction action,uint32_t port){return gameplay_controls().sample(action,port,true);}
    EPOK_FUNCTION(BlueprintPure, Id="71f0e8c8-5774-592c-be22-4bd8112fc3c8") static GameplayActionSample context_sample(InputCapture token,GameplayAction action,bool frame){return gameplay_controls().context_sample(token,action,frame);}
    EPOK_FUNCTION(BlueprintCallable, EventConsumption, Id="2d2065bb-6975-5aad-911b-1477cd80060d") static bool take_pressed(GameplayAction action,uint32_t port){return gameplay_controls().take_pressed(action,port);}
    EPOK_FUNCTION(BlueprintCallable, EventConsumption, Id="97c72dbb-73f4-52a1-83c8-dd6f7c20c8e8") static bool take_frame_pressed(GameplayAction action,uint32_t port){return gameplay_controls().take_pressed(action,port,true);}
    EPOK_FUNCTION(BlueprintCallable, EventConsumption, Id="571c9a21-7652-5180-8ed4-0defb633dc18") static bool context_take_pressed(InputCapture token,GameplayAction action,bool frame){return gameplay_controls().context_take_pressed(token,action,frame);}
    EPOK_FUNCTION(BlueprintCallable, Id="ba1fa132-dfc6-5a7d-9854-1cb2538a4357") static bool bind_button(GameplayAction action,Button button,uint32_t port){return gameplay_controls().bind_button(action,button,port);}
    EPOK_FUNCTION(BlueprintCallable, Id="145bb004-9d7a-59bc-a8d3-adade20eee99") static bool bind_axis(GameplayAction action,Axis axis,Button negative,Button positive,uint32_t port){return gameplay_controls().bind_axis(action,axis,negative,positive,port);}
    EPOK_FUNCTION(BlueprintPure, Id="26561bbe-17e7-5273-9fb8-3b9470cdfdba") static bool button_held(Button button,uint32_t port=0){return gameplay_controls().button_held(button,port);}
    EPOK_FUNCTION(BlueprintPure, Id="7161d176-d7c9-5216-92cf-e6f9a89b0360") static bool button_pressed(Button button,uint32_t port=0){return gameplay_controls().button_pressed(button,port);}
    EPOK_FUNCTION(BlueprintPure, Id="be003a37-8b9e-5548-865c-86f284b1b4a8") static bool button_frame_pressed(Button button,uint32_t port=0){return gameplay_controls().button_pressed(button,port,true);}
    EPOK_FUNCTION(BlueprintPure, Id="b32b08b3-db6f-5e0a-83fa-5576d2fad3ad") static int32_t axis_raw(Axis axis,uint32_t port=0){return gameplay_controls().axis_raw(axis,port);}
    EPOK_FUNCTION(BlueprintPure, Id="11f4139a-4bb3-5d08-976e-3a0e8ab660a5") static Fixed axis_value(Axis axis,uint32_t port){return Fixed(gameplay_controls().axis_raw(axis,port),Fixed::RAW);}
    EPOK_FUNCTION(BlueprintPure, Id="e0e243c1-bdd8-5930-95c6-c1e0ab449bbc") static GameplayActionSample axis(Axis axis,uint32_t port){
        GameplayActionSample result;
        if(port<4&&unsigned(axis)<4){result.connected=input.connected(port);result.value=axis_value(axis,port);result.held=result.value.raw()!=0;}
        return result;
    }
};
}
