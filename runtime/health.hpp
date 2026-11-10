#pragma once
#include "epok.hpp"
#include <stdint.h>
// Allocation-free health values. Gameplay owns eligibility and consequences.
namespace epok {
struct EPOK_VALUE(Configuration,Id="2143303e-5890-5495-93b1-1d0b0b7e430b") HealthState {
    uint32_t maximum=0,current=0;
};
enum class HealthResult:uint8_t {Unchanged,Applied,InvalidState};
struct EPOK_VALUE(Id="4a383a62-69d6-5324-8dee-04b22fe3c141") HealthChange {
    HealthState state{};
    uint32_t applied=0;
    bool depleted=false,revived=false;
    HealthResult result=HealthResult::Unchanged;
};
// Only bounded arithmetic lives here. The caller owns damage eligibility,
// invulnerability, cooldowns, audiovisual feedback and death/reward decisions.
struct EPOK_FUNCTION_LIBRARY(Category="Combat",Capability=health,Id="a42d5289-8a70-510e-b702-f1021ac4027c") HealthLibrary {
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="df6acd25-a526-5605-86e3-1a73c31737b8") static bool valid(HealthState state){return state.current<=state.maximum;}
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="43903550-2746-5f1c-af57-ac221db6681c") static HealthState make_state(uint32_t maximum,uint32_t initial){return {maximum,initial<maximum?initial:maximum};}
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="c9fd8130-59a5-5513-8864-2ad5ee70c1af") static bool is_depleted(HealthState state){return valid(state)&&state.current==0;}
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="564672a4-0a4a-51e6-ad32-4d8f447b2da7") static HealthChange damage(HealthState state,int32_t amount){
        HealthChange change;change.state=state;
        if(!valid(state)){change.result=HealthResult::InvalidState;return change;}
        if(amount<=0||!state.current)return change;
        change.applied=uint32_t(amount)<state.current?uint32_t(amount):state.current;
        change.state.current-=change.applied;
        change.depleted=change.state.current==0;
        change.result=HealthResult::Applied;return change;
    }
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="adc1907c-df26-56b3-8d13-25015e4792f2") static HealthChange heal(HealthState state,int32_t amount){
        HealthChange change;change.state=state;
        if(!valid(state)){change.result=HealthResult::InvalidState;return change;}
        if(amount<=0||state.current==state.maximum)return change;
        const uint32_t room=state.maximum-state.current;
        change.applied=uint32_t(amount)<room?uint32_t(amount):room;
        change.state.current+=change.applied;
        change.revived=state.current==0;
        change.result=HealthResult::Applied;return change;
    }
};
}
