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

---@class Spawner : epok.Actor3D
local Spawner = epok.Actor3D:extend()

-- Seconds this actor stays alive before destroying itself.
Spawner.lifetime = 5.0
-- `epok.Hidden` is the one wrapper: the same value, kept out of the Inspector.
Spawner.elapsed = epok.Hidden(0.0)
Spawner.spawned = epok.Hidden(0)

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
