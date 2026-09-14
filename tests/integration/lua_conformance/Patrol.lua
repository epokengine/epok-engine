-- Lua -> Lua inheritance: Patrol derives from Guard, which derives from the
-- reflected C++ EnemyBase. `tick` is NOT overridden, so Patrol runs Guard's.
local Patrol = epok.class {
    id = "2a8b5c41-3d5e-4f69-8b82-4c6d8e0f1a22",
    name = "Patrol",
    extends = "Guard",
    properties = {},
    functions = {
        damage = { overrides = "damage",
            parameters = { { name = "amount", type = "Fixed" } }, returns = "void" }
    }
}

function Patrol:damage(amount)
    epok.super(Patrol, self):damage(amount)
    self.count = self.count + 1
end

return Patrol
