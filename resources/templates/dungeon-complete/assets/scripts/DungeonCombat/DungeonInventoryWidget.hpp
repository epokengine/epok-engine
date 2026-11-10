#pragma once
#include "DungeonEconomyWidgetBase.hpp"
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=UI,Id="f0474611-dd51-5cea-9702-500ba3907dfd") DungeonInventoryWidget:public DungeonEconomyWidgetBase {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("f0474611-dd51-5cea-9702-500ba3907dfd");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    uint32_t menu_kind() const override {
        return 3u;
    }
    DungeonEconomyIntent input(DungeonEconomyButtons buttons) override {
        if (screen.busy) {
            return make_intent(0u, screen.selected, screen.port);
        }
        if (buttons.cancel) {
            return make_intent(2u, screen.selected, screen.port);
        }
        if (buttons.up) {
            assign_selected((((screen.selected + 4u) - 1u) % 4u));
        }
        if (buttons.down) {
            assign_selected(((screen.selected + 1u) % 4u));
        }
        if (buttons.confirm) {
            return make_intent(1u, screen.selected, screen.port);
        }
        return make_intent(0u, screen.selected, screen.port);
    }
    void refresh(DungeonEconomyScreen state) override {
        assign_screen(state);
        if (!(policy_live())) {
            return;
        }
        clear_buffer();
        if (((screen.menu == 0u) && (screen.card_phase == 1u))) {
            append_text(4u);
        } else {
            append_text(3u);
        }
        copy_text(title);
        clear_buffer();
        append_text(5u);
        append_unsigned(read_saved().ore);
        append_text(6u);
        append_unsigned(read_saved().credits);
        copy_text(summary);
        clear_buffer();
        if (((screen.menu == 0u) && (screen.card_phase == 1u))) {
            append_text(14u);
        } else {
            append_text(18u);
            if ((screen.selected == 0u)) {
                append_text(8u);
            } else {
                append_text(9u);
            }
            row(0u);
            append_text(10u);
            if ((screen.selected == 1u)) {
                append_text(8u);
            } else {
                append_text(9u);
            }
            row(1u);
            append_text(10u);
            if ((screen.selected == 2u)) {
                append_text(8u);
            } else {
                append_text(9u);
            }
            row(2u);
            append_text(10u);
            if ((screen.selected == 3u)) {
                append_text(8u);
            } else {
                append_text(9u);
            }
            row(3u);
            append_text(10u);
            append_text(37u);
        }
        copy_text(body);
        clear_buffer();
        if ((screen.notice == 32u)) {
            append_text(132u);
            append_unsigned(screen.progress);
        } else {
            if ((screen.notice == 36u)) {
                append_text((200u + read_pocket().error));
            } else {
                if ((screen.notice >= 300u)) {
                    append_text(screen.notice);
                } else {
                    append_text((screen.notice + 100u));
                }
            }
        }
        copy_text(status);
        clear_buffer();
        if (screen.busy) {
            append_text(13u);
        } else {
            append_text(11u);
        }
        copy_text(footer);
        return;
    }
    void row(uint32_t index) {
        if ((index == 0u)) {
            append_text(30u);
            append_unsigned(read_saved().medkits);
            append_text(31u);
            append_unsigned(read_tuning().medkit_heal);
            append_text(32u);
            return;
        }
        if ((index == 1u)) {
            append_text(33u);
            append_unsigned(read_saved().tanks);
            append_text(34u);
            return;
        }
        if ((index == 2u)) {
            append_text(35u);
            append_unsigned(read_saved().buster);
            append_text(31u);
            append_unsigned(([&](){auto* target=policy_value();return target?target->damage_bonus():0u;}()));
            append_text(36u);
            return;
        }
        append_text(22u);
        return;
    }
    void begin_play()override{if(!has_parent())return;const auto parent=parent_reference();if(!parent.get()){epok::bp::api::destroy(id());return;}auto* owner=epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyPresenterEndpoint>(parent):nullptr;if(owner&&!owner->widget_ready(id()))epok::bp::api::destroy(id());}
};
