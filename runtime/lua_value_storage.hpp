#pragma once
#include <stddef.h>
#include <stdint.h>

namespace epok::lua::detail {
// Immutable snapshots stay at stable addresses. The compiler supplies every
// parameter/local root at statement boundaries; only unrooted temporaries are
// reusable. Each re-entrant call has an independent frame and root mask.
template<class Value, size_t Depth, size_t Slots> class ValueStorage {
    static_assert(Depth > 0 && Slots > 0 && Slots <= 64);
    Value values_[Depth][Slots]{};
    uint64_t occupied_[Depth]{};
public:
    void begin(size_t frame) { if(frame < Depth) occupied_[frame] = 0; }
    bool contains(size_t frame, const void* candidate) const {
        if(frame >= Depth) return false;
        for(size_t index = 0; index < Slots; ++index)
            if((occupied_[frame] & (uint64_t(1) << index)) && candidate == &values_[frame][index]) return true;
        return false;
    }
    Value* retain(size_t frame, const Value& value) {
        if(frame >= Depth) return nullptr;
        for(size_t index = 0; index < Slots; ++index) {
            const uint64_t bit = uint64_t(1) << index;
            if(!(occupied_[frame] & bit)) {
                values_[frame][index] = value;
                occupied_[frame] |= bit;
                return &values_[frame][index];
            }
        }
        return nullptr;
    }
    uint64_t root(size_t frame, const void* candidate) const {
        if(frame >= Depth) return 0;
        for(size_t index = 0; index < Slots; ++index)
            if((occupied_[frame] & (uint64_t(1) << index)) && candidate == &values_[frame][index]) return uint64_t(1) << index;
        return 0;
    }
    void sweep(size_t frame, uint64_t roots) { if(frame < Depth) occupied_[frame] &= roots; }
};
}
