---@class DungeonRunFlow : DungeonRunFlowBase
local DungeonRunFlow = DungeonRunFlowBase:extend()

---@return Bool
function DungeonRunFlow:timing_valid()
    local t = self.timing
    return t.music_lead >= 0.0 and t.music_lead <= 60.0
        and t.fade_in >= 0.0 and t.fade_in <= 60.0
        and t.control_delay >= 0.0 and t.control_delay <= 60.0
        and t.death_hold >= 0.0 and t.death_hold <= 60.0
        and t.fade_out >= 0.0 and t.fade_out <= 60.0
        and t.restart_hold >= 0.0 and t.restart_hold <= 60.0
        and self.reactor_entry_z >= -4096.0 and self.reactor_entry_z <= 4096.0
end

---@return Fixed
function DungeonRunFlow:phase_duration()
    local phase = self.status.phase
    if phase == 2 then return self.timing.fade_in end
    if phase == 3 then return self.timing.control_delay end
    if phase == 5 then return self.timing.death_hold end
    if phase == 6 then return self.timing.fade_out end
    if phase == 7 then return self.timing.restart_hold end
    return self.timing.music_lead
end

---@param duration Fixed
---@param elapsed Fixed
---@return UInt32
function DungeonRunFlow:fade_progress(duration, elapsed)
    if duration <= 0.0 then return 255 end
    local time = elapsed
    if time > duration then time = duration end
    return epok.math.truncate_unsigned(time * 255.0 / duration)
end

---@return UInt32
function DungeonRunFlow:opacity()
    local s = self.status
    if s.phase == 0 or s.phase == 1 or s.phase == 7 then return 255 end
    if s.phase == 2 then return 255 - self:fade_progress(self.timing.fade_in, s.elapsed) end
    if s.phase == 6 then return self:fade_progress(self.timing.fade_out, s.elapsed) end
    return 0
end

---@return Fixed
function DungeonRunFlow:gain()
    local s = self.status
    if s.phase == 7 then return 0.0 end
    if s.phase == 6 then
        local duration = self.timing.fade_out
        if duration <= 0.0 then return 0.0 end
        local time = s.elapsed
        if time > duration then time = duration end
        return 1.0 - time / duration
    end
    return 1.0
end

function DungeonRunFlow:refresh()
    local s = self.status
    s.valid = self:timing_valid() and s.phase <= 7 and s.state <= 3
        and s.elapsed >= 0.0 and s.elapsed <= 61.0
    s.playing = s.phase == 4
    s.alive = s.state < 2
    s.restart = false
    s.stop_music = s.phase == 7
    s.fade = self:opacity()
    s.music_gain = self:gain()
    self:assign_status(s)
end

---@override
function DungeonRunFlow:start()
    local s = self.status
    s.state = 0
    s.phase = 0
    s.elapsed = 0.0
    self:assign_status(s)
    self:refresh()
end

---@override
function DungeonRunFlow:die()
    local s = self.status
    if s.state >= 2 then return end
    s.state = 3
    s.phase = 5
    s.elapsed = 0.0
    self:assign_status(s)
    self:refresh()
end

---@override
function DungeonRunFlow:win()
    local s = self.status
    if s.state ~= 1 then return end
    s.state = 2
    self:assign_status(s)
    self:refresh()
end

---@override
function DungeonRunFlow:try_enter_boss(has_key, player_z)
    local s = self.status
    if s.state ~= 0 or not has_key or player_z <= self.reactor_entry_z then return false end
    s.state = 1
    self:assign_status(s)
    self:refresh()
    return true
end

---@override
function DungeonRunFlow:restart_requested(pressed)
    return pressed and self.status.playing and self.status.state >= 2
end

---@override
function DungeonRunFlow:advance(dt, music_playing)
    if dt <= 0.0 or dt > 1.0 then return end
    self:refresh()
    local s = self.status
    if not s.valid then return end
    if s.phase == 0 then
        if music_playing then
            s.phase = 1
            s.elapsed = 0.0
            self:assign_status(s)
        end
        self:refresh()
        return
    end
    if s.phase == 4 then return end
    local duration = self:phase_duration()
    s.elapsed = s.elapsed + dt
    self:assign_status(s)
    if s.elapsed < duration then self:refresh() return end
    s.elapsed = s.elapsed - duration
    self:assign_status(s)
    if s.phase == 7 then
        self:refresh()
        s = self.status
        s.restart = true
        self:assign_status(s)
        return
    end
    s.phase = s.phase + 1
    if s.phase == 4 then s.elapsed = 0.0 end
    self:assign_status(s)
    self:refresh()
end

return DungeonRunFlow
