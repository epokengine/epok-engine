#define EPOK_HAS_SEQUENCES 1
#define EPOK_SEQUENCE_HOST_TEST 1
#ifndef EPOK_TEST_MIXED_MUSIC
#define EPOK_NATIVE_SEQUENCES_ONLY 1
#endif
#include <cassert>
#include <cstdio>
#include <cstring>
#include <initializer_list>
#include "audio-runtime/sequence_service.hpp"
namespace epok {
void music_play(AudioSource*){} void music_stop(AudioSource*){} bool music_is_playing(const AudioSource*){return false;}
}
static void w16(uint8_t* p,uint16_t v){p[0]=uint8_t(v);p[1]=uint8_t(v>>8);}
static void w32(uint8_t* p,uint32_t v){w16(p,uint16_t(v));w16(p+2,uint16_t(v>>16));}
static int occupied(){int count=0;for(auto& v:epok::audio_voices)count+=v.sequence>=0;return count;}
int main(){
    using namespace epok;using namespace epok::native_music;
    alignas(4) uint8_t data[448]={'E','P','S','B',2,0,48,0,1,0,2,0};
    w32(data+12,48);w32(data+16,72);w32(data+20,360);w32(data+24,384);w32(data+28,448);w32(data+36,2);
    w32(data+48,384);w32(data+52,64);w32(data+56,22050);w32(data+60,84);w32(data+64,28);w32(data+68,56);
    data[401]=3;data[417]=1;data[433]=7;
    for(int z=0;z<2;++z){int at=72+144*z;data[at+7]=data[at+9]=127;data[at+10]=60;data[at+11]=data[at+12]=255;data[at+13]=3;
        for(int base:{at+40,at+72}){for(int o:{0,4,8})w32(data+base+o,uint32_t(-32768));w32(data+base+12,uint32_t(-12000));}
        w32(data+at+104,uint32_t(-32768));w32(data+at+120,uint32_t(-32768));}
    psx_audio::Bank bank{data,sizeof(data)};assert(bank.valid());
    const Command events[]={{0,Mark,0,0},{0,Group,0,2},{0,Start,0,0},{0,Start,1,0},
        {4000,Pitch,0,3072},{4000,Left,1,1000},{8000,Release,0,0},{8000,Release,1,0},
        {12000,Cut,0,0},{12000,Cut,1,0},{16000,Loop,0,0}};
    const Tone patch{0,2048,6000,3000,0x1208,0x1fcc,3,0,0};
    alignas(4) uint8_t bytes[40+sizeof(events)+sizeof(patch)]={'E','P','S','Q',3,0,40,0};
    w16(bytes+8,96);w16(bytes+10,16);w32(bytes+12,sizeof(events)/8);w32(bytes+20,1);
    std::memcpy(bytes+40,events,sizeof(events));std::memcpy(bytes+40+sizeof(events),&patch,sizeof(patch));
    psx_audio::Sequence song{bytes,sizeof(bytes),&bank};assert(song.valid());
    bytes[40+2*8+5]=24;assert(!song.valid());bytes[40+2*8+5]=0;
    bytes[40+2*8+6]=1;assert(!song.valid());bytes[40+2*8+6]=0;
    alignas(4) uint8_t sample[64]{};AudioClip clips[]={{sample,64,22050,true},{nullptr,0,0,false,nullptr,&song}};
    assert(audio_initialize(clips,2));assert(sequence_prepare());
    AudioSource source;source.clip=1;source.play();sequence_service(1000);assert(occupied()==2);
    sequence_service(1000);assert(SPU_KEY_ON_LOW==3 && SPU_VOICES[0].adsrLo==patch.adsr1 && SPU_VOICES[1].adsrHi==patch.adsr2);
    sequence_service(3000);assert(SPU_VOICES[0].sampleRate==3072 && SPU_VOICES[1].volumeLeft<1100);
    sequence_service(4000);assert(SPU_KEY_OFF_LOW==3 && SPU_VOICES[0].sampleRepeatAddr==bank.addresses[0]+4);
    sequence_service(4000);assert(occupied()==0);sequence_service(4000);assert(occupied()==2 && music_sequence_stats.loops==1);
    source.stop();sequence_service(2000);assert(!occupied() && !bank.pins);
    // A denied layered note must not leave a half-playing chord.
    AudioSource effects[24];for(int n=0;n<23;++n){effects[n].clip=0;effects[n].priority=255;effects[n].play();}
    source.play();sequence_service(1000);assert(!occupied() && music_sequence_stats.denied_notes==1);
    source.stop();sequence_service(2000);for(auto& s:effects)s.stop();
    // Stealing one layer retires the whole logical note; old commands cannot
    // change a voice which now belongs to an SFX.
    source.play();sequence_service(1000);sequence_service(1000);
    for(auto& s:effects){s.clip=0;s.priority=255;s.play();}assert(!occupied());
    sequence_service(8000);assert(!occupied());source.stop();sequence_service(2000);for(auto& s:effects)s.stop();
    // Source gain changes remain available, outside the IRQ.
    source.play();sequence_service(1000);sequence_service(1000);auto before=SPU_VOICES[0].volumeLeft;
    psx_audio::instances[0].parameters.volume=2048;psx_audio::instances[0].native_parameters(0);assert(SPU_VOICES[0].volumeLeft<before);
    source.enabled=false;sequence_update_sources();sequence_service(2000);assert(!source.is_playing() && !bank.pins);
    assert(music_sequence_stats.error==0 && sequence_lock_depth==0);
    std::printf("native_music_service passed; instances=%zu physical=%zu bytes\n",sizeof(psx_audio::instances),sizeof(psx_audio::physical));
}
