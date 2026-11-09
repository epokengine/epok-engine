---@class DungeonDoor : epok.DoorComponent
local DungeonDoor = epok.DoorComponent:extend()

DungeonDoor.lock_id = epok.UInt32(0)

---@override
function DungeonDoor:tick(delta_seconds)
    if epok.is_valid(self.requirement_flag) and self:requirement_met() then
        self:open(self:owner_id())
    end
    DungeonDoor.super.tick(self, delta_seconds)
end

---@override
function DungeonDoor:door_changed(previous, current)
    if current == epok.DoorState.Open then
        local owner = epok.cast(epok.owner(), "epok::Actor")
        if epok.is_valid(owner) then
            owner:set_active(false)
        end
    end
end

return DungeonDoor
