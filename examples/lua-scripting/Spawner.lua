-- A Lua-only lifecycle: this class creates other actors and ends its own run
-- without a line of C++ and without a Blueprint.
--
-- `epok.spawn("Spinner")` is the Blueprint Spawn node, resolved statically by
-- class name. `self:destroy()` is the reflected `Actor::destroy` every actor
-- inherits, reached through the ordinary `self:` call syntax. Both behave the
-- same in all three execution modes.
--
-- A spawn made from inside an event is queued and runs when the current batch
-- finishes, exactly as the Blueprint node is: the reference it returns is not
-- live yet. Profile v1 therefore drives an actor's lifetime from that actor's
-- own body -- `Spinner` expires on its own `lifetime` -- rather than by holding
-- a reference to it here.
local Spawner = epok.class {
    id = "5c8e1a37-9b42-4d6f-8e15-7a3b9c2d4e60",
    name = "Spawner",
    extends = "epok::Actor3D",
    properties = {
        -- Seconds this actor stays alive before destroying itself.
        lifetime = {
            type = "Fixed", default = 5.0, editable = true
        },
        elapsed = {
            type = "Fixed", default = 0.0, editable = false
        },
        spawned = {
            type = "Int32", default = 0, editable = false
        }
    },
    functions = {}
}

function Spawner:begin_play()
    -- A numeric `for` needs constant bounds in the profile, so the batch size
    -- is a literal rather than a property.
    for i = 1, 3 do
        epok.spawn("Spinner")
        self.spawned = self.spawned + 1
    end
end

function Spawner:tick(delta_seconds)
    self.elapsed = self.elapsed + delta_seconds
    if self.elapsed >= self.lifetime then
        self:destroy()
    end
end

return Spawner
