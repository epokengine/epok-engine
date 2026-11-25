---@class RaceBoost : epok.ActorComponent
local RaceBoost = epok.ActorComponent:extend()

RaceBoost.charge_sound = epok.ComponentRef(epok.AudioComponent)
RaceBoost.release_sound = epok.ComponentRef(epok.AudioComponent)
RaceBoost.plume = epok.ComponentRef(epok.SceneComponent3D)
RaceBoost.wake = epok.ComponentRef(epok.ParticleEmitterComponent)
RaceBoost.lens = epok.ComponentRef(epok.Camera3DComponent)
RaceBoost.gauge = epok.ComponentRef(epok.ProgressBarComponent)

RaceBoost.charge_left = epok.Hidden(0.0)
RaceBoost.thrust_left = epok.Hidden(0.0)
RaceBoost.pad_contact = epok.Hidden(false)

function RaceBoost:clear()
    self.charge_left = 0.0
    self.thrust_left = 0.0
    self.pad_contact = false
    self.charge_sound:stop()
    self.release_sound:stop()
    self.plume:set_local_scale(1.0, 1.0, 0.26)
    self.wake:set_rate(12.0)
    self.lens:set_field_of_view(78.0)
    self.gauge:set_value(0.0)
end

-- A strip commits the charge on entry, even if the ship crosses its far edge
-- before the wind-up finishes. The returned strength peaks on the ignition tick.
---@param on_pad Bool
---@param dt Fixed
---@return Fixed
function RaceBoost:advance(on_pad, dt)
    if on_pad and not self.pad_contact and self.charge_left <= 0.0 and self.thrust_left <= 0.0 then
        self.charge_left = 0.38
        self.gauge:set_colors(255, 174, 61, 31, 45, 64)
        self.charge_sound:play()
    end
    self.pad_contact = on_pad
    if self.charge_left > 0.0 then
        self.charge_left = epok.math.move_toward(self.charge_left, 0.0, dt)
        if self.charge_left > 0.0 then
            local fill = 1.0 - self.charge_left / 0.38
            self.gauge:set_value(fill)
            self.plume:set_local_scale(1.0, 0.9 + fill * 0.5, 0.30 + fill * 0.62)
            self.lens:set_field_of_view(78.0 - fill * 3.0)
            return 0.0
        end
        self.thrust_left = 1.1
        self.gauge:set_colors(76, 250, 229, 31, 45, 64)
        self.release_sound:play()
        self.wake:burst(7)
        self.wake:set_rate(38.0)
    elseif self.thrust_left > 0.0 then
        self.thrust_left = epok.math.move_toward(self.thrust_left, 0.0, dt)
        if self.thrust_left <= 0.0 then
            self.wake:set_rate(12.0)
        end
    else
        return 0.0
    end
    local thrust = self.thrust_left / 1.1
    self.gauge:set_value(thrust)
    self.plume:set_local_scale(1.0, 1.0 + thrust * 0.28, 0.26 + thrust * 1.46)
    self.lens:set_field_of_view(78.0 + thrust * 7.0)
    return thrust
end

return RaceBoost
