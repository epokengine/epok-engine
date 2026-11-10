---@class DungeonEconomyStation : DungeonEconomyStationBase
local DungeonEconomyStation = DungeonEconomyStationBase:extend()
---@override
function DungeonEconomyStation:can_interact(initiator)
    if not epok.is_valid(self.hud) then return false end
    if self.horizontal_range <= 0.0 or self.height_range <= 0.0 then return false end
    local sample = self:measure(initiator)
    if not sample.valid or sample.vertical <= -self.height_range or sample.vertical >= self.height_range then return false end
    local limit = self:squared_range(self.horizontal_range)
    if sample.high > limit.high or (sample.high == limit.high and sample.low >= limit.low) then return false end
    return self.hud:station_available(self.input_port)
end
---@override
function DungeonEconomyStation:interaction_started(initiator,port)
    if epok.is_valid(self.hud) then self.hud:open_station(self.menu,port) end
end
---@override
function DungeonEconomyStation:focus_gained(initiator,port)
    if epok.is_valid(self.hud) then self.hud:station_focus(self.menu,true) end
end
---@override
function DungeonEconomyStation:focus_lost(initiator,port)
    if epok.is_valid(self.hud) then self.hud:station_focus(self.menu,false) end
end
return DungeonEconomyStation
