-- Fixture for tests/integration/verify_lua_modes.py.
--
-- The SAME file is compiled by all three LuaExecution modes. Nothing in it can
-- observe which mode is active; every probe below therefore has to come out
-- byte-identical in Native C++, VM bytecode and VM source.
local Guard = epok.class {
    id = "1f9a6b30-2c4d-4e58-9a71-3b5c7d9e0f11",
    name = "Guard",
    extends = "EnemyBase",
    properties = {
        speed   = { type = "Fixed",  default = 1.5,  editable = true },
        count   = { type = "Int32",  default = 0,    editable = true },
        armed   = { type = "Bool",   default = true, editable = true },
        stamina = { type = "UInt32", default = 7,    editable = true },
        ticks   = { type = "Int32",  default = 0,    editable = true },
        -- Holds the actor this class spawns at runtime, so a later tick can ask
        -- whether it is still alive.
        target  = { type = "ActorRef", editable = false }
    },
    functions = {
        report = { callable = true,
            parameters = {}, returns = "Fixed" },
        on_alert = { overrides = "on_alert",
            parameters = {}, returns = "void" }
    }
}

function Guard:report()
    return self.health + self.speed
end

function Guard:on_alert()
    self.count = self.count + 1
end

function Guard:begin_play()
    -- Exactly once: the native base writes the probes the next lines read back.
    epok.super(Guard, self):begin_play()
    local base = self.slot
    self:probe(base + 0, 1)
    self:probe(base + 1, self.hits)
    self:probe_f(base + 2, self.health)
    self:probe_f(base + 3, self.speed)
    self:probe(base + 4, self.count)
    self:probe_u(base + 5, self.stamina)
    self:probe_b(base + 6, self.armed)
end

function Guard:on_disable()
    self:probe(self.slot + 12, 1)
end

function Guard:end_play(end_play_reason)
    self:probe(self.slot + 13, 1)
end

function Guard:tick(delta_seconds)
    self:mark_begin()
    local base = self.slot
    local n = self.ticks + 1
    self.ticks = n
    self:probe(base + 7, n)
    if n == 1 then
        -- Lua -> native callable, then an inherited property written from Lua.
        self:damage(0.25)
        self.hits = self.hits + 10
        self:probe(base + 8, self.count)
        self:probe(base + 9, self.hits)
        self:probe_f(base + 10, self.health)
        self:probe_f(base + 11, self:report())
    end
    -- Intrinsic transform access: no C++ helper, no declared property. The
    -- step is a constant rather than `delta_seconds` so the final value is a tick count
    -- and cannot drift between modes with the frame rate.
    self.rotation.y = self.rotation.y + 0.25
    self.position.x = self.position.x + 0.5
    if base == 0 then
        self:probe_f(64, self.rotation.y)
        self:probe_f(65, self.position.x)
    else
        self:probe_f(66, self.rotation.y)
        self:probe_f(67, self.position.x)
    end
    -- The builtin surface. Everything below is authored in Lua alone: no C++
    -- helper, no Blueprint node, no reflected function of the fixture base. Each
    -- call lowers to the `epok::bp::api` entry point the matching Blueprint node
    -- uses, so the three modes must agree with each other and with a graph.
    if n == 2 and base == 0 then
        local made = epok.spawn("Sentinel")
        self.target = made
        -- A spawn is queued and runs when the current batch finishes, exactly
        -- as the Blueprint Spawn node behaves: the reference is not live yet.
        self:probe_b(84, epok.is_valid(made))
        -- No pad is connected under the harness, so every button reads false in
        -- every mode.
        self:probe_b(75, epok.input.held(0, 0))
        self:probe_b(76, epok.input.pressed(0, 0))
        self:probe_b(77, epok.input.released(0, 0))
        -- The project has a single scene, so the request is rejected: the Bool
        -- result is the whole observation and nothing changes.
        self:probe_b(78, epok.request_scene(9))
        self:probe_b(79, epok.is_valid(self.ref))
    end
    -- The class predicates, asked about a reference that is certainly live: the
    -- actor running the body. `self.ref` is the value the Blueprint Self node
    -- produces, and the answers are the runtime's own class graph.
    if n == 10 and base == 0 then
        self:probe_b(70, epok.is_valid(self.ref))
        self:probe_b(71, epok.is_a(self.ref, "Guard"))
        self:probe_b(72, epok.is_a(self.ref, "Patrol"))
        local wrong = epok.cast(self.ref, "Patrol")
        self:probe_b(73, epok.is_valid(wrong))
        local widened = epok.cast(self.ref, "EnemyBase")
        self:probe_b(74, epok.is_valid(widened))
    end
    if n == 10 and base > 0 then
        -- The same expressions reached from the derived class, which inherits
        -- this very body: a Patrol is a Guard and casts to itself.
        self:probe_b(85, epok.is_a(self.ref, "Guard"))
        self:probe_b(86, epok.is_a(self.ref, "Patrol"))
        local narrowed = epok.cast(self.ref, "Patrol")
        self:probe_b(87, epok.is_valid(narrowed))
    end
    if n == 40 and base == 0 then
        -- The spawned actor destroyed itself from its own Lua tick, through the
        -- inherited reflected Actor::destroy, so the stored reference is stale.
        self:probe_b(82, epok.is_valid(self.target))
        self:probe(83, n)
    end
    if n == 1 and base == 0 then
        -- Int32 edges.
        local i = self.ticks
        i = 2147483647
        i = i + 1
        self:probe(16, i)
        i = -2147483647
        i = i - 2
        self:probe(17, i)
        local m = -i
        i = 65536
        i = i * 65536
        self:probe(18, i)
        i = 7
        i = i / 0
        self:probe(19, i)
        i = 7
        i = i % 0
        self:probe(20, i)
        self:probe(21, m)
        -- UInt32 edges.
        local u = self.stamina
        u = 4294967295
        u = u + 1
        self:probe_u(22, u)
        u = 0
        u = u - 1
        self:probe_u(23, u)
        u = 65536
        u = u * 65536
        self:probe_u(24, u)
        u = 9
        u = u / 0
        self:probe_u(25, u)
        u = 9
        u = u % 0
        self:probe_u(26, u)
        -- Fixed edges.
        local f = self.speed
        f = 1.000244140625
        f = f * -0.300048828125
        self:probe_f(27, f)
        f = 5.0
        f = f / 0.0
        self:probe_f(28, f)
        f = -2.5
        self:probe(29, epok.to_int(f))
        i = 1000000
        self:probe_f(30, epok.to_fixed(i))
        -- Unsigned ordering on the sign bit.
        u = 2147483648
        local ordered = self.armed
        ordered = u > 1
        self:probe_b(31, ordered)
        -- Short circuit: neither call may reach `bump`.
        local ok = self.armed
        ok = false
        ok = ok and self:bump()
        ok = true
        ok = ok or self:bump()
        -- One real call, so the counter proves the side effect is observable.
        ok = self:bump()
        self:probe_b(14, ok)
        -- Evaluation order of two calls with side effects: 1 then 2.
        self:probe(52, self:next_id() * 10 + self:next_id())
        -- Constant-bounded numeric for with a negative step.
        local s = self.ticks
        s = 0
        for k = 10, 1, -2 do
            s = s + k
        end
        -- Definite assignment through every branch.
        local d = self.ticks
        if s > 100 then
            d = 1
        elseif s == 30 then
            d = 2
        else
            d = 3
        end
        self:probe(53, s * 10 + d)
        self:probe(15, d)
    end
    self:mark_end()
end

return Guard
