#pragma once
#include "epok.hpp"
#include "DungeonEconomy.hpp"
#include "DungeonAudio.hpp"
#include "DungeonRunFlowBase.hpp"
#include "DungeonPlayerVitalsBase.hpp"
#include "DungeonEncounterBase.hpp"
#include "DungeonPlayerCombatBase.hpp"
#include "projectile3d.hpp"
#include "DungeonSceneBindings.hpp"

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
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::SpatialQueryContextComponent", Id="dec4ee46-3951-59ea-88a4-6cde2aa2f963") epok::ObjectId collision_queries{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonRunFlowBase",Id="ebcd4186-af50-5ed8-bab7-71ddedcacc05") epok::ObjectId run_flow{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="69bda44c-ff7b-500c-8fad-101596a1c107") epok::ObjectId player_actor{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerVitalsBase",Id="6daf3d70-dfab-5bff-a102-d863924b1c28") epok::ObjectId player_vitals{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ProjectilePool3DComponent",Id="a2ba286d-869f-5ef6-8a8b-560646a324f8") epok::ObjectId projectile_pool{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEncounterBase",Id="6cce0487-47f5-5328-92a3-9de000b0043f") epok::ObjectId encounter_policy{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerCombatBase",Id="48657156-44e9-50f7-8a32-e224a71c35f9") epok::ObjectId combat_policy{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEconomyPolicyBase",Id="c1977e79-b900-59b2-8e6b-e0af6a963c39") epok::ObjectId economy_policy{};
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("8d2bbf58-44f9-4ef8-adbb-2823ee734ac1");
    uint64_t class_id() const override{return static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonAudioBindings",Id="4132d6b9-8f1c-5dcb-b7dc-a025c1281ebc") epok::ObjectId audio_bindings{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEncounterBindings",Id="5e17ebfb-e7f9-5045-89a3-0aad4a3bd6e6") epok::ObjectId encounter_bindings{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonWorldBindings",Id="e5857fe8-aa87-567c-8831-52ad0f2f31fe") epok::ObjectId world_bindings{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEffectBindings",Id="49c3fe12-8d9f-503a-991e-e5be536a0802") epok::ObjectId effect_bindings{};
    enum RunState:uint8_t {Explore,BossFight,Victory,Defeat};
    void begin_play() override;
    void tick(epok::Fixed dt) override;
    void on_frame(uint32_t microseconds) override;
    void end_play(epok::EndPlayReason) override;
    bool fire(const epok::Fixed* origin,const epok::Fixed* direction);
    bool player_alive() const{auto* rules=flow_rules();return rules&&rules->status.alive;}
    bool ready() const{return initialized;}
    bool running()const{return begun_;}
    bool controls_enabled() const;
    void damage_player(int amount);
    void reset_run();
    bool handle_interfaces(epok::Fixed dt){return economy.input(*this,dt);}
    epok::HealthState health_state()const{auto* rules=vital_rules();return rules?rules->health:epok::HealthState{};}
    uint32_t health_value()const{return health_state().current;}
    bool can_heal_player()const{auto* rules=vital_rules();return rules&&rules->can_heal();}
    DungeonEconomyPolicyBase* economy_rules()const{return economy_policy_binding_.resolve(economy_policy);}
    uint32_t collision_revision()const{return spatial_revision;}
    uint32_t camera_collision_revision()const{return structural_revision;}
    // This game owns its collider mutations: player moves, doors, guard/boss
    // movement, activation and deaths all publish through changed()/move().
    // Keep the static station synchronized until a run reset invalidates it.
    epok::SpatialQueryBatch& spatial_queries(){
        auto* registry=epok::active_object_registry;
        auto* context=registry?registry->resolve<epok::SpatialQueryContextComponent>(collision_queries):nullptr;
        return context?context->batch():queries;
    }
    void heal_player(int amount){if(auto* rules=vital_rules()){rules->heal(amount);hud_dirty=true;++dungeon_telemetry.heals;sound(DungeonSound::Heal);}}
    void sound(DungeonSound event);
    bool is_target(const epok::ActorData* value)const;
    bool aiming=false;
private:
    DungeonEconomy economy;
    DungeonHUDSnapshot published_hud{};
    bool published_view=false;
    struct Encounter {epok::Fixed home[3]{};epok::HealthState health{60,60};bool defeated=false;};
    struct Enemy {epok::DataHandle data{};int encounter=-1;DungeonGuardState rules{};};
    struct Effect {epok::DataHandle data{};uint16_t age=0,kind=0;bool playing=false;};
    struct Door {epok::DataHandle left{},right{};const epok::MeshGeometry* detail[2]{};epok::Fixed center[3]{},progress=0.0;uint8_t axis=0,tint_state=255,seen_state=0;bool locked=false,resting=false,seen_key=false,seen_open=false,seen_safety=false;epok::Fixed seen_dt=0.0;};
    struct Pickup {epok::DataHandle data{};epok::Fixed home[3]{};bool taken=false;};
    Encounter encounters[6]{};Enemy enemies[3]{};Effect effects[4]{};
    Door doors[4]{};Pickup health_packs[3]{};
    epok::DataHandle player{},boss{},keycard{};
    uint32_t damage_flash_us=0;
    epok::ObjectId sounds[7]{};
    epok::ObjectId music{};
    epok::AudioSource* audio_source(epok::ObjectId id) const {
        auto* registry=epok::active_object_registry;
        auto* component=registry?registry->resolve<epok::AudioComponent>(id):nullptr;
        return component?component->source:nullptr;
    }
    epok::Fixed music_volume=.35;
    mutable DungeonRunFlowBinding run_flow_binding_;
    mutable DungeonPlayerVitalsBinding player_vitals_binding_;
    mutable DungeonRulesBinding<epok::ProjectilePool3DComponent> projectile_binding_;
    mutable DungeonEncounterBinding encounter_binding_;
    mutable DungeonPlayerCombatBinding combat_binding_;
    mutable DungeonEconomyPolicyBinding economy_policy_binding_;
    mutable DungeonRulesBinding<DungeonAudioBindings> audio_bindings_binding_;
    mutable DungeonRulesBinding<DungeonEncounterBindings> encounter_bindings_binding_;
    mutable DungeonRulesBinding<DungeonWorldBindings> world_bindings_binding_;
    mutable DungeonRulesBinding<DungeonEffectBindings> effect_bindings_binding_;
    DungeonAudioBindings* audio_bindings_data()const{return audio_bindings_binding_.resolve(audio_bindings);}
    DungeonEncounterBindings* encounter_bindings_data()const{return encounter_bindings_binding_.resolve(encounter_bindings);}
    DungeonWorldBindings* world_bindings_data()const{return world_bindings_binding_.resolve(world_bindings);}
    DungeonEffectBindings* effect_bindings_data()const{return effect_bindings_binding_.resolve(effect_bindings);}
    DungeonRunFlowBase* flow_rules()const;
    DungeonPlayerVitalsBase* vital_rules()const{return player_vitals_binding_.resolve(player_vitals);}
    epok::ProjectilePool3DComponent* projectiles()const{return projectile_binding_.resolve(projectile_pool);}
    DungeonEncounterBase* encounter_rules()const{return encounter_binding_.resolve(encounter_policy);}
    DungeonPlayerCombatBase* combat_rules()const{return combat_binding_.resolve(combat_policy);}
    int16_t sound_clips[unsigned(DungeonSound::Count)]{};
    epok::DataHandle room_geometry[5]{};
    epok::Fixed spawn[3]{},boss_home[3]{},key_home[3]{};
    epok::Fixed elapsed=0.0,ai_elapsed=0.0;
    DungeonBossState boss_rules{};
    RunState state=Explore;epok::HealthState boss_health{280,280};int kills=0;bool has_key=false,initialized=false,hud_dirty=true,boss_hud_visible=false;
    uint32_t spatial_revision=1,structural_revision=1;
    uint32_t restart_count_=0,active_enemies_=0;bool begun_=false;
    epok::SpatialQueryBatch queries;
    void initialize();void update_enemies(epok::Fixed dt,epok::SpatialQueryBatch& queries);void update_boss(epok::Fixed dt,epok::SpatialQueryBatch& queries);
    void start_presentation();bool update_presentation(epok::Fixed dt);void publish_presentation();
    void update_visibility();
    void update_damage_feedback(uint32_t microseconds,bool restart=false);void clear_damage_feedback();
    void update_shots(epok::Fixed dt,const epok::SpatialQueryBatch* queries=nullptr);void update_doors(epok::Fixed dt);void update_pickups(epok::Fixed dt);void update_hud();
    bool damage_enemy(epok::ActorData* target,uint32_t amount);
    void enemy_fire(epok::ActorData& from,bool fan);
    void effect(const epok::Fixed* point,uint16_t kind);
    void unload(Enemy& enemy);bool line_of_sight(const epok::ActorData& from,const epok::ActorData& to,const epok::SpatialQueryBatch* queries=nullptr);
};
