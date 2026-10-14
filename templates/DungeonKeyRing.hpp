#pragma once
#include <stdint.h>

// The keys picked up in the current map, one bit per lock number (1 to 31). Each key
// clears its bit when the map starts; locked doors open once theirs is set.
namespace dungeon {
inline uint32_t key_ring = 0;
inline uint32_t key_bit(uint32_t lock) { return lock ? 1u << (lock & 31u) : 0u; }
}  // namespace dungeon
