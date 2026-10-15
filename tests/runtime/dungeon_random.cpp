// The console twin of the dungeon generator's random stream must reproduce the
// editor's numbers exactly; src/dungeon_random.rs asserts the same values.
#include <cassert>
#include <cstdint>
#include <cstdio>
#include "../../runtime/dungeon_random.hpp"

using epok::dungeon::Random;

int main() {
    Random zero(0);
    const uint32_t zero_values[] = {0xe308dc58u, 0x4392d0e4u, 0x03318f97u, 0xac593a63u};
    for (uint32_t value : zero_values) assert(zero.next() == value);
    Random seeded(12345);
    const uint32_t seeded_values[] = {0x1eea3cc1u, 0x1a40a62eu, 0xfc4cd240u, 0xdffc5b56u};
    for (uint32_t value : seeded_values) assert(seeded.next() == value);
    Random digits(12345);
    const uint32_t digit_values[] = {1, 1, 9, 8, 9, 3, 8, 4, 6, 1, 2, 7};
    for (uint32_t value : digit_values) assert(digits.below(10) == value);
    Random large(0xdeadbeefu);
    const uint32_t large_values[] = {537, 599, 291, 628, 540, 741};
    for (uint32_t value : large_values) assert(large.below(1000) == value);
    Random bounded(7);
    for (int i = 0; i < 2000; ++i) {
        const int32_t value = bounded.range(-3, 4);
        assert(value >= -3 && value <= 4);
    }
    assert(bounded.range(9, 2) == 9 && bounded.below(0) == 0);
    Random a = Random::stream(42, 1), b = Random::stream(42, 1);
    assert(a.next() == b.next());
    std::puts("Dungeon random stream tests passed.");
    return 0;
}
