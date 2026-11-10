#define EPOK_WORLD_GAMEPLAY 1
#include "actor_scene_fixture.hpp"
#ifndef DRAFT_ENCOUNTER_HEADER
#define DRAFT_ENCOUNTER_HEADER "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonEncounter.hpp"
#endif
#include DRAFT_ENCOUNTER_HEADER
#include "CharacterMotion.hpp"
#include "DungeonProfiling.hpp"
#include <cstdio>
using namespace epok;
Fixed clamp01(Fixed v){return v<Fixed(0.0)?Fixed(0.0):(v>Fixed(1.0)?Fixed(1.0):v);}
int64_t distance2(const Fixed* a,const Fixed* b){const int64_t x=(a[0]-b[0]).raw(),z=(a[2]-b[2]).raw();return x*x+z*z;}
bool near(const Fixed* a,const Fixed* b,Fixed r){return distance2(a,b)<=int64_t(r.raw())*r.raw();}
void direction(const Fixed* from,const Fixed* to,Fixed* out){
    int64_t n=0;for(int k=0;k<3;++k){out[k]=to[k]-from[k];n+=int64_t(out[k].raw())*out[k].raw();}
    // Decisions and projectiles are limited to a local 22-metre neighbourhood.
    const Fixed length(int32_t(character_motion::root(uint32_t(n>>4)))*4,Fixed::RAW);
    if(length.raw())for(int k=0;k<3;++k)out[k]/=length;
}
void tint(ActorData* d,int r,int g,int b){if(d){d->material.color[0]=uint8_t(r);d->material.color[1]=uint8_t(g);d->material.color[2]=uint8_t(b);}}
struct {uint32_t doors_opened=0;} dungeon_telemetry;
struct Reference {ActorData* data=nullptr;ActorData* get()const{return data;}};
struct Doors {
    enum RunState {Explore,BossFight,Victory,Defeat};RunState state=Explore;bool has_key=false;
    struct Door {Reference left{},right{};Fixed center[3]{};Fixed progress=0.0;uint8_t axis=0,tint_state=255;bool locked=false;};
    Door doors[4]{};Reference player{};uint32_t spatial_revision=1,structural_revision=1;
    struct Query {uint32_t changes=0;void changed(ActorData&){++changes;}} query;
    DungeonEncounter policy;
    DungeonEncounter* encounter_rules(){return &policy;}Query& spatial_queries(){return query;}
};
struct LegacyDoors:Doors {void update_doors(Fixed dt);};
struct CandidateDoors:Doors {void update_doors(Fixed dt);};
void LegacyDoors::update_doors(Fixed dt){
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
void CandidateDoors::update_doors(Fixed dt){
    DungeonMeasure measure(4);auto* player_data=player.get();auto* rules=encounter_rules();if(!player_data||!rules)return;
    for(unsigned index=0;index<4;++index){auto& door=doors[index];
        const auto step=rules->world_door_step(index,door.locked,has_key,uint32_t(state),door.progress,
            near(player_data->transform.position,door.center,rules->world_door_radius(false)),
            near(player_data->transform.position,door.center,rules->world_door_radius(true)),dt);
        door.progress=step.progress;if(step.moved){++spatial_revision;++structural_revision;}if(step.opened)++dungeon_telemetry.doors_opened;
        door.tint_state=uint8_t(step.locked_tint);
        for(unsigned side=0;side<2;++side)if(auto* data=(side?door.right:door.left).get()){
            data->transform.position[door.axis]=door.center[door.axis]+(side?step.offset:-step.offset);
            if(step.moved)spatial_queries().changed(*data);tint(data,int(step.color.red),int(step.color.green),int(step.color.blue));
        }
    }
}
static void same(Fixed a,Fixed b){if(a!=b){std::fprintf(stderr,"Q12 mismatch %d != %d\n",a.raw(),b.raw());assert(false);}}
static void same(GameplayVector3 a,GameplayVector3 b){same(a.x,b.x);same(a.y,b.y);same(a.z,b.z);}
static Fixed legacy_heading(Fixed x,Fixed z){return Fixed(character_motion::heading(x.raw(),z.raw()),Fixed::RAW);}
int main(){
    DungeonEncounter rules;uint32_t checks=0;
    static_assert(sizeof(DungeonWorldHit)<=9*4&&sizeof(DungeonWorldDoor)<=9*4&&sizeof(DungeonWorldMotion)<=9*4);
    for(uint32_t base:{0u,1u,20u,0x7ffffffeu,0x7fffffffu,0x80000000u,UINT32_MAX})for(uint32_t bonus:{0u,1u,8u,0x7fffffffu,0x80000000u,UINT32_MAX}){
        const int amount=int(base>INT32_MAX?INT32_MAX:base);const int64_t total=int64_t(amount)+int64_t(bonus);
        const auto expected=uint32_t(total>INT32_MAX?INT32_MAX:total<INT32_MIN?INT32_MIN:total);
        assert(rules.world_damage_total(base,bonus)==expected);++checks;
    }
    for(uint32_t mask=0;mask<64;++mask)for(Fixed dead:{Fixed(-1.0),Fixed(0.0),Fixed(.85)})for(uint32_t hp:{0u,1u,UINT32_MAX}){
        const bool boss=mask&1,alive=mask&2,fight=mask&4,registered=mask&8,defeated=mask&16;
        assert(rules.world_damage_eligible(boss,alive,fight,hp,registered,dead,defeated)==(alive&&(boss?(fight&&hp>0):(registered&&dead<=Fixed(0.0)&&!defeated))));++checks;
    }
    for(bool boss:{false,true})for(bool depleted:{false,true})for(uint32_t left:{0u,1u,60u}){
        const auto hit=rules.world_hit_result(boss,depleted,left);
        assert(hit.defeated==(!boss&&depleted)&&hit.disable_collider==depleted&&hit.death_clip==depleted);
        assert(hit.victory==(boss&&depleted)&&hit.reset_projectiles==(boss&&depleted)&&hit.explosion==(boss&&depleted));
        assert(hit.hud_dirty==(boss||depleted)&&hit.reward==(!boss&&depleted?1u:0u));
        assert(hit.sound==(depleted?(boss?8u:4u):(left?2u:boss?8u:4u)));++checks;
    }
    assert(rules.world_player_death(true).defeat&&rules.world_player_death(true).clear_aim&&rules.world_player_death(true).reset_projectiles);
    assert(!rules.world_player_death(false).defeat&&rules.world_hide_boss(2,false)&&!rules.world_hide_boss(2,true));checks+=2;
    for(bool boss:{false,true})for(bool flash:{false,true})for(bool windup:{false,true})for(bool fan:{false,true}){
        const auto color=rules.world_enemy_color(boss,flash?Fixed(.1):Fixed(0.0),windup?Fixed(.1):Fixed(0.0),fan);
        const uint32_t r=boss?(flash?255:windup?255:255):(flash?255:windup?255:100);
        const uint32_t g=boss?(flash?90:windup?150:fan?190:255):(flash?110:windup?180:210);
        const uint32_t b=boss?(flash?90:windup?55:fan?150:255):(flash?110:windup?85:255);
        assert(color.red==r&&color.green==g&&color.blue==b);++checks;
    }
    const auto shot=rules.world_shot_color();assert(shot.red==255&&shot.green==100&&shot.blue==50);++checks;
    for(bool boss:{false,true})for(uint32_t action=0;action<8;++action)for(int current:{-1,0,1,2}){
        const auto animation=rules.world_enemy_animation(boss,action,current);
        const int clip=action==4||action==1?0:(action==2||(!boss&&action==3))?1:-1;
        const bool play=action==4||(clip>=0&&current!=clip);
        assert(animation.play==play&&animation.clip==clip&&animation.loop&&animation.charge_cue==(boss&&action==4));++checks;
    }
    const auto death=rules.world_death_animation();assert(death.clip==2&&death.play&&!death.loop);++checks;
    // Exact frozen body of update_doors, including boss safety reopening and
    // the original material/cache behavior, versus the new native adapter body.
    for(uint32_t sample=0;sample<2048;++sample){
        ActorData legacy_actors[9]{},candidate_actors[9]{};LegacyDoors old;CandidateDoors next;
        old.player.data=&legacy_actors[0];next.player.data=&candidate_actors[0];old.has_key=next.has_key=sample&1;
        old.state=Doors::RunState(sample%4);next.state=old.state;
        for(unsigned k=0;k<3;++k){const Fixed value(int32_t((sample>>(k+1))%13)*4096,Fixed::RAW);legacy_actors[0].transform.position[k]=candidate_actors[0].transform.position[k]=value;}
        for(uint32_t door=0;door<4;++door){
            old.doors[door].left.data=&legacy_actors[1+door*2];old.doors[door].right.data=&legacy_actors[2+door*2];
            next.doors[door].left.data=&candidate_actors[1+door*2];next.doors[door].right.data=&candidate_actors[2+door*2];
            old.doors[door].locked=next.doors[door].locked=(sample>>(door+2))&1;
            old.doors[door].axis=next.doors[door].axis=door%2?2:0;
            old.doors[door].progress=next.doors[door].progress=Fixed(int32_t((sample+door*17)%4097),Fixed::RAW);
        }
        const Fixed dt(137,Fixed::RAW);dungeon_telemetry.doors_opened=0;old.update_doors(dt);const auto opened=dungeon_telemetry.doors_opened;
        dungeon_telemetry.doors_opened=0;next.update_doors(dt);assert(dungeon_telemetry.doors_opened==opened);
        assert(old.query.changes==next.query.changes&&old.spatial_revision==next.spatial_revision&&old.structural_revision==next.structural_revision);
        for(uint32_t door=0;door<4;++door){same(old.doors[door].progress,next.doors[door].progress);assert(old.doors[door].tint_state==next.doors[door].tint_state);}
        for(uint32_t actor=1;actor<9;++actor)for(uint32_t k=0;k<3;++k){same(legacy_actors[actor].transform.position[k],candidate_actors[actor].transform.position[k]);assert(legacy_actors[actor].material.color[k]==candidate_actors[actor].material.color[k]);}
        ++checks;
    }
    for(uint32_t frame=0;frame<100000;++frame){const Fixed elapsed(int32_t(frame*137),Fixed::RAW);same(rules.world_pickup_bob(elapsed),sin_degrees(elapsed*Fixed(120.0))*Fixed(.12));++checks;}
    for(uint32_t kind=0;kind<4;++kind)for(int32_t x=-10;x<=10;++x)for(int32_t z=30;z<=57;++z){
        const GameplayVector3 position{Fixed(x*4096,Fixed::RAW),0.0,Fixed(z*4096,Fixed::RAW)};
        const bool expected=kind==0||kind==2?x<-8:kind==1?x>8:z>32&&z<55;
        assert(rules.world_pickup_visible(kind,position)==expected);++checks;
    }
    for(bool can:{false,true})for(bool nearby:{false,true})for(Fixed height:{Fixed(-1.0),Fixed(0.0),Fixed(2.2),Fixed(3.0)}){
        assert(rules.world_pickup_eligible(can,nearby,height)==(can&&nearby&&height<Fixed(2.2)));++checks;
    }
    assert(rules.world_pickup_heal()==35&&rules.world_pickup_radius()==1.0);++checks;
    assert(!rules.world_boss_due(Fixed(136,Fixed::RAW))&&rules.world_boss_due(Fixed(137,Fixed::RAW)));++checks;
    assert(rules.world_patrol_goal({2.0,0.0,3.0},false).x==.75&&rules.world_patrol_goal({2.0,0.0,3.0},true).x==3.25);++checks;
    for(bool boss:{false,true})for(uint32_t action=0;action<7;++action)for(bool side:{false,true})for(bool close:{false,true})for(bool seen:{false,true}){
        const GameplayVector3 from{1.0,0.0,2.0},player{4.0,3.0,9.0},home{2.0,0.0,3.0};const Fixed dt(137,Fixed::RAW);
        const auto motion=rules.world_enemy_motion(boss,action,side,close,seen,from,player,home,dt,.85);
        const bool moving=action==2||(!boss&&action==3),patrol=!boss&&action==3;
        assert(motion.move==moving&&motion.turn==(boss||(close&&seen))&&motion.patrol_turn==patrol&&motion.valid);
        same(motion.yaw,legacy_heading(player.x-from.x,player.z-from.z));
        if(moving){Fixed origin[]={from.x,from.y,from.z},goal[]={player.x,from.y,player.z},dir[3];
            if(patrol){goal[0]=home.x+(side?Fixed(1.25):Fixed(-1.25));goal[1]=home.y;goal[2]=home.z;}
            direction(origin,goal,dir);const Fixed speed=boss?Fixed(.85):patrol?Fixed(.6):Fixed(1.05);
            same(motion.delta,{dir[0]*speed*dt,Fixed(-.05),dir[2]*speed*dt});if(patrol)same(motion.patrol_yaw,legacy_heading(dir[0],dir[2]));}
        ++checks;
    }
    // Editable rules and bounded malformed inputs, separate from legacy claims.
    rules.world_damage.guard_reward=7;assert(rules.world_hit_result(false,true,0).reward==7&&rules.world_reward_count(5,7)==12);
    assert(rules.world_reward_count(INT32_MAX,UINT32_MAX)==INT32_MAX);checks+=2;
    rules.world_damage.boss_victory=false;assert(!rules.world_hit_result(true,true,0).victory);++checks;
    rules.guard_colors.normal_red=UINT32_MAX;rules.boss_colors.fan_green=UINT32_MAX;assert(rules.world_enemy_color(false,0.0,0.0,false).red==255&&rules.world_enemy_color(true,0.0,0.0,true).green==255);++checks;
    rules.world_animation.idle_clip=12;rules.world_animation.move_clip=18;rules.world_animation.death_clip=7;assert(rules.world_enemy_animation(false,4,-1).clip==12&&rules.world_enemy_animation(true,2,0).clip==18&&rules.world_death_animation().clip==7);++checks;
    rules.world_doors.speed=2.0;rules.world_doors.travel=3.0;const auto door=rules.world_door_step(0,false,false,0,0.0,true,false,.5);assert(door.progress==1.0&&door.offset==3.75);++checks;
    rules.world_doors.speed=Fixed(INT32_MAX,Fixed::RAW);rules.world_doors.closed_offset=Fixed(INT32_MAX,Fixed::RAW);rules.world_doors.travel=Fixed(INT32_MAX,Fixed::RAW);
    const auto limited_door=rules.world_door_step(0,false,false,0,Fixed(INT32_MAX,Fixed::RAW),true,false,Fixed(INT32_MAX,Fixed::RAW));assert(limited_door.progress==1.0&&limited_door.offset==16.0);++checks;
    rules.world_pickups.heal=80;rules.world_pickups.radius=2.0;rules.world_pickups.maximum_y=4.0;assert(rules.world_pickup_heal()==80&&rules.world_pickup_radius()==2.0&&rules.world_pickup_eligible(true,true,3.0));++checks;
    rules.world_pickups.heal=UINT32_MAX;rules.world_pickups.bob_rate=Fixed(INT32_MAX,Fixed::RAW);rules.world_pickups.bob_extent=Fixed(INT32_MAX,Fixed::RAW);
    assert(rules.world_pickup_heal()==INT32_MAX);const auto bounded_bob=rules.world_pickup_bob(Fixed(INT32_MAX,Fixed::RAW));assert(bounded_bob>=-8.0&&bounded_bob<=8.0);checks+=2;
    for(GameplayVector3 delta: {GameplayVector3{64.0,0.0,0.0},GameplayVector3{Fixed(INT32_MAX,Fixed::RAW),0.0,0.0}})assert(!rules.world_aim_direction(delta).valid);checks+=2;
    std::printf("PASS %u Game A/B closure checkpoints; frozen door bodies/geometry, damage saturation, consequences, presentation, pickups and motion\n",checks);
}
