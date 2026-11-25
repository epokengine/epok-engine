---@class RacePilot : epok.ActorComponent
local RacePilot = epok.ActorComponent:extend()

RacePilot.camera = epok.ComponentRef(epok.SceneComponent3D)
RacePilot.lap_text = epok.ComponentRef(epok.TextComponent)
RacePilot.speed_text = epok.ComponentRef(epok.TextComponent)
RacePilot.timer = epok.ComponentRef(epok.TextComponent)
RacePilot.ready = epok.ComponentRef(epok.TextComponent)
RacePilot.finish = epok.ComponentRef(epok.TextComponent)
RacePilot.motor = epok.ComponentRef(epok.AudioComponent)
RacePilot.booster = epok.ComponentRef(RaceBoost)
RacePilot.scrape = epok.ComponentRef(epok.AudioComponent)
RacePilot.chime = epok.ComponentRef(epok.AudioComponent)

RacePilot.distance = epok.Hidden(0.0)
RacePilot.speed = epok.Hidden(0.0)
RacePilot.lane = epok.Hidden(0.0)
RacePilot.clock = epok.Hidden(-3.0)
RacePilot.lap = epok.Hidden(epok.Int32(1))

---@override
function RacePilot:begin_play()
    epok.scene.set_camera(self.camera:owner_id())
    self:restart()
end

function RacePilot:restart()
    self.distance = 0.0
    self.speed = 0.0
    self.lane = 0.0
    self.booster:clear()
    self.clock = -3.0
    self.lap = 1
    self.finish:set_enabled(false)
    self.ready:set_enabled(true)
    self.lap_text:set_number(1)
    self.speed_text:set_number(0)
    self.timer:set_stopwatch(0.0)
    self.motor:set_pitch(0.75)
    self.motor:play()
    self.chime:play()
    self:place()
end

-- Both transforms use the mesh's own baked world-space rings. The trailing
-- camera turns with the ship, including when the banking takes it upside down.
function RacePilot:place()
    local ship = epok.spline.sample(0, self.distance, self.lane, 0.31)
    if not ship.valid then
        return
    end
    self.position.x = ship.position.x
    self.position.y = ship.position.y
    self.position.z = ship.position.z
    self.rotation.x = ship.rotation.x
    self.rotation.y = ship.rotation.y
    self.rotation.z = ship.rotation.z
    local chase = epok.spline.sample(0, self.distance - 3.65, self.lane * 0.76, 1.46)
    self.camera:set_local_position(chase.position.x, chase.position.y, chase.position.z)
    -- Aim along the chord between the camera and craft. At a tight crest the
    -- tangent at the nose alone would push the craft below the screen.
    local aim = epok.spline.sample(0, self.distance - 1.825, 0.0, 0.0)
    self.camera:set_local_rotation(aim.rotation.x, aim.rotation.y, aim.rotation.z)
end

---@override
function RacePilot:tick(delta_seconds)
    if self.lap > 3 then
        if epok.gameplay_input.button_pressed(3, 0) then
            self:restart()
        end
        return
    end
    if self.clock < 0.0 then
        local before = epok.to_int(-self.clock)
        self.clock = self.clock + delta_seconds
        if self.clock >= 0.0 then
            self.ready:set_enabled(false)
            self.chime:set_pitch(1.4)
            self.chime:play()
            self.speed = 14.0
        elseif before ~= epok.to_int(-self.clock) and before < 3 then
            self.chime:set_pitch(0.85)
            self.chime:play()
        end
        return
    end

    local track = epok.spline.sample(0, self.distance, 0.0, 0.0)
    if not track.valid then
        return
    end
    local stick = epok.gameplay_input.axis(0, 0)
    local steer = stick.value
    if epok.gameplay_input.button_held(7, 0) then
        steer = -1.0
    elseif epok.gameplay_input.button_held(5, 0) then
        steer = 1.0
    end
    local target = 24.0
    if epok.gameplay_input.button_held(14, 0) or epok.gameplay_input.button_held(4, 0) then
        target = 43.0
    end
    local drift = epok.math.clamp(track.bend * self.speed * self.speed * 0.045, -3.1, 3.1)
    local side = self.lane + (steer * 7.25 - drift) * delta_seconds
    local edge = epok.math.clamp(track.half_width - 0.64, 0.0, 100.0)
    self.lane = epok.math.clamp(side, -edge, edge)
    local scraped = self.lane ~= side and self.speed > 20.0
    if scraped then
        self.speed = self.speed * 0.78
        self.booster:clear()
        self.lane = self.lane * 0.88
        self.scrape:play()
    end
    -- Painted strips occupy different lanes of the same spline surface.
    local inner = track.half_width * 0.15
    local outer = track.half_width * 0.72
    local left_pad = track.marker == 1 and self.lane >= -outer and self.lane <= -inner
    local right_pad = track.marker == 2 and self.lane >= inner and self.lane <= outer
    local thrust = self.booster:advance((left_pad or right_pad) and not scraped, delta_seconds)
    if thrust > 0.0 then
        target = 61.0
    end
    if epok.gameplay_input.button_held(6, 0) or epok.gameplay_input.button_held(15, 0) then
        target = 14.0
    elseif thrust >= 1.0 then
        self.speed = 53.0
    end
    self.speed = epok.math.move_toward(self.speed, target, 29.0 * delta_seconds)
    self.clock = self.clock + delta_seconds
    self.distance = self.distance + self.speed * delta_seconds
    if self.distance >= track.length then
        self.distance = self.distance - track.length
        self.lap = self.lap + 1
        self.chime:set_pitch(1.15)
        self.chime:play()
        if self.lap > 3 then
            self.booster:clear()
            self.finish:set_enabled(true)
            self.motor:stop()
            self.speed = 0.0
        else
            self.lap_text:set_number(self.lap)
        end
    end
    self:place()
    self.motor:set_pitch(0.72 + self.speed * 0.029)
    self.speed_text:set_number(epok.to_int(self.speed * 12.0))
    self.timer:set_stopwatch(self.clock)
end

return RacePilot
