---@class DungeonKey : epok.ActorComponent
local DungeonKey = epok.ActorComponent:extend()

-- A dungeon key. The first time its `player` enters its trigger it is picked up: its
-- actor deactivates, which hides it and removes its collider, and every door whose
-- key is this actor opens. While `player` names no live actor, anything entering
-- picks it up. The Dungeon Generator sets `lock`, makes its Player the `player` and
-- points the doors at their key. The C++ and Blueprint flavors behave the same way.
-- Trigger phases are written as the numbers they are stored as: 0 Enter.

DungeonKey.lock = epok.UInt32(0)
DungeonKey.player = epok.ActorRef()

---@override
function DungeonKey:trigger_event(other, phase)
    if phase == 0 and (not epok.is_valid(self.player) or other == self.player) then
        -- The key is a 3D or a 2D actor; calls need its concrete class.
        local key3d = epok.cast(epok.owner(), "epok::Actor3D")
        local key2d = epok.cast(epok.owner(), "epok::Actor2D")
        if epok.is_valid(key3d) then
            key3d:set_active(false)
        elseif epok.is_valid(key2d) then
            key2d:set_active(false)
        end
    end
end

return DungeonKey
