#pragma once
#include "epok.hpp"
#include "DungeonInventory.hpp"
#include "DungeonPocketLink.hpp"

class DungeonGame;
struct EconomyTelemetry {
    uint32_t menu=0,selection=0,slot=0,ore=0,credits=0,medkits=0,tanks=0,buster=0;
    uint32_t transfers=0,installed=0,error=0,saves=0,loaded=0,operation=0;
};
extern EconomyTelemetry economy_telemetry;

class DungeonEconomy {
public:
    void initialize();
    bool input(DungeonGame& game,epok::Fixed dt);
    bool blocking() const{return blocked;}
    unsigned damage_bonus()const{return saved.buster*8;}
    void hub_visible(bool visible){if(auto* d=pocket.get())d->mesh=visible;if(auto* d=computer.get())d->mesh=visible;}
private:
    enum Menu:uint8_t {Closed,Pocket,Shop,Inventory};
    enum Operation:uint8_t {Idle,Loading,Saving,Acknowledge,Installing,Inspecting,Sending,CheckingReceipt};
    dungeon_inventory::Save saved{},pending{};
    epok::DataHandle player{},pocket{},computer{},panel{},title{},summary{},body{},status{},footer{},prompt{},accent{};
    epok::Fixed elapsed=0.0,ack_time=0.0;
    Menu menu=Closed;Operation operation=Idle;
    unsigned selected=0,port=0;bool blocked=false,loaded=false,ack_pending=false;
    uint32_t request=0,ack_total=0,ack_serial=0;int heal_pending=0;
    DungeonPocketLink link;
    unsigned installation_percent=101;
    char notice[120]="Save data uses memory card slot 1.";
    struct View {epok::DataHandle data{};epok::RectTransform rect{};epok::Text text{};bool image=false,progress=false;};
    View views[6]{};
    struct Flags {epok::DataHandle data{};bool text=false,image=false,progress=false;};
    Flags flags[11]{};bool showing=false,view_dirty=true;
    Menu shown_menu=Closed;unsigned shown_selected=0,shown_port=0;int shown_nearby=-1;
    const char* success="Saved.";
    void show();void message(const char* text);void publish();
    void poll(DungeonGame& game,epok::Fixed dt);
    void load();bool commit(const char* text);
    void install();void transfer();void receive_transfer();void acknowledge();
    void activate(DungeonGame& game);
    int nearby()const;
};
