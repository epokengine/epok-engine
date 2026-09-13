#include "../../runtime/instrument_allocator.hpp"
#include <cassert>
#include <cstdio>
using namespace epok::instrument::allocation;
int main() {
    Slot slots[24]{};
    Request request{0,128,2,16,16,1};
    auto plan = reserve(slots, request);
    assert(plan.fits && plan.start_mask == 3 && !plan.evict_mask);
    // Two simultaneous sequences share a physical ceiling; free hardware voices
    // do not let the next sequence exceed it.
    for (int i = 0; i < 16; ++i) slots[i] = {true,true,uint8_t(i / 8),uint8_t(i / 2),128,1,uint32_t(i / 2)};
    request.owner = 2;
    plan = reserve(slots,request);
    assert(plan.fits && plan.evict_mask == 3 && plan.start_mask == 3);
    for (auto& s : slots) s = {};
    for (int i = 0; i < 23; ++i) slots[i] = {true,false,0,0,220,0,uint32_t(i)};
    plan = reserve(slots,request);
    assert(!plan.fits && !plan.evict_mask && !plan.start_mask);
    // A lower-priority group is evicted together, even if one slot would suffice.
    slots[0] = {true,true,1,7,100,1,1}; slots[1] = slots[0];
    request.layers = 2;
    plan = reserve(slots,request);
    assert(plan.fits && plan.evict_mask == 3 && plan.start_mask == 3);
    // Mixed-priority input cannot leak a higher-priority group member.
    slots[1].priority = 220;
    assert(!reserve(slots,request).fits);
    request.layers = 17;
    assert(!reserve(slots,request).fits);
    for (auto& s : slots) s = {};
    slots[0] = {true,true,0,3,128,1,1};
    slots[1] = {true,true,0,3,128,2,1};
    request = {0,128,1,1,24,1};
    plan = reserve(slots,request);
    assert(plan.fits && plan.evict_mask == 1); // Generation distinguishes a recycled instance.
    std::puts("instrument_allocator: complete groups, aggregate ceiling, priority and generation passed");
}
