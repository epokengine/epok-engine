---@class DungeonKey : epok.PickupComponent
local DungeonKey = epok.PickupComponent:extend()

DungeonKey.lock_id = epok.UInt32(0)

---@override
function DungeonKey:collected(initiator, port)
    local owner = epok.cast(epok.owner(), "epok::Actor")
    if epok.is_valid(owner) then
        owner:set_active(false)
    end
end

return DungeonKey
