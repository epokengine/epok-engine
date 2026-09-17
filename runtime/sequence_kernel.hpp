#pragma once
// Portable musical state machine. No SDK, allocation, device address or gameplay callback.
// Event ordering and integer PPQN time are identical in host audition and console playback.
#include <cstdint>
#include <cstddef>

namespace epok::sequence {
// Values 0..8 are the EPSQ v1 wire contract.  Keep them stable.
enum Op : uint8_t { NoteOn, NoteOff, Program, Control, Bend, Tempo, LoopStart, LoopEnd, End, Parameter, BankProgram };
struct Event { uint32_t tick; uint8_t op, channel, a, b; uint32_t value; };
static_assert(sizeof(Event) == 12);
struct alignas(4) Channel {
    uint8_t program = 0, volume = 100, pan = 64, expression = 127, modulation = 0, sustain = 0, sostenuto = 0, coarse_tuning = 64;
    uint16_t bank = 0, bend = 8192, bend_range_cents = 200, fine_tuning = 8192;
    uint8_t reverb = 0;
    bool reverb_set = false;
    uint16_t reserved = 0; // Complete the aligned snapshot word deterministically.
    int64_t pitch_cents100() const {
        const int64_t numerator = int64_t(int(bend) - 8192) * bend_range_cents * 100 +
            int64_t(int(fine_tuning) - 8192) * 10000;
        return numerator / 8192 + int64_t(int(coarse_tuning) - 64) * 10000;
    }
};
inline void copy_channels(Channel (&destination)[16],const Channel (&source)[16]){
#if defined(__mips__)
    // The SDK's generic memcpy copies bytes. These aligned fixed-size snapshots
    // are restored inside the IRQ at a loop boundary, so copy complete words.
    static_assert(sizeof(Channel)%4==0 && alignof(Channel)>=4);
    auto* to=static_cast<unsigned char*>(__builtin_assume_aligned(destination,4));
    const auto* from=static_cast<const unsigned char*>(__builtin_assume_aligned(source,4));
    for(size_t offset=0;offset<sizeof(destination);offset+=4){
        uint32_t word;__builtin_memcpy(&word,from+offset,4);__builtin_memcpy(to+offset,&word,4);
    }
#else
    for(unsigned i=0;i<16;++i)destination[i]=source[i];
#endif
}
struct Note {
    uint64_t age = 0; uint8_t channel = 0, key = 0, velocity = 0;
    bool active = false, down = false, held = false, sustain_held = false, sostenuto_held = false, releasing = false;
    uint16_t key_ordinal=0;
    uint8_t previous_key_note=255,next_key_note=255;
    void initialize(uint64_t serial,uint8_t input_channel,uint8_t input_key,uint8_t input_velocity,bool enabled=true){
        // Direct field stores avoid the PSX SDK's byte-wise aggregate copy on
        // each note admission and retirement. All logical fields are reset.
        age=serial;channel=input_channel;key=input_key;velocity=input_velocity;
        active=down=enabled;held=sustain_held=sostenuto_held=releasing=false;
        key_ordinal=0;previous_key_note=next_key_note=255;
    }
};
enum class Error : uint8_t { None, InvalidInput, ServiceOverflow, MissingInstrument, ClockOverflow };

inline bool valid_event(const Event& e, bool extended = true) {
    if (e.channel > 15 || e.a > 127 || e.b > 127) return false;
    if (!extended && e.op > End) return false;
    switch (e.op) {
        case NoteOn: case NoteOff: case Program: case LoopStart: case LoopEnd: case End: return true;
        case Tempo: return e.value && e.value <= 0xffffff;
        case Bend: return e.value <= 16383;
        case Control:
            if (e.a == 7 || e.a == 10 || e.a == 11 || e.a == 64) return true;
            if (!extended) return false;
            if (e.a == 1 || e.a == 66 || e.a == 120 || e.a == 121 || e.a == 123) return true;
            return e.a == 91 || ((e.a == 92 || e.a == 93 || e.a == 95) && e.b == 0);
        case Parameter:
            return extended && e.b == 0 && ((e.a == 0 && e.value <= 12827) || (e.a == 1 && e.value <= 16383) ||
                (e.a == 2 && e.value <= 127));
        case BankProgram: return extended && e.b == 0 && e.value <= 16383;
        default: return false;
    }
}
struct Kernel {
    static constexpr uint16_t MaxVoices = 128;
    static constexpr uint32_t MaxEvents = 65536, MaxServiceEvents = 1024, MaxServiceCommands = 4096;
    Channel channels[16]{}, loop_channels[16]{};
    Note notes[MaxVoices]{};
    struct KeyLifetime {uint16_t issued=0,consumed=0;uint8_t head=255,tail=255;};
    KeyLifetime key_lifetimes[16][128]{};
    uint32_t pending_keys[16][4]{};
    uint32_t free_notes[4]{};
    uint16_t retired_channels = 0;
    uint16_t active_notes = 0;
    uint8_t channel_notes[16]{};
    const Event* events = nullptr;
    uint32_t count = 0, cursor = 0, tick = 0, tempo = 500000, loop_cursor = 0, loop_tick = 0, loop_tempo = 500000;
    uint64_t clock = 0, event_time = 0, age = 0;
    uint16_t ppqn = 0, limit = 16;
    uint32_t steals = 0, peak = 0, loops = 0, commands = 0;
    bool running = false, has_loop = false;
    Error error = Error::None;

    bool begin(const Event* data, uint32_t size, uint16_t division, uint16_t voices) {
        *this = Kernel{};
        if (!data || !size || size > MaxEvents || !division || division > 32767 || !voices || voices > MaxVoices) return fail(Error::InvalidInput);
        for (uint32_t i = 0; i < size; ++i) {
            const auto& e = data[i];
            if ((i && e.tick < data[i - 1].tick) || !valid_event(e)) return fail(Error::InvalidInput);
        }
        events = data; count = size; ppqn = division; limit = voices; running = true;
        reset_free_notes();
        return true;
    }
    // Only after startup validation of an immutable resident payload. Avoid rescanning
    // thousands of events in an IRQ. Fixed storage is reset without a large stack temporary.
    void begin_validated(const Event* data, uint32_t size, uint16_t division, uint16_t voices) {
        for (auto& c : channels) c = Channel{};
        for (auto& c : loop_channels) c = Channel{};
        for (auto& n : notes) n = Note{};
        clear_retired();
        events = data; count = size; ppqn = division; limit = voices;
        reset_free_notes();
        cursor = tick = loop_cursor = loop_tick = steals = peak = loops = commands = 0;
        active_notes=0;
        for(auto& channel_count:channel_notes)channel_count=0;
        tempo = loop_tempo = 500000; clock = event_time = age = 0;
        running = true; has_loop = false; error = Error::None;
    }
    bool fail(Error e) { error = e; running = false; return false; }
    static unsigned first_bit(uint32_t mask){
        unsigned bit=0;
        if(!(mask&0xffff)){mask>>=16;bit+=16;}
        if(!(mask&0xff)){mask>>=8;bit+=8;}
        if(!(mask&15)){mask>>=4;bit+=4;}
        if(!(mask&3)){mask>>=2;bit+=2;}
        return bit+(!(mask&1));
    }
    void reset_free_notes(){
        for(unsigned word=0;word<4;++word){
            const unsigned available=limit>word*32?limit-word*32:0;
            free_notes[word]=available>=32?UINT32_MAX:(uint32_t(1)<<available)-1;
        }
    }
    void unlink_key_note(uint16_t slot){
        auto& note=notes[slot];
        auto& key=key_lifetimes[note.channel][note.key];
        if(note.previous_key_note!=255)notes[note.previous_key_note].next_key_note=note.next_key_note;
        else key.head=note.next_key_note;
        if(note.next_key_note!=255)notes[note.next_key_note].previous_key_note=note.previous_key_note;
        else key.tail=note.previous_key_note;
        note.previous_key_note=note.next_key_note=255;
    }
    void clear_retired() {
        // Discard only outstanding key lifetimes at a loop/stop. Consumed keys
        // need no clearing, and their serials may wrap without losing order.
        for(uint8_t channel=0;channel<16;++channel)if(retired_channels&(1u<<channel)){
            for(unsigned word=0;word<4;++word){
                uint32_t mask=pending_keys[channel][word];
                while(mask){
                    const unsigned bit=first_bit(mask);
                    auto& lifetime=key_lifetimes[channel][word*32+bit];lifetime.consumed=lifetime.issued;
                    lifetime.head=lifetime.tail=255;
                    mask&=mask-1;
                }
                pending_keys[channel][word]=0;
            }
        }
        retired_channels=0;
    }
    void retire(uint16_t slot) {
        if (slot >= limit) return;
        const auto& n = notes[slot];
        if(!n.active)return;
        if(n.down)unlink_key_note(slot);
        --active_notes;--channel_notes[n.channel];
        free_notes[slot/32]|=uint32_t(1)<<(slot%32);
        notes[slot].initialize(0,0,0,0,false);
    }
    uint32_t active() const { return active_notes; }
    bool command() {
        if (commands == MaxServiceCommands) return fail(Error::ServiceOverflow);
        ++commands; return true;
    }
    template<class Backend> void cut_all(Backend& backend, bool budgeted = false) {
        // Emergency stop always retires at most MaxVoices after a budget fault.
        for (uint16_t i = 0; i < limit && active_notes; ++i) if (notes[i].active) {
            if (budgeted && !command()) return;
            backend.cut(i); retire(i);
        }
        clear_retired();
    }
    template<class Backend> void stop(Backend& backend) { running = false; cut_all(backend); }
    template<class Backend> void release(uint16_t slot, Backend& backend) {
        if (notes[slot].releasing) return;
        if (!command()) return;
        if(notes[slot].down)unlink_key_note(slot);
        notes[slot].down = notes[slot].held = notes[slot].sustain_held = notes[slot].sostenuto_held = false;
        notes[slot].releasing = true;
        backend.release(slot);
    }
    template<class Backend> void release_if_unheld(uint16_t slot, Backend& backend) {
        auto& note = notes[slot];
        note.held = note.sustain_held || note.sostenuto_held;
        if (!note.down && !note.held) release(slot, backend);
    }
    template<class Backend> void release_pedal(uint8_t channel, bool sustain, Backend& backend) {
        for (uint16_t i = 0; i < limit; ++i) {
            auto& note = notes[i];
            if (!note.active || note.channel != channel) continue;
            if (sustain) note.sustain_held = false; else note.sostenuto_held = false;
            note.held = note.sustain_held || note.sostenuto_held;
            if (!note.down && !note.held) release(i, backend);
            if (!running) return;
        }
    }
    template<class Backend> void all_notes_off(uint8_t channel, Backend& backend) {
        for (uint16_t i = 0; i < limit; ++i) {
            auto& note = notes[i];
            if (!note.active || note.channel != channel || !note.down) continue;
            unlink_key_note(i);note.down = false; note.sustain_held = channels[channel].sustain;
            release_if_unheld(i, backend);
            if (!running) return;
        }
    }
    template<class Backend> void all_sound_off(uint8_t channel, Backend& backend) {
        for (uint16_t i = 0; i < limit; ++i) if (notes[i].active && notes[i].channel == channel) {
            if (!command()) return;
            backend.cut(i); retire(i);
        }
    }
    template<class Backend>
#if defined(__mips__) && defined(__GNUC__)
    // Dense same-tick dispatch is an IRQ hot path on the PSX. Its event and
    // command bounds are unchanged; host builds keep their normal flags.
    __attribute__((optimize("O3")))
#endif
    void advance(uint32_t microseconds, Backend& backend) {
        if (!running) return;
        const uint64_t increment = uint64_t(microseconds) * ppqn;
        if (clock > UINT64_MAX - increment) { fail(Error::ClockOverflow); cut_all(backend); return; }
        clock += increment;
        uint32_t processed = 0; commands = 0;
        while (running && cursor < count) {
            const auto& e = events[cursor];
            auto due=event_time;
            if(e.tick!=tick){
                const uint64_t delta = uint64_t(e.tick - tick) * tempo;
                if (event_time > UINT64_MAX - delta) { fail(Error::ClockOverflow); break; }
                due+=delta;
                if (due > clock) break;
            }
            // Events at the current tick share the already-reached deadline.
            // Avoid rebuilding 64-bit time for every member of a chord/setup.
            // A hardware backend may distribute a dense chord over adjacent
            // services. Keep the absolute musical clock and event order intact;
            // the deferred event retains its original deadline, including loops.
            if constexpr(requires { backend.defer_note(e,cursor); }) {
                if(e.op==NoteOn && backend.defer_note(e,cursor))break;
            }
            if (++processed > MaxServiceEvents || commands > MaxServiceCommands) { fail(Error::ServiceOverflow); break; }
            event_time = due; tick = e.tick; ++cursor;
            if constexpr(requires { backend.event(e); })backend.event(e);
            auto& channel = channels[e.channel];
            switch (e.op) {
                case NoteOn: {
                    auto& lifetime=key_lifetimes[e.channel][e.a];
                    if(uint16_t(lifetime.issued-lifetime.consumed)==UINT16_MAX){fail(Error::ServiceOverflow);break;}
                    uint16_t slot = limit;
                    for(unsigned word=0;word<4;++word)if(free_notes[word]){slot=uint16_t(word*32+first_bit(free_notes[word]));break;}
                    if (slot == limit) {
                        slot = 0;
                        for (uint16_t i = 1; i < limit; ++i) if (notes[i].age < notes[slot].age) slot = i;
                        if (!command()) break;
                        backend.cut(slot); retire(slot); ++steals;
                    }
                    if (!command()) break;
                    notes[slot].initialize(++age,e.channel,e.a,e.b);
                    free_notes[slot/32]&=~(uint32_t(1)<<(slot%32));
                    notes[slot].key_ordinal=++lifetime.issued;
                    notes[slot].previous_key_note=lifetime.tail;
                    if(lifetime.tail!=255)notes[lifetime.tail].next_key_note=uint8_t(slot);
                    else lifetime.head=uint8_t(slot);
                    lifetime.tail=uint8_t(slot);
                    pending_keys[e.channel][e.a/32]|=uint32_t(1)<<(e.a%32);
                    retired_channels|=uint16_t(1u<<e.channel);
                    ++active_notes;
                    ++channel_notes[e.channel];
                    if (!backend.start(slot, notes[slot], channel)) { fail(Error::MissingInstrument); break; }
                    const auto n = active(); if (n > peak) peak = n;
                    break;
                }
                case NoteOff: {
                    auto& lifetime=key_lifetimes[e.channel][e.a];
                    if(lifetime.consumed==lifetime.issued)break;
                    const auto ordinal=++lifetime.consumed;
                    if(lifetime.consumed==lifetime.issued)pending_keys[e.channel][e.a/32]&=~(uint32_t(1)<<(e.a%32));
                    const uint16_t slot=lifetime.head;
                    // Retired serials are tombstones. The first remaining live
                    // key-down node is the only possible FIFO match.
                    if (slot!=255 && notes[slot].key_ordinal==ordinal) {
                        unlink_key_note(slot);
                        notes[slot].down = false;
                        notes[slot].sustain_held = channel.sustain;
                        release_if_unheld(slot, backend);
                    }
                    break;
                }
                case Program: channel.program = e.a; channel.bank = 0; break;
                case BankProgram: channel.program = e.a; channel.bank = uint16_t(e.value); break;
                case Bend:
                    if(channel.bend!=e.value){channel.bend=uint16_t(e.value);update(e.channel,backend);}
                    break;
                case Parameter:
                    if (e.a == 0) {if(channel.bend_range_cents==e.value)break;channel.bend_range_cents = uint16_t(e.value);}
                    else if (e.a == 1) {if(channel.fine_tuning==e.value)break;channel.fine_tuning = uint16_t(e.value);}
                    else {if(channel.coarse_tuning==e.value)break;channel.coarse_tuning = uint8_t(e.value);}
                    update(e.channel, backend); break;
                case Control:
                    if (e.a == 7) {if(channel.volume==e.b)break;channel.volume = e.b;}
                    else if (e.a == 10) {if(channel.pan==e.b)break;channel.pan = e.b;}
                    else if (e.a == 11) {if(channel.expression==e.b)break;channel.expression = e.b;}
                    else if (e.a == 1) {if(channel.modulation==e.b)break;channel.modulation = e.b;}
                    else if (e.a == 91) {if(channel.reverb==e.b && channel.reverb_set)break;channel.reverb = e.b; channel.reverb_set = true; }
                    else if (e.a == 64) {
                        const bool was_on = channel.sustain;
                        channel.sustain = e.b >= 64;
                        if(was_on==bool(channel.sustain))break;
                        if (was_on && !channel.sustain) release_pedal(e.channel, true, backend);
                    } else if (e.a == 66) {
                        const bool was_on = channel.sostenuto;
                        channel.sostenuto = e.b >= 64;
                        if(was_on==bool(channel.sostenuto))break;
                        if (!was_on && channel.sostenuto) {
                            for (uint16_t i = 0; i < limit; ++i) {
                                if (notes[i].active && notes[i].channel == e.channel && notes[i].down) notes[i].sostenuto_held = true;
                            }
                        } else if (was_on && !channel.sostenuto) {
                            release_pedal(e.channel, false, backend);
                        }
                        update(e.channel, backend);break;
                    } else if (e.a == 120) { all_sound_off(e.channel, backend); break;
                    } else if (e.a == 123) { all_notes_off(e.channel, backend); break;
                    } else if (e.a == 121) {
                        channel.expression = 127; channel.bend = 8192; channel.modulation = 0;
                        const bool sustain = channel.sustain, sostenuto = channel.sostenuto;
                        channel.sustain = channel.sostenuto = 0;
                        if (sustain || sostenuto) release_pedal(e.channel, true, backend);
                        if (running && sostenuto) release_pedal(e.channel, false, backend);
                    } else if (e.a == 92 || e.a == 93 || e.a == 95) {
                        if (e.b) fail(Error::InvalidInput);
                        break;
                    } else { fail(Error::InvalidInput); break; }
                    update(e.channel, backend); break;
                case Tempo: tempo = e.value; break;
                case LoopStart:
                    has_loop = true; loop_cursor = cursor; loop_tick = tick; loop_tempo = tempo;
                    copy_channels(loop_channels,channels);
                    break;
                case LoopEnd:
                    if (!has_loop || tick <= loop_tick) { fail(Error::InvalidInput); break; }
                    // A loop boundary cuts tails and restores state from immediately before loop_start.
                    cut_all(backend, true);
                    if (!running) break;
                    copy_channels(channels,loop_channels);
                    cursor = loop_cursor; tick = loop_tick; tempo = loop_tempo; ++loops;
                    break;
                case End:
                    for (uint16_t i = 0; i < limit; ++i) if (notes[i].active) release(i, backend);
                    running = false; break;
            }
        }
        if (running && cursor == count) fail(Error::InvalidInput); // Every payload has an explicit end.
        if (error != Error::None) cut_all(backend);
    }
    template<class Backend> void update(uint8_t channel, Backend& backend) {
        if(!channel_notes[channel])return;
        uint16_t remaining=channel_notes[channel];
        for (uint16_t i = 0; i < limit && remaining; ++i) if (notes[i].active && notes[i].channel==channel) {
            --remaining;
            if (!command()) return;
            backend.update(i, channels[channel]);
        }
    }
};
}
