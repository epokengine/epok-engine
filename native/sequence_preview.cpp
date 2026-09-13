#include "sequence_preview.h"
#include <cmath>
#include <new>
#include <algorithm>

namespace {
struct Voice {
    const EpokSequenceZone* zone = nullptr;
    double position = 0, step = 0;
    float level = 0, release_level = 0, left = 0, right = 0;
    uint64_t age = 0, release_age = 0;
    uint8_t key = 0, velocity = 0;
    bool releasing = false;
};
struct Preview {
    epok::sequence::Kernel kernel;
    Voice voices[epok::sequence::Kernel::MaxVoices]{};
    const EpokSequenceZone* zones = nullptr;
    uint32_t zone_count = 0, rate = 44100, clipped = 0;
    uint64_t frames = 0, micros = 0;
    void cut(uint16_t slot) { voices[slot] = Voice{}; }
    void release(uint16_t slot) {
        auto& v = voices[slot];
        if (!v.releasing) { v.releasing = true; v.release_level = v.level; v.release_age = 0; }
    }
    void update(uint16_t slot, const epok::sequence::Channel& c) {
        auto& v = voices[slot]; if (!v.zone) return;
        const auto& z = *v.zone;
        const double cents = double(c.pitch_cents100()) / 100.0;
        v.step = double(z.rate) / rate * std::pow(2.0, (double(v.key) - z.root_key + z.cents / 100.0 + cents / 100.0) / 12.0);
        const float pan = std::clamp(z.pan + (float(c.pan) - 64.f) / 64.f, -1.f, 1.f);
        const float gain = z.gain * float(v.velocity) / 127.f * float(c.volume) / 127.f * float(c.expression) / 127.f;
        v.left = gain * std::sqrt((1.f - pan) * .5f);
        v.right = gain * std::sqrt((1.f + pan) * .5f);
    }
    bool start(uint16_t slot, const epok::sequence::Note& n, const epok::sequence::Channel& c) {
        // The preview's v1 zone payload has only bank 0, as does EPSB v1.
        if (c.bank) return false;
        cut(slot);
        for (uint32_t i = 0; i < zone_count; ++i) {
            const auto& z = zones[i];
            if (z.program == c.program && z.drum_key == (n.channel == 9 ? n.key : 255) && n.key >= z.key_lo && n.key <= z.key_hi && n.velocity >= z.velocity_lo && n.velocity <= z.velocity_hi) {
                auto& v = voices[slot]; v.zone = &z; v.key = n.key; v.velocity = n.velocity;
                update(slot, c); return true;
            }
        }
        return false;
    }
    void render(int16_t* output, uint32_t count) {
        for (uint32_t f = 0; f < count; ++f) {
            const auto now = frames * 1000000 / rate;
            kernel.advance(uint32_t(now - micros), *this); micros = now; ++frames;
            float left = 0, right = 0;
            for (uint16_t i = 0; i < kernel.limit; ++i) {
                auto& v = voices[i]; if (!v.zone) continue;
                const auto& z = *v.zone;
                if (z.loop_end > z.loop_start && v.position >= z.loop_end) v.position = z.loop_start + std::fmod(v.position - z.loop_start, z.loop_end - z.loop_start);
                if (v.position >= z.frames) { cut(i); kernel.retire(i); continue; }
                if (v.releasing) {
                    const double release_frames = double(z.release_ms) * rate / 1000.0;
                    if (double(v.release_age) >= release_frames) { cut(i); kernel.retire(i); continue; }
                    v.level = v.release_level * float(1.0 - double(v.release_age++) / release_frames);
                } else {
                    const double ms = double(v.age++) * 1000.0 / rate;
                    if (ms < z.attack_ms) v.level = float(ms / z.attack_ms);
                    else if (ms < z.attack_ms + z.decay_ms) v.level = 1.f - (1.f - z.sustain) * float((ms - z.attack_ms) / z.decay_ms);
                    else v.level = z.sustain;
                }
                const auto a = uint32_t(v.position);
                auto b = a + 1;
                if (z.loop_end > z.loop_start && b == z.loop_end) b = z.loop_start;
                if (b >= z.frames) b = a;
                const float fraction = float(v.position - a);
                const auto sample = [&](uint16_t ch) {
                    const auto c = uint16_t(z.channels == 1 ? 0 : ch);
                    const float x = z.samples[a * z.channels + c], y = z.samples[b * z.channels + c];
                    return x + (y - x) * fraction;
                };
                left += sample(0) * v.level * v.left; right += sample(1) * v.level * v.right;
                v.position += v.step;
            }
            for (uint32_t c = 0; c < 2; ++c) {
                const auto value = c ? right : left;
                if (value < -1.f || value > 1.f) ++clipped;
                output[f * 2 + c] = int16_t(std::clamp(value, -1.f, 1.f) * 32767.f);
            }
        }
    }
};
}
extern "C" void* epok_sequence_create(const epok::sequence::Event* events, uint32_t count, uint16_t ppqn, uint16_t voices, const EpokSequenceZone* zones, uint32_t zone_count, uint32_t rate) {
    if (!zones || !zone_count || zone_count > 128 || rate < 8000 || rate > 192000) return nullptr;
    for (uint32_t i = 0; i < zone_count; ++i) {
        const auto& z = zones[i];
        if (!z.samples || !z.frames || z.rate < 8000 || z.rate > 192000 || z.channels < 1 || z.channels > 2 || z.loop_end > z.frames || z.loop_start > z.loop_end) return nullptr;
    }
    auto* p = new(std::nothrow) Preview;
    if (!p) return nullptr;
    if (!p->kernel.begin(events, count, ppqn, voices)) { delete p; return nullptr; }
    p->zones = zones; p->zone_count = zone_count; p->rate = rate; return p;
}
extern "C" void epok_sequence_destroy(void* p) { delete static_cast<Preview*>(p); }
extern "C" int epok_sequence_render(void* handle, int16_t* output, uint32_t frames) {
    if (!handle || !output || frames > 4096) return -1;
    auto& p = *static_cast<Preview*>(handle); p.render(output, frames);
    return int(p.kernel.error);
}
extern "C" EpokSequenceStats epok_sequence_stats(const void* handle) {
    const auto& p = *static_cast<const Preview*>(handle);
    return {uint32_t(p.kernel.error), p.kernel.peak, p.kernel.steals, p.kernel.loops, p.clipped};
}
