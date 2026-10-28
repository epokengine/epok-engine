---@class RumbleDemo : epok.Actor3D
local RumbleDemo = epok.Actor3D:extend()

RumbleDemo.next_scene = epok.UInt32(1)
RumbleDemo.port_label = epok.ComponentRef(epok.TextComponent)
RumbleDemo.state_label = epok.ComponentRef(epok.TextComponent)
RumbleDemo.result_label = epok.ComponentRef(epok.TextComponent)
RumbleDemo.frame_label = epok.ComponentRef(epok.TextComponent)
RumbleDemo.frames = epok.Hidden(epok.UInt32(0))
RumbleDemo.selected_port = epok.Hidden(epok.UInt32(0))
RumbleDemo.last_result = epok.Hidden(epok.UInt32(0))
RumbleDemo.state_code = epok.Hidden(epok.UInt32(0))

---@override
function RumbleDemo:on_frame(frame_microseconds)
    self.frames = self.frames + 1
    self.frame_label:set_unsigned(self.frames)
    -- Pad 1 supplies the controls. SELECT changes the output slot independently.
    -- These are PSX button indices, shared by physical and mapped desktop input.
    if epok.input.frame_pressed(0, 0) then
        self.selected_port = (self.selected_port + 1) % 4
        self.last_result = 0
    end
    if epok.input.frame_pressed(3, 0) then
        epok.time.set_paused(not epok.time.paused())
    end
    if epok.input.frame_pressed(10, 0) then
        epok.input.stop_rumble(self.selected_port)
    end
    if epok.input.frame_pressed(11, 0) then
        local state = epok.input.rumble_snapshot(self.selected_port)
        epok.input.set_rumble_enabled(not state.enabled, self.selected_port)
    end
    if not epok.time.paused() then
        local requested = false
        local accepted = false
        if epok.input.frame_pressed(14, 0) then
            requested = true
            accepted = epok.input.rumble(192, false, 250, self.selected_port)
        elseif epok.input.frame_pressed(13, 0) then
            requested = true
            accepted = epok.input.rumble(0, true, 250, self.selected_port)
        elseif epok.input.frame_pressed(15, 0) then
            requested = true
            accepted = epok.input.rumble(192, true, 150, self.selected_port)
        elseif epok.input.frame_pressed(12, 0) then
            requested = true
            accepted = epok.input.rumble(192, true, 2000, self.selected_port)
        end
        if requested then
            if accepted then
                self.last_result = 1
            else
                self.last_result = 2
            end
        end
        if epok.input.frame_pressed(9, 0) then
            epok.scene.request(self.next_scene)
        end
    end

    local state = epok.input.rumble_snapshot(self.selected_port)
    self.state_code = 0
    if epok.time.paused() then
        self.state_code = 6
    elseif not state.connected then
        self.state_code = 0
    elseif not state.enabled then
        self.state_code = 3
    elseif state.probing then
        self.state_code = 2
    elseif not state.supported then
        self.state_code = 1
    elseif state.active then
        self.state_code = 5
    else
        self.state_code = 4
    end
    self.port_label:set_unsigned(self.selected_port + 1)
    self.state_label:set_unsigned(self.state_code)
    self.result_label:set_unsigned(self.last_result)
end

return RumbleDemo
