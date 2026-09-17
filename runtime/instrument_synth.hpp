#pragma once
// Bounded integer SoundFont voice control for EPSB v2. The sample cursor and
// mixer stay in the host/console backends; this state produces their parameters.
// Modulation is Q30 at its sources and Q16 in destination units. Envelope time
// is rounded to the nearest microsecond and accumulated in 64 bits; pitch is
// rounded to 0.01 cent, gain to Q15 and pan toward zero to one permille.
#include "instrument_bank.hpp"
#include <cstdint>
#include <cstddef>

namespace epok::instrument::synth {

struct Controls {
    uint8_t cc[128];
    uint16_t bend = 8192;
    uint16_t bend_range_cents = 200;
    uint16_t fine_tuning = 8192;
    uint8_t coarse_tuning = 64;
    uint8_t poly_pressure = 0;
    uint8_t channel_pressure = 0;
    uint32_t reserved=0; // Natural word alignment for bounded PSX copies.
    Controls() {
#ifdef __mips__
        for(unsigned i=0;i<128;i+=4){const uint32_t zero=0;__builtin_memcpy(cc+i,&zero,4);}
#else
        for(auto& value:cc)value=0;
#endif
        cc[7] = 100; cc[10] = 64; cc[11] = 127;
    }
    bool valid() const;
};

enum class Error : uint8_t {
    None,
    InvalidBank,
    InvalidZone,
    InvalidNote,
    InvalidControls,
    UnsupportedDestination,
    ArithmeticOverflow,
};

struct Output {
    // One unit is 0.01 cent. This includes root/scale/tune, SoundFont
    // modulation, modulation envelope/LFOs and MIDI channel tuning.
    int32_t pitch_cents_x100 = 0;
    uint16_t gain_q15 = 0;
    int16_t pan_permille = 0;
    bool finished = true;
    // Neutral control signals for host-only dynamic filter preview. The neutral value is
    // 32768; LFOs are signed and modulation envelope is unipolar.
    uint16_t modulation_envelope_q15 = 0;
    int32_t modulation_lfo_q15 = 0;
    int32_t vibrato_lfo_q15 = 0;
    uint16_t reverb_send_permille = 0;
};

class State {
public:
    State() = default;
    // `bank.data` must outlive this state. The loader validates BankView once;
    // start performs bounded local table/span checks and never scans sample bytes.
    Error start(const BankView& bank, uint16_t zone_index, uint8_t key,
                uint8_t velocity, const Controls& controls);
    // PSX loader has validated the immutable bank and the kernel has validated
    // controls/note data. Caller must select an in-range, matching zone first.
    // Source-only filter control signals may be omitted when the cooked voice
    // proves they can never contribute to target pitch or gain.
    Error start_validated(const BankView& bank, uint16_t zone_index, uint8_t key,
                          uint8_t velocity, const Controls& controls);
    // Host source-preview adapters may construct the same neutral wire structs
    // without fabricating an EPSB container. Both referenced spans must outlive State.
    Error start_voice(const Zone& zone, const Modulation* modulations,
                      uint16_t modulation_count, uint8_t key, uint8_t velocity,
                      const Controls& controls);
    void release();
    Error update_controls(const Controls& controls);
    const Output& advance(uint32_t microseconds);
    const Output& output() const { return output_; }
    bool active() const { return started_ && !output_.finished; }
    Error error() const { return error_; }
    void reset();
    // The immutable prepared source must outlive this voice, like its bank.
    void copy_initial(const State& source);
    // Editor compiler access: SPU handles the volume envelope; bake only the
    // remaining modulation into sparse register commands before shipping.
    struct HardwareParameters {uint32_t delay,attack,hold,decay,release;uint16_t sustain,gain;};
    HardwareParameters hardware_parameters() const;
    Output hardware_output();

private:
    enum class Phase : uint8_t { Delay, Attack, Hold, Decay, Sustain, Release, Done };
    struct EnvelopeState {
        uint64_t elapsed = 0;
        uint64_t release_duration = 0;
        uint64_t cached_duration = UINT64_MAX;
        uint64_t progress_reciprocal_q48 = 0;
        uint32_t release_attenuation_q16 = 0;
        uint16_t level = 0;
        uint16_t release_level = 0;
        Phase phase = Phase::Done;
    };
    struct EnvelopeParameters {
        // Clamped SF2 times reach at most 146.3 million microseconds, including
        // the 1440-centibel decay scale. Accumulated clocks remain 64-bit.
        uint32_t delay = 0, attack = 0, hold = 0, decay = 0, release = 0;
        uint32_t sustain_q16 = 0;
        bool volume = false;
    };
    // Fractional phase is retained so advance chunking cannot detune an LFO.
    struct LfoState { uint64_t phase_q48 = 0, updated_us = 0; };

    const Zone* zone_ = nullptr;
    const Modulation* modulations_ = nullptr;
    uint16_t modulation_count_ = 0;
    uint8_t key_ = 0, velocity_ = 0, actual_key_ = 0, actual_velocity_ = 0;
    Controls controls_{};
    int64_t modulation_q16_[30]{};
    EnvelopeParameters volume_parameters_{}, modulation_parameters_{};
    EnvelopeState volume_envelope_{}, modulation_envelope_{};
    LfoState modulation_lfo_{}, vibrato_lfo_{};
    uint64_t age_microseconds_ = 0;
    uint64_t modulation_lfo_delay_ = 0, vibrato_lfo_delay_ = 0;
    uint64_t modulation_lfo_rate_q48_ = 0, vibrato_lfo_rate_q48_ = 0;
    int64_t base_pitch_q16_ = 0, attenuation_q16_ = 0, pan_q16_ = 0;
    int64_t mod_env_pitch_q16_ = 0, mod_lfo_pitch_q16_ = 0;
    int64_t vib_lfo_pitch_q16_ = 0, mod_lfo_volume_q16_ = 0;
    uint16_t static_gain_q15_ = 32768;
    Output output_{};
    Error error_ = Error::None;
    bool started_ = false;
    bool released_ = false;
    uint8_t advance_mask_=0x87; // Three target contributors plus host control signals.
    const State* initial_parameters_=nullptr;

    Error rebuild_parameters(bool force,uint32_t dirty=UINT32_MAX);
    Error begin_voice(const Zone& zone,const Modulation* modulations,uint16_t count,
                      uint8_t key,uint8_t velocity,const Controls& controls,bool target_only=false);
    void refresh_output();
    void synchronize_lfos(bool force);
    static uint32_t envelope_progress(EnvelopeState& state,uint64_t duration);
};

namespace detail {
inline constexpr int64_t q30 = int64_t(1) << 30;
inline constexpr int64_t q16 = int64_t(1) << 16;
template<size_t Bytes> inline void copy_block(void* destination,const void* source){
    auto* to=static_cast<unsigned char*>(destination);
    const auto* from=static_cast<const unsigned char*>(source);
#if defined(__mips__)
    static_assert(Bytes%4==0);
    to=static_cast<unsigned char*>(__builtin_assume_aligned(to,4));
    from=static_cast<const unsigned char*>(__builtin_assume_aligned(from,4));
    size_t offset=0;
    for(;offset+16<=Bytes;offset+=16)__builtin_memcpy(to+offset,from+offset,16);
    for(;offset<Bytes;offset+=4)__builtin_memcpy(to+offset,from+offset,4);
#else
    for(size_t offset=0;offset<Bytes;++offset)to[offset]=from[offset];
#endif
}
template<class T> inline void copy_value(T& destination,const T& source){
    static_assert(__is_trivially_copyable(T));
#if defined(__mips__)
    // Nugget's freestanding memcpy is bytewise. Copy this aligned, fixed-size
    // object representation with explicit constant-size builtins; these lower
    // to word loads/stores without violating C++ aliasing rules.
    if constexpr(alignof(T)>=4 && sizeof(T)%4==0){
    auto* to=static_cast<unsigned char*>(__builtin_assume_aligned(&destination,4));
    const auto* from=static_cast<const unsigned char*>(__builtin_assume_aligned(&source,4));
    for(size_t offset=0;offset<sizeof(T);offset+=4)__builtin_memcpy(to+offset,from+offset,4);
    }else destination=source;
#else
    destination=source;
#endif
}

inline int64_t clamp(int64_t value, int64_t low, int64_t high) {
    return value < low ? low : value > high ? high : value;
}

// SF2 2.04 section 8.2.4's 96 dB concave curve sampled at every native
// seven-bit input. Wider sources interpolate this immutable table.
inline constexpr uint32_t concave_q30[128] = {
    0,1535977,3084193,4644844,6218133,7804264,9403451,11015909,
    12641860,14281533,15935160,17602982,19285245,20982199,22694105,24421228,
    26163841,27922225,29696667,31487464,33294919,35119346,36961067,38820412,
    40697722,42593347,44507649,46440999,48393779,50366386,52359225,54372715,
    56407289,58463393,60541489,62642050,64765569,66912553,69083526,71279031,
    73499627,75745895,78018437,80317873,82644847,85000028,87384108,89797805,
    92241863,94717056,97224189,99764095,102337643,104945737,107589316,110269359,
    112986887,115742962,118538695,121375243,124253815,127175675,130142146,133154610,
    136214517,139323384,142482805,145694449,148960073,152281523,155660739,159099768,
    162600765,166166004,169797886,173498952,177271887,181119539,185044926,189051255,
    193141934,197320591,201591094,205957571,210424436,214996413,219678569,224476345,
    229395596,234442635,239624277,244947900,250421503,256053783,261854215,267833151,
    274001925,280372989,286960063,293778309,300844544,308177492,315798079,323729795,
    331999131,340636111,349674953,359154885,369121175,379626440,390732323,402511691,
    415051530,428456849,442856032,458408335,475314651,493833425,514304980,537190231,
    563135389,593086876,628511966,671868771,727765416,806547312,941225852,1073741824
};

inline int64_t concave(int64_t normalized) {
    if (normalized <= 0) return 0;
    if (normalized >= q30) return q30;
    const uint64_t scaled = uint64_t(normalized) * 127;
    const uint32_t index = uint32_t(scaled >> 30);
    const uint32_t fraction = uint32_t(scaled & (q30 - 1));
    const int64_t first = concave_q30[index], second = concave_q30[index + 1];
    return first + ((second - first) * fraction >> 30);
}
inline int64_t convex(int64_t normalized) {
    if (normalized <= 0) return 0;
    if (normalized >= q30) return q30;
    return q30 - concave(q30 - normalized);
}

// Tenth-order integer exp2 on [0,1), followed by an integral power-of-two.
// Across the admitted SF2 ranges its error is below one 0.01-cent output unit.
inline constexpr uint64_t pow2_cents_q30(int64_t cents_q16) {
    constexpr int64_t period = 1200 * q16;
    int64_t whole = cents_q16 / period;
    int64_t remainder = cents_q16 % period;
    if (remainder < 0) { remainder += period; --whole; }
    const int64_t x = remainder * q30 / period;
    constexpr int64_t coefficients[11] = {
        1073741824,744261118,257941248,59597083,
        10327387,1431680,165394,16377,1419,109,8
    };
    int64_t value = coefficients[10];
    for (int i = 9; i >= 0; --i) value = coefficients[i] + ((value * x) >> 30);
    if (whole >= 0) {
        if (whole > 20) return UINT64_MAX;
        return uint64_t(value) << whole;
    }
    return whole < -62 ? 0 : uint64_t(value) >> -whole;
}

// One-cent knots, generated at compile time. Linear interpolation bounds pitch
// error below 0.0001 cent and removes polynomial evaluation from the R3000 path.
struct OctaveTable {
    uint32_t values[1201]{};
    constexpr OctaveTable() {
        for(uint32_t cents=0;cents<=1200;++cents)
            values[cents]=uint32_t(pow2_cents_q30(int64_t(cents)*q16));
    }
};
inline constexpr OctaveTable octave_table{};
inline uint64_t fast_pow2_cents_q30(int64_t cents_q16) {
    // Every admitted time/LFO parameter fits Q16 in i32. Keep the wider fallback
    // for this general helper, without narrowing unsupported external values.
    if(cents_q16<INT32_MIN || cents_q16>INT32_MAX)return pow2_cents_q30(cents_q16);
    constexpr int32_t period=1200*65536;
    const int32_t cents=int32_t(cents_q16);
    int32_t whole=cents/period,remainder=cents%period;
    if(remainder<0){remainder+=period;--whole;}
    const uint32_t index=uint32_t(remainder)>>16,fraction=uint32_t(remainder)&65535;
    const uint32_t first=octave_table.values[index],delta=octave_table.values[index+1]-first;
    const uint64_t value=first+((uint64_t(delta)*fraction)>>16);
    return whole>=0?value<<unsigned(whole):value>>unsigned(-whole);
}

inline constexpr uint16_t gain_at_whole_centibel(uint32_t centibels) {
    // 10^(-cB/200) == 2^(-cB * 19.931568569... / 1200).
    const int64_t cents_q16 = int64_t(centibels) * q16;
    const int64_t equivalent_cents = -(cents_q16 * 19931569ll / 1000000ll);
    const uint64_t ratio = pow2_cents_q30(equivalent_cents);
    const uint64_t value = (ratio * 32768ull + (1ull << 29)) >> 30;
    return uint16_t(value > 32768 ? 32768 : value);
}

struct AttenuationTable {
    uint16_t values[1001]{};
    constexpr AttenuationTable() {
        for (uint32_t centibels = 0; centibels <= 1000; ++centibels)
            values[centibels] = gain_at_whole_centibel(centibels);
    }
};
inline constexpr AttenuationTable attenuation_table{};

inline uint64_t time_microseconds(int64_t timecents_q16, int32_t maximum,
                                  bool zero_sentinel) {
    if (zero_sentinel && timecents_q16 <= int64_t(-32768) * q16) return 0;
    timecents_q16 = clamp(timecents_q16, int64_t(-12000) * q16,
                         int64_t(maximum) * q16);
    return (fast_pow2_cents_q30(timecents_q16) * 1000000ull + (1ull << 29)) >> 30;
}

inline uint16_t attenuation_gain_q15(int64_t centibels_q16) {
    centibels_q16 = clamp(centibels_q16, 0, 1440 * q16);
    if (centibels_q16 >= 1000 * q16) return 0;
    const uint32_t whole = uint32_t(centibels_q16 / q16);
    const uint32_t fraction = uint32_t(centibels_q16) & uint32_t(q16 - 1);
    const uint32_t upper = attenuation_table.values[whole];
    const uint32_t drop = upper - attenuation_table.values[whole + 1];
    return uint16_t(upper - ((drop * fraction + uint32_t(q16 / 2)) >> 16));
}

inline uint64_t lfo_rate_q48(int64_t frequency_cents_q16) {
    frequency_cents_q16 = clamp(frequency_cents_q16, int64_t(-16000) * q16,
                                int64_t(4500) * q16);
    const uint64_t ratio = fast_pow2_cents_q30(frequency_cents_q16);
    const uint64_t microhertz = (8175799ull * ratio + (1ull << 29)) >> 30;
    // Reduce 2^48 / 10^12 by 4096 so the product remains within uint64_t.
    return microhertz * (1ull << 36) / 244140625ull;
}

inline uint64_t advance_phase(uint64_t phase_q48, uint64_t rate_q48, uint64_t microseconds) {
    constexpr uint64_t phase_mask = (uint64_t(1) << 48) - 1;
    // Unsigned overflow preserves the low 48 product bits needed by the phase.
    return (phase_q48 + rate_q48 * uint64_t(microseconds)) & phase_mask;
}

inline int32_t triangle_q15(uint64_t phase_q48) {
    const uint32_t phase = uint32_t(phase_q48 >> 16);
    const uint32_t position = phase >> 16;
    if (position < 16384) return int32_t(position * 2);
    if (position < 49152) return 32768 - int32_t((position - 16384) * 2);
    return -32768 + int32_t((position - 49152) * 2);
}

inline int64_t scale_q15(int64_t value, int32_t factor) {
    const int64_t product = value * factor;
    return product < 0 ? -((-product) >> 15) : product >> 15;
}

inline uint32_t attenuation_for_gain_q16(uint16_t gain) {
    if (gain >= 32768) return 0;
    if (!gain) return 1000 * uint32_t(q16);
    uint32_t low = 0, high = 1000;
    while (low + 1 < high) {
        const uint32_t middle = (low + high) / 2;
        if (attenuation_table.values[middle] >= gain) low = middle;
        else high = middle;
    }
    const uint32_t upper = attenuation_table.values[low];
    const uint32_t drop = upper - attenuation_table.values[high];
    const uint32_t fraction = drop ? ((upper - gain) * uint32_t(q16) + drop / 2) / drop : 0;
    return low * uint32_t(q16) + fraction;
}

inline bool local_bank_span(const BankView& bank, uint16_t zone_index,
                            const Zone*& zone, const Modulation*& modulations) {
    if (!bank.data || uintptr_t(bank.data) % 4 || bank.size < 48 ||
        bank.data[0] != 'E' || bank.data[1] != 'P' || bank.data[2] != 'S' || bank.data[3] != 'B' ||
        u16(bank.data + 4) != 2 || u16(bank.data + 6) != 48 || zone_index >= bank.zone_count()) return false;
    const uint64_t zone_offset = u32(bank.data + 16), modulation_offset = u32(bank.data + 20);
    const uint64_t zone_end = zone_offset + uint64_t(bank.zone_count()) * sizeof(Zone);
    const uint64_t modulation_end = modulation_offset + uint64_t(bank.modulation_count()) * sizeof(Modulation);
    if (zone_offset < 48 || zone_end > bank.size || modulation_offset != zone_end || modulation_end > bank.size) return false;
    zone = reinterpret_cast<const Zone*>(bank.data + zone_offset) + zone_index;
    if (zone->mod_count > 32 || zone->mod_begin > bank.modulation_count() ||
        zone->mod_count > bank.modulation_count() - zone->mod_begin) return false;
    modulations = reinterpret_cast<const Modulation*>(bank.data + modulation_offset) + zone->mod_begin;
    return true;
}

inline int64_t source_q30(uint16_t bits, uint8_t key, uint8_t velocity,
                          const Controls& controls, bool& valid) {
    if (bits & 0xf000) { valid = false; return 0; }
    const uint16_t index = bits & 127;
    const bool controller = (bits & 128) != 0;
    if (!controller && index == 0) return q30; // constant ignores all flags
    int64_t raw = 0, range = 128;
    if (controller) raw = controls.cc[index];
    else switch (index) {
        case 2: raw = velocity; break;
        case 3: raw = key; break;
        case 10: raw = controls.poly_pressure; break;
        case 13: raw = controls.channel_pressure; break;
        case 14: raw = controls.bend; range = 16384; break;
        case 16: raw = controls.bend_range_cents; range = 12700; break;
        default: valid = false; return 0;
    }
    int64_t directed = (bits & 256) ? range - 1 - raw : raw;
    const int64_t maximum = range==128 ? q30-q30/128 : range==16384 ? q30-q30/16384 : (range-1)*q30/range;
    const int64_t normalized_input = range==128 ? directed*(q30/128)
        : range==16384 ? directed*(q30/16384) : directed*q30/range;
    const uint16_t curve = (bits >> 10) & 3;
    const bool bipolar = (bits & 512) != 0;
    if (!bipolar) {
        if (curve == 0) return normalized_input;
        if (curve == 3) return directed * 2 >= range ? q30 : 0;
        if (directed <= 0) return 0;
        // Native seven-bit inputs land exactly on the published curve samples;
        // avoid introducing an interpolation rounding unit at those points.
        if (range == 128) {
            const int64_t value = curve == 1 ? concave_q30[directed]
                                             : q30 - concave_q30[127 - directed];
            return value > maximum ? maximum : value;
        }
        int64_t normalized = directed * q30 / (range - 1);
        int64_t value = curve == 1 ? concave(normalized) : convex(normalized);
        return value > maximum ? maximum : value;
    }
    const bool pitch_wheel = !controller && index == 14;
    int64_t value = (!pitch_wheel && directed == range - 1)
        ? maximum : -q30 + ((range==128 || range==16384) ? 2*normalized_input : 2*directed*q30/range);
    if (curve == 0) return value;
    if (curve == 3) return value >= 0 ? q30 : -q30;
    const bool negative = value < 0;
    int64_t magnitude = negative ? -value : value;
    int64_t normalized = maximum ? magnitude * q30 / maximum : 0;
    int64_t shaped = curve == 1 ? concave(normalized) : convex(normalized);
    if (!negative && shaped > maximum) shaped = maximum;
    return negative ? -shaped : shaped;
}
} // namespace detail

inline bool Controls::valid() const {
    if (bend > 16383 || bend_range_cents > 12827 || fine_tuning > 16383 ||
        coarse_tuning > 127 || poly_pressure > 127 || channel_pressure > 127) return false;
#ifdef __mips__
    for(unsigned i=0;i<128;i+=4){uint32_t word;__builtin_memcpy(&word,cc+i,4);if(word&0x80808080u)return false;}
#else
    for (uint8_t value : cc) if (value > 127) return false;
#endif
    return true;
}

namespace detail {
inline bool same_controls(const Controls& left, const Controls& right) {
    if (left.bend != right.bend || left.bend_range_cents != right.bend_range_cents ||
        left.fine_tuning != right.fine_tuning || left.coarse_tuning != right.coarse_tuning ||
        left.poly_pressure != right.poly_pressure || left.channel_pressure != right.channel_pressure) return false;
#ifdef __mips__
    for(unsigned i=0;i<128;i+=4){uint32_t a,b;__builtin_memcpy(&a,left.cc+i,4);__builtin_memcpy(&b,right.cc+i,4);if(a!=b)return false;}
#else
    for (uint16_t i = 0; i < 128; ++i) if (left.cc[i] != right.cc[i]) return false;
#endif
    return true;
}
inline bool source_changed(uint16_t source,const Controls& a,const Controls& b){
    const auto index=source&127;
    if(source&128)return a.cc[index]!=b.cc[index];
    switch(index){
        case 10:return a.poly_pressure!=b.poly_pressure;
        case 13:return a.channel_pressure!=b.channel_pressure;
        case 14:return a.bend!=b.bend;
        case 16:return a.bend_range_cents!=b.bend_range_cents;
        default:return false; // Constant, key and velocity cannot change on a live note.
    }
}
} // namespace detail

inline void State::reset() {
    zone_ = nullptr; modulations_ = nullptr; modulation_count_ = 0;
    key_ = velocity_ = actual_key_ = actual_velocity_ = 0;
    for (auto& value : modulation_q16_) value = 0;
    volume_parameters_ = {}; modulation_parameters_ = {};
    volume_envelope_ = {}; modulation_envelope_ = {};
    modulation_lfo_ = {}; vibrato_lfo_ = {};
    modulation_lfo_delay_ = vibrato_lfo_delay_ = 0;
    modulation_lfo_rate_q48_ = vibrato_lfo_rate_q48_ = 0;
    base_pitch_q16_ = attenuation_q16_ = pan_q16_ = 0;
    mod_env_pitch_q16_ = mod_lfo_pitch_q16_ = vib_lfo_pitch_q16_ = mod_lfo_volume_q16_ = 0;
    static_gain_q15_ = 32768;
    age_microseconds_ = 0; output_ = {}; error_ = Error::None;
    started_ = false; released_ = false;advance_mask_=0x87;initial_parameters_=nullptr;
}
inline void State::copy_initial(const State& source){
    if(this==&source)return;
    zone_=source.zone_;modulations_=source.modulations_;modulation_count_=source.modulation_count_;
    key_=source.key_;velocity_=source.velocity_;actual_key_=source.actual_key_;actual_velocity_=source.actual_velocity_;
    controls_.fine_tuning=source.controls_.fine_tuning;controls_.coarse_tuning=source.controls_.coarse_tuning;
    // Envelope/LFO clocks and output parameters belong to each physical voice.
    // The much larger controller/modulator baseline stays shared until changed.
    constexpr size_t offset=offsetof(State,volume_parameters_);
    static_assert(offset%4==0);
    detail::copy_block<sizeof(State)-offset>(&volume_parameters_,&source.volume_parameters_);
    initial_parameters_=source.initial_parameters_?source.initial_parameters_:&source;
}

inline Error State::start(const BankView& bank, uint16_t zone_index, uint8_t key,
                          uint8_t velocity, const Controls& controls) {
    const auto fail=[&](Error error){reset();return error_=error;};
    if (key > 127 || velocity > 127) return fail(Error::InvalidNote);
    if (!controls.valid()) return fail(Error::InvalidControls);
    const Zone* zone = nullptr; const Modulation* modulations = nullptr;
    if (bank.data && bank.size >= 12 && zone_index >= bank.zone_count()) return fail(Error::InvalidZone);
    if (!detail::local_bank_span(bank, zone_index, zone, modulations)) return fail(Error::InvalidBank);
    return start_voice(*zone, modulations, zone->mod_count, key, velocity, controls);
}

inline Error State::start_validated(const BankView& bank,uint16_t zone_index,uint8_t key,
                                    uint8_t velocity,const Controls& controls){
    reset();
    const auto& zone=bank.zone(zone_index);
    return begin_voice(zone,zone.mod_count?&bank.modulation(zone.mod_begin):nullptr,zone.mod_count,key,velocity,controls,true);
}

inline Error State::start_voice(const Zone& zone, const Modulation* modulations,
                                uint16_t modulation_count, uint8_t key, uint8_t velocity,
                                const Controls& controls) {
    reset();
    if (key > 127 || velocity > 127) return error_ = Error::InvalidNote;
    if (!controls.valid()) return error_ = Error::InvalidControls;
    if (modulation_count > 32 || (modulation_count && !modulations) ||
        (modulations && uintptr_t(modulations) % alignof(Modulation)) ||
        zone.key_lo > zone.key_hi || zone.key_hi > 127 || zone.velocity_lo > zone.velocity_hi ||
        zone.velocity_hi > 127 || zone.root_key > 127 || (zone.fixed_key != 255 && zone.fixed_key > 127) ||
        (zone.fixed_velocity != 255 && zone.fixed_velocity > 127) ||
        !instrument::range(zone.tune, -14000, 14000) || !instrument::range(zone.scale, 0, 1200) ||
        !instrument::range(zone.attenuation, 0, 1440) || !instrument::range(zone.pan, -500, 500) ||
        !instrument::range(zone.mod_env_pitch, -12000, 12000) || !instrument::range(zone.reverb, 0, 1000) ||
        !instrument::valid_envelope(zone.volume_envelope, true) ||
        !instrument::valid_envelope(zone.modulation_envelope, false) ||
        !instrument::valid_lfo(zone.modulation_lfo, false) ||
        !instrument::valid_lfo(zone.vibrato_lfo, true)) return error_ = Error::InvalidBank;
    for (uint16_t i = 0; i < modulation_count; ++i) {
        const auto& modulation = modulations[i];
        if (!instrument::valid_source(modulation.source) || !instrument::valid_source(modulation.amount_source) ||
            modulation.flags > 1 || modulation.reserved) return error_ = Error::InvalidBank;
    }
    if (key < zone.key_lo || key > zone.key_hi || velocity < zone.velocity_lo || velocity > zone.velocity_hi)
        return error_ = Error::InvalidNote;
    return begin_voice(zone,modulations,modulation_count,key,velocity,controls);
}

inline Error State::begin_voice(const Zone& zone,const Modulation* modulations,uint16_t modulation_count,
                                uint8_t key,uint8_t velocity,const Controls& controls,bool target_only){
    zone_ = &zone; modulations_ = modulations; modulation_count_ = modulation_count;
    key_ = key; velocity_ = velocity;
    actual_key_ = zone.fixed_key == 255 ? key : zone.fixed_key;
    actual_velocity_ = zone.fixed_velocity == 255 ? velocity : zone.fixed_velocity;
    controls_ = controls; started_ = true;
    if(target_only){
        advance_mask_=uint8_t((zone.mod_env_pitch?1:0)|((zone.modulation_lfo.pitch || zone.modulation_lfo.volume)?2:0)|(zone.vibrato_lfo.pitch?4:0));
        // Include every potential nonzero destination, even when its current
        // controller is zero: enabling vibrato later must retain its phase.
        for(uint16_t i=0;i<modulation_count;++i)if(modulations[i].amount){
            switch(modulations[i].destination){
                case ModEnvPitch:advance_mask_|=1;break;
                case ModLfoPitch:case ModLfoVolume:advance_mask_|=2;break;
                case VibLfoPitch:advance_mask_|=4;break;
                default:break;
            }
        }
    }
    volume_envelope_.phase = Phase::Delay;
    modulation_envelope_.phase = advance_mask_&1?Phase::Delay:Phase::Done;
    if ((error_ = rebuild_parameters(true)) != Error::None) { output_.finished = true; started_ = false; return error_; }
    advance(0);
    return error_;
}

inline Error State::update_controls(const Controls& controls) {
    if (!started_) return error_ == Error::None ? Error::InvalidZone : error_;
    if (!controls.valid()) {
        error_ = Error::InvalidControls; output_.gain_q15 = 0;
        output_.finished = true; started_ = false;
        return error_;
    }
    const auto& previous=initial_parameters_?initial_parameters_->controls_:controls_;
    if (detail::same_controls(previous, controls)) return Error::None;
    // Integrate old frequencies before any control can change a rate/depth.
    synchronize_lfos(true);
    uint32_t dirty=0;
    for(uint16_t i=0;i<modulation_count_;++i){
        const auto& mod=modulations_[i];
        if(detail::source_changed(mod.source,previous,controls) || detail::source_changed(mod.amount_source,previous,controls))
            dirty|=uint32_t(1)<<mod.destination;
    }
    if(initial_parameters_){
        detail::copy_block<sizeof(modulation_q16_)>(modulation_q16_,initial_parameters_->modulation_q16_);
        initial_parameters_=nullptr;
    }
    constexpr uint32_t envelope_destinations=((uint32_t(1)<<29)-1)^((uint32_t(1)<<13)-1);
    if(!(dirty&envelope_destinations)){
        detail::copy_value(controls_,controls);
        error_=rebuild_parameters(false,dirty);
        if(error_!=Error::None){output_.finished=true;started_=false;output_.gain_q15=0;return error_;}
        refresh_output();return error_;
    }
    const EnvelopeParameters old_volume_parameters = volume_parameters_;
    const EnvelopeParameters old_modulation_parameters = modulation_parameters_;
    const uint64_t old_volume_release = volume_envelope_.release_duration;
    const uint64_t old_modulation_release = modulation_envelope_.release_duration;
    controls_ = controls;
    error_ = rebuild_parameters(false,dirty);
    if (error_ != Error::None) { output_.finished = true; started_ = false; return error_; }
    uint64_t new_volume_release = old_volume_release;
    uint64_t new_modulation_release = old_modulation_release;
    if (released_) {
        if(volume_parameters_.release!=old_volume_parameters.release)new_volume_release = uint64_t(volume_parameters_.release) *
            volume_envelope_.release_attenuation_q16 / (1000 * detail::q16);
        if(modulation_parameters_.release!=old_modulation_parameters.release)new_modulation_release = uint64_t(modulation_parameters_.release) *
            modulation_envelope_.release_attenuation_q16 / detail::q16;
    }
    const auto retime = [](EnvelopeState& state, const EnvelopeParameters& old_parameters,
                           const EnvelopeParameters& new_parameters, uint64_t old_release,
                           uint64_t new_release) {
        const auto duration = [&](const EnvelopeParameters& parameters, uint64_t release) -> uint64_t {
            switch (state.phase) {
                case Phase::Delay: return parameters.delay;
                case Phase::Attack: return parameters.attack;
                case Phase::Hold: return parameters.hold;
                case Phase::Decay: return parameters.decay;
                case Phase::Release: return release;
                default: return uint64_t(0);
            }
        };
        const uint64_t old_duration = duration(old_parameters, old_release);
        const uint64_t new_duration = duration(new_parameters, new_release);
        if(old_duration==new_duration){state.release_duration=new_release;return;}
        if (old_duration && state.elapsed < old_duration)
            state.elapsed = state.elapsed * new_duration / old_duration;
        else if (!new_duration) state.elapsed = 0;
        state.release_duration = new_release;
        state.cached_duration = UINT64_MAX;
    };
    // Changing envelope time preserves normalized phase, preventing a control
    // refresh from producing a gain/pitch discontinuity.
    retime(volume_envelope_, old_volume_parameters, volume_parameters_, old_volume_release, new_volume_release);
    retime(modulation_envelope_, old_modulation_parameters, modulation_parameters_, old_modulation_release, new_modulation_release);
    if (volume_envelope_.phase == Phase::Sustain)
        volume_envelope_.level = detail::attenuation_gain_q15(volume_parameters_.sustain_q16);
    if (modulation_envelope_.phase == Phase::Sustain)
        modulation_envelope_.level = uint16_t(32768 - modulation_parameters_.sustain_q16 / 2000);
    // Re-evaluate a partially completed stage against any newly modulated
    // duration or sustain value without advancing musical time.
    advance(0);
    return error_;
}

inline Error State::rebuild_parameters(bool force,uint32_t dirty) {
    using namespace detail;
    if(force)for(uint16_t i=0;i<modulation_count_;++i){
        const auto& mod=modulations_[i];
        if((mod.destination<Pitch || mod.destination>Reverb) && !(mod.destination==Chorus && mod.amount==0))return Error::UnsupportedDestination;
    }
    uint32_t changed = 0;
    for (uint16_t destination = Pitch; destination <= Reverb; ++destination) {
        const uint32_t bit=uint32_t(1)<<destination;
        if(!force && !(dirty&bit))continue;
        int64_t sum=0;bool valid=true;
        for(uint16_t i=0;i<modulation_count_;++i){
            const auto& mod=modulations_[i];if(mod.destination!=destination)continue;
            const int64_t first=source_q30(mod.source,actual_key_,actual_velocity_,controls_,valid);
            const int64_t second=source_q30(mod.amount_source,actual_key_,actual_velocity_,controls_,valid);
            if(!valid)return Error::InvalidBank;
            const int64_t factor=first*second/q30;
            int64_t value=int64_t(mod.amount)*factor/(q30/q16);
            if((mod.flags&1) && value<0)value=-value;
            if(mod.flags>1)return Error::InvalidBank;
            if((value>0 && sum>INT64_MAX-value) || (value<0 && sum<INT64_MIN-value))return Error::ArithmeticOverflow;
            sum+=value;
        }
        if(force || modulation_q16_[destination]!=sum)changed|=bit;
        modulation_q16_[destination]=sum;
    }
    const auto changed_destination = [&](uint16_t destination) {
        return (changed & (uint32_t(1) << destination)) != 0;
    };
    const auto effective = [&](uint16_t destination, int32_t base) {
        return int64_t(base) * q16 + modulation_q16_[destination];
    };
    if(changed_destination(Scale) || changed_destination(Pitch)){
        const int64_t scale=clamp(effective(Scale,zone_->scale),0,1200*q16);
        base_pitch_q16_=int64_t(zone_->tune)*q16+int64_t(int(actual_key_)-int(zone_->root_key))*scale+modulation_q16_[Pitch];
    }
    if(changed_destination(Attenuation)){
        attenuation_q16_=clamp(effective(Attenuation,zone_->attenuation),0,1440*q16);
        static_gain_q15_=attenuation_gain_q15(attenuation_q16_);
    }
    if(changed_destination(Pan))pan_q16_=clamp(effective(Pan,zone_->pan),-500*q16,500*q16);
    if(changed_destination(Reverb))output_.reverb_send_permille=uint16_t(clamp(effective(Reverb,zone_->reverb),0,1000*q16)/q16);
    if(changed_destination(ModEnvPitch))mod_env_pitch_q16_=clamp(effective(ModEnvPitch,zone_->mod_env_pitch),-12000*q16,12000*q16);
    if(changed_destination(ModLfoPitch))mod_lfo_pitch_q16_=clamp(effective(ModLfoPitch,zone_->modulation_lfo.pitch),-12000*q16,12000*q16);
    if(changed_destination(VibLfoPitch))vib_lfo_pitch_q16_=clamp(effective(VibLfoPitch,zone_->vibrato_lfo.pitch),-12000*q16,12000*q16);
    if(changed_destination(ModLfoVolume))mod_lfo_volume_q16_=clamp(effective(ModLfoVolume,zone_->modulation_lfo.volume),-960*q16,960*q16);

    const auto envelope = [&](const Envelope& source, bool volume, uint16_t first,
                              EnvelopeParameters& target) {
        if(!(changed&(uint32_t(255)<<first)))return;
        const int64_t hold_key = clamp(effective(first + 6, source.hold_key), -1200 * q16, 1200 * q16);
        const int64_t decay_key = clamp(effective(first + 7, source.decay_key), -1200 * q16, 1200 * q16);
        const int64_t key_delta = 60 - int64_t(actual_key_);
        if (force || changed_destination(first))
            target.delay = uint32_t(time_microseconds(effective(first, source.delay), 5000, true));
        if (force || changed_destination(first + 1))
            target.attack = uint32_t(time_microseconds(effective(first + 1, source.attack), 8000, true));
        if (force || changed_destination(first + 2) || changed_destination(first + 6))
            target.hold = uint32_t(time_microseconds(effective(first + 2, source.hold) + hold_key * key_delta, 5000, true));
        const bool sustain_changed = force || changed_destination(first + 4);
        if (sustain_changed)
            target.sustain_q16 = uint32_t(clamp(effective(first + 4, source.sustain), 0,
                int64_t(volume ? 1440 : 1000) * q16));
        if (force || changed_destination(first + 3) || changed_destination(first + 7) || sustain_changed) {
            target.decay = target.sustain_q16 ? uint32_t(time_microseconds(effective(first + 3, source.decay) + decay_key * key_delta, 8000, false)) : 0;
            // Both decay generators describe a full-scale excursion. Stop when
            // the authored sustain reduction is reached, including fractions.
            if(target.sustain_q16 && target.sustain_q16!=1000*q16)
                target.decay = uint32_t(uint64_t(target.decay) * target.sustain_q16 / (1000 * q16));
        }
        if (force || changed_destination(first + 5))
            target.release = uint32_t(time_microseconds(effective(first + 5, source.release), 8000, false));
        target.volume = volume;
    };
    envelope(zone_->volume_envelope, true, VolEnvDelay, volume_parameters_);
    if(advance_mask_&1)envelope(zone_->modulation_envelope, false, ModEnvDelay, modulation_parameters_);
    if ((advance_mask_&2) && (force || changed_destination(ModLfoDelay)))
        modulation_lfo_delay_ = time_microseconds(effective(ModLfoDelay, zone_->modulation_lfo.delay), 5000, true);
    if ((advance_mask_&4) && (force || changed_destination(VibLfoDelay)))
        vibrato_lfo_delay_ = time_microseconds(effective(VibLfoDelay, zone_->vibrato_lfo.delay), 5000, true);
    if ((advance_mask_&2) && (force || changed_destination(ModLfoFrequency)))
        modulation_lfo_rate_q48_ = lfo_rate_q48(effective(ModLfoFrequency, zone_->modulation_lfo.frequency));
    if ((advance_mask_&4) && (force || changed_destination(VibLfoFrequency)))
        vibrato_lfo_rate_q48_ = lfo_rate_q48(effective(VibLfoFrequency, zone_->vibrato_lfo.frequency));
    // LFO control changes are prospective: accumulated phase is retained,
    // while a moved delay gates output and future integration without inventing
    // phase for time that elapsed under the previous control value.
    return Error::None;
}

inline void State::release() {
    if (!started_ || released_ || output_.finished) return;
    released_ = true;
    const auto volume_attenuation = [&]() -> uint32_t {
        switch (volume_envelope_.phase) {
            case Phase::Delay: return 1000 * uint32_t(detail::q16);
            case Phase::Attack: return detail::attenuation_for_gain_q16(volume_envelope_.level);
            case Phase::Hold: return 0;
            case Phase::Decay:
                return volume_parameters_.decay ? uint32_t(uint64_t(volume_parameters_.sustain_q16) *
                    volume_envelope_.elapsed / volume_parameters_.decay) : volume_parameters_.sustain_q16;
            case Phase::Sustain: return volume_parameters_.sustain_q16;
            default: return 1000 * uint32_t(detail::q16);
        }
    };
    const auto modulation_level = [&]() -> uint32_t {
        switch (modulation_envelope_.phase) {
            case Phase::Delay: return 0;
            case Phase::Attack: return uint32_t(modulation_envelope_.level) * 2;
            case Phase::Hold: return uint32_t(detail::q16);
            case Phase::Decay:
                return modulation_parameters_.decay ? uint32_t(detail::q16 -
                    uint64_t(modulation_parameters_.sustain_q16) * modulation_envelope_.elapsed /
                    (1000ull * modulation_parameters_.decay)) : uint32_t(detail::q16 - modulation_parameters_.sustain_q16 / 1000);
            case Phase::Sustain: return uint32_t(detail::q16 - modulation_parameters_.sustain_q16 / 1000);
            default: return 0;
        }
    };
    const uint32_t attenuation = volume_attenuation();
    const uint32_t remaining_attenuation = attenuation < 1000 * detail::q16
        ? 1000 * uint32_t(detail::q16) - attenuation : 0;
    volume_envelope_.release_duration = remaining_attenuation==1000*detail::q16?volume_parameters_.release
        : remaining_attenuation?uint64_t(volume_parameters_.release) * remaining_attenuation / (1000 * detail::q16):0;
    volume_envelope_.release_attenuation_q16 = remaining_attenuation;
    volume_envelope_.release_level = volume_envelope_.level;
    volume_envelope_.phase = Phase::Release; volume_envelope_.elapsed = 0;
    if(advance_mask_&1){
    const uint32_t mod_level = modulation_level();
    modulation_envelope_.release_duration = mod_level==detail::q16?modulation_parameters_.release
        : mod_level?uint64_t(modulation_parameters_.release) * mod_level / detail::q16:0;
    modulation_envelope_.release_attenuation_q16 = mod_level;
    modulation_envelope_.release_level = modulation_envelope_.level;
    modulation_envelope_.phase = Phase::Release; modulation_envelope_.elapsed = 0;
    }
    // A positive release begins at the existing amplitude and phase. Its first
    // timed update can initialize the reciprocal, avoiding a zero-time rebuild
    // for every simultaneous Note Off. Zero-length releases still retire now.
    if(!volume_envelope_.release_duration || ((advance_mask_&1) && !modulation_envelope_.release_duration))advance(0);
}

inline void State::synchronize_lfos(bool force){
    const auto update=[&](LfoState& state,uint64_t delay,uint64_t rate){
        const uint64_t before=state.updated_us>delay?state.updated_us-delay:0;
        const uint64_t after=age_microseconds_>delay?age_microseconds_-delay:0;
        state.phase_q48=detail::advance_phase(state.phase_q48,rate,after-before);
        state.updated_us=age_microseconds_;
    };
    const bool visible=force || (advance_mask_&128);
    if((advance_mask_&2) && (visible || mod_lfo_pitch_q16_ || mod_lfo_volume_q16_))update(modulation_lfo_,modulation_lfo_delay_,modulation_lfo_rate_q48_);
    if((advance_mask_&4) && (visible || vib_lfo_pitch_q16_))update(vibrato_lfo_,vibrato_lfo_delay_,vibrato_lfo_rate_q48_);
}

inline uint32_t State::envelope_progress(EnvelopeState& state,uint64_t duration){
    using namespace detail;
    if(!duration)return uint32_t(q16);
    if(state.cached_duration!=duration){
        state.cached_duration=duration;
        state.progress_reciprocal_q48=((uint64_t(1)<<48)+duration-1)/duration;
    }
    if(duration>=65536 && duration<=UINT32_MAX){
        const auto elapsed=uint32_t(state.elapsed),length=uint32_t(duration);
        uint32_t part=uint32_t(uint64_t(elapsed)*uint32_t(state.progress_reciprocal_q48)>>32);
        if(part>uint32_t(q16))part=uint32_t(q16);
        const uint64_t numerator=uint64_t(elapsed)<<16;
        if(part && uint64_t(part)*length>numerator)--part;
        else if(part<uint32_t(q16) && uint64_t(part+1)*length<=numerator)++part;
        return part;
    }
    uint64_t part=(state.elapsed*state.progress_reciprocal_q48)>>32;
    if(part>uint64_t(q16))part=q16;
    const uint64_t numerator=state.elapsed<<16;
    if(part && part*duration>numerator)--part;
    else if(part<uint64_t(q16) && (part+1)*duration<=numerator)++part;
    return uint32_t(part);
}

#if defined(__mips__) && defined(__GNUC__)
// This bounded arithmetic loop is a measured PSX hot path. Keep the stronger
// optimization local; renderer, game scripts and legacy audio keep their flags.
__attribute__((optimize("O3")))
#endif
inline const Output& State::advance(uint32_t microseconds) {
    using namespace detail;
    if (!started_ || error_ != Error::None) return output_;
    const uint16_t old_level=volume_envelope_.level;
    age_microseconds_ = UINT64_MAX - age_microseconds_ < microseconds ? UINT64_MAX : age_microseconds_ + microseconds;
    // Silent target LFOs catch up exactly when enabled. Their phase includes
    // frequency changes and long delays; source/filter signals remain eager.
    synchronize_lfos(false);

    // A target release with no audible modulation cannot transition early or
    // change pitch/pan. Keep its exact envelope equation in a small hot path.
    if(!(advance_mask_&129) && !output_.finished && !mod_env_pitch_q16_ &&
        !mod_lfo_pitch_q16_ && !vib_lfo_pitch_q16_ && !mod_lfo_volume_q16_ &&
        volume_envelope_.phase==Phase::Release && volume_envelope_.elapsed<volume_envelope_.release_duration &&
        microseconds<volume_envelope_.release_duration-volume_envelope_.elapsed){
        auto& state=volume_envelope_;state.elapsed+=microseconds;
        const auto progress=envelope_progress(state,state.release_duration);
        state.level=uint16_t(uint32_t(state.release_level)*attenuation_gain_q15(
            int64_t(uint64_t(state.release_attenuation_q16)*progress>>16))>>15);
        const auto gain=(uint32_t(state.level)*static_gain_q15_+16384)>>15;
        output_.gain_q15=uint16_t(gain>32767?32767:gain);
        return output_;
    }

    const auto step = [&](EnvelopeState& state, const EnvelopeParameters& parameters) {
        uint64_t remaining = microseconds;
        for (unsigned transitions = 0; transitions < 7; ++transitions) {
            if (state.phase == Phase::Done || state.phase == Phase::Sustain) break;
            uint64_t duration = 0;
            switch (state.phase) {
                case Phase::Delay: duration = parameters.delay; state.level = 0; break;
                case Phase::Attack: duration = parameters.attack; break;
                case Phase::Hold: duration = parameters.hold; state.level = 32768; break;
                case Phase::Decay: duration = parameters.decay; break;
                case Phase::Release: duration = state.release_duration; break;
                default: break;
            }
            const uint64_t available = duration > state.elapsed ? duration - state.elapsed : 0;
            const uint64_t consumed = remaining < available ? remaining : available;
            state.elapsed += consumed; remaining -= consumed;
            const uint32_t progress=envelope_progress(state,duration);
            switch (state.phase) {
                case Phase::Attack:
                    // Volume attack is linear amplitude; the modulation
                    // envelope uses the SF2 convex control curve (gen 26).
                    state.level = parameters.volume ? uint16_t(progress >> 1)
                        : uint16_t(convex(int64_t(progress) << 14) >> 15);
                    break;
                case Phase::Decay:
                    if (parameters.volume) state.level = attenuation_gain_q15(
                        int64_t(uint64_t(parameters.sustain_q16) * uint32_t(progress) >> 16));
                    else state.level = uint16_t(32768 -
                        uint64_t(parameters.sustain_q16) * progress / (2000ull * q16));
                    break;
                case Phase::Release:
                    if (parameters.volume) state.level = uint16_t(uint32_t(state.release_level) *
                        attenuation_gain_q15(int64_t(uint64_t(state.release_attenuation_q16) * uint32_t(progress) >> 16)) >> 15);
                    else state.level = uint16_t(uint64_t(state.release_level) *
                        (q16 - (progress >= uint64_t(q16) ? q16 : progress)) >> 16);
                    break;
                default: break;
            }
            if (state.elapsed < duration) break;
            state.elapsed = 0;
            switch (state.phase) {
                case Phase::Delay: state.phase = Phase::Attack; state.level = 0; break;
                case Phase::Attack: state.phase = Phase::Hold; state.level = 32768; break;
                case Phase::Hold: state.phase = Phase::Decay; break;
                case Phase::Decay:
                    state.phase = Phase::Sustain;
                    state.level = parameters.volume ? attenuation_gain_q15(parameters.sustain_q16)
                        : uint16_t(32768 - parameters.sustain_q16 / 2000);
                    break;
                case Phase::Release: state.phase = Phase::Done; state.level = 0; break;
                default: break;
            }
            if (!remaining && state.phase != Phase::Delay && state.phase != Phase::Attack &&
                state.phase != Phase::Hold && state.phase != Phase::Decay && state.phase != Phase::Release) break;
        }
    };
    step(volume_envelope_, volume_parameters_);
    if(advance_mask_&1)step(modulation_envelope_, modulation_parameters_);
    if(!(advance_mask_&128) && !output_.finished && volume_envelope_.phase!=Phase::Done &&
        !mod_env_pitch_q16_ && !mod_lfo_pitch_q16_ && !vib_lfo_pitch_q16_ && !mod_lfo_volume_q16_){
        if(old_level!=volume_envelope_.level){
            const auto gain=(uint32_t(volume_envelope_.level)*static_gain_q15_+16384)>>15;
            output_.gain_q15=uint16_t(gain>32767?32767:gain);
        }
        return output_;
    }
    refresh_output();
    return output_;
}

inline State::HardwareParameters State::hardware_parameters() const {
    return {volume_parameters_.delay,volume_parameters_.attack,volume_parameters_.hold,
        volume_parameters_.decay,volume_parameters_.release,
        detail::attenuation_gain_q15(volume_parameters_.sustain_q16),static_gain_q15_};
}
inline Output State::hardware_output() {
    const auto saved=output_;const auto level=volume_envelope_.level;const auto phase=volume_envelope_.phase;
    volume_envelope_.level=32768;volume_envelope_.phase=Phase::Sustain;
    refresh_output();const auto result=output_;
    volume_envelope_.level=level;volume_envelope_.phase=phase;output_=saved;return result;
}
inline void State::refresh_output() {
    using namespace detail;
    if (!started_ || error_ != Error::None || volume_envelope_.phase == Phase::Done) {
        output_.gain_q15 = 0; output_.modulation_envelope_q15 = 0;
        output_.modulation_lfo_q15 = output_.vibrato_lfo_q15 = 0;
        output_.finished = true; return;
    }
    const int32_t mod_triangle = (advance_mask_&2) && age_microseconds_ > modulation_lfo_delay_ ? triangle_q15(modulation_lfo_.phase_q48) : 0;
    const int32_t vib_triangle = (advance_mask_&4) && age_microseconds_ > vibrato_lfo_delay_ ? triangle_q15(vibrato_lfo_.phase_q48) : 0;
    int64_t pitch = base_pitch_q16_;
    if (mod_env_pitch_q16_) pitch += scale_q15(mod_env_pitch_q16_, modulation_envelope_.level);
    if (mod_lfo_pitch_q16_) pitch += scale_q15(mod_lfo_pitch_q16_, mod_triangle);
    if (vib_lfo_pitch_q16_) pitch += scale_q15(vib_lfo_pitch_q16_, vib_triangle);
    const int64_t scaled_pitch = pitch * 100;
    int64_t cents_x100 = scaled_pitch >= 0 ? (scaled_pitch + q16 / 2) / q16
                                           : (scaled_pitch - q16 / 2) / q16;
    cents_x100 += int64_t(int(controls_.fine_tuning) - 8192) * 10000 / 8192;
    cents_x100 += int64_t(int(controls_.coarse_tuning) - 64) * 10000;
    if (cents_x100 < INT32_MIN || cents_x100 > INT32_MAX) {
        error_ = Error::ArithmeticOverflow; output_.gain_q15 = 0;
        output_.modulation_envelope_q15 = 0;
        output_.modulation_lfo_q15 = output_.vibrato_lfo_q15 = 0;
        output_.finished = true; started_ = false; return;
    }
    int64_t attenuation = attenuation_q16_;
    // A positive modulation LFO excursion raises volume (SF2 generator 13),
    // so it subtracts from attenuation rather than increasing attenuation.
    if (mod_lfo_volume_q16_) attenuation -= scale_q15(mod_lfo_volume_q16_, mod_triangle);
    attenuation = clamp(attenuation, 0, 1440 * q16);
    const uint32_t static_gain = mod_lfo_volume_q16_ ? attenuation_gain_q15(attenuation) : static_gain_q15_;
    output_.pitch_cents_x100 = int32_t(cents_x100);
    const uint32_t gain = (uint32_t(volume_envelope_.level) * static_gain + 16384) >> 15;
    output_.gain_q15 = uint16_t(gain > 32767 ? 32767 : gain);
    output_.pan_permille = int16_t(clamp(pan_q16_ / q16, -500, 500));
    output_.modulation_envelope_q15 = modulation_envelope_.level;
    output_.modulation_lfo_q15 = mod_triangle;
    output_.vibrato_lfo_q15 = vib_triangle;
    output_.finished = false;
}

} // namespace epok::instrument::synth
