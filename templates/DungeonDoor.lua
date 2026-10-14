---@class DungeonDoor : epok.ActorComponent
local DungeonDoor = epok.ActorComponent:extend()

-- A locked dungeon door. Its collider blocks the doorway until its key has been
-- picked up (the key actor is no longer active); then the door's actor deactivates,
-- which hides it and removes the collider. The Dungeon Generator sets `lock` and
-- `key`. The C++ and Blueprint flavors behave the same way.

DungeonDoor.lock = epok.UInt32(0)
DungeonDoor.key = epok.ActorRef()

---@override
function DungeonDoor:tick(delta_seconds)
    -- Keys and doors are 3D or 2D actors; calls need their concrete class.
    local key3d = epok.cast(self.key, "epok::Actor3D")
    local key2d = epok.cast(self.key, "epok::Actor2D")
    local taken = false
    if epok.is_valid(key3d) then
        taken = not key3d:active()
    elseif epok.is_valid(key2d) then
        taken = not key2d:active()
    end
    if taken then
        local door3d = epok.cast(epok.owner(), "epok::Actor3D")
        local door2d = epok.cast(epok.owner(), "epok::Actor2D")
        if epok.is_valid(door3d) then
            door3d:set_active(false)
        elseif epok.is_valid(door2d) then
            door2d:set_active(false)
        end
    end
end

return DungeonDoor
