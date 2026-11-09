---@class PlatformerController : epok.LocalPlayer2DInputComponent
local PlatformerController = epok.LocalPlayer2DInputComponent:extend()

-- Extend calculate_command for project rules. Tune speed on this component,
-- gravity/jumps on the motor, and playback on the sprite driver.

return PlatformerController
