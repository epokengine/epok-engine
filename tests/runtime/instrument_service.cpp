#define EPOK_HAS_SEQUENCES 1
#define EPOK_SEQUENCE_HOST_TEST 1
#include <cassert>
#include <cstdio>
#include <vector>
#include "audio-runtime/sequence_service.hpp"
namespace epok {
inline AudioSource* xa=nullptr;
void music_play(AudioSource* s){xa=s;}
void music_stop(AudioSource* s){if(xa==s)xa=nullptr;}
bool music_is_playing(const AudioSource* s){return xa==s;}
}
void w16(uint8_t* p,uint16_t v){p[0]=uint8_t(v);p[1]=uint8_t(v>>8);}
void w32(uint8_t* p,uint32_t v){w16(p,uint16_t(v));w16(p+2,uint16_t(v>>16));}
int occupied(){int result=0;for(const auto& v:epok::audio_voices)result+=v.sequence>=0;return result;}
void inject(epok::psx_audio::Instance& instance,epok::sequence::Event event){
    auto& k=instance.kernel;const auto* events=k.events;const auto count=k.count,cursor=k.cursor;
    event.tick=k.tick;
    const epok::sequence::Event stream[]={event,{UINT32_MAX,epok::sequence::End,0,0,0,0}};
    k.events=stream;k.count=2;k.cursor=0;k.advance(0,instance);
    k.events=events;k.count=count;k.cursor=cursor;
}
int main(){
    using namespace epok;
    alignas(4) uint8_t data[448]={'E','P','S','B',2,0,48,0,1,0,2,0};
    w32(data+12,48);w32(data+16,72);w32(data+20,360);w32(data+24,384);w32(data+28,448);w32(data+36,2);
    w32(data+48,384);w32(data+52,64);w32(data+56,22050);w32(data+60,84);w32(data+64,28);w32(data+68,56);
    data[401]=3;data[417]=1;data[433]=7;
    for(int z=0;z<2;++z){
        const int at=72+144*z;
        data[at+7]=data[at+9]=127;data[at+10]=60;data[at+11]=data[at+12]=255;data[at+13]=3;
        w32(data+at+20,100);w32(data+at+28,z?250:uint32_t(-250));
        for(int base:{at+40,at+72}){
            for(int offset:{0,4,8})w32(data+base+offset,uint32_t(-32768));
            w32(data+base+12,uint32_t(-12000));w32(data+base+20,0);
        }
        w32(data+at+104,uint32_t(-32768));w32(data+at+120,uint32_t(-32768));
    }
    psx_audio::Bank bank{data,sizeof(data)};assert(bank.valid());
    alignas(4) uint8_t sequence_data[76]={'E','P','S','Q',2,0,40,0};
    w16(sequence_data+8,1000);w16(sequence_data+10,16);w32(sequence_data+12,3);
    sequence_data[44]=sequence::NoteOn;sequence_data[46]=60;sequence_data[47]=100;
    w32(sequence_data+52,100);sequence_data[56]=sequence::NoteOff;sequence_data[58]=60;
    w32(sequence_data+64,100);sequence_data[68]=sequence::End;
    instrument::preparation::Storage<2,3,2> prepared;
    psx_audio::Sequence sequence{sequence_data,sizeof(sequence_data),&bank,&prepared.cache};assert(sequence.valid());
    alignas(4) uint8_t sfx_bytes[64]{};
    AudioClip clips[]={{sfx_bytes,64,22050,true},{nullptr,0,0,false,nullptr,&sequence},{nullptr,0,37800,true,"M0000002.XA;1"}};
    assert(audio_initialize(clips,3));assert(sequence_prepare());
    AudioSource source;source.clip=1;source.play();sequence_service(1000);
    auto& first=psx_audio::instances[0];assert(first.kernel.limit==128 && first.kernel.active()==1 && occupied()==2);
    sequence_service(1000);assert(SPU_VOICES[0].sampleRate==2048 && SPU_VOICES[1].sampleRate==2048);
    assert(SPU_KEY_ON_LOW==3); // Both layers must reach the shared KON latch.
    assert(SPU_VOICES[0].volumeLeft>SPU_VOICES[0].volumeRight && SPU_VOICES[1].volumeRight>SPU_VOICES[1].volumeLeft);
    // Controllers and release between control ticks must advance the old
    // envelope first, then apply the new operation without retroactive timing.
    sequence_service(1000);
    auto expected=psx_audio::physical[0].synthesis;
    sequence_service(1500);
    expected.advance(1500);
    auto changed=first.kernel.channels[0];changed.expression=70;
    expected.update_controls(psx_audio::instrument_controls(changed));
    first.update_library(0,changed,true);
    assert(psx_audio::physical[0].synthesis.output().gain_q15==expected.output().gain_q15);
    assert(psx_audio::physical[0].synthesis.output().pitch_cents_x100==expected.output().pitch_cents_x100);
    sequence_service(500);expected.advance(500);expected.release();
    // ENDX is sticky after the first traversal; release must not cut both tails.
    HW_U16(0x1f801d9c)=3;
    first.release(0);
    assert(psx_audio::physical[0].synthesis.output().gain_q15==expected.output().gain_q15);
    sequence_service(4000);expected.advance(4000);assert(occupied()==2);
    assert(psx_audio::physical[0].synthesis.output().gain_q15==expected.output().gain_q15);
    assert(SPU_VOICES[0].sampleRepeatAddr==bank.addresses[0]+4 && SPU_VOICES[1].sampleRepeatAddr==bank.addresses[0]+4);
    source.stop();sequence_service(2000);assert(!occupied() && !bank.pins);HW_U16(0x1f801d9c)=0;
    // Note-off can arrive while both starts are pending; key-on keeps the tail address.
    w32(sequence_data+52,0);w32(sequence_data+64,0);
    source.play();sequence_service(1000);assert(occupied()==2);
    sequence_service(1000);assert(SPU_VOICES[0].sampleRepeatAddr==bank.addresses[0]+4);
    for(int i=0;i<1600;++i)sequence_service(1000);
    assert(!occupied() && !bank.pins && !source.is_playing());
    w32(sequence_data+52,10000);w32(sequence_data+64,10000);
    // Repeat above voice 15: the second KON register must retain the chord too.
    AudioSource lower[16];for(auto& s:lower){s.clip=0;s.play();}
    source.play();sequence_service(1000);sequence_service(1000);
    assert(SPU_KEY_ON_HIGH==3 && occupied()==2);
    source.stop();sequence_service(2000);for(auto& s:lower)s.stop();
    // Two-layer admission is atomic even with one hardware voice still free.
    AudioSource sfx[24];for(int i=0;i<23;++i){sfx[i].clip=0;sfx[i].priority=255;sfx[i].play();}
    const auto denied=music_sequence_stats.denied_notes;
    source.play();sequence_service(1000);assert(occupied()==0 && music_sequence_stats.denied_notes==denied+1);
    assert(first.kernel.active()==0);for(int i=0;i<23;++i)assert(sfx[i].is_playing());
    source.stop();sequence_service(2000);for(auto& s:sfx)s.stop();
    // High-priority SFX steal the entire old layered note, leaving its tombstone.
    source.play();sequence_service(1000);sequence_service(1000);
    for(auto& s:sfx){s.clip=0;s.priority=255;s.play();}
    assert(!occupied() && first.kernel.active()==0);
    source.stop();sequence_service(2000);for(auto& s:sfx)s.stop();
    // Across instances the configured music ceiling is physical, including layers.
    AudioSource other;other.clip=1;source.play();other.play();sequence_service(1000);
    auto& second=psx_audio::instances[1];
    sequence::Event note{0,sequence::NoteOn,0,60,100,0};
    for(int i=0;i<4;++i){inject(first,note);inject(second,note);}
    assert(occupied()==16 && first.kernel.active()+second.kernel.active()==8);
    // Exclusion is scoped to channel and instrument, and removes whole groups.
    source.stop();other.stop();sequence_service(2000);assert(!bank.pins);
    w16(data+72+14,1);w16(data+216+14,1);
    source.play();sequence_service(1000);inject(first,note);
    assert(occupied()==2 && first.kernel.active()==1);
    note.channel=1;inject(first,note);assert(occupied()==4 && first.kernel.active()==2);
    source.clip=2;source.play();assert(xa==&source);sequence_service(2000);assert(!occupied() && !bank.pins);
    source.stop();source.clip=1;
    // Reverb is a global leased resource. Zero CC91 suppresses even a positive
    // bank send, and clearing the old owner cannot leak wet sends into XA/SFX.
    w32(data+40,1);w32(data+44,8192);w32(data+72+36,200);w32(data+216+36,200);
    assert(bank.valid() && sequence_prepare());
    assert(psx_audio::reverb_resource.reserved_bytes()==9920);
    source.play();sequence_service(1000);sequence_service(1000);
    assert(psx_audio::reverb_resource.active() && SPU_REVERB_EN_LOW==3 && SPU_REVERB_LEFT==8192);
    inject(first,{0,sequence::Control,0,91,0,0});assert(SPU_REVERB_EN_LOW==0);
    inject(first,{0,sequence::Control,0,91,100,0});assert(SPU_REVERB_EN_LOW==3);
    const auto capacity=music_sequence_stats.capacity_errors;other.play();
    assert(!other.is_playing() && source.is_playing() && music_sequence_stats.capacity_errors==capacity+1);
    source.stop();assert(!SPU_REVERB_EN_LOW && !SPU_REVERB_LEFT && !(SPU_CTRL&0x80));
    sequence_service(2000);assert(!bank.pins);
    source.play();sequence_service(1000);sequence_service(1000);assert(SPU_REVERB_EN_LOW==3);
    source.clip=2;source.play();sequence_service(2000);assert(!bank.pins && !SPU_REVERB_LEFT && !SPU_REVERB_EN_LOW);
    assert(music_sequence_stats.error==11);music_sequence_stats.error=0; // expected lease-conflict diagnostic
    assert(sequence_lock_depth==0 && !music_sequence_stats.error);
    std::puts("instrument_service: layers, early release/repeat addresses, sticky ENDX, ceilings, priority, exclusion, FIFO, pins and XA passed");
}
