-- Lua -> Lua inheritance: Patrol derives from Guard, which derives from the
-- reflected C++ EnemyBase. `tick` is NOT overridden, so Patrol runs Guard's.

---@class Patrol : Guard
local Patrol = Guard:extend()

---@id 6f2c8d40-1a3b-4c5d-9e70-2f4a6b8c0d13
---@override
---@param amount Fixed
function Patrol:damage(amount)
    Patrol.super.damage(self, amount)
    self.count = self.count + 1
end

return Patrol
