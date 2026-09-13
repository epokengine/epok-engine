#include "../../runtime/instrument_reverb.hpp"
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>

using namespace epok::instrument::reverb;

struct MockHardware {
    struct Event { uint8_t kind; uintptr_t address; uint32_t value; };
    inline static std::array<uint16_t, 128> registers{};
    inline static std::vector<Event> events;
    inline static bool fail_zero = false;
    static size_t index(uintptr_t address) {
        assert(address >= 0x1f801d00 && address < 0x1f801e00);
        return size_t((address - 0x1f801d00) / 2);
    }
    static void reset() {
        registers.fill(0); events.clear(); fail_zero = false;
        registers[index(0x1f801daa)] = 0xc000;
    }
    static uint16_t read16(uintptr_t address) { return registers[index(address)]; }
    static void write16(uintptr_t address, uint16_t value) {
        registers[index(address)] = value; events.push_back({0, address, value});
    }
    static bool zero_spu(uint32_t address, uint32_t bytes) {
        events.push_back({1, address, bytes}); return !fail_zero;
    }
};

using Reverb = Resource<MockHardware>;

uint16_t reg(uintptr_t address) { return MockHardware::read16(address); }
size_t zero_count() {
    size_t count = 0;
    for (const auto& event : MockHardware::events) if (event.kind == 1) ++count;
    return count;
}

void dry_and_budget_contract() {
    MockHardware::reset(); Reverb reverb;
    assert(reverb.prepare(4096, Preset::Dry, 1) == Error::InvalidDepth);
    assert(reverb.prepare(4096, Preset::Dry, 0) == Error::None);
    assert(reverb.prepared() && !reverb.active());
    assert(reverb.reserved_bytes() == 0 && reverb.reserved_begin() == spu_bytes);
    assert(reverb.acquire(2, 9) == Error::None);
    assert(!reverb.active() && reverb.send(0, true) == Error::NotOwner);
    const auto events_before_bad_budget = MockHardware::events;
    assert(reverb.prepare(room_begin + 1, Preset::Room, 1000) == Error::SampleBudgetConflict);
    assert(MockHardware::events.size() == events_before_bad_budget.size());
    assert(zero_count() == 0);
}

void room_prepare_writes_exact_preset_and_reservation() {
    MockHardware::reset(); Reverb reverb;
    assert(reverb.prepare(room_begin, Preset::Room, 0x4321) == Error::None);
    assert(reverb.prepared() && !reverb.active());
    assert(reverb.reserved_begin() == room_begin && reverb.reserved_bytes() == 0x26c0);
    assert(zero_count() == 1);
    assert(MockHardware::events.back().kind == 1);
    assert(reg(0x1f801da2) == room_begin / 8 && reg(0x1f801dac) == 4);
    for (uint16_t i = 0; i < 32; ++i) assert(reg(0x1f801dc0 + uintptr_t(i) * 2) == room_registers[i]);
    assert(reg(0x1f801d84) == 0 && reg(0x1f801d86) == 0);
    assert((reg(0x1f801daa) & 0x0080) == 0);
}

void lease_conflict_send_masks_and_tail_handoff() {
    MockHardware::reset(); Reverb reverb;
    assert(reverb.prepare(0x20000, Preset::Room, 0x1234) == Error::None);
    assert(reverb.acquire(4, 10) == Error::None);
    assert(reverb.active() && reverb.owner() == 4 && reverb.generation() == 10);
    assert(reg(0x1f801d84) == 0x1234 && reg(0x1f801d86) == 0x1234);
    assert(reg(0x1f801daa) & 0x0080);
    assert(reverb.acquire(5, 11) == Error::LeaseConflict);
    assert(reverb.send(0, true) == Error::None);
    assert(reverb.send(17, true) == Error::None);
    assert(reg(0x1f801d98) == 1 && reg(0x1f801d9a) == 2);
    assert(reverb.send(24, true) == Error::InvalidVoice);
    assert(reverb.release(5, 11) == Error::NotOwner);
    assert(reverb.active() && reverb.send_mask() == 0x20001);
    reverb.clear_voice(17);
    assert(reverb.send_mask() == 1 && reg(0x1f801d9a) == 0);
    assert(reverb.release(4, 10) == Error::None);
    assert(!reverb.active() && reverb.send_mask() == 0);
    assert(reg(0x1f801d84) == 0 && (reg(0x1f801daa) & 0x0080) == 0);

    const size_t before = zero_count();
    const size_t handoff_event_begin = MockHardware::events.size();
    assert(reverb.acquire(5, 11) == Error::None);
    assert(zero_count() == before + 1);
    size_t zero_event = MockHardware::events.size(), unmute_event = MockHardware::events.size();
    for (size_t i = handoff_event_begin; i < MockHardware::events.size(); ++i) {
        const auto& event = MockHardware::events[i];
        if (event.kind == 1 && zero_event == MockHardware::events.size()) zero_event = i;
        if (event.kind == 0 && event.address == 0x1f801daa && (event.value & 0x80)) unmute_event = i;
    }
    assert(zero_event < unmute_event);
}

void dma_failure_and_teardown_stay_muted() {
    MockHardware::reset(); Reverb reverb;
    MockHardware::fail_zero = true;
    assert(reverb.prepare(4096, Preset::Room, 32767) == Error::DmaTimeout);
    assert(!reverb.prepared() && !reverb.active());
    assert((reg(0x1f801daa) & 0x80) == 0 && reg(0x1f801d84) == 0);
    MockHardware::fail_zero = false;
    assert(reverb.prepare(4096, Preset::Room, 32767) == Error::None);
    assert(reverb.acquire(1, 1) == Error::None);
    MockHardware::fail_zero = true;
    assert(reverb.teardown() == Error::DmaTimeout);
    assert(reverb.prepared() && !reverb.active() && reverb.reserved_bytes() == room_bytes);
    assert(reg(0x1f801d84) == 0 && reg(0x1f801d86) == 0);
    assert(reg(0x1f801d98) == 0 && reg(0x1f801d9a) == 0);
    assert((reg(0x1f801daa) & 0x80) == 0);
    MockHardware::fail_zero = false;
    assert(reverb.teardown() == Error::None);
    assert(!reverb.prepared() && !reverb.active() && reverb.reserved_bytes() == 0);
    assert(reg(0x1f801d98) == 0 && reg(0x1f801d9a) == 0);
}

int main() {
    static_assert(room_begin == 0x7d940 && room_bytes / 64 == 155);
    static_assert(sizeof(Reverb) <= 24, "global reverb state stays bounded");
    dry_and_budget_contract();
    room_prepare_writes_exact_preset_and_reservation();
    lease_conflict_send_masks_and_tail_handoff();
    dma_failure_and_teardown_stay_muted();
    std::printf("instrument_reverb: Resource=%zu bytes, Room=%u bytes; mocked preset, reservation, lease and DMA failures passed\n",
        sizeof(Reverb), room_bytes);
}
