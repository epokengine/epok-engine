---@class CoinPickup : epok.ActorComponent
local CoinPickup = epok.ActorComponent:extend()

-- A collectible: the first time anything enters its trigger shape, the coin's
-- actor deactivates, which hides it and removes its collider.
-- Trigger phases are written as the numbers they are stored as: 0 Enter.

---@override
function CoinPickup:trigger_event(other, phase)
    if phase == 0 then
        local coin = epok.cast(epok.owner(), "epok::Actor2D")
        coin:set_active(false)
    end
end

return CoinPickup
