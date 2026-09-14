-- A Lua class that extends the reflected C++ class EnemyBase.
--
-- `epok.class{}` is metadata, not code: the editor reads it statically from the
-- source, so it is never executed to discover the class.
local Guard = epok.class {
    profile = 1,
    id = "7bb2a7b2-3a7c-4285-b11e-d6252ac5b7d2",
    name = "Guard",
    extends = "EnemyBase",
    properties = {
        -- Own storage on the generated subclass, editable in the Inspector.
        alert_speed = {
            id = "cfe69522-9c18-463d-a9a8-0f6248d89cf1",
            type = "Fixed", default = 2.5, editable = true
        },
        awake = {
            id = "93cb889c-3bf9-4fd5-b4d0-6dcf8eea46e8",
            type = "Bool", default = false, editable = true
        }
    },
    functions = {
        -- A new callable method. Blueprints and other classes can call it.
        wake_up = {
            id = "996c8f28-cdf4-4b4e-8ea9-e1f10ec3105a", callable = true,
            parameters = { { name = "amount", type = "Fixed" } },
            returns = "Fixed"
        },
        -- An override of a reflected parent event needs an explicit
        -- `overrides` entry; its signature must match the parent exactly.
        defeated = {
            id = "72527bdb-aecf-48a2-ba1e-4543322a8082",
            overrides = "defeated", returns = "void"
        }
    }
}

-- Lifecycle events (begin_play, tick, end_play, on_enable, on_disable) need no
-- `functions` entry: their signature comes from the reflected parent.
function Guard:begin_play()
    -- Lexically qualified parent call: always EnemyBase::begin_play(), never
    -- the most-derived runtime type. It names the enclosing class and `self`.
    epok.super(Guard, self):begin_play()
    self.awake = false
end

-- `tick` takes the reflected parameter name. `epok::Actor` declares
-- `virtual void tick(Fixed)` with no parameter name, so reflection calls it
-- `arg0` and the Lua override must use that exact name.
function Guard:tick(arg0)
    if self.awake then
        -- Fixed (Q12) arithmetic. `2.5` above was stored as raw 10240.
        self.health = self.health - self.alert_speed * arg0
    end
end

function Guard:wake_up(amount)
    self.awake = true
    -- An inherited reflected `BlueprintCallable` declared by the native parent.
    -- It is reached through the same dispatch as a method of this class, in
    -- every execution mode; `health` is the parent's Fixed field it writes.
    self:apply_damage(amount)
    if self.health < 0.0 then
        -- Calls another method of this same class through the C++ virtual, so
        -- a derived class that overrides `defeated` still wins.
        self:defeated()
    end
    return self.health
end

function Guard:defeated()
    self.awake = false
end

-- The file ends by returning the local the class table is bound to.
return Guard
