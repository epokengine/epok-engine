#pragma once
#include "epok.hpp"
#include "DungeonEconomy.hpp"
#include "DungeonAudio.hpp"
#include "DungeonFlow.hpp"

// Project-owned gameplay. Render actors and projectiles are resident pools;
// encounters retain their health when streamed out of the nearby active set.
struct CombatTelemetry {
    uint32_t ticks=0,spawned=0,active=0,shots=0,hits=0,immune=0,walls=0,recovered=0,muzzle_errors=0;
    int32_t muzzle[3]{};uint32_t states[3]{};
};
extern CombatTelemetry combat_telemetry;
struct DungeonTelemetry {
    uint32_t ticks=0,state=0,health=100,key=0,kills=0,boss_health=280;
    uint32_t enemy_shots=0,damage=0,heals=0,doors_opened=0,restarts=0;
};
extern DungeonTelemetry dungeon_telemetry;
// Cumulative budget counters cover every frame, including frames between host
// debugger samples. Phase maxima are diagnostic; the global limit stays 25ms.
struct DungeonFpsTelemetry {uint32_t frames=0,slow=0,max_us=0,explore_max=0,combat_max=0,interface_max=0,startup_max=0;};
extern DungeonFpsTelemetry dungeon_fps_telemetry;
struct DungeonFlowTelemetry {
    uint32_t phase=0,elapsed=0,fade=255,music_volume=0,music_playing=0,inputs_locked=1,starts=0,restarts=0;
};
extern DungeonFlowTelemetry dungeon_flow_telemetry;

class EPOK_CLASS(Blueprintable, Owners=World3D, Id="8d2bbf58-44f9-4ef8-adbb-2823ee734ac1") DungeonGame : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere, AssetKind=DataTable, Id="4c7d9dd3-68a9-4bd5-930e-872df2918c65")
    uint64_t sound_effects=0;
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("8d2bbf58-44f9-4ef8-adbb-2823ee734ac1");
    uint64_t class_id() const override{return static_class_id;}
    static DungeonGame* current;
    enum RunState:uint8_t {Explore,BossFight,Victory,Defeat};
    void begin_play() override;
    void tick(epok::Fixed dt) override;
    void on_frame(uint32_t microseconds) override;
    void end_play(epok::EndPlayReason) override;
    bool fire(const epok::Fixed* origin,const epok::Fixed* direction);
    bool player_alive() const{return state!=Defeat&&state!=Victory;}
    bool ready() const{return initialized;}
    bool controls_enabled() const{return initialized&&flow.playing()&&player_alive();}
    void damage_player(int amount);
    void reset_run();
    bool handle_interfaces(epok::Fixed dt){return economy.input(*this,dt);}
    int health_value()const{return health;}
    uint32_t collision_revision()const{return spatial_revision;}
    uint32_t camera_collision_revision()const{return structural_revision;}
    // This game owns its collider mutations: player moves, doors, guard/boss
    // movement, activation and deaths all publish through changed()/move().
    // Keep the static station synchronized until a run reset invalidates it.
    epok::SpatialQueryBatch& spatial_queries(){return queries;}
    void heal_player(int amount){health+=amount;if(health>100)health=100;hud_dirty=true;++dungeon_telemetry.heals;sound(DungeonSound::Heal);}
    void sound(DungeonSound event);
    bool aiming=false;
private:
    static DungeonEconomy economy;
    struct Encounter {epok::Fixed home[3]{};int health=60;bool defeated=false;};
    struct Enemy {epok::DataHandle data{};int encounter=-1;epok::Fixed timer=1.0,windup=0.0,flash=0.0,dead_time=0.0,sight_timer=0.0,ai_time=0.0;bool patrol=false,can_see=false;};
    struct Shot {epok::DataHandle data{};epok::Fixed velocity[3]{},life=0.0;uint16_t age=0;};
    struct Effect {epok::DataHandle data{};uint16_t age=0,kind=0;bool playing=false;};
    struct Door {epok::DataHandle left{},right{};const epok::MeshGeometry* detail[2]{};epok::Fixed center[3]{},progress=0.0;uint8_t axis=0,tint_state=255;bool locked=false;};
    struct Pickup {epok::DataHandle data{};epok::Fixed home[3]{};bool taken=false;};
    Encounter encounters[6]{};Enemy enemies[3]{};Shot shots[8]{};Effect effects[4]{};
    Door doors[4]{};Pickup health_packs[3]{};
    epok::DataHandle player{},boss{},keycard{},health_bar{},health_label{},key_label{},objective{},boss_bar{},boss_label{},message{},message_panel{},reticle{};
    epok::DataHandle damage_border{};
    static constexpr uint32_t damage_flash_duration_us=220000;
    uint32_t damage_flash_us=0;
    epok::DataHandle sounds[7]{};
    epok::DataHandle music{};
    epok::Fixed music_volume=.35;
    DungeonFlow flow;
    int16_t sound_clips[unsigned(DungeonSound::Count)]{};
    epok::DataHandle room_geometry[5]{};
    epok::Fixed spawn[3]{},boss_home[3]{},key_home[3]{};
    epok::Fixed invulnerable=0.0,elapsed=0.0,ai_elapsed=0.0,boss_timer=1.8,boss_windup=0.0,boss_flash=0.0;
    RunState state=Explore;int health=100,boss_health=280,kills=0;bool has_key=false,initialized=false,hud_dirty=true,boss_hud_visible=false;
    uint32_t spatial_revision=1,structural_revision=1;
    epok::SpatialQueryBatch queries;
    void initialize();void update_enemies(epok::Fixed dt,epok::SpatialQueryBatch& queries);void update_boss(epok::Fixed dt,epok::SpatialQueryBatch& queries);
    void start_presentation();bool update_presentation(epok::Fixed dt);void publish_presentation();
    void update_visibility();
    void update_damage_feedback(uint32_t microseconds);void clear_damage_feedback();
    void update_shots(epok::Fixed dt,const epok::SpatialQueryBatch* queries=nullptr);void update_doors(epok::Fixed dt);void update_pickups(epok::Fixed dt);void update_hud();
    bool damage_enemy(epok::ActorData* target,int amount);
    void enemy_fire(epok::ActorData& from,bool fan);
    void effect(const epok::Fixed* point,uint16_t kind);
    void unload(Enemy& enemy);bool line_of_sight(const epok::ActorData& from,const epok::ActorData& to,const epok::SpatialQueryBatch* queries=nullptr);
};
