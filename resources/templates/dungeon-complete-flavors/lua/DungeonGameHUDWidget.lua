---@class DungeonGameHUDWidget : DungeonEconomyPresenterBase
local DungeonGameHUDWidget = DungeonEconomyPresenterBase:extend()
---@override
function DungeonGameHUDWidget:take(decision)
    if (decision.notice ~= 0) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, decision.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
    end
    if (decision.notice == 32) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, decision.amount, self.screen.cue, self.screen.visible, self.screen.busy))
    end
    if (decision.heal > 0) then
        self:assign_heal(decision.heal)
    end
    if ((((((((((((((decision.notice == 3) or (decision.notice == 4)) or (decision.notice == 5)) or (decision.notice == 7)) or (decision.notice == 8)) or (decision.notice == 16)) or (decision.notice == 17)) or (decision.notice == 21)) or (decision.notice == 23)) or (decision.notice == 30)) or (decision.notice == 31)) or (decision.notice == 35)) or (decision.notice == 36)) or (decision.notice >= 40)) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 1, self.screen.visible, self.screen.busy))
    end
    if ((decision.notice == 24) or (decision.notice == 33)) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 2, self.screen.visible, self.screen.busy))
    end
    return
end
---@override
function DungeonGameHUDWidget:load()
    self.policy:stage_card_metadata()
    local accepted = epok.memory_card.read_staged_name(0)
    local status = epok.memory_card.snapshot()
    if accepted then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 1, status.request, 20, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
    else
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, (300 + self:card_error(status.last_rejection)), self.screen.progress, 1, self.screen.visible, self.screen.busy))
        self:assign_stats(self:make_stats(self.runtime.transfers, self.runtime.saves, (256 + self:card_error(status.last_rejection))))
    end
    return
end
---@override
function DungeonGameHUDWidget:save()
    if not (self.policy:can_save()) then
        self:take(self.policy:save_started(0, false))
        return
    end
    self.policy:stage_card_metadata()
    self.policy:stage_pending_payload()
    local accepted = epok.memory_card.write_staged_named(92, 0)
    local status = epok.memory_card.snapshot()
    self:take(self.policy:save_started(status.request, accepted))
    if accepted then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 2, status.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
    else
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, (300 + self:card_error(status.last_rejection)), self.screen.progress, 1, self.screen.visible, self.screen.busy))
        self:assign_stats(self:make_stats(self.runtime.transfers, self.runtime.saves, (256 + self:card_error(status.last_rejection))))
    end
    return
end
---@override
function DungeonGameHUDWidget:dispatch(decision)
    self:take(decision)
    if (decision.action == 1) then
        self:load()
        return
    end
    if (((decision.action == 2) or (decision.action == 3)) or (decision.action == 5)) then
        self:take(self.policy:pocket_execute(decision.action, self.screen.port))
        return
    end
    if (decision.action == 4) then
        self:save()
        if ((decision.amount > 0) and (self:read_tx().phase == 2)) then
            self:assign_stats(self:make_stats((self.runtime.transfers + 1), self.runtime.saves, 0))
        end
    end
    return
end
---@override
function DungeonGameHUDWidget:poll_card()
    if (self.screen.card_phase == 0) then
        return
    end
    local status = epok.memory_card.snapshot()
    if (status.completed ~= self.screen.request) then
        return
    end
    if (self.screen.card_phase == 1) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 0, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
        if ((self:card_state(status.state) == 2) and (status.payload_bytes ~= 92)) then
            self:take(self.policy:load_result(self.policy:empty_save(), false, false))
            self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, 52, self.screen.progress, 1, self.screen.visible, self.screen.busy))
            return
        end
        self:take(self.policy:load_result(self.policy:loaded_payload(), (self:card_state(status.state) == 2), (self:card_error(status.error) == 10)))
        if ((self:card_state(status.state) ~= 2) and (self:card_error(status.error) ~= 10)) then
            self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, (300 + self:card_error(status.error)), self.screen.progress, 1, self.screen.visible, self.screen.busy))
            self:assign_stats(self:make_stats(self.runtime.transfers, self.runtime.saves, (256 + self:card_error(status.error))))
        end
        return
    end
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 0, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
    local done = self.policy:save_completed(status.completed, (self:card_state(status.state) == 2))
    self:dispatch(done)
    if done.changed then
        self:assign_stats(self:make_stats(self.runtime.transfers, (self.runtime.saves + 1), self.runtime.error))
    end
    if (self:card_state(status.state) ~= 2) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, (300 + self:card_error(status.error)), self.screen.progress, 1, self.screen.visible, self.screen.busy))
        self:assign_stats(self:make_stats(self.runtime.transfers, self.runtime.saves, (256 + self:card_error(status.error))))
    end
    if ((done.changed and (done.heal == 0)) and (done.action ~= 5)) then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 2, self.screen.visible, self.screen.busy))
    end
    return
end
---@override
function DungeonGameHUDWidget:close_view()
    if epok.is_valid(self.view) then
        self.view:destroy()
    end
    self:assign_view(self:empty_reference())
    epok.gameplay_input.release(self:capture_token())
    self:assign_capture(self:empty_capture())
    self:assign_pending(false)
    self:assign_wait(0)
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, false, self.screen.busy))
    self:render_hud(self:read_hud())
    return
end
---@override
function DungeonGameHUDWidget:widget_ready(candidate)
    if ((not (self.screen.visible) or not (self.runtime.spawn_pending)) or not (epok.gameplay_input.valid(self:capture_token()))) then
        return false
    end
    if (self:parent_of(candidate) ~= self:self_reference()) then
        return false
    end
    local ready = epok.cast(candidate, "DungeonEconomyWidgetBase")
    if not (epok.is_valid(ready)) then
        return false
    end
    self:assign_view(ready)
    self:assign_pending(false)
    self:assign_wait(0)
    return true
end
---@override
function DungeonGameHUDWidget:prepare_view()
    self:hide_hud()
    if (epok.is_valid(self.view) and ((((self.screen.menu == 2) and (self.view:menu_kind() ~= 2)) or ((self.screen.menu == 3) and (self.view:menu_kind() ~= 3))) or (((self.screen.menu == 0) or (self.screen.menu == 1)) and (self.view:menu_kind() ~= 1)))) then
        self:close_view()
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, true, self.screen.busy))
    end
    if not (epok.gameplay_input.valid(self:capture_token())) then
        self:assign_capture(epok.gameplay_input.capture(self:self_reference(), epok.gameplay_input.all_actions(), 100, self.runtime.input_port))
    end
    if not (epok.gameplay_input.valid(self:capture_token())) then
        return
    end
    if not (epok.is_valid(self.view)) then
        if self.runtime.spawn_pending then
            self:assign_wait((self.runtime.spawn_wait + 1))
            if (self.runtime.spawn_wait > 1) then
                self:shutdown()
                self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 1, self.screen.visible, self.screen.busy))
                return
            end
        else
            self:assign_pending(true)
            if (self.screen.menu == 2) then
                epok.spawn("DungeonShopWidget")
            end
            if (self.screen.menu == 3) then
                epok.spawn("DungeonInventoryWidget")
            end
            if ((self.screen.menu ~= 2) and (self.screen.menu ~= 3)) then
                epok.spawn("DungeonTerminalWidget")
            end
        end
    end
    if epok.is_valid(self.view) then
        self.view:assign_policy(self.policy)
        self.view:refresh(self.screen)
    end
    return
end
---@override
function DungeonGameHUDWidget:shutdown()
    self:close_view()
    self:assign_screen(self:make_screen(0, 0, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 0, self.screen.visible, self.screen.busy))
    return
end
---@override
function DungeonGameHUDWidget:frame(context, microseconds)
    if not (self:frame_initialize(context, microseconds)) then
        return self:make_output(((self.screen.menu ~= 0) or self.screen.busy), not (self.screen.visible), self:pending_heal(), self.screen.cue, self.screen.menu, self.screen.selected)
    end
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, ((self.screen.card_phase ~= 0) or (self:read_pocket().operation ~= 0))))
    if not (context.controls) then
        self:assign_screen(self:make_screen(0, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
    end
    if (((self.screen.menu == 0) and not (self.screen.busy)) and context.controls) then
        if epok.gameplay_input.take_frame_pressed(self:input_action(12), self.runtime.input_port) then
            self:assign_screen(self:make_screen(3, 0, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 4, self.screen.visible, self.screen.busy))
        end
    end
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, ((self.screen.menu ~= 0) or (self.screen.card_phase == 1)), self.screen.busy))
    if not (self.screen.visible) then
        self:close_view()
        return self:make_output(((self.screen.menu ~= 0) or self.screen.busy), not (self.screen.visible), self:pending_heal(), self.screen.cue, self.screen.menu, self.screen.selected)
    end
    self:prepare_view()
    if not (epok.gameplay_input.valid(self:capture_token())) then
        self:shutdown()
        return self:make_output(((self.screen.menu ~= 0) or self.screen.busy), not (self.screen.visible), self:pending_heal(), self.screen.cue, self.screen.menu, self.screen.selected)
    end
    return self:frame_controls(context)
end
---@override
function DungeonGameHUDWidget:ratio_double(value, denominator)
    if (value.remainder >= (denominator - value.remainder)) then
        return self:make_ratio((value.remainder - (denominator - value.remainder)), ((value.quotient * 2) + 1))
    end
    return self:make_ratio((value.remainder * 2), (value.quotient * 2))
end
---@override
function DungeonGameHUDWidget:ratio_add(value, numerator, denominator, bit)
    local doubled = self:ratio_double(value, denominator)
    if (bit == 0) then
        return doubled
    end
    if (doubled.remainder >= (denominator - numerator)) then
        return self:make_ratio((doubled.remainder - (denominator - numerator)), (doubled.quotient + 1))
    end
    return self:make_ratio((doubled.remainder + numerator), doubled.quotient)
end
---@override
function DungeonGameHUDWidget:ratio_scale(numerator, denominator, scale)
    if (denominator == 0) then
        return self:make_ratio(0, 0)
    end
    if (numerator >= denominator) then
        return self:make_ratio(0, scale)
    end
    local r0 = self:make_ratio(0, 0)
    local r1 = self:ratio_add(r0, numerator, denominator, ((scale / 32768) % 2))
    local r2 = self:ratio_add(r1, numerator, denominator, ((scale / 16384) % 2))
    local r3 = self:ratio_add(r2, numerator, denominator, ((scale / 8192) % 2))
    local r4 = self:ratio_add(r3, numerator, denominator, ((scale / 4096) % 2))
    local r5 = self:ratio_add(r4, numerator, denominator, ((scale / 2048) % 2))
    local r6 = self:ratio_add(r5, numerator, denominator, ((scale / 1024) % 2))
    local r7 = self:ratio_add(r6, numerator, denominator, ((scale / 512) % 2))
    local r8 = self:ratio_add(r7, numerator, denominator, ((scale / 256) % 2))
    local r9 = self:ratio_add(r8, numerator, denominator, ((scale / 128) % 2))
    local r10 = self:ratio_add(r9, numerator, denominator, ((scale / 64) % 2))
    local r11 = self:ratio_add(r10, numerator, denominator, ((scale / 32) % 2))
    local r12 = self:ratio_add(r11, numerator, denominator, ((scale / 16) % 2))
    local r13 = self:ratio_add(r12, numerator, denominator, ((scale / 8) % 2))
    local r14 = self:ratio_add(r13, numerator, denominator, ((scale / 4) % 2))
    local r15 = self:ratio_add(r14, numerator, denominator, ((scale / 2) % 2))
    local r16 = self:ratio_add(r15, numerator, denominator, ((scale / 1) % 2))
    return r16
end
---@override
function DungeonGameHUDWidget:hide_hud()
    self:assign_modal(true)
    self:hud_flags(self.energy_label, false, false, false)
    self:hud_flags(self.energy_bar, false, false, false)
    self:hud_flags(self.energy_panel, false, false, false)
    self:hud_flags(self.boss_bar, false, false, false)
    self:hud_flags(self.reticle, false, false, false)
    self:hud_flags(self.message_panel, false, false, false)
    self:hud_flags(self.objective, false, false, false)
    self:hud_flags(self.key_status, false, false, false)
    self:hud_flags(self.game_message, false, false, false)
    self:hud_flags(self.boss_label, false, false, false)
    self:hud_flags(self.controls_hint, false, false, false)
    self:hud_border(false)
    return
end
---@override
function DungeonGameHUDWidget:damage_hud(value)
    self:assign_damage(value)
    if self.screen.visible then
        self:hud_border(false)
        return
    end
    self:hud_damage_copy(value)
    return
end
---@override
function DungeonGameHUDWidget:render_hud(state)
    if self:hud_same(state, self.screen.visible) then
        return
    end
    self:assign_hud(state, self.screen.visible)
    if self.screen.visible then
        self:hide_hud()
        return
    end
    local boss_show = ((state.state == 1) or ((state.state == 0) and state.has_key))
    local ended = ((state.state == 2) or (state.state == 3))
    self:hud_flags(self.energy_label, true, false, false)
    self:hud_flags(self.energy_panel, false, true, false)
    self:hud_flags(self.energy_bar, false, (state.health > 0), false)
    self:hud_flags(self.boss_bar, false, false, boss_show)
    self:hud_flags(self.boss_label, boss_show, false, false)
    self:hud_flags(self.reticle, (state.aiming and state.alive), false, false)
    self:hud_flags(self.message_panel, false, ended, false)
    self:hud_flags(self.game_message, ended, false, false)
    self:hud_flags(self.objective, true, false, false)
    self:hud_flags(self.key_status, true, false, false)
    self:hud_flags(self.controls_hint, true, false, false)
    self:hud_number(self.energy_label, 400, state.health)
    if state.has_key then
        self:hud_key(self.key_status, 401)
    else
        self:hud_key(self.key_status, 402)
    end
    if (state.state == 2) then
        self:hud_key(self.objective, 403)
        self:hud_key(self.game_message, 408)
    else
        if (state.state == 3) then
            self:hud_key(self.objective, 404)
            self:hud_key(self.game_message, 409)
        else
            if boss_show then
                self:hud_key(self.objective, 405)
            else
                if state.has_key then
                    self:hud_key(self.objective, 406)
                else
                    self:hud_key(self.objective, 407)
                end
            end
        end
    end
    if (self.runtime.station == 1) then
        self:hud_key(self.controls_hint, 410)
    else
        if (self.runtime.station == 2) then
            self:hud_key(self.controls_hint, 411)
        else
            self:hud_key(self.controls_hint, 412)
        end
    end
    local energy = self:ratio_scale(state.health, state.maximum, 4096)
    local source = self:ratio_scale(state.health, state.maximum, self:hud_source_width(self.energy_bar))
    self:hud_crop(self.energy_bar, (self:hud_authored_width(self.energy_bar) * self:hud_fixed(energy.quotient)), source.quotient)
    if (source.remainder > 0) then
        self:hud_crop(self.energy_bar, (self:hud_authored_width(self.energy_bar) * self:hud_fixed(energy.quotient)), (source.quotient + 1))
    end
    local percentage = self:ratio_scale(state.health, state.maximum, 100)
    if ((percentage.quotient < 25) or ((percentage.quotient == 25) and (percentage.remainder == 0))) then
        self:hud_tint(self.energy_bar, self.style.critical_red, self.style.critical_green, self.style.critical_blue)
    else
        if ((percentage.quotient < 60) or ((percentage.quotient == 60) and (percentage.remainder == 0))) then
            self:hud_tint(self.energy_bar, self.style.warning_red, self.style.warning_green, self.style.warning_blue)
        else
            self:hud_tint(self.energy_bar, self.style.normal_red, self.style.normal_green, self.style.normal_blue)
        end
    end
    local boss_ratio = self:ratio_scale(state.boss_health, state.boss_maximum, 4096)
    self:hud_progress(self.boss_bar, self:hud_fixed(boss_ratio.quotient))
    self:damage_hud(self:read_damage())
    return
end
---@override
function DungeonGameHUDWidget:station_available(port)
    return (((self.runtime.controls and (port == self.runtime.input_port)) and (self.screen.menu == 0)) and not (self.screen.busy))
end
---@override
function DungeonGameHUDWidget:open_station(menu, port)
    if not (self:station_available(port)) then
        return false
    end
    if ((menu ~= 1) and (menu ~= 2)) then
        return false
    end
    self:assign_screen(self:make_screen(menu, 0, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 4, true, self.screen.busy))
    self:assign_open_cue(true)
    self:prepare_view()
    return true
end
---@override
function DungeonGameHUDWidget:station_focus(menu, focused)
    if focused then
        self:assign_station(menu)
    else
        if (self.runtime.station == menu) then
            self:assign_station(0)
        end
    end
    self:render_hud(self:read_hud())
    return
end
---@override
function DungeonGameHUDWidget:frame_initialize(context, microseconds)
    self:assign_controls(context.controls)
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 0, self.screen.visible, self.screen.busy))
    if self.runtime.open_cue then
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 4, self.screen.visible, self.screen.busy))
        self:assign_open_cue(false)
    end
    self:assign_heal(0)
    if not (self:same_policy()) then
        self:shutdown()
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 0, 0, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, false))
        self:assign_started(false)
        self:assign_bound(self.policy)
    end
    if not (self:policy_live()) then
        self:shutdown()
        self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 0, 0, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, false))
        self:assign_started(false)
        return false
    end
    if not (self.runtime.started) then
        self:assign_started(true)
        if (self:read_tx().phase == 2) then
            self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, 2, self:read_tx().request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
        else
            if not (self:read_loaded()) then
                self:load()
            end
        end
    end
    self:poll_card()
    if (self:read_pocket().operation ~= 0) then
        self:dispatch(self.policy:pocket_update(context.dt, microseconds))
        self:assign_stats(self:make_stats(self.runtime.transfers, self.runtime.saves, self:read_pocket().error))
    end
    return true
end
---@override
function DungeonGameHUDWidget:frame_controls(context)
    if (epok.is_valid(self.view) and not (self.screen.busy)) then
        local vertical = epok.gameplay_input.context_sample(self:capture_token(), self:input_action(1), true)
        local horizontal = epok.gameplay_input.context_sample(self:capture_token(), self:input_action(0), true)
        local cancel_key = epok.gameplay_input.context_take_pressed(self:capture_token(), self:input_action(7), true)
        local inventory_key = epok.gameplay_input.context_take_pressed(self:capture_token(), self:input_action(12), true)
        local confirm_key = epok.gameplay_input.context_take_pressed(self:capture_token(), self:input_action(6), true)
        local buttons = self:make_buttons((vertical.pressed and (vertical.value > 0.0)), (vertical.pressed and (vertical.value < 0.0)), (horizontal.pressed and (horizontal.value <= 0.0)), (horizontal.pressed and (horizontal.value >= 0.0)), confirm_key, (cancel_key or inventory_key))
        local intent = self.view:input(buttons)
        if ((intent.selected ~= self.screen.selected) or (intent.port ~= self.screen.port)) then
            self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 3, self.screen.visible, self.screen.busy))
        end
        self:assign_screen(self:make_screen(self.screen.menu, intent.selected, intent.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, self.screen.busy))
        if (intent.command == 2) then
            self.policy:leave()
            self:assign_screen(self:make_screen(0, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 5, self.screen.visible, self.screen.busy))
            self:close_view()
            return self:make_output(((self.screen.menu ~= 0) or self.screen.busy), not (self.screen.visible), self:pending_heal(), self.screen.cue, self.screen.menu, self.screen.selected)
        end
        if (intent.command == 1) then
            self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, 6, self.screen.visible, self.screen.busy))
            self:dispatch(self.policy:activate(self.screen.menu, self.screen.selected, context.can_heal))
        end
    end
    self:assign_screen(self:make_screen(self.screen.menu, self.screen.selected, self.screen.port, self.screen.card_phase, self.screen.request, self.screen.notice, self.screen.progress, self.screen.cue, self.screen.visible, ((self.screen.card_phase ~= 0) or (self:read_pocket().operation ~= 0))))
    if epok.is_valid(self.view) then
        self.view:refresh(self.screen)
    end
    return self:make_output(((self.screen.menu ~= 0) or self.screen.busy), not (self.screen.visible), self:pending_heal(), self.screen.cue, self.screen.menu, self.screen.selected)
end
---@override
function DungeonGameHUDWidget:on_disable() self:shutdown() end
---@override
function DungeonGameHUDWidget:end_play(end_play_reason) self:shutdown() end
return DungeonGameHUDWidget
