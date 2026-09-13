#include "instrument_preview.h"

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>

namespace {
void w16(uint8_t* data, uint32_t offset, uint16_t value) {
    data[offset] = uint8_t(value); data[offset + 1] = uint8_t(value >> 8);
}
void w32(uint8_t* data, uint32_t offset, uint32_t value) {
    w16(data, offset, uint16_t(value)); w16(data, offset + 2, uint16_t(value >> 16));
}

std::vector<uint8_t> bank(uint16_t zones, uint8_t program, bool until_release = false) {
    const uint32_t zone_offset = 72;
    const uint32_t modulation_offset = zone_offset + uint32_t(zones) * 144;
    const uint32_t data_offset = (modulation_offset + 63) / 64 * 64;
    const uint32_t frames = until_release ? 84 : 56;
    std::vector<uint8_t> data(data_offset + 64);
    data[0] = 'E'; data[1] = 'P'; data[2] = 'S'; data[3] = 'B';
    w16(data.data(), 4, 2); w16(data.data(), 6, 48); w16(data.data(), 8, 1); w16(data.data(), 10, zones);
    w32(data.data(), 12, 48); w32(data.data(), 16, zone_offset); w32(data.data(), 20, modulation_offset);
    w32(data.data(), 24, data_offset); w32(data.data(), 28, uint32_t(data.size())); w32(data.data(), 32, 0); w32(data.data(), 36, 2);
    w32(data.data(), 48, data_offset); w32(data.data(), 52, 64); w32(data.data(), 56, 44100); w32(data.data(), 60, frames);
    if (until_release) { w32(data.data(), 64, 28); w32(data.data(), 68, 56); }
    const uint32_t first = data_offset + 16;
    data[first + 1] = until_release ? 3 : 1;
    for (uint32_t i = 2; i < 16; ++i) data[first + i] = 0x11;
    if (until_release) {
        data[first + 16 + 1] = 1;
        for (uint32_t i = 2; i < 16; ++i) data[first + 16 + i] = 0x11;
        data[first + 32 + 1] = 7;
    } else {
        data[first + 16 + 1] = 7;
    }
    for (uint16_t zone = 0; zone < zones; ++zone) {
        const uint32_t at = zone_offset + uint32_t(zone) * 144;
        data[at + 4] = program; data[at + 6] = 0; data[at + 7] = 127; data[at + 8] = 0; data[at + 9] = 127;
        data[at + 10] = 60; data[at + 11] = 255; data[at + 12] = 255; data[at + 13] = until_release ? 3 : 0;
        for (uint32_t envelope : {at + 40, at + 72}) {
            w32(data.data(), envelope, uint32_t(-32768));
            w32(data.data(), envelope + 4, uint32_t(-32768));
            w32(data.data(), envelope + 8, uint32_t(-32768));
            w32(data.data(), envelope + 12, uint32_t(-12000));
        }
        // A one-second release keeps a released voice observable while the test
        // verifies that its UntilRelease cursor reaches the retained tail.
        w32(data.data(), at + 40 + 20, 0);
    }
    return data;
}

EpokInstrumentPcm pcm(const std::vector<float>& samples) {
    return {samples.data(), uint32_t(samples.size())};
}
epok::sequence::Event event(uint32_t tick, uint8_t op, uint8_t key = 0, uint8_t velocity = 0) {
    return {tick, op, 0, key, velocity, 0};
}

void layers_are_atomic_and_missing_zones_fail() {
    auto layers = bank(2, 0);
    std::vector<float> samples(56, .25f);
    const auto p = pcm(samples);
    const epok::sequence::Event events[] = {event(0, epok::sequence::NoteOn, 60, 100), event(10, epok::sequence::End)};
    void* preview = epok_instrument_create(events, 2, 500, 1, layers.data(), uint32_t(layers.size()), &p, 1, 44100);
    assert(preview);
    int16_t output[2]{};
    assert(epok_instrument_render(preview, output, 1) == 0);
    const auto stats = epok_instrument_stats(preview);
    assert(stats.denied == 1 && stats.physical_peak == 0 && stats.error == 0);
    epok_instrument_destroy(preview);

    auto absent = bank(1, 1);
    preview = epok_instrument_create(events, 2, 500, 24, absent.data(), uint32_t(absent.size()), &p, 1, 44100);
    assert(preview);
    assert(epok_instrument_render(preview, output, 1) == int(epok::sequence::Error::MissingInstrument));
    epok_instrument_destroy(preview);
}

void repeats_remain_independent_and_until_release_keeps_tail() {
    auto ordinary = bank(1, 0);
    std::vector<float> ordinary_pcm(56, .25f);
    const auto p = pcm(ordinary_pcm);
    const epok::sequence::Event repeated[] = {
        event(0, epok::sequence::NoteOn, 60, 100), event(0, epok::sequence::NoteOn, 60, 100),
        event(1, epok::sequence::NoteOff, 60), event(2, epok::sequence::NoteOff, 60), event(3, epok::sequence::End),
    };
    void* preview = epok_instrument_create(repeated, 5, 500, 24, ordinary.data(), uint32_t(ordinary.size()), &p, 1, 44100);
    assert(preview);
    std::vector<int16_t> output(4096 * 2);
    assert(epok_instrument_render(preview, output.data(), 4096) == 0);
    const auto repeated_stats = epok_instrument_stats(preview);
    assert(repeated_stats.logical_peak == 2 && repeated_stats.physical_peak == 2 && !repeated_stats.denied);
    epok_instrument_destroy(preview);

    auto looping = bank(1, 0, true);
    std::vector<float> loop_pcm(84, .25f);
    const auto lp = pcm(loop_pcm);
    const epok::sequence::Event release[] = {
        event(0, epok::sequence::NoteOn, 60, 100), event(75, epok::sequence::NoteOff, 60), event(150, epok::sequence::End),
    };
    preview = epok_instrument_create(release, 3, 500, 24, looping.data(), uint32_t(looping.size()), &lp, 1, 44100);
    assert(preview);
    output.assign(8000 * 2, 0);
    assert(epok_instrument_render(preview, output.data(), 4096) == 0);
    assert(epok_instrument_render(preview, output.data() + 4096 * 2, 3904) == 0);
    const auto loop_stats = epok_instrument_stats(preview);
    assert(loop_stats.sample_loops > 0 && loop_stats.loops == 0);
    assert(output[80] != 0);
    epok_instrument_destroy(preview);
}

void output_is_block_deterministic_and_bad_stream_stops() {
    auto bytes = bank(1, 0);
    std::vector<float> source(56, .25f);
    const auto p = pcm(source);
    const epok::sequence::Event complete[] = {event(0, epok::sequence::NoteOn, 60, 100), event(10, epok::sequence::End)};
    void* whole = epok_instrument_create(complete, 2, 500, 24, bytes.data(), uint32_t(bytes.size()), &p, 1, 44100);
    void* chunks = epok_instrument_create(complete, 2, 500, 24, bytes.data(), uint32_t(bytes.size()), &p, 1, 44100);
    assert(whole && chunks);
    std::vector<int16_t> expected(4096 * 2), actual(4096 * 2);
    assert(epok_instrument_render(whole, expected.data(), 4096) == 0);
    assert(epok_instrument_render(chunks, actual.data(), 1000) == 0);
    assert(epok_instrument_render(chunks, actual.data() + 2000, 3096) == 0);
    assert(expected == actual);
    epok_instrument_destroy(whole); epok_instrument_destroy(chunks);

    const epok::sequence::Event malformed[] = {event(0, epok::sequence::NoteOn, 60, 100)};
    void* bad = epok_instrument_create(malformed, 1, 500, 24, bytes.data(), uint32_t(bytes.size()), &p, 1, 44100);
    assert(bad);
    int16_t one[2]{};
    assert(epok_instrument_render(bad, one, 1) == int(epok::sequence::Error::InvalidInput));
    epok_instrument_destroy(bad);
}
} // namespace

int main() {
    layers_are_atomic_and_missing_zones_fail();
    repeats_remain_independent_and_until_release_keeps_tail();
    output_is_block_deterministic_and_bad_stream_stops();
    std::puts("instrument_preview: layered allocation, lifetime, loops and deterministic chunks passed");
}
