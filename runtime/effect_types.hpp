#pragma once
#include "epok.hpp"

namespace epok {
// Reflected through the same Clang catalog as behaviours. Effect layers are
// dedicated pool records, never Behaviour instances or scene Object slots.
class EPOK_CLASS(Id="27550d6d-5bba-4618-9d9e-33b9605bfb6c") EffectLayer {
public:
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="af33a70b-5855-4430-8cba-bc2b38119e86") bool enabled=true;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="37e43711-dba0-4bca-8499-952a73450d44") bool playing=true;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="775f36b0-2b5b-4920-aa9f-981b166507ef") Fixed opacity=1.0;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="dbd149e3-2164-4bd8-87bc-92f53bdc0f74") Fixed size=1.0;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="a8ab3325-89dd-416f-a946-082292c297c6") Fixed position[3]={0.0,0.0,0.0};
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="c06779d1-42a0-43ae-8712-fbac4a641bcf") Fixed color[3]={1.0,1.0,1.0};
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="286d0bc0-1d5e-4b47-974d-b87119cb63df") Fixed rate=8.0;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="41c3e62b-b9df-4df6-86b8-638e7851c4f3") Fixed velocity[3]={0.0,1.0,0.0};
    EPOK_FUNCTION(TimelineAction,Id="a5cd1cfd-e3a5-4b6e-96d0-c8c5bb5f43f5") void play(){enabled=true;playing=true;}
    EPOK_FUNCTION(TimelineAction,Id="52f8c306-ee5f-4a2b-957b-bca9d99d9ec3") void stop(){playing=false;}
    EPOK_FUNCTION(TimelineCallable,Id="c2650304-c940-408c-a8a8-05641c2a0489") void burst(uint32_t count){
        const uint32_t room=256-emitter.pending,accepted=count<room?count:room;
        emitter.pending+=uint16_t(accepted);
        const uint32_t dropped=count-accepted;
        dropped_requests=UINT32_MAX-dropped_requests<dropped?UINT32_MAX:dropped_requests+dropped;
    }
    Sprite sprite;
    ParticleEmitter emitter;
    uint32_t dropped_requests=0;
    bool runtime_active=true;
    bool runtime_visible=true;
};
// This type only exists at runtime. Authoring uses EffectLayerRef UUIDs.
struct EffectLayerHandle {uint16_t index=0xffff;uint32_t generation=0;};
}
