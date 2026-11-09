---@class DungeonPlayerVitals : DungeonPlayerVitalsBase
local DungeonPlayerVitals = DungeonPlayerVitalsBase:extend()

---@override
function DungeonPlayerVitals:start()
    local t = self.tuning
    local available = t.maximum > 0 and t.hit_immunity >= 0.0 and t.hit_immunity <= 60.0
    self:assign(epok.combat.make_state(t.maximum, t.maximum), 0.0, available)
end

---@override
function DungeonPlayerVitals:advance(dt)
    if not self.valid or dt <= 0.0 or dt > 1.0 or self.immunity <= 0.0 then return end
    local next = self.immunity - dt
    if next < 0.0 then next = 0.0 end
    self:assign(self.health, next, self.valid)
end

---@override
function DungeonPlayerVitals:damage(amount, enabled)
    local actual = amount
    if not self.valid or not enabled or amount <= 0 or self.immunity > 0.0 then actual = 0 end
    local change = epok.combat.damage(self.health, actual)
    if change.applied > 0 then self:assign(change.state, self.tuning.hit_immunity, self.valid) end
    return change
end

---@override
function DungeonPlayerVitals:heal(amount)
    local actual = amount
    if not self.valid then actual = 0 end
    local change = epok.combat.heal(self.health, actual)
    if change.applied > 0 then self:assign(change.state, self.immunity, self.valid) end
    return change
end

---@override
function DungeonPlayerVitals:can_heal()
    local health = self.health
    return self.valid and epok.combat.valid(health) and health.current < health.maximum
end

return DungeonPlayerVitals
