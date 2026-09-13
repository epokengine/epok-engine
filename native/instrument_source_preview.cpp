#include "instrument_source_preview.h"

#include "instrument_bank.hpp"
#include "instrument_synth.hpp"

#include <algorithm>
#include <cmath>
#include <limits>
#include <new>

namespace {
using epok::instrument::Envelope;
using epok::instrument::Lfo;
using epok::instrument::Modulation;
using epok::instrument::Zone;
using epok::instrument::synth::Controls;
using epok::instrument::synth::Error;
using epok::instrument::synth::State;

constexpr uint16_t kPhysicalVoices = 128;
constexpr uint16_t kLogicalVoices = epok::sequence::Kernel::MaxVoices;
constexpr int64_t kMaximumModulationAmount = 1073741824ll;

struct Biquad {
    float b0 = 1, b1 = 0, b2 = 0, a1 = 0, a2 = 0, z1 = 0, z2 = 0;
    float process(float value) {
        const float output = b0 * value + z1;
        z1 = b1 * value - a1 * output + z2;
        z2 = b2 * value - a2 * output;
        return output;
    }
    void configure(int64_t cents, int64_t centibels, uint32_t rate) {
        // SF2.04 defines a second-order resonant pole pair.  The Q conversion
        // below uses its specified peak height in centibels (0 cB is the
        // Butterworth 0.707 reference); frequencies above source Nyquist are
        // represented by the closest stable digital pole pair.
        const double hertz = 8.175798915643707 * std::pow(2.0, double(cents) / 1200.0);
        const double cutoff = std::clamp(hertz, 1.0, std::max(1.0, double(rate) * .49));
        const double q = std::pow(10.0, (double(centibels) / 10.0 - 3.01) / 20.0);
        const double omega = 2.0 * 3.14159265358979323846 * cutoff / rate;
        const double alpha = std::sin(omega) / (2.0 * std::max(q, .0001));
        const double cosine = std::cos(omega);
        const double normal = 1.0 + alpha;
        const double dc_gain = std::pow(10.0, -double(centibels) / 400.0);
        b0 = float(dc_gain * (1.0 - cosine) * .5 / normal);
        b1 = float(dc_gain * (1.0 - cosine) / normal);
        b2 = b0;
        a1 = float(-2.0 * cosine / normal);
        a2 = float((1.0 - alpha) / normal);
    }
};

struct Position {
    int64_t start = 0, end = 0, loop_start = 0, loop_end = 0;
};

struct PhysicalVoice {
    State state{};
    Controls controls{};
    Zone zone{};
    Modulation synth_modulations[32]{};
    uint16_t synth_modulation_count = 0;
    uint16_t logical = kLogicalVoices;
    uint16_t region = 0;
    uint8_t channel = 0;
    bool active = false, released = false;
    double position = 0.0;
    Position bounds{};
    Biquad filter{};
};

struct Preview {
    epok::sequence::Kernel kernel{};
    const EpokSourceRegion* regions = nullptr;
    uint32_t region_count = 0;
    const EpokSourcePcm* samples = nullptr;
    uint32_t sample_count = 0, rate = 44100;
    PhysicalVoice voices[kPhysicalVoices]{};
    uint64_t frames = 0, micros = 0, synth_micros = 0;
    uint32_t age = 0, physical_steals = 0, denied = 0, sample_loops = 0, clipped = 0;
    uint32_t physical_peak = 0, local_error = 0;
    float comparison_gain = 1;

    static Controls controls_for(const epok::sequence::Channel& channel) {
        Controls controls;
        controls.cc[1] = channel.modulation;
        controls.cc[7] = channel.volume;
        controls.cc[10] = channel.pan;
        controls.cc[11] = channel.expression;
        controls.cc[91] = channel.reverb;
        controls.cc[64] = channel.sustain ? 127 : 0;
        controls.cc[66] = channel.sostenuto ? 127 : 0;
        controls.bend = channel.bend;
        controls.bend_range_cents = channel.bend_range_cents;
        controls.fine_tuning = channel.fine_tuning;
        controls.coarse_tuning = channel.coarse_tuning;
        return controls;
    }

    static bool same_controls(const Controls& left, const Controls& right) {
        return std::equal(left.cc, left.cc + 128, right.cc) && left.bend == right.bend &&
            left.bend_range_cents == right.bend_range_cents && left.fine_tuning == right.fine_tuning &&
            left.coarse_tuning == right.coarse_tuning && left.poly_pressure == right.poly_pressure &&
            left.channel_pressure == right.channel_pressure;
    }

    void error(Error value) {
        if (value != Error::None && !local_error) local_error = 0x100u + static_cast<uint8_t>(value);
    }
    void source_error() { if (!local_error) local_error = 0x200; }

    bool has_logical(uint16_t logical) const {
        for (const auto& voice : voices) if (voice.active && voice.logical == logical) return true;
        return false;
    }
    void clear(uint16_t physical) {
        if (physical < kPhysicalVoices) voices[physical] = PhysicalVoice{};
    }
    void clear_logical(uint16_t logical, bool retire) {
        for (uint16_t i = 0; i < kPhysicalVoices; ++i)
            if (voices[i].active && voices[i].logical == logical) clear(i);
        if (retire && logical < kernel.limit && kernel.notes[logical].active) kernel.retire(logical);
    }
    void finish(uint16_t physical) {
        const uint16_t logical = voices[physical].logical;
        clear(physical);
        if (logical < kernel.limit && !has_logical(logical) && kernel.notes[logical].active) kernel.retire(logical);
    }
    void peak() {
        uint32_t active = 0;
        for (const auto& voice : voices) active += voice.active;
        physical_peak = std::max(physical_peak, active);
    }
    bool deny(uint16_t logical) {
        ++denied;
        if (logical < kernel.limit && kernel.notes[logical].active) kernel.retire(logical);
        return true;
    }

    static bool source_value(uint16_t source, uint8_t key, uint8_t velocity, const Controls& controls, int64_t& value) {
        bool valid = true;
        value = epok::instrument::synth::detail::source_q30(source, key, velocity, controls, valid);
        return valid;
    }
    static bool add_checked(int64_t& target, int64_t value) {
        if ((value > 0 && target > INT64_MAX - value) || (value < 0 && target < INT64_MIN - value)) return false;
        target += value; return true;
    }
    static bool modulation_value(const EpokSourceModulation& mod, uint8_t key, uint8_t velocity,
                                 const Controls& controls, int64_t& value) {
        int64_t first = 0, second = 0;
        if (mod.amount < -kMaximumModulationAmount || mod.amount > kMaximumModulationAmount ||
            !source_value(mod.source, key, velocity, controls, first) ||
            !source_value(mod.amount_source, key, velocity, controls, second)) return false;
        // The accepted amount range makes both products fit signed 64 bits.
        value = (mod.amount * first) / epok::instrument::synth::detail::q30;
        value = (value * second) / epok::instrument::synth::detail::q30;
        if (mod.flags & 1) value = value == INT64_MIN ? INT64_MAX : std::llabs(value);
        return true;
    }
    static bool is_filter_destination(uint16_t destination) {
        return destination == EpokSourceFilterCents || destination == EpokSourceFilterCentibels ||
            destination == EpokSourceModLfoFilter || destination == EpokSourceModEnvFilter;
    }
    static bool is_position_destination(uint16_t destination) {
        return destination >= EpokSourceStartFrames && destination <= EpokSourceLoopEndFrames;
    }
    static uint16_t synth_destination(uint16_t destination) {
        using namespace epok::instrument;
        switch (destination) {
            case EpokSourceModLfoPitch: return ModLfoPitch;
            case EpokSourceVibLfoPitch: return VibLfoPitch;
            case EpokSourceModEnvPitch: return ModEnvPitch;
            case EpokSourceModLfoVolume: return ModLfoVolume;
            case EpokSourceChorus: return Chorus;
            case EpokSourceReverb: return Reverb;
            case EpokSourcePan: return Pan;
            case EpokSourceModLfoDelay: return ModLfoDelay;
            case EpokSourceModLfoFrequency: return ModLfoFrequency;
            case EpokSourceVibLfoDelay: return VibLfoDelay;
            case EpokSourceVibLfoFrequency: return VibLfoFrequency;
            case EpokSourceModEnvDelay: return ModEnvDelay;
            case EpokSourceModEnvAttack: return ModEnvAttack;
            case EpokSourceModEnvHold: return ModEnvHold;
            case EpokSourceModEnvDecay: return ModEnvDecay;
            case EpokSourceModEnvSustain: return ModEnvSustain;
            case EpokSourceModEnvRelease: return ModEnvRelease;
            case EpokSourceModEnvHoldKey: return ModEnvHoldKey;
            case EpokSourceModEnvDecayKey: return ModEnvDecayKey;
            case EpokSourceVolEnvDelay: return VolEnvDelay;
            case EpokSourceVolEnvAttack: return VolEnvAttack;
            case EpokSourceVolEnvHold: return VolEnvHold;
            case EpokSourceVolEnvDecay: return VolEnvDecay;
            case EpokSourceVolEnvSustain: return VolEnvSustain;
            case EpokSourceVolEnvRelease: return VolEnvRelease;
            case EpokSourceVolEnvHoldKey: return VolEnvHoldKey;
            case EpokSourceVolEnvDecayKey: return VolEnvDecayKey;
            case EpokSourceAttenuation: return Attenuation;
            case EpokSourcePitch: return Pitch;
            case EpokSourceScale: return Scale;
            default: return 0;
        }
    }
    static Zone make_zone(const EpokSourceRegion& source) {
        Zone zone{};
        zone.sample = source.sample; zone.bank = source.bank; zone.program = source.program; zone.percussion = source.percussion;
        zone.key_lo = source.key_lo; zone.key_hi = source.key_hi; zone.velocity_lo = source.velocity_lo; zone.velocity_hi = source.velocity_hi;
        zone.root_key = source.root_key; zone.fixed_key = source.fixed_key; zone.fixed_velocity = source.fixed_velocity; zone.loop_mode = source.loop_mode;
        zone.exclusive_class = source.exclusive_class; zone.tune = source.tune; zone.scale = source.scale;
        zone.attenuation = source.attenuation; zone.pan = source.pan; zone.mod_env_pitch = source.mod_env_pitch; zone.reverb = source.reverb;
        zone.volume_envelope = {source.volume_envelope.delay, source.volume_envelope.attack, source.volume_envelope.hold,
            source.volume_envelope.decay, source.volume_envelope.sustain, source.volume_envelope.release,
            source.volume_envelope.hold_key, source.volume_envelope.decay_key};
        zone.modulation_envelope = {source.modulation_envelope.delay, source.modulation_envelope.attack, source.modulation_envelope.hold,
            source.modulation_envelope.decay, source.modulation_envelope.sustain, source.modulation_envelope.release,
            source.modulation_envelope.hold_key, source.modulation_envelope.decay_key};
        zone.modulation_lfo = {source.modulation_lfo.delay, source.modulation_lfo.frequency, source.modulation_lfo.pitch, source.modulation_lfo.volume};
        zone.vibrato_lfo = {source.vibrato_lfo.delay, source.vibrato_lfo.frequency, source.vibrato_lfo.pitch, source.vibrato_lfo.volume};
        return zone;
    }
    bool build_synth(PhysicalVoice& voice, const EpokSourceRegion& source, uint8_t key, uint8_t velocity, const Controls& controls) {
        voice.zone = make_zone(source); voice.synth_modulation_count = 0;
        for (uint16_t i = 0; i < source.modulation_count; ++i) {
            const auto& mod = source.modulations[i];
            // The source audition has an explicit dry policy. Preserve effect
            // destinations in the ABI/report, but do not ask the shared
            // parameter state to create a wet bus that this backend lacks.
            if (is_filter_destination(mod.destination) || is_position_destination(mod.destination) ||
                mod.destination == EpokSourceChorus || mod.destination == EpokSourceReverb) continue;
            const uint16_t destination = synth_destination(mod.destination);
            if (!destination || mod.amount < INT32_MIN || mod.amount > INT32_MAX) return false;
            auto& target = voice.synth_modulations[voice.synth_modulation_count++];
            target = {mod.source, mod.amount_source, destination, uint16_t(mod.flags & 1), int32_t(mod.amount), 0};
        }
        const auto result = voice.state.start_voice(voice.zone, voice.synth_modulations, voice.synth_modulation_count, key, velocity, controls);
        if (result != Error::None) { error(result); return false; }
        voice.state.advance(0);
        return true;
    }
    bool parameters(PhysicalVoice& voice, const epok::sequence::Note& note) {
        const auto& region = regions[voice.region];
        const auto& sample = samples[region.sample];
        int64_t start = region.start_offset, end = int64_t(sample.frames) + region.end_offset;
        int64_t loop_start = region.loop_start, loop_end = region.loop_end;
        int64_t filter_cents = region.filter_cents, filter_centibels = region.filter_centibels;
        int64_t mod_lfo_filter = region.modulation_lfo.filter, mod_env_filter = region.mod_env_filter;
        for (uint16_t i = 0; i < region.modulation_count; ++i) {
            const auto& mod = region.modulations[i];
            int64_t value = 0;
            if (!modulation_value(mod, region.fixed_key==255?note.key:region.fixed_key,
                region.fixed_velocity==255?note.velocity:region.fixed_velocity, voice.controls, value)) return false;
            switch (mod.destination) {
                case EpokSourceStartFrames: if (!add_checked(start, value)) return false; break;
                case EpokSourceEndFrames: if (!add_checked(end, value)) return false; break;
                case EpokSourceLoopStartFrames: if (!add_checked(loop_start, value)) return false; break;
                case EpokSourceLoopEndFrames: if (!add_checked(loop_end, value)) return false; break;
                case EpokSourceFilterCents: if (!add_checked(filter_cents, value)) return false; break;
                case EpokSourceFilterCentibels: if (!add_checked(filter_centibels, value)) return false; break;
                case EpokSourceModLfoFilter: if (!add_checked(mod_lfo_filter, value)) return false; break;
                case EpokSourceModEnvFilter: if (!add_checked(mod_env_filter, value)) return false; break;
                default: break;
            }
        }
        if (start < 0 || start >= end || end > sample.frames || (region.loop_mode &&
            (loop_start < start || loop_start >= loop_end || loop_end > end))) return false;
        voice.bounds = {start, end, loop_start, loop_end};
        const auto output = voice.state.output();
        filter_cents += mod_env_filter * output.modulation_envelope_q15 / 32768;
        filter_cents += mod_lfo_filter * output.modulation_lfo_q15 / 32768;
        filter_cents = std::clamp<int64_t>(filter_cents, 1500, 13500);
        filter_centibels = std::clamp<int64_t>(filter_centibels, 0, 960);
        // Filtering follows interpolation, once per output frame. Cutoff Hz
        // does not transpose with the sample's rate or the note's pitch.
        voice.filter.configure(filter_cents, filter_centibels, rate);
        return true;
    }

    bool matches(const EpokSourceRegion& region, const epok::sequence::Note& note, const epok::sequence::Channel& channel) const {
        return region.bank == channel.bank && region.program == channel.program &&
            region.percussion == uint8_t(note.channel == 9) && note.key >= region.key_lo && note.key <= region.key_hi &&
            note.velocity >= region.velocity_lo && note.velocity <= region.velocity_hi;
    }
    bool same_instrument(const EpokSourceRegion& left, const EpokSourceRegion& right) const {
        return left.bank == right.bank && left.program == right.program && left.percussion == right.percussion;
    }
    bool start(uint16_t logical, const epok::sequence::Note& note, const epok::sequence::Channel& channel) {
        uint16_t matching[128]{}; uint16_t matching_count = 0;
        for (uint16_t i = 0; i < region_count; ++i) if (matches(regions[i], note, channel)) matching[matching_count++] = i;
        if (!matching_count) return false;
        if (matching_count > kPhysicalVoices) return deny(logical);
        const auto controls = controls_for(channel);
        if (!controls.valid()) { error(Error::InvalidControls); return false; }
        // Start preflight guarantees no old group is cut before all new layers are representable.
        for (uint16_t i = 0; i < matching_count; ++i) {
            PhysicalVoice probe{}; probe.controls = controls; probe.region = matching[i];
            if (!build_synth(probe, regions[matching[i]], note.key, note.velocity, controls) || !parameters(probe, note)) { source_error(); return false; }
        }
        bool victims[kLogicalVoices]{};
        for (uint16_t incoming = 0; incoming < matching_count; ++incoming) {
            const auto& region = regions[matching[incoming]];
            if (!region.exclusive_class) continue;
            for (const auto& voice : voices) if (voice.active && voice.channel == note.channel) {
                const auto& existing = regions[voice.region];
                if (existing.exclusive_class == region.exclusive_class && same_instrument(existing, region)) victims[voice.logical] = true;
            }
        }
        uint16_t free = 0;
        for (const auto& voice : voices) free += !voice.active;
        for (uint16_t logical_old = 0; logical_old < kernel.limit; ++logical_old) if (victims[logical_old]) {
            uint16_t count = 0; for (const auto& voice : voices) count += voice.active && voice.logical == logical_old;
            free += count;
        }
        if (free < matching_count) { source_error();return false; }
        for (uint16_t old = 0; old < kernel.limit; ++old) if (victims[old]) clear_logical(old, true);
        uint16_t next = 0;
        for (uint16_t physical = 0; physical < kPhysicalVoices; ++physical) {
            if (next == matching_count) break;
            if (voices[physical].active) continue;
            auto& voice = voices[physical];
            voice.active = true; voice.logical = logical; voice.region = matching[next++]; voice.channel = note.channel; voice.controls = controls;
            if (!build_synth(voice, regions[voice.region], note.key, note.velocity, controls) || !parameters(voice, note)) {
                source_error(); clear_logical(logical, false); return false;
            }
            voice.position = double(voice.bounds.start);
        }
        ++age; peak(); return true;
    }
    void cut(uint16_t logical) { clear_logical(logical, false); }
    void release(uint16_t logical) {
        for (auto& voice : voices) if (voice.active && voice.logical == logical) { voice.released = true; voice.state.release(); }
    }
    void update(uint16_t logical, const epok::sequence::Channel& channel) {
        const auto controls = controls_for(channel);
        for (auto& voice : voices) if (voice.active && voice.logical == logical) {
            voice.controls = controls; error(voice.state.update_controls(controls)); voice.state.advance(0);
            if (!parameters(voice, kernel.notes[logical])) source_error();
        }
    }
    void advance_synth(uint64_t now) {
        while (synth_micros + 1000 <= now) {
            synth_micros += 1000;
            for (uint16_t i = 0; i < kPhysicalVoices; ++i) {
                auto& voice = voices[i]; if (!voice.active) continue;
                const auto controls = controls_for(kernel.channels[voice.channel]);
                if (!same_controls(controls, voice.controls)) { voice.controls = controls; error(voice.state.update_controls(controls)); }
                const auto output = voice.state.advance(1000); error(voice.state.error());
                if (!parameters(voice, kernel.notes[voice.logical])) { source_error(); finish(i); continue; }
                if (output.finished) finish(i);
            }
        }
    }
    bool position_valid(PhysicalVoice& voice) {
        const auto& region = regions[voice.region];
        const bool loop = region.loop_mode == 1 || (region.loop_mode == 3 && !voice.released);
        if (loop && voice.position >= voice.bounds.loop_end) {
            const double span = double(voice.bounds.loop_end - voice.bounds.loop_start);
            const double traversals = std::floor((voice.position - voice.bounds.loop_start) / span);
            voice.position = double(voice.bounds.loop_start) + std::fmod(voice.position - voice.bounds.loop_start, span);
            const uint64_t amount = traversals > 0 ? uint64_t(traversals) : 0;
            sample_loops = amount > UINT32_MAX - sample_loops ? UINT32_MAX : sample_loops + uint32_t(amount);
        }
        return voice.position >= voice.bounds.start && voice.position < voice.bounds.end;
    }
    float interpolate(PhysicalVoice& voice) {
        const auto& region = regions[voice.region]; const auto& sample = samples[region.sample];
        if (!position_valid(voice)) return 0;
        const uint32_t first = uint32_t(voice.position);
        uint32_t second = first + 1;
        const bool loop = region.loop_mode == 1 || (region.loop_mode == 3 && !voice.released);
        if (loop && second >= voice.bounds.loop_end) second = uint32_t(voice.bounds.loop_start);
        if (second >= voice.bounds.end) second = first;
        const float fraction = float(voice.position - first);
        return voice.filter.process(sample.framesdata[first] + (sample.framesdata[second] - sample.framesdata[first]) * fraction);
    }
    void render(int16_t* output, uint32_t count) {
        for (uint32_t frame = 0; frame < count; ++frame) {
            const uint64_t now = frames * 1000000ull / rate;
            kernel.advance(uint32_t(now - micros), *this); micros = now; advance_synth(now); ++frames;
            if(kernel.steals){source_error();return;}
            float left = 0, right = 0;
            for (uint16_t i = 0; i < kPhysicalVoices; ++i) {
                auto& voice = voices[i]; if (!voice.active) continue;
                const auto control = voice.state.output();
                if (control.finished || !position_valid(voice)) { finish(i); continue; }
                const float value = interpolate(voice); const float gain = float(control.gain_q15) / 32767.0f;
                const float pan = float(std::clamp<int>(control.pan_permille, -500, 500) + 500) / 1000.0f;
                left += value * gain * (1 - pan); right += value * gain * pan;
                const auto& sample = samples[regions[voice.region].sample];
                const double pitch = std::pow(2.0, double(control.pitch_cents_x100) / 120000.0);
                voice.position += pitch * double(sample.rate) / rate;
            }
            for (uint32_t channel = 0; channel < 2; ++channel) {
                const float value = (channel ? right : left) * comparison_gain;
                if(!std::isfinite(value)){source_error();return;}
                if (value < -1 || value > 1) ++clipped;
                output[frame * 2 + channel] = int16_t(std::clamp(value, -1.0f, 1.0f) * 32767.0f);
            }
        }
    }
};

bool valid_region(const EpokSourceRegion& region, const EpokSourcePcm* samples, uint32_t sample_count) {
    if (region.sample >= sample_count || region.bank > 16383 || region.program > 127 || region.percussion > 1 ||
        region.key_lo > region.key_hi || region.key_hi > 127 || region.velocity_lo > region.velocity_hi || region.velocity_hi > 127 ||
        region.root_key > 127 || (region.fixed_key != 255 && region.fixed_key > 127) ||
        (region.fixed_velocity != 255 && region.fixed_velocity > 127) || (region.loop_mode != 0 && region.loop_mode != 1 && region.loop_mode != 3) ||
        region.exclusive_class > 127 || region.modulation_count > 32 || (!region.modulations && region.modulation_count)) return false;
    const int64_t frames = samples[region.sample].frames;
    if (region.start_offset < 0 || region.end_offset < -frames || region.end_offset > INT64_MAX - frames) return false;
    const int64_t end = frames + region.end_offset;
    if (region.start_offset >= end || (region.loop_mode &&
        (region.loop_start < region.start_offset || region.loop_start >= region.loop_end || region.loop_end > end))) return false;
    for (uint16_t i = 0; i < region.modulation_count; ++i) {
        const auto& mod = region.modulations[i]; bool valid = true;
        epok::instrument::synth::detail::source_q30(mod.source, 0, 0, Controls{}, valid);
        epok::instrument::synth::detail::source_q30(mod.amount_source, 0, 0, Controls{}, valid);
        if (!valid || mod.destination < EpokSourceStartFrames || mod.destination > EpokSourceScale || mod.flags & ~1 ||
            mod.amount < -kMaximumModulationAmount || mod.amount > kMaximumModulationAmount) return false;
    }
    return true;
}
bool valid_pcm(const EpokSourcePcm* samples, uint32_t count) {
    if (!samples || !count || count > 128) return false;
    for (uint32_t i = 0; i < count; ++i) {
        if (!samples[i].framesdata || !samples[i].frames || samples[i].rate < 400 || samples[i].rate > 192000) return false;
        for (uint32_t frame = 0; frame < samples[i].frames; ++frame) if (!std::isfinite(samples[i].framesdata[frame])) return false;
    }
    return true;
}
} // namespace

extern "C" void* epok_source_instrument_create(const epok::sequence::Event* events, uint32_t count, uint16_t ppqn,
                                                   const EpokSourceRegion* regions, uint32_t region_count,
                                                   const EpokSourcePcm* samples, uint32_t sample_count, uint32_t rate) {
    if (!events || !regions || !count || !region_count || region_count > 128 || !rate || rate < 8000 || rate > 192000 || !valid_pcm(samples, sample_count)) return nullptr;
    for (uint32_t i = 0; i < region_count; ++i) if (!valid_region(regions[i], samples, sample_count)) return nullptr;
    auto* preview = new(std::nothrow) Preview;
    if (!preview) return nullptr;
    if (!preview->kernel.begin(events, count, ppqn, kLogicalVoices)) { delete preview; return nullptr; }
    preview->regions = regions; preview->region_count = region_count; preview->samples = samples; preview->sample_count = sample_count; preview->rate = rate;
    return preview;
}
extern "C" void epok_source_instrument_destroy(void* handle) { delete static_cast<Preview*>(handle); }
extern "C" bool epok_source_instrument_set_gain(void* handle,float gain) {
    if(!handle || !std::isfinite(gain) || gain<0 || gain>1)return false;
    static_cast<Preview*>(handle)->comparison_gain=gain;return true;
}
extern "C" int epok_source_instrument_render(void* handle, int16_t* output, uint32_t frames) {
    if (!handle || !output || frames > 4096) return -1;
    auto& preview = *static_cast<Preview*>(handle); preview.render(output, frames);
    return preview.local_error ? int(preview.local_error) : int(preview.kernel.error);
}
extern "C" EpokSourcePreviewStats epok_source_instrument_stats(const void* handle) {
    if (!handle) return {};
    const auto& preview = *static_cast<const Preview*>(handle);
    const uint32_t error = preview.local_error ? preview.local_error : uint32_t(preview.kernel.error);
    return {error, preview.kernel.peak, preview.physical_peak, preview.kernel.steals + preview.physical_steals,
        preview.denied, preview.kernel.loops, preview.clipped, preview.sample_loops};
}
