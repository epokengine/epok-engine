-- A Lua class that extends another Lua class. Guard is a real C++ subclass in
-- every execution mode, so deriving from it uses the ordinary parent rule.

---@class Patrol : Guard
local Patrol = Guard:extend()

Patrol.steps_remaining = 8
Patrol.stride = 1.5
Patrol.travelled = 0.0

function Patrol:tick(delta_seconds)
    -- Reaches Guard::tick, which is the Lua body in Guard.lua.
    Patrol.super.tick(self, delta_seconds)
    if self.steps_remaining > 0 then
        self.steps_remaining = self.steps_remaining - 1
        self.travelled = self.travelled + self.stride * delta_seconds
    end
end

---@param legs Int32
function Patrol:reset_route(legs)
    self.steps_remaining = legs
    self.travelled = 0.0
    -- Constant-bounded numeric for. The loop variable is Int32.
    for leg = 1, 4 do
        -- Int32 -> Fixed is explicit; there is no implicit numeric coercion.
        self.travelled = self.travelled + epok.to_fixed(leg)
    end
end

return Patrol
