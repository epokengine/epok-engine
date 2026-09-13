#pragma once

#include <cstdint>
#include "sequence_kernel.hpp"

// Source-library host audition ABI.  PCM remains in the SoundFont's decoded
// coordinate system: no ADPCM block, resampling, channel conversion, or target
// filter bake is represented here.
struct EpokSourcePcm {
    const float* framesdata;
    uint32_t frames;
    uint32_t rate;
};

struct EpokSourceEnvelope {
    int32_t delay, attack, hold, decay, sustain, release, hold_key, decay_key;
};

struct EpokSourceLfo {
    int32_t delay, frequency, pitch, filter, volume;
};

// `source`/`amount_source` use the SoundFont source-bit representation shared
// with instrument_synth.hpp. `flags & 1` is the absolute-transform bit.
struct EpokSourceModulation {
    uint16_t source, amount_source, destination, flags;
    int64_t amount;
};

enum EpokSourceDestination : uint16_t {
    EpokSourceStartFrames = 1, EpokSourceEndFrames, EpokSourceLoopStartFrames, EpokSourceLoopEndFrames,
    EpokSourceModLfoPitch, EpokSourceVibLfoPitch, EpokSourceModEnvPitch,
    EpokSourceFilterCents, EpokSourceFilterCentibels, EpokSourceModLfoFilter, EpokSourceModEnvFilter,
    EpokSourceModLfoVolume, EpokSourceChorus, EpokSourceReverb, EpokSourcePan,
    EpokSourceModLfoDelay, EpokSourceModLfoFrequency, EpokSourceVibLfoDelay, EpokSourceVibLfoFrequency,
    EpokSourceModEnvDelay, EpokSourceModEnvAttack, EpokSourceModEnvHold, EpokSourceModEnvDecay,
    EpokSourceModEnvSustain, EpokSourceModEnvRelease, EpokSourceModEnvHoldKey, EpokSourceModEnvDecayKey,
    EpokSourceVolEnvDelay, EpokSourceVolEnvAttack, EpokSourceVolEnvHold, EpokSourceVolEnvDecay,
    EpokSourceVolEnvSustain, EpokSourceVolEnvRelease, EpokSourceVolEnvHoldKey, EpokSourceVolEnvDecayKey,
    EpokSourceAttenuation, EpokSourcePitch, EpokSourceScale,
};

struct EpokSourceRegion {
    uint16_t sample, bank;
    uint8_t program, percussion, key_lo, key_hi, velocity_lo, velocity_hi;
    uint8_t root_key, fixed_key, fixed_velocity, loop_mode;
    uint16_t exclusive_class;
    int32_t tune, scale, attenuation, pan;
    int32_t filter_cents, filter_centibels, mod_env_pitch, mod_env_filter;
    int32_t reverb, chorus;
    int64_t start_offset, end_offset, loop_start, loop_end;
    EpokSourceEnvelope volume_envelope, modulation_envelope;
    EpokSourceLfo modulation_lfo, vibrato_lfo;
    const EpokSourceModulation* modulations;
    uint16_t modulation_count;
    uint16_t reserved;
};

struct EpokSourcePreviewStats {
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
void* epok_source_instrument_create(
    const epok::sequence::Event* events, uint32_t count, uint16_t ppqn,
    const EpokSourceRegion* regions, uint32_t region_count,
    const EpokSourcePcm* samples, uint32_t sample_count, uint32_t rate);
void epok_source_instrument_destroy(void*);
int epok_source_instrument_render(void*, int16_t* stereo_output, uint32_t frames);
EpokSourcePreviewStats epok_source_instrument_stats(const void*);
bool epok_source_instrument_set_gain(void*, float);
}
