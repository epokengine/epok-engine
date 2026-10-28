#include "DungeonEconomy.hpp"
#include "DungeonGame.hpp"
#include "DungeonProfiling.hpp"
#include "pocketstation_apps.hpp"
using namespace epok;
using namespace dungeon_inventory;
EconomyTelemetry economy_telemetry{};
namespace {
constexpr const char* app_name="BEEPOKPASTEROID";
constexpr const char* save_name="EPOK-DUNGEON";
#if __has_include("pocket-apps.hh")
// Resolve the authored name at compile time. Only this app's image needs to
// occupy PSX RAM, even when the project also holds the untouched starter demo.
constexpr PocketApp miner_image(){
    for(const auto& app:epok::pocket_apps){
        const char* a=app.filename;const char* b=app_name;
        while(*a&&*a==*b){++a;++b;}if(*a==*b)return app;
    }
    return {};
}
constexpr PocketApp miner=miner_image();
#else
constexpr PocketApp miner{};
#endif
struct Lines {
    char value[512]{};unsigned size=0;
    Lines& text(const char* s){while(s&&*s&&size<510)value[size++]=*s++;value[size]=0;return *this;}
    Lines& number(uint32_t v){char d[12];unsigned n=0;do{d[n++]=char('0'+v%10);v/=10;}while(v);while(n&&size<510)value[size++]=d[--n];value[size]=0;return *this;}
};
DataHandle named(const char* name){auto* d=find_actor_data(name);return d?handle(d):DataHandle{};}
void text(DataHandle h,const char* value){if(auto* d=h.get())d->text.set_text(value);}
void cue(DungeonSound event){if(auto* game=DungeonGame::current)game->sound(event);}
bool near(const ActorData* p,const ActorData* station){
    if(!p||!station)return false;
    const int64_t x=(p->transform.position[0]-station->transform.position[0]).raw(),z=(p->transform.position[2]-station->transform.position[2]).raw();
    const Fixed y=p->transform.position[1]-station->transform.position[1];
    return x*x+z*z<int64_t(10240)*10240&&y>Fixed(-2.6)&&y<Fixed(2.6);
}
}
void DungeonEconomy::initialize(){
    *this=DungeonEconomy{};
    player=named("Player Aqua");pocket=named("PocketStation Terminal");computer=named("Trading PC");
    panel=named("Message Panel");title=named("Objective");summary=named("Key Status");
    body=named("Game Message");status=named("Boss Label");footer=named("Controls Hint");prompt=footer;
    const char* names[]={"Energy Label","Energy Bar","Energy Panel","Key Status","Objective","Boss Label","Boss Bar","Message Panel","Game Message","Aim Reticle","Controls Hint"};
    for(unsigned i=0;i<11;++i)flags[i].data=named(names[i]);
    const DataHandle altered[]={title,summary,body,status,footer,panel};
    for(unsigned i=0;i<6;++i)views[i].data=altered[i];
    load();show();
}
void DungeonEconomy::message(const char* s){
    view_dirty=true;
    unsigned i=0,column=0;
    while(s&&*s&&i<sizeof(notice)-1){
        if(*s==' '){unsigned word=0;while(s[1+word]&&s[1+word]!=' ')++word;
            if(column+1+word>32){notice[i++]='\n';column=0;++s;continue;}}
        notice[i++]=*s++;++column;
    }
    notice[i]=0;
}
void DungeonEconomy::publish(){
    economy_telemetry.menu=menu;economy_telemetry.selection=selected;economy_telemetry.slot=port;
    economy_telemetry.ore=saved.ore;economy_telemetry.credits=saved.credits;economy_telemetry.medkits=saved.medkits;
    economy_telemetry.tanks=saved.tanks;economy_telemetry.buster=saved.buster;economy_telemetry.loaded=loaded;economy_telemetry.operation=operation;
}
void DungeonEconomy::load(){
    if(memory_card.busy()){message("Memory card busy. Select Load save to retry.");return;}
    if(!memory_card.read(save_name,0)){message(card_error_message(memory_card.status().last_rejection));return;}
    operation=Loading;request=memory_card.status().request;message("Loading inventory from memory card slot 1...");
}
bool DungeonEconomy::commit(const char* result){
    if(!loaded){message("Load your save first. Check memory card slot 1.");heal_pending=0;return false;}
    if(!valid(pending)){message("Invalid inventory data. Nothing changed.");heal_pending=0;return false;}
    if(!memory_card.write(save_name,"Dungeon - Asteroid Supply",&pending,sizeof(pending),0)){
        message(card_error_message(memory_card.status().last_rejection));heal_pending=0;return false;
    }
    request=memory_card.status().request;operation=Saving;success=result;message("Saving... Keep memory card slot 1 connected.");return true;
}
void DungeonEconomy::acknowledge(){
    link.start(DungeonPocketLink::Receipt,miner,port,ack_total,ack_serial);
    operation=Sending;ack_time=0.0;message("Materials saved. Syncing PocketStation receipt...");
}
void DungeonEconomy::poll(DungeonGame& game,Fixed dt){
    if(operation>=Installing){
        const auto previous=operation;
        if(previous==CheckingReceipt)ack_time+=dt;
        link.tick(uint32_t(dt.raw())*1000000u/4096u);
        if(!link.done()){
            if(previous==Installing&&installation_percent!=link.percent()){
                installation_percent=link.percent();Lines progress;progress.text("Installing Asteroid Catcher... ").number(installation_percent).text("%");message(progress.value);
            }
            return;
        }
        operation=Idle;
        if(link.error!=PocketError::OK){
            cue(DungeonSound::UiError);
            economy_telemetry.error=unsigned(link.error);
            message(previous==Sending||previous==CheckingReceipt?"Materials saved. Reconnect to sync the receipt.":link.detail?link.detail:pocket_error_message(link.error));return;
        }
        if(previous==Installing){++economy_telemetry.installed;economy_telemetry.error=0;cue(DungeonSound::UiSuccess);message("Ready! LEFT/RIGHT: move. FIRE: start. Hold FIRE 5s: menu.");}
        else if(previous==Inspecting)receive_transfer();
        else if(previous==Sending)operation=Acknowledge;
        else if(uint32_t(link.values[2])==magic&&uint32_t(link.values[3])>=ack_total){ack_pending=false;cue(DungeonSound::UiSuccess);message("Asteroids transferred and saved.");}
        else if(ack_time>Fixed(4.0))message("Materials saved. Reconnect to sync the receipt.");
        else operation=Acknowledge;
        return;
    }
    if(operation==Acknowledge){
        const Fixed before=ack_time;ack_time+=dt;
        if(before.raw()/(4096/3)!=ack_time.raw()/(4096/3)){
            link.start(DungeonPocketLink::ReceiptStatus,miner,port,0,ack_serial);operation=CheckingReceipt;
        }
        return;
    }
    if(operation==Idle||memory_card.busy())return;
    const auto& s=memory_card.status();if(s.completed!=request)return;
    const auto previous=operation;operation=Idle;
    if(s.state!=CardState::Succeeded){
        if(previous==Loading&&s.error==CardError::FileNotFound){saved=Save{};loaded=true;message("New inventory. Saves use memory card slot 1.");}
        else{cue(DungeonSound::UiError);message(card_error_message(s.error));economy_telemetry.error=uint32_t(s.error)+256;}
        heal_pending=0;return;
    }
    if(previous==Loading){
        Save candidate{};
        if(memory_card.size()!=sizeof(candidate)){loaded=false;cue(DungeonSound::UiError);message("Unsupported inventory save. Nothing overwritten.");return;}
        auto* out=reinterpret_cast<uint8_t*>(&candidate);for(unsigned i=0;i<sizeof(candidate);++i)out[i]=memory_card.data()[i];
        if(!valid(candidate)){loaded=false;cue(DungeonSound::UiError);message("Invalid inventory save. Nothing overwritten.");return;}
        saved=candidate;loaded=true;message("Inventory loaded from memory card slot 1.");
    }else{
        saved=pending;++economy_telemetry.saves;message(success);
        if(!heal_pending&&!ack_pending)cue(DungeonSound::UiSuccess);
        if(heal_pending){game.heal_player(heal_pending);heal_pending=0;}
        if(ack_pending)acknowledge();
    }
}
void DungeonEconomy::install(){
    const PocketApp* app=nullptr;
#if __has_include("pocket-apps.hh")
    if(miner.image)app=&miner;
#endif
    if(!app){message("Asteroid Catcher is missing from this build.");return;}
    link.start(DungeonPocketLink::Install,*app,port);operation=Installing;installation_percent=101;
    message("Checking PocketStation before installation...");
}
void DungeonEconomy::transfer(){
    if(!loaded){message("Load your save first. Check memory card slot 1.");return;}
    link.start(DungeonPocketLink::Inspect,miner,port);operation=Inspecting;message("Checking PocketStation materials...");
}
void DungeonEconomy::receive_transfer(){
    const auto& state=link.status;const auto* values=link.values;
    if(state.running_block!=link.block){message("Start Asteroid Catcher before transferring.");return;}
    if(uint32_t(values[2])!=magic||values[4]!=1){message("Wait for Asteroid Catcher, then try again.");return;}
    pending=saved;
    const int amount=dungeon_inventory::transfer(pending,state.serial,uint32_t(values[0]),uint32_t(values[3]));
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
int DungeonEconomy::nearby()const{
    const auto* p=player.get();if(near(p,pocket.get()))return 1;if(near(p,computer.get()))return 2;return 0;
}
void DungeonEconomy::activate(DungeonGame& game){
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
        if(game.health_value()>=100){cue(DungeonSound::UiError);message("Energy is full. Item kept in your inventory.");return;}
        --count;heal_pending=selected?100:35;commit(selected?"Energy tank used.":"Medkit used: +35 energy.");
    }
}
bool DungeonEconomy::input(DungeonGame& game,Fixed dt){
    DungeonMeasure measure(0);
    elapsed+=dt;
    if(auto* d=pocket.get()){
        d->transform.position[1]=Fixed(1.55)+sin_degrees(elapsed*Fixed(110.0))*Fixed(.14);
        d->transform.rotation[1]+=dt*Fixed(65.0);if(d->transform.rotation[1]>Fixed(360.0))d->transform.rotation[1]-=Fixed(360.0);
    }
    const bool was_busy=operation!=Idle;poll(game,dt);
    blocked=menu!=Closed||was_busy||operation!=Idle;
    if(!game.controls_enabled()){menu=Closed;show();return true;}
    if(was_busy||operation!=Idle){show();return true;}
    if(menu==Closed){
        if(epok::input.pressed(Button::Triangle)){menu=Inventory;selected=0;blocked=true;cue(DungeonSound::UiOpen);}
        else if(epok::input.pressed(Button::Cross)){
            const int station=nearby();if(station){menu=station==1?Pocket:Shop;selected=0;blocked=true;cue(DungeonSound::UiOpen);message(station==1?"Choose your PocketStation slot with LEFT/RIGHT.":"Sell minerals and resupply for your next run.");}
        }
    }else if(epok::input.pressed(Button::Circle)||epok::input.pressed(Button::Triangle)){menu=Closed;cue(DungeonSound::UiClose);}
    else{
        if(menu==Pocket&&(epok::input.pressed(Button::Left)||epok::input.pressed(Button::Right))){port^=1;cue(DungeonSound::UiMove);message("Selected slot changed. X confirms the action.");}
        const unsigned count=menu==Pocket?3:menu==Shop?5:4;
        if(epok::input.pressed(Button::Up)){selected=(selected+count-1)%count;cue(DungeonSound::UiMove);}
        if(epok::input.pressed(Button::Down)){selected=(selected+1)%count;cue(DungeonSound::UiMove);}
        if(epok::input.pressed(Button::Cross)){cue(DungeonSound::UiConfirm);activate(game);}
    }
    show();return blocked;
}
void DungeonEconomy::show(){
    const bool loading=menu==Closed&&operation==Loading;
    const bool open=menu!=Closed||loading;
    set_world3d_visible(!open);
    const int station=open?0:nearby();
    if(!view_dirty&&showing==open&&menu==shown_menu&&selected==shown_selected&&port==shown_port&&
        (open||station==shown_nearby)){publish();return;}
    view_dirty=false;shown_menu=menu;shown_selected=selected;shown_port=port;shown_nearby=station;
    if(open&&!showing){
        for(auto& v:views)if(auto* d=v.data.get()){
            v.rect=d->rect;v.text=d->text;v.image=d->image.enabled;v.progress=d->progress.enabled;
        }
        for(auto& f:flags)if(auto* d=f.data.get()){
            f.text=d->text.enabled;f.image=d->image.enabled;f.progress=d->progress.enabled;
            d->text.enabled=d->image.enabled=d->progress.enabled=false;
        }
        const DataHandle labels[]={title,summary,body,status,footer};
        const int y[]={28,46,69,176,203},height[]={12,12,100,24,12};
        for(unsigned i=0;i<5;++i)if(auto* d=labels[i].get()){
            d->rect.anchor_min[0]=d->rect.anchor_max[0]=d->rect.pivot[0]=0.0;
            d->rect.anchor_min[1]=d->rect.anchor_max[1]=d->rect.pivot[1]=1.0;
            d->rect.position[0]=28.0;d->rect.position[1]=Fixed(-y[i]*4096,Fixed::RAW);d->rect.size[0]=266.0;d->rect.size[1]=Fixed(height[i]*4096,Fixed::RAW);
            if(auto* b=body.get())d->text.font=b->text.font;
            d->text.align=TextAlign::Left;d->text.wrap=false;d->text.color[0]=190;d->text.color[1]=225;d->text.color[2]=235;
        }
        if(auto* d=title.get()){d->text.color[0]=115;d->text.color[1]=255;d->text.color[2]=212;}
        if(auto* d=summary.get()){d->text.color[0]=255;d->text.color[1]=201;d->text.color[2]=99;}
        if(auto* d=panel.get()){d->rect.position[0]=16.0;d->rect.position[1]=-16.0;d->rect.size[0]=288.0;d->rect.size[1]=202.0;}
    }else if(!open&&showing){
        for(const auto& v:views)if(auto* d=v.data.get()){d->rect=v.rect;d->text=v.text;d->image.enabled=v.image;d->progress.enabled=v.progress;}
        for(const auto& f:flags)if(auto* d=f.data.get()){d->text.enabled=f.text;d->image.enabled=f.image;d->progress.enabled=f.progress;}
    }
    showing=open;
    if(open)if(auto* d=panel.get())d->image.enabled=true;
    const DataHandle labels[]={title,summary,body,status,footer};
    if(open)for(const auto h:labels)if(auto* d=h.get())d->text.enabled=true;
    if(!open)if(auto* d=prompt.get()){
        d->text.enabled=menu==Closed;
        d->text.set_text(station==1?"X: POCKETSTATION   TRIANGLE: ITEMS":station==2?"X: TRADING PC   TRIANGLE: ITEMS":"TRIANGLE: INVENTORY");
    }
    if(!open){publish();return;}
    text(title,loading?"MEMORY CARD":menu==Pocket?"POCKETSTATION LINK":menu==Shop?"ASTEROID EXCHANGE":"INVENTORY / EQUIPMENT");
    Lines meta;meta.text("ORE ").number(saved.ore).text("   CREDITS ").number(saved.credits);
    if(menu==Pocket)meta.text("   SLOT ").number(port+1);text(summary,meta.value);
    Lines lines;
    if(loading){
        lines.text("Loading your supply inventory...\n\nKeep memory card slot 1 connected.");
        text(footer,"PLEASE WAIT...");
    }else if(menu==Pocket){
        lines.text("Install Asteroid Catcher to\ncollect ore on PocketStation.\nReturn here to transfer it.\n\n");
        lines.text(selected==0?"> ":"  ").text("Install / start minigame\n");
        lines.text(selected==1?"> ":"  ").text("Transfer asteroids\n");
        lines.text(selected==2?"> ":"  ").text("Load save (memory card 1)");
        text(footer,"< > SLOT   X: OK   O: CLOSE");
    }else if(menu==Shop){
        lines.text("Sell ore for 10 credits each.\n\n");
        lines.text(selected==0?"> ":"  ").text("Sell all asteroids\n");
        lines.text(selected==1?"> ":"  ").text("Medkit +35 energy     30 C\n");
        lines.text(selected==2?"> ":"  ").text("Energy tank (full)    80 C\n");
        lines.text(selected==3?"> ":"  ").text("Buster +8 damage     ").number(price(saved,2)).text(" C\n");
        lines.text(selected==4?"> ":"  ").text("Load save (memory card 1)");
        text(footer,"UP/DOWN: SELECT  X: OK  O: CLOSE");
    }else{
        lines.text("Choose an item to restore energy.\n\n");
        lines.text(selected==0?"> ":"  ").text("MEDKIT       x").number(saved.medkits).text("   +35 energy\n");
        lines.text(selected==1?"> ":"  ").text("ENERGY TANK  x").number(saved.tanks).text("   Full energy\n");
        lines.text(selected==2?"> ":"  ").text("BUSTER       LV ").number(saved.buster).text("   +").number(saved.buster*8).text(" damage\n");
        lines.text(selected==3?"> ":"  ").text("Load save (memory card 1)\n\nSell ore at the trading PC.");
        text(footer,"UP/DOWN: SELECT  X: OK  O: CLOSE");
    }
    if(operation!=Idle)text(footer,"PLEASE WAIT... KEEP CARD IN SLOT");
    text(body,lines.value);text(status,notice);publish();
}
