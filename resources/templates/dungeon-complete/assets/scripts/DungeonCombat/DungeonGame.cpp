#include "DungeonGame.hpp"
#include "DungeonPlayerController.hpp"
#include "DungeonHudAssets.hpp"
#include "CharacterMotion.hpp"
#include "DungeonProfiling.hpp"
#include "retained.hpp"
#if defined(__mips__)
DungeonProfile dungeon_profile{};
DungeonProfile dungeon_completed_profile{};
struct DungeonPeakProfile {uint32_t values[103]{};};
DungeonPeakProfile dungeon_peak_profile{};
DungeonPeakProfile dungeon_explore_peak_profile{};
#endif
using namespace epok;
DungeonGame* DungeonGame::current=nullptr;
DungeonEconomy DungeonGame::economy{};
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
void direction(const Fixed* from,const Fixed* to,Fixed* out){
    int64_t n=0;for(int k=0;k<3;++k){out[k]=to[k]-from[k];n+=int64_t(out[k].raw())*out[k].raw();}
    // Decisions and projectiles are limited to a local 22-metre neighbourhood.
    const Fixed length(int32_t(character_motion::root(uint32_t(n>>4)))*4,Fixed::RAW);
    if(length.raw())for(int k=0;k<3;++k)out[k]/=length;
}
void tint(ActorData* d,int r,int g,int b){if(d){d->material.color[0]=uint8_t(r);d->material.color[1]=uint8_t(g);d->material.color[2]=uint8_t(b);}}
void number(Text& text,const char* prefix,unsigned value){
    char result[48];unsigned n=0;while(*prefix&&n<30)result[n++]=*prefix++;
    char digits[10];unsigned count=0;do{digits[count++]=char('0'+value%10);value/=10;}while(value&&count<10);
    while(count)result[n++]=digits[--count];result[n]=0;text.set_text(result);
}
DataHandle named(const char* name){auto* d=find_actor_data(name);return d?handle(d):DataHandle{};}
}

void DungeonGame::begin_play(){
    current=this;initialized=false;health=100;state=Explore;
    flow.begin();SceneLibrary::set_screen_fade(255);
    // The loader has bound every actor before begin_play. Establish the entry
    // view immediately, before the first 60 Hz tick initializes game services.
    const char* hidden[]={"Central amber light","Key Bay amber light","Med Bay amber light","Reactor amber light",
        "Door 1 Left","Door 1 Right","Door 2 Left","Door 2 Right","Door 3 Left","Door 3 Right",
        "Access Key","Health Pack 0","Health Pack 1","Health Pack 2"};
    for(const auto* name:hidden)if(auto* d=find_actor_data(name))d->mesh=false;
    if(auto* d=find_actor_data("Target Reactor Guardian"))set_active(d,false);
}
void DungeonGame::on_frame(uint32_t microseconds){
    if(!microseconds)return;
#ifdef __mips__
    dungeon_completed_profile=dungeon_profile;
    const bool combat=state==BossFight||combat_telemetry.active;
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
        (state==BossFight||combat_telemetry.active)?fps.combat_max:fps.explore_max;
    if(microseconds>maximum)maximum=microseconds;
}
void DungeonGame::end_play(EndPlayReason){if(current==this){if(auto* d=music.get())d->audio.stop();current=nullptr;set_world3d_visible(true);SceneLibrary::set_screen_fade(0);}}
void DungeonGame::start_presentation(){
    flow.begin();SceneLibrary::set_screen_fade(255);
    if(auto* d=music.get()){d->audio.stop();d->audio.volume=music_volume;d->audio.play();++dungeon_flow_telemetry.starts;}
    publish_presentation();
}
void DungeonGame::publish_presentation(){
    auto& t=dungeon_flow_telemetry;t.phase=flow.phase;t.elapsed=flow.elapsed;t.fade=flow.fade();t.inputs_locked=!controls_enabled();
    if(auto* d=music.get()){t.music_volume=d->audio.volume.raw();t.music_playing=d->audio.is_playing();}
    else{t.music_volume=0;t.music_playing=0;}
}
bool DungeonGame::update_presentation(Fixed dt){
    if(flow.playing())return false;
    bool playing=false;if(auto* d=music.get())playing=d->audio.is_playing();
    if(flow.step(uint32_t(dt.raw()),playing)){
        // Reset the resident level in place: all encounters, projectiles,
        // pickups, doors, player and camera return to their authored start.
        // Inventory and its asynchronous card service remain alive.
        ++dungeon_flow_telemetry.restarts;reset_run();return true;
    }
    SceneLibrary::set_screen_fade(flow.fade());
    if(auto* d=music.get()){
        d->audio.volume=music_volume*Fixed(flow.music_gain(),Fixed::RAW);
        if(flow.phase==DungeonFlow::RestartHold&&playing)d->audio.stop();
    }
    publish_presentation();return !flow.playing();
}
void DungeonGame::sound(DungeonSound event){
    DungeonMeasure measure(7);
    const unsigned i=unsigned(event);
    if(!initialized||i>=unsigned(DungeonSound::Count))return;
    const auto& cue=dungeon_audio::cues[i];
    auto* d=sounds[cue.channel].get();
    if(sound_clips[i]<0||!d){++dungeon_audio_telemetry.missing;return;}
    d->audio.stop();d->audio.clip=sound_clips[i];
    d->audio.volume=Fixed(cue.volume,Fixed::RAW);d->audio.pitch=1.0;d->audio.play();
    ++dungeon_audio_telemetry.events[i];
}
void DungeonGame::initialize(){
    auto* owner=get_owner();auto* body=owner?owner->data():nullptr;if(!body)return;
    player=handle(body);for(int k=0;k<3;++k)spawn[k]=body->transform.position[k];
    const char* guards[]={"Target Guard 0","Target Guard 1","Target Guard 2"};
    const Fixed encounter_positions[6][3]={{-4.0,0.0,21.0},{5.0,0.0,29.0},{-27.0,0.0,20.0},{-23.0,0.0,29.0},{27.0,0.0,26.0},{0.0,0.0,40.0}};
    for(int i=0;i<3;++i)enemies[i].data=named(guards[i]);
    for(int i=0;i<6;++i)for(int k=0;k<3;++k)encounters[i].home[k]=encounter_positions[i][k];
    boss=named("Target Reactor Guardian");if(auto* d=boss.get())for(int k=0;k<3;++k)boss_home[k]=d->transform.position[k];
    keycard=named("Access Key");if(auto* d=keycard.get())for(int k=0;k<3;++k)key_home[k]=d->transform.position[k];
    const char* packs[]={"Health Pack 0","Health Pack 1","Health Pack 2"};
    for(int i=0;i<3;++i){health_packs[i].data=named(packs[i]);if(auto* d=health_packs[i].data.get())for(int k=0;k<3;++k)health_packs[i].home[k]=d->transform.position[k];}
    const char* left[]={"Door 0 Left","Door 1 Left","Door 2 Left","Door 3 Left"};
    const char* right[]={"Door 0 Right","Door 1 Right","Door 2 Right","Door 3 Right"};
    for(int i=0;i<4;++i){auto& door=doors[i];door.left=named(left[i]);door.right=named(right[i]);door.axis=(i==1||i==2)?2:0;door.locked=i==3;
        auto* a=door.left.get();auto* b=door.right.get();if(a&&b){door.detail[0]=a->geometry;door.detail[1]=b->geometry;for(int k=0;k<3;++k)door.center[k]=(a->transform.position[k]+b->transform.position[k])/Fixed(2.0);}}
    const char* fx[]={"Combat FX 0","Combat FX 1","Combat FX 2","Combat FX 3","Combat FX 4","Combat FX 5","Combat FX 6","Combat FX 7","Combat FX 8","Combat FX 9","Combat FX 10","Combat FX 11"};
    for(int i=0;i<12;++i)if(auto* d=find_actor_data(fx[i])){d->sprite.enabled=false;if(i<8)shots[i].data=handle(d);else effects[i-8].data=handle(d);}
    const char* audio[]={"Player Weapon Sound","World Impact Sound","Robot Impact Sound","Player Damage Sound","Enemy Weapon Sound","Robot Destruction Sound","PocketStation Terminal"};
    for(unsigned i=0;i<7;++i)sounds[i]=named(audio[i]);
    music=named("Reactor Station");if(auto* d=music.get())music_volume=d->audio.volume;
    for(unsigned i=0;i<unsigned(DungeonSound::Count);++i){
        const auto clip=epok::assets::find(sound_effects,dungeon_audio::cues[i].key,epok::assets::Kind::AudioClip);
        sound_clips[i]=epok::assets::audio_clip(clip);
    }
    const char* rooms[]={"Entry amber light","Central amber light","Key Bay amber light","Med Bay amber light","Reactor amber light"};
    for(int i=0;i<5;++i)room_geometry[i]=named(rooms[i]);
    health_bar=named("Energy Bar");health_label=named("Energy Label");key_label=named("Key Status");objective=named("Objective");
    boss_bar=named("Boss Bar");boss_label=named("Boss Label");message=named("Game Message");message_panel=named("Message Panel");reticle=named("Aim Reticle");
    initialized=true;reset_run();economy.initialize();
    for(const auto& room:room_geometry)prewarm_geometry(room.get());
    for(const auto& enemy:enemies)prewarm_geometry(enemy.data.get());
    prewarm_geometry(boss.get());
}
void DungeonGame::unload(Enemy& e){
    ++spatial_revision;
    if(auto* d=e.data.get()){d->collider.enabled=false;set_active(d,false);if(initialized)queries.changed(*d);}
    e.encounter=-1;e.dead_time=0.0;e.windup=0.0;e.flash=0.0;e.can_see=false;e.sight_timer=0.0;
}
void DungeonGame::reset_run(){
    if(!initialized)return;
    ++spatial_revision;++structural_revision;
    queries=SpatialQueryBatch{};
    const unsigned runs=dungeon_telemetry.restarts+1;dungeon_telemetry={};dungeon_telemetry.restarts=runs;combat_telemetry={};
    health=100;boss_health=280;kills=0;has_key=false;state=Explore;aiming=false;boss_hud_visible=false;
    elapsed=0.0;ai_elapsed=0.0;invulnerable=0.0;boss_timer=1.8;boss_windup=0.0;boss_flash=0.0;
    for(auto& e:encounters){e.health=60;e.defeated=false;}
    for(auto& e:enemies)unload(e);
    if(auto* d=boss.get()){for(int k=0;k<3;++k)d->transform.position[k]=boss_home[k];d->collider.enabled=false;set_active(d,false);tint(d,255,255,255);}
    if(auto* d=keycard.get()){for(int k=0;k<3;++k)d->transform.position[k]=key_home[k];set_active(d,true);}
    for(auto& p:health_packs){p.taken=false;if(auto* d=p.data.get()){for(int k=0;k<3;++k)d->transform.position[k]=p.home[k];set_active(d,true);}}
    for(auto& s:shots){s.life=0.0;if(auto* d=s.data.get())d->sprite.enabled=false;}
    for(auto& f:effects){f.playing=false;if(auto* d=f.data.get())d->sprite.enabled=false;}
    for(auto& door:doors){door.progress=0.0;door.tint_state=255;for(unsigned side=0;side<2;++side)if(auto* d=(side?door.right:door.left).get()){
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
    if(!controls_enabled()||amount<=0||invulnerable.raw()>0)return;
    health-=amount;if(health<0)health=0;invulnerable=.7;hud_dirty=true;++dungeon_telemetry.damage;sound(DungeonSound::PlayerDamage);
    if(!health){state=Defeat;aiming=false;flow.die();publish_presentation();for(auto& s:shots){s.life=0.0;if(auto* d=s.data.get())d->sprite.enabled=false;}}
}
void DungeonGame::effect(const Fixed* point,uint16_t kind){
    for(auto& f:effects)if(!f.playing)if(auto* d=f.data.get()){
        f.playing=true;f.age=0;f.kind=kind;for(int k=0;k<3;++k)d->transform.position[k]=point[k];
        d->sprite.size[0]=d->sprite.size[1]=kind?Fixed(.48):Fixed(.28);d->sprite.enabled=true;
        const unsigned frame=kind?6:4;d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;return;
    }
}
bool DungeonGame::damage_enemy(ActorData* target,int amount){
    if(!target||!player_alive())return false;
    ++spatial_revision;
    amount+=int(economy.damage_bonus());
    if(target==boss.get()&&state==BossFight&&boss_health>0){
        boss_health-=amount;if(boss_health<0)boss_health=0;boss_flash=.16;hud_dirty=true;++combat_telemetry.hits;
        sound(boss_health?DungeonSound::RobotHit:DungeonSound::BossDestroyed);
        if(!boss_health){target->collider.enabled=false;spatial_queries().changed(*target);target->animator.play(2,false);state=Victory;
            for(auto& s:shots){s.life=0.0;if(auto* d=s.data.get())d->sprite.enabled=false;}
            effect(target->transform.position,1);}
        return true;
    }
    for(auto& e:enemies)if(e.data.get()==target&&e.encounter>=0&&e.dead_time.raw()<=0){
        auto& spawn=encounters[e.encounter];if(spawn.defeated)return false;
        spawn.health-=amount;e.flash=.14;++combat_telemetry.hits;
        sound(spawn.health>0?DungeonSound::RobotHit:DungeonSound::RobotDestroyed);
        if(spawn.health<=0){spawn.health=0;spawn.defeated=true;++kills;e.dead_time=.85;target->collider.enabled=false;spatial_queries().changed(*target);target->animator.play(2,false);hud_dirty=true;}
        return true;
    }
    return false;
}
bool DungeonGame::fire(const Fixed* origin,const Fixed* dir){
    if(!initialized||!player_alive())return false;
    for(unsigned i=0;i<4;++i){auto& s=shots[i];if(s.life.raw()>0)continue;auto* d=s.data.get();if(!d)continue;
        s.life=1.7;s.age=0;for(int k=0;k<3;++k){d->transform.position[k]=origin[k];s.velocity[k]=dir[k]*Fixed(22.0);combat_telemetry.muzzle[k]=origin[k].raw();}
        d->sprite.size[0]=d->sprite.size[1]=.18;d->sprite.color[0]=d->sprite.color[1]=d->sprite.color[2]=255;
        d->sprite.region[0]=d->sprite.region[1]=0;d->sprite.enabled=true;effect(origin,0);++combat_telemetry.shots;sound(DungeonSound::PlayerShot);return true;
    }
    return false;
}
void DungeonGame::enemy_fire(ActorData& from,bool fan){
    auto* body=player.get();if(!body)return;
    Fixed origin[3]={from.transform.position[0],from.transform.position[1]+center_height(from),from.transform.position[2]};
    Fixed destination[3]={body->transform.position[0],body->transform.position[1]+Fixed(.95),body->transform.position[2]},dir[3];direction(origin,destination,dir);
    bool fired=false;
    for(int pellet=fan?-1:0;pellet<=(fan?1:0);++pellet){
        for(unsigned i=4;i<8;++i){auto& s=shots[i];if(s.life.raw()>0)continue;auto* d=s.data.get();if(!d)continue;
            s.life=2.5;s.age=0;const Fixed angle(pellet*12*4096,Fixed::RAW),c=cos_degrees(angle),sn=sin_degrees(angle);
            const Fixed vx=dir[0]*c+dir[2]*sn,vz=dir[2]*c-dir[0]*sn;
            for(int k=0;k<3;++k)d->transform.position[k]=origin[k];
            const Fixed speed=fan?Fixed(8.0):Fixed(7.0);
            s.velocity[0]=vx*speed;s.velocity[1]=dir[1]*speed;s.velocity[2]=vz*speed;
            d->sprite.size[0]=d->sprite.size[1]=fan?Fixed(.32):Fixed(.24);
            d->sprite.color[0]=255;d->sprite.color[1]=100;d->sprite.color[2]=50;
            d->sprite.region[0]=d->sprite.region[1]=0;d->sprite.enabled=true;++dungeon_telemetry.enemy_shots;effect(origin,0);fired=true;break;
        }
    }
    if(fired)sound(DungeonSound::EnemyShot);
}
void DungeonGame::update_enemies(Fixed dt,SpatialQueryBatch& queries){
    DungeonMeasure measure(6);
    auto* body=player.get();if(!body)return;
    // Maintain only nearby encounters. Defeated encounters never respawn.
    for(auto& e:enemies)if(e.encounter>=0){auto* d=e.data.get();if(!d)continue;
        if(e.dead_time.raw()>0){e.dead_time-=dt;if(e.dead_time.raw()<=0)unload(e);continue;}
        if(state==BossFight||!near(body->transform.position,encounters[e.encounter].home,Fixed(22.0)))unload(e);
    }
    if(state==Explore)for(int n=0;n<6;++n){if(encounters[n].defeated||!near(body->transform.position,encounters[n].home,Fixed(14.0)))continue;
        bool assigned=false;for(auto& e:enemies)if(e.encounter==n)assigned=true;if(assigned)continue;
        for(auto& e:enemies)if(e.encounter<0)if(auto* d=e.data.get()){
            e.encounter=n;e.timer=1.1;e.windup=0.0;e.flash=0.0;e.patrol=false;e.can_see=false;
            // Stagger the first update as a delay, rather than charging an
            // unelapsed tick and running physics on this activation tick.
            e.sight_timer=Fixed((n%3+1)*137,Fixed::RAW);e.ai_time=Fixed((n&1)?-68:0,Fixed::RAW);
            for(int k=0;k<3;++k)d->transform.position[k]=encounters[n].home[k];d->collider.enabled=true;set_active(d,true);queries.changed(*d);d->animator.play(0,true);++combat_telemetry.spawned;break;
        }
    }
    unsigned active=0;
    for(auto& e:enemies){auto* d=e.data.get();if(e.encounter<0||!d)continue;++active;if(e.dead_time.raw()>0)continue;
        // Every guard still advances at 30 Hz; alternate phases spread their
        // physics/visibility work over the 60 Hz simulation ticks.
        e.ai_time+=dt;if(e.ai_time<Fixed(1.0/30.0))continue;
        const Fixed enemy_dt=e.ai_time;e.ai_time=0.0;
        const Fixed dt=enemy_dt;
        e.timer-=dt;if(e.flash.raw()>0)e.flash-=dt;
        e.sight_timer-=dt;
        const bool close=near(d->transform.position,body->transform.position,Fixed(11.0));
        if(!close)e.can_see=false;
        else if(e.sight_timer.raw()<=0){e.can_see=line_of_sight(*d,*body,&queries);e.sight_timer=.1;}
        const bool visible=close&&e.can_see;
        if(visible){
            const Fixed dx=body->transform.position[0]-d->transform.position[0],dz=body->transform.position[2]-d->transform.position[2];
            d->transform.rotation[1]=Fixed(character_motion::heading(dx.raw(),dz.raw()),Fixed::RAW);
            queries.changed(*d);
            if(e.windup.raw()>0){e.windup-=dt;if(e.windup.raw()<=0){enemy_fire(*d,false);e.timer=2.1;}}
            else if(e.timer.raw()<=0){e.windup=.5;d->animator.play(0,true);}
            else if(!near(d->transform.position,body->transform.position,Fixed(5.0))){
                Fixed to[3]={body->transform.position[0],d->transform.position[1],body->transform.position[2]},dir[3];direction(d->transform.position,to,dir);
                const Fixed delta[3]={dir[0]*Fixed(1.05)*dt,Fixed(-.05),dir[2]*Fixed(1.05)*dt};queries.move_and_slide(*d,delta);
                if(d->animator.clip!=1)d->animator.play(1,true);
            }else if(d->animator.clip!=0)d->animator.play(0,true);
        }else{
            e.windup=0.0;Fixed to[3];for(int k=0;k<3;++k)to[k]=encounters[e.encounter].home[k];to[0]+=e.patrol?Fixed(1.25):Fixed(-1.25);
            if(near(d->transform.position,to,Fixed(.2)))e.patrol=!e.patrol;
            Fixed dir[3];direction(d->transform.position,to,dir);const Fixed delta[3]={dir[0]*Fixed(.6)*dt,Fixed(-.05),dir[2]*Fixed(.6)*dt};queries.move_and_slide(*d,delta);
            d->transform.rotation[1]=Fixed(character_motion::heading(dir[0].raw(),dir[2].raw()),Fixed::RAW);if(d->animator.clip!=1)d->animator.play(1,true);
            queries.sync(*d);
        }
        if(e.flash.raw()>0)tint(d,255,110,110);else if(e.windup.raw()>0)tint(d,255,180,85);else tint(d,100,210,255);
    }
    combat_telemetry.active=active;
    // Distant guards cannot change the support of a stationary player. Moving
    // input still queries physics normally; nearby colliders invalidate rest.
    for(const auto& e:enemies)if(e.encounter>=0)if(auto* d=e.data.get();d&&
        near(d->transform.position,body->transform.position,Fixed(4.0))){++spatial_revision;break;}
}
void DungeonGame::update_boss(Fixed dt,SpatialQueryBatch& queries){
    auto* d=boss.get();auto* p=player.get();if(!d||!p||state!=BossFight)return;
    ++spatial_revision;
    const bool phase_two=boss_health<=140;boss_timer-=dt;if(boss_flash.raw()>0)boss_flash-=dt;
    const Fixed dx=p->transform.position[0]-d->transform.position[0],dz=p->transform.position[2]-d->transform.position[2];
    d->transform.rotation[1]=Fixed(character_motion::heading(dx.raw(),dz.raw()),Fixed::RAW);
    queries.changed(*d);
    if(boss_windup.raw()>0){boss_windup-=dt;if(boss_windup.raw()<=0){enemy_fire(*d,phase_two);boss_timer=phase_two?Fixed(1.35):Fixed(1.9);}}
    else if(boss_timer.raw()<=0&&line_of_sight(*d,*p,&queries)){boss_windup=phase_two?Fixed(.5):Fixed(.7);d->animator.play(0,true);sound(DungeonSound::BossCharge);}
    else if(!near(d->transform.position,p->transform.position,Fixed(7.0))){
        Fixed to[3]={p->transform.position[0],d->transform.position[1],p->transform.position[2]},dir[3];direction(d->transform.position,to,dir);
        const Fixed speed=phase_two?Fixed(1.3):Fixed(.85),delta[3]={dir[0]*speed*dt,Fixed(-.05),dir[2]*speed*dt};queries.move_and_slide(*d,delta);
        if(d->animator.clip!=1)d->animator.play(1,true);
    }else if(d->animator.clip!=0)d->animator.play(0,true);
    if(boss_flash.raw()>0)tint(d,255,90,90);else if(boss_windup.raw()>0)tint(d,255,150,55);else tint(d,255,phase_two?190:255,phase_two?150:255);
}
void DungeonGame::update_shots(Fixed dt,const SpatialQueryBatch* shared){
    DungeonMeasure measure(3);
    SpatialQueryBatch own_queries;const auto& world_queries=shared?*shared:own_queries;
    for(unsigned team=0;team<2;++team){
        RaycastQuery queries[4];SpatialHit hits[4];unsigned indices[4],count=0;
        for(unsigned i=team*4;i<team*4+4;++i){auto& s=shots[i];auto* d=s.data.get();if(!d||s.life.raw()<=0)continue;
            auto& q=queries[count];indices[count++]=i;for(int k=0;k<3;++k){q.origin[k]=d->transform.position[k];q.displacement[k]=s.velocity[k]*dt;}}
        if(count)world_queries.raycast_batch(queries,hits,count,team?5:3,team?nullptr:player.get());
        for(unsigned n=0;n<count;++n){auto& s=shots[indices[n]];auto* d=s.data.get();if(!d)continue;const auto& hit=hits[n];
            if(hit){auto* target=hit_entity(hit).get();if(team){if(target==player.get())damage_player(state==BossFight?16:12);}
                else if(target&&(target->collider.layer&2))damage_enemy(target,20);else{++combat_telemetry.walls;sound(DungeonSound::WorldHit);}
                effect(hit.point,1);s.life=0.0;
            }else{for(int k=0;k<3;++k)d->transform.position[k]+=queries[n].displacement[k];s.life-=dt;}
            d->sprite.region[0]=((s.age++/2)%4)*32;if(s.life.raw()<=0)d->sprite.enabled=false;
        }
    }
    for(auto& f:effects)if(f.playing){auto* d=f.data.get();if(!d){f.playing=false;continue;}++f.age;
        const unsigned frame=f.kind?6+f.age/2:(f.age<4?4:5),end=f.kind?10:6;
        if(f.age>=end){d->sprite.enabled=false;f.playing=false;}else{d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;}}
}
void DungeonGame::update_doors(Fixed dt){
    DungeonMeasure measure(4);
    auto* p=player.get();if(!p)return;
    for(unsigned i=0;i<4;++i){auto& door=doors[i];bool open=(!door.locked||has_key||state==Victory)&&near(p->transform.position,door.center,Fixed(5.0));
        if(i==3&&state==BossFight)open=false;
        // Never close through the player. The locked gate cannot be opened by
        // this safety rule until it has legitimately started opening.
        if(!open&&door.progress.raw()>0&&near(p->transform.position,door.center,Fixed(2.0)))open=true;
        const Fixed before=door.progress;door.progress=clamp01(door.progress+(open?dt:-dt)*Fixed(1.4));
        if(before!=door.progress){++spatial_revision;++structural_revision;}
        if(before<Fixed(1.0)&&door.progress==Fixed(1.0))++dungeon_telemetry.doors_opened;
        const uint8_t tint_state=uint8_t(i==3&&!has_key);
        if(before==door.progress&&door.tint_state==tint_state)continue;
        door.tint_state=tint_state;
        const Fixed easing=door.progress*door.progress*(Fixed(3.0)-Fixed(2.0)*door.progress);
        for(unsigned side=0;side<2;++side)if(auto* d=(side?door.right:door.left).get()){
            d->transform.position[door.axis]=door.center[door.axis]+(side?Fixed(1.0):Fixed(-1.0))*(Fixed(.75)+easing*Fixed(1.7));
            if(before!=door.progress)spatial_queries().changed(*d);
            if(i==3&&!has_key)tint(d,255,130,110);else tint(d,255,255,255);
        }
    }
}
void DungeonGame::update_pickups(Fixed dt){
    auto* p=player.get();if(!p)return;const Fixed bob=sin_degrees(elapsed*Fixed(120.0))*Fixed(.12);
    if(!has_key)if(auto* d=keycard.get()){
        d->mesh=p->transform.position[0]<Fixed(-8.0);
        if(d->mesh)d->transform.position[1]=key_home[1]+bob;
        if(near(p->transform.position,key_home,Fixed(1.0))&&p->transform.position[1]<Fixed(2.2)){
            has_key=true;set_active(d,false);hud_dirty=true;sound(DungeonSound::Pickup);effect(d->transform.position,0);
        }
    }
    for(unsigned n=0;n<3;++n){auto& pack=health_packs[n];if(!pack.taken)if(auto* d=pack.data.get()){
        d->mesh=n==0?p->transform.position[0]>Fixed(8.0):(n==1?p->transform.position[0]<Fixed(-8.0):(p->transform.position[2]>Fixed(32.0)&&p->transform.position[2]<Fixed(55.0)));
        if(d->mesh)d->transform.position[1]=pack.home[1]+bob;
        // Full health leaves the pack in the world for a later return.
        if(health<100&&near(p->transform.position,pack.home,Fixed(1.0))&&p->transform.position[1]<Fixed(2.2)){
            health+=35;if(health>100)health=100;pack.taken=true;set_active(d,false);++dungeon_telemetry.heals;hud_dirty=true;sound(DungeonSound::Heal);
        }
    }}
}
void DungeonGame::update_hud(){
    // Reveal the next objective with the key pickup, preparing the boss overlay
    // before room/door work can coincide with its initial text/layout build.
    // Building that text/layout together with two simulation steps and the
    // first boss pose caused the room-entry frame to exceed the 25 ms budget.
    const bool boss_hud=state==BossFight||(state==Explore&&has_key);
    if(boss_hud!=boss_hud_visible){boss_hud_visible=boss_hud;hud_dirty=true;}
    DungeonMeasure measure(8);
    if(auto* d=reticle.get())d->text.enabled=aiming&&player_alive();
    if(!hud_dirty)return;hud_dirty=false;
    if(auto* d=health_bar.get()){
        const Fixed fraction=Fixed(health*4096,Fixed::RAW)/Fixed(100);
        d->progress.value=fraction;d->progress.enabled=false;
        // Crop the source cells as the destination shrinks: cell spacing stays
        // constant, rather than squeezing the complete artwork into less space.
        d->rect.size[0]=Fixed(dungeon_hud::fill_width)*fraction;
        d->image.region[2]=uint16_t((dungeon_hud::source_width*health+99)/100);
        d->image.enabled=health>0;
        const uint8_t normal[]={148,255,102},warning[]={255,205,75},critical[]={255,85,65};
        const auto* color=health<=25?critical:(health<=60?warning:normal);
        for(int k=0;k<3;++k)d->image.color[k]=color[k];
    }
    if(auto* d=health_label.get())number(d->text,"ENERGY ",unsigned(health));
    if(auto* d=key_label.get())d->text.set_text(has_key?"KEY: YES":"KEY: --");
    if(auto* d=objective.get())d->text.set_text(state==Victory?"MISSION COMPLETE":(state==Defeat?"MISSION FAILED":(boss_hud?"DEFEAT THE REACTOR GUARDIAN":(has_key?"UNLOCK THE REACTOR":"FIND THE ACCESS KEY"))));
    if(auto* d=boss_bar.get()){d->progress.enabled=boss_hud;d->progress.value=Fixed(boss_health*4096,Fixed::RAW)/Fixed(280);}
    if(auto* d=boss_label.get())d->text.enabled=boss_hud;
    const bool ended=state==Victory||state==Defeat;
    if(auto* d=message_panel.get())d->image.enabled=ended;
    if(auto* d=message.get()){d->text.enabled=ended;d->text.set_text(state==Victory?"REACTOR SECURED\n\nENTER: PLAY AGAIN":"OUT OF ENERGY");}
}
void DungeonGame::tick(Fixed dt){
    DungeonMeasure measure(5);
    if(!initialized)initialize();if(!initialized||dt.raw()<=0)return;
    if(update_presentation(dt)){
        aiming=false;update_hud();dungeon_telemetry.state=state;dungeon_telemetry.health=health;return;
    }
    if(economy.blocking()){aiming=false;dungeon_telemetry.health=health;return;}
    ++combat_telemetry.ticks;++dungeon_telemetry.ticks;elapsed+=dt;update_visibility();
    if(invulnerable.raw()>0){invulnerable-=dt;if(invulnerable.raw()<0)invulnerable=0.0;}
    if(player_alive()){
        update_pickups(dt);
        if(state==Explore&&has_key)if(auto* p=player.get();p&&p->transform.position[2]>Fixed(53.0)){
            state=BossFight;hud_dirty=true;if(auto* d=boss.get()){d->collider.enabled=true;set_active(d,true);spatial_queries().changed(*d);d->animator.play(0,true);}
        }
        auto& queries=spatial_queries();if(auto* p=player.get())queries.changed(*p);update_doors(dt);update_enemies(dt,queries);ai_elapsed+=dt;
        if(ai_elapsed>=Fixed(1.0/30.0)){const Fixed step=ai_elapsed;ai_elapsed=0.0;update_boss(step,queries);}
        update_shots(dt,&queries);
    }else{
        update_doors(dt);
        update_shots(dt);
        if(state==Victory)if(auto* d=boss.get();d&&!d->animator.playing)set_active(d,false);
        if(input.pressed(Button::Start))reset_run();
    }
    update_hud();dungeon_telemetry.state=state;dungeon_telemetry.health=health;dungeon_telemetry.key=has_key;dungeon_telemetry.kills=kills;dungeon_telemetry.boss_health=boss_health;
}
