#pragma once
#include <cstdint>
#include "sequence_kernel.hpp"

// Mono, decoded target PCM. `frames` includes the mandatory initial silent ADPCM
// block and is checked against the EPSB v2 Sample record before preview starts.
struct EpokInstrumentPcm {
    const float* framesdata;
    uint32_t frames;
};

struct EpokInstrumentStats {
    uint32_t error;
    uint32_t logical_peak;
    uint32_t physical_peak;
    uint32_t steals;
    uint32_t denied;
    uint32_t loops;
    uint32_t clipped;
    uint32_t sample_loops;
};

extern "C" {
void* epok_instrument_create(
    const epok::sequence::Event* events,
    uint32_t count,
    uint16_t ppqn,
    uint16_t voice_limit,
    const uint8_t* bank_bytes,
    uint32_t bank_len,
    const EpokInstrumentPcm* samples,
    uint32_t sample_count,
    uint32_t rate);
void epok_instrument_destroy(void*);
int epok_instrument_render(void*, int16_t* stereo_output, uint32_t frames);
EpokInstrumentStats epok_instrument_stats(const void*);
int epok_instrument_prepared_count(const epok::sequence::Event*,uint32_t count,uint16_t ppqn,const uint8_t*,uint32_t size,uint32_t* references);
}
