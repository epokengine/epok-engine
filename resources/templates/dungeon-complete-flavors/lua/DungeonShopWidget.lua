---@class DungeonShopWidget : DungeonEconomyWidgetBase
local DungeonShopWidget = DungeonEconomyWidgetBase:extend()
---@override
function DungeonShopWidget:menu_kind()
    return 2
end
---@override
function DungeonShopWidget:input(buttons)
    if self.screen.busy then
        return self:make_intent(0, self.screen.selected, self.screen.port)
    end
    if buttons.cancel then
        return self:make_intent(2, self.screen.selected, self.screen.port)
    end
    if buttons.up then
        self:assign_selected((((self.screen.selected + 5) - 1) % 5))
    end
    if buttons.down then
        self:assign_selected(((self.screen.selected + 1) % 5))
    end
    if buttons.confirm then
        return self:make_intent(1, self.screen.selected, self.screen.port)
    end
    return self:make_intent(0, self.screen.selected, self.screen.port)
end
---@override
function DungeonShopWidget:refresh(state)
    self:assign_screen(state)
    if not (self:policy_live()) then
        return
    end
    self:clear_buffer()
    if ((self.screen.menu == 0) and (self.screen.card_phase == 1)) then
        self:append_text(4)
    else
        self:append_text(2)
    end
    self:copy_text(self.title)
    self:clear_buffer()
    self:append_text(5)
    self:append_unsigned(self:read_saved().ore)
    self:append_text(6)
    self:append_unsigned(self:read_saved().credits)
    self:copy_text(self.summary)
    self:clear_buffer()
    if ((self.screen.menu == 0) and (self.screen.card_phase == 1)) then
        self:append_text(14)
    else
        self:append_text(16)
        self:append_unsigned(self:read_tuning().sale_rate)
        self:append_text(17)
        if (self.screen.selected == 0) then
            self:append_text(8)
        else
            self:append_text(9)
        end
        self:row(0)
        self:append_text(10)
        if (self.screen.selected == 1) then
            self:append_text(8)
        else
            self:append_text(9)
        end
        self:row(1)
        self:append_text(10)
        if (self.screen.selected == 2) then
            self:append_text(8)
        else
            self:append_text(9)
        end
        self:row(2)
        self:append_text(10)
        if (self.screen.selected == 3) then
            self:append_text(8)
        else
            self:append_text(9)
        end
        self:row(3)
        self:append_text(10)
        if (self.screen.selected == 4) then
            self:append_text(8)
        else
            self:append_text(9)
        end
        self:row(4)
        self:append_text(10)
    end
    self:copy_text(self.body)
    self:clear_buffer()
    if (self.screen.notice == 32) then
        self:append_text(132)
        self:append_unsigned(self.screen.progress)
    else
        if (self.screen.notice == 36) then
            self:append_text((200 + self:read_pocket().error))
        else
            if (self.screen.notice >= 300) then
                self:append_text(self.screen.notice)
            else
                self:append_text((self.screen.notice + 100))
            end
        end
    end
    self:copy_text(self.status)
    self:clear_buffer()
    if self.screen.busy then
        self:append_text(13)
    else
        self:append_text(11)
    end
    self:copy_text(self.footer)
    return
end
---@param index UInt32
function DungeonShopWidget:row(index)
    if (index == 0) then
        self:append_text(23)
        return
    end
    if (index == 1) then
        self:append_text(24)
        self:append_unsigned(self:read_tuning().medkit_heal)
        self:append_text(25)
        self:append_unsigned(self.policy:price(self:read_saved(), 0))
        self:append_text(26)
        return
    end
    if (index == 2) then
        self:append_text(27)
        self:append_unsigned(self.policy:price(self:read_saved(), 1))
        self:append_text(26)
        return
    end
    if (index == 3) then
        self:append_text(28)
        self:append_unsigned(self:read_tuning().buster_damage)
        self:append_text(29)
        self:append_unsigned(self.policy:price(self:read_saved(), 2))
        self:append_text(26)
        return
    end
    self:append_text(22)
    return
end
---@override
function DungeonShopWidget:begin_play()
    if not self:has_parent() then return end
    local parent = self:parent_reference()
    if not epok.is_valid(parent) then self:destroy() return end
    local owner = epok.cast(parent, "DungeonEconomyPresenterEndpoint")
    if epok.is_valid(owner) and not owner:widget_ready(self:self_reference()) then self:destroy() end
end
return DungeonShopWidget
