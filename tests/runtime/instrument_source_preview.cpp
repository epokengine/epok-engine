#include "instrument_source_preview.h"

#include <algorithm>
#include <cassert>
#include <cstdio>
#include <cmath>
#include <vector>

namespace {
EpokSourceRegion region(uint8_t program = 0, uint8_t loop_mode = 0) {
    EpokSourceRegion value{};
    value.sample = 0; value.bank = 0; value.program = program;
    value.key_hi = value.velocity_hi = 127; value.root_key = 60;
    value.fixed_key = value.fixed_velocity = 255; value.loop_mode = loop_mode;
    value.filter_cents = 13500;
    value.volume_envelope.delay = value.volume_envelope.attack = value.volume_envelope.hold = -32768;
    value.volume_envelope.decay = -12000; value.volume_envelope.release = 0;
    value.modulation_envelope.delay = value.modulation_envelope.attack = value.modulation_envelope.hold = -32768;
    value.modulation_envelope.decay = -12000; value.modulation_envelope.release = 0;
    value.modulation_envelope.sustain = 1000;
    value.modulation_lfo.delay = value.vibrato_lfo.delay = -32768;
    value.modulation_lfo.frequency = value.vibrato_lfo.frequency = -16000;
    return value;
}
EpokSourcePcm pcm(const std::vector<float>& values) { return {values.data(), uint32_t(values.size()), 44100}; }
epok::sequence::Event event(uint32_t tick, uint8_t op, uint8_t key = 0, uint8_t velocity = 0) {
    return {tick, op, 0, key, velocity, 0};
}

void source_layers_remain_atomic_and_repeated_notes_are_fifo() {
    auto left = region(); auto right = region(); right.pan = 500;
    EpokSourceRegion layers[] = {left, right};
    std::vector<float> values(96, .25f); const auto source = pcm(values);
    const epok::sequence::Event repeated[] = {
        event(0, epok::sequence::NoteOn, 60, 100), event(0, epok::sequence::NoteOn, 60, 100),
        event(4, epok::sequence::NoteOff, 60), event(8, epok::sequence::NoteOff, 60), event(12, epok::sequence::End),
    };
    void* preview = epok_source_instrument_create(repeated, 5, 500, layers, 2, &source, 1, 44100);
    assert(preview);
    std::vector<int16_t> audio(4096 * 2);
    assert(epok_source_instrument_render(preview, audio.data(), 4096) == 0);
    const auto stats = epok_source_instrument_stats(preview);
    assert(stats.logical_peak == 2 && stats.physical_peak == 4 && !stats.denied);
    epok_source_instrument_destroy(preview);
}

void no_matching_source_region_is_an_error_and_malformed_stream_stops() {
    auto selected = region(1);
    std::vector<float> values(64, .25f); const auto source = pcm(values);
    const epok::sequence::Event note[] = {event(0, epok::sequence::NoteOn, 60, 100), event(1, epok::sequence::End)};
    void* preview = epok_source_instrument_create(note, 2, 500, &selected, 1, &source, 1, 44100);
    assert(preview);
    int16_t output[2]{};
    assert(epok_source_instrument_render(preview, output, 1) == int(epok::sequence::Error::MissingInstrument));
    epok_source_instrument_destroy(preview);

    selected = region();
    preview = epok_source_instrument_create(note, 1, 500, &selected, 1, &source, 1, 44100);
    assert(preview);
    assert(epok_source_instrument_render(preview, output, 1) == int(epok::sequence::Error::InvalidInput));
    epok_source_instrument_destroy(preview);
}

void source_loop_release_filter_and_chunks_are_live_and_deterministic() {
    auto selected = region(0, 3);
    selected.loop_start = 8; selected.loop_end = 24;
    EpokSourceModulation filter{};
    filter.source = 0x0081; // CC1, unipolar linear.
    filter.amount_source = 0; // SoundFont constant.
    filter.destination = EpokSourceFilterCents;
    filter.amount = -9000;
    selected.modulations = &filter; selected.modulation_count = 1;
    std::vector<float> values(64);
    for (uint32_t i = 0; i < values.size(); ++i) values[i] = i & 1 ? .8f : -.8f;
    const auto source = pcm(values);
    const epok::sequence::Event commands[] = {
        event(0, epok::sequence::NoteOn, 60, 100), event(2, epok::sequence::Control, 1, 127),
        event(20, epok::sequence::NoteOff, 60), event(80, epok::sequence::End),
    };
    void* whole = epok_source_instrument_create(commands, 4, 500, &selected, 1, &source, 1, 44100);
    void* chunks = epok_source_instrument_create(commands, 4, 500, &selected, 1, &source, 1, 44100);
    assert(whole && chunks);
    std::vector<int16_t> expected(4096 * 2), actual(4096 * 2);
    assert(epok_source_instrument_render(whole, expected.data(), 4096) == 0);
    assert(epok_source_instrument_render(chunks, actual.data(), 999) == 0);
    assert(epok_source_instrument_render(chunks, actual.data() + 1998, 3097) == 0);
    assert(expected == actual);
    const epok::sequence::Event dry_commands[] = {
        event(0, epok::sequence::NoteOn, 60, 100), event(20, epok::sequence::NoteOff, 60), event(80, epok::sequence::End),
    };
    void* dry = epok_source_instrument_create(dry_commands, 3, 500, &selected, 1, &source, 1, 44100);
    assert(dry);
    std::vector<int16_t> dry_audio(4096 * 2);
    assert(epok_source_instrument_render(dry, dry_audio.data(), 4096) == 0);
    assert(expected != dry_audio);
    epok_source_instrument_destroy(dry);
    const auto stats = epok_source_instrument_stats(whole);
    assert(stats.sample_loops > 0 && stats.loops == 0);
    assert(std::any_of(expected.begin(), expected.end(), [](int16_t value) { return value != 0; }));
    // Note-off at 20 ms leaves the UntilRelease loop and reaches its raw tail.
    assert(std::any_of(expected.begin() + 900 * 2, expected.begin() + 940 * 2, [](int16_t value) { return value != 0; }));
    epok_source_instrument_destroy(whole); epok_source_instrument_destroy(chunks);
}
void source_filter_cutoff_uses_output_rate_and_comparison_gain_precedes_clipping() {
    const auto amplitude=[](uint32_t input_rate,int cutoff,float gain,int resonance=0){
        auto selected=region();selected.filter_cents=cutoff;selected.filter_centibels=resonance;
        std::vector<float> values(input_rate);
        for(uint32_t i=0;i<input_rate;++i)values[i]=float(.25*std::sin(2*3.141592653589793*1000*i/input_rate));
        EpokSourcePcm source{values.data(),input_rate,input_rate};
        const epok::sequence::Event commands[]={event(0,epok::sequence::NoteOn,60,100),event(1000,epok::sequence::End)};
        void* p=epok_source_instrument_create(commands,2,500,&selected,1,&source,1,44100);assert(p);
        assert(epok_source_instrument_set_gain(p,gain));
        std::vector<int16_t> output(8192);assert(epok_source_instrument_render(p,output.data(),4096)==0);
        double energy=0;for(int i=2048;i<4096;++i)energy+=double(output[i*2])*output[i*2];
        epok_source_instrument_destroy(p);return std::sqrt(energy/2048);
    };
    // A Butterworth pole pair is -3.01 dB at its 1 kHz cutoff. The source
    // sampling frequency cannot halve that physical cutoff after resampling.
    const double half=amplitude(22050,8314,1)/amplitude(22050,13500,1);
    const double full=amplitude(44100,8314,1)/amplitude(44100,13500,1);
    assert(std::abs(half-std::sqrt(.5))<.025 && std::abs(full-std::sqrt(.5))<.025 && std::abs(half-full)<.01);
    const double scaled=amplitude(22050,13500,.5f)/amplitude(22050,13500,1);
    assert(std::abs(scaled-.5)<.002);
    const double resonant=amplitude(44100,13500,1,200)/amplitude(44100,13500,1);
    assert(std::abs(resonant-std::pow(10.,-10./20.))<.01);
}
} // namespace

int main() {
    source_layers_remain_atomic_and_repeated_notes_are_fifo();
    no_matching_source_region_is_an_error_and_malformed_stream_stops();
    source_loop_release_filter_and_chunks_are_live_and_deterministic();
    source_filter_cutoff_uses_output_rate_and_comparison_gain_precedes_clipping();
    std::puts("instrument_source_preview: source PCM, filters, loops and deterministic chunks passed");
}
