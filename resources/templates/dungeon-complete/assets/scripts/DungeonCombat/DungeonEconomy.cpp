#include "DungeonEconomy.hpp"
#include "DungeonGame.hpp"
#include "DungeonProfiling.hpp"
using namespace epok;
EconomyTelemetry economy_telemetry{};
namespace {
void cue(DungeonGame& game,uint32_t id){
    switch(id){case 1:game.sound(DungeonSound::UiError);break;case 2:game.sound(DungeonSound::UiSuccess);break;
    case 3:game.sound(DungeonSound::UiMove);break;case 4:game.sound(DungeonSound::UiOpen);break;
    case 5:game.sound(DungeonSound::UiClose);break;case 6:game.sound(DungeonSound::UiConfirm);break;default:break;}
}
}
DungeonEconomyPresenterBase* DungeonEconomy::presenter()const{
    auto* value=active_object_registry?active_object_registry->resolve<DungeonEconomyPresenterBase>(presenter_):nullptr;
    return value&&is_active(value)?value:nullptr;
}
void DungeonEconomy::initialize(ObjectId policy){
    *this=DungeonEconomy{};policy_=policy;
    if(auto* value=rules()){
        presenter_=value->hud_owner;
        if(auto* data=gameplay_actor_data(value->pocket_station))pocket_=handle(data);
        if(auto* data=gameplay_actor_data(value->trading_station))computer_=handle(data);
    }
    if(auto* value=presenter())value->policy=policy;
}
void DungeonEconomy::hub_visible(bool visible){if(auto* data=pocket_.get())data->mesh=visible;if(auto* data=computer_.get())data->mesh=visible;}
void DungeonEconomy::publish_hud(DungeonHUDSnapshot value){if(auto* hud=presenter())hud->render_hud(value);}
void DungeonEconomy::publish_damage(DungeonHUDDamage value){if(auto* hud=presenter())hud->damage_hud(value);}
bool DungeonEconomy::input(DungeonGame& game,Fixed dt){
    DungeonMeasure measure(0);
    // The interface reads frame edges, so a catch-up step gives it nothing new.
    // It runs once per rendered frame and still receives every step's time.
    elapsed_+=dt;if(!frame_due_)return blocked_;
    frame_due_=false;const Fixed step=elapsed_;elapsed_=0.0;
    auto* view=presenter();auto* policy=rules();
    if(!view||!policy){blocked_=false;presenting_=false;set_world3d_visible(true);return false;}
    const uint64_t elapsed=step.raw()>0?uint64_t(uint32_t(step.raw()))*1000000u/4096u:0;
    const DungeonEconomyUIContext context{game.controls_enabled(),game.can_heal_player(),0,step};
    const auto result=view->frame(context,elapsed>5000000u?5000000u:uint32_t(elapsed));
    if(presenter()!=view||rules()!=policy){blocked_=false;presenting_=false;set_world3d_visible(true);return false;}
    blocked_=result.blocked;presenting_=!result.world_visible;set_world3d_visible(result.world_visible);
    if(result.heal)game.heal_player(result.heal>uint32_t(INT32_MAX)?INT32_MAX:int(result.heal));
    cue(game,result.cue);
    economy_telemetry.menu=result.menu;economy_telemetry.selection=result.selected;economy_telemetry.slot=view->screen.port;
    economy_telemetry.ore=policy->saved.ore;economy_telemetry.credits=policy->saved.credits;
    economy_telemetry.medkits=policy->saved.medkits;economy_telemetry.tanks=policy->saved.tanks;economy_telemetry.buster=policy->saved.buster;
    economy_telemetry.loaded=policy->loaded;economy_telemetry.installed=policy->pocket.installed;economy_telemetry.error=view->runtime.error;
    economy_telemetry.transfers=view->runtime.transfers;economy_telemetry.saves=view->runtime.saves;
    economy_telemetry.operation=policy->pocket.operation?policy->pocket.operation:view->screen.card_phase;
    return blocked_;
}
