#include "sequence_kernel.hpp"
#include <cassert>
#include <cmath>
#include <cstdio>
#include <vector>
using namespace epok::sequence;
struct Backend {
    unsigned starts = 0, releases = 0, cuts = 0, updates = 0;
    std::vector<unsigned> keys, programs, banks;
    std::vector<int64_t> pitch;
    bool start(uint16_t, const Note& n, const Channel& c) { ++starts; keys.push_back(n.key); programs.push_back(c.program); banks.push_back(c.bank); return !c.bank; }
    void release(uint16_t) { ++releases; }
    void cut(uint16_t) { ++cuts; }
    void update(uint16_t, const Channel& c) { ++updates; pitch.push_back(c.pitch_cents100()); }
};
struct SlicedBackend : Backend {
    Kernel* kernel=nullptr;
    unsigned remaining=0;
    std::vector<uint64_t> deadlines, serviced;
    bool defer_note(const Event&,uint32_t){if(!remaining)return true;--remaining;return false;}
    bool start(uint16_t index,const Note& note,const Channel& channel){
        deadlines.push_back(kernel->event_time);serviced.push_back(kernel->clock);
        return Backend::start(index,note,channel);
    }
};
struct LoopTrace {
    Kernel* kernel=nullptr;
    std::vector<uint64_t> deadlines;
    std::vector<int64_t> pitches;
    std::vector<unsigned> values;
    bool start(uint16_t,const Note& note,const Channel& c){
        deadlines.push_back(kernel->event_time);pitches.push_back(c.pitch_cents100());
        values.push_back(note.key|(unsigned(c.volume)<<8)|(unsigned(c.sustain)<<16)|
            (unsigned(c.program)<<17)|(unsigned(c.reverb_set)<<24));return true;
    }
    void cut(uint16_t){} void release(uint16_t){} void update(uint16_t,const Channel&){}
};
int main() {
    {
        const Event original[]={{0,LoopStart,0,0,0,0},{0,Tempo,0,0,0,400000},
            {0,Parameter,0,0,0,1200},{0,Control,0,64,127,0},{0,Control,0,91,0,0},{0,Program,0,5,0,0},
            {0,NoteOn,0,60,100,0},{0,Control,0,7,80,0},{1,Bend,0,0,0,9600},
            {1,NoteOn,0,64,100,0},{2,Control,0,64,0,0},{3,LoopEnd,0,0,0,0}};
        std::vector<Event> cooked(std::begin(original),std::end(original));
        // Known equivalent fixture: capture after the five initial assignments.
        std::rotate(cooked.begin(),cooked.begin()+1,cooked.begin()+6);
        Kernel a,b;LoopTrace first{&a},second{&b};
        assert(a.begin(original,12,1000,16) && b.begin(cooked.data(),12,1000,16));
        for(unsigned n=0;n<100;++n){a.advance(137,first);b.advance(137,second);}
        assert(a.loops>=10 && a.loops==b.loops);
        assert(first.deadlines==second.deadlines && first.pitches==second.pitches && first.values==second.values);
        assert(first.deadlines[0]==0 && first.deadlines[1]==400000 && first.deadlines[2]==1200000);
        assert(first.pitches[0]==0 && first.pitches[1]==20625 && first.pitches[2]==0);
    }
    const Event song[] = {
        {0, Program, 0, 5, 0, 0}, {0, Tempo, 0, 0, 0, 500000},
        {0, NoteOn, 0, 60, 100, 0}, {48, Control, 0, 64, 127, 0},
        {96, NoteOff, 0, 60, 0, 0}, {96, Tempo, 0, 0, 0, 250000},
        {192, Control, 0, 64, 0, 0}, {192, End, 0, 0, 0, 0}
    };
    Kernel k; Backend b;
    assert(k.begin(song, 8, 96, 16)); k.advance(0, b);
    assert(b.starts == 1 && b.programs[0] == 5);
    k.advance(499999, b); assert(k.notes[0].down && b.releases == 0);
    k.advance(1, b); assert(k.notes[0].held && !k.notes[0].down && b.releases == 0);
    k.advance(249999, b); assert(k.running && b.releases == 0);
    k.advance(1, b); assert(!k.running && b.releases >= 1 && k.error == Error::None);
    k.stop(b); assert(k.active() == 0 && b.cuts == 1);

    // Resolved RPN values use hundredths of a cent and update notes already sounding.
    Channel parameter_oracle; parameter_oracle.bend_range_cents = 1200; parameter_oracle.bend = 8832;
    assert(parameter_oracle.pitch_cents100() == 9375);
    parameter_oracle.bend = 9600; assert(parameter_oracle.pitch_cents100() == 20625);
    parameter_oracle.bend = 8192; parameter_oracle.coarse_tuning = 76;
    assert(parameter_oracle.pitch_cents100() == 120000 && std::abs(std::pow(2.0, double(parameter_oracle.pitch_cents100()) / 120000.0) - 2.0) < 1e-12);
    const Event parameters[] = {{0, NoteOn, 0, 60, 100, 0}, {1, Parameter, 0, 0, 0, 1200},
        {2, Bend, 0, 0, 0, 8832}, {3, Parameter, 0, 1, 0, 16383}, {4, Parameter, 0, 2, 0, 76}, {5, End, 0, 0, 0, 0}};
    assert(k.begin(parameters, 6, 1000, 1)); b = Backend{}; k.advance(10000, b);
    assert(b.updates == 4 && b.pitch[1] == 9375 && b.pitch[2] > b.pitch[1] && b.pitch[3] > b.pitch[2]);

    // Render workload/chunk size cannot alter tick deadlines or simultaneous ordering.
    assert(k.begin(song, 8, 96, 16)); b = Backend{}; k.advance(750000, b);
    Kernel fine; Backend f; assert(fine.begin(song, 8, 96, 16));
    for (unsigned i = 0; i < 750; ++i) fine.advance(1000, f);
    assert(f.keys == b.keys && f.programs == b.programs && fine.event_time == k.event_time);

    const Event loop[] = {{0, Program, 0, 3, 0, 0}, {0, LoopStart, 0, 0, 0, 0}, {0, NoteOn, 0, 60, 127, 0},
        {1, Program, 0, 9, 0, 0}, {96, LoopEnd, 0, 0, 0, 0}};
    assert(k.begin(loop, 5, 96, 1)); b = Backend{};
    k.advance(1000000, b);
    assert(k.loops == 2 && b.starts == 3 && b.cuts == 2 && b.programs == std::vector<unsigned>({3,3,3}));

    // Slicing a chord retains its event order and absolute musical deadlines.
    // Delaying admission must not accumulate into loop drift.
    const Event sliced[]={{0,LoopStart,0,0,0,0},{0,NoteOn,0,60,100,0},
        {0,NoteOn,0,61,100,0},{0,Program,0,7,0,0},{0,NoteOn,0,62,100,0},
        {0,NoteOn,0,63,100,0},{10,LoopEnd,0,0,0,0}};
    SlicedBackend sliced_backend;sliced_backend.kernel=&k;
    assert(k.begin(sliced,7,1000,4));
    for(unsigned ms=0;ms<=20;++ms){
        sliced_backend.remaining=2;k.advance(ms?1000:0,sliced_backend);
    }
    assert(k.loops==4 && k.clock==20000000 && sliced_backend.starts==18);
    for(unsigned n=0;n<sliced_backend.starts;++n){
        assert(sliced_backend.keys[n]==60+n%4);
        assert(sliced_backend.programs[n]==(n%4<2?0:7));
        assert(sliced_backend.deadlines[n]==uint64_t(n/4)*5000000);
        assert(sliced_backend.serviced[n]-sliced_backend.deadlines[n]==(n%4<2?0:1000000));
    }

    // Loop snapshots include resolved tuning and bank/program state.
    const Event parameter_loop[] = {{0, BankProgram, 0, 4, 0, 0}, {0, Parameter, 0, 0, 0, 1200}, {0, LoopStart, 0, 0, 0, 0},
        {0, NoteOn, 0, 60, 127, 0}, {1, BankProgram, 0, 9, 0, 0}, {1, Parameter, 0, 2, 0, 76}, {2, LoopEnd, 0, 0, 0, 0}};
    assert(k.begin(parameter_loop, 7, 1000, 1)); b = Backend{}; k.advance(10000, b);
    assert(k.loops >= 2 && b.programs[0] == 4 && b.programs[1] == 4 && b.pitch.size() >= 2);

    // Bank identity is accepted by EPSQ v2 but cannot silently select bank 0 in a v1 backend.
    const Event missing_bank[] = {{0, BankProgram, 0, 4, 0, 1}, {0, NoteOn, 0, 60, 100, 0}, {1, End, 0, 0, 0, 0}};
    assert(k.begin(missing_bank, 3, 1000, 1)); b = Backend{}; k.advance(0, b);
    assert(k.error == Error::MissingInstrument && b.starts == 1 && b.banks[0] == 1);

    // Sostenuto captures only notes that are down when the pedal is depressed; reset releases pedal tails once.
    const Event pedals[] = {{0, NoteOn, 0, 60, 100, 0}, {1, Control, 0, 66, 127, 0}, {2, NoteOn, 0, 62, 100, 0},
        {3, NoteOff, 0, 60, 0, 0}, {4, NoteOff, 0, 62, 0, 0}, {5, Control, 0, 64, 127, 0},
        {6, Control, 0, 121, 0, 0}, {7, Control, 0, 123, 0, 0}, {8, Control, 0, 120, 0, 0}, {9, End, 0, 0, 0, 0}};
    assert(k.begin(pedals, 10, 1000, 2)); b = Backend{}; k.advance(10000, b);
    assert(b.releases == 2 && b.cuts == 2 && !k.active() && k.channels[0].expression == 127 && !k.channels[0].sustain && !k.channels[0].sostenuto);

    // Reset and pedal-up clear sostenuto latches even while a key is still down.
    const Event reset_while_down[] = {{0, NoteOn, 0, 64, 100, 0}, {1, Control, 0, 66, 127, 0}, {2, Control, 0, 64, 127, 0},
        {3, Control, 0, 121, 0, 0}, {4, NoteOff, 0, 64, 0, 0}, {10, End, 0, 0, 0, 0}};
    assert(k.begin(reset_while_down, 6, 1000, 1)); b = Backend{}; k.advance(3000, b);
    assert(b.releases == 1 && !k.notes[0].down && !k.notes[0].sustain_held && !k.notes[0].sostenuto_held);

    // Pedal-up clears a captured sostenuto note even if its key is still down.
    const Event sostenuto_up_while_down[] = {{0, NoteOn, 0, 65, 100, 0}, {1, Control, 0, 66, 127, 0},
        {2, Control, 0, 66, 0, 0}, {3, NoteOff, 0, 65, 0, 0}, {10, End, 0, 0, 0, 0}};
    assert(k.begin(sostenuto_up_while_down, 5, 1000, 1)); b = Backend{}; k.advance(3000, b);
    assert(b.releases == 1 && !k.notes[0].down && !k.notes[0].sostenuto_held);

    // A stolen/naturally retired note's late NoteOff cannot release a replacement of the same key.
    const Event stolen[] = {{0, NoteOn, 0, 60, 127, 0}, {1, NoteOn, 0, 60, 127, 0},
        {2, NoteOff, 0, 60, 0, 0}, {3, NoteOff, 0, 60, 0, 0}, {4, End, 0, 0, 0, 0}};
    assert(k.begin(stolen, 5, 100, 1)); b = Backend{};
    k.advance(10000, b); assert(k.steals == 1 && b.releases == 0 && k.notes[0].down);
    k.advance(5000, b); assert(b.releases == 1 && !k.notes[0].down);
    k.stop(b); assert(k.active() == 0);

    // A younger repeated key may finish first (shorter sample/velocity zone).
    // Its retirement must not consume the older still-held note's Note Off.
    const Event younger_first[]={{0,NoteOn,0,60,100,0},{1,NoteOn,0,60,50,0},
        {2,NoteOff,0,60,0,0},{3,NoteOff,0,60,0,0},{4,End,0,0,0,0}};
    b=Backend{};assert(k.begin(younger_first,5,1000,2));k.advance(500,b);
    k.retire(1);k.advance(500,b);
    assert(b.releases==1 && !k.notes[0].down);
    k.advance(500,b);assert(b.releases==1);
    const Event middle_first[]={{0,NoteOn,0,60,100,0},{1,NoteOn,0,60,80,0},{2,NoteOn,0,60,50,0},
        {3,NoteOff,0,60,0,0},{4,NoteOff,0,60,0,0},{5,NoteOff,0,60,0,0},{6,End,0,0,0,0}};
    b=Backend{};assert(k.begin(middle_first,7,1000,3));k.advance(1000,b);k.retire(1);
    k.advance(500,b);assert(b.releases==1 && !k.notes[0].down && k.notes[2].down);
    k.advance(500,b);assert(b.releases==1 && k.notes[2].down);
    k.advance(500,b);assert(b.releases==2 && k.key_lifetimes[0][60].head==255);

    const Event serial_loop[]={{0,LoopStart,0,0,0,0},{0,NoteOn,0,60,100,0},
        {1,NoteOff,0,60,0,0},{2,LoopEnd,0,0,0,0}};
    b=Backend{};assert(k.begin(serial_loop,4,1000,1));
    for(unsigned i=0;i<70000;++i)k.advance(1000,b);
    assert(k.error==Error::None && b.starts==70001 && b.releases==70000);

    std::vector<Event> dense(1025, Event{0, Program, 0, 0, 0, 0});
    dense.push_back(Event{1, End, 0, 0, 0, 0});
    assert(k.begin(dense.data(), unsigned(dense.size()), 96, 16));
    k.advance(0, b); assert(k.error == Error::ServiceOverflow && !k.running && k.active() == 0);
    dense.clear();
    for(unsigned i=0;i<128;++i)dense.push_back(Event{0,NoteOn,0,uint8_t(i),100,0});
    for(unsigned i=0;i<40;++i)dense.push_back(Event{0,Control,0,7,uint8_t(i&1?100:101),0});
    dense.push_back(Event{1,End,0,0,0,0});
    b=Backend{};assert(k.begin(dense.data(),unsigned(dense.size()),96,128));k.advance(0,b);
    assert(k.error==Error::ServiceOverflow && !k.active());
    assert(b.starts+b.releases+b.updates==Kernel::MaxServiceCommands);
    assert(b.cuts==128); // Bounded emergency retirement after the command budget.
    // Repeated initialization messages keep their stream position, without
    // rebuilding active voices when the effective control value did not change.
    const Event unchanged[]={{0,NoteOn,0,60,100,0},{1,Bend,0,0,0,8192},
        {1,Parameter,0,0,0,200},{1,Parameter,0,1,0,8192},{1,Parameter,0,2,0,64},
        {1,Control,0,7,100,0},{1,Control,0,64,0,0},{1,Control,0,66,0,0},{10,End,0,0,0,0}};
    b=Backend{};assert(k.begin(unchanged,9,1000,1));k.advance(1000,b);
    assert(b.updates==0 && k.active()==1 && k.cursor==8);
    assert(!k.begin(nullptr, 0, 0, 0));
    std::printf("Sequence kernel: PPQN timing, chunk independence, stable ordering, sustain, loop restoration, late note-off, bounded overflow passed; kernel=%zu bytes, event=%zu bytes\n", sizeof(Kernel), sizeof(Event));
}
