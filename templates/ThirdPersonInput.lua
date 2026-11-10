---@class ThirdPersonController : epok.LocalPlayer3DInputComponent
local ThirdPersonController = epok.LocalPlayer3DInputComponent:extend()

-- Extend calculate_command to add project movement rules. The shared motor,
-- animation driver and camera components keep their normal automatic ticks.
---@override
function ThirdPersonController:begin_play()
    ThirdPersonController.super.begin_play(self)
    if epok.is_valid(self.camera) then
        self.camera:make_active()
    end
end

return ThirdPersonController
