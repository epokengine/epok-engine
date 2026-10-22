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
    // Deterministic independent layouts: the fast no-eviction path must pick
    // exactly the same channels as the complete allocator, or defer to it.
    uint32_t seed=0x593a21;
    auto random=[&](){seed=seed*1664525u+1013904223u;return seed;};
    unsigned fast=0;
    for(unsigned trial=0;trial<100000;++trial){
        Request r{uint8_t(random()%4),uint8_t(random()%256),uint8_t(1+random()%8),uint8_t(1+random()%24),uint8_t(1+random()%24),1+random()%2};
        uint32_t available=0;unsigned own=0,music=0;
        for(unsigned p=0;p<24;++p){
            const auto bits=random();slots[p]={bool(bits&4),bool(bits&32),uint8_t((bits>>8)%4),uint8_t((bits>>16)%8),uint8_t(bits>>24),1+random()%2,random()};
            if(!slots[p].occupied)available|=1u<<p;
            else if(slots[p].musical){++music;if(slots[p].owner==r.owner&&slots[p].generation==r.generation)++own;}
        }
        const auto reference=reserve(slots,r),quick=reserve_free(available,own,music,r);
        if(quick.fits){++fast;assert(reference.fits&&!reference.evict_mask&&reference.start_mask==quick.start_mask);}
        else assert(!reference.fits||reference.evict_mask);
    }
    assert(fast>1000);
    std::puts("instrument_allocator: groups, ceilings, priority, generations and 100000 fast-path equivalence cases passed");
}
