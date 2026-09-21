---@class Spinner : epok.ActorComponent
local Spinner = epok.ActorComponent:extend()

-- The smallest possible behaviour: turn the actor this component is attached to.
-- Degrees per second, editable per instance from the Inspector.
Spinner.speed = 45.0

---@override
function Spinner:tick(delta_seconds)
    self.rotation.y = self.rotation.y + self.speed * delta_seconds
end

return Spinner
