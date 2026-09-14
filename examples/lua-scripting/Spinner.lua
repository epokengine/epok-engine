-- Intrinsic transform access: a Lua class that moves itself with no C++ helper.
--
-- `position`, `rotation` and `scale` are not declared anywhere. Every World3D
-- class has them, and they address the actor's root component through the same
-- runtime functions as the Blueprint Get/Set Position, Rotation and Scale
-- nodes, identically in all three execution modes.

---@class Spinner : epok.Actor3D
local Spinner = epok.Actor3D:extend()

-- Turn rate, in Q12 units per second of `delta_seconds`.
Spinner.speed = 90.0
Spinner.drift = 0.5
-- Seconds this actor lives before destroying itself; 0 means forever.
-- `Spawner` relies on this default, because a spawned instance starts from its
-- class defaults.
Spinner.lifetime = 3.0
Spinner.age = epok.Hidden(0.0)

function Spinner:tick(delta_seconds)
    -- Only components are readable and writable: `self.rotation` as a whole is
    -- a vector, and whole vectors do not cross the boundary in profile v1.
    self.rotation.y = self.rotation.y + self.speed * delta_seconds
    self.position.x = self.position.x + self.drift * delta_seconds
    -- The end of this actor's own run, authored in Lua: `destroy` is the
    -- reflected `Actor::destroy` every actor inherits.
    self.age = self.age + delta_seconds
    if self.lifetime > 0.0 and self.age >= self.lifetime then
        self:destroy()
    end
end

return Spinner
