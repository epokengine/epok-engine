---@class ThirdPersonController : epok.ActorComponent
local ThirdPersonController = epok.ActorComponent:extend()

-- Project-owned third-person movement, animation and an obstacle-aware orbit
-- camera, written in Lua. It behaves exactly like the C++ and Blueprint flavors
-- of the same template: every number below is exactly representable in the
-- engine's Q12 fixed point, and everything that is not a plain multiply is an
-- engine operation, so the three implementations agree bit for bit on the
-- PlayStation as well as on the desktop.
--
-- Enumerators are written as the numbers they are stored as:
--   Axis   0 LeftX  1 LeftY  2 RightX  3 RightY
--   Button 4 Up  5 Right  6 Down  7 Left  14 Cross
--   Locomotion state 0 Idle  1 Walk  2 Run  3 JumpUp  4 JumpDown  5 Land

-- Scene bindings. Typed references, filled in by the template when the project
-- is created, so renaming an actor never breaks the controller. `camera` is the
-- camera's transform rather than the actor: writing it takes three scalars, so
-- the controller never holds a whole vector and compiles in every Lua
-- execution mode. A class may declare sixteen properties in total, which is
-- why the tuning below lives as named literals in the methods that use it
-- rather than as editable fields.
ThirdPersonController.camera = epok.ComponentRef(epok.SceneComponent3D)
ThirdPersonController.visual = epok.ComponentRef(epok.Mesh3DComponent)
ThirdPersonController.idle_clip = epok.UInt32(0)
ThirdPersonController.walk_clip = epok.UInt32(1)
ThirdPersonController.run_clip = epok.UInt32(2)
ThirdPersonController.jump_up_clip = epok.UInt32(3)
ThirdPersonController.jump_down_clip = epok.UInt32(4)
ThirdPersonController.land_clip = epok.UInt32(5)

-- Runtime state.
ThirdPersonController.camera_yaw = epok.Hidden(0.0)
ThirdPersonController.camera_pitch = epok.Hidden(18.0)
ThirdPersonController.facing = epok.Hidden(0.0)
ThirdPersonController.velocity = epok.Hidden(epok.Vector3(0.0, 0.0, 0.0))
ThirdPersonController.grounded = epok.Hidden(false)
ThirdPersonController.animation_state = epok.Hidden(epok.UInt32(0))
ThirdPersonController.playing_state = epok.Hidden(epok.UInt32(5))
ThirdPersonController.playback_phase = epok.Hidden(0.0)

---@override
function ThirdPersonController:begin_play()
    self.facing = self.rotation.y
    self.camera_yaw = self.facing
    self.camera_pitch = 18.0
    self.velocity.x = 0.0
    self.velocity.y = 0.0
    self.velocity.z = 0.0
    self.grounded = false
    self.animation_state = 0
    self.playing_state = 5
    self.playback_phase = 0.0
    epok.scene.set_camera(self.camera:owner_id())
    self:place_camera()
end

---@override
function ThirdPersonController:tick(delta_seconds)
    if delta_seconds <= 0.0 then
        return
    end
    self:orbit_camera(delta_seconds)
    self:move(delta_seconds)
    self:place_camera()
end

--- Right stick or mouse look. Yaw wraps; pitch is clamped so the boom never
--- passes over the character's head or under the floor.
---@param delta_seconds Fixed
function ThirdPersonController:orbit_camera(delta_seconds)
    local right_x = epok.input.axis(2, 0)
    local right_y = epok.input.axis(3, 0)
    local look = epok.math.stick_intent(right_x.value, right_y.value)
    self.camera_yaw = epok.math.wrap_degrees(
        self.camera_yaw + look.x * 140.0 * delta_seconds)
    self.camera_pitch = epok.math.clamp(
        self.camera_pitch - look.y * 70.0 * delta_seconds,
        -10.0, 55.0)
end

--- Camera-relative movement, gravity, jumping and step handling.
---@param delta_seconds Fixed
function ThirdPersonController:move(delta_seconds)
    local body = epok.owner()

    -- A held direction is full deflection, so the keyboard and the stick agree.
    local digital_x = 0.0
    local digital_z = 0.0
    if epok.input.held(5, 0) then
        digital_x = digital_x + 1.0
    end
    if epok.input.held(7, 0) then
        digital_x = digital_x - 1.0
    end
    if epok.input.held(4, 0) then
        digital_z = digital_z + 1.0
    end
    if epok.input.held(6, 0) then
        digital_z = digital_z - 1.0
    end
    local left_x = epok.input.axis(0, 0)
    local left_y = epok.input.axis(1, 0)
    local move_x = left_x.value
    local move_z = left_y.value
    if digital_x ~= 0.0 or digital_z ~= 0.0 then
        move_x = digital_x
        move_z = digital_z
    end

    local intent = epok.math.stick_intent(move_x, move_z)
    local yaw_sin = epok.math.sine_degrees(self.camera_yaw)
    local yaw_cos = epok.math.cosine_degrees(self.camera_yaw)
    local desired_x = (yaw_sin * intent.y + yaw_cos * intent.x) * 6.0
    local desired_z = (yaw_cos * intent.y - yaw_sin * intent.x) * 6.0
    if intent.strength ~= 0.0 then
        local target = self.camera_yaw + epok.math.heading_degrees(intent.x, intent.y)
        self.facing = epok.math.move_toward_degrees(self.facing, target,
            540.0 * delta_seconds)
    end

    local response = 3.0 * delta_seconds
    if self.grounded then
        response = 12.0 * delta_seconds
    end
    if response > 1.0 then
        response = 1.0
    end
    self.velocity.x = self.velocity.x + (desired_x - self.velocity.x) * response
    self.velocity.z = self.velocity.z + (desired_z - self.velocity.z) * response

    local jumped = self.grounded and epok.input.pressed(14, 0)
    if self.grounded then
        self.velocity.y = 0.0
        if jumped then
            self.velocity.y = 9.0
        end
    else
        self.velocity.y = self.velocity.y - 24.0 * delta_seconds
    end
    -- A small downward bias keeps a grounded character on ramps and steps.
    local fall = self.velocity.y * delta_seconds
    if self.grounded and not jumped then
        fall = -0.05
    end

    local before_x = self.position.x
    local before_z = self.position.z
    local movement = epok.math.vector3(self.velocity.x * delta_seconds, fall,
        self.velocity.z * delta_seconds)
    local result = epok.collision.move(body, movement, 4294967295)
    self.grounded = result.grounded
    if self.velocity.y > 0.0 and result.displacement.y + 0.001 < fall then
        self.velocity.y = 0.0
    end
    if result.blocked and self.grounded and not jumped then
        self:step_over(movement.x - result.displacement.x, movement.z - result.displacement.z)
    end

    self.rotation.y = self.facing
    local speed = epok.math.length2((self.position.x - before_x) / delta_seconds,
        (self.position.z - before_z) / delta_seconds)
    self:animate(delta_seconds, speed, jumped)
end

--- Lift, retry the blocked part of the step, then settle back down. A curb or
--- a ramp lip is walked over; a wall still stops the character.
---@param remaining_x Fixed
---@param remaining_z Fixed
function ThirdPersonController:step_over(remaining_x, remaining_z)
    if remaining_x == 0.0 and remaining_z == 0.0 then
        return
    end
    local body = epok.owner()
    local lift = epok.collision.move(body, epok.math.vector3(0.0, 0.6, 0.0),
        4294967295)
    epok.collision.move(body, epok.math.vector3(remaining_x, 0.0, remaining_z), 4294967295)
    local settle = epok.collision.move(body,
        epok.math.vector3(0.0, -0.05 - lift.displacement.y, 0.0), 4294967295)
    self.grounded = settle.grounded
end

--- Idle, Walk, Run, JumpUp, JumpDown and Land, chosen from ground contact,
--- vertical speed and ground speed. Walk and Run advance with distance rather
--- than with the clock, so the feet never skate.
---@param delta_seconds Fixed
---@param speed Fixed
---@param jumped Bool
function ThirdPersonController:animate(delta_seconds, speed, jumped)
    -- `playing_state` always names the clip the animator is on, so the clip
    -- index never needs a field of its own.
    local playing = self:clip_for(self.playing_state)
    local frames = self.visual:clip_frames(playing)
    local finished = true
    if frames > 1 then
        local playback = self.visual:playback_state()
        finished = playback.ticks >= (frames - 1) * 2
    end
    local next_state = self:choose_state(speed, finished, jumped)
    if next_state ~= self.playing_state then
        local clip = self:clip_for(next_state)
        if self.visual:play_clip(clip, self:clip_loops(next_state)) then
            self.playing_state = next_state
            playing = clip
            self.playback_phase = 0.0
        end
    end
    if next_state == 1 or next_state == 2 then
        self.visual:pause_animation()
        local cycle = 6.0
        if next_state == 1 then
            cycle = 2.6
        end
        local rate = epok.math.clamp(speed / cycle, 0.2, 1.6)
        self.playback_phase = self.playback_phase + rate * delta_seconds * 60.0
        -- Wrapping the cycle needs no unbounded loop: the phase advances by at
        -- most 1.6 ticks in a fixed step and the shortest clip loops over two,
        -- so four passes can never leave anything left to wrap.
        local period = self.visual:clip_loop_ticks(playing)
        for pass = 1, 4 do
            if self.playback_phase >= period then
                self.playback_phase = self.playback_phase - period
            end
        end
        self.visual:set_animation_position(self.playback_phase)
    end
end

--- The locomotion state machine. Speeds are compared against exact Q12
--- thresholds: 0.119873046875 is the idle cut-off, 3.60009765625 starts a run
--- and 3.0 keeps one going, so a steady jog does not flicker.
---@param speed Fixed
---@param finished Bool
---@param jumped Bool
---@return UInt32
function ThirdPersonController:choose_state(speed, finished, jumped)
    local state = self.animation_state
    if jumped then
        state = 3
    elseif not self.grounded then
        state = 4
        if self.velocity.y > 0.0 then
            state = 3
        end
    elseif state == 3 or state == 4 then
        state = 5
    elseif state ~= 5 or finished then
        state = 1
        if speed < 0.119873046875 then
            state = 0
        elseif speed > 3.60009765625 then
            state = 2
        elseif self.animation_state == 2 and speed > 3.0 then
            state = 2
        end
    end
    self.animation_state = state
    return state
end

---@param state UInt32
---@return UInt32
function ThirdPersonController:clip_for(state)
    local clip = self.land_clip
    if state == 0 then
        clip = self.idle_clip
    elseif state == 1 then
        clip = self.walk_clip
    elseif state == 2 then
        clip = self.run_clip
    elseif state == 3 then
        clip = self.jump_up_clip
    elseif state == 4 then
        clip = self.jump_down_clip
    end
    return clip
end

---@param state UInt32
---@return Bool
function ThirdPersonController:clip_loops(state)
    local looping = false
    if state == 0 or state == 1 or state == 2 then
        looping = true
    end
    return looping
end

--- Spring the camera onto a boom behind the character and pull it in when the
--- boom would pass through level geometry.
function ThirdPersonController:place_camera()
    local reach = 5.4 * epok.math.cosine_degrees(self.camera_pitch)
    local yaw_sin = epok.math.sine_degrees(self.camera_yaw)
    local yaw_cos = epok.math.cosine_degrees(self.camera_yaw)
    local origin = epok.math.vector3(self.position.x, self.position.y + 1.1,
        self.position.z)
    local offset = epok.math.vector3(-yaw_sin * reach,
        5.4 * epok.math.sine_degrees(self.camera_pitch), -yaw_cos * reach)
    local hit = epok.collision.raycast_segment(origin, offset, 4294967295, epok.owner(), false)
    if hit.hit then
        -- Stop short of the surface so the near plane never clips it.
        local fraction = hit.fraction - 0.04
        if fraction < 0.12 then
            fraction = 0.12
        end
        offset = epok.math.scale(offset, fraction)
    end
    self.camera:set_local_position(origin.x + offset.x, origin.y + offset.y,
        origin.z + offset.z)
    self.camera:set_local_rotation(self.camera_pitch, self.camera_yaw, 0.0)
end

return ThirdPersonController
