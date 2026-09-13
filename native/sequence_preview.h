#pragma once
#include <cstdint>
#include "sequence_kernel.hpp"
struct EpokSequenceZone {
    const float* samples;
    uint32_t frames, rate, loop_start, loop_end;
    uint16_t channels;
    uint8_t program, drum_key, key_lo, key_hi, velocity_lo, velocity_hi, root_key, reserved;
    float cents, gain, pan, attack_ms, decay_ms, sustain, release_ms;
};
struct EpokSequenceStats { uint32_t error, peak, steals, loops, clipped; };
extern "C" {
void* epok_sequence_create(const epok::sequence::Event*, uint32_t, uint16_t, uint16_t, const EpokSequenceZone*, uint32_t, uint32_t);
void epok_sequence_destroy(void*);
int epok_sequence_render(void*, int16_t*, uint32_t);
EpokSequenceStats epok_sequence_stats(const void*);
}
