#pragma once
// A bounded reservation plan for layered notes. Callers apply all evictions and
// starts only when fits==true, preserving a note's complete physical voice group.
#include <cstdint>

namespace epok::instrument::allocation {
struct Slot {
    bool occupied, musical;
    uint8_t owner, note, priority;
    uint32_t generation, age;
};
struct Request {
    uint8_t owner = 0, priority = 128, layers = 1, instance_limit = 16, aggregate_limit = 16;
    uint32_t generation = 0;
};
struct Plan { uint32_t evict_mask = 0, start_mask = 0; bool fits = false; };
inline bool same_group(const Slot& a, const Slot& b) {
    return a.musical && b.musical && a.owner == b.owner && a.generation == b.generation && a.note == b.note;
}
inline Plan reserve(const Slot (&slots)[24], const Request& request) {
    Plan result;
    if (!request.layers || request.layers > 24 || !request.instance_limit || request.instance_limit > 24 ||
        !request.aggregate_limit || request.aggregate_limit > 24 || request.layers > request.instance_limit || request.layers > request.aggregate_limit) return result;
    uint32_t alive = 0;
    for (int i = 0; i < 24; ++i) if (slots[i].occupied) alive |= 1u << i;
    for (int iteration = 0; iteration <= 24; ++iteration) {
        unsigned free = 0, own = 0, music = 0;
        for (int i = 0; i < 24; ++i) {
            if (!(alive & (1u << i))) { ++free; continue; }
            if (slots[i].musical) {
                ++music;
                if (slots[i].owner == request.owner && slots[i].generation == request.generation) ++own;
            }
        }
        const bool own_full = own + request.layers > request.instance_limit;
        const bool music_full = music + request.layers > request.aggregate_limit;
        if (!own_full && !music_full && free >= request.layers) {
            for (int i = 0, n = 0; i < 24 && n < request.layers; ++i) if (!(alive & (1u << i))) { result.start_mask |= 1u << i; ++n; }
            result.fits = true; return result;
        }
        int chosen = -1;
        for (int i = 0; i < 24; ++i) {
            const auto& slot = slots[i];
            if (!(alive & (1u << i)) || slot.priority > request.priority) continue;
            if (own_full && (!slot.musical || slot.owner != request.owner || slot.generation != request.generation)) continue;
            if (music_full && !slot.musical) continue;
            bool eligible = true;
            for (int j = 0; j < 24; ++j) if ((alive & (1u << j)) && same_group(slot, slots[j]) && slots[j].priority > request.priority) eligible = false;
            if (!eligible) continue;
            if (chosen < 0 || slot.priority < slots[chosen].priority || (slot.priority == slots[chosen].priority && slot.age < slots[chosen].age)) chosen = i;
        }
        if (chosen < 0) return Plan{}; // Do not expose a partial eviction plan.
        for (int i = 0; i < 24; ++i) if ((alive & (1u << i)) && (i == chosen || same_group(slots[chosen], slots[i]))) {
            alive &= ~(1u << i); result.evict_mask |= 1u << i;
        }
    }
    return Plan{};
}
}
