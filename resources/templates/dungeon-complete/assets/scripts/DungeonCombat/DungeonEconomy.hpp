#pragma once
#include "epok.hpp"
#include "DungeonEconomyPolicyBase.hpp"
#include "DungeonEconomyPresenterBase.hpp"
class DungeonGame;
struct EconomyTelemetry {uint32_t menu=0,selection=0,slot=0,ore=0,credits=0,medkits=0,tanks=0,buster=0,transfers=0,installed=0,error=0,saves=0,loaded=0,operation=0;};
extern EconomyTelemetry economy_telemetry;

// Project integration bridge. The chosen UI class owns menus, capture, content,
// Card scheduling and transaction completion. This bridge copies world context
// and forwards the chosen outputs to existing rendering/audio/health sinks.
class DungeonEconomy {
public:
    void initialize(epok::ObjectId policy);
    bool input(DungeonGame&,epok::Fixed dt);
    bool blocking()const{return blocked_;}
    bool presenting()const{return presenting_;}
    void begin_frame(){frame_due_=true;}
    unsigned damage_bonus()const{auto* p=rules();return p?p->damage_bonus():0;}
    void hub_visible(bool visible);
    void publish_hud(DungeonHUDSnapshot);
    void publish_damage(DungeonHUDDamage);
private:
    epok::ObjectId policy_{},presenter_{};
    mutable DungeonEconomyPolicyBinding binding_;
    epok::DataHandle pocket_{},computer_{};
    epok::Fixed elapsed_=0.0;
    bool blocked_=false,presenting_=false,frame_due_=true;
    DungeonEconomyPolicyBase* rules()const{return binding_.resolve(policy_);}
    DungeonEconomyPresenterBase* presenter()const;
};
