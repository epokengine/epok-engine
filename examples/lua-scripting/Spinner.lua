-- Intrinsic transform access: a Lua class that moves itself with no C++ helper.
--
-- `position`, `rotation` and `scale` are not declared anywhere. Every World3D
-- class has them, and they address the actor's root component through the same
-- runtime functions as the Blueprint Get/Set Position, Rotation and Scale
-- nodes, identically in all three execution modes.
local Spinner = epok.class {
    id = "0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c",
    name = "Spinner",
    extends = "epok::Actor3D",
    properties = {
        -- Turn rate, in Q12 units per second of `delta_seconds`.
        speed = {
            type = "Fixed", default = 90.0, editable = true
        },
        drift = {
            type = "Fixed", default = 0.5, editable = true
        },
        -- Seconds this actor lives before destroying itself; 0 means forever.
        -- `Spawner` relies on this default, because a spawned instance starts
        -- from its class defaults.
        lifetime = {
            type = "Fixed", default = 3.0, editable = true
        },
        age = {
            type = "Fixed", default = 0.0, editable = false
        }
    },
    functions = {}
}

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
