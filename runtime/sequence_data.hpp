#pragma once
#include "instrument_preparation.hpp"
// EPSQ v1/v2 share the 40-byte header and 12-byte event layout.
#include "sequence_kernel.hpp"
#include "instrument_bank.hpp"
#include "native_music_data.hpp"

namespace epok::psx_audio {
inline uint16_t read16(const uint8_t* p) { return uint16_t(p[0]) | uint16_t(p[1]) << 8; }
inline uint32_t read32(const uint8_t* p) { return uint32_t(read16(p)) | uint32_t(read16(p + 2)) << 16; }
struct Sample {
    uint32_t offset, size, rate, frames, loop_start, loop_end;
};
struct Zone {
    uint16_t sample;
    uint8_t program, drum_key, key_lo, key_hi, velocity_lo, velocity_hi, root_key, reserved;
    int16_t cents;
    uint16_t gain;
    int16_t pan;
    uint32_t attack_ms, decay_ms;
    uint16_t sustain, reserved2;
    uint32_t release_ms;
};
static_assert(sizeof(Sample) == 24 && sizeof(Zone) == 32);
struct Bank {
    const uint8_t* data = nullptr;
    uint32_t size = 0;
    uint16_t addresses[128]{};
    bool ready = false;
    uint16_t pins = 0;
    bool is_library() const { return data && size >= 8 && read16(data + 4) == 2; }
    instrument::BankView library() const { return {data, size}; }
    uint16_t sample_count() const { return read16(data + 8); }
    uint16_t zone_count() const { return read16(data + 10); }
    const Sample& sample(uint16_t i) const { return reinterpret_cast<const Sample*>(data + read32(data + 12))[i]; }
    const Zone& zone(uint16_t i) const { return reinterpret_cast<const Zone*>(data + read32(data + 16))[i]; }
    bool valid() const {
        if (is_library()) return library().valid();
        if (!data || uintptr_t(data) % 4 || size < 32 || data[0] != 'E' || data[1] != 'P' || data[2] != 'S' || data[3] != 'B' ||
            read16(data + 4) != 1 || read16(data + 6) != 32 || read32(data + 12) != 32 || read32(data + 24) != size || read32(data + 28)) return false;
        const uint32_t samples = sample_count(), zones = zone_count(), zo = read32(data + 16), pcm = read32(data + 20);
        if (!samples || samples > 128 || !zones || zones > 128 || zo != 32 + samples * 24 ||
            pcm != (zo + zones * 32 + 63) / 64 * 64 || pcm > size) return false;
        uint32_t expected = pcm;
        for (uint16_t i = 0; i < samples; ++i) {
            const auto& s = sample(i);
            if (s.offset != expected || !s.size || s.size % 64 || s.size > size - expected ||
                (s.rate != 11025 && s.rate != 22050 && s.rate != 44100) || s.frames < 56 || s.frames % 28 ||
                uint64_t(s.frames / 28 + 1) * 16 > s.size ||
                (s.loop_end && (s.loop_start < 28 || s.loop_start >= s.loop_end || s.loop_end != s.frames || s.loop_start % 28)) ||
                (!s.loop_end && s.loop_start)) return false;
            expected += s.size;
        }
        if (expected != size || size - pcm > 512 * 1024 - 4096) return false;
        for (uint16_t i = 0; i < zones; ++i) {
            const auto& z = zone(i);
            if (z.sample >= samples || z.program > 127 || (z.drum_key != 255 && z.drum_key > 127) ||
                z.key_lo > z.key_hi || z.key_hi > 127 || !z.velocity_lo || z.velocity_lo > z.velocity_hi || z.velocity_hi > 127 ||
                z.root_key > 127 || z.reserved || z.reserved2 || z.cents < -10000 || z.cents > 10000 || z.gain > 16384 ||
                z.pan < -16384 || z.pan > 16384 || z.attack_ms > 60000 || z.decay_ms > 60000 || z.release_ms > 60000 || z.sustain > 32767) return false;
        }
        return true;
    }
};
struct Sequence {
    const uint8_t* data;
    uint32_t size;
    Bank* bank;
    instrument::preparation::Cache* prepared=nullptr;
    bool is_native()const{return data && size>=40 && read16(data+4)==3;}
    native_music::View native()const{return {data,size};}
    uint16_t ppqn() const { return read16(data + 8); }
    uint16_t voices() const { return read16(data + 10); }
    uint32_t count() const { return read32(data + 12); }
    const sequence::Event* events() const { return reinterpret_cast<const sequence::Event*>(data + 40); }
    bool valid() const {
        if (!data || uintptr_t(data) % 4 || size < 40 || size > 256 * 1024) return false;
        const uint16_t version = read16(data + 4);
        if(version==3)return bank && bank->valid() && data[0]=='E' && data[1]=='P' && data[2]=='S' && data[3]=='Q' && native().valid(bank->sample_count());
        if (!bank || !bank->valid() ||
            data[0] != 'E' || data[1] != 'P' || data[2] != 'S' || data[3] != 'Q' ||
            (version != 1 && version != 2) || read16(data + 6) != 40 || !ppqn() || ppqn() > 32767 || !voices() || voices() > 24 ||
            !count() || count() > sequence::Kernel::MaxEvents || count() != (size - 40) / 12 || (size - 40) % 12 || read32(data + 20)) return false;
        bool loop = false, ended = false;
        uint32_t loop_tick = 0;
        for (uint32_t i = 0; i < count(); ++i) {
            const auto& e = events()[i];
            if (ended || (i && e.tick < events()[i - 1].tick) || !sequence::valid_event(e, version == 2)) return false;
            if (e.op == sequence::LoopStart) { if (loop) return false; loop = true; loop_tick = e.tick; }
            if (e.op == sequence::LoopEnd) { if (!loop || e.tick <= loop_tick) return false; ended = true; }
            if (e.op == sequence::End) { if (loop) return false; ended = true; }
        }
        return ended;
    }
};
}
