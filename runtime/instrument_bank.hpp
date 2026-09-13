#pragma once
// EPSB v2 contract. Immutable, relative-offset tables; validation never allocates.
// EPSB v1 is still owned by sequence_data.hpp and is never reinterpreted as v2.
#include <cstdint>
#include <cstddef>

namespace epok::instrument {
inline uint16_t u16(const uint8_t* p) { return uint16_t(p[0]) | uint16_t(p[1]) << 8; }
inline uint32_t u32(const uint8_t* p) { return uint32_t(u16(p)) | uint32_t(u16(p + 2)) << 16; }
struct Sample { uint32_t offset, size, rate, frames, loop_start, loop_end; };
struct Envelope {
    int32_t delay, attack, hold, decay, sustain, release, hold_key, decay_key;
};
struct Lfo { int32_t delay, frequency, pitch, volume; };
struct Zone {
    uint16_t sample, bank;
    uint8_t program, percussion, key_lo, key_hi, velocity_lo, velocity_hi;
    uint8_t root_key, fixed_key, fixed_velocity, loop_mode;
    uint16_t exclusive_class;
    int32_t tune, scale, attenuation, pan, mod_env_pitch, reverb;
    Envelope volume_envelope, modulation_envelope;
    Lfo modulation_lfo, vibrato_lfo;
    uint16_t mod_begin, mod_count;
    uint32_t reserved;
};
struct Modulation { uint16_t source, amount_source, destination, flags; int32_t amount; uint32_t reserved; };
static_assert(sizeof(Sample) == 24 && sizeof(Envelope) == 32 && sizeof(Lfo) == 16);
static_assert(sizeof(Zone) == 144 && offsetof(Zone, volume_envelope) == 40 && offsetof(Zone, mod_begin) == 136);
static_assert(sizeof(Modulation) == 16);

enum Destination : uint16_t {
    Pitch = 1, Scale, Attenuation, Pan, ModEnvPitch, ModLfoPitch, VibLfoPitch, ModLfoVolume,
    ModLfoDelay, ModLfoFrequency, VibLfoDelay, VibLfoFrequency,
    VolEnvDelay, VolEnvAttack, VolEnvHold, VolEnvDecay, VolEnvSustain, VolEnvRelease, VolEnvHoldKey, VolEnvDecayKey,
    ModEnvDelay, ModEnvAttack, ModEnvHold, ModEnvDecay, ModEnvSustain, ModEnvRelease, ModEnvHoldKey, ModEnvDecayKey,
    Reverb, Chorus
};
inline bool range(int32_t value, int32_t lo, int32_t hi) { return value >= lo && value <= hi; }
inline bool time(int32_t value, int32_t maximum, bool zero) { return (zero && value == -32768) || range(value, -12000, maximum); }
inline bool valid_envelope(const Envelope& e, bool volume) {
    return time(e.delay, 5000, true) && time(e.attack, 8000, true) && time(e.hold, 5000, true) &&
        time(e.decay, 8000, false) && time(e.release, 8000, false) && range(e.sustain, 0, volume ? 1440 : 1000) &&
        range(e.hold_key, -1200, 1200) && range(e.decay_key, -1200, 1200);
}
inline bool valid_lfo(const Lfo& lfo, bool vibrato) {
    return time(lfo.delay, 5000, true) && range(lfo.frequency, -16000, 4500) && range(lfo.pitch, -12000, 12000) &&
        (vibrato ? lfo.volume == 0 : range(lfo.volume, -960, 960));
}
inline bool valid_source(uint16_t bits) {
    if (bits & 0xf000) return false;
    if (bits & 128) {
        const auto cc = bits & 127;
        return cc != 0 && cc != 6 && !(cc >= 32 && cc <= 63) && !(cc >= 98 && cc <= 101) && cc < 120;
    }
    switch (bits & 127) { case 0: case 2: case 3: case 10: case 13: case 14: case 16: return true; default: return false; }
}
struct BankView {
    const uint8_t* data;
    uint32_t size;
    uint16_t sample_count() const { return u16(data + 8); }
    uint16_t zone_count() const { return u16(data + 10); }
    uint32_t modulation_count() const { return u32(data + 32); }
    uint32_t sample_bytes() const { return size - u32(data + 24); }
    uint32_t reverb_preset() const { return u32(data + 40); }
    uint16_t reverb_depth() const { return uint16_t(u32(data + 44)); }
    uint32_t reverb_bytes() const { return reverb_preset() == 1 ? 0x26c0 : 0; }
    const Sample& sample(uint16_t i) const { return reinterpret_cast<const Sample*>(data + 48)[i]; }
    const Zone& zone(uint16_t i) const { return reinterpret_cast<const Zone*>(data + u32(data + 16))[i]; }
    const Modulation& modulation(uint16_t i) const { return reinterpret_cast<const Modulation*>(data + u32(data + 20))[i]; }
    // Analysis may inspect an over-budget payload; playback always enforces it.
    bool valid(bool enforce_spu_budget = true) const {
        if (!data || uintptr_t(data) % 4 || size < 48 || size > 64 * 1024 * 1024 || data[0] != 'E' || data[1] != 'P' || data[2] != 'S' || data[3] != 'B' ||
            u16(data + 4) != 2 || u16(data + 6) != 48 || u32(data + 12) != 48 || u32(data + 28) != size ||
            u32(data + 36) != 2 || reverb_preset() > 1 || u32(data + 44) > 32767 || (!reverb_preset() && u32(data + 44))) return false;
        const uint32_t ns = sample_count(), nz = zone_count(), nm = modulation_count();
        const uint32_t zo = u32(data + 16), mo = u32(data + 20), so = u32(data + 24);
        if (!ns || ns > 128 || !nz || nz > 128 || nm > 4096 || zo != 48 + ns * 24 || mo != zo + nz * 144 ||
            so != (mo + nm * 16 + 63) / 64 * 64 || so > size || (enforce_spu_budget && size - so > 512 * 1024 - 4096 - reverb_bytes())) return false;
        uint32_t expected = so;
        for (uint16_t i = 0; i < ns; ++i) {
            const auto& s = sample(i);
            if (s.offset != expected || s.size < 64 || s.size % 64 || s.size > size - expected || s.rate < 400 || s.rate > 44100 ||
                s.frames < 56 || s.frames % 28 || uint64_t(s.frames / 28 + 1) * 16 > s.size ||
                (s.loop_end ? (s.loop_start < 28 || s.loop_start >= s.loop_end || s.loop_end > s.frames || s.loop_start % 28 || s.loop_end % 28) : s.loop_start != 0)) return false;
            // Initial silence, predictive data, end/repeat flags and terminal block
            // are part of the bank contract, not unvalidated decoder input.
            for (uint32_t b = 0; b < 16; ++b) if (data[s.offset + b]) return false;
            const uint32_t blocks = s.frames / 28;
            for (uint32_t b = 1; b < blocks; ++b) {
                const uint8_t header = data[s.offset + b * 16], flags = data[s.offset + b * 16 + 1];
                if ((header >> 4) > 4 || (header & 15) > 12 || flags & ~7) return false;
                uint8_t wanted = b == blocks - 1 ? 1 : 0;
                if (s.loop_end) {
                    if (b == s.loop_end / 28 - 1) wanted = 3;
                    if (b == s.loop_start / 28) wanted |= 4;
                }
                const bool manual_repeat = s.loop_end && b == s.loop_start / 28 && flags == (wanted & ~4);
                if (flags != wanted && !manual_repeat) return false;
            }
            if (data[s.offset + blocks * 16] || data[s.offset + blocks * 16 + 1] != 7) return false;
            for (uint32_t b = blocks * 16 + 2; b < s.size; ++b) if (data[s.offset + b]) return false;
            expected += s.size;
        }
        if (expected != size) return false;
        uint32_t expected_mod = 0;
        for (uint16_t i = 0; i < nz; ++i) {
            const auto& z = zone(i);
            if (z.sample >= ns || z.bank > 16383 || z.program > 127 || z.percussion > 1 || z.key_lo > z.key_hi || z.key_hi > 127 ||
                z.velocity_lo > z.velocity_hi || z.velocity_hi > 127 || z.root_key > 127 ||
                (z.fixed_key != 255 && z.fixed_key > 127) || (z.fixed_velocity != 255 && z.fixed_velocity > 127) ||
                (z.loop_mode != 0 && z.loop_mode != 1 && z.loop_mode != 3) || z.exclusive_class > 127 ||
                !range(z.tune, -14000, 14000) || !range(z.scale, 0, 1200) || !range(z.attenuation, 0, 1440) || !range(z.pan, -500, 500) ||
                !range(z.mod_env_pitch, -12000, 12000) || !range(z.reverb, 0, 1000) || (!reverb_preset() && z.reverb) || !valid_envelope(z.volume_envelope, true) ||
                !valid_envelope(z.modulation_envelope, false) || !valid_lfo(z.modulation_lfo, false) || !valid_lfo(z.vibrato_lfo, true) ||
                z.mod_begin != expected_mod || z.mod_count > 32 || z.mod_count > nm - expected_mod || z.reserved) return false;
            const auto& s = sample(z.sample);
            if ((z.loop_mode != 0) != (s.loop_end != 0) || (z.loop_mode == 1 && s.loop_end != s.frames)) return false;
            if (z.loop_mode == 3 && (data[s.offset + s.loop_start / 28 * 16 + 1] & 4)) return false;
            if (z.loop_mode == 1 && !(data[s.offset + s.loop_start / 28 * 16 + 1] & 4)) return false;
            expected_mod += z.mod_count;
        }
        if (expected_mod != nm) return false;
        for (uint16_t i = 0; i < nm; ++i) {
            const auto& m = modulation(i);
            if (!valid_source(m.source) || !valid_source(m.amount_source) || m.destination < Pitch || m.destination > Chorus ||
                m.flags > 1 || m.reserved || (m.destination == Chorus && m.amount) || (!reverb_preset() && m.destination == Reverb && m.amount)) return false;
        }
        return true;
    }
};
}
