---@class CoinPickup : epok.PickupComponent
local CoinPickup = epok.PickupComponent:extend()

-- The shared pickup validates the player and commits collection once.
-- Edit this project callback to add rewards or different presentation.

---@override
function CoinPickup:collected(initiator, port)
    local coin = epok.cast(epok.owner(), "epok::Actor")
    coin:set_active(false)
end

return CoinPickup
