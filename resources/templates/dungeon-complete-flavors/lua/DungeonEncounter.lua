---@class DungeonEncounter : DungeonEncounterBase
local DungeonEncounter = DungeonEncounterBase:extend()

---@override
function DungeonEncounter:starting_health(boss)
    local maximum = self.health_tuning.guard_health
    if boss then maximum = self.health_tuning.boss_health end
    return epok.combat.make_state(maximum, maximum)
end

---@override
function DungeonEncounter:should_activate(defeated, exploring, near_home)
    return not defeated and exploring and near_home
end

---@override
function DungeonEncounter:should_unload(state, boss_fight, near_home)
    return state.dead_time <= 0.0 and (boss_fight or not near_home)
end

---@override
function DungeonEncounter:guard_start(encounter)
    ---@type Fixed
    local sight = 137.0 / 4096.0
    if encounter % 3 == 1 then sight = 274.0 / 4096.0 end
    if encounter % 3 == 2 then sight = 411.0 / 4096.0 end
    ---@type Fixed
    local phase = 0.0
    if encounter % 2 == 1 then phase = -68.0 / 4096.0 end
    return self:make_guard(self.guard_timing.guard_initial_delay, 0.0, 0.0, 0.0, sight, phase, false, false)
end

---@override
function DungeonEncounter:guard_schedule(state, dt)
    if dt <= 0.0 or dt > 1.0 then return self:make_guard_step(state, 0.0, 0) end
    local s = state
    if s.dead_time > 0.0 then
        s.dead_time = s.dead_time - dt
        if s.dead_time <= 0.0 then return self:make_guard_step(s, 0.0, 6) end
        return self:make_guard_step(s, 0.0, 0)
    end
    s.ai_time = s.ai_time + dt
    if s.ai_time < self.guard_timing.ai_interval then return self:make_guard_step(s, 0.0, 0) end
    local elapsed = s.ai_time
    s.ai_time = 0.0
    s.timer = s.timer - elapsed
    if s.flash > 0.0 then s.flash = s.flash - elapsed end
    s.sight_timer = s.sight_timer - elapsed
    return self:make_guard_step(s, elapsed, 7)
end

---@override
function DungeonEncounter:guard_decide(state, dt, close, observed, within_range, at_patrol_goal)
    local s = state
    if not close then
        s.can_see = false
    elseif s.sight_timer <= 0.0 then
        s.can_see = observed
        s.sight_timer = self.guard_timing.sight_interval
    end
    if close and s.can_see then
        if s.windup > 0.0 then
            s.windup = s.windup - dt
            if s.windup <= 0.0 then
                s.timer = self.guard_timing.guard_cooldown
                return self:make_guard_step(s, dt, 5)
            end
            return self:make_guard_step(s, dt, 0)
        end
        if s.timer <= 0.0 then
            s.windup = self.guard_timing.guard_windup
            return self:make_guard_step(s, dt, 4)
        end
        if within_range then return self:make_guard_step(s, dt, 1) end
        return self:make_guard_step(s, dt, 2)
    end
    s.windup = 0.0
    if at_patrol_goal then s.patrol = not s.patrol end
    return self:make_guard_step(s, dt, 3)
end

---@override
function DungeonEncounter:guard_hit(state, depleted)
    local s = state
    s.flash = self.guard_timing.guard_flash
    if depleted then s.dead_time = self.guard_timing.guard_death end
    return s
end

---@override
function DungeonEncounter:boss_start()
    return self:make_boss(self.boss_timing.boss_initial_delay, 0.0, 0.0)
end

---@override
function DungeonEncounter:boss_needs_sight(state, dt)
    return state.windup <= 0.0 and state.timer - dt <= 0.0
end

---@override
function DungeonEncounter:boss_advance(state, health, dt, observed, within_range)
    local s = state
    local fan = health <= self.health_tuning.boss_phase_health
    local speed = self.boss_timing.boss_speed
    if fan then speed = self.boss_timing.boss_fan_speed end
    if dt <= 0.0 or dt > 1.0 then return self:make_boss_step(s, 0, fan, speed) end
    s.timer = s.timer - dt
    if s.flash > 0.0 then s.flash = s.flash - dt end
    if s.windup > 0.0 then
        s.windup = s.windup - dt
        if s.windup <= 0.0 then
            s.timer = self.boss_timing.boss_cooldown
            if fan then s.timer = self.boss_timing.boss_fan_cooldown end
            return self:make_boss_step(s, 5, fan, speed)
        end
        return self:make_boss_step(s, 0, fan, speed)
    end
    if s.timer <= 0.0 and observed then
        s.windup = self.boss_timing.boss_windup
        if fan then s.windup = self.boss_timing.boss_fan_windup end
        return self:make_boss_step(s, 4, fan, speed)
    end
    if within_range then return self:make_boss_step(s, 1, fan, speed) end
    return self:make_boss_step(s, 2, fan, speed)
end

---@override
function DungeonEncounter:boss_hit(state)
    local s = state
    s.flash = self.boss_timing.boss_flash
    return s
end

---@override
function DungeonEncounter:projectile(origin, direction, pellet, fan)
    local t = self.shot_tuning
    local angle = epok.math.wrap_degrees(t.fan_degrees)
    if pellet < 0 then angle = -angle elseif pellet == 0 then angle = 0.0 end
    local cosine = epok.math.cosine_degrees(angle)
    local sine = epok.math.sine_degrees(angle)
    local speed = t.speed
    if fan then speed = t.fan_speed end
    local velocity = epok.math.vector3((direction.x * cosine + direction.z * sine) * speed,
        direction.y * speed, (direction.z * cosine - direction.x * sine) * speed)
    return epok.projectiles.spawn(origin, velocity, t.lifetime, 1)
end

---@override
function DungeonEncounter:shot_size(fan)
    if fan then return self.shot_tuning.fan_size end
    return self.shot_tuning.size
end

---@override
function DungeonEncounter:shot_damage(boss_fight)
    if boss_fight then return self.shot_tuning.boss_damage end
    return self.shot_tuning.damage
end

---@override
function DungeonEncounter:pellet_count(fan)
    if fan then return 3 end
    return 1
end

---@override
function DungeonEncounter:pellet_index(ordinal, fan)
    if not fan then return epok.to_int(0.0) end
    if ordinal == 0 then return epok.to_int(-1.0) end
    if ordinal == 2 then return epok.to_int(1.0) end
    return epok.to_int(0.0)
end

---@override
function DungeonEncounter:shot_target(player_position)
    return epok.math.vector3(player_position.x, player_position.y + self.shot_tuning.target_height, player_position.z)
end

-- Generated authored rules: DungeonEncounter
---@override
function DungeonEncounter:world_damage_eligible(boss, alive, boss_fight, health, registered, dead_time, defeated)
    if not (alive) then
        return false
    end
    if boss then
        return (boss_fight and (health > 0))
    end
    return ((registered and (dead_time <= 0.0)) and not (defeated))
end

---@override
function DungeonEncounter:world_damage_total(base, bonus)
    local rule2 = epok.math.truncate_unsigned(0.0)
    if (base > 2147483647) then
        rule2 = 2147483647
    else
        rule2 = base
    end
    local damage_base_1_1 = rule2
    if (bonus > (2147483647 - damage_base_1_1)) then
        return 2147483647
    end
    return (damage_base_1_1 + bonus)
end

---@override
function DungeonEncounter:world_reward_count(count, amount)
    if ((count > 2147483647) or (amount > (2147483647 - count))) then
        return 2147483647
    end
    return (count + amount)
end

---@override
function DungeonEncounter:world_hit_result(boss, depleted, remaining)
    if boss then
        if depleted then
            return self:make_world_DungeonWorldHit(false, true, true, self.world_damage.boss_victory, self.world_damage.boss_reset, true, true, self.world_damage.boss_reward, 8)
        end
        local rule1 = epok.math.truncate_unsigned(0.0)
        if (remaining > 0) then
            rule1 = 2
        else
            if boss then
                rule1 = 8
            else
                rule1 = 4
            end
        end
        return self:make_world_DungeonWorldHit(false, false, false, false, false, false, true, 0, rule1)
    end
    if depleted then
        return self:make_world_DungeonWorldHit(true, true, true, false, false, false, true, self.world_damage.guard_reward, 4)
    end
    local rule2 = epok.math.truncate_unsigned(0.0)
    if (remaining > 0) then
        rule2 = 2
    else
        if boss then
            rule2 = 8
        else
            rule2 = 4
        end
    end
    return self:make_world_DungeonWorldHit(false, false, false, false, false, false, false, 0, rule2)
end

---@override
function DungeonEncounter:world_player_death(depleted)
    if depleted then
        return self:make_world_DungeonWorldDeath(self.world_damage.player_defeat, true, self.world_damage.player_reset)
    end
    return self:make_world_DungeonWorldDeath(false, false, false)
end

---@override
function DungeonEncounter:world_hide_boss(state, playing)
    return (((state == 2) and not (playing)) and self.world_damage.hide_victorious_boss)
end

---@override
function DungeonEncounter:world_enemy_color(boss, flash, windup, fan)
    if boss then
        if (flash > 0.0) then
            local rule1 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.hit_red > 255) then
                rule1 = 255
            else
                rule1 = self.boss_colors.hit_red
            end
            local rule2 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.hit_green > 255) then
                rule2 = 255
            else
                rule2 = self.boss_colors.hit_green
            end
            local rule3 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.hit_blue > 255) then
                rule3 = 255
            else
                rule3 = self.boss_colors.hit_blue
            end
            return self:make_world_DungeonWorldColor(rule1, rule2, rule3)
        end
        if (windup > 0.0) then
            local rule4 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.windup_red > 255) then
                rule4 = 255
            else
                rule4 = self.boss_colors.windup_red
            end
            local rule5 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.windup_green > 255) then
                rule5 = 255
            else
                rule5 = self.boss_colors.windup_green
            end
            local rule6 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.windup_blue > 255) then
                rule6 = 255
            else
                rule6 = self.boss_colors.windup_blue
            end
            return self:make_world_DungeonWorldColor(rule4, rule5, rule6)
        end
        if fan then
            local rule7 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.fan_red > 255) then
                rule7 = 255
            else
                rule7 = self.boss_colors.fan_red
            end
            local rule8 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.fan_green > 255) then
                rule8 = 255
            else
                rule8 = self.boss_colors.fan_green
            end
            local rule9 = epok.math.truncate_unsigned(0.0)
            if (self.boss_colors.fan_blue > 255) then
                rule9 = 255
            else
                rule9 = self.boss_colors.fan_blue
            end
            return self:make_world_DungeonWorldColor(rule7, rule8, rule9)
        end
        local rule10 = epok.math.truncate_unsigned(0.0)
        if (self.boss_colors.normal_red > 255) then
            rule10 = 255
        else
            rule10 = self.boss_colors.normal_red
        end
        local rule11 = epok.math.truncate_unsigned(0.0)
        if (self.boss_colors.normal_green > 255) then
            rule11 = 255
        else
            rule11 = self.boss_colors.normal_green
        end
        local rule12 = epok.math.truncate_unsigned(0.0)
        if (self.boss_colors.normal_blue > 255) then
            rule12 = 255
        else
            rule12 = self.boss_colors.normal_blue
        end
        return self:make_world_DungeonWorldColor(rule10, rule11, rule12)
    end
    if (flash > 0.0) then
        local rule13 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.hit_red > 255) then
            rule13 = 255
        else
            rule13 = self.guard_colors.hit_red
        end
        local rule14 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.hit_green > 255) then
            rule14 = 255
        else
            rule14 = self.guard_colors.hit_green
        end
        local rule15 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.hit_blue > 255) then
            rule15 = 255
        else
            rule15 = self.guard_colors.hit_blue
        end
        return self:make_world_DungeonWorldColor(rule13, rule14, rule15)
    end
    if (windup > 0.0) then
        local rule16 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.windup_red > 255) then
            rule16 = 255
        else
            rule16 = self.guard_colors.windup_red
        end
        local rule17 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.windup_green > 255) then
            rule17 = 255
        else
            rule17 = self.guard_colors.windup_green
        end
        local rule18 = epok.math.truncate_unsigned(0.0)
        if (self.guard_colors.windup_blue > 255) then
            rule18 = 255
        else
            rule18 = self.guard_colors.windup_blue
        end
        return self:make_world_DungeonWorldColor(rule16, rule17, rule18)
    end
    local rule19 = epok.math.truncate_unsigned(0.0)
    if (self.guard_colors.normal_red > 255) then
        rule19 = 255
    else
        rule19 = self.guard_colors.normal_red
    end
    local rule20 = epok.math.truncate_unsigned(0.0)
    if (self.guard_colors.normal_green > 255) then
        rule20 = 255
    else
        rule20 = self.guard_colors.normal_green
    end
    local rule21 = epok.math.truncate_unsigned(0.0)
    if (self.guard_colors.normal_blue > 255) then
        rule21 = 255
    else
        rule21 = self.guard_colors.normal_blue
    end
    return self:make_world_DungeonWorldColor(rule19, rule20, rule21)
end

---@override
function DungeonEncounter:world_shot_color()
    local rule1 = epok.math.truncate_unsigned(0.0)
    if (self.enemy_shot_color.red > 255) then
        rule1 = 255
    else
        rule1 = self.enemy_shot_color.red
    end
    local rule2 = epok.math.truncate_unsigned(0.0)
    if (self.enemy_shot_color.green > 255) then
        rule2 = 255
    else
        rule2 = self.enemy_shot_color.green
    end
    local rule3 = epok.math.truncate_unsigned(0.0)
    if (self.enemy_shot_color.blue > 255) then
        rule3 = 255
    else
        rule3 = self.enemy_shot_color.blue
    end
    return self:make_world_DungeonWorldColor(rule1, rule2, rule3)
end

---@override
function DungeonEncounter:world_death_animation()
    local rule1 = epok.to_int(0.0)
    if (self.world_animation.death_clip < 0) then
        rule1 = 0
    else
        if (self.world_animation.death_clip > 32767) then
            rule1 = 32767
        else
            rule1 = self.world_animation.death_clip
        end
    end
    return self:make_world_DungeonWorldClip(rule1, true, false, false)
end

---@override
function DungeonEncounter:world_enemy_animation(boss, action, current)
    if (action == 4) then
        local rule1 = epok.to_int(0.0)
        if (self.world_animation.idle_clip < 0) then
            rule1 = 0
        else
            if (self.world_animation.idle_clip > 32767) then
                rule1 = 32767
            else
                rule1 = self.world_animation.idle_clip
            end
        end
        return self:make_world_DungeonWorldClip(rule1, true, self.world_animation.loop, boss)
    end
    if ((action == 2) or (not (boss) and (action == 3))) then
        local rule3 = epok.to_int(0.0)
        if (self.world_animation.move_clip < 0) then
            rule3 = 0
        else
            if (self.world_animation.move_clip > 32767) then
                rule3 = 32767
            else
                rule3 = self.world_animation.move_clip
            end
        end
        local clip_index_1_2 = rule3
        return self:make_world_DungeonWorldClip(clip_index_1_2, (current ~= clip_index_1_2), self.world_animation.loop, false)
    end
    if (action == 1) then
        local rule5 = epok.to_int(0.0)
        if (self.world_animation.idle_clip < 0) then
            rule5 = 0
        else
            if (self.world_animation.idle_clip > 32767) then
                rule5 = 32767
            else
                rule5 = self.world_animation.idle_clip
            end
        end
        local clip_index_2_4 = rule5
        return self:make_world_DungeonWorldClip(clip_index_2_4, (current ~= clip_index_2_4), self.world_animation.loop, false)
    end
    return self:make_world_DungeonWorldClip(-1, false, self.world_animation.loop, false)
end

---@override
function DungeonEncounter:world_door_open(index, locked, has_key, state, progress, near_open, near_safety)
    if (((index == 3) and (state == 1)) and ((progress > 0.0) and near_safety)) then
        return true
    end
    if ((index == 3) and (state == 1)) then
        return false
    end
    if (((not (locked) or has_key) or (state == 2)) and near_open) then
        return true
    end
    return ((progress > 0.0) and near_safety)
end

---@override
function DungeonEncounter:world_door_radius(safety)
    if safety then
        return epok.math.clamp(self.world_doors.safety_radius, 0.0, 64.0)
    end
    return epok.math.clamp(self.world_doors.open_radius, 0.0, 64.0)
end

---@override
function DungeonEncounter:world_door_step(index, locked, has_key, state, progress, near_open, near_safety, dt)
    local door_open_2_1 = self:world_door_open(index, locked, has_key, state, progress, near_open, near_safety)
    local rule3 = 0.0
    if door_open_2_1 then
        rule3 = epok.math.clamp((epok.math.clamp(progress, 0.0, 1.0) + (epok.math.clamp(dt, 0.0, 1.0) * epok.math.clamp(self.world_doors.speed, 0.0, 16.0))), 0.0, 1.0)
    else
        rule3 = epok.math.clamp((epok.math.clamp(progress, 0.0, 1.0) - (epok.math.clamp(dt, 0.0, 1.0) * epok.math.clamp(self.world_doors.speed, 0.0, 16.0))), 0.0, 1.0)
    end
    local door_progress_1_2 = rule3
    if ((index == 3) and not (has_key)) then
        local rule4 = epok.math.truncate_unsigned(0.0)
        if (self.world_doors.locked_red > 255) then
            rule4 = 255
        else
            rule4 = self.world_doors.locked_red
        end
        local rule5 = epok.math.truncate_unsigned(0.0)
        if (self.world_doors.locked_green > 255) then
            rule5 = 255
        else
            rule5 = self.world_doors.locked_green
        end
        local rule6 = epok.math.truncate_unsigned(0.0)
        if (self.world_doors.locked_blue > 255) then
            rule6 = 255
        else
            rule6 = self.world_doors.locked_blue
        end
        return self:make_world_DungeonWorldDoor(door_progress_1_2, door_open_2_1, (progress ~= door_progress_1_2), ((progress < 1.0) and (door_progress_1_2 == 1.0)), ((index == 3) and not (has_key)), (epok.math.clamp(self.world_doors.closed_offset, 0.0, 8.0) + (((door_progress_1_2 * door_progress_1_2) * (3.0 - (2.0 * door_progress_1_2))) * epok.math.clamp(self.world_doors.travel, 0.0, 8.0))), self:make_world_DungeonWorldColor(rule4, rule5, rule6))
    end
    return self:make_world_DungeonWorldDoor(door_progress_1_2, door_open_2_1, (progress ~= door_progress_1_2), ((progress < 1.0) and (door_progress_1_2 == 1.0)), ((index == 3) and not (has_key)), (epok.math.clamp(self.world_doors.closed_offset, 0.0, 8.0) + (((door_progress_1_2 * door_progress_1_2) * (3.0 - (2.0 * door_progress_1_2))) * epok.math.clamp(self.world_doors.travel, 0.0, 8.0))), self:make_world_DungeonWorldColor(255, 255, 255))
end

---@override
function DungeonEncounter:world_pickup_visible(kind, position)
    if ((kind == 0) or (kind == 2)) then
        return ((position).x < epok.math.clamp(self.world_pickups.left_x, -4096.0, 4096.0))
    end
    if (kind == 1) then
        return ((position).x > epok.math.clamp(self.world_pickups.right_x, -4096.0, 4096.0))
    end
    return (((position).z > epok.math.clamp(self.world_pickups.minimum_z, -4096.0, 4096.0)) and ((position).z < epok.math.clamp(self.world_pickups.maximum_z, -4096.0, 4096.0)))
end

---@override
function DungeonEncounter:world_pickup_eligible(can_collect, nearby, height)
    return ((can_collect and nearby) and (height < epok.math.clamp(self.world_pickups.maximum_y, -32.0, 32.0)))
end

---@override
function DungeonEncounter:world_pickup_radius()
    return epok.math.clamp(self.world_pickups.radius, 0.0, 64.0)
end

---@override
function DungeonEncounter:world_pickup_heal()
    local rule1 = epok.math.truncate_unsigned(0.0)
    if (self.world_pickups.heal > 2147483647) then
        rule1 = 2147483647
    else
        rule1 = self.world_pickups.heal
    end
    return rule1
end

---@override
function DungeonEncounter:world_pickup_bob(elapsed)
    return (epok.math.sine_degrees((epok.math.wrap_degrees(elapsed) * epok.math.floor(epok.math.clamp(self.world_pickups.bob_rate, 0.0, 720.0)))) * epok.math.clamp(self.world_pickups.bob_extent, 0.0, 8.0))
end

---@override
function DungeonEncounter:world_aim_direction(delta)
    local rule1 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
    if (((false or (((delta).x < -64.0) or ((delta).x > 64.0))) or (((delta).y < -64.0) or ((delta).y > 64.0))) or (((delta).z < -64.0) or ((delta).z > 64.0))) then
        rule1 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
    else
        if (epok.math.length(delta) >= 64.0) then
            rule1 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
        else
            if ((epok.math.floor((epok.math.length(delta) * 1024.0)) / 1024.0) > 0.0) then
                rule1 = self:make_world_DungeonWorldDirection(epok.math.vector3(((delta).x / (epok.math.floor((epok.math.length(delta) * 1024.0)) / 1024.0)), ((delta).y / (epok.math.floor((epok.math.length(delta) * 1024.0)) / 1024.0)), ((delta).z / (epok.math.floor((epok.math.length(delta) * 1024.0)) / 1024.0))), true)
            else
                rule1 = self:make_world_DungeonWorldDirection(delta, true)
            end
        end
    end
    return rule1
end

---@override
function DungeonEncounter:world_patrol_goal(home, patrol_side)
    local rule1 = 0.0
    if patrol_side then
        rule1 = ((home).x + epok.math.clamp(self.movement.patrol_extent, 0.0, 16.0))
    else
        rule1 = ((home).x - epok.math.clamp(self.movement.patrol_extent, 0.0, 16.0))
    end
    return epok.math.vector3(rule1, (home).y, (home).z)
end

---@override
function DungeonEncounter:world_enemy_motion(boss, action, patrol_side, close, seen, origin, player, home, dt, boss_speed)
    if ((action == 2) or (not (boss) and (action == 3))) then
        local rule2 = epok.math.vector3(0.0, 0.0, 0.0)
        if (not (boss) and (action == 3)) then
            local rule3 = 0.0
            if patrol_side then
                rule3 = ((home).x + epok.math.clamp(self.movement.patrol_extent, 0.0, 16.0))
            else
                rule3 = ((home).x - epok.math.clamp(self.movement.patrol_extent, 0.0, 16.0))
            end
            rule2 = epok.math.vector3(rule3, (home).y, (home).z)
        else
            rule2 = epok.math.vector3((player).x, (origin).y, (player).z)
        end
        local motion_goal_2_1 = rule2
        local rule5 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
        if (((false or (((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).x < -64.0) or ((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).x > 64.0))) or (((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).y < -64.0) or ((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).y > 64.0))) or (((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).z < -64.0) or ((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).z > 64.0))) then
            rule5 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
        else
            if (epok.math.length(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))) >= 64.0) then
                rule5 = self:make_world_DungeonWorldDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
            else
                if ((epok.math.floor((epok.math.length(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))) * 1024.0)) / 1024.0) > 0.0) then
                    rule5 = self:make_world_DungeonWorldDirection(epok.math.vector3(((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).x / (epok.math.floor((epok.math.length(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))) * 1024.0)) / 1024.0)), ((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).y / (epok.math.floor((epok.math.length(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))) * 1024.0)) / 1024.0)), ((epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))).z / (epok.math.floor((epok.math.length(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z))) * 1024.0)) / 1024.0))), true)
                else
                    rule5 = self:make_world_DungeonWorldDirection(epok.math.vector3(((motion_goal_2_1).x - (origin).x), ((motion_goal_2_1).y - (origin).y), ((motion_goal_2_1).z - (origin).z)), true)
                end
            end
        end
        local direction_1_4 = rule5
        if not ((direction_1_4).valid) then
            return self:make_world_DungeonWorldMotion(epok.math.vector3(0.0, 0.0, 0.0), epok.math.heading_degrees(epok.math.clamp(((player).x - (origin).x), -64.0, 64.0), epok.math.clamp(((player).z - (origin).z), -64.0, 64.0)), 0.0, false, (boss or (close and seen)), false, false)
        end
        local rule7 = 0.0
        if boss then
            rule7 = epok.math.clamp(boss_speed, 0.0, 16.0)
        else
            if (action == 3) then
                rule7 = epok.math.clamp(self.movement.patrol_speed, 0.0, 16.0)
            else
                rule7 = epok.math.clamp(self.movement.chase_speed, 0.0, 16.0)
            end
        end
        local motion_speed_4_6 = rule7
        local clamp_3_8 = epok.math.clamp(dt, 0.0, 1.0)
        if (not (boss) and (action == 3)) then
            return self:make_world_DungeonWorldMotion(epok.math.vector3(((((direction_1_4).value).x * motion_speed_4_6) * clamp_3_8), epok.math.clamp(self.world_motion.ground_step, -1.0, 0.0), ((((direction_1_4).value).z * motion_speed_4_6) * clamp_3_8)), epok.math.heading_degrees(epok.math.clamp(((player).x - (origin).x), -64.0, 64.0), epok.math.clamp(((player).z - (origin).z), -64.0, 64.0)), epok.math.heading_degrees(((direction_1_4).value).x, ((direction_1_4).value).z), true, (boss or (close and seen)), true, true)
        end
        return self:make_world_DungeonWorldMotion(epok.math.vector3(((((direction_1_4).value).x * motion_speed_4_6) * clamp_3_8), epok.math.clamp(self.world_motion.ground_step, -1.0, 0.0), ((((direction_1_4).value).z * motion_speed_4_6) * clamp_3_8)), epok.math.heading_degrees(epok.math.clamp(((player).x - (origin).x), -64.0, 64.0), epok.math.clamp(((player).z - (origin).z), -64.0, 64.0)), 0.0, true, (boss or (close and seen)), false, true)
    end
    return self:make_world_DungeonWorldMotion(epok.math.vector3(0.0, 0.0, 0.0), epok.math.heading_degrees(epok.math.clamp(((player).x - (origin).x), -64.0, 64.0), epok.math.clamp(((player).z - (origin).z), -64.0, 64.0)), 0.0, false, (boss or (close and seen)), false, true)
end

---@override
function DungeonEncounter:world_boss_due(elapsed)
    return (elapsed >= epok.math.clamp(self.world_motion.boss_interval, 0.000244140625, 1.0))
end

return DungeonEncounter
