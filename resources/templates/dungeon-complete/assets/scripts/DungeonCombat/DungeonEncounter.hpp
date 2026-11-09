#pragma once
#include "DungeonEncounterBase.hpp"
EPOK_SPEED_BEGIN

class EPOK_CLASS(Blueprintable,Id="825024e2-d9f8-5578-8198-bc52b5c217f2") DungeonEncounter:public DungeonEncounterBase {
public:
    epok::HealthState starting_health(bool boss)const override{
        const auto maximum=boss?health_tuning.boss_health:health_tuning.guard_health;
        return epok::HealthLibrary::make_state(maximum,maximum);
    }
    bool should_activate(bool defeated,bool exploring,bool near_home)const override{return !defeated&&exploring&&near_home;}
    bool should_unload(DungeonGuardState state,bool boss_fight,bool near_home)const override{return state.dead_time<=epok::Fixed(0.0)&&(boss_fight||!near_home);}
    DungeonGuardState guard_start(uint32_t encounter) override{
        return {guard_timing.guard_initial_delay,0.0,0.0,0.0,epok::Fixed((encounter%3+1)*137,epok::Fixed::RAW),epok::Fixed((encounter&1)?-68:0,epok::Fixed::RAW),false,false};
    }
    DungeonGuardStep guard_schedule(DungeonGuardState state,epok::Fixed dt) override{
        if(dt<=epok::Fixed(0.0)||dt>epok::Fixed(1.0))return {state,0.0,0};
        if(state.dead_time>epok::Fixed(0.0)){
            state.dead_time-=dt;
            return {state,0.0,state.dead_time<=epok::Fixed(0.0)?6u:0u};
        }
        state.ai_time+=dt;
        if(state.ai_time<guard_timing.ai_interval)return {state,0.0,0};
        dt=state.ai_time;state.ai_time=0.0;state.timer-=dt;
        if(state.flash>epok::Fixed(0.0))state.flash-=dt;
        state.sight_timer-=dt;
        return {state,dt,7};
    }
    DungeonGuardStep guard_decide(DungeonGuardState state,epok::Fixed dt,bool close,bool observed,bool within_range,bool at_patrol_goal) override{
        if(!close)state.can_see=false;
        else if(state.sight_timer<=epok::Fixed(0.0)){state.can_see=observed;state.sight_timer=guard_timing.sight_interval;}
        if(close&&state.can_see){
            if(state.windup>epok::Fixed(0.0)){
                state.windup-=dt;
                if(state.windup<=epok::Fixed(0.0)){state.timer=guard_timing.guard_cooldown;return {state,dt,5};}
                return {state,dt,0};
            }
            if(state.timer<=epok::Fixed(0.0)){state.windup=guard_timing.guard_windup;return {state,dt,4};}
            return {state,dt,within_range?1u:2u};
        }
        state.windup=0.0;if(at_patrol_goal)state.patrol=!state.patrol;
        return {state,dt,3};
    }
    DungeonGuardState guard_hit(DungeonGuardState state,bool depleted) override{
        state.flash=guard_timing.guard_flash;if(depleted)state.dead_time=guard_timing.guard_death;
        return state;
    }
    DungeonBossState boss_start() override{return {boss_timing.boss_initial_delay,0.0,0.0};}
    bool boss_needs_sight(DungeonBossState state,epok::Fixed dt)const override{return state.windup<=epok::Fixed(0.0)&&state.timer-dt<=epok::Fixed(0.0);}
    DungeonBossStep boss_advance(DungeonBossState state,uint32_t health,epok::Fixed dt,bool observed,bool within_range) override{
        const bool fan=health<=health_tuning.boss_phase_health;
        const auto speed=fan?boss_timing.boss_fan_speed:boss_timing.boss_speed;
        if(dt<=epok::Fixed(0.0)||dt>epok::Fixed(1.0))return {state,0,fan,speed};
        state.timer-=dt;if(state.flash>epok::Fixed(0.0))state.flash-=dt;
        if(state.windup>epok::Fixed(0.0)){
            state.windup-=dt;
            if(state.windup<=epok::Fixed(0.0)){state.timer=fan?boss_timing.boss_fan_cooldown:boss_timing.boss_cooldown;return {state,5,fan,speed};}
            return {state,0,fan,speed};
        }
        if(state.timer<=epok::Fixed(0.0)&&observed){state.windup=fan?boss_timing.boss_fan_windup:boss_timing.boss_windup;return {state,4,fan,speed};}
        return {state,within_range?1u:2u,fan,speed};
    }
    DungeonBossState boss_hit(DungeonBossState state) override{
        state.flash=boss_timing.boss_flash;return state;
    }
    epok::ProjectileSpawn3D projectile(epok::GameplayVector3 origin,epok::GameplayVector3 direction,int32_t pellet,bool fan)const override{
        auto angle=epok::MathLibrary::wrap_degrees(shot_tuning.fan_degrees);
        if(pellet<0)angle=-angle;else if(!pellet)angle=0.0;
        const auto cosine=epok::MathLibrary::cosine_degrees(angle),sine=epok::MathLibrary::sine_degrees(angle);
        const auto speed=fan?shot_tuning.fan_speed:shot_tuning.speed;
        const epok::GameplayVector3 velocity{(direction.x*cosine+direction.z*sine)*speed,
            direction.y*speed,(direction.z*cosine-direction.x*sine)*speed};
        return epok::ProjectileLibrary::spawn(origin,velocity,shot_tuning.lifetime,1);
    }
    epok::Fixed shot_size(bool fan)const override{return fan?shot_tuning.fan_size:shot_tuning.size;}
    uint32_t shot_damage(bool boss_fight)const override{return boss_fight?shot_tuning.boss_damage:shot_tuning.damage;}
    uint32_t pellet_count(bool fan)const override{return fan?3u:1u;}
    int32_t pellet_index(uint32_t ordinal,bool fan)const override{
        if(!fan)return 0;if(!ordinal)return -1;if(ordinal==2)return 1;return 0;
    }
    epok::GameplayVector3 shot_target(epok::GameplayVector3 player_position)const override{
        return {player_position.x,player_position.y+shot_tuning.target_height,player_position.z};
    }
    uint32_t world_channel(uint32_t value)const{
        if((value > 255u)){
            return 255u;
        }
        return value;
    }
    bool world_damage_eligible(bool boss,bool alive,bool boss_fight,uint32_t health,bool registered,epok::Fixed dead_time,bool defeated)const override{
        if(!(alive)){
            return false;
        }
        if(boss){
            return (boss_fight && (health > 0u));
        }
        return ((registered && (dead_time <= epok::Fixed(0.0))) && !(defeated));
    }
    uint32_t world_damage_base(uint32_t base)const{
        if((base > 2147483647u)){
            return 2147483647u;
        }
        return base;
    }
    uint32_t world_damage_total(uint32_t base,uint32_t bonus)const override{
        const auto damage_base_1=world_damage_base(base);
        if((bonus > (2147483647u - damage_base_1))){
            return 2147483647u;
        }
        return (damage_base_1 + bonus);
    }
    uint32_t world_reward_count(uint32_t count,uint32_t amount)const override{
        if(((count > 2147483647u) || (amount > (2147483647u - count)))){
            return 2147483647u;
        }
        return (count + amount);
    }
    uint32_t world_hit_sound(bool boss,uint32_t remaining)const{
        if((remaining > 0u)){
            return 2u;
        }
        if(boss){
            return 8u;
        }
        return 4u;
    }
    DungeonWorldHit world_hit_result(bool boss,bool depleted,uint32_t remaining)const override{
        if(boss){
            if(depleted){
                return make_world_DungeonWorldHit(false, true, true, world_damage.boss_victory, world_damage.boss_reset, true, true, world_damage.boss_reward, 8u);
            }
            return make_world_DungeonWorldHit(false, false, false, false, false, false, true, 0u, world_hit_sound(boss, remaining));
        }
        if(depleted){
            return make_world_DungeonWorldHit(true, true, true, false, false, false, true, world_damage.guard_reward, 4u);
        }
        return make_world_DungeonWorldHit(false, false, false, false, false, false, false, 0u, world_hit_sound(boss, remaining));
    }
    DungeonWorldDeath world_player_death(bool depleted)const override{
        if(depleted){
            return make_world_DungeonWorldDeath(world_damage.player_defeat, true, world_damage.player_reset);
        }
        return make_world_DungeonWorldDeath(false, false, false);
    }
    bool world_hide_boss(uint32_t state,bool playing)const override{
        return (((state == 2u) && !(playing)) && world_damage.hide_victorious_boss);
    }
    DungeonWorldColor world_enemy_color(bool boss,epok::Fixed flash,epok::Fixed windup,bool fan)const override{
        if(boss){
            if((flash > epok::Fixed(0.0))){
                return make_world_DungeonWorldColor(world_channel(boss_colors.hit_red), world_channel(boss_colors.hit_green), world_channel(boss_colors.hit_blue));
            }
            if((windup > epok::Fixed(0.0))){
                return make_world_DungeonWorldColor(world_channel(boss_colors.windup_red), world_channel(boss_colors.windup_green), world_channel(boss_colors.windup_blue));
            }
            if(fan){
                return make_world_DungeonWorldColor(world_channel(boss_colors.fan_red), world_channel(boss_colors.fan_green), world_channel(boss_colors.fan_blue));
            }
            return make_world_DungeonWorldColor(world_channel(boss_colors.normal_red), world_channel(boss_colors.normal_green), world_channel(boss_colors.normal_blue));
        }
        if((flash > epok::Fixed(0.0))){
            return make_world_DungeonWorldColor(world_channel(guard_colors.hit_red), world_channel(guard_colors.hit_green), world_channel(guard_colors.hit_blue));
        }
        if((windup > epok::Fixed(0.0))){
            return make_world_DungeonWorldColor(world_channel(guard_colors.windup_red), world_channel(guard_colors.windup_green), world_channel(guard_colors.windup_blue));
        }
        return make_world_DungeonWorldColor(world_channel(guard_colors.normal_red), world_channel(guard_colors.normal_green), world_channel(guard_colors.normal_blue));
    }
    DungeonWorldColor world_shot_color()const override{
        return make_world_DungeonWorldColor(world_channel(enemy_shot_color.red), world_channel(enemy_shot_color.green), world_channel(enemy_shot_color.blue));
    }
    int32_t world_clip_index(int32_t clip)const{
        if((clip < 0)){
            return 0;
        }
        if((clip > 32767)){
            return 32767;
        }
        return clip;
    }
    DungeonWorldClip world_death_animation()const override{
        return make_world_DungeonWorldClip(world_clip_index(world_animation.death_clip), true, false, false);
    }
    DungeonWorldClip world_enemy_animation(bool boss,uint32_t action,int32_t current)const override{
        if((action == 4u)){
            return make_world_DungeonWorldClip(world_clip_index(world_animation.idle_clip), true, world_animation.loop, boss);
        }
        if(((action == 2u) || (!(boss) && (action == 3u)))){
            const auto clip_index_1=world_clip_index(world_animation.move_clip);
            return make_world_DungeonWorldClip(clip_index_1, (current != clip_index_1), world_animation.loop, false);
        }
        if((action == 1u)){
            const auto clip_index_2=world_clip_index(world_animation.idle_clip);
            return make_world_DungeonWorldClip(clip_index_2, (current != clip_index_2), world_animation.loop, false);
        }
        return make_world_DungeonWorldClip(-1, false, world_animation.loop, false);
    }
    bool world_door_open(uint32_t index,bool locked,bool has_key,uint32_t state,epok::Fixed progress,bool near_open,bool near_safety)const override{
        if((((index == 3u) && (state == 1u)) && ((progress > epok::Fixed(0.0)) && near_safety))){
            return true;
        }
        if(((index == 3u) && (state == 1u))){
            return false;
        }
        if((((!(locked) || has_key) || (state == 2u)) && near_open)){
            return true;
        }
        return ((progress > epok::Fixed(0.0)) && near_safety);
    }
    epok::Fixed world_door_radius(bool safety)const override{
        if(safety){
            return epok::MathLibrary::clamp(world_doors.safety_radius, epok::Fixed(0.0), epok::Fixed(64.0));
        }
        return epok::MathLibrary::clamp(world_doors.open_radius, epok::Fixed(0.0), epok::Fixed(64.0));
    }
    epok::Fixed world_door_progress(epok::Fixed progress,bool opening,epok::Fixed dt)const{
        if(opening){
            return epok::MathLibrary::clamp((epok::MathLibrary::clamp(progress, epok::Fixed(0.0), epok::Fixed(1.0)) + (epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0)) * epok::MathLibrary::clamp(world_doors.speed, epok::Fixed(0.0), epok::Fixed(16.0)))), epok::Fixed(0.0), epok::Fixed(1.0));
        }
        return epok::MathLibrary::clamp((epok::MathLibrary::clamp(progress, epok::Fixed(0.0), epok::Fixed(1.0)) - (epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0)) * epok::MathLibrary::clamp(world_doors.speed, epok::Fixed(0.0), epok::Fixed(16.0)))), epok::Fixed(0.0), epok::Fixed(1.0));
    }
    DungeonWorldDoor world_door_step(uint32_t index,bool locked,bool has_key,uint32_t state,epok::Fixed progress,bool near_open,bool near_safety,epok::Fixed dt)const override{
        const auto door_open_2=world_door_open(index, locked, has_key, state, progress, near_open, near_safety);
        const auto door_progress_1=world_door_progress(progress, door_open_2, dt);
        if(((index == 3u) && !(has_key))){
            return make_world_DungeonWorldDoor(door_progress_1, door_open_2, (progress != door_progress_1), ((progress < epok::Fixed(1.0)) && (door_progress_1 == epok::Fixed(1.0))), ((index == 3u) && !(has_key)), (epok::MathLibrary::clamp(world_doors.closed_offset, epok::Fixed(0.0), epok::Fixed(8.0)) + (((door_progress_1 * door_progress_1) * (epok::Fixed(3.0) - (epok::Fixed(2.0) * door_progress_1))) * epok::MathLibrary::clamp(world_doors.travel, epok::Fixed(0.0), epok::Fixed(8.0)))), make_world_DungeonWorldColor(world_channel(world_doors.locked_red), world_channel(world_doors.locked_green), world_channel(world_doors.locked_blue)));
        }
        return make_world_DungeonWorldDoor(door_progress_1, door_open_2, (progress != door_progress_1), ((progress < epok::Fixed(1.0)) && (door_progress_1 == epok::Fixed(1.0))), ((index == 3u) && !(has_key)), (epok::MathLibrary::clamp(world_doors.closed_offset, epok::Fixed(0.0), epok::Fixed(8.0)) + (((door_progress_1 * door_progress_1) * (epok::Fixed(3.0) - (epok::Fixed(2.0) * door_progress_1))) * epok::MathLibrary::clamp(world_doors.travel, epok::Fixed(0.0), epok::Fixed(8.0)))), make_world_DungeonWorldColor(255u, 255u, 255u));
    }
    bool world_pickup_visible(uint32_t kind,epok::GameplayVector3 position)const override{
        if(((kind == 0u) || (kind == 2u))){
            return ((position).x < epok::MathLibrary::clamp(world_pickups.left_x, epok::Fixed(-4096.0), epok::Fixed(4096.0)));
        }
        if((kind == 1u)){
            return ((position).x > epok::MathLibrary::clamp(world_pickups.right_x, epok::Fixed(-4096.0), epok::Fixed(4096.0)));
        }
        return (((position).z > epok::MathLibrary::clamp(world_pickups.minimum_z, epok::Fixed(-4096.0), epok::Fixed(4096.0))) && ((position).z < epok::MathLibrary::clamp(world_pickups.maximum_z, epok::Fixed(-4096.0), epok::Fixed(4096.0))));
    }
    bool world_pickup_eligible(bool can_collect,bool nearby,epok::Fixed height)const override{
        return ((can_collect && nearby) && (height < epok::MathLibrary::clamp(world_pickups.maximum_y, epok::Fixed(-32.0), epok::Fixed(32.0))));
    }
    epok::Fixed world_pickup_radius()const override{
        return epok::MathLibrary::clamp(world_pickups.radius, epok::Fixed(0.0), epok::Fixed(64.0));
    }
    uint32_t world_pickup_heal()const override{
        return world_damage_base(world_pickups.heal);
    }
    epok::Fixed world_pickup_bob(epok::Fixed elapsed)const override{
        return (epok::MathLibrary::sine_degrees((epok::MathLibrary::wrap_degrees(elapsed) * epok::MathLibrary::floor(epok::MathLibrary::clamp(world_pickups.bob_rate, epok::Fixed(0.0), epok::Fixed(720.0))))) * epok::MathLibrary::clamp(world_pickups.bob_extent, epok::Fixed(0.0), epok::Fixed(8.0)));
    }
    DungeonWorldDirection world_direction(epok::GameplayVector3 delta)const{
        if((((false || (((delta).x < epok::Fixed(-64.0)) || ((delta).x > epok::Fixed(64.0)))) || (((delta).y < epok::Fixed(-64.0)) || ((delta).y > epok::Fixed(64.0)))) || (((delta).z < epok::Fixed(-64.0)) || ((delta).z > epok::Fixed(64.0))))){
            return make_world_DungeonWorldDirection(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), false);
        }
        const auto length_2=epok::MathLibrary::length(delta);
        if((length_2 >= epok::Fixed(64.0))){
            return make_world_DungeonWorldDirection(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), false);
        }
        const auto quotient_1=(epok::MathLibrary::floor((length_2 * epok::Fixed(1024.0))) / epok::Fixed(1024.0));
        if((quotient_1 > epok::Fixed(0.0))){
            return make_world_DungeonWorldDirection(epok::MathLibrary::vector3(((delta).x / quotient_1), ((delta).y / quotient_1), ((delta).z / quotient_1)), true);
        }
        return make_world_DungeonWorldDirection(delta, true);
    }
    DungeonWorldDirection world_aim_direction(epok::GameplayVector3 delta)const override{
        return world_direction(delta);
    }
    epok::Fixed world_patrol_x(epok::Fixed home,bool side)const{
        if(side){
            return (home + epok::MathLibrary::clamp(movement.patrol_extent, epok::Fixed(0.0), epok::Fixed(16.0)));
        }
        return (home - epok::MathLibrary::clamp(movement.patrol_extent, epok::Fixed(0.0), epok::Fixed(16.0)));
    }
    epok::GameplayVector3 world_patrol_goal(epok::GameplayVector3 home,bool patrol_side)const override{
        return epok::MathLibrary::vector3(world_patrol_x((home).x, patrol_side), (home).y, (home).z);
    }
    epok::GameplayVector3 world_motion_goal(bool boss,uint32_t action,epok::GameplayVector3 origin,epok::GameplayVector3 player,epok::GameplayVector3 home,bool patrol_side)const{
        if((!(boss) && (action == 3u))){
            return epok::MathLibrary::vector3(world_patrol_x((home).x, patrol_side), (home).y, (home).z);
        }
        return epok::MathLibrary::vector3((player).x, (origin).y, (player).z);
    }
    epok::Fixed world_motion_speed(bool boss,uint32_t action,epok::Fixed boss_speed)const{
        if(boss){
            return epok::MathLibrary::clamp(boss_speed, epok::Fixed(0.0), epok::Fixed(16.0));
        }
        if((action == 3u)){
            return epok::MathLibrary::clamp(movement.patrol_speed, epok::Fixed(0.0), epok::Fixed(16.0));
        }
        return epok::MathLibrary::clamp(movement.chase_speed, epok::Fixed(0.0), epok::Fixed(16.0));
    }
    DungeonWorldMotion world_enemy_motion(bool boss,uint32_t action,bool patrol_side,bool close,bool seen,epok::GameplayVector3 origin,epok::GameplayVector3 player,epok::GameplayVector3 home,epok::Fixed dt,epok::Fixed boss_speed)const override{
        if(((action == 2u) || (!(boss) && (action == 3u)))){
            const auto motion_goal_2=world_motion_goal(boss, action, origin, player, home, patrol_side);
            const auto direction_1=world_direction(epok::MathLibrary::vector3(((motion_goal_2).x - (origin).x), ((motion_goal_2).y - (origin).y), ((motion_goal_2).z - (origin).z)));
            if(!((direction_1).valid)){
                return make_world_DungeonWorldMotion(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), epok::MathLibrary::heading_degrees(epok::MathLibrary::clamp(((player).x - (origin).x), epok::Fixed(-64.0), epok::Fixed(64.0)), epok::MathLibrary::clamp(((player).z - (origin).z), epok::Fixed(-64.0), epok::Fixed(64.0))), epok::Fixed(0.0), false, (boss || (close && seen)), false, false);
            }
            const auto motion_speed_4=world_motion_speed(boss, action, boss_speed);
            const auto clamp_3=epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0));
            if((!(boss) && (action == 3u))){
                return make_world_DungeonWorldMotion(epok::MathLibrary::vector3(((((direction_1).value).x * motion_speed_4) * clamp_3), epok::MathLibrary::clamp(world_motion.ground_step, epok::Fixed(-1.0), epok::Fixed(0.0)), ((((direction_1).value).z * motion_speed_4) * clamp_3)), epok::MathLibrary::heading_degrees(epok::MathLibrary::clamp(((player).x - (origin).x), epok::Fixed(-64.0), epok::Fixed(64.0)), epok::MathLibrary::clamp(((player).z - (origin).z), epok::Fixed(-64.0), epok::Fixed(64.0))), epok::MathLibrary::heading_degrees(((direction_1).value).x, ((direction_1).value).z), true, (boss || (close && seen)), true, true);
            }
            return make_world_DungeonWorldMotion(epok::MathLibrary::vector3(((((direction_1).value).x * motion_speed_4) * clamp_3), epok::MathLibrary::clamp(world_motion.ground_step, epok::Fixed(-1.0), epok::Fixed(0.0)), ((((direction_1).value).z * motion_speed_4) * clamp_3)), epok::MathLibrary::heading_degrees(epok::MathLibrary::clamp(((player).x - (origin).x), epok::Fixed(-64.0), epok::Fixed(64.0)), epok::MathLibrary::clamp(((player).z - (origin).z), epok::Fixed(-64.0), epok::Fixed(64.0))), epok::Fixed(0.0), true, (boss || (close && seen)), false, true);
        }
        return make_world_DungeonWorldMotion(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), epok::MathLibrary::heading_degrees(epok::MathLibrary::clamp(((player).x - (origin).x), epok::Fixed(-64.0), epok::Fixed(64.0)), epok::MathLibrary::clamp(((player).z - (origin).z), epok::Fixed(-64.0), epok::Fixed(64.0))), epok::Fixed(0.0), false, (boss || (close && seen)), false, true);
    }
    bool world_boss_due(epok::Fixed elapsed)const override{
        return (elapsed >= epok::MathLibrary::clamp(world_motion.boss_interval, epok::Fixed(0.000244140625), epok::Fixed(1.0)));
    }
};
EPOK_SPEED_END
