void LegacyEconomy::activate(LegacyGame& game){
    if(menu==Pocket){if(selected==0)install();else if(selected==1)transfer();else load();return;}
    if(!loaded){if(selected==4||menu==Inventory&&selected==3)load();else message("Load your save first. Check memory card slot 1.");return;}
    pending=saved;heal_pending=0;ack_pending=false;
    if(menu==Shop){
        if(selected==4){load();return;}
        if(!selected){if(!sell(pending)){cue(DungeonSound::UiError);message("No asteroids to sell, or credit limit reached.");return;}commit("Asteroids sold for 10 credits each.");}
        else{
            if(!buy(pending,selected-1)){cue(DungeonSound::UiError);message(selected==3&&saved.buster==3?"Buster is already at maximum level.":"Not enough credits, or inventory is full.");return;}
            commit(selected==3?"Buster upgraded! Damage increased.":"Item purchased. Open inventory with TRIANGLE.");
        }
    }else if(menu==Inventory){
        if(selected==3){load();return;}
        if(selected==2){message("Buster modules are equipped automatically.");return;}
        auto& count=selected?pending.tanks:pending.medkits;
        if(!count){cue(DungeonSound::UiError);message("You do not have this item. Visit the trading PC.");return;}
        if(!game.can_heal_player()){cue(DungeonSound::UiError);message("Energy is full. Item kept in your inventory.");return;}
        --count;heal_pending=selected?100:35;commit(selected?"Energy tank used.":"Medkit used: +35 energy.");
    }
}
void LegacyEconomy::receive_transfer(){
    const auto& state=link.status;const auto* values=link.values;
    if(state.running_block!=link.block){message("Start Asteroid Catcher before transferring.");return;}
    if(uint32_t(values[2])!=magic||values[4]!=1){message("Wait for Asteroid Catcher, then try again.");return;}
    pending=saved;
    const int amount=old_inventory::transfer(pending,state.serial,uint32_t(values[0]),uint32_t(values[3]));
    if(amount<0){message(amount==-2?"Receipt list full. Use a registered PocketStation.":"Card counters do not match. Nothing transferred.");return;}
    ack_total=uint32_t(values[0]);ack_serial=state.serial;
    if(!amount){
        if(uint32_t(values[3])<ack_total){ack_pending=true;acknowledge();}
        else message("No new asteroids. Play the minigame to collect more.");
        return;
    }
    ack_pending=true;
    if(commit("Asteroids received and saved.")){++economy_telemetry.transfers;economy_telemetry.error=0;}
    else ack_pending=false;
}
