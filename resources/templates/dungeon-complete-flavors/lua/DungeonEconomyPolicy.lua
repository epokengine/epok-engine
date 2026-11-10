---@class DungeonEconomyPolicy : DungeonEconomyPolicyBase
local DungeonEconomyPolicy = DungeonEconomyPolicyBase:extend()

---@param phase UInt32
---@param heal UInt32
---@param ack Bool
---@param serial UInt32
---@param total UInt32
function DungeonEconomyPolicy:intent(phase, heal, ack, serial, total)
    self:assign_tx(self:make_tx(phase, 0, serial, total, heal, ack, 0.0))
end
---@param serial UInt32
---@param total UInt32
---@return Bool
function DungeonEconomyPolicy:durable(serial, total)
    local r0 = self:saved_receipt(0)
    if r0.serial == serial and r0.total >= total and r0.total > 0 then return true end
    local r1 = self:saved_receipt(1)
    if r1.serial == serial and r1.total >= total and r1.total > 0 then return true end
    local r2 = self:saved_receipt(2)
    if r2.serial == serial and r2.total >= total and r2.total > 0 then return true end
    local r3 = self:saved_receipt(3)
    if r3.serial == serial and r3.total >= total and r3.total > 0 then return true end
    local r4 = self:saved_receipt(4)
    if r4.serial == serial and r4.total >= total and r4.total > 0 then return true end
    local r5 = self:saved_receipt(5)
    if r5.serial == serial and r5.total >= total and r5.total > 0 then return true end
    local r6 = self:saved_receipt(6)
    if r6.serial == serial and r6.total >= total and r6.total > 0 then return true end
    local r7 = self:saved_receipt(7)
    if r7.serial == serial and r7.total >= total and r7.total > 0 then return true end
    return false
end
---@override
function DungeonEconomyPolicy:valid(value)
    local t = self.tuning
    if value.signature ~= 1095980081 or value.version ~= 1 or value.ore > t.balance_limit or value.credits > t.balance_limit
        or value.medkits > t.medkit_cap or value.tanks > t.tank_cap or value.buster > t.buster_cap then return false end
    if value.r0.total > t.balance_limit then return false end
    if value.r1.total > t.balance_limit then return false end
    if value.r2.total > t.balance_limit then return false end
    if value.r3.total > t.balance_limit then return false end
    if value.r4.total > t.balance_limit then return false end
    if value.r5.total > t.balance_limit then return false end
    if value.r6.total > t.balance_limit then return false end
    if value.r7.total > t.balance_limit then return false end
    return true
end
---@override
function DungeonEconomyPolicy:price(value, item)
    if item == 0 then return self.tuning.medkit_price end
    if item == 1 then return self.tuning.tank_price end
    if value.buster == 4294967295 then return 0 end
    if self.tuning.buster_price > 4294967295 / (value.buster + 1) then return 0 end
    return self.tuning.buster_price * (value.buster + 1)
end
---@override
function DungeonEconomyPolicy:damage_bonus()
    if self.tuning.buster_damage > 0 and self.saved.buster > 4294967295 / self.tuning.buster_damage then return 4294967295 end
    return self.saved.buster * self.tuning.buster_damage
end
---@override
function DungeonEconomyPolicy:find_receipt(serial)
    local r0 = self:pending_receipt(0)
    if r0.serial == serial and r0.total > 0 then return 0 end
    local r1 = self:pending_receipt(1)
    if r1.serial == serial and r1.total > 0 then return 1 end
    local r2 = self:pending_receipt(2)
    if r2.serial == serial and r2.total > 0 then return 2 end
    local r3 = self:pending_receipt(3)
    if r3.serial == serial and r3.total > 0 then return 3 end
    local r4 = self:pending_receipt(4)
    if r4.serial == serial and r4.total > 0 then return 4 end
    local r5 = self:pending_receipt(5)
    if r5.serial == serial and r5.total > 0 then return 5 end
    local r6 = self:pending_receipt(6)
    if r6.serial == serial and r6.total > 0 then return 6 end
    local r7 = self:pending_receipt(7)
    if r7.serial == serial and r7.total > 0 then return 7 end
    if r0.total == 0 then self:write_receipt(0, serial, 0) return 0 end
    if r1.total == 0 then self:write_receipt(1, serial, 0) return 1 end
    if r2.total == 0 then self:write_receipt(2, serial, 0) return 2 end
    if r3.total == 0 then self:write_receipt(3, serial, 0) return 3 end
    if r4.total == 0 then self:write_receipt(4, serial, 0) return 4 end
    if r5.total == 0 then self:write_receipt(5, serial, 0) return 5 end
    if r6.total == 0 then self:write_receipt(6, serial, 0) return 6 end
    if r7.total == 0 then self:write_receipt(7, serial, 0) return 7 end
    return 8
end
---@override
function DungeonEconomyPolicy:transfer(serial, total, claimed)
    if total > self.tuning.balance_limit or claimed > total then return self:make_transfer(-1, 0) end
    if not self:valid(self.pending) then return self:make_transfer(-5, 0) end
    local index = self:find_receipt(serial)
    if index >= 8 then return self:make_transfer(-2, 0) end
    local receipt = self:pending_receipt(index)
    local previous = receipt.total
    if claimed > previous then previous = claimed end
    if total < previous then return self:make_transfer(-3, 0) end
    local amount = total - previous
    local s = self.pending
    if amount > self.tuning.balance_limit - s.ore then return self:make_transfer(-4, 0) end
    self:wallet(s.ore + amount, s.credits, s.medkits, s.tanks, s.buster)
    self:write_receipt(index, serial, total)
    return self:make_transfer(0, amount)
end
---@override
function DungeonEconomyPolicy:sell()
    local s = self.pending
    if not self:valid(s) or s.ore == 0 or self.tuning.sale_rate == 0 then return false end
    if s.ore > (self.tuning.balance_limit - s.credits) / self.tuning.sale_rate then return false end
    self:wallet(0, s.credits + s.ore * self.tuning.sale_rate, s.medkits, s.tanks, s.buster)
    return true
end
---@override
function DungeonEconomyPolicy:buy(item)
    local s = self.pending
    if item > 2 or not self:valid(s) then return false end
    if item == 0 and s.medkits >= self.tuning.medkit_cap then return false end
    if item == 1 and s.tanks >= self.tuning.tank_cap then return false end
    if item == 2 then
        if s.buster >= self.tuning.buster_cap or s.buster == 4294967295 then return false end
        if self.tuning.buster_price > 4294967295 / (s.buster + 1) then return false end
    end
    local cost = self:price(s, item)
    if s.credits < cost then return false end
    if item == 0 then self:wallet(s.ore, s.credits - cost, s.medkits + 1, s.tanks, s.buster) end
    if item == 1 then self:wallet(s.ore, s.credits - cost, s.medkits, s.tanks + 1, s.buster) end
    if item == 2 then self:wallet(s.ore, s.credits - cost, s.medkits, s.tanks, s.buster + 1) end
    return true
end
---@override
function DungeonEconomyPolicy:activate(menu, selection, can_heal)
    if self.tx.phase ~= 0 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if menu == 1 then
        if selection == 0 then return self:decision(2, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        if selection == 1 then
            if not self.loaded then return self:decision(0, 1, 0, 0, 0, self.tx.serial, self.tx.total, false) end
            return self:decision(3, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
        end
        return self:decision(1, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if not self.loaded then
        if selection == 4 or (menu == 3 and selection == 3) then return self:decision(1, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        return self:decision(0, 1, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if not self:valid(self.saved) then return self:decision(0, 2, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    self:assign_pending(self.saved)
    self:intent(0, 0, false, 0, 0)
    if menu == 2 then
        if selection == 4 then return self:decision(1, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        if selection == 0 then
            if not self:sell() then return self:decision(0, 3, 0, 0, 0, self.tx.serial, self.tx.total, false) end
            self:intent(1, 0, false, 0, 0)
            return self:decision(4, 11, 0, 0, 0, self.tx.serial, self.tx.total, true)
        end
        if not self:buy(selection - 1) then
            if selection == 3 and self.saved.buster == self.tuning.buster_cap then return self:decision(0, 5, 0, 0, 0, self.tx.serial, self.tx.total, false) end
            return self:decision(0, 4, 0, 0, 0, self.tx.serial, self.tx.total, false)
        end
        self:intent(1, 0, false, 0, 0)
        if selection == 3 then return self:decision(4, 10, 0, 0, 0, self.tx.serial, self.tx.total, true) end
        return self:decision(4, 9, 0, 0, 0, self.tx.serial, self.tx.total, true)
    end
    if menu == 3 then
        if selection == 3 then return self:decision(1, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        if selection == 2 then return self:decision(0, 6, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        if selection > 3 then return self:decision(0, 27, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        local s = self.pending
        if selection == 0 then
            if s.medkits == 0 then return self:decision(0, 7, 0, 0, 0, self.tx.serial, self.tx.total, false) end
            if not can_heal then return self:decision(0, 8, 0, 0, 0, self.tx.serial, self.tx.total, false) end
            self:wallet(s.ore, s.credits, s.medkits - 1, s.tanks, s.buster)
            self:intent(1, self.tuning.medkit_heal, false, 0, 0)
            return self:decision(4, 12, 0, 0, 0, self.tx.serial, self.tx.total, true)
        end
        if s.tanks == 0 then return self:decision(0, 7, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        if not can_heal then return self:decision(0, 8, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        self:wallet(s.ore, s.credits, s.medkits, s.tanks - 1, s.buster)
        self:intent(1, self.tuning.tank_heal, false, 0, 0)
        return self:decision(4, 13, 0, 0, 0, self.tx.serial, self.tx.total, true)
    end
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:receive(running, signature, version, serial, total, claimed)
    if self.tx.phase ~= 0 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not self.loaded then return self:decision(0, 1, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not running then return self:decision(0, 14, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if signature ~= 1095980081 or version ~= 1 then return self:decision(0, 15, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    self:assign_pending(self.saved)
    local change = self:transfer(serial, total, claimed)
    if change.result < 0 then
        if change.result == -2 then return self:decision(0, 16, change.result, 0, 0, self.tx.serial, self.tx.total, false) end
        return self:decision(0, 17, change.result, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if change.amount == 0 then
        if claimed < total and self:durable(serial, total) then
            self:intent(3, 0, true, serial, total)
            return self:decision(5, 19, 0, 0, 0, self.tx.serial, self.tx.total, false)
        end
        return self:decision(0, 18, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    self:intent(1, 0, true, serial, total)
    return self:decision(4, 19, 0, change.amount, 0, self.tx.serial, self.tx.total, true)
end
---@override
function DungeonEconomyPolicy:can_save()
    return self.loaded and self.tx.phase == 1 and self:valid(self.pending)
end
---@override
function DungeonEconomyPolicy:save_started(request, accepted)
    if self.tx.phase ~= 1 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not accepted or not self:can_save() then self:intent(0, 0, false, 0, 0) return self:decision(0, 30, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local t = self.tx
    t.phase = 2
    t.request = request
    self:assign_tx(t)
    return self:decision(0, 20, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:save_completed(request, succeeded)
    if self.tx.phase ~= 2 or request ~= self.tx.request then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not succeeded then self:intent(0, 0, false, 0, 0) return self:decision(0, 21, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    self:assign_saved(self.pending)
    local t = self.tx
    local heal = t.heal
    t.heal = 0
    if t.ack_pending and self:durable(t.serial, t.total) then
        t.phase = 3
        self:assign_tx(t)
        return self:decision(5, 22, 0, 0, heal, self.tx.serial, self.tx.total, true)
    end
    t.phase = 0
    self:assign_tx(t)
    return self:decision(0, 22, 0, 0, heal, self.tx.serial, self.tx.total, true)
end
---@override
function DungeonEconomyPolicy:ack_started(accepted)
    if self.tx.phase ~= 3 or not self.tx.ack_pending or not self:durable(self.tx.serial, self.tx.total) then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local t = self.tx
    if not accepted then t.phase = 0 self:assign_tx(t) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    t.phase = 4
    t.ack_time = 0.0
    self:assign_tx(t)
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:ack_sent(succeeded)
    if self.tx.phase ~= 4 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local t = self.tx
    if succeeded then t.phase = 5 else t.phase = 0 end
    self:assign_tx(t)
    if succeeded then return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:advance_ack(dt)
    if dt <= 0.0 or dt > 1.0 or (self.tx.phase ~= 5 and self.tx.phase ~= 6) then return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local t = self.tx
    local before = t.ack_time
    t.ack_time = t.ack_time + dt
    if t.phase == 5 and epok.math.truncate_unsigned(before / (1365.0 / 4096.0)) ~= epok.math.truncate_unsigned(t.ack_time / (1365.0 / 4096.0)) then
        t.phase = 6
        self:assign_tx(t)
        return self:decision(6, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    self:assign_tx(t)
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:receipt_checked(succeeded, serial, signature, claimed)
    if self.tx.phase ~= 6 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local t = self.tx
    if not succeeded or serial ~= t.serial then t.phase = 0 self:assign_tx(t) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if signature == 1095980081 and claimed >= t.total then
        t.phase = 0
        t.ack_pending = false
        self:assign_tx(t)
        return self:decision(0, 24, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if t.ack_time > 4.0 then t.phase = 0 self:assign_tx(t) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    t.phase = 5
    self:assign_tx(t)
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:load_result(candidate, succeeded, missing)
    if self.tx.phase ~= 0 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if missing then
        self:assign_saved(self:empty_save())
        self:assign_pending(self.saved)
        self:assign_loaded(true)
        self:intent(0, 0, false, 0, 0)
        return self:decision(0, 29, 0, 0, 0, self.tx.serial, self.tx.total, true)
    end
    if not succeeded then self:assign_loaded(false) return self:decision(0, 21, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not self:valid(candidate) then self:assign_loaded(false) return self:decision(0, 2, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    self:assign_saved(candidate)
    self:assign_pending(self.saved)
    self:assign_loaded(true)
    self:intent(0, 0, false, 0, 0)
    return self:decision(0, 28, 0, 0, 0, self.tx.serial, self.tx.total, true)
end
---@override
function DungeonEconomyPolicy:leave()
    if self.tx.phase > 1 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    self:assign_pending(self.saved)
    self:intent(0, 0, false, 0, 0)
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:pocket_begin(action, app_available, port)
    if self.pocket.operation ~= 0 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if port > 1 then return self:decision(0, 35, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if action ~= 2 and action ~= 3 and action ~= 5 then return self:decision(0, 27, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if (action == 2 or action == 3) and self.tx.phase ~= 0 then return self:decision(0, 26, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if action == 3 and not self.loaded then return self:decision(0, 1, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not app_available then
        if action == 5 then self:ack_started(false) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
        return self:decision(0, 31, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if action == 5 then
        local gate = self:ack_started(true)
        if self.tx.phase ~= 4 then return gate end
    end
    local q = self.pocket
    q.epoch = q.epoch + 1
    if q.epoch == 0 then q.epoch = 1 end
    if action == 2 then q.operation = 4 q.progress = 101 end
    if action == 3 then q.operation = 5 end
    if action == 5 then q.operation = 6 end
    q.port = port
    q.error = 0
    self:assign_pocket(q)
    if action == 2 then return self:decision(7, 37, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if action == 3 then return self:decision(8, 38, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    return self:decision(9, 39, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:pocket_progress(epoch, percent)
    if epoch ~= self.pocket.epoch or self.pocket.operation ~= 4 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if percent > 100 or percent == self.pocket.progress then return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local q = self.pocket
    q.progress = percent
    self:assign_pocket(q)
    return self:decision(0,  32,  0,  percent,  0,  self.tx.serial,  self.tx.total,  false)
end
---@override
function DungeonEconomyPolicy:pocket_request_failed(epoch, error)
    if epoch ~= self.pocket.epoch or self.pocket.operation < 4 or self.pocket.operation > 7 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local q = self.pocket
    local previous = q.operation
    q.operation = 0
    q.error = error
    self:assign_pocket(q)
    if previous == 6 then self:ack_sent(false) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if previous == 7 then self:receipt_checked(false, self.tx.serial, 0, 0) return self:decision(0, 23, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    return self:decision(0, 36, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
---@override
function DungeonEconomyPolicy:pocket_observe(sample)
    if sample.epoch ~= self.pocket.epoch or self.pocket.operation < 4 or self.pocket.operation > 7 then return self:decision(0, 25, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    if not sample.ready then
        if self.pocket.operation == 4 then return self:pocket_progress(sample.epoch, sample.percent) end
        return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
    end
    if sample.error ~= 0 then return self:pocket_request_failed(sample.epoch, sample.error) end
    local q = self.pocket
    local previous = q.operation
    q.operation = 0
    q.error = 0
    if previous == 4 then
        q.installed = q.installed + 1
        self:assign_pocket(q)
        return self:decision(0,  33,  0,  0,  0,  self.tx.serial,  self.tx.total,  true)
    end
    self:assign_pocket(q)
    if previous == 5 then return self:receive(sample.running_block == sample.expected_block, sample.signature, sample.version, sample.serial, sample.total, sample.claimed) end
    if previous == 6 then
        local result = self:ack_sent(true)
        if self.tx.phase == 5 then q.operation = 3 self:assign_pocket(q) end
        return result
    end
    local result = self:receipt_checked(true, sample.serial, sample.signature, sample.claimed)
    if self.tx.phase == 5 then q.operation = 3 self:assign_pocket(q) end
    return result
end
---@override
function DungeonEconomyPolicy:pocket_advance(dt)
    if self.pocket.operation ~= 3 and self.pocket.operation ~= 7 then return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false) end
    local result = self:advance_ack(dt)
    if result.action ~= 6 then return result end
    local q = self.pocket
    q.epoch = q.epoch + 1
    if q.epoch == 0 then q.epoch = 1 end
    q.operation = 7
    self:assign_pocket(q)
    return self:decision(10, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end

---@param error UInt32
---@param detail UInt32
function DungeonEconomyPolicy:wire_fail(error, detail)
    self:wire_set_word(14, error)
    self:wire_set_word(15, detail)
    self:wire_set_word(13, 1)
    self:wire_set_word(1, 0)
    return
end
function DungeonEconomyPolicy:wire_finish()
    self:wire_fail(0, 0)
    return
end
---@param command UInt32
function DungeonEconomyPolicy:wire_packet(command)
    self:wire_set_word(24, 0)
    for repeat_18 = 1, 140 do
        self:wire_set_byte((2448 + self:wire_word(24)), 0)
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_byte((2448 + 0), 129)
    self:wire_set_byte((2448 + 1), command)
    return
end
---@param next UInt32
---@param length UInt32
---@param length_at Int32
function DungeonEconomyPolicy:wire_send(next, length, length_at)
    if not (self:wire_channel_exists()) then
        self:wire_fail(11, 0)
        return
    end
    if not (self.pocket_channel:clear_request()) then
        self:wire_fail(12, 0)
        return
    end
    self:wire_set_word(24, 0)
    for repeat_19 = 1, 140 do
        if (self:wire_word(24) < length) then
            if not (self.pocket_channel:set_request_byte(self:wire_word(24), self:wire_byte((2448 + self:wire_word(24))))) then
                self:wire_fail(11, 0)
                return
            end
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_word(1, next)
    self:wire_assign_token(self.pocket_channel:begin(length, length_at))
    if (self:wire_token().request == 0) then
        self:wire_fail(self:wire_error_code(self.pocket_channel:last_rejection()), 0)
    end
    return
end
---@param next UInt32
---@param address UInt32
function DungeonEconomyPolicy:wire_read(next, address)
    self:wire_set_word(10, address)
    self:wire_packet(82)
    self:wire_set_byte((2448 + 4), (address / 256))
    self:wire_set_byte((2448 + 5), (address % 256))
    self:wire_send(next, 140, -1)
    return
end
---@param next UInt32
function DungeonEconomyPolicy:wire_status(next)
    self:wire_packet(90)
    self:wire_send(next, 21, -1)
    return
end
function DungeonEconomyPolicy:wire_values()
    self:wire_packet(91)
    self:wire_set_byte((2448 + 2), 130)
    self:wire_send(4, 70, 4)
    return
end
---@param next UInt32
function DungeonEconomyPolicy:wire_launch(next)
    self:wire_packet(89)
    self:wire_set_byte((2448 + 4), self:wire_word(9))
    self:wire_send(next, 9, -1)
    return
end
function DungeonEconomyPolicy:wire_wait_start()
    self:wire_set_word(17, (self:wire_word(17) + 1))
    if (self:wire_word(17) > 120) then
        self:wire_fail(5, 40)
        return
    end
    self:wire_status(18)
    return
end
---@return Bool
function DungeonEconomyPolicy:wire_valid_sector()
    if ((((self:wire_word(23) ~= 140) or (self:wire_byte((0 + 139)) ~= 71)) or (self:wire_byte((0 + 8)) ~= (self:wire_word(10) / 256))) or (self:wire_byte((0 + 9)) ~= (self:wire_word(10) % 256))) then
        self:wire_fail(5, 0)
        return false
    end
    self:wire_set_word(25, epok.math.bit_xor((self:wire_word(10) / 256), (self:wire_word(10) % 256)))
    self:wire_set_word(24, 0)
    for repeat_20 = 1, 128 do
        self:wire_set_word(25, epok.math.bit_xor(self:wire_word(25), self:wire_byte((0 + (self:wire_word(24) + 10)))))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if (self:wire_word(25) ~= self:wire_byte((0 + 138))) then
        self:wire_fail(5, 0)
        return false
    end
    return true
end
---@return Bool
function DungeonEconomyPolicy:wire_parse_status()
    if ((self:wire_word(23) ~= 21) or (self:wire_byte((0 + 2)) ~= 18)) then
        self:wire_fail(2, 0)
        return false
    end
    self:wire_set_word(28, ((self:wire_byte((0 + 3)) * 256) + self:wire_byte((0 + 4))))
    self:wire_set_word(29, self:wire_u32(9))
    return true
end
function DungeonEconomyPolicy:wire_capture_status()
    self:wire_set_word(21, self:wire_word(28))
    self:wire_set_word(22, self:wire_word(29))
    return
end
---@param at UInt32
---@return UInt32
function DungeonEconomyPolicy:wire_source(at)
    if (((self:wire_word(11) ~= 0) and (self:wire_word(2) == 0)) and (self:wire_word(4) == (self:wire_app_save_offset() / 128))) then
        return self:wire_byte((2304 + at))
    end
    return self:wire_app_image_byte((((self:wire_word(2) * 8192) + (self:wire_word(4) * 128)) + at))
end
---@param next UInt32
---@param address UInt32
---@param source UInt32
function DungeonEconomyPolicy:wire_write(next, address, source)
    self:wire_packet(87)
    self:wire_set_byte((2448 + 4), (address / 256))
    self:wire_set_byte((2448 + 5), (address % 256))
    self:wire_set_word(25, epok.math.bit_xor((address / 256), (address % 256)))
    self:wire_set_word(24, 0)
    for repeat_21 = 1, 128 do
        if (source == 0) then
            self:wire_set_word(30, self:wire_source(self:wire_word(24)))
        else
            self:wire_set_word(30, self:wire_byte((source + self:wire_word(24))))
        end
        self:wire_set_byte((2448 + (self:wire_word(24) + 6)), self:wire_word(30))
        self:wire_set_word(25, epok.math.bit_xor(self:wire_word(25), self:wire_word(30)))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_byte((2448 + 134), self:wire_word(25))
    self:wire_send(next, 138, -1)
    return
end
function DungeonEconomyPolicy:wire_write_data()
    self:wire_write(6, ((self:wire_byte((2432 + self:wire_word(2))) * 64) + self:wire_word(4)), 0)
    return
end
---@param block UInt32
---@return Bool
function DungeonEconomyPolicy:wire_named(block)
    self:wire_set_word(24, 0)
    for repeat_22 = 1, 20 do
        if (self:wire_byte((((256 + (block * 128)) + 10) + self:wire_word(24))) ~= self:wire_app_name_byte(self:wire_word(24))) then
            return false
        end
        if (self:wire_app_name_byte(self:wire_word(24)) == 0) then
            return true
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    return (self:wire_app_name_byte(20) == 0)
end
function DungeonEconomyPolicy:wire_allocate()
    self:wire_set_word(3, (self:wire_app_size() / 8192))
    self:wire_set_word(26, 0)
    self:wire_set_word(24, 0)
    for repeat_23 = 1, 15 do
        self:wire_set_word(31, (self:wire_word(24) + 1))
        if ((self:wire_word(26) < self:wire_word(3)) and ((self:wire_byte((256 + (self:wire_word(31) * 128))) / 16) == 10)) then
            self:wire_set_byte((2432 + self:wire_word(26)), self:wire_word(31))
            self:wire_set_word(26, (self:wire_word(26) + 1))
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if (self:wire_word(26) ~= self:wire_word(3)) then
        if (self:wire_word(8) ~= 0) then
            self:wire_fail(8, 41)
            return
        else
            self:wire_fail(8, 0)
            return
        end
    end
    self:wire_set_word(9, self:wire_byte(2432))
    self:wire_set_word(2, 0)
    self:wire_set_word(4, 0)
    self:wire_write_data()
    return
end
function DungeonEconomyPolicy:wire_existing()
    self:wire_set_word(4, 0)
    self:wire_set_word(18, 2166136261)
    self:wire_set_word(19, 2166136261)
    self:wire_set_word(20, 2166136261)
    self:wire_read(13, (self:wire_word(8) * 64))
    return
end
function DungeonEconomyPolicy:wire_directory_ready()
    self:wire_set_word(27, 0)
    self:wire_set_word(9, 0)
    self:wire_set_word(26, 0)
    self:wire_set_word(2, 0)
    for repeat_24 = 1, 15 do
        self:wire_set_word(31, (self:wire_word(2) + 1))
        if (self:wire_byte((256 + (self:wire_word(31) * 128))) == 81) then
            if self:wire_named(self:wire_word(31)) then
                self:wire_set_word(27, (self:wire_word(27) + 1))
                self:wire_set_word(26, self:wire_word(31))
            end
        end
        self:wire_set_word(2, (self:wire_word(2) + 1))
    end
    if (self:wire_word(27) > 1) then
        self:wire_fail(5, 42)
        return
    end
    self:wire_set_word(9, self:wire_word(26))
    if (self:wire_word(9) ~= 0) then
        if (self:wire_word(0) ~= 0) then
            self:wire_status(3)
            return
        end
        if (self:wire_app_save_offset() == 0) then
            self:wire_launch(8)
            return
        end
        if ((((self:wire_app_size() ~= 8192) or (self:wire_u32(((256 + (self:wire_word(9) * 128)) + 4)) ~= 8192)) or (self:wire_byte(((256 + (self:wire_word(9) * 128)) + 8)) ~= 255)) or (self:wire_byte(((256 + (self:wire_word(9) * 128)) + 9)) ~= 255)) then
            self:wire_fail(5, 43)
            return
        end
        self:wire_set_word(8, self:wire_word(9))
        self:wire_status(10)
        return
    end
    if (self:wire_word(0) ~= 0) then
        self:wire_fail(7, 0)
        return
    end
    self:wire_allocate()
    return
end
---@param item UInt32
function DungeonEconomyPolicy:wire_commit_frame(item)
    self:wire_set_word(31, (256 + (self:wire_byte((2432 + item)) * 128)))
    self:wire_set_word(24, 0)
    for repeat_25 = 1, 128 do
        self:wire_set_byte((self:wire_word(31) + self:wire_word(24)), 0)
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if (item == 0) then
        self:wire_set_byte(self:wire_word(31), 81)
        self:wire_set_u32((self:wire_word(31) + 4), self:wire_app_size())
        self:wire_set_word(24, 0)
        for repeat_26 = 1, 20 do
            self:wire_set_byte(((self:wire_word(31) + 10) + self:wire_word(24)), self:wire_app_name_byte(self:wire_word(24)))
            self:wire_set_word(24, (self:wire_word(24) + 1))
        end
    else
        if ((item + 1) == self:wire_word(3)) then
            self:wire_set_byte(self:wire_word(31), 83)
        else
            self:wire_set_byte(self:wire_word(31), 82)
        end
    end
    if ((item + 1) < self:wire_word(3)) then
        self:wire_set_byte((self:wire_word(31) + 8), (self:wire_byte(((2432 + item) + 1)) - 1))
        self:wire_set_byte((self:wire_word(31) + 9), 0)
    else
        self:wire_set_byte((self:wire_word(31) + 8), 255)
        self:wire_set_byte((self:wire_word(31) + 9), 255)
    end
    self:wire_set_word(25, 0)
    self:wire_set_word(24, 0)
    for repeat_27 = 1, 127 do
        self:wire_set_word(25, epok.math.bit_xor(self:wire_word(25), self:wire_byte((self:wire_word(31) + self:wire_word(24)))))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_byte((self:wire_word(31) + 127), self:wire_word(25))
    return
end
function DungeonEconomyPolicy:wire_commit()
    self:wire_set_word(2, 0)
    for repeat_28 = 1, 15 do
        if (self:wire_word(2) < self:wire_word(3)) then
            self:wire_commit_frame(self:wire_word(2))
        end
        self:wire_set_word(2, (self:wire_word(2) + 1))
    end
    self:wire_set_word(2, (self:wire_word(3) - 1))
    self:wire_write(7, self:wire_byte((2432 + self:wire_word(2))), (256 + (self:wire_byte((2432 + self:wire_word(2))) * 128)))
    return
end
function DungeonEconomyPolicy:wire_retire()
    self:wire_set_word(31, (256 + (self:wire_word(8) * 128)))
    self:wire_set_byte(self:wire_word(31), 161)
    self:wire_set_word(25, 0)
    self:wire_set_word(24, 0)
    for repeat_29 = 1, 127 do
        self:wire_set_word(25, epok.math.bit_xor(self:wire_word(25), self:wire_byte((self:wire_word(31) + self:wire_word(24)))))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_byte((self:wire_word(31) + 127), self:wire_word(25))
    self:wire_write(16, self:wire_word(8), self:wire_word(31))
    return
end
---@override
function DungeonEconomyPolicy:protocol_start(mode, amount, expected_serial)
    if ((self:wire_word(13) == 0) and (self:wire_word(1) ~= 0)) then
        return false
    end
    if not (self:wire_channel_exists()) then
        self:wire_fail(11, 0)
        return false
    end
    self:wire_set_word(0, 0)
    self:wire_set_word(1, 0)
    self:wire_set_word(2, 0)
    self:wire_set_word(3, 0)
    self:wire_set_word(4, 0)
    self:wire_set_word(5, 0)
    self:wire_set_word(6, 0)
    self:wire_set_word(7, 0)
    self:wire_set_word(8, 0)
    self:wire_set_word(9, 0)
    self:wire_set_word(10, 0)
    self:wire_set_word(11, 0)
    self:wire_set_word(12, 0)
    self:wire_set_word(13, 0)
    self:wire_set_word(14, 0)
    self:wire_set_word(15, 0)
    self:wire_set_word(16, 0)
    self:wire_set_word(17, 0)
    self:wire_set_word(18, 0)
    self:wire_set_word(19, 0)
    self:wire_set_word(20, 0)
    self:wire_set_word(21, 0)
    self:wire_set_word(22, 0)
    self:wire_set_word(23, 0)
    self:wire_set_word(24, 0)
    self:wire_set_word(25, 0)
    self:wire_set_word(26, 0)
    self:wire_set_word(27, 0)
    self:wire_set_word(28, 0)
    self:wire_set_word(29, 0)
    self:wire_set_word(30, 0)
    self:wire_set_word(31, 0)
    self:wire_set_word(0, mode)
    self:wire_set_word(7, amount)
    self:wire_set_word(6, expected_serial)
    if ((((((mode > 3) or not (self:wire_app_available())) or (self:wire_app_size() == 0)) or ((self:wire_app_size() % 8192) ~= 0)) or (self:wire_app_size() > 122880)) or ((self:wire_app_save_offset() ~= 0) and (((self:wire_app_save_offset() % 128) ~= 0) or (self:wire_app_save_offset() > (self:wire_app_size() - 128))))) then
        self:wire_fail(11, 0)
        return false
    end
    if (mode >= 2) then
        self:wire_status(3)
    else
        self:wire_packet(88)
        self:wire_send(1, 5, -1)
    end
    return (self:wire_word(13) == 0)
end
---@override
function DungeonEconomyPolicy:protocol_cancel()
    if self:wire_channel_exists() then
        self.pocket_channel:cancel(self:wire_token())
    end
    self:wire_fail(11, 44)
    return
end
---@override
function DungeonEconomyPolicy:protocol_percent()
    if ((self:wire_word(13) ~= 0) and (self:wire_word(14) == 0)) then
        return 100
    end
    if (self:wire_word(3) == 0) then
        return 0
    end
    return ((self:wire_word(5) * 99) / (self:wire_word(3) * 64))
end
---@override
function DungeonEconomyPolicy:protocol_observation()
    return self:wire_make_observation((self:wire_word(13) ~= 0), self:wire_epoch(), self:wire_word(14), self:wire_word(21), self:wire_word(9), self:wire_word(22), self:wire_u32(2608), self:wire_u32(2616), self:wire_u32(2600), self:wire_u32(2612), self:protocol_percent())
end
function DungeonEconomyPolicy:wire_phase_probe()
    if ((self:wire_word(23) ~= 5) or (self:wire_byte((0 + 2)) ~= 2)) then
        self:wire_fail(2, 0)
        return
    end
    self:wire_set_word(2, 0)
    self:wire_read(2, 0)
    return
end
function DungeonEconomyPolicy:wire_phase_directory()
    if not (self:wire_valid_sector()) then
        return
    end
    self:wire_set_word(25, 0)
    self:wire_set_word(24, 0)
    for repeat_30 = 1, 128 do
        self:wire_set_byte(((256 + (self:wire_word(2) * 128)) + self:wire_word(24)), self:wire_byte((0 + (self:wire_word(24) + 10))))
        if (self:wire_word(24) < 127) then
            self:wire_set_word(25, epok.math.bit_xor(self:wire_word(25), self:wire_byte((0 + (self:wire_word(24) + 10)))))
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if (self:wire_word(25) ~= self:wire_byte(((256 + (self:wire_word(2) * 128)) + 127))) then
        self:wire_fail(5, 0)
        return
    end
    if ((self:wire_word(2) == 0) and ((self:wire_byte(256) ~= 77) or (self:wire_byte(257) ~= 67))) then
        self:wire_fail(6, 0)
        return
    end
    self:wire_set_word(2, (self:wire_word(2) + 1))
    if (self:wire_word(2) < 16) then
        self:wire_read(2, self:wire_word(2))
    else
        self:wire_directory_ready()
    end
    return
end
function DungeonEconomyPolicy:wire_phase_upgrade_status()
    if not (self:wire_parse_status()) then
        return
    end
    self:wire_capture_status()
    if (self:wire_word(21) == self:wire_word(8)) then
        self:wire_packet(89)
        self:wire_set_byte((2448 + 5), (48 + self:wire_word(8)))
        self:wire_send(11, 9, -1)
    else
        self:wire_existing()
    end
    return
end
function DungeonEconomyPolicy:wire_phase_stop()
    if ((self:wire_word(23) ~= 9) or (self:wire_byte((0 + 2)) ~= 6)) then
        self:wire_fail(5, 0)
        return
    end
    self:wire_status(12)
    return
end
function DungeonEconomyPolicy:wire_phase_stopped()
    if not (self:wire_parse_status()) then
        return
    end
    if (self:wire_word(29) ~= self:wire_word(22)) then
        self:wire_fail(5, 45)
        return
    end
    if (self:wire_word(28) == self:wire_word(8)) then
        self:wire_set_word(16, (self:wire_word(16) + 1))
        if (self:wire_word(16) >= 120) then
            self:wire_fail(9, 0)
            return
        end
        self:wire_status(12)
    else
        self:wire_existing()
    end
    return
end
function DungeonEconomyPolicy:wire_phase_existing()
    if not (self:wire_valid_sector()) then
        return
    end
    self:wire_set_word(24, 0)
    for repeat_31 = 1, 128 do
        if (self:wire_word(4) ~= 42) then
            self:wire_set_word(18, epok.math.wrapping_multiply(epok.math.bit_xor(self:wire_word(18), self:wire_byte((0 + (self:wire_word(24) + 10)))), 16777619))
        else
            self:wire_set_byte((2304 + self:wire_word(24)), self:wire_byte((0 + (self:wire_word(24) + 10))))
        end
        if (self:wire_word(4) ~= (self:wire_app_save_offset() / 128)) then
            self:wire_set_word(19, epok.math.wrapping_multiply(epok.math.bit_xor(self:wire_word(19), self:wire_byte((0 + (self:wire_word(24) + 10)))), 16777619))
            self:wire_set_word(20, epok.math.wrapping_multiply(epok.math.bit_xor(self:wire_word(20), self:wire_app_image_byte(((self:wire_word(4) * 128) + self:wire_word(24)))), 16777619))
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_word(4, (self:wire_word(4) + 1))
    if (self:wire_word(4) < 64) then
        self:wire_read(13, ((self:wire_word(8) * 64) + self:wire_word(4)))
        return
    end
    if (self:wire_word(19) == self:wire_word(20)) then
        self:wire_launch(8)
        return
    end
    if (self:wire_word(18) ~= 2567055923) then
        self:wire_fail(5, 46)
        return
    end
    if (((self:wire_u32(2304) > 1000000) or (self:wire_u32(2308) > 1000000)) or (self:wire_u32(2312) > self:wire_u32(2308))) then
        self:wire_fail(5, 47)
        return
    end
    self:wire_set_word(11, 1)
    self:wire_status(14)
    return
end
function DungeonEconomyPolicy:wire_phase_identity()
    if not (self:wire_parse_status()) then
        return
    end
    if (self:wire_word(29) ~= self:wire_word(22)) then
        self:wire_fail(5, 45)
        return
    end
    if (self:wire_word(28) == self:wire_word(8)) then
        self:wire_fail(9, 0)
        return
    end
    self:wire_allocate()
    return
end
function DungeonEconomyPolicy:wire_phase_verify()
    if not (self:wire_valid_sector()) then
        return
    end
    self:wire_set_word(24, 0)
    for repeat_32 = 1, 128 do
        if (self:wire_byte((0 + (self:wire_word(24) + 10))) ~= self:wire_source(self:wire_word(24))) then
            self:wire_fail(5, 48)
            return
        end
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    self:wire_set_word(4, (self:wire_word(4) + 1))
    if (self:wire_word(4) == 64) then
        self:wire_set_word(4, 0)
        self:wire_set_word(2, (self:wire_word(2) + 1))
    end
    if (self:wire_word(2) < self:wire_word(3)) then
        self:wire_read(15, ((self:wire_byte((2432 + self:wire_word(2))) * 64) + self:wire_word(4)))
    else
        self:wire_commit()
    end
    return
end
function DungeonEconomyPolicy:wire_phase_retire()
    if ((self:wire_word(23) ~= 138) or (self:wire_byte((0 + 137)) ~= 71)) then
        self:wire_fail(4, 0)
        return
    end
    self:wire_launch(8)
    return
end
function DungeonEconomyPolicy:wire_phase_status()
    if not (self:wire_parse_status()) then
        return
    end
    self:wire_capture_status()
    if ((self:wire_word(6) ~= 0) and (self:wire_word(22) ~= self:wire_word(6))) then
        self:wire_fail(5, 0)
        return
    end
    if ((self:wire_word(0) == 1) and (self:wire_word(21) ~= self:wire_word(9))) then
        if (self:wire_word(21) ~= 0) then
            self:wire_fail(5, 49)
            return
        end
        self:wire_set_word(12, 1)
        self:wire_launch(17)
        return
    end
    if (self:wire_word(0) == 2) then
        self:wire_packet(92)
        self:wire_set_byte((2448 + 2), 128)
        self:wire_set_byte((2448 + 4), 5)
        self:wire_set_byte((2448 + 6), 209)
        self:wire_set_u32(2455, self:wire_word(7))
        self:wire_send(9, 12, -1)
    else
        self:wire_values()
    end
    return
end
function DungeonEconomyPolicy:wire_phase_start()
    if ((self:wire_word(23) ~= 9) or (self:wire_byte((0 + 2)) ~= 6)) then
        self:wire_fail(5, 0)
        return
    end
    self:wire_wait_start()
    return
end
function DungeonEconomyPolicy:wire_phase_await()
    if not (self:wire_parse_status()) then
        return
    end
    if (self:wire_word(29) ~= self:wire_word(22)) then
        self:wire_fail(5, 50)
        return
    end
    if (self:wire_word(28) == self:wire_word(9)) then
        self:wire_capture_status()
        self:wire_values()
    else
        if (self:wire_word(28) == 0) then
            self:wire_wait_start()
        else
            self:wire_fail(5, 51)
            return
        end
    end
    return
end
function DungeonEconomyPolicy:wire_phase_shared()
    if ((((self:wire_word(23) ~= 70) or (self:wire_byte((0 + 2)) ~= 255)) or (self:wire_byte((0 + 3)) ~= 0)) or (self:wire_byte((0 + 4)) ~= 64)) then
        if (self:wire_word(12) ~= 0) then
            self:wire_wait_start()
        else
            self:wire_fail(5, 0)
            return
        end
        return
    end
    self:wire_set_word(24, 0)
    for repeat_33 = 1, 64 do
        self:wire_set_byte((2600 + self:wire_word(24)), self:wire_byte((0 + (self:wire_word(24) + 5))))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if ((self:wire_word(12) ~= 0) and ((self:wire_u32(2608) ~= 1095980081) or (self:wire_u32(2616) ~= 1))) then
        self:wire_wait_start()
    else
        self:wire_status(5)
    end
    return
end
function DungeonEconomyPolicy:wire_phase_confirm()
    if not (self:wire_parse_status()) then
        return
    end
    if ((self:wire_word(29) ~= self:wire_word(22)) or (self:wire_word(28) ~= self:wire_word(21))) then
        self:wire_fail(5, 0)
        return
    end
    self:wire_finish()
    return
end
function DungeonEconomyPolicy:wire_phase_data()
    if ((self:wire_word(23) ~= 138) or (self:wire_byte((0 + 137)) ~= 71)) then
        self:wire_fail(4, 0)
        return
    end
    self:wire_set_word(5, (self:wire_word(5) + 1))
    self:wire_set_word(4, (self:wire_word(4) + 1))
    if (self:wire_word(4) == 64) then
        self:wire_set_word(4, 0)
        self:wire_set_word(2, (self:wire_word(2) + 1))
    end
    if (self:wire_word(2) < self:wire_word(3)) then
        self:wire_write_data()
    else
        self:wire_set_word(2, 0)
        self:wire_set_word(4, 0)
        self:wire_read(15, (self:wire_byte(2432) * 64))
    end
    return
end
function DungeonEconomyPolicy:wire_phase_commit()
    if ((self:wire_word(23) ~= 138) or (self:wire_byte((0 + 137)) ~= 71)) then
        self:wire_fail(4, 0)
        return
    end
    if (self:wire_word(2) ~= 0) then
        self:wire_set_word(2, (self:wire_word(2) - 1))
        self:wire_write(7, self:wire_byte((2432 + self:wire_word(2))), (256 + (self:wire_byte((2432 + self:wire_word(2))) * 128)))
        return
    end
    if (self:wire_word(11) ~= 0) then
        self:wire_retire()
    else
        self:wire_launch(8)
    end
    return
end
function DungeonEconomyPolicy:wire_phase_launch()
    if ((self:wire_word(23) ~= 9) or (self:wire_byte((0 + 2)) ~= 6)) then
        self:wire_fail(5, 0)
        return
    end
    self:wire_finish()
    return
end
function DungeonEconomyPolicy:wire_phase_send()
    if ((((self:wire_word(23) ~= 12) or (self:wire_byte((0 + 2)) ~= 255)) or (self:wire_byte((0 + 3)) ~= 1)) or (self:wire_byte((0 + 5)) ~= 5)) then
        self:wire_fail(5, 0)
        return
    end
    self:wire_finish()
    return
end
---@override
function DungeonEconomyPolicy:protocol_tick(microseconds)
    if ((self:wire_word(13) ~= 0) or (self:wire_word(1) == 0)) then
        return (self:wire_word(13) ~= 0)
    end
    if not (self:wire_channel_exists()) then
        self:wire_fail(11, 0)
        return true
    end
    self.pocket_channel:poll(self:wire_token(), microseconds)
    local sample = self.pocket_channel:snapshot(self:wire_token())
    if not (sample.valid) then
        self:wire_fail(11, 0)
        return true
    end
    if not (sample.ready) then
        return false
    end
    self:wire_set_word(14, self:wire_error_code(sample.error))
    if (self:wire_word(14) ~= 0) then
        self:wire_fail(self:wire_word(14), 0)
        return true
    end
    self:wire_set_word(23, sample.received)
    self:wire_set_word(24, 0)
    for repeat_34 = 1, 140 do
        self:wire_set_byte(self:wire_word(24), self.pocket_channel:reply_byte(self:wire_token(), self:wire_word(24)))
        self:wire_set_word(24, (self:wire_word(24) + 1))
    end
    if (self:wire_word(1) == 1) then
        self:wire_phase_probe()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 2) then
        self:wire_phase_directory()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 3) then
        self:wire_phase_status()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 4) then
        self:wire_phase_shared()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 5) then
        self:wire_phase_confirm()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 6) then
        self:wire_phase_data()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 7) then
        self:wire_phase_commit()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 8) then
        self:wire_phase_launch()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 9) then
        self:wire_phase_send()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 10) then
        self:wire_phase_upgrade_status()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 11) then
        self:wire_phase_stop()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 12) then
        self:wire_phase_stopped()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 13) then
        self:wire_phase_existing()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 14) then
        self:wire_phase_identity()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 15) then
        self:wire_phase_verify()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 16) then
        self:wire_phase_retire()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 17) then
        self:wire_phase_start()
        return (self:wire_word(13) ~= 0)
    end
    if (self:wire_word(1) == 18) then
        self:wire_phase_await()
        return (self:wire_word(13) ~= 0)
    end
    self:wire_fail(11, 0)
    return true
end
---@override
function DungeonEconomyPolicy:pocket_execute(action, port)
    local gate = self:pocket_begin(action, self:wire_app_available(), port)
    if (((gate.action ~= 7) and (gate.action ~= 8)) and (gate.action ~= 9)) then
        return gate
    end
    self:wire_configure_channel(port, false)
    if (action == 2) then
        self:protocol_start(0, 0, 0)
    end
    if (action == 3) then
        self:protocol_start(1, 0, 0)
    end
    if (action == 5) then
        self:protocol_start(2, self.tx.total, self.tx.serial)
    end
    if (self:wire_word(13) ~= 0) then
        return self:pocket_request_failed(self.pocket.epoch, self:wire_word(14))
    end
    return gate
end
---@override
function DungeonEconomyPolicy:pocket_update(dt, microseconds)
    if (self.pocket.operation >= 4) then
        if (self.pocket.operation == 7) then
            self:advance_ack(dt)
        end
        self:protocol_tick(microseconds)
        if (((self:wire_word(13) ~= 0) and (self:wire_word(14) ~= 0)) and ((self.pocket.operation == 4) or (self.pocket.operation == 5))) then
            local failed = self:pocket_request_failed(self.pocket.epoch, self:wire_word(14))
            if (self:wire_word(15) ~= 0) then
                return self:decision(0, self:wire_word(15), 0, 0, 0, self.tx.serial, self.tx.total, false)
            end
            return failed
        end
        return self:pocket_observe(self:protocol_observation())
    end
    if (self.pocket.operation == 3) then
        local next = self:pocket_advance(dt)
        if (next.action == 10) then
            self:wire_configure_channel(self.pocket.port, false)
            if not (self:protocol_start(3, 0, self.tx.serial)) then
                return self:pocket_request_failed(self.pocket.epoch, self:wire_word(14))
            end
            return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
        end
        return next
    end
    return self:decision(0, 0, 0, 0, 0, self.tx.serial, self.tx.total, false)
end
return DungeonEconomyPolicy
