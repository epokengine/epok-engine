#include "DungeonGame.hpp"
#include "DungeonPlayerController.hpp"
#include "DungeonHudAssets.hpp"
#include "DungeonProfiling.hpp"
#include "retained.hpp"
EPOK_SPEED_BEGIN
#if defined(__mips__)
DungeonProfile dungeon_profile{};
DungeonProfile dungeon_completed_profile{};
struct DungeonPeakProfile {uint32_t values[103]{};};
DungeonPeakProfile dungeon_peak_profile{};
DungeonPeakProfile dungeon_explore_peak_profile{};
#endif
using namespace epok;
CombatTelemetry combat_telemetry{};
DungeonTelemetry dungeon_telemetry{};
DungeonFpsTelemetry dungeon_fps_telemetry{};
DungeonAudioTelemetry dungeon_audio_telemetry{};
DungeonFlowTelemetry dungeon_flow_telemetry{};
namespace {
// At a distance a 0.18 m thick closed panel has the same silhouette as its
// textured centre plane. Close panels keep the authored cube and its sides.
const int16_t door_plane_vertices[4][3]={{-2048,-2048,0},{2048,-2048,0},{2048,2048,0},{-2048,2048,0}};
const MeshQuad door_plane_faces[1]={{{0,1,2,3},0}};
const MeshGeometry door_plane{door_plane_vertices,4,door_plane_faces,1,false,{},{},{2048,2048,1}};
Fixed clamp01(Fixed v){return v<Fixed(0.0)?Fixed(0.0):(v>Fixed(1.0)?Fixed(1.0):v);}
int64_t distance2(const Fixed* a,const Fixed* b){const int64_t x=(a[0]-b[0]).raw(),z=(a[2]-b[2]).raw();return x*x+z*z;}
bool near(const Fixed* a,const Fixed* b,Fixed r){return distance2(a,b)<=int64_t(r.raw())*r.raw();}
Fixed center_height(const ActorData& a){return a.collider.center[1]*a.transform.scale[1];}
void tint(ActorData* d,int r,int g,int b){if(d){d->material.color[0]=uint8_t(r);d->material.color[1]=uint8_t(g);d->material.color[2]=uint8_t(b);}}
DataHandle bound(ObjectId id){auto* data=gameplay_actor_data(id);return data?handle(data):DataHandle{};}
}

bool DungeonGame::is_target(const ActorData* value)const{
    const auto* refs=encounter_bindings_data();if(!refs||!value||!value->owner)return false;
    const ObjectId id=value->owner->id();
    return id==refs->guard0||id==refs->guard1||id==refs->guard2||id==refs->boss;
}

DungeonRunFlowBase* DungeonGame::flow_rules()const{
    return run_flow_binding_.resolve(run_flow);
}
bool DungeonGame::controls_enabled()const{
    auto* rules=flow_rules();
    auto* vitals=vital_rules();
    return initialized&&rules&&vitals&&vitals->valid&&rules->status.valid&&rules->status.playing&&rules->status.alive;
}
void DungeonGame::begin_play(){
    begun_=true;initialized=false;state=Explore;restart_count_=0;active_enemies_=0;
    if(auto* rules=vital_rules())rules->start();
    if(auto* rules=flow_rules())rules->start();SceneLibrary::set_screen_fade(255);
    // The loader has bound every actor before begin_play. Establish the entry
    // view immediately, before the first 60 Hz tick initializes game services.
    auto* world=world_bindings_data();auto* encounter=encounter_bindings_data();
    if(world){
        const ObjectId hidden[]={world->room1,world->room2,world->room3,world->room4,
            world->door1_left,world->door1_right,world->door2_left,world->door2_right,world->door3_left,world->door3_right};
        for(auto id:hidden)if(auto* data=gameplay_actor_data(id))data->mesh=false;
    }
    if(encounter){
        const ObjectId hidden[]={encounter->key,encounter->pack0,encounter->pack1,encounter->pack2};
        for(auto id:hidden)if(auto* data=gameplay_actor_data(id))data->mesh=false;
        if(auto* data=gameplay_actor_data(encounter->boss))set_active(data,false);
    }
}
void DungeonGame::on_frame(uint32_t microseconds){
    economy.begin_frame();
    if(!microseconds)return;
    update_damage_feedback(microseconds);
#ifdef __mips__
    dungeon_completed_profile=dungeon_profile;
    const bool combat=state==BossFight||active_enemies_;
    if(initialized&&!economy.blocking()&&microseconds>(combat?dungeon_fps_telemetry.combat_max:dungeon_fps_telemetry.explore_max)){
        const auto& p=performance_stats;auto* v=combat?dungeon_peak_profile.values:dungeon_explore_peak_profile.values;
        const uint32_t phases[]={p.frame,microseconds,p.simulation_scanlines,p.collision_scanlines,p.render_scanlines,
            p.vertex_scanlines,p.polygon_scanlines,p.prepare_scanlines,p.finish_scanlines,p.steps,
            p.scripts_scanlines,p.animation_scanlines,p.motion_scanlines,p.audio_scanlines,
            p.static_mesh_scanlines,p.dynamic_mesh_scanlines};
        for(unsigned i=0;i<16;++i)v[i]=phases[i];
        for(unsigned i=0;i<12;++i)v[16+i]=dungeon_profile.scanlines[i];
        if(auto* body=player.get())for(unsigned i=0;i<3;++i)v[28+i]=uint32_t(body->transform.position[i].raw());
        const auto& retained=epok::retained_stats;
        v[31]=retained.objects;v[32]=retained.rebuilds;v[33]=retained.build_scanlines;
        v[34]=retained.quads;v[35]=retained.slot_top;v[36]=retained.dynamic_low;
        for(unsigned i=0;i<64;++i)v[37+i]=epok::rendered_actor_scanlines(i);
        v[101]=p.hud_scanlines;v[102]=p.sprite_scanlines;
    }
#endif
    auto& fps=dungeon_fps_telemetry;++fps.frames;if(microseconds>25000)++fps.slow;
    if(microseconds>fps.max_us)fps.max_us=microseconds;
    auto& maximum=!initialized?fps.startup_max:economy.blocking()?fps.interface_max:
        (state==BossFight||active_enemies_)?fps.combat_max:fps.explore_max;
    if(microseconds>maximum)maximum=microseconds;
}
void DungeonGame::end_play(EndPlayReason){begun_=false;clear_damage_feedback();if(auto* d=audio_source(music))d->stop();set_world3d_visible(true);SceneLibrary::set_screen_fade(0);}
void DungeonGame::start_presentation(){
    if(auto* rules=flow_rules())rules->start();state=Explore;SceneLibrary::set_screen_fade(255);
    if(auto* d=audio_source(music)){d->stop();d->volume=music_volume;d->play();++dungeon_flow_telemetry.starts;}
    publish_presentation();
}
void DungeonGame::publish_presentation(){
    auto& t=dungeon_flow_telemetry;auto* rules=flow_rules();
    if(rules){const auto& status=rules->status;state=RunState(status.state);t.phase=status.phase;t.elapsed=uint32_t(status.elapsed.raw());t.fade=status.fade;}
    t.inputs_locked=!controls_enabled();
    if(auto* d=audio_source(music)){t.music_volume=d->volume.raw();t.music_playing=d->is_playing();}
    else{t.music_volume=0;t.music_playing=0;}
}
bool DungeonGame::update_presentation(Fixed dt){
    auto* rules=flow_rules();if(!rules)return true;
    if(rules->status.playing)return false;
    if(rules->restart_requested(GameplayInputLibrary::button_pressed(Button::Start))){reset_run();return true;}
    bool playing=false;if(auto* d=audio_source(music))playing=d->is_playing();
    rules->advance(dt,playing);
    if(rules->status.restart){
        // Reset the resident level in place: all encounters, projectiles,
        // pickups, doors, player and camera return to their authored start.
        // Inventory and its asynchronous card service remain alive.
        ++dungeon_flow_telemetry.restarts;reset_run();return true;
    }
    SceneLibrary::set_screen_fade(rules->status.fade);
    if(auto* d=audio_source(music)){
        d->volume=music_volume*rules->status.music_gain;
        if(rules->status.stop_music&&playing)d->stop();
    }
    publish_presentation();return !rules->status.playing;
}
void DungeonGame::sound(DungeonSound event){
    DungeonMeasure measure(7);
    const unsigned i=unsigned(event);
    if(!initialized||i>=unsigned(DungeonSound::Count))return;
    const auto& cue=dungeon_audio::cues[i];
    auto* d=audio_source(sounds[cue.channel]);
    if(sound_clips[i]<0||!d){++dungeon_audio_telemetry.missing;return;}
    d->stop();d->clip=sound_clips[i];
    d->volume=Fixed(cue.volume,Fixed::RAW);d->pitch=1.0;d->play();
    ++dungeon_audio_telemetry.events[i];
}
void DungeonGame::initialize(){
    auto* body=epok::gameplay_actor_data(player_actor);if(!body)return;
    player=handle(body);for(int k=0;k<3;++k)spawn[k]=body->transform.position[k];
    auto* actors=encounter_bindings_data();auto* world=world_bindings_data();
    auto* fx_bindings=effect_bindings_data();auto* audio_route=audio_bindings_data();
    if(!actors||!world||!fx_bindings||!audio_route)return;
    const ObjectId guards[]={actors->guard0,actors->guard1,actors->guard2};
    const Fixed encounter_positions[6][3]={{-4.0,0.0,21.0},{5.0,0.0,29.0},{-27.0,0.0,20.0},{-23.0,0.0,29.0},{27.0,0.0,26.0},{0.0,0.0,40.0}};
    for(int i=0;i<3;++i)enemies[i].data=bound(guards[i]);
    for(int i=0;i<6;++i)for(int k=0;k<3;++k)encounters[i].home[k]=encounter_positions[i][k];
    boss=bound(actors->boss);if(auto* d=boss.get())for(int k=0;k<3;++k)boss_home[k]=d->transform.position[k];
    keycard=bound(actors->key);if(auto* d=keycard.get())for(int k=0;k<3;++k)key_home[k]=d->transform.position[k];
    const ObjectId packs[]={actors->pack0,actors->pack1,actors->pack2};
    for(int i=0;i<3;++i){health_packs[i].data=bound(packs[i]);if(auto* d=health_packs[i].data.get())for(int k=0;k<3;++k)health_packs[i].home[k]=d->transform.position[k];}
    const ObjectId left[]={world->door0_left,world->door1_left,world->door2_left,world->door3_left};
    const ObjectId right[]={world->door0_right,world->door1_right,world->door2_right,world->door3_right};
    for(int i=0;i<4;++i){auto& door=doors[i];door.left=bound(left[i]);door.right=bound(right[i]);door.axis=(i==1||i==2)?2:0;door.locked=i==3;
        auto* a=door.left.get();auto* b=door.right.get();if(a&&b){door.detail[0]=a->geometry;door.detail[1]=b->geometry;for(int k=0;k<3;++k)door.center[k]=(a->transform.position[k]+b->transform.position[k])/Fixed(2.0);}}
    const ObjectId fx[]={fx_bindings->effect0,fx_bindings->effect1,fx_bindings->effect2,fx_bindings->effect3,fx_bindings->effect4,fx_bindings->effect5,fx_bindings->effect6,fx_bindings->effect7,fx_bindings->effect8,fx_bindings->effect9,fx_bindings->effect10,fx_bindings->effect11};
    for(int i=0;i<12;++i)if(auto* d=gameplay_actor_data(fx[i])){d->sprite.enabled=false;if(i>=8)effects[i-8].data=handle(d);}
    const ObjectId audio[]={audio_route->player_shot_source,audio_route->world_hit_source,audio_route->robot_hit_source,audio_route->player_damage_source,audio_route->enemy_shot_source,audio_route->robot_destroyed_source,audio_route->ui_source};
    for(unsigned i=0;i<7;++i)sounds[i]=audio[i];
    music=audio_route->music_source;if(auto* d=audio_source(music))music_volume=d->volume;
    for(unsigned i=0;i<unsigned(DungeonSound::Count);++i){
        const auto clip=epok::assets::find(sound_effects,dungeon_audio::cues[i].key,epok::assets::Kind::AudioClip);
        sound_clips[i]=epok::assets::audio_clip(clip);
    }
    const ObjectId rooms[]={world->room0,world->room1,world->room2,world->room3,world->room4};
    for(int i=0;i<5;++i)room_geometry[i]=bound(rooms[i]);
    initialized=true;reset_run();economy.initialize(economy_policy);update_hud();
    for(const auto& room:room_geometry)prewarm_geometry(room.get());
    for(const auto& enemy:enemies)prewarm_geometry(enemy.data.get());
    prewarm_geometry(boss.get());
}
void DungeonGame::unload(Enemy& e){
    ++spatial_revision;
    if(auto* d=e.data.get()){d->collider.enabled=false;set_active(d,false);if(initialized)spatial_queries().changed(*d);}
    e.encounter=-1;e.rules.dead_time=0.0;e.rules.windup=0.0;e.rules.flash=0.0;e.rules.can_see=false;e.rules.sight_timer=0.0;
}
void DungeonGame::reset_run(){
    if(!initialized)return;
    clear_damage_feedback();
    ++spatial_revision;++structural_revision;
    auto* registry=epok::active_object_registry;
    auto* context=registry?registry->resolve<SpatialQueryContextComponent>(collision_queries):nullptr;
    if(context)context->reset_queries();else spatial_queries()=SpatialQueryBatch{};
    ++restart_count_;dungeon_telemetry={};dungeon_telemetry.restarts=restart_count_;combat_telemetry={};active_enemies_=0;
    if(auto* rules=vital_rules())rules->start();
    auto* encounters_policy=encounter_rules();
    boss_health=encounters_policy?encounters_policy->starting_health(true):HealthState{};kills=0;has_key=false;state=Explore;aiming=false;boss_hud_visible=false;
    elapsed=0.0;ai_elapsed=0.0;boss_rules=encounters_policy?encounters_policy->boss_start():DungeonBossState{};
    for(auto& e:encounters){e.health=encounters_policy?encounters_policy->starting_health(false):HealthState{};e.defeated=false;}
    for(auto& e:enemies)unload(e);
    if(auto* d=boss.get()){for(int k=0;k<3;++k)d->transform.position[k]=boss_home[k];d->collider.enabled=false;set_active(d,false);if(encounters_policy){const auto color=encounters_policy->world_enemy_color(true,0.0,0.0,false);tint(d,int(color.red),int(color.green),int(color.blue));}}
    if(auto* d=keycard.get()){for(int k=0;k<3;++k)d->transform.position[k]=key_home[k];set_active(d,true);}
    for(auto& p:health_packs){p.taken=false;if(auto* d=p.data.get()){for(int k=0;k<3;++k)d->transform.position[k]=p.home[k];set_active(d,true);}}
    if(auto* pool=projectiles())pool->reset();
    for(auto& f:effects){f.playing=false;if(auto* d=f.data.get())d->sprite.enabled=false;}
    for(auto& door:doors){door.progress=0.0;door.tint_state=255;door.resting=false;for(unsigned side=0;side<2;++side)if(auto* d=(side?door.right:door.left).get()){
        for(int k=0;k<3;++k)d->transform.position[k]=door.center[k];d->transform.position[door.axis]+=(side?Fixed(.75):Fixed(-.75));d->collider.enabled=true;}}
    if(auto* d=player.get()){for(int k=0;k<3;++k)d->transform.position[k]=spawn[k];d->transform.rotation[1]=0.0;}
    auto* a=get_owner();auto* level=a&&active_object_registry?active_object_registry->resolve<Level>(a->level_id()):nullptr;
    if(level)if(auto* controller=level->get_component<DungeonPlayerController>(*a))controller->reset_player();
    hud_dirty=true;update_visibility();update_hud();
    reset_motion_interpolation();start_presentation();
}
void DungeonGame::update_visibility(){
    auto* p=player.get();if(!p)return;
    const Fixed x=p->transform.position[0],z=p->transform.position[2];
    // Conservative portal neighbourhoods include the view before a door starts
    // opening, and the full camera boom. Collision and lights stay resident.
    const bool central=z>Fixed(0.0)&&z<Fixed(55.0)&&x>Fixed(-20.0)&&x<Fixed(20.0)&&
        !(state==BossFight&&z>Fixed(53.0)&&doors[3].progress.raw()==0);
    const bool visible[]={z<Fixed(14.0),central,x<Fixed(-8.0),x>Fixed(8.0),z>Fixed(42.0)};
    for(int i=0;i<5;++i)if(auto* d=room_geometry[i].get())d->mesh=visible[i];
    // Door meshes follow the same conservative room neighbourhood as their
    // adjoining rooms. Colliders remain resident for every spatial query.
    for(unsigned i=0;i<4;++i){const bool draw=visible[1]||visible[i?i+1:0];
        const bool detailed=near(p->transform.position,doors[i].center,Fixed(10.0));
        for(unsigned side=0;side<2;++side)if(auto* d=(side?doors[i].right:doors[i].left).get()){
            d->mesh=draw;d->geometry=detailed?doors[i].detail[side]:&door_plane;
        }
    }
    economy.hub_visible(visible[0]);
}
bool DungeonGame::line_of_sight(const ActorData& from,const ActorData& to,const SpatialQueryBatch* queries){
    Fixed eye[3]={from.transform.position[0],from.transform.position[1]+center_height(from),from.transform.position[2]},delta[3];
    for(int k=0;k<3;++k)delta[k]=to.transform.position[k]-eye[k];delta[1]+=center_height(to);
    return !(queries?queries->raycast(eye,delta,1,&from):raycast(eye,delta,1,&from));
}
void DungeonGame::damage_player(int amount){
    auto* rules=vital_rules();auto* world=encounter_rules();if(!rules||!world)return;
    const auto change=rules->damage(amount,controls_enabled());if(!change.applied)return;
    hud_dirty=true;++dungeon_telemetry.damage;sound(DungeonSound::PlayerDamage);
    update_damage_feedback(0,true);
    if(auto* combat=combat_rules()){const auto rumble=combat->damage_rumble();InputLibrary::rumble(rumble.large,rumble.small,rumble.duration_ms,0);}
    const auto death=world->world_player_death(change.depleted);
    if(death.defeat)if(auto* flow=flow_rules())flow->die();
    if(death.clear_aim)aiming=false;if(change.depleted)publish_presentation();
    if(death.reset_projectiles)if(auto* pool=projectiles())pool->reset();
}
void DungeonGame::update_damage_feedback(uint32_t microseconds,bool restart){
    auto* rules=combat_rules();if(!rules)return;
    const auto feedback=rules->damage_feedback(damage_flash_us,microseconds,restart);damage_flash_us=feedback.remaining_us;
    economy.publish_damage({feedback.enabled,feedback.border_pixels,feedback.red,feedback.green,feedback.blue});
}
void DungeonGame::clear_damage_feedback(){
    damage_flash_us=0;
    economy.publish_damage({false,0,0,0,0});
    InputLibrary::stop_rumble(0);
}
void DungeonGame::effect(const Fixed* point,uint16_t kind){
    auto* rules=combat_rules();if(!rules)return;const auto visual=rules->effect_step(kind,0,false);if(!visual.playing)return;
    for(auto& f:effects)if(!f.playing)if(auto* d=f.data.get()){
        f.playing=true;f.age=uint16_t(visual.age);f.kind=kind;for(int k=0;k<3;++k)d->transform.position[k]=point[k];
        d->sprite.size[0]=d->sprite.size[1]=visual.size;d->sprite.enabled=true;
        d->sprite.region[0]=(visual.frame%8)*32;d->sprite.region[1]=(visual.frame/8)*32;return;
    }
}
bool DungeonGame::damage_enemy(ActorData* target,uint32_t base){
    if(!target||!player_alive())return false;auto* rules=encounter_rules();if(!rules)return false;
    ++spatial_revision;const int amount=int32_t(rules->world_damage_total(base,economy.damage_bonus()));
    if(target==boss.get()&&rules->world_damage_eligible(true,player_alive(),state==BossFight,boss_health.current,false,0.0,false)){
        const auto change=HealthLibrary::damage(boss_health,amount);boss_health=change.state;boss_rules=rules->boss_hit(boss_rules);++combat_telemetry.hits;
        const auto result=rules->world_hit_result(true,change.depleted,boss_health.current);
        if(result.hud_dirty)hud_dirty=true;sound(DungeonSound(result.sound));
        kills=int(rules->world_reward_count(uint32_t(kills),result.reward));
        if(result.disable_collider){target->collider.enabled=false;spatial_queries().changed(*target);}
        if(result.death_clip){const auto animation=rules->world_death_animation();if(animation.play)target->animator.play(animation.clip,animation.loop);}
        if(result.victory)if(auto* flow=flow_rules()){flow->win();state=RunState(flow->status.state);}
        if(result.reset_projectiles)if(auto* pool=projectiles())pool->reset();
        if(result.explosion)effect(target->transform.position,1);return true;
    }
    for(auto& enemy:enemies)if(enemy.data.get()==target&&enemy.encounter>=0){
        auto& spawn=encounters[enemy.encounter];
        if(!rules->world_damage_eligible(false,player_alive(),state==BossFight,spawn.health.current,true,enemy.rules.dead_time,spawn.defeated))continue;
        const auto change=HealthLibrary::damage(spawn.health,amount);spawn.health=change.state;enemy.rules=rules->guard_hit(enemy.rules,change.depleted);++combat_telemetry.hits;
        const auto result=rules->world_hit_result(false,change.depleted,spawn.health.current);sound(DungeonSound(result.sound));
        if(result.defeated)spawn.defeated=true;kills=int(rules->world_reward_count(uint32_t(kills),result.reward));
        if(result.disable_collider){target->collider.enabled=false;spatial_queries().changed(*target);}
        if(result.death_clip){const auto animation=rules->world_death_animation();if(animation.play)target->animator.play(animation.clip,animation.loop);}
        if(result.hud_dirty)hud_dirty=true;
        if(result.victory)if(auto* flow=flow_rules()){flow->win();state=RunState(flow->status.state);}
        if(result.reset_projectiles)if(auto* pool=projectiles())pool->reset();
        if(result.explosion)effect(target->transform.position,1);return true;
    }
    return false;
}
bool DungeonGame::fire(const Fixed* origin,const Fixed* dir){
    if(!initialized||!player_alive())return false;
    auto* pool=projectiles();auto* rules=combat_rules();if(!pool||!rules)return false;
    const auto launched=pool->launch(0,4,rules->projectile(gameplay_vector(origin),gameplay_vector(dir)));
    auto* actor=active_object_registry->resolve<Actor3D>(launched.actor);auto* d=actor?actor->data():nullptr;
    if(launched.result!=ProjectileResult::Applied||!d)return false;
    for(int k=0;k<3;++k)combat_telemetry.muzzle[k]=origin[k].raw();
    const auto visual=rules->shot_visual();d->sprite.size[0]=d->sprite.size[1]=visual.size;
    d->sprite.color[0]=uint8_t(visual.red);d->sprite.color[1]=uint8_t(visual.green);d->sprite.color[2]=uint8_t(visual.blue);
    effect(origin,0);++combat_telemetry.shots;sound(DungeonSound::PlayerShot);return true;
}
void DungeonGame::enemy_fire(ActorData& from,bool fan){
    auto* body=player.get();auto* pool=projectiles();auto* rules=encounter_rules();if(!body||!pool||!rules)return;
    Fixed origin[3]={from.transform.position[0],from.transform.position[1]+center_height(from),from.transform.position[2]};
    Fixed dir[3];const auto aimed=rules->world_aim_direction(MathLibrary::subtract(rules->shot_target(gameplay_vector(body->transform.position)),gameplay_vector(origin)));
    if(!aimed.valid)return;gameplay_vector(aimed.value,dir);
    bool fired=false;
    const auto count=rules->pellet_count(fan);
    for(uint32_t ordinal=0;ordinal<count;++ordinal){
        const auto pellet=rules->pellet_index(ordinal,fan);
        const auto launched=pool->launch(4,4,rules->projectile(gameplay_vector(origin),gameplay_vector(dir),pellet,fan));
        auto* actor=active_object_registry->resolve<Actor3D>(launched.actor);auto* d=actor?actor->data():nullptr;
        if(launched.result!=ProjectileResult::Applied||!d)continue;
        d->sprite.size[0]=d->sprite.size[1]=rules->shot_size(fan);
        const auto color=rules->world_shot_color();d->sprite.color[0]=uint8_t(color.red);d->sprite.color[1]=uint8_t(color.green);d->sprite.color[2]=uint8_t(color.blue);
        ++dungeon_telemetry.enemy_shots;effect(origin,0);fired=true;
    }
    if(fired)sound(DungeonSound::EnemyShot);
}
void DungeonGame::update_enemies(Fixed dt,SpatialQueryBatch& queries){
    DungeonMeasure measure(6);
    auto* body=player.get();auto* rules=encounter_rules();if(!body||!rules)return;
    // Keep three resident visual actors and six persistent encounter records.
    for(auto& e:enemies)if(e.encounter>=0){auto* d=e.data.get();if(!d)continue;
        if(e.rules.dead_time.raw()>0){
            const auto step=rules->guard_schedule(e.rules,dt);e.rules=step.state;
            if(step.action==6)unload(e);continue;
        }
        if(rules->should_unload(e.rules,state==BossFight,near(body->transform.position,encounters[e.encounter].home,rules->movement.retention_radius)))unload(e);
    }
    for(int n=0;n<6;++n){
        if(!rules->should_activate(encounters[n].defeated,state==Explore,near(body->transform.position,encounters[n].home,rules->movement.activation_radius)))continue;
        bool assigned=false;for(auto& e:enemies)if(e.encounter==n)assigned=true;if(assigned)continue;
        for(auto& e:enemies)if(e.encounter<0)if(auto* d=e.data.get()){
            e.encounter=n;e.rules=rules->guard_start(uint32_t(n));
            for(int k=0;k<3;++k)d->transform.position[k]=encounters[n].home[k];d->collider.enabled=true;set_active(d,true);queries.changed(*d);d->animator.play(0,true);++combat_telemetry.spawned;break;
        }
    }
    unsigned active=0;
    for(auto& e:enemies){auto* d=e.data.get();if(e.encounter<0||!d)continue;++active;if(e.rules.dead_time.raw()>0)continue;
        const auto scheduled=rules->guard_schedule(e.rules,dt);e.rules=scheduled.state;if(scheduled.action!=7)continue;
        const Fixed step_dt=scheduled.dt;
        const bool close=near(d->transform.position,body->transform.position,rules->movement.sight_radius);
        const bool observed=close&&e.rules.sight_timer.raw()<=0?line_of_sight(*d,*body,&queries):e.rules.can_see;
        Fixed patrol_goal[3];gameplay_vector(rules->world_patrol_goal(gameplay_vector(encounters[e.encounter].home),e.rules.patrol),patrol_goal);
        const auto decision=rules->guard_decide(e.rules,step_dt,close,observed,
            near(d->transform.position,body->transform.position,rules->movement.guard_range),
            near(d->transform.position,patrol_goal,rules->movement.patrol_arrival));e.rules=decision.state;
        const auto motion=rules->world_enemy_motion(false,decision.action,e.rules.patrol,close,e.rules.can_see,
            gameplay_vector(d->transform.position),gameplay_vector(body->transform.position),gameplay_vector(encounters[e.encounter].home),step_dt,0.0);
        if(motion.turn){d->transform.rotation[1]=motion.yaw;queries.changed(*d);}
        if(decision.action==5)enemy_fire(*d,false);
        if(motion.move){Fixed delta[3];gameplay_vector(motion.delta,delta);queries.move_and_slide(*d,delta);}
        if(motion.patrol_turn)d->transform.rotation[1]=motion.patrol_yaw;
        const auto animation=rules->world_enemy_animation(false,decision.action,d->animator.clip);
        if(animation.play)d->animator.play(animation.clip,animation.loop);
        if(motion.patrol_turn)queries.sync(*d);
        if(epok::time.drawn_step){const auto color=rules->world_enemy_color(false,e.rules.flash,e.rules.windup,false);tint(d,int(color.red),int(color.green),int(color.blue));}
    }
    active_enemies_=active;combat_telemetry.active=active_enemies_;
    for(const auto& e:enemies)if(e.encounter>=0)if(auto* d=e.data.get();d&&near(d->transform.position,body->transform.position,Fixed(4.0))){++spatial_revision;break;}
}
void DungeonGame::update_boss(Fixed dt,SpatialQueryBatch& queries){
    auto* d=boss.get();auto* p=player.get();auto* rules=encounter_rules();if(!d||!p||!rules||state!=BossFight)return;
    ++spatial_revision;
    const bool needs_sight=rules->boss_needs_sight(boss_rules,dt);
    const bool observed=needs_sight&&line_of_sight(*d,*p,&queries);
    const auto decision=rules->boss_advance(boss_rules,boss_health.current,dt,observed,near(d->transform.position,p->transform.position,rules->boss_timing.boss_range));boss_rules=decision.state;
    const auto motion=rules->world_enemy_motion(true,decision.action,false,true,true,gameplay_vector(d->transform.position),
        gameplay_vector(p->transform.position),{},dt,decision.speed);
    if(motion.turn){d->transform.rotation[1]=motion.yaw;queries.changed(*d);}
    if(decision.action==5)enemy_fire(*d,decision.fan);
    if(motion.move){Fixed delta[3];gameplay_vector(motion.delta,delta);queries.move_and_slide(*d,delta);}
    const auto animation=rules->world_enemy_animation(true,decision.action,d->animator.clip);
    if(animation.play)d->animator.play(animation.clip,animation.loop);if(animation.charge_cue)sound(DungeonSound::BossCharge);
    if(epok::time.drawn_step){const auto color=rules->world_enemy_color(true,boss_rules.flash,boss_rules.windup,decision.fan);tint(d,int(color.red),int(color.green),int(color.blue));}
}
void DungeonGame::update_shots(Fixed dt,const SpatialQueryBatch* shared){
    DungeonMeasure measure(3);
    auto* pool=projectiles();auto* combat=combat_rules();if(!combat)return;
    for(unsigned team=0;pool&&team<2;++team){
        if(!pool->active_in(team*4,4))continue;
        const auto trace=combat->trace_settings(team!=0);
        const ProjectileTraceFilter filter{trace.mask,trace.ignore_player?gameplay_actor_id(player):ObjectId{},false};
        const auto batch=shared?pool->step_shared(team*4,4,dt,filter,*shared):ProjectileStep4{};
        ProjectileStep unshared[4];
        const ProjectileStep* results[4]={&batch.first,&batch.second,&batch.third,&batch.fourth};
        if(!shared){const auto ticket=pool->step(team*4,4,dt,filter);for(unsigned i=0;i<4;++i){unshared[i]=pool->result(ticket,i);results[i]=&unshared[i];}}
        for(const auto* result:results)if(result->state==ProjectileStepState::Hit){
            const auto& step=*result;
            auto* target=gameplay_actor_data(step.target);
            const auto impact=combat->impact(team!=0,target!=nullptr,target==player.get(),target?target->collider.layer:0);
            if(impact==2){if(auto* rules=encounter_rules()){
                const auto damage=rules->shot_damage(state==BossFight);damage_player(int32_t(damage>INT32_MAX?INT32_MAX:damage));
            }}
            else if(impact==1){
                if(auto* rules=combat_rules())damage_enemy(target,rules->tuning.damage);
            }
            else if(impact==0){++combat_telemetry.walls;sound(DungeonSound::WorldHit);}
            Fixed point[3];gameplay_vector(step.point,point);effect(point,1);
        }
    }
    // Effects are drawn once per frame, so a catch-up step need not advance them.
    if(epok::time.drawn_step)for(auto& f:effects)if(f.playing){auto* d=f.data.get();if(!d){f.playing=false;continue;}
        const auto visual=combat->effect_step(f.kind,f.age,true);f.age=uint16_t(visual.age);f.playing=visual.playing;
        if(!visual.playing)d->sprite.enabled=false;
        else{d->sprite.size[0]=d->sprite.size[1]=visual.size;d->sprite.region[0]=(visual.frame%8)*32;d->sprite.region[1]=(visual.frame/8)*32;}}
}
void DungeonGame::update_doors(Fixed dt){
    DungeonMeasure measure(4);auto* player_data=player.get();auto* rules=encounter_rules();if(!player_data||!rules)return;
    const Fixed open_radius=rules->world_door_radius(false),safety_radius=rules->world_door_radius(true);
    for(unsigned index=0;index<4;++index){auto& door=doors[index];
        const bool near_open=near(player_data->transform.position,door.center,open_radius);
        const bool near_safety=near(player_data->transform.position,door.center,safety_radius);
        // The rule is pure, so a resting door with the same inputs keeps its result.
        if(door.resting&&door.seen_open==near_open&&door.seen_safety==near_safety&&door.seen_key==has_key
            &&door.seen_state==uint8_t(state)&&door.seen_dt==dt)continue;
        const Fixed progress=door.progress;
        const auto step=rules->world_door_step(index,door.locked,has_key,uint32_t(state),progress,near_open,near_safety,dt);
        door.resting=!step.moved&&!step.opened&&step.progress==progress;door.seen_open=near_open;door.seen_safety=near_safety;
        door.seen_key=has_key;door.seen_state=uint8_t(state);door.seen_dt=dt;
        door.progress=step.progress;if(step.moved){++spatial_revision;++structural_revision;}if(step.opened)++dungeon_telemetry.doors_opened;
        door.tint_state=uint8_t(step.locked_tint);
        for(unsigned side=0;side<2;++side)if(auto* data=(side?door.right:door.left).get()){
            data->transform.position[door.axis]=door.center[door.axis]+(side?step.offset:-step.offset);
            if(step.moved)spatial_queries().changed(*data);tint(data,int(step.color.red),int(step.color.green),int(step.color.blue));
        }
    }
}
void DungeonGame::update_pickups(Fixed dt){
    auto* body=player.get();auto* rules=encounter_rules();if(!body||!rules)return;const Fixed radius=rules->world_pickup_radius();
    // Pickup reach is checked every step. The bob and draw flags are presentation.
    const bool present=epok::time.drawn_step;const Fixed bob=present?rules->world_pickup_bob(elapsed):Fixed(0.0);
    if(!has_key)if(auto* data=keycard.get()){
        if(present){data->mesh=rules->world_pickup_visible(0,gameplay_vector(body->transform.position));if(data->mesh)data->transform.position[1]=key_home[1]+bob;}
        if(rules->world_pickup_eligible(true,near(body->transform.position,key_home,radius),body->transform.position[1])){
            has_key=true;set_active(data,false);hud_dirty=true;sound(DungeonSound::Pickup);effect(data->transform.position,0);
        }
    }
    bool can_heal=can_heal_player();
    for(unsigned index=0;index<3;++index){auto& pack=health_packs[index];if(!pack.taken)if(auto* data=pack.data.get()){
        if(present){data->mesh=rules->world_pickup_visible(index+1,gameplay_vector(body->transform.position));if(data->mesh)data->transform.position[1]=pack.home[1]+bob;}
        if(rules->world_pickup_eligible(can_heal,near(body->transform.position,pack.home,radius),body->transform.position[1])){
            auto* vitals=vital_rules();if(!vitals||!vitals->heal(int32_t(rules->world_pickup_heal())).applied)continue;
            pack.taken=true;set_active(data,false);++dungeon_telemetry.heals;hud_dirty=true;sound(DungeonSound::Heal);can_heal=can_heal_player();
        }
    }}
}
void DungeonGame::update_hud(){
    DungeonMeasure measure(8);
    const auto energy=health_state();const auto* flow=flow_rules();
    const DungeonHUDSnapshot value{energy.current,energy.maximum,boss_health.current,boss_health.maximum,
        uint32_t(state),flow?flow->status.phase:0,has_key,aiming,player_alive(),false};
    // The widget compares too, but only after a registry lookup and a virtual call.
    const auto& last=published_hud;
    if(!hud_dirty&&published_view==economy.presenting()&&last.health==value.health&&last.maximum==value.maximum
        &&last.boss_health==value.boss_health&&last.boss_maximum==value.boss_maximum&&last.state==value.state
        &&last.phase==value.phase&&last.has_key==value.has_key&&last.aiming==value.aiming&&last.alive==value.alive)return;
    published_hud=value;published_view=economy.presenting();
    economy.publish_hud(value);hud_dirty=false;
}
void DungeonGame::tick(Fixed dt){
    DungeonMeasure measure(5);
    if(!initialized)initialize();if(!initialized||dt.raw()<=0)return;
    if(update_presentation(dt)){
        aiming=false;update_hud();dungeon_telemetry.state=state;dungeon_telemetry.health=health_value();return;
    }
    if(economy.blocking()){aiming=false;dungeon_telemetry.health=health_value();return;}
    ++combat_telemetry.ticks;++dungeon_telemetry.ticks;elapsed+=dt;
    if(epok::time.drawn_step)update_visibility();
    if(auto* rules=vital_rules())rules->advance(dt);
    if(player_alive()){
        update_pickups(dt);
        if(auto* p=player.get())if(auto* rules=flow_rules();rules&&rules->try_enter_boss(has_key,p->transform.position[2])){
            state=RunState(rules->status.state);hud_dirty=true;if(auto* d=boss.get()){d->collider.enabled=true;set_active(d,true);spatial_queries().changed(*d);d->animator.play(0,true);}
        }
        auto& queries=spatial_queries();if(auto* p=player.get())queries.changed(*p);update_doors(dt);update_enemies(dt,queries);ai_elapsed+=dt;
        if(encounter_rules()&&encounter_rules()->world_boss_due(ai_elapsed)){const Fixed step=ai_elapsed;ai_elapsed=0.0;update_boss(step,queries);}
        update_shots(dt,&queries);
    }else{
        update_doors(dt);
        update_shots(dt);
        if(auto* data=boss.get())if(auto* rules=encounter_rules();rules&&rules->world_hide_boss(uint32_t(state),data->animator.playing))set_active(data,false);
    }
    if(auto* rules=flow_rules();rules&&rules->restart_requested(GameplayInputLibrary::button_pressed(Button::Start)))reset_run();
    if(epok::time.drawn_step||hud_dirty)update_hud();dungeon_telemetry.state=state;dungeon_telemetry.health=health_value();dungeon_telemetry.key=has_key;dungeon_telemetry.kills=kills;dungeon_telemetry.boss_health=boss_health.current;
}
EPOK_SPEED_END
