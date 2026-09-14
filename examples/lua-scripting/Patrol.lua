-- A Lua class that extends another Lua class. Guard is a real C++ subclass in
-- every execution mode, so deriving from it uses the ordinary parent rule.
local Patrol = epok.class {
    id = "5e21b03e-da85-417a-867b-420cd25b01a4",
    name = "Patrol",
    extends = "Guard",
    properties = {
        steps_remaining = {
            type = "Int32", default = 8, editable = true
        },
        stride = {
            type = "Fixed", default = 1.5, editable = true
        },
        travelled = {
            type = "Fixed", default = 0.0, editable = true
        }
    },
    functions = {
        reset_route = {
            callable = true,
            parameters = { { name = "legs", type = "Int32" } },
            returns = "void"
        }
    }
}

function Patrol:tick(delta_seconds)
    -- Reaches Guard::tick, which is the Lua body in Guard.lua.
    epok.super(Patrol, self):tick(delta_seconds)
    if self.steps_remaining > 0 then
        self.steps_remaining = self.steps_remaining - 1
        self.travelled = self.travelled + self.stride * delta_seconds
    end
end

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
