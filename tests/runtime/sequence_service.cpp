#define EPOK_HAS_SEQUENCES 1
#define EPOK_SEQUENCE_HOST_TEST 1
#include <cassert>
#include <cmath>
#include <cstdio>
#include <utility>
#include <initializer_list>
#include "audio-runtime/sequence_service.hpp"
namespace epok {
inline AudioSource* xa=nullptr;
void music_play(AudioSource* s){xa=s;}
void music_stop(AudioSource* s){if(xa==s)xa=nullptr;}
bool music_is_playing(const AudioSource* s){return xa==s;}
}
void w16(uint8_t* p,uint16_t v){p[0]=uint8_t(v);p[1]=uint8_t(v>>8);}
void w32(uint8_t* p,uint32_t v){w16(p,uint16_t(v));w16(p+2,uint16_t(v>>16));}
int main(){
    using namespace epok;
    alignas(4) uint8_t bank_bytes[192]={'E','P','S','B',1,0,32,0,1,0,1,0};
    w32(bank_bytes+12,32);w32(bank_bytes+16,56);w32(bank_bytes+20,128);w32(bank_bytes+24,192);
    for(auto [offset,value]:{std::pair{32,128}, {36,64}, {40,22050}, {44,56}, {48,28}, {52,56}})w32(bank_bytes+offset,value);
    bank_bytes[59]=255;bank_bytes[61]=127;bank_bytes[62]=1;bank_bytes[63]=127;bank_bytes[64]=60;
    w16(bank_bytes+68,4096);w32(bank_bytes+72,2);w32(bank_bytes+76,5);w16(bank_bytes+80,32767);w32(bank_bytes+84,20);
    bank_bytes[145]=7;
    psx_audio::Bank bank{bank_bytes,sizeof(bank_bytes)};
    assert(bank.valid());
    w32(bank_bytes+32,UINT32_MAX);assert(!bank.valid());w32(bank_bytes+32,128);
    bank_bytes[61]=128;assert(!bank.valid());bank_bytes[61]=127;
    alignas(4) uint8_t sequence_bytes[76]={'E','P','S','Q',1,0,40,0};
    w16(sequence_bytes+8,1000);w16(sequence_bytes+10,16);w32(sequence_bytes+12,3);w32(sequence_bytes+16,20);
    sequence_bytes[44]=sequence::NoteOn;sequence_bytes[46]=60;sequence_bytes[47]=100;
    w32(sequence_bytes+52,20);sequence_bytes[56]=sequence::NoteOff;sequence_bytes[58]=60;
    w32(sequence_bytes+64,20);sequence_bytes[68]=sequence::End;
    psx_audio::Sequence sequence{sequence_bytes,sizeof(sequence_bytes),&bank};
    alignas(4) uint8_t short_sequence[4]{};
    assert((!psx_audio::Sequence{short_sequence,sizeof(short_sequence),&bank}.valid()));
    assert(sequence.valid());sequence_bytes[68]=sequence::Program;assert(!sequence.valid());sequence_bytes[68]=sequence::End;
    // v1 has the original opcode/controller subset; v2 keeps the bytes/layout and adds typed controls.
    sequence_bytes[56]=sequence::Control;sequence_bytes[58]=66;assert(!sequence.valid());
    sequence_bytes[4]=2;assert(sequence.valid());
    sequence_bytes[56]=sequence::Parameter;sequence_bytes[58]=3;assert(!sequence.valid());
    sequence_bytes[58]=0;w32(sequence_bytes+60,12828);assert(!sequence.valid());
    w32(sequence_bytes+60,1200);assert(sequence.valid());
    sequence_bytes[4]=1;assert(!sequence.valid());
    sequence_bytes[4]=1;sequence_bytes[56]=sequence::NoteOff;sequence_bytes[58]=60;w32(sequence_bytes+60,0);assert(sequence.valid());
    alignas(4) uint8_t sfx_bytes[64]{};
    AudioClip clips[]={{sfx_bytes,64,22050,true},{nullptr,0,0,false,nullptr,&sequence},{nullptr,0,37800,true,"M0000002.XA;1"}};
    assert(audio_initialize(clips,3));assert(sequence_prepare());assert(bank.addresses[0]==520);
    AudioSource source;source.clip=1;source.play();assert(source.is_playing());assert(bank.pins==1);
    sequence_service(1000);assert(psx_audio::instances[0].kernel.active()==1);assert(psx_audio::physical[0].pending);
    sequence_service(1000);assert(!psx_audio::physical[0].pending);assert(SPU_VOICES[0].sampleRate==2048);
    for(int i=0;i<35;++i)sequence_service(1000);
    assert(!source.is_playing());assert(bank.pins==0);assert(music_sequence_stats.ends==1);
    // z.cents is already hundredths of a cent in EPSB v1. A +50-cent zone reaches the SPU at the matching step.
    w16(bank_bytes+66,5000);
    const auto cents50=uint16_t(2048.0*std::pow(2.0,50.0/1200.0));
    // A v2 tuning parameter changes the actual SPU step for an already-started note.
    sequence_bytes[4]=2;sequence_bytes[44]=sequence::Parameter;sequence_bytes[46]=2;sequence_bytes[47]=0;w32(sequence_bytes+48,76);
    w32(sequence_bytes+52,0);sequence_bytes[56]=sequence::NoteOn;sequence_bytes[58]=60;sequence_bytes[59]=100;
    w32(sequence_bytes+64,1000);sequence_bytes[68]=sequence::End;
    assert(sequence.valid());source.play();sequence_service(1000);sequence_service(1000);
    const auto cents50_octave=uint16_t(4096.0*std::pow(2.0,50.0/1200.0));
    assert(cents50 > 2048 && std::abs(int(psx_audio::physical[0].pitch)-int(cents50_octave))<=1 && std::abs(int(SPU_VOICES[0].sampleRate)-int(cents50_octave))<=1);
    // The pre-division octave check saturates rather than overflowing the numerator.
    auto& extreme=psx_audio::instances[0];auto& channel=extreme.kernel.channels[0];
    channel.bend_range_cents=12827;channel.bend=16383;channel.coarse_tuning=127;psx_audio::physical[0].key=127;
    const auto clamps=music_sequence_stats.pitch_clamps;extreme.update(0,channel);
    assert(psx_audio::physical[0].pitch==0x3fff && music_sequence_stats.pitch_clamps==clamps+1);
    source.stop();sequence_service(2000);assert(bank.pins==0);
    // A nonzero BankProgram is a valid v2 request, but EPSB v1 must reject it instead of mapping it to bank 0.
    sequence_bytes[44]=sequence::BankProgram;sequence_bytes[46]=7;w32(sequence_bytes+48,1);
    assert(sequence.valid());source.play();sequence_service(1000);sequence_service(1000);
    assert(music_sequence_stats.error==uint32_t(sequence::Error::MissingInstrument)+3 && !source.is_playing());
    sequence_service(2000);assert(bank.pins==0);
    // Leave the legacy fixture in its original form for all remaining transport tests.
    w16(bank_bytes+66,0);
    sequence_bytes[4]=1;sequence_bytes[44]=sequence::NoteOn;sequence_bytes[46]=60;sequence_bytes[47]=100;
    w32(sequence_bytes+52,20);sequence_bytes[56]=sequence::NoteOff;sequence_bytes[58]=60;w32(sequence_bytes+60,0);
    w32(sequence_bytes+64,20);sequence_bytes[68]=sequence::End;assert(sequence.valid());
    // Four start mailboxes are bounded. Stop cancels pending starts and releases
    // pins only after the physical retirement interval, before slot reuse.
    AudioSource queued[5];for(auto& q:queued)q.clip=1;
    for(auto& q:queued)q.play();
    assert(music_sequence_stats.capacity_errors==1 && !queued[4].is_playing());assert(bank.pins==4);
    for(auto& q:queued)q.stop();assert(bank.pins==4);
    sequence_service(1000);assert(bank.pins==4);sequence_service(1000);assert(bank.pins==0);
    for(const auto& p:psx_audio::physical)assert(!p.pending);
    // Sequence cannot evict higher-priority SFX. Refused notes are counted and
    // their later note-offs cannot affect another note or generation.
    AudioSource sfx[24];for(auto& s:sfx){s.clip=0;s.priority=255;s.play();}
    source.priority=128;source.play();sequence_service(1000);assert(music_sequence_stats.denied_notes==1);
    for(const auto& s:sfx)assert(s.is_playing());source.stop();sequence_service(2000);
    for(auto& s:sfx)s.stop();
    // SFX can steal a musical physical voice, preserving note-off tombstones.
    source.play();sequence_service(1000);sequence_service(1000);
    for(auto& s:sfx){s.clip=0;s.priority=255;s.play();}
    assert(music_sequence_stats.steals>=1);for(auto& s:sfx)s.stop();
    source.stop();sequence_service(2000);assert(bank.pins==0);
    // Switching to XA retires the sequence without retaining a source pointer.
    source.play();sequence_service(1000);source.clip=2;source.play();assert(xa==&source);
    assert(!sequence_is_playing(&source));sequence_service(2000);assert(bank.pins==0);
    source.clip=1;source.play();assert(xa==nullptr);sequence_service(1000);
    source.enabled=false;audio_tick();assert(!source.is_playing());sequence_service(2000);assert(bank.pins==0);
    source.enabled=true;source.play();sequence_service(1000);sequence_clock_fault();assert(!source.is_playing());
    sequence_service(2000);assert(bank.pins==0 && music_sequence_stats.clock_faults==1);
    assert(sequence_lock_depth==0);
    std::printf("PSX host transport: %zu bytes instance pool, %zu physical state, %zu stats; pins retired, priority/disable/XA/clock faults passed\n",sizeof(psx_audio::instances),sizeof(psx_audio::physical),sizeof(music_sequence_stats));
}
