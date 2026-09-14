-- Fixture for tests/integration/verify_lua_modes.py.
--
-- A Lua actor that only exists at runtime: nothing places it in the scene and
-- no C++ code spawns it. `Guard` creates it with `epok.spawn("Sentinel")`, and
-- it destroys itself through the inherited reflected `Actor::destroy`. The
-- whole lifecycle is therefore authored in Lua alone.
--
-- Nothing places this class in a scene, so the project deliberately records no
-- identity for it: the acceptance run therefore exercises the adoption path,
-- where the first catalog refresh writes an entry for a script that arrived
-- without one and names the class by it from that moment on.

---@class Sentinel : EnemyBase
local Sentinel = EnemyBase:extend()

Sentinel.seen = 0

-- `EnemyBase::begin_play` is deliberately NOT called: it registers the actor in
-- the native fixture table and drives the numeric battery, which belongs to the
-- two scene actors alone.
function Sentinel:begin_play()
    self:probe(80, 1)
end

function Sentinel:tick(delta_seconds)
    local n = self.seen + 1
    self.seen = n
    self:probe(81, n)
    if n == 1 then
        -- The spawned actor answers the predicates about itself.
        self:probe_b(88, epok.is_a(self.ref, "Sentinel"))
        self:probe_b(89, epok.is_valid(self.ref))
    end
    if n == 20 then
        self:destroy()
    end
end

return Sentinel
