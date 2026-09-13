#include "instrument_preview.h"

#include "instrument_allocator.hpp"
#include "instrument_bank.hpp"
#include "instrument_synth.hpp"
#include "instrument_preparation.hpp"

#include <algorithm>
#include <cmath>
#include <limits>
#include <new>

namespace {

constexpr uint16_t kPhysicalVoices = 24;
constexpr uint16_t kLogicalVoices = epok::sequence::Kernel::MaxVoices;

struct PhysicalVoice {
    epok::instrument::synth::State state{};
    epok::instrument::synth::Controls controls{};
    uint16_t logical = kLogicalVoices;
    uint16_t zone = 0;
    uint8_t channel = 0;
    double position = 0.0;
    bool active = false;
    bool released = false;
};

struct Preview {
    epok::sequence::Kernel kernel{};
    epok::instrument::BankView bank{};
    const EpokInstrumentPcm* samples = nullptr;
    uint32_t sample_count = 0;
    uint32_t rate = 44100;
    uint64_t frames = 0;
    uint64_t micros = 0;
    uint64_t synth_micros = 0;
    uint32_t age = 0;
    uint32_t physical_steals = 0;
    uint32_t denied = 0;
    uint32_t sample_loops = 0;
    uint32_t clipped = 0;
    uint32_t local_error = 0;
    uint32_t physical_peak = 0;
    epok::instrument::allocation::Slot slots[kPhysicalVoices]{};
    PhysicalVoice voices[kPhysicalVoices]{};

    static epok::instrument::synth::Controls controls_for(const epok::sequence::Channel& channel) {
        epok::instrument::synth::Controls controls;
        controls.cc[1] = channel.modulation;
        controls.cc[7] = channel.volume;
        controls.cc[10] = channel.pan;
        controls.cc[11] = channel.expression;
        controls.cc[64] = channel.sustain ? 127 : 0;
        controls.cc[66] = channel.sostenuto ? 127 : 0;
        controls.cc[91] = channel.reverb;
        controls.bend = channel.bend;
        controls.bend_range_cents = channel.bend_range_cents;
        controls.fine_tuning = channel.fine_tuning;
        controls.coarse_tuning = channel.coarse_tuning;
        return controls;
    }

    static bool same_controls(const epok::instrument::synth::Controls& left,
                              const epok::instrument::synth::Controls& right) {
        return std::equal(left.cc, left.cc + 128, right.cc) && left.bend == right.bend &&
            left.bend_range_cents == right.bend_range_cents && left.fine_tuning == right.fine_tuning &&
            left.coarse_tuning == right.coarse_tuning && left.poly_pressure == right.poly_pressure &&
            left.channel_pressure == right.channel_pressure;
    }

    void set_error(epok::instrument::synth::Error error) {
        if (error != epok::instrument::synth::Error::None && !local_error) {
            local_error = 0x100u + static_cast<uint8_t>(error);
        }
    }

    // Capacity/priority denial still consumes this note-on: retire it now so it
    // cannot occupy a logical slot until its later note-off arrives.
    bool deny(uint16_t logical) {
        ++denied;
        if (logical < kernel.limit && kernel.notes[logical].active) kernel.retire(logical);
        return true;
    }

    bool has_logical(uint16_t logical) const {
        for (const auto& voice : voices) if (voice.active && voice.logical == logical) return true;
        return false;
    }

    void clear_physical(uint16_t physical) {
        if (physical >= kPhysicalVoices) return;
        voices[physical].state.reset();
        voices[physical] = PhysicalVoice{};
        slots[physical] = epok::instrument::allocation::Slot{};
    }

    void clear_logical(uint16_t logical, bool retire) {
        if (logical >= kernel.limit) return;
        for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
            if (voices[physical].active && voices[physical].logical == logical) clear_physical(physical);
        }
        if (retire && kernel.notes[logical].active) kernel.retire(logical);
    }

    void clear_if_finished(uint16_t physical) {
        const uint16_t logical = voices[physical].logical;
        clear_physical(physical);
        if (logical < kernel.limit && !has_logical(logical) && kernel.notes[logical].active) kernel.retire(logical);
    }

    void refresh_peak() {
        uint32_t active = 0;
        for (const auto& voice : voices) active += voice.active;
        physical_peak = std::max(physical_peak, active);
    }

    bool matches(const epok::instrument::Zone& zone, const epok::sequence::Note& note,
                 const epok::sequence::Channel& channel) const {
        return zone.bank == channel.bank && zone.program == channel.program &&
            zone.percussion == uint8_t(note.channel == 9) && note.key >= zone.key_lo && note.key <= zone.key_hi &&
            note.velocity >= zone.velocity_lo && note.velocity <= zone.velocity_hi;
    }

    bool same_instrument(const epok::instrument::Zone& left, const epok::instrument::Zone& right) const {
        return left.bank == right.bank && left.program == right.program && left.percussion == right.percussion;
    }

    void mark_exclusive_victims(const uint16_t* matching, uint16_t matching_count, uint8_t channel,
                                epok::instrument::allocation::Slot (&staged)[kPhysicalVoices],
                                bool (&victim)[kLogicalVoices]) const {
        for (uint16_t i = 0; i < matching_count; ++i) {
            const auto& incoming = bank.zone(matching[i]);
            if (!incoming.exclusive_class) continue;
            for (const auto& voice : voices) {
                if (!voice.active || voice.channel != channel) continue;
                const auto& existing = bank.zone(voice.zone);
                if (existing.exclusive_class == incoming.exclusive_class && same_instrument(existing, incoming)) {
                    victim[voice.logical] = true;
                }
            }
        }
        for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
            if (slots[physical].occupied && slots[physical].musical && victim[voices[physical].logical]) {
                staged[physical] = epok::instrument::allocation::Slot{};
            }
        }
    }

    void apply_victims(const bool (&victim)[kLogicalVoices], bool count_steal) {
        for (uint16_t logical = 0; logical < kernel.limit; ++logical) {
            if (!victim[logical]) continue;
            clear_logical(logical, true);
            if (count_steal) ++physical_steals;
        }
    }

    void apply_evictions(uint32_t mask) {
        bool victim[kLogicalVoices]{};
        for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
            if ((mask & (uint32_t(1) << physical)) && voices[physical].active) victim[voices[physical].logical] = true;
        }
        apply_victims(victim, true);
    }

    void cut(uint16_t logical) { clear_logical(logical, false); }

    void release(uint16_t logical) {
        for (auto& voice : voices) {
            if (voice.active && voice.logical == logical) {
                voice.released = true;
                voice.state.release();
            }
        }
    }

    void update(uint16_t logical, const epok::sequence::Channel& channel) {
        const auto controls = controls_for(channel);
        for (auto& voice : voices) {
            if (!voice.active || voice.logical != logical) continue;
            voice.controls = controls;
            set_error(voice.state.update_controls(controls));
            voice.state.advance(0);
        }
    }

    bool start(uint16_t logical, const epok::sequence::Note& note, const epok::sequence::Channel& channel) {
        uint16_t matching[128]{};
        uint16_t matching_count = 0;
        for (uint16_t zone = 0; zone < bank.zone_count(); ++zone) {
            if (matches(bank.zone(zone), note, channel)) matching[matching_count++] = zone;
        }
        if (!matching_count) {
            return false;
        }
        const auto controls = controls_for(channel);
        if (!controls.valid()) {
            set_error(epok::instrument::synth::Error::InvalidControls);
            return false;
        }
        if (matching_count > kPhysicalVoices) {
            return deny(logical);
        }

        epok::instrument::allocation::Slot staged[kPhysicalVoices];
        for (uint16_t i = 0; i < kPhysicalVoices; ++i) staged[i] = slots[i];
        bool exclusive_victim[kLogicalVoices]{};
        mark_exclusive_victims(matching, matching_count, note.channel, staged, exclusive_victim);
        epok::instrument::allocation::Request bounded{};
        bounded.owner = 0;
        bounded.priority = 128;
        bounded.layers = static_cast<uint8_t>(matching_count);
        // Logical slots distinguish overlapping same-key notes; generation is
        // the instance lifetime so its physical ceiling counts every note.
        bounded.generation = 1;
        bounded.instance_limit = physical_limit;
        bounded.aggregate_limit = physical_limit;
        const auto plan = epok::instrument::allocation::reserve(staged, bounded);
        if (!plan.fits) {
            return deny(logical);
        }
        // Starting every layer is preflighted before a single old group is cut.
        for (uint16_t i = 0; i < matching_count; ++i) {
            epok::instrument::synth::State probe;
            const auto error = probe.start(bank, matching[i], note.key, note.velocity, controls);
            if (error != epok::instrument::synth::Error::None) {
                set_error(error);
                return false;
            }
        }
        apply_victims(exclusive_victim, false);
        apply_evictions(plan.evict_mask);

        uint16_t match_index = 0;
        for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
            if (!(plan.start_mask & (uint32_t(1) << physical))) continue;
            auto& voice = voices[physical];
            voice.active = true;
            voice.logical = logical;
            voice.zone = matching[match_index++];
            voice.channel = note.channel;
            voice.controls = controls;
            const auto error = voice.state.start(bank, voice.zone, note.key, note.velocity, controls);
            if (error != epok::instrument::synth::Error::None) {
                // The same call was already preflighted, so this is only a defensive failure path.
                set_error(error);
                clear_logical(logical, false);
                return false;
            }
            slots[physical] = {true, true, 0, static_cast<uint8_t>(logical), 128,
                               1, ++age};
        }
        refresh_peak();
        return true;
    }

    uint8_t physical_limit = kPhysicalVoices;

    void advance_synth(uint64_t now) {
        while (synth_micros + 1000 <= now) {
            synth_micros += 1000;
            for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
                auto& voice = voices[physical];
                if (!voice.active) continue;
                const auto controls = controls_for(kernel.channels[voice.channel]);
                if (!same_controls(controls, voice.controls)) {
                    voice.controls = controls;
                    set_error(voice.state.update_controls(controls));
                }
                const auto output = voice.state.advance(1000);
                set_error(voice.state.error());
                if (output.finished) clear_if_finished(physical);
            }
        }
    }

    bool position_valid(PhysicalVoice& voice, const epok::instrument::Sample& sample,
                        const epok::instrument::Zone& zone) {
        const bool loop = sample.loop_end && (zone.loop_mode == 1 || (zone.loop_mode == 3 && !voice.released));
        if (loop && voice.position >= sample.loop_end) {
            const double span = double(sample.loop_end - sample.loop_start);
            const double traversals = std::floor((voice.position - sample.loop_start) / span);
            voice.position = double(sample.loop_start) + std::fmod(voice.position - sample.loop_start, span);
            constexpr uint32_t maximum = std::numeric_limits<uint32_t>::max();
            const auto added = traversals >= double(maximum - sample_loops)
                ? maximum - sample_loops : static_cast<uint32_t>(traversals);
            sample_loops += added;
        }
        return voice.position < sample.frames;
    }

    float interpolate(PhysicalVoice& voice, const epok::instrument::Sample& sample,
                      const epok::instrument::Zone& zone) {
        if (!position_valid(voice, sample, zone)) return 0.0f;
        const uint32_t first = static_cast<uint32_t>(voice.position);
        uint32_t second = first + 1;
        const bool loop = sample.loop_end && (zone.loop_mode == 1 || (zone.loop_mode == 3 && !voice.released));
        if (loop && second >= sample.loop_end) second = sample.loop_start;
        if (second >= sample.frames) second = first;
        const float fraction = static_cast<float>(voice.position - first);
        const float a = samples[zone.sample].framesdata[first];
        const float b = samples[zone.sample].framesdata[second];
        return a + (b - a) * fraction;
    }

    void render(int16_t* output, uint32_t count) {
        for (uint32_t frame = 0; frame < count; ++frame) {
            const uint64_t now = frames * 1000000ull / rate;
            kernel.advance(static_cast<uint32_t>(now - micros), *this);
            micros = now;
            advance_synth(now);
            ++frames;

            float left = 0.0f, right = 0.0f;
            for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
                auto& voice = voices[physical];
                if (!voice.active) continue;
                const auto& zone = bank.zone(voice.zone);
                const auto& sample = bank.sample(zone.sample);
                const auto control = voice.state.output();
                if (control.finished || !position_valid(voice, sample, zone)) {
                    clear_if_finished(physical);
                    continue;
                }
                // Host preview interpolation is deliberately linear, not PSX SPU interpolation.
                const float value = interpolate(voice, sample, zone);
                const float gain = float(control.gain_q15) / 32767.0f;
                const float pan = float(std::clamp<int>(control.pan_permille, -500, 500) + 500) / 1000.0f;
                left += value * gain * (1.0f - pan);
                right += value * gain * pan;
                const double ratio = std::pow(2.0, double(control.pitch_cents_x100) / 120000.0);
                voice.position += ratio * double(sample.rate) / rate;
            }
            for (uint32_t channel = 0; channel < 2; ++channel) {
                const float value = channel ? right : left;
                if (value < -1.0f || value > 1.0f) ++clipped;
                output[frame * 2 + channel] = static_cast<int16_t>(std::clamp(value, -1.0f, 1.0f) * 32767.0f);
            }
        }
    }
};

bool valid_pcm(const epok::instrument::BankView& bank, const EpokInstrumentPcm* samples, uint32_t count) {
    if (!samples || count != bank.sample_count()) return false;
    for (uint16_t i = 0; i < count; ++i) {
        if (!samples[i].framesdata || samples[i].frames != bank.sample(i).frames) return false;
        for (uint32_t frame = 0; frame < samples[i].frames; ++frame) if (!std::isfinite(samples[i].framesdata[frame])) return false;
    }
    return true;
}

} // namespace

extern "C" void* epok_instrument_create(const epok::sequence::Event* events, uint32_t count, uint16_t ppqn,
                                          uint16_t voice_limit, const uint8_t* bank_bytes, uint32_t bank_len,
                                          const EpokInstrumentPcm* samples, uint32_t sample_count, uint32_t rate) {
    if (!bank_bytes || !rate || rate < 8000 || rate > 192000 || !voice_limit || voice_limit > kPhysicalVoices) return nullptr;
    const epok::instrument::BankView bank{bank_bytes, bank_len};
    if (!bank.valid() || !valid_pcm(bank, samples, sample_count)) return nullptr;
    auto* preview = new(std::nothrow) Preview;
    if (!preview) return nullptr;
    if (!preview->kernel.begin(events, count, ppqn, kLogicalVoices)) { delete preview; return nullptr; }
    preview->bank = bank;
    preview->samples = samples;
    preview->sample_count = sample_count;
    preview->rate = rate;
    preview->physical_limit = static_cast<uint8_t>(voice_limit);
    return preview;
}

extern "C" void epok_instrument_destroy(void* handle) { delete static_cast<Preview*>(handle); }

extern "C" int epok_instrument_render(void* handle, int16_t* stereo_output, uint32_t frames) {
    if (!handle || !stereo_output || frames > 4096) return -1;
    auto& preview = *static_cast<Preview*>(handle);
    preview.render(stereo_output, frames);
    return preview.local_error ? static_cast<int>(preview.local_error) : static_cast<int>(preview.kernel.error);
}

extern "C" EpokInstrumentStats epok_instrument_stats(const void* handle) {
    if (!handle) return {};
    const auto& preview = *static_cast<const Preview*>(handle);
    const uint32_t error = preview.local_error ? preview.local_error : static_cast<uint32_t>(preview.kernel.error);
    return {error, preview.kernel.peak, preview.physical_peak, preview.kernel.steals + preview.physical_steals,
            preview.denied, preview.kernel.loops, preview.clipped, preview.sample_loops};
}

extern "C" int epok_instrument_prepared_count(const epok::sequence::Event* events,uint32_t count,uint16_t ppqn,
                                               const uint8_t* bytes,uint32_t size,uint32_t* references){
    using namespace epok::instrument::preparation;
    Key keys[MaxStarts]{};uint16_t buckets[bucket_count(MaxStarts)]{};
    Cache cache{keys,nullptr,buckets,MaxStarts,bucket_count(MaxStarts)};
    if(!references || !cache.prepare({bytes,size},events,count,ppqn))return -1;
    *references=cache.reference_count;return cache.count;
}
