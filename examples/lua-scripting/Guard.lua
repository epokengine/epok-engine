-- A Lua class that extends the reflected C++ class EnemyBase.
--
-- `epok.class{}` is metadata, not code: the editor reads it statically from the
-- source, so it is never executed to discover the class.
local Guard = epok.class {
    id = "7bb2a7b2-3a7c-4285-b11e-d6252ac5b7d2",
    name = "Guard",
    extends = "EnemyBase",
    properties = {
        -- Own storage on the generated subclass, editable in the Inspector.
        alert_speed = {
            type = "Fixed", default = 2.5, editable = true
        },
        awake = {
            type = "Bool", default = false, editable = true
        }
    },
    functions = {
        -- A new callable method. Blueprints and other classes can call it.
        wake_up = {
            callable = true,
            parameters = { { name = "amount", type = "Fixed" } },
            returns = "Fixed"
        },
        -- An override of a reflected parent event needs an explicit
        -- `overrides` entry; its signature must match the parent exactly.
        defeated = {
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
-- `delta_seconds` and the Lua override must use that exact name.
function Guard:tick(delta_seconds)
    if self.awake then
        -- Fixed (Q12) arithmetic. `2.5` above was stored as raw 10240.
        self.health = self.health - self.alert_speed * delta_seconds
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
