#pragma once
#include "DungeonPlayerVitalsBase.hpp"

class EPOK_CLASS(Blueprintable,Id="070a2ac6-e00f-5e9c-8bd2-560acb25ad2f") DungeonPlayerVitals:public DungeonPlayerVitalsBase {
public:
    void start()override{
        const bool available=tuning.maximum>0&&tuning.hit_immunity>=epok::Fixed(0.0)&&tuning.hit_immunity<=epok::Fixed(60.0);
        assign(epok::HealthLibrary::make_state(tuning.maximum,tuning.maximum),0.0,available);
    }
    void advance(epok::Fixed dt)override{
        if(!valid||dt<=epok::Fixed(0.0)||dt>epok::Fixed(1.0)||immunity<=epok::Fixed(0.0))return;
        const auto next=immunity-dt;
        assign(health,next<epok::Fixed(0.0)?epok::Fixed(0.0):next,valid);
    }
    epok::HealthChange damage(int32_t amount,bool enabled)override{
        const bool accepted=valid&&enabled&&amount>0&&immunity<=epok::Fixed(0.0);
        const auto change=epok::HealthLibrary::damage(health,accepted?amount:0);
        if(change.applied)assign(change.state,tuning.hit_immunity,valid);
        return change;
    }
    epok::HealthChange heal(int32_t amount)override{
        const auto change=epok::HealthLibrary::heal(health,valid?amount:0);
        if(change.applied)assign(change.state,immunity,valid);
        return change;
    }
    bool can_heal()const override{
        return valid&&epok::HealthLibrary::valid(health)&&health.current<health.maximum;
    }
};
