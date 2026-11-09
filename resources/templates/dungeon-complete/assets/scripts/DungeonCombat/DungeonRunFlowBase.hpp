#pragma once
#include "epok.hpp"
#include "DungeonRulesBinding.hpp"

// Shared project data/dispatch contract. Each authoring flavor supplies the
// transitions; this base deliberately contains no run-flow implementation.
struct EPOK_VALUE(Configuration,Id="a8bbbfe3-fa65-5375-b588-667722fac083") DungeonRunTiming {
    epok::Fixed music_lead=1.0,fade_in=1.0,control_delay=2.0;
    epok::Fixed death_hold=2.0,fade_out=1.0,restart_hold=2.0;
};

struct EPOK_VALUE(Configuration,Id="ccb46a9c-b22b-57b3-82f2-6d8c9b5f0cb0") DungeonRunSnapshot {
    uint32_t state=0,phase=0;
    epok::Fixed elapsed=0.0;
    uint32_t fade=255;
    epok::Fixed music_gain=1.0;
    bool playing=false,alive=true,restart=false,stop_music=false,valid=true;
};
class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="a0aa09c9-b6d3-5ae4-b6ab-dc99050463f4") DungeonRunFlowBase:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("a0aa09c9-b6d3-5ae4-b6ab-dc99050463f4");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,Id="202d836c-2271-52a2-8db0-8d36f4da6c3d") DungeonRunTiming timing{1.0,1.0,2.0,2.0,1.0,2.0};
    EPOK_PROPERTY(EditAnywhere,Id="8e8627cd-3e49-5557-94b6-3f60be28de73") epok::Fixed reactor_entry_z=53.0;
    EPOK_PROPERTY(Id="68467e3b-f7f7-5a05-a54c-102cd208a22d") DungeonRunSnapshot status{0,0,0.0,255,1.0,false,true,false,false,true};
    // A typed state write, shared by all providers. Decisions remain in the
    // derived project's graphs/source, while status is not an editable default.
    EPOK_FUNCTION(BlueprintCallable,Id="e3729b65-3558-5d55-aadb-b427a1e577b1") void assign_status(DungeonRunSnapshot value){status=value;}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="9b6a7d68-efe4-56ef-880b-867786725a48") virtual void start(){}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="6229c53c-6439-543a-8fe4-4f665d4ec86c") virtual void die(){}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="28a88de5-ef38-5c50-9a66-24305ddc75e0") virtual void win(){}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="b362cddc-ce9d-5a05-8e1c-748f10a9a701") virtual void advance(epok::Fixed dt,bool music_playing){(void)dt;(void)music_playing;}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="213c8b98-c8f2-500d-b590-dadac0609dd2") virtual bool try_enter_boss(bool has_key,epok::Fixed player_z){(void)has_key;(void)player_z;return false;}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="0f57f68c-d789-528f-8b81-c691149a1dbb") virtual bool restart_requested(bool pressed)const{(void)pressed;return false;}
};

// The binding caches type validation, not progression decisions. Every access
// still validates registry identity, the handle generation and live storage.
using DungeonRunFlowBinding=DungeonRulesBinding<DungeonRunFlowBase>;
