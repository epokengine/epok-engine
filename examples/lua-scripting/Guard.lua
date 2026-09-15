-- A Lua class that extends the reflected C++ class EnemyBase.
--
-- The declaration is source text, not code: the editor reads the `extend()`
-- call, the property assignments and the method definitions statically, so a
-- class is never executed to be discovered.

---@class Guard : EnemyBase
local Guard = EnemyBase:extend()

-- Own storage on the generated subclass. A literal declares both the type and
-- the default: 2.5 is Fixed, false is Bool, an integer would be Int32.
Guard.alert_speed = 2.5
Guard.awake = false

-- Lifecycle events (begin_play, tick, end_play, on_enable, on_disable) need no
-- annotation: their signature comes from the reflected parent.
function Guard:begin_play()
    -- Lexically qualified parent call: always EnemyBase::begin_play(), never
    -- the most-derived runtime type. The class is named literally and the
    -- running instance is passed explicitly.
    Guard.super.begin_play(self)
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

-- A new callable method. Blueprints and other classes can call it; its
-- signature comes from the annotations the language server already reads.
---@param amount Fixed
---@return Fixed
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

-- An override of a reflected parent event that is not one of the five lifecycle
-- names is marked with `---@override`; its signature comes from the parent.
---@override
function Guard:defeated()
    self.awake = false
end

-- The file ends by returning the local the class is bound to.
return Guard
