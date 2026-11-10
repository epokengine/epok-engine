#pragma once
#include "DungeonEconomyPresenterBase.hpp"
#include "DungeonTerminalWidget.hpp"
#include "DungeonShopWidget.hpp"
#include "DungeonInventoryWidget.hpp"
class EPOK_CLASS(Blueprintable,Placeable,PlacedOnly,Domain=UI,Id="b220c109-6a29-53a3-bbbc-dcf880abe73d") DungeonGameHUDWidget:public DungeonEconomyPresenterBase {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("b220c109-6a29-53a3-bbbc-dcf880abe73d");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    void take(DungeonSupplyDecision decision) {
        if ((decision.notice != 0u)) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, decision.notice, screen.progress, screen.cue, screen.visible, screen.busy));
        }
        if ((decision.notice == 32u)) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, decision.amount, screen.cue, screen.visible, screen.busy));
        }
        if ((decision.heal > 0u)) {
            assign_heal(decision.heal);
        }
        if (((((((((((((((decision.notice == 3u) || (decision.notice == 4u)) || (decision.notice == 5u)) || (decision.notice == 7u)) || (decision.notice == 8u)) || (decision.notice == 16u)) || (decision.notice == 17u)) || (decision.notice == 21u)) || (decision.notice == 23u)) || (decision.notice == 30u)) || (decision.notice == 31u)) || (decision.notice == 35u)) || (decision.notice == 36u)) || (decision.notice >= 40u))) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 1u, screen.visible, screen.busy));
        }
        if (((decision.notice == 24u) || (decision.notice == 33u))) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 2u, screen.visible, screen.busy));
        }
        return;
    }
    void load() {
        ([&]() {auto* target=policy_value();if(target)target->stage_card_metadata();}());
        const auto accepted = epok::MemoryCardLibrary::read_staged_name(0u);
        const auto status = epok::MemoryCardLibrary::snapshot();
        if (accepted) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, 1u, status.request, 20u, screen.progress, screen.cue, screen.visible, screen.busy));
        } else {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, (300u + card_error(status.last_rejection)), screen.progress, 1u, screen.visible, screen.busy));
            assign_stats(make_stats(runtime.transfers, runtime.saves, (256u + card_error(status.last_rejection))));
        }
        return;
    }
    void save() {
        if (!(([&]() -> bool {auto* target=policy_value();return target?target->can_save():bool{};}()))) {
            take(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->save_started(0u, false):DungeonSupplyDecision{};}()));
            return;
        }
        ([&]() {auto* target=policy_value();if(target)target->stage_card_metadata();}());
        ([&]() {auto* target=policy_value();if(target)target->stage_pending_payload();}());
        const auto accepted = epok::MemoryCardLibrary::write_staged_named(92u, 0u);
        const auto status = epok::MemoryCardLibrary::snapshot();
        take(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->save_started(status.request, accepted):DungeonSupplyDecision{};}()));
        if (accepted) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, 2u, status.request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
        } else {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, (300u + card_error(status.last_rejection)), screen.progress, 1u, screen.visible, screen.busy));
            assign_stats(make_stats(runtime.transfers, runtime.saves, (256u + card_error(status.last_rejection))));
        }
        return;
    }
    void dispatch(DungeonSupplyDecision decision) {
        take(decision);
        if ((decision.action == 1u)) {
            load();
            return;
        }
        if ((((decision.action == 2u) || (decision.action == 3u)) || (decision.action == 5u))) {
            take(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->pocket_execute(decision.action, screen.port):DungeonSupplyDecision{};}()));
            return;
        }
        if ((decision.action == 4u)) {
            save();
            if (((decision.amount > 0u) && (read_tx().phase == 2u))) {
                assign_stats(make_stats((runtime.transfers + 1u), runtime.saves, 0u));
            }
        }
        return;
    }
    void poll_card() {
        if ((screen.card_phase == 0u)) {
            return;
        }
        const auto status = epok::MemoryCardLibrary::snapshot();
        if ((status.completed != screen.request)) {
            return;
        }
        if ((screen.card_phase == 1u)) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, 0u, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
            if (((card_state(status.state) == 2u) && (status.payload_bytes != 92u))) {
                take(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->load_result(([&]() -> DungeonEconomySave {auto* target=policy_value();return target?target->empty_save():DungeonEconomySave{};}()), false, false):DungeonSupplyDecision{};}()));
                assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, 52u, screen.progress, 1u, screen.visible, screen.busy));
                return;
            }
            take(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->load_result(([&]() -> DungeonEconomySave {auto* target=policy_value();return target?target->loaded_payload():DungeonEconomySave{};}()), (card_state(status.state) == 2u), (card_error(status.error) == 10u)):DungeonSupplyDecision{};}()));
            if (((card_state(status.state) != 2u) && (card_error(status.error) != 10u))) {
                assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, (300u + card_error(status.error)), screen.progress, 1u, screen.visible, screen.busy));
                assign_stats(make_stats(runtime.transfers, runtime.saves, (256u + card_error(status.error))));
            }
            return;
        }
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, 0u, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
        const auto done = ([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->save_completed(status.completed, (card_state(status.state) == 2u)):DungeonSupplyDecision{};}());
        dispatch(done);
        if (done.changed) {
            assign_stats(make_stats(runtime.transfers, (runtime.saves + 1u), runtime.error));
        }
        if ((card_state(status.state) != 2u)) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, (300u + card_error(status.error)), screen.progress, 1u, screen.visible, screen.busy));
            assign_stats(make_stats(runtime.transfers, runtime.saves, (256u + card_error(status.error))));
        }
        if (((done.changed && (done.heal == 0u)) && (done.action != 5u))) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 2u, screen.visible, screen.busy));
        }
        return;
    }
    void close_view() {
        if (epok::bp::api::valid(view)) {
            epok::bp::api::destroy(view);
        }
        assign_view(empty_reference());
        epok::GameplayInputLibrary::release(capture_token());
        assign_capture(empty_capture());
        assign_pending(false);
        assign_wait(0u);
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, false, screen.busy));
        render_hud(read_hud());
        return;
    }
    bool widget_ready(epok::ObjectId candidate) override {
        if (((!(screen.visible) || !(runtime.spawn_pending)) || !(epok::GameplayInputLibrary::valid(capture_token())))) {
            return false;
        }
        if ((parent_of(candidate) != self_reference())) {
            return false;
        }
        const auto ready = epok::bp::api::cast(epok::detail::compact_class_id("cd4bf230-fb5d-5200-b004-1fa3c3132bfb"),candidate);
        if (!(epok::bp::api::valid(ready))) {
            return false;
        }
        assign_view(ready);
        assign_pending(false);
        assign_wait(0u);
        return true;
    }
    void prepare_view() {
        hide_hud();
        if ((epok::bp::api::valid(view) && ((((screen.menu == 2u) && (([&]() -> uint32_t {auto* target=view_value();return target?target->menu_kind():uint32_t{};}()) != 2u)) || ((screen.menu == 3u) && (([&]() -> uint32_t {auto* target=view_value();return target?target->menu_kind():uint32_t{};}()) != 3u))) || (((screen.menu == 0u) || (screen.menu == 1u)) && (([&]() -> uint32_t {auto* target=view_value();return target?target->menu_kind():uint32_t{};}()) != 1u))))) {
            close_view();
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, true, screen.busy));
        }
        if (!(epok::GameplayInputLibrary::valid(capture_token()))) {
            assign_capture(epok::GameplayInputLibrary::capture(self_reference(), epok::GameplayInputLibrary::all_actions(), 100u, runtime.input_port));
        }
        if (!(epok::GameplayInputLibrary::valid(capture_token()))) {
            return;
        }
        if (!(epok::bp::api::valid(view))) {
            if (runtime.spawn_pending) {
                assign_wait((runtime.spawn_wait + 1u));
                if ((runtime.spawn_wait > 1u)) {
                    shutdown();
                    assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 1u, screen.visible, screen.busy));
                    return;
                }
            } else {
                assign_pending(true);
                if ((screen.menu == 2u)) {
                    epok::bp::api::spawn(epok::detail::compact_class_id("733195d2-87b9-5164-b675-53372b0a569c"),id());
                }
                if ((screen.menu == 3u)) {
                    epok::bp::api::spawn(epok::detail::compact_class_id("f0474611-dd51-5cea-9702-500ba3907dfd"),id());
                }
                if (((screen.menu != 2u) && (screen.menu != 3u))) {
                    epok::bp::api::spawn(epok::detail::compact_class_id("f5ddf390-875f-56c6-8efd-9265383c29ff"),id());
                }
            }
        }
        if (epok::bp::api::valid(view)) {
            ([&]() {auto* target=view_value();if(target)target->assign_policy(policy);}());
            ([&]() {auto* target=view_value();if(target)target->refresh(screen);}());
        }
        return;
    }
    void shutdown() override {
        close_view();
        assign_screen(make_screen(0u, 0u, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 0u, screen.visible, screen.busy));
        return;
    }
    DungeonEconomyUIOutput frame(DungeonEconomyUIContext context,uint32_t microseconds) override {
        if (!(frame_initialize(context, microseconds))) {
            return make_output(((screen.menu != 0u) || screen.busy), !(screen.visible), pending_heal(), screen.cue, screen.menu, screen.selected);
        }
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, ((screen.card_phase != 0u) || (read_pocket().operation != 0u))));
        if (!(context.controls)) {
            assign_screen(make_screen(0u, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
        }
        if ((((screen.menu == 0u) && !(screen.busy)) && context.controls)) {
            if (epok::GameplayInputLibrary::take_frame_pressed(input_action(12u), runtime.input_port)) {
                assign_screen(make_screen(3u, 0u, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 4u, screen.visible, screen.busy));
            }
        }
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, ((screen.menu != 0u) || (screen.card_phase == 1u)), screen.busy));
        if (!(screen.visible)) {
            close_view();
            return make_output(((screen.menu != 0u) || screen.busy), !(screen.visible), pending_heal(), screen.cue, screen.menu, screen.selected);
        }
        prepare_view();
        if (!(epok::GameplayInputLibrary::valid(capture_token()))) {
            shutdown();
            return make_output(((screen.menu != 0u) || screen.busy), !(screen.visible), pending_heal(), screen.cue, screen.menu, screen.selected);
        }
        return frame_controls(context);
    }
    DungeonHUDRatio ratio_double(DungeonHUDRatio value,uint32_t denominator) {
        if ((value.remainder >= (denominator - value.remainder))) {
            return make_ratio((value.remainder - (denominator - value.remainder)), ((value.quotient * 2u) + 1u));
        }
        return make_ratio((value.remainder * 2u), (value.quotient * 2u));
    }
    DungeonHUDRatio ratio_add(DungeonHUDRatio value,uint32_t numerator,uint32_t denominator,uint32_t bit) {
        const auto doubled = ratio_double(value, denominator);
        if ((bit == 0u)) {
            return doubled;
        }
        if ((doubled.remainder >= (denominator - numerator))) {
            return make_ratio((doubled.remainder - (denominator - numerator)), (doubled.quotient + 1u));
        }
        return make_ratio((doubled.remainder + numerator), doubled.quotient);
    }
    DungeonHUDRatio ratio_scale(uint32_t numerator,uint32_t denominator,uint32_t scale) {
        if ((denominator == 0u)) {
            return make_ratio(0u, 0u);
        }
        if ((numerator >= denominator)) {
            return make_ratio(0u, scale);
        }
        const auto r0 = make_ratio(0u, 0u);
        const auto r1 = ratio_add(r0, numerator, denominator, ((scale / 32768u) % 2u));
        const auto r2 = ratio_add(r1, numerator, denominator, ((scale / 16384u) % 2u));
        const auto r3 = ratio_add(r2, numerator, denominator, ((scale / 8192u) % 2u));
        const auto r4 = ratio_add(r3, numerator, denominator, ((scale / 4096u) % 2u));
        const auto r5 = ratio_add(r4, numerator, denominator, ((scale / 2048u) % 2u));
        const auto r6 = ratio_add(r5, numerator, denominator, ((scale / 1024u) % 2u));
        const auto r7 = ratio_add(r6, numerator, denominator, ((scale / 512u) % 2u));
        const auto r8 = ratio_add(r7, numerator, denominator, ((scale / 256u) % 2u));
        const auto r9 = ratio_add(r8, numerator, denominator, ((scale / 128u) % 2u));
        const auto r10 = ratio_add(r9, numerator, denominator, ((scale / 64u) % 2u));
        const auto r11 = ratio_add(r10, numerator, denominator, ((scale / 32u) % 2u));
        const auto r12 = ratio_add(r11, numerator, denominator, ((scale / 16u) % 2u));
        const auto r13 = ratio_add(r12, numerator, denominator, ((scale / 8u) % 2u));
        const auto r14 = ratio_add(r13, numerator, denominator, ((scale / 4u) % 2u));
        const auto r15 = ratio_add(r14, numerator, denominator, ((scale / 2u) % 2u));
        const auto r16 = ratio_add(r15, numerator, denominator, ((scale / 1u) % 2u));
        return r16;
    }
    void hide_hud() {
        assign_modal(true);
        hud_flags(energy_label, false, false, false);
        hud_flags(energy_bar, false, false, false);
        hud_flags(energy_panel, false, false, false);
        hud_flags(boss_bar, false, false, false);
        hud_flags(reticle, false, false, false);
        hud_flags(message_panel, false, false, false);
        hud_flags(objective, false, false, false);
        hud_flags(key_status, false, false, false);
        hud_flags(game_message, false, false, false);
        hud_flags(boss_label, false, false, false);
        hud_flags(controls_hint, false, false, false);
        hud_border(false);
        return;
    }
    void damage_hud(DungeonHUDDamage value) {
        assign_damage(value);
        if (screen.visible) {
            hud_border(false);
            return;
        }
        hud_damage_copy(value);
        return;
    }
    void render_hud(DungeonHUDSnapshot state) {
        if (hud_same(state, screen.visible)) {
            return;
        }
        assign_hud(state, screen.visible);
        if (screen.visible) {
            hide_hud();
            return;
        }
        const auto boss_show = ((state.state == 1u) || ((state.state == 0u) && state.has_key));
        const auto ended = ((state.state == 2u) || (state.state == 3u));
        hud_flags(energy_label, true, false, false);
        hud_flags(energy_panel, false, true, false);
        hud_flags(energy_bar, false, (state.health > 0u), false);
        hud_flags(boss_bar, false, false, boss_show);
        hud_flags(boss_label, boss_show, false, false);
        hud_flags(reticle, (state.aiming && state.alive), false, false);
        hud_flags(message_panel, false, ended, false);
        hud_flags(game_message, ended, false, false);
        hud_flags(objective, true, false, false);
        hud_flags(key_status, true, false, false);
        hud_flags(controls_hint, true, false, false);
        hud_number(energy_label, 400u, state.health);
        if (state.has_key) {
            hud_key(key_status, 401u);
        } else {
            hud_key(key_status, 402u);
        }
        if ((state.state == 2u)) {
            hud_key(objective, 403u);
            hud_key(game_message, 408u);
        } else {
            if ((state.state == 3u)) {
                hud_key(objective, 404u);
                hud_key(game_message, 409u);
            } else {
                if (boss_show) {
                    hud_key(objective, 405u);
                } else {
                    if (state.has_key) {
                        hud_key(objective, 406u);
                    } else {
                        hud_key(objective, 407u);
                    }
                }
            }
        }
        if ((runtime.station == 1u)) {
            hud_key(controls_hint, 410u);
        } else {
            if ((runtime.station == 2u)) {
                hud_key(controls_hint, 411u);
            } else {
                hud_key(controls_hint, 412u);
            }
        }
        const auto energy = ratio_scale(state.health, state.maximum, 4096u);
        const auto source = ratio_scale(state.health, state.maximum, hud_source_width(energy_bar));
        hud_crop(energy_bar, (hud_authored_width(energy_bar) * hud_fixed(energy.quotient)), source.quotient);
        if ((source.remainder > 0u)) {
            hud_crop(energy_bar, (hud_authored_width(energy_bar) * hud_fixed(energy.quotient)), (source.quotient + 1u));
        }
        const auto percentage = ratio_scale(state.health, state.maximum, 100u);
        if (((percentage.quotient < 25u) || ((percentage.quotient == 25u) && (percentage.remainder == 0u)))) {
            hud_tint(energy_bar, style.critical_red, style.critical_green, style.critical_blue);
        } else {
            if (((percentage.quotient < 60u) || ((percentage.quotient == 60u) && (percentage.remainder == 0u)))) {
                hud_tint(energy_bar, style.warning_red, style.warning_green, style.warning_blue);
            } else {
                hud_tint(energy_bar, style.normal_red, style.normal_green, style.normal_blue);
            }
        }
        const auto boss_ratio = ratio_scale(state.boss_health, state.boss_maximum, 4096u);
        hud_progress(boss_bar, hud_fixed(boss_ratio.quotient));
        damage_hud(read_damage());
        return;
    }
    bool station_available(uint32_t port) const {
        return (((runtime.controls && (port == runtime.input_port)) && (screen.menu == 0u)) && !(screen.busy));
    }
    bool open_station(uint32_t menu,uint32_t port) {
        if (!(station_available(port))) {
            return false;
        }
        if (((menu != 1u) && (menu != 2u))) {
            return false;
        }
        assign_screen(make_screen(menu, 0u, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 4u, true, screen.busy));
        assign_open_cue(true);
        prepare_view();
        return true;
    }
    void station_focus(uint32_t menu,bool focused) {
        if (focused) {
            assign_station(menu);
        } else {
            if ((runtime.station == menu)) {
                assign_station(0u);
            }
        }
        render_hud(read_hud());
        return;
    }
    bool frame_initialize(DungeonEconomyUIContext context,uint32_t microseconds) {
        assign_controls(context.controls);
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 0u, screen.visible, screen.busy));
        if (runtime.open_cue) {
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 4u, screen.visible, screen.busy));
            assign_open_cue(false);
        }
        assign_heal(0u);
        if (!(same_policy())) {
            shutdown();
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, 0u, 0u, screen.notice, screen.progress, screen.cue, screen.visible, false));
            assign_started(false);
            assign_bound(policy);
        }
        if (!(policy_live())) {
            shutdown();
            assign_screen(make_screen(screen.menu, screen.selected, screen.port, 0u, 0u, screen.notice, screen.progress, screen.cue, screen.visible, false));
            assign_started(false);
            return false;
        }
        if (!(runtime.started)) {
            assign_started(true);
            if ((read_tx().phase == 2u)) {
                assign_screen(make_screen(screen.menu, screen.selected, screen.port, 2u, read_tx().request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
            } else {
                if (!(read_loaded())) {
                    load();
                }
            }
        }
        poll_card();
        if ((read_pocket().operation != 0u)) {
            dispatch(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->pocket_update(context.dt, microseconds):DungeonSupplyDecision{};}()));
            assign_stats(make_stats(runtime.transfers, runtime.saves, read_pocket().error));
        }
        return true;
    }
    DungeonEconomyUIOutput frame_controls(DungeonEconomyUIContext context) {
        if ((epok::bp::api::valid(view) && !(screen.busy))) {
            const auto vertical = epok::GameplayInputLibrary::context_sample(capture_token(), input_action(1u), true);
            const auto horizontal = epok::GameplayInputLibrary::context_sample(capture_token(), input_action(0u), true);
            const auto cancel_key = epok::GameplayInputLibrary::context_take_pressed(capture_token(), input_action(7u), true);
            const auto inventory_key = epok::GameplayInputLibrary::context_take_pressed(capture_token(), input_action(12u), true);
            const auto confirm_key = epok::GameplayInputLibrary::context_take_pressed(capture_token(), input_action(6u), true);
            const auto buttons = make_buttons((vertical.pressed && (vertical.value > epok::Fixed(0.0))), (vertical.pressed && (vertical.value < epok::Fixed(0.0))), (horizontal.pressed && (horizontal.value <= epok::Fixed(0.0))), (horizontal.pressed && (horizontal.value >= epok::Fixed(0.0))), confirm_key, (cancel_key || inventory_key));
            const auto intent = ([&]() -> DungeonEconomyIntent {auto* target=view_value();return target?target->input(buttons):DungeonEconomyIntent{};}());
            if (((intent.selected != screen.selected) || (intent.port != screen.port))) {
                assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 3u, screen.visible, screen.busy));
            }
            assign_screen(make_screen(screen.menu, intent.selected, intent.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, screen.busy));
            if ((intent.command == 2u)) {
                ([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->leave():DungeonSupplyDecision{};}());
                assign_screen(make_screen(0u, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 5u, screen.visible, screen.busy));
                close_view();
                return make_output(((screen.menu != 0u) || screen.busy), !(screen.visible), pending_heal(), screen.cue, screen.menu, screen.selected);
            }
            if ((intent.command == 1u)) {
                assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, 6u, screen.visible, screen.busy));
                dispatch(([&]() -> DungeonSupplyDecision {auto* target=policy_value();return target?target->activate(screen.menu, screen.selected, context.can_heal):DungeonSupplyDecision{};}()));
            }
        }
        assign_screen(make_screen(screen.menu, screen.selected, screen.port, screen.card_phase, screen.request, screen.notice, screen.progress, screen.cue, screen.visible, ((screen.card_phase != 0u) || (read_pocket().operation != 0u))));
        if (epok::bp::api::valid(view)) {
            ([&]() {auto* target=view_value();if(target)target->refresh(screen);}());
        }
        return make_output(((screen.menu != 0u) || screen.busy), !(screen.visible), pending_heal(), screen.cue, screen.menu, screen.selected);
    }
    void on_disable()override{shutdown();}
    void end_play(epok::EndPlayReason)override{shutdown();}
};
