#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t platformer_dialogue_probe[8]={};}
class EPOK_CLASS(Blueprintable,Owners=World2D,Id="8b1c0661-2abc-510b-9ba5-c12af10cd6bc") PlatformerDialogueProbe:public epok::ActorComponent {
    uint32_t frame_=0,failures_=0,started_=0,masked_=0,released_=0,glyphs_=0;
    epok::DialogueHandle session_{};
public:
    static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character2D") epok::ObjectId player{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::InteractorComponent") epok::ObjectId interactor{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::DialogueInteractionComponent") epok::ObjectId dialogue{};
    void tick(epok::Fixed)override{
        if(frame_>=14)return;
        using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;using D=epok::DialogueSessionLibrary;
        auto* r=epok::active_object_registry;
        auto* character=r?r->resolve<epok::Character2D>(player):nullptr;
        auto* input=r?r->resolve<epok::InteractorComponent>(interactor):nullptr;
        auto* source=r?r->resolve<epok::DialogueInteractionComponent>(dialogue):nullptr;
        if(!character||!input||!source){failures_|=1;return;}
        if(frame_==10){
            input->refresh();
            if(input->focused_target()!=dialogue)failures_|=2;
            if(input->interact()!=epok::InteractionResult::Accepted)failures_|=4;
            session_=source->session();
            if(D::valid(session_)){++started_;glyphs_=D::snapshot(session_).glyphs;}else failures_|=8;
        }
        if(frame_==11){
            if(G::captured(A::Jump,0))++masked_;else failures_|=16;
        }
        if(frame_==12){
            if(character->motion().jumped)failures_|=32;
            if(!D::valid(session_))failures_|=64;
            D::cancel(session_);
            if(!D::valid(session_)&&!G::captured(A::Jump,0))++released_;else failures_|=128;
        }
        const uint16_t buttons=frame_==11?uint16_t(1u<<unsigned(epok::Button::Cross)):0;
        epok::input.sample(0,true,buttons);epok::gameplay_controls().begin_frame();
        epok::input.begin_tick();epok::gameplay_controls().begin_tick();
        if(frame_==13){auto* p=platformer_dialogue_probe;p[0]=PLATFORMER_DIALOGUE_MAGIC+EPOK_LUA_MODE;
            p[1]=14;p[2]=failures_;p[3]=started_;p[4]=masked_;p[5]=released_;p[6]=glyphs_;
            p[7]=epok::object_storage::hooks.statistics?epok::object_storage::hooks.statistics->rejected:0xffffffffu;}
        ++frame_;
    }
};
