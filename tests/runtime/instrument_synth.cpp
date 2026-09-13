#include "../../runtime/instrument_synth.hpp"
#include "../../runtime/instrument_preparation.hpp"
#include <array>
#include <cassert>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <limits>

using namespace epok::instrument;
using namespace epok::instrument::synth;

void w16(uint8_t* p, uint16_t value) { p[0] = uint8_t(value); p[1] = uint8_t(value >> 8); }
void w32(uint8_t* p, uint32_t value) { w16(p, uint16_t(value)); w16(p + 2, uint16_t(value >> 16)); }

struct Fixture {
    alignas(4) std::array<uint8_t, 1024> bytes{};
    uint32_t size = 0;
    explicit Fixture(uint16_t modulation_count = 0) {
        const uint32_t zone_offset = 72, modulation_offset = 216;
        const uint32_t data_offset = (modulation_offset + modulation_count * 16 + 63) / 64 * 64;
        size = data_offset + 64;
        bytes[0] = 'E'; bytes[1] = 'P'; bytes[2] = 'S'; bytes[3] = 'B';
        w16(bytes.data() + 4, 2); w16(bytes.data() + 6, 48);
        w16(bytes.data() + 8, 1); w16(bytes.data() + 10, 1);
        w32(bytes.data() + 12, 48); w32(bytes.data() + 16, zone_offset);
        w32(bytes.data() + 20, modulation_offset); w32(bytes.data() + 24, data_offset);
        w32(bytes.data() + 28, size); w32(bytes.data() + 32, modulation_count);
        w32(bytes.data() + 36, 2);
        auto& sample = *reinterpret_cast<Sample*>(bytes.data() + 48);
        sample = {data_offset, 64, 22050, 56, 0, 0};
        auto& z = zone();
        z.key_hi = z.velocity_hi = 127; z.root_key = 60;
        z.fixed_key = z.fixed_velocity = 255; z.scale = 100;
        z.volume_envelope = envelope(true); z.modulation_envelope = envelope(false);
        z.modulation_lfo = lfo(); z.vibrato_lfo = lfo();
        z.mod_begin = 0; z.mod_count = modulation_count;
        bytes[data_offset + 17] = 1;
        bytes[data_offset + 33] = 7;
    }
    static Envelope envelope(bool) {
        return {-32768, -32768, -32768, -12000, 0, -12000, 0, 0};
    }
    static Lfo lfo() { return {-32768, 0, 0, 0}; }
    Zone& zone() { return *reinterpret_cast<Zone*>(bytes.data() + 72); }
    Modulation& modulation(uint16_t index) {
        return reinterpret_cast<Modulation*>(bytes.data() + 216)[index];
    }
    BankView view() const { return {bytes.data(), size}; }
};

void default_controls_and_pitch_contract() {
    Fixture fixture(1);
    fixture.modulation(0) = {uint16_t(14 | 512), 16, Pitch, 0, 12700, 0};
    auto bank = fixture.view(); assert(bank.valid(false));
    Controls controls;
    assert(controls.cc[7] == 100 && controls.cc[10] == 64 && controls.cc[11] == 127);
    controls.bend_range_cents = 1200; controls.bend = 8832;
    State state;
    assert(state.start(bank, 0, 60, 100, controls) == Error::None);
    assert(state.output().pitch_cents_x100 == 9375);
    controls.bend = 9600;
    assert(state.update_controls(controls) == Error::None);
    assert(state.output().pitch_cents_x100 == 20625);
    controls.bend = 8192; controls.fine_tuning = 12288; controls.coarse_tuning = 65;
    assert(state.update_controls(controls) == Error::None);
    assert(state.output().pitch_cents_x100 == 15000);
}

void fixed_inputs_direction_amount_source_absolute_and_sum() {
    Fixture fixture(3);
    auto& z = fixture.zone(); z.fixed_key = 12; z.fixed_velocity = 100;
    fixture.modulation(0) = {3, 0, Pan, 0, 128, 0};
    fixture.modulation(1) = {uint16_t(2 | 256), 0, Pan, 0, 128, 0};
    fixture.modulation(2) = {uint16_t(14 | 512), 0, Pitch, 1, 100, 0};
    auto bank = fixture.view(); assert(bank.valid(false));
    Controls controls; controls.bend = 4096;
    State state; assert(state.start(bank, 0, 90, 20, controls) == Error::None);
    assert(state.output().pan_permille == 39);
    assert(state.output().pitch_cents_x100 == -475000);
}

void source_curve_endpoints_match_the_rust_oracle() {
    Controls controls;
    bool valid = true;
    using epok::instrument::synth::detail::q30;
    using epok::instrument::synth::detail::source_q30;
    assert(source_q30(3, 64, 1, controls, valid) == q30 / 2);
    assert(source_q30(uint16_t(3 | 256), 64, 1, controls, valid) == int64_t(63) * q30 / 128);
    assert(source_q30(uint16_t(3 | 512), 64, 1, controls, valid) == 0);
    assert(source_q30(uint16_t(3 | 512), 127, 1, controls, valid) == int64_t(127) * q30 / 128);
    assert(source_q30(uint16_t(3 | 512 | 3072), 63, 1, controls, valid) == -q30);
    assert(source_q30(uint16_t(3 | 512 | 3072), 64, 1, controls, valid) == q30);
    controls.bend = 16383;
    assert(source_q30(uint16_t(14 | 512), 0, 0, controls, valid) == int64_t(8191) * q30 / 8192);
    const auto concave = source_q30(uint16_t(3 | 1024), 32, 0, controls, valid);
    const auto convex_mirror = source_q30(uint16_t(3 | 2048), 95, 0, controls, valid);
    assert(concave == epok::instrument::synth::detail::concave_q30[32]);
    assert(concave + convex_mirror >= int64_t(127) * q30 / 128 - 2);
    assert(valid);
}

void delay_attack_hold_and_key_scaled_decay_are_time_based() {
    Fixture fixture;
    auto& z = fixture.zone();
    z.volume_envelope.delay = -12000;
    z.volume_envelope.attack = -12000;
    z.volume_envelope.hold = 0;
    z.volume_envelope.hold_key = 100;
    z.mod_env_pitch = 100;
    z.modulation_envelope.decay = 0;
    z.modulation_envelope.decay_key = 100;
    z.modulation_envelope.sustain = 600;
    auto bank = fixture.view(); assert(bank.valid(false));
    State state; assert(state.start(bank, 0, 72, 100, Controls{}) == Error::None);
    assert(state.output().gain_q15 == 0);
    state.advance(977); // delay rounded from 976.5625 us
    assert(state.output().gain_q15 == 0);
    state.advance(489);
    assert(state.output().gain_q15 > 16000 && state.output().gain_q15 < 17000);
    state.advance(488); // attack reaches full; key-scaled hold is 500 ms
    assert(state.output().gain_q15 == 32767);
    state.advance(499999);
    assert(state.output().gain_q15 == 32767);
    state.advance(1);
    state.advance(250000); // The key-scaled 500 ms mod decay has already reached 40% sustain.
    assert(state.output().pitch_cents_x100 > 123900 && state.output().pitch_cents_x100 < 124100);
}

void every_retained_runtime_destination_is_accepted() {
    Fixture fixture(28);
    for (uint16_t destination = Pitch; destination <= ModEnvDecayKey; ++destination)
        fixture.modulation(destination - 1) = {0, 0, destination, 0, 0, 0};
    auto bank = fixture.view(); assert(bank.valid(false));
    State state;
    assert(state.start(bank, 0, 60, 100, Controls{}) == Error::None);
}

void control_updates_recompute_a_sustaining_envelope() {
    Fixture fixture(1);
    auto& z = fixture.zone();
    z.volume_envelope.decay = -12000;
    z.volume_envelope.sustain = 120;
    fixture.modulation(0) = {uint16_t(1 | 128), 0, VolEnvSustain, 0, 128, 0};
    auto bank = fixture.view(); assert(bank.valid(false));
    Controls controls;
    State state; assert(state.start(bank, 0, 60, 100, controls) == Error::None);
    state.advance(117);
    const uint16_t original = state.output().gain_q15;
    controls.cc[1] = 127;
    assert(state.update_controls(controls) == Error::None);
    assert(state.output().gain_q15 < original);
}

void envelopes_lfo_and_release_advance_across_frame_boundaries() {
    Fixture fixture;
    auto& z = fixture.zone();
    z.mod_env_pitch = 1200;
    z.modulation_envelope.decay = 0;
    z.modulation_envelope.sustain = 600;
    z.volume_envelope.decay = 0;
    z.volume_envelope.sustain = 120;
    z.volume_envelope.release = 0;
    auto bank = fixture.view(); assert(bank.valid(false));
    State state; assert(state.start(bank, 0, 60, 100, Controls{}) == Error::None);
    // Volume decay reaches 120 cB in 120 ms; modulation decay uses its full second.
    state.advance(120000);
    assert(state.output().gain_q15 > 8100 && state.output().gain_q15 < 8400);
    assert(state.output().pitch_cents_x100 > 105500 && state.output().pitch_cents_x100 < 105700);
    state.release();
    state.advance(879999);
    assert(!state.output().finished);
    state.advance(1);
    assert(state.output().finished && state.output().gain_q15 == 0 && !state.active());

    Fixture lfo_fixture;
    lfo_fixture.zone().modulation_lfo.pitch = 100;
    auto lfo_bank = lfo_fixture.view(); assert(lfo_bank.valid(false));
    State lfo; assert(lfo.start(lfo_bank, 0, 60, 100, Controls{}) == Error::None);
    lfo.advance(30579); // approximately one quarter cycle at MIDI-key-zero frequency
    assert(lfo.output().pitch_cents_x100 > 9900);
}

void invalid_inputs_and_unsupported_destinations_fail_explicitly() {
    Fixture fixture(1);
    fixture.modulation(0) = {0, 0, Chorus, 0, 1, 0};
    State state;
    assert(state.start(fixture.view(), 0, 60, 100, Controls{}) == Error::UnsupportedDestination);
    Controls invalid; invalid.cc[1] = 128;
    assert(state.start(fixture.view(), 0, 60, 100, invalid) == Error::InvalidControls);
    assert(state.start(fixture.view(), 1, 60, 100, Controls{}) == Error::InvalidZone);
    assert(state.start(fixture.view(), 0, 128, 100, Controls{}) == Error::InvalidNote);
    Fixture valid_fixture;
    assert(state.start(valid_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    assert(state.update_controls(invalid) == Error::InvalidControls);
    assert(state.output().finished && !state.active());
    assert(state.start_voice(valid_fixture.zone(), nullptr, 0, 60, 100, Controls{}) == Error::None);
    assert(state.start_voice(valid_fixture.zone(), nullptr, 1, 60, 100, Controls{}) == Error::InvalidBank);
}

void reverb_send_is_explicit_and_controller_driven() {
    Fixture fixture(1);
    fixture.zone().reverb = 100;
    fixture.modulation(0) = {uint16_t(91 | 128), 0, Reverb, 0, 1000, 0};
    Controls controls;
    State state;
    assert(state.start_voice(fixture.zone(), &fixture.modulation(0), 1, 60, 100, controls) == Error::None);
    assert(state.output().reverb_send_permille == 100);
    controls.cc[91] = 127;
    assert(state.update_controls(controls) == Error::None);
    assert(state.output().reverb_send_permille == 1000);
}

void independent_envelope_and_lfo_oracles() {
    for(int cents=-16000;cents<8000;cents+=13)for(int fraction:{0,16384,32768,49152}){
        const int64_t fixed=int64_t(cents)*65536+fraction;
        const double actual=double(epok::instrument::synth::detail::fast_pow2_cents_q30(fixed));
        const double expected=std::pow(2.,double(fixed)/(1200.*65536.))*double(1ull<<30);
        // Low Q30 magnitudes add quantization error; account for one output unit.
        assert(std::abs(actual-expected)<=1+expected*0.00000006);
    }
    Fixture attack_fixture;
    attack_fixture.zone().volume_envelope.attack = 0;
    State attack;
    assert(attack.start(attack_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    attack.advance(500000);
    assert(std::abs(int(attack.output().gain_q15) - 16384) <= 1 &&
        "SF2 attack is linear amplitude over the full attack time");

    Fixture mod_attack_fixture;
    mod_attack_fixture.zone().modulation_envelope.attack=0;
    mod_attack_fixture.zone().modulation_envelope.release=0;
    mod_attack_fixture.zone().volume_envelope.release=0;
    State mod_attack;
    assert(mod_attack.start(mod_attack_fixture.view(),0,60,100,Controls{})==Error::None);
    mod_attack.advance(500000);
    // Convex(x)=1-concave(1-x); independently evaluate its logarithmic curve.
    const double convex_half=1+(40./96.)*std::log10(.5);
    assert(std::abs(int(mod_attack.output().modulation_envelope_q15)-int(std::lround(32768*convex_half)))<=2);
    const auto release_level=mod_attack.output().modulation_envelope_q15;
    mod_attack.release();
    assert(mod_attack.output().modulation_envelope_q15==release_level);
    mod_attack.advance(100000);
    assert(std::abs(int(mod_attack.output().modulation_envelope_q15)-int(release_level)+3277)<=2);

    Fixture tremolo_fixture;
    tremolo_fixture.zone().attenuation=120;
    tremolo_fixture.zone().modulation_lfo.volume=60;
    State tremolo;
    assert(tremolo.start(tremolo_fixture.view(),0,60,100,Controls{})==Error::None);
    tremolo.advance(uint32_t(std::llround(1000000./(4.*8.175798915643707))));
    assert(std::abs(int(tremolo.output().gain_q15)-int(std::lround(32768*std::pow(10.,-60./200.))))<=2);

    Fixture volume_fixture;
    auto& volume = volume_fixture.zone().volume_envelope;
    volume.decay = 0; volume.sustain = 120; volume.release = 0;
    State volume_state;
    assert(volume_state.start(volume_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    volume_state.advance(60000);
    const auto expected_decay = int(std::lround(32767.0 * std::pow(10.0, -60.0 / 200.0)));
    assert(std::abs(int(volume_state.output().gain_q15) - expected_decay) <= 1);
    volume_state.advance(60000);
    const auto expected_sustain = int(std::lround(32767.0 * std::pow(10.0, -120.0 / 200.0)));
    assert(std::abs(int(volume_state.output().gain_q15) - expected_sustain) <= 1);
    volume_state.release(); volume_state.advance(500000);
    const auto expected_release = int(std::lround(double(expected_sustain) * std::pow(10.0, -500.0 / 200.0)));
    assert(std::abs(int(volume_state.output().gain_q15) - expected_release) <= 1);
    volume_state.advance(380000);
    assert(volume_state.output().finished && volume_state.output().gain_q15 == 0);

    Fixture modulation_fixture;
    auto& modulation = modulation_fixture.zone();
    modulation.mod_env_pitch = 1000;
    modulation.modulation_envelope.decay = 0;
    modulation.modulation_envelope.sustain = 600;
    modulation.modulation_envelope.release = 0;
    modulation.volume_envelope.release = 0;
    State modulation_state;
    assert(modulation_state.start(modulation_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    modulation_state.advance(300000);
    assert(std::abs(modulation_state.output().pitch_cents_x100 - 70000) <= 2);
    modulation_state.advance(300000);
    assert(std::abs(modulation_state.output().pitch_cents_x100 - 40000) <= 2);
    modulation_state.release();
    modulation_state.advance(200000);
    assert(std::abs(modulation_state.output().pitch_cents_x100 - 20000) <= 2);
    modulation_state.advance(200000);
    assert(std::abs(modulation_state.output().pitch_cents_x100) <= 2);

    Fixture silent_fixture;
    silent_fixture.zone().volume_envelope.decay = 0;
    silent_fixture.zone().volume_envelope.sustain = 1000;
    State silent;
    assert(silent.start(silent_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    silent.advance(1000000); silent.release();
    assert(silent.output().finished && "a voice already 100 dB down has no release tail");

    for (int input = 1; input < 127; ++input) {
        const double normalized = double(input) / 127.0;
        const auto oracle = int64_t(std::llround((-(40.0 / 96.0) * std::log10(1.0 - normalized)) * double(1ll << 30)));
        assert(std::llabs(int64_t(epok::instrument::synth::detail::concave_q30[input]) - oracle) <= 1);
    }

    Fixture lfo_fixture;
    lfo_fixture.zone().modulation_lfo.pitch = 100;
    State lfo; assert(lfo.start(lfo_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    const double frequency = 8.175798915643707;
    const uint32_t quarter = uint32_t(std::llround(1000000.0 / (4.0 * frequency)));
    lfo.advance(quarter);
    assert(std::abs(lfo.output().pitch_cents_x100 - 10000) <= 2);
    assert(lfo.output().modulation_lfo_q15 > 32760);
    lfo.advance(quarter);
    assert(std::abs(lfo.output().pitch_cents_x100) <= 2);
    lfo.advance(quarter);
    assert(std::abs(lfo.output().pitch_cents_x100 + 10000) <= 2);

    Fixture chunk_fixture;
    auto& chunk_zone = chunk_fixture.zone();
    chunk_zone.modulation_lfo.pitch = 173;
    chunk_zone.modulation_lfo.frequency = -123;
    chunk_zone.modulation_envelope.decay = 0;
    chunk_zone.modulation_envelope.sustain = 375;
    chunk_zone.mod_env_pitch = 321;
    State chunked, single;
    assert(chunked.start(chunk_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    assert(single.start(chunk_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    for (uint32_t i = 0; i < 1000; ++i) chunked.advance(1000);
    single.advance(1000000);
    assert(chunked.output().pitch_cents_x100 == single.output().pitch_cents_x100);
    assert(chunked.output().gain_q15 == single.output().gain_q15);
}

void extreme_modulation_amounts_remain_bounded() {
    Fixture cancelling(32);
    for (uint16_t i = 0; i < 32; ++i)
        cancelling.modulation(i) = {0, 0, Pitch, 0,
            i & 1 ? std::numeric_limits<int32_t>::min() : std::numeric_limits<int32_t>::max(), 0};
    State state;
    assert(state.start(cancelling.view(), 0, 60, 100, Controls{}) == Error::None);
    assert(state.output().pitch_cents_x100 == -1600);

    Fixture absolute(32);
    for (uint16_t i = 0; i < 32; ++i)
        absolute.modulation(i) = {0, 0, Pan, 1, std::numeric_limits<int32_t>::min(), 0};
    assert(state.start(absolute.view(), 0, 60, 100, Controls{}) == Error::None);
    assert(state.output().pan_permille == 500);

    Fixture dynamic(32);
    for (uint16_t i = 0; i < 32; ++i)
        dynamic.modulation(i) = {uint16_t(1 | 128), 0, Pitch, 0, std::numeric_limits<int32_t>::max(), 0};
    Controls controls;
    assert(state.start(dynamic.view(), 0, 60, 100, controls) == Error::None);
    controls.cc[1] = 127;
    assert(state.update_controls(controls) == Error::ArithmeticOverflow);
    assert(state.output().finished && !state.active());
}

void release_time_control_updates_are_bounded_and_immediate() {
    Fixture fixture(1);
    fixture.zone().volume_envelope.release = 0;
    fixture.modulation(0) = {uint16_t(1 | 128), 0, VolEnvRelease, 0, 1200, 0};
    Controls controls;
    State state;
    assert(state.start(fixture.view(), 0, 60, 100, controls) == Error::None);
    state.release(); state.advance(100000);
    const uint16_t original = state.output().gain_q15;
    controls.cc[1] = 127;
    assert(state.update_controls(controls) == Error::None);
    assert(std::abs(int(state.output().gain_q15) - int(original)) <= 1 &&
        "release-time modulation preserves normalized envelope phase");
    state.advance(2000000);
    assert(state.output().finished);
}

void benchmark_advance_24_voices_32_modulations() {
    Fixture fixture(32);
    for (uint16_t i = 0; i < 32; ++i)
        fixture.modulation(i) = {0, 0, uint16_t(Pitch + i % 28), 0, 0, 0};
    fixture.zone().modulation_lfo.pitch = 100;
    std::array<State, 24> states;
    for (auto& state : states)
        assert(state.start(fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    constexpr uint32_t iterations = 100000;
    int64_t checksum = 0;
    const auto begin = std::chrono::steady_clock::now();
    for (uint32_t i = 0; i < iterations; ++i)
        for (auto& state : states) checksum += state.advance(1000).pitch_cents_x100;
    const auto elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - begin).count();
    std::printf("instrument_synth: State=%zu bytes, advance(1000)=%lld ns/call, checksum=%lld\n",
        sizeof(State), static_cast<long long>(elapsed / (iterations * states.size())),
        static_cast<long long>(checksum));

    Fixture transient_fixture(32);
    for (uint16_t i = 0; i < 32; ++i)
        transient_fixture.modulation(i) = {0, 0, uint16_t(Pitch + i % 28), 0, 0, 0};
    transient_fixture.zone().volume_envelope.attack = 8000;
    transient_fixture.zone().modulation_envelope.attack = 8000;
    for (auto& state : states)
        assert(state.start(transient_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    const auto transient_begin = std::chrono::steady_clock::now();
    for (uint32_t i = 0; i < iterations; ++i)
        for (auto& state : states) checksum += state.advance(1000).gain_q15;
    const auto transient_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - transient_begin).count();
    std::printf("instrument_synth: advance(1000) two active envelopes=%lld ns/call, checksum=%lld\n",
        static_cast<long long>(transient_elapsed / (iterations * states.size())),
        static_cast<long long>(checksum));

    Fixture update_fixture(32);
    for (uint16_t i = 0; i < 32; ++i)
        update_fixture.modulation(i) = {uint16_t(1 | 128), 0, uint16_t(Pitch + i % 28), 0, 1, 0};
    for (auto& state : states)
        assert(state.start(update_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    Controls controls;
    constexpr uint32_t update_iterations = 5000;
    const auto update_begin = std::chrono::steady_clock::now();
    for (uint32_t i = 0; i < update_iterations; ++i) {
        controls.cc[1] = uint8_t(i & 1 ? 127 : 0);
        for (auto& state : states) {
            assert(state.update_controls(controls) == Error::None);
            checksum += state.output().pitch_cents_x100;
        }
    }
    const auto update_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - update_begin).count();
    std::printf("instrument_synth: update_controls(32 mods)=%lld ns/call, checksum=%lld\n",
        static_cast<long long>(update_elapsed / (update_iterations * states.size())),
        static_cast<long long>(checksum));

    Fixture pitch_fixture(32);
    for (uint16_t i = 0; i < 32; ++i)
        pitch_fixture.modulation(i) = {uint16_t(1 | 128), 0, Pitch, 0, 1, 0};
    for (auto& state : states)
        assert(state.start(pitch_fixture.view(), 0, 60, 100, Controls{}) == Error::None);
    controls = Controls{};
    const auto pitch_begin = std::chrono::steady_clock::now();
    for (uint32_t i = 0; i < update_iterations; ++i) {
        controls.cc[1] = uint8_t(i & 1 ? 127 : 0);
        for (auto& state : states) {
            assert(state.update_controls(controls) == Error::None);
            checksum += state.output().pitch_cents_x100;
        }
    }
    const auto pitch_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - pitch_begin).count();
    const auto same_begin = std::chrono::steady_clock::now();
    for (uint32_t i = 0; i < update_iterations; ++i)
        for (auto& state : states) {
            assert(state.update_controls(controls) == Error::None);
            checksum += state.output().pitch_cents_x100;
        }
    const auto same_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - same_begin).count();
    std::printf("instrument_synth: update pitch-only=%lld ns/call, unchanged=%lld ns/call, checksum=%lld\n",
        static_cast<long long>(pitch_elapsed / (update_iterations * states.size())),
        static_cast<long long>(same_elapsed / (update_iterations * states.size())),
        static_cast<long long>(checksum));
}

void prepared_starts_preserve_controls_and_loop_initialization(){
    using namespace epok;
    Fixture fixture(1);
    fixture.modulation(0)={uint16_t(14|512),16,Pitch,0,12700,0};
    fixture.zone().volume_envelope.attack=-3600;
    const sequence::Event events[]={
        {0,sequence::Parameter,0,0,0,1200},
        {0,sequence::LoopStart,0,0,0,0},
        {0,sequence::NoteOn,0,60,100,0},
        {1,sequence::NoteOff,0,60,0,0},
        {2,sequence::Bend,0,0,0,9600},
        {2,sequence::NoteOn,0,60,100,0},
        {3,sequence::NoteOff,0,60,0,0},
        {4,sequence::LoopEnd,0,0,0,0}};
    instrument::preparation::Storage<2,8,2> storage;
    auto& cache=storage.cache;
    assert(cache.prepare(fixture.view(),events,8,96) && cache.count==2);
    assert(cache.reference_count==2 && cache.event_map[2].count==1 && cache.event_map[5].count==1);
    assert(cache.references[cache.event_map[2].offset]!=cache.references[cache.event_map[5].offset]);
    sequence::Channel channel;channel.bend_range_cents=1200;
    for(auto bend:{8192,9600}){
        channel.bend=uint16_t(bend);
        const auto* seed=cache.find(0,60,100,channel);assert(seed);
        const auto expected_pitch=std::floor(4096.0*fixture.view().sample(0).rate/44100.0*
            std::pow(2.0,seed->output().pitch_cents_x100/120000.0));
        assert(std::abs(double(cache.pitches[seed-cache.states])-expected_pitch)<=1);
        assert(!seed->output().finished);
        State prepared,direct;prepared.copy_initial(*seed);
        assert(direct.start(fixture.view(),0,60,100,instrument::preparation::controls(channel))==Error::None);
        for(unsigned i=0;i<200;++i){
            const auto a=prepared.advance(1000),b=direct.advance(1000);
            assert(a.pitch_cents_x100==b.pitch_cents_x100 && a.gain_q15==b.gain_q15 && a.finished==b.finished);
        }
        assert(seed->output().gain_q15==0); // Copies cannot age the shared seed.
    }
    assert(!cache.find(0,61,100,channel));
    instrument::preparation::Storage<1> small;
    assert(!small.cache.prepare(fixture.view(),events,8,96) && !small.cache.ready);
    // Host sizing has identical deduplication without allocating voice states.
    instrument::preparation::Key keys[2];uint16_t buckets[4]{};
    instrument::preparation::Cache sizing{keys,nullptr,buckets,2,4};
    assert(sizing.prepare(fixture.view(),events,8,96) && sizing.count==cache.count);
    std::printf("instrument_preparation: %zu bytes for two distinct starts; loop/control/capacity checks passed\n",sizeof(storage));
    Fixture future(2);
    future.modulation(0)={uint16_t(14|512),16,Pitch,0,12700,0};
    future.modulation(1)={uint16_t(128|1),0,VibLfoPitch,0,50,0};
    Controls controls;
    State seed,projected,direct;
    assert(seed.start_validated(future.view(),0,60,100,controls)==Error::None);
    projected.copy_initial(seed);
    assert(direct.start(future.view(),0,60,100,controls)==Error::None);
    for(unsigned i=0;i<1000;++i){
        if(i==500){
            controls.cc[1]=127;controls.bend=9600;controls.fine_tuning=9000;
            assert(projected.update_controls(controls)==Error::None && direct.update_controls(controls)==Error::None);
        }
        const auto a=projected.advance(1000),b=direct.advance(1000);
        assert(a.pitch_cents_x100==b.pitch_cents_x100 && a.gain_q15==b.gain_q15 && a.finished==b.finished);
    }
    assert(seed.output().pitch_cents_x100==0); // Deferred controllers do not mutate the seed.
    controls.cc[1]=0;
    assert(projected.update_controls(controls)==Error::None && direct.update_controls(controls)==Error::None);
    projected.advance(UINT32_MAX);direct.advance(UINT32_MAX);
    projected.advance(1000);direct.advance(1000);
    controls.cc[1]=127;
    assert(projected.update_controls(controls)==Error::None && direct.update_controls(controls)==Error::None);
    assert(projected.output().pitch_cents_x100==direct.output().pitch_cents_x100);
}

int main() {
    static_assert(sizeof(State) < 768, "24 bounded voices must not hide dynamic storage");
    default_controls_and_pitch_contract();
    fixed_inputs_direction_amount_source_absolute_and_sum();
    source_curve_endpoints_match_the_rust_oracle();
    delay_attack_hold_and_key_scaled_decay_are_time_based();
    every_retained_runtime_destination_is_accepted();
    control_updates_recompute_a_sustaining_envelope();
    envelopes_lfo_and_release_advance_across_frame_boundaries();
    invalid_inputs_and_unsupported_destinations_fail_explicitly();
    reverb_send_is_explicit_and_controller_driven();
    independent_envelope_and_lfo_oracles();
    extreme_modulation_amounts_remain_bounded();
    release_time_control_updates_are_bounded_and_immediate();
    prepared_starts_preserve_controls_and_loop_initialization();
    benchmark_advance_24_voices_32_modulations();
    std::puts("instrument_synth: integer modulation, envelopes, LFO, tuning and errors passed");
}
