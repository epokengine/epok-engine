---@class PlatformerController : epok.ActorComponent
local PlatformerController = epok.ActorComponent:extend()

-- Project-owned side-scrolling movement, written in Lua. It behaves exactly
-- like the C++ and Blueprint flavors of the same template: run with Left/Right,
-- jump with Cross, face the way the player moves and switch between the idle
-- and run animations. Velocities are pixels per second.
--
-- Buttons are written as the numbers they are stored as:
--   5 Right  7 Left  14 Cross

-- Scene bindings: typed references to the components of the same actor, filled
-- in by the template when the project is created.
PlatformerController.body = epok.ComponentRef(epok.CharacterBody2DComponent)
PlatformerController.sprite = epok.ComponentRef(epok.Sprite2DComponent)
PlatformerController.speed = 96.0
PlatformerController.jump_speed = 264.0
PlatformerController.gravity = 720.0
PlatformerController.idle_animation = epok.UInt32(0)
PlatformerController.run_animation = epok.UInt32(1)

---@override
function PlatformerController:tick(delta_seconds)
    local move = 0.0
    if epok.input.held(5, 0) then
        move = move + self.speed
    end
    if epok.input.held(7, 0) then
        move = move - self.speed
    end
    local fall = self.body:velocity_y() + self.gravity * delta_seconds
    if self.body:is_on_floor() and epok.input.pressed(14, 0) then
        fall = 0.0 - self.jump_speed
    end
    self.body:set_velocity(move, fall)
    self.body:move_and_slide()
    if move ~= 0.0 then
        self.sprite:set_flip_h(move < 0.0)
        self.sprite:play(self.run_animation)
    else
        self.sprite:play(self.idle_animation)
    end
end

return PlatformerController
