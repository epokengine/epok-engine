#pragma once
#include "epok.hpp"

class EPOK_CLASS(Blueprintable,Id="2f9ae3a6-9a31-5fd8-b4be-d8b576c04a6e") SpellTarget:public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere,Id="15daaaf6-4dce-5f1c-9579-8093e5e05633") uint32_t health=100;
    EPOK_PROPERTY(EditAnywhere,Id="0975dbca-57ee-5f7c-98b7-3eb28f65da32") epok::EntityHandle status;
    EPOK_PROPERTY(Id="661f2aff-c7eb-5d0a-a14b-8106ceda6a1f") uint32_t hits=0;
    EPOK_FUNCTION(BlueprintCallable,Id="744d3067-c444-5f92-bc82-370453e28402") void take_damage(uint32_t amount);
    void update(epok::Transform&,epok::Fixed)override{}
};

// Native input and HUD only. BP_Fireball owns playback, Impact damage, particle
// drain and cancellation through the existing Blueprint continuation contract.
class EPOK_CLASS(Blueprintable,Id="f105601e-3192-55ac-a4f4-eab9e00ede6f") SpellCombat:public epok::Behaviour {
    epok::effects::Handle playback;
    bool busy=false;
public:
    EPOK_PROPERTY(EditAnywhere,Id="717934b9-1edc-5070-bf56-86aa159207f6") epok::EntityHandle target;
    EPOK_PROPERTY(EditAnywhere,Id="d1a4152e-7a2a-5a33-987f-5c7aad636277") epok::EntityHandle status;
    EPOK_FUNCTION(BlueprintEvent,Id="828ccb17-2714-5e87-b632-7f3ebe7cac40") virtual void cast_spell(uint32_t power,epok::EntityHandle victim){}
    EPOK_FUNCTION(BlueprintEvent,Id="5faf5559-3735-563d-82d5-d8ac2b7deb20") virtual void cancel_spell(){}
    EPOK_FUNCTION(BlueprintCallable,Id="b89ccdb9-e181-5213-b361-3224c1d98fa8") void remember_effect(epok::effects::Handle value);
    EPOK_FUNCTION(BlueprintPure,Id="3b3e1bbe-87e9-561e-83ea-7eabb949b4ba") epok::effects::Handle current_effect()const{return playback;}
    EPOK_FUNCTION(BlueprintCallable,Id="c9829781-275b-5926-9d43-4ce9d590575b") void finished();
    EPOK_FUNCTION(BlueprintCallable,Id="7cd708ba-b328-5be3-b9f1-4ddfd5339983") void cancelled();
    void start(epok::Transform&)override;
    void update(epok::Transform&,epok::Fixed)override;
    void frame_update(epok::Transform&,uint32_t)override;
};
