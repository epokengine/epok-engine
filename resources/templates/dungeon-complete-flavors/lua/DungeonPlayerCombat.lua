---@class DungeonPlayerCombat : DungeonPlayerCombatBase
local DungeonPlayerCombat = DungeonPlayerCombatBase:extend()

---@override
function DungeonPlayerCombat:attack(state, grounded, jumped, held, clip_playing, clips_ready)
    local phase = state.phase
    local active = state.attacking
    local sent = state.shot_sent
    local clip = epok.to_int(-1.0)
    local invalid = false
    local starting = false
    if active and (jumped or not grounded) then
        return self:make_attack_step(self:make_attack_state(phase, false, sent), -1, false, true, false)
    end
    if active and not clip_playing then
        if phase == 0 then
            phase = 1
            clip = 1
            sent = false
        elseif phase == 1 then
            if held then phase = 0; clip = 0 else phase = 2; clip = 2 end
        else
            active = false
            invalid = true
        end
    end
    if not active and grounded and not jumped and held and clips_ready then
        phase = 0
        active = true
        sent = false
        clip = 0
        starting = true
    end
    local fire = active and not sent and phase == 1
    if fire then sent = true end
    return self:make_attack_step(self:make_attack_state(phase, active, sent), clip, fire, invalid, starting)
end

---@override
function DungeonPlayerCombat:blocks_movement(state, grounded, held)
    return state.attacking or (grounded and held)
end

---@override
function DungeonPlayerCombat:target_input(free_held, free_pressed, lock_pressed)
    if free_pressed then return 1 end
    if not free_held and lock_pressed then return 2 end
    return 0
end

---@override
function DungeonPlayerCombat:target_check(phase, candidate, within_keep)
    if not candidate or not within_keep then return self:make_target_check(phase, false, false) end
    phase = phase + 1
    if phase >= self.tuning.visibility_period then return self:make_target_check(0, true, true) end
    return self:make_target_check(phase, true, false)
end

---@override
function DungeonPlayerCombat:candidate(active, has_mesh, collider_enabled, marked)
    return active and has_mesh and collider_enabled and marked
end

---@override
function DungeonPlayerCombat:after_selected(distance, index, selected_distance, selected_index)
    return distance.high > selected_distance.high or (distance.high == selected_distance.high and (distance.low > selected_distance.low or (distance.low == selected_distance.low and index > selected_index)))
end

---@override
function DungeonPlayerCombat:prefer_candidate(distance, index, best_exists, best_distance, best_index, cycling)
    return not best_exists or distance.high < best_distance.high or (distance.high == best_distance.high and (distance.low < best_distance.low or (cycling and distance.low == best_distance.low and index < best_index)))
end

---@override
function DungeonPlayerCombat:projectile(origin, direction)
    local speed = self.tuning.projectile_speed
    local velocity = epok.math.vector3(direction.x * speed, direction.y * speed, direction.z * speed)
    return epok.projectiles.spawn(origin, velocity, self.tuning.projectile_lifetime, 0)
end


---@param value UInt32
---@param minimum UInt32
---@param maximum UInt32
---@return UInt32
function DungeonPlayerCombat:bounded(value, minimum, maximum)
    if value < minimum then return minimum end
    if value > maximum then return maximum end
    return value
end
---@param value Fixed
---@return Fixed
function DungeonPlayerCombat:bounded_size(value)
    if value < 0.0 then return 0.0 end
    if value > 32.0 then return 32.0 end
    return value
end
---@override
function DungeonPlayerCombat:trace_settings(enemy)
    if enemy then return self:make_impact_trace(self.impact_tuning.enemy_trace_mask, false) end
    return self:make_impact_trace(self.impact_tuning.player_trace_mask, true)
end
---@override
function DungeonPlayerCombat:impact(enemy, target_exists, target_is_player, target_layers)
    if enemy then
        if target_is_player then return 2 end
        return 3
    end
    if target_exists and (target_layers / 2) % 2 ~= 0 then return 1 end
    return 0
end
---@override
function DungeonPlayerCombat:shot_visual()
    local t = self.shot_style
    local bounded1 = t.red
    if bounded1 < 0 then
        bounded1 = 0
    else
        if bounded1 > 255 then
            bounded1 = 255
        end
    end
    local bounded2 = t.green
    if bounded2 < 0 then
        bounded2 = 0
    else
        if bounded2 > 255 then
            bounded2 = 255
        end
    end
    local bounded3 = t.blue
    if bounded3 < 0 then
        bounded3 = 0
    else
        if bounded3 > 255 then
            bounded3 = 255
        end
    end
    return self:make_shot_visual(epok.math.clamp(t.size, 0.0, 32.0), bounded1, bounded2, bounded3)
end
---@override
function DungeonPlayerCombat:effect_step(kind, age, advance)
    local t = self.effect_tuning
    local bounded4 = age
    if bounded4 < 0 then
        bounded4 = 0
    else
        if bounded4 > 65535 then
            bounded4 = 65535
        end
    end
    age = bounded4
    if advance and age < 65535 then age = age + 1 end
    if kind ~= 0 then
        local bounded5 = t.hit_end_age
        if bounded5 < 0 then
            bounded5 = 0
        else
            if bounded5 > 65535 then
                bounded5 = 65535
            end
        end
        local last = bounded5
        local size = epok.math.clamp(t.hit_size, 0.0, 32.0)
        if age >= last then return self:make_effect_step(age, 0, size, false) end
        local bounded6 = t.hit_first_frame
        if bounded6 < 0 then
            bounded6 = 0
        else
            if bounded6 > 2047 then
                bounded6 = 2047
            end
        end
        local first = bounded6
        local bounded7 = t.hit_ticks_per_frame
        if bounded7 < 1 then
            bounded7 = 1
        else
            if bounded7 > 65535 then
                bounded7 = 65535
            end
        end
        local stride = bounded7
        local bounded8 = first + age / stride
        if bounded8 < 0 then
            bounded8 = 0
        else
            if bounded8 > 2047 then
                bounded8 = 2047
            end
        end
        local frame = bounded8
        return self:make_effect_step(age, frame, size, true)
    end
    local bounded9 = t.muzzle_end_age
    if bounded9 < 0 then
        bounded9 = 0
    else
        if bounded9 > 65535 then
            bounded9 = 65535
        end
    end
    local last = bounded9
    local size = epok.math.clamp(t.muzzle_size, 0.0, 32.0)
    if age >= last then return self:make_effect_step(age, 0, size, false) end
    local bounded10 = t.muzzle_switch_age
    if bounded10 < 0 then
        bounded10 = 0
    else
        if bounded10 > 65535 then
            bounded10 = 65535
        end
    end
    if age < bounded10 then
        local bounded11 = t.muzzle_first_frame
        if bounded11 < 0 then
            bounded11 = 0
        else
            if bounded11 > 2047 then
                bounded11 = 2047
            end
        end
        return self:make_effect_step(age, bounded11, size, true)
    end
    local bounded12 = t.muzzle_second_frame
    if bounded12 < 0 then
        bounded12 = 0
    else
        if bounded12 > 2047 then
            bounded12 = 2047
        end
    end
    return self:make_effect_step(age, bounded12, size, true)
end
---@override
function DungeonPlayerCombat:damage_feedback(remaining_us, elapsed_us, restart)
    local t = self.feedback_tuning
    local bounded13 = t.duration_us
    if bounded13 < 0 then
        bounded13 = 0
    else
        if bounded13 > 10000000 then
            bounded13 = 10000000
        end
    end
    local duration = bounded13
    if duration == 0 then return self:make_damage_feedback(0, false, 0, 0, 0, 0) end
    local bounded14 = remaining_us
    if bounded14 < 0 then
        bounded14 = 0
    else
        if bounded14 > duration then
            bounded14 = duration
        end
    end
    local remaining = bounded14
    if restart then remaining = duration end
    if elapsed_us >= remaining then return self:make_damage_feedback(0, false, 0, 0, 0, 0) end
    remaining = remaining - elapsed_us
    local bounded15 = t.border_pixels
    if bounded15 < 1 then
        bounded15 = 1
    else
        if bounded15 > 64 then
            bounded15 = 64
        end
    end
    local width = bounded15
    local bounded16 = t.red_base
    if bounded16 < 0 then
        bounded16 = 0
    else
        if bounded16 > 255 then
            bounded16 = 255
        end
    end
    local red = bounded16
    local bounded17 = t.red_span
    if bounded17 < 0 then
        bounded17 = 0
    else
        if bounded17 > 255 - red then
            bounded17 = 255 - red
        end
    end
    local span = bounded17
    local bounded18 = t.green
    if bounded18 < 0 then
        bounded18 = 0
    else
        if bounded18 > 255 then
            bounded18 = 255
        end
    end
    local bounded19 = t.blue
    if bounded19 < 0 then
        bounded19 = 0
    else
        if bounded19 > 255 then
            bounded19 = 255
        end
    end
    return self:make_damage_feedback(remaining, true, (width * remaining + duration - 1) / duration, red + span * remaining / duration, bounded18, bounded19)
end
---@override
function DungeonPlayerCombat:damage_rumble()
    local t = self.feedback_tuning
    local bounded20 = t.rumble_large
    if bounded20 < 0 then
        bounded20 = 0
    else
        if bounded20 > 255 then
            bounded20 = 255
        end
    end
    local bounded21 = t.rumble_duration_ms
    if bounded21 < 0 then
        bounded21 = 0
    else
        if bounded21 > 60000 then
            bounded21 = 60000
        end
    end
    return self:make_damage_rumble(bounded20, t.rumble_small, bounded21)
end

-- Generated authored rules: DungeonPlayerCombat
---@override
function DungeonPlayerCombat:control_gate(ready, interface_block, controls, alive)
    if ((not (ready) or interface_block) or not (controls)) then
        return 1
    end
    if not (alive) then
        return 2
    end
    return 0
end

---@override
function DungeonPlayerCombat:control_intent(x, y, right, left, up, down, suppress)
    if suppress then
        return epok.math.stick_intent(0.0, 0.0)
    end
    local rule2 = 0.0
    if (right and not (left)) then
        rule2 = 1.0
    else
        if (left and not (right)) then
            rule2 = -1.0
        else
            rule2 = 0.0
        end
    end
    local button_axis_1_1 = rule2
    local rule3 = 0.0
    if (up and not (down)) then
        rule3 = 1.0
    else
        if (down and not (up)) then
            rule3 = -1.0
        else
            rule3 = 0.0
        end
    end
    if ((button_axis_1_1 ~= 0.0) or (rule3 ~= 0.0)) then
        local rule4 = 0.0
        if (up and not (down)) then
            rule4 = 1.0
        else
            if (down and not (up)) then
                rule4 = -1.0
            else
                rule4 = 0.0
            end
        end
        return epok.math.stick_intent(button_axis_1_1, rule4)
    end
    return epok.math.stick_intent(x, y)
end

---@override
function DungeonPlayerCombat:control_camera_start(facing)
    local wrap_degrees_2_1 = epok.math.wrap_degrees(facing)
    local rule3 = epok.math.clamp(self.control_camera.initial_pitch, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
    local pitch_1_2 = rule3
    return self:make_controls_DungeonControlCamera(wrap_degrees_2_1, pitch_1_2, wrap_degrees_2_1, pitch_1_2, 0.0, 0.0, 0)
end

---@override
function DungeonPlayerCombat:control_camera_step(state, input)
    local rule2 = epok.math.clamp((state).pitch, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
    local rule3 = epok.math.clamp((state).goal_pitch, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
    local rule4 = 0.0
    if ((input).free_aim or (input).target) then
        rule4 = 1.0
    else
        rule4 = 0.0
    end
    local rule5 = epok.math.truncate_unsigned(0.0)
    if (input).free_aim then
        rule5 = 2
    else
        if (input).target then
            rule5 = 1
        else
            rule5 = 0
        end
    end
    local rule1 = self:make_controls_DungeonControlCamera(epok.math.wrap_degrees((state).yaw), rule2, epok.math.wrap_degrees((state).goal_yaw), rule3, epok.math.clamp((state).hold, 0.0, 10.0), epok.math.move_toward(epok.math.clamp((state).blend, 0.0, 1.0), rule4, (epok.math.clamp(self.control_camera.blend_rate, 0.0, 32.0) * epok.math.clamp((input).dt, 0.0, 1.0))), rule5)
    local rule6 = self:make_controls_DungeonControlCamera(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, epok.math.truncate_unsigned(0.0))
    if ((((input).look).strength > 0.0) and ((rule1).mode ~= 1)) then
        if ((rule1).hold <= 0.0) then
            local rule7 = epok.math.clamp(((rule1).pitch - ((((input).look).y * epok.math.clamp(self.control_camera.pitch_rate, 0.0, 720.0)) * epok.math.clamp((input).dt, 0.0, 1.0))), epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
            rule6 = self:make_controls_DungeonControlCamera((rule1).yaw, (rule1).pitch, epok.math.wrap_degrees(((rule1).yaw + ((((input).look).x * epok.math.clamp(self.control_camera.orbit_rate, 0.0, 720.0)) * epok.math.clamp((input).dt, 0.0, 1.0)))), rule7, epok.math.clamp(self.control_camera.manual_time, 0.0, 10.0), (rule1).blend, (rule1).mode)
        else
            local rule8 = epok.math.clamp(((rule1).goal_pitch - ((((input).look).y * epok.math.clamp(self.control_camera.pitch_rate, 0.0, 720.0)) * epok.math.clamp((input).dt, 0.0, 1.0))), epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
            rule6 = self:make_controls_DungeonControlCamera((rule1).yaw, (rule1).pitch, epok.math.wrap_degrees(((rule1).goal_yaw + ((((input).look).x * epok.math.clamp(self.control_camera.orbit_rate, 0.0, 720.0)) * epok.math.clamp((input).dt, 0.0, 1.0)))), rule8, epok.math.clamp(self.control_camera.manual_time, 0.0, 10.0), (rule1).blend, (rule1).mode)
        end
    else
        rule6 = self:make_controls_DungeonControlCamera((rule1).yaw, (rule1).pitch, (rule1).goal_yaw, (rule1).goal_pitch, epok.math.move_toward(epok.math.clamp((rule1).hold, 0.0, 10.0), 0.0, epok.math.clamp((input).dt, 0.0, 1.0)), (rule1).blend, (rule1).mode)
    end
    local rule9 = self:make_controls_DungeonControlCamera(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, epok.math.truncate_unsigned(0.0))
    if ((rule6).mode == 1) then
        local rule10 = epok.math.clamp((input).target_pitch, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
        rule9 = self:make_controls_DungeonControlCamera((rule6).yaw, (rule6).pitch, (input).target_yaw, rule10, (rule6).hold, (rule6).blend, (rule6).mode)
    else
        if (((rule6).mode == 0) and ((rule6).hold <= 0.0)) then
            if (input).moving then
                rule9 = self:make_controls_DungeonControlCamera((rule6).yaw, (rule6).pitch, (rule6).yaw, (rule6).goal_pitch, (rule6).hold, (rule6).blend, (rule6).mode)
            else
                rule9 = self:make_controls_DungeonControlCamera((rule6).yaw, (rule6).pitch, (input).facing, (rule6).goal_pitch, (rule6).hold, (rule6).blend, (rule6).mode)
            end
        else
            rule9 = rule6
        end
    end
    local rule12 = 0.0
    if (((rule9).mode ~= 0) or ((rule9).hold > 0.0)) then
        rule12 = epok.math.clamp(self.control_camera.manual_response, 0.0, 32.0)
    else
        rule12 = epok.math.clamp(self.control_camera.normal_response, 0.0, 32.0)
    end
    local rule13 = 0.0
    if (((rule9).mode ~= 0) or ((rule9).hold > 0.0)) then
        rule13 = epok.math.clamp(self.control_camera.manual_cap, 0.0, 720.0)
    else
        rule13 = epok.math.clamp(self.control_camera.normal_cap, 0.0, 720.0)
    end
    local rule14 = 0.0
    if ((epok.math.delta_degrees(((rule9).yaw + epok.math.clamp(((epok.math.delta_degrees(epok.math.wrap_degrees((rule9).yaw), epok.math.wrap_degrees((rule9).goal_yaw)) * (rule12 * epok.math.clamp((input).dt, 0.0, 1.0))) / (1.0 + (rule12 * epok.math.clamp((input).dt, 0.0, 1.0)))), (0.0 - (rule13 * epok.math.clamp((input).dt, 0.0, 1.0))), (rule13 * epok.math.clamp((input).dt, 0.0, 1.0)))), (rule9).goal_yaw) > -0.01) and (epok.math.delta_degrees(((rule9).yaw + epok.math.clamp(((epok.math.delta_degrees(epok.math.wrap_degrees((rule9).yaw), epok.math.wrap_degrees((rule9).goal_yaw)) * (rule12 * epok.math.clamp((input).dt, 0.0, 1.0))) / (1.0 + (rule12 * epok.math.clamp((input).dt, 0.0, 1.0)))), (0.0 - (rule13 * epok.math.clamp((input).dt, 0.0, 1.0))), (rule13 * epok.math.clamp((input).dt, 0.0, 1.0)))), (rule9).goal_yaw) < 0.01)) then
        rule14 = (rule9).goal_yaw
    else
        rule14 = ((rule9).yaw + epok.math.clamp(((epok.math.delta_degrees(epok.math.wrap_degrees((rule9).yaw), epok.math.wrap_degrees((rule9).goal_yaw)) * (rule12 * epok.math.clamp((input).dt, 0.0, 1.0))) / (1.0 + (rule12 * epok.math.clamp((input).dt, 0.0, 1.0)))), (0.0 - (rule13 * epok.math.clamp((input).dt, 0.0, 1.0))), (rule13 * epok.math.clamp((input).dt, 0.0, 1.0))))
    end
    local rule15 = epok.math.clamp(((rule9).pitch + ((((rule9).goal_pitch - (rule9).pitch) * (epok.math.clamp(self.control_camera.pitch_response, 0.0, 32.0) * epok.math.clamp((input).dt, 0.0, 1.0))) / (1.0 + (epok.math.clamp(self.control_camera.pitch_response, 0.0, 32.0) * epok.math.clamp((input).dt, 0.0, 1.0))))), epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
    local rule11 = self:make_controls_DungeonControlCamera(epok.math.wrap_degrees(rule14), rule15, (rule9).goal_yaw, (rule9).goal_pitch, (rule9).hold, (rule9).blend, (rule9).mode)
    return rule11
end

---@override
function DungeonPlayerCombat:control_movement_step(intent, state, facing, target_yaw, target, attack_owns, fire_pressed, dt)
    local sine_degrees_3_1 = epok.math.sine_degrees((state).yaw)
    local cosine_degrees_2_2 = epok.math.cosine_degrees((state).yaw)
    local clamp_1_3 = epok.math.clamp(self.control_movement.speed, 0.0, 64.0)
    local rule4 = 0.0
    if (((state).mode == 0) and ((intent).strength > 0.0)) then
        rule4 = epok.math.move_toward_degrees(epok.math.clamp(epok.math.clamp(facing, -1440.0, 1440.0), -1440.0, 1440.0), ((state).yaw + epok.math.heading_degrees((intent).x, (intent).y)), (epok.math.clamp(self.control_movement.turn_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
    else
        if (((state).mode == 1) and target) then
            if (((intent).strength > 0.0) and not (attack_owns)) then
                rule4 = epok.math.move_toward_degrees(epok.math.clamp(epok.math.clamp(facing, -1440.0, 1440.0), -1440.0, 1440.0), ((state).yaw + epok.math.heading_degrees((intent).x, (intent).y)), (epok.math.clamp(self.control_movement.turn_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
            else
                if fire_pressed then
                    rule4 = epok.math.wrap_degrees(epok.math.wrap_degrees(target_yaw))
                else
                    rule4 = epok.math.move_toward_degrees(epok.math.clamp(epok.math.clamp(facing, -1440.0, 1440.0), -1440.0, 1440.0), epok.math.wrap_degrees(epok.math.wrap_degrees(target_yaw)), (epok.math.clamp(self.control_movement.turn_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
                end
            end
        else
            if ((state).mode == 2) then
                rule4 = epok.math.move_toward_degrees(epok.math.clamp(epok.math.clamp(facing, -1440.0, 1440.0), -1440.0, 1440.0), (state).yaw, (epok.math.clamp(self.control_movement.turn_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
            else
                rule4 = epok.math.clamp(epok.math.clamp(facing, -1440.0, 1440.0), -1440.0, 1440.0)
            end
        end
    end
    return self:make_controls_DungeonControlMove((((sine_degrees_3_1 * (intent).y) + (cosine_degrees_2_2 * (intent).x)) * clamp_1_3), (((cosine_degrees_2_2 * (intent).y) - (sine_degrees_3_1 * (intent).x)) * clamp_1_3), rule4)
end

---@override
function DungeonPlayerCombat:control_view_proposal(position, state, shoulder_distance)
    local sine_degrees_8_1 = epok.math.sine_degrees((state).yaw)
    local cosine_degrees_7_2 = epok.math.cosine_degrees((state).yaw)
    local clamp_6_3 = epok.math.clamp((state).blend, 0.0, 1.0)
    local clamp_5_4 = epok.math.clamp(self.control_view.boom, 0.01, 64.0)
    local clamp_4_5 = epok.math.clamp(self.control_view.height, -32.0, 32.0)
    local product_3_6 = (epok.math.clamp(self.control_view.shoulder_offset, -32.0, 32.0) * clamp_6_3)
    local product_2_7 = ((epok.math.clamp(shoulder_distance, epok.math.clamp(self.control_view.shoulder_min, 0.01, 64.0), 64.0) - clamp_5_4) * clamp_6_3)
    local product_1_8 = ((clamp_5_4 + product_2_7) * epok.math.cosine_degrees((state).pitch))
    return self:make_controls_DungeonControlView(epok.math.vector3((position).x, ((position).y + (clamp_4_5 + ((epok.math.clamp(self.control_view.aim_height, -32.0, 32.0) - clamp_4_5) * clamp_6_3))), (position).z), epok.math.vector3(((cosine_degrees_7_2 * product_3_6) - (sine_degrees_8_1 * product_1_8)), ((clamp_5_4 + product_2_7) * epok.math.sine_degrees((state).pitch)), ((0.0 - (sine_degrees_8_1 * product_3_6)) - (cosine_degrees_7_2 * product_1_8))), (clamp_5_4 + product_2_7), (state).yaw, (state).pitch)
end

---@override
function DungeonPlayerCombat:control_focus_height(enabled, center, scale)
    if enabled then
        return (epok.math.clamp(center, -32.0, 32.0) * epok.math.clamp(scale, -32.0, 32.0))
    end
    return epok.math.clamp(self.control_aim.focus_height, -32.0, 32.0)
end

---@override
function DungeonPlayerCombat:control_target_angles(origin, target, focus)
    local clamp_3_1 = epok.math.clamp(((target).z - (origin).z), -32.0, 32.0)
    local clamp_2_2 = epok.math.clamp(((target).x - (origin).x), -32.0, 32.0)
    local length2_1_3 = epok.math.length2(clamp_2_2, clamp_3_1)
    if (length2_1_3 == 0.0) then
        if ((target).y >= (origin).y) then
            local rule4 = epok.math.clamp(89.0, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
            return self:make_controls_DungeonControlAngles(epok.math.heading_degrees(clamp_2_2, clamp_3_1), rule4)
        end
        local rule5 = epok.math.clamp(-89.0, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), epok.math.clamp(self.control_camera.pitch_max, epok.math.clamp(self.control_camera.pitch_min, -89.0, 89.0), 89.0))
        return self:make_controls_DungeonControlAngles(epok.math.heading_degrees(clamp_2_2, clamp_3_1), rule5)
    end
    return self:make_controls_DungeonControlAngles(epok.math.heading_degrees(clamp_2_2, clamp_3_1), epok.math.delta_degrees(0.0, epok.math.heading_degrees(epok.math.clamp((((origin).y + epok.math.clamp(self.control_view.height, -32.0, 32.0)) - ((target).y + focus)), -128.0, 128.0), length2_1_3)))
end

---@override
function DungeonPlayerCombat:control_aim_step(current, camera_pitch, target_pitch, dt, free_aim, target)
    local clamp_1_1 = epok.math.clamp(self.control_aim.pitch_min, -89.0, 89.0)
    if free_aim then
        return epok.math.move_toward(current, epok.math.clamp(camera_pitch, clamp_1_1, epok.math.clamp(self.control_aim.pitch_max, clamp_1_1, 89.0)), (epok.math.clamp(self.control_aim.pose_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
    end
    if target then
        return epok.math.move_toward(current, epok.math.clamp(target_pitch, clamp_1_1, epok.math.clamp(self.control_aim.pitch_max, clamp_1_1, 89.0)), (epok.math.clamp(self.control_aim.pose_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
    end
    return epok.math.move_toward(current, epok.math.clamp(0.0, clamp_1_1, epok.math.clamp(self.control_aim.pitch_max, clamp_1_1, 89.0)), (epok.math.clamp(self.control_aim.pose_rate, 0.0, 1440.0) * epok.math.clamp(dt, 0.0, 1.0)))
end

---@override
function DungeonPlayerCombat:control_marker_begin()
    return epok.math.truncate_unsigned(epok.math.floor(epok.math.clamp(self.control_marker.intro_frames, 0.0, 64.0)))
end

---@override
function DungeonPlayerCombat:control_marker_step(intro, spin)
    local clamp_3_1 = epok.math.clamp(self.control_marker.base_radius, 1.0, 4096.0)
    local floor_2_2 = epok.math.floor(epok.math.clamp(spin, 0.0, 63.0))
    local move_toward_1_3 = epok.math.move_toward(epok.math.floor(epok.math.clamp(intro, 0.0, epok.math.floor(epok.math.clamp(self.control_marker.intro_frames, 0.0, 64.0)))), 0.0, 1.0)
    local rule4 = 0.0
    if (move_toward_1_3 > 0.0) then
        rule4 = epok.math.clamp(((((epok.math.floor(epok.math.clamp(self.control_marker.intro_frames, 0.0, 64.0)) + 1.0) - move_toward_1_3) * epok.math.clamp(self.control_marker.brightness_step, 0.0, 255.0)) * 2.0), 0.0, 255.0)
    else
        rule4 = epok.math.clamp((epok.math.clamp(self.control_marker.brightness_end, 0.0, 255.0) * 2.0), 0.0, 255.0)
    end
    return self:make_controls_DungeonControlMarker(move_toward_1_3, ((floor_2_2 + 1.0) - (epok.math.floor(((floor_2_2 + 1.0) / 64.0)) * 64.0)), (epok.math.clamp(self.control_marker.size, 0.0, 32.0) * epok.math.clamp(((clamp_3_1 + (epok.math.clamp(self.control_marker.step_radius, 0.0, 4096.0) * move_toward_1_3)) / clamp_3_1), 0.0, 64.0)), epok.math.truncate_unsigned(rule4), (move_toward_1_3 > 0.0))
end

---@override
function DungeonPlayerCombat:control_rig(bones)
    if (bones == 13) then
        return self:make_controls_DungeonControlRig(2, 6, 7)
    end
    return self:make_controls_DungeonControlRig(-1, -1, -1)
end

---@override
function DungeonPlayerCombat:control_animation_select(state, playing, grounded, vertical_speed, speed, finished, jumped)
    if jumped then
        local rule1 = 0.0
        if false then
            rule1 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
        else
            if false then
                rule1 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                rule1 = 0.0
            end
        end
        return self:make_controls_DungeonControlAnimation(3, false, (false or false), (3 ~= playing), rule1)
    end
    if not (grounded) then
        if (vertical_speed > 0.0) then
            local rule2 = 0.0
            if false then
                rule2 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                if false then
                    rule2 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
                else
                    rule2 = 0.0
                end
            end
            return self:make_controls_DungeonControlAnimation(3, false, (false or false), (3 ~= playing), rule2)
        end
        local rule3 = 0.0
        if false then
            rule3 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
        else
            if false then
                rule3 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                rule3 = 0.0
            end
        end
        return self:make_controls_DungeonControlAnimation(4, false, (false or false), (4 ~= playing), rule3)
    end
    if ((state == 3) or (state == 4)) then
        local rule4 = 0.0
        if false then
            rule4 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
        else
            if false then
                rule4 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                rule4 = 0.0
            end
        end
        return self:make_controls_DungeonControlAnimation(5, false, (false or false), (5 ~= playing), rule4)
    end
    if ((state ~= 5) or finished) then
        if (speed < epok.math.clamp(self.control_animation.idle_speed, 0.0, 64.0)) then
            local rule5 = 0.0
            if false then
                rule5 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                if false then
                    rule5 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
                else
                    rule5 = 0.0
                end
            end
            return self:make_controls_DungeonControlAnimation(0, true, (false or false), (0 ~= playing), rule5)
        end
        if ((speed > epok.math.clamp(self.control_animation.run_speed, 0.0, 64.0)) or ((state == 2) and (speed > epok.math.clamp(self.control_animation.keep_run_speed, 0.0, 64.0)))) then
            local rule6 = 0.0
            if false then
                rule6 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                if true then
                    rule6 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
                else
                    rule6 = 0.0
                end
            end
            return self:make_controls_DungeonControlAnimation(2, true, (false or true), (2 ~= playing), rule6)
        end
        local rule7 = 0.0
        if true then
            rule7 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
        else
            if false then
                rule7 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
            else
                rule7 = 0.0
            end
        end
        return self:make_controls_DungeonControlAnimation(1, true, (true or false), (1 ~= playing), rule7)
    end
    local rule8 = 0.0
    if (state == 1) then
        rule8 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.walk_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
    else
        if (state == 2) then
            rule8 = epok.math.clamp((epok.math.clamp(speed, 0.0, 64.0) / epok.math.clamp(self.control_animation.run_cycle, 0.001, 64.0)), epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), epok.math.clamp(self.control_animation.rate_max, epok.math.clamp(self.control_animation.rate_min, 0.0, 2.0), 2.0))
        else
            rule8 = 0.0
        end
    end
    return self:make_controls_DungeonControlAnimation(state, (state <= 2), ((state == 1) or (state == 2)), (state ~= playing), rule8)
end

---@override
function DungeonPlayerCombat:control_animation_clock(phase, frames, rate, dt, model_valid)
    local rule2 = epok.math.truncate_unsigned(0.0)
    if (phase > (4294967295 - epok.math.truncate_unsigned((((epok.math.clamp(rate, 0.0, 2.0) * epok.math.clamp(dt, 0.0, 1.0)) * 60.0) * 4096.0)))) then
        rule2 = 4294967295
    else
        rule2 = (phase + epok.math.truncate_unsigned((((epok.math.clamp(rate, 0.0, 2.0) * epok.math.clamp(dt, 0.0, 1.0)) * 60.0) * 4096.0)))
    end
    local phase_add_1_1 = rule2
    if model_valid then
        local rule4 = epok.math.truncate_unsigned(0.0)
        if (frames > 262144) then
            rule4 = 262144
        else
            if (frames <= 1) then
                rule4 = 2
            else
                rule4 = frames
            end
        end
        local product_3_3 = ((rule4 - 1) * 8192)
        local product_2_5 = ((phase_add_1_1 / product_3_3) * product_3_3)
        return self:make_controls_DungeonControlClock((phase_add_1_1 - product_2_5), ((phase_add_1_1 - product_2_5) / 4096))
    end
    return self:make_controls_DungeonControlClock(phase_add_1_1, 0)
end

---@override
function DungeonPlayerCombat:control_direction(yaw, pitch)
    local cosine_degrees_1_1 = epok.math.cosine_degrees(pitch)
    return epok.math.vector3((epok.math.sine_degrees(yaw) * cosine_degrees_1_1), (0.0 - epok.math.sine_degrees(pitch)), (epok.math.cosine_degrees(yaw) * cosine_degrees_1_1))
end

---@override
function DungeonPlayerCombat:control_aim_ray(yaw, pitch)
    local sine_degrees_5_1 = epok.math.sine_degrees(pitch)
    local cosine_degrees_4_2 = epok.math.cosine_degrees(pitch)
    local product_3_3 = (epok.math.sine_degrees(yaw) * cosine_degrees_4_2)
    local product_2_4 = (epok.math.cosine_degrees(yaw) * cosine_degrees_4_2)
    local clamp_1_5 = epok.math.clamp(self.control_aim.ray_distance, 0.0, 128.0)
    return epok.math.vector3(((epok.math.vector3(product_3_3, (0.0 - sine_degrees_5_1), product_2_4)).x * clamp_1_5), ((epok.math.vector3(product_3_3, (0.0 - sine_degrees_5_1), product_2_4)).y * clamp_1_5), ((epok.math.vector3(product_3_3, (0.0 - sine_degrees_5_1), product_2_4)).z * clamp_1_5))
end

---@override
function DungeonPlayerCombat:control_aim_mask(muzzle)
    if muzzle then
        return self.control_aim.muzzle_mask
    end
    return self.control_aim.ray_mask
end

---@override
function DungeonPlayerCombat:control_shot_aim(target, mode)
    if target then
        return 1
    end
    if (mode == 2) then
        return 2
    end
    return 0
end

---@override
function DungeonPlayerCombat:control_direction_toward(delta)
    if (((((delta).x < -64.0) or ((delta).x > 64.0)) or (((delta).y < -64.0) or ((delta).y > 64.0))) or (((delta).z < -64.0) or ((delta).z > 64.0))) then
        return self:make_controls_DungeonControlDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
    end
    local length_2_1 = epok.math.length(delta)
    if (length_2_1 >= 64.0) then
        return self:make_controls_DungeonControlDirection(epok.math.vector3(0.0, 0.0, 0.0), false)
    end
    local quotient_1_2 = (epok.math.floor((length_2_1 * 1024.0)) / 1024.0)
    if (quotient_1_2 > 0.0) then
        return self:make_controls_DungeonControlDirection(epok.math.vector3(((delta).x / quotient_1_2), ((delta).y / quotient_1_2), ((delta).z / quotient_1_2)), true)
    end
    return self:make_controls_DungeonControlDirection(delta, true)
end

---@override
function DungeonPlayerCombat:control_motion_speed(displacement, dt)
    if (dt <= 0.0) then
        return 0.0
    end
    local clamp_1_1 = epok.math.clamp(dt, 0.0, 1.0)
    return epok.math.length2((epok.math.clamp((displacement).x, -64.0, 64.0) / clamp_1_1), (epok.math.clamp((displacement).z, -64.0, 64.0) / clamp_1_1))
end

return DungeonPlayerCombat
