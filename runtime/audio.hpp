#pragma once
#include "epok.hpp"
#include "common/hardware/dma.h"
#include "common/hardware/spu.h"
#include "spu_transfer.hpp"
#ifdef EPOK_HAS_SEQUENCES
#include "sequence_lock.hpp"
#endif

namespace epok {
namespace psx_audio { struct Sequence; }
struct AudioClip {const uint8_t* data;uint32_t size,rate;bool looping;const char* music=nullptr;const psx_audio::Sequence* sequence=nullptr;};
void music_play(AudioSource*);void music_stop(AudioSource*);bool music_is_playing(const AudioSource*);
#ifdef EPOK_HAS_SEQUENCES
void sequence_play(AudioSource*);void sequence_stop(AudioSource*);bool sequence_is_playing(const AudioSource*);
void sequence_voice_stolen(int);void sequence_update_sources();
#endif
inline const AudioClip* audio_bank=nullptr;
inline size_t audio_count=0;
inline uint16_t audio_addresses[512]={};
struct AudioVoice {
    AudioSource* owner=nullptr;uint32_t started=0;int clip=-1;
#ifdef EPOK_HAS_SEQUENCES
    int8_t sequence=-1;uint8_t note=0,priority=128;
#endif
};
inline AudioVoice audio_voices[24];
inline uint32_t audio_frame=0;
inline bool audio_ready=false;
inline uint32_t audio_upload_address=4096;
inline bool audio_initialize(const AudioClip* clips,size_t count){
    if(!count)return true;
    if(count>512)return false;
    audio_bank=clips;audio_count=count;audio_ready=false;
    DPCR=DPCR|0x000b0000;
    SBUS_DEV4_CTRL=SBUS_DEV4_CTRL&~0x0f000000;
    SPU_CTRL=0;SPU_KEY_OFF_LOW=0xffff;SPU_KEY_OFF_HIGH=0xff;
    SPU_VOL_MAIN_LEFT=0x3fff;SPU_VOL_MAIN_RIGHT=0x3fff;
    SPU_VOL_CD_LEFT=0;SPU_VOL_CD_RIGHT=0;SPU_VOL_EXT_LEFT=0;SPU_VOL_EXT_RIGHT=0;
    SPU_REVERB_LEFT=0;SPU_REVERB_RIGHT=0;SPU_REVERB_EN_LOW=0;SPU_REVERB_EN_HIGH=0;
    SPU_PITCH_MOD_LOW=0;SPU_PITCH_MOD_HIGH=0;SPU_NOISE_EN_LOW=0;SPU_NOISE_EN_HIGH=0;
    SPU_RAM_DTC=4;SPU_CTRL=0x8000;
    uint32_t address=4096;
    for(size_t i=0;i<count;++i){
        const auto& clip=clips[i];
        audio_addresses[i]=0;
        if(clip.music || (!clip.data && !clip.size))continue;
        if(!clip.data)return false;
        if(clip.size==0 || clip.size%64 || address+clip.size>512*1024)return false;
        audio_addresses[i]=address>>3;
        if(!spu::upload(clip.data,address,clip.size))return false;
        address+=clip.size;
    }
    SPU_CTRL=0xc000;
    for(int i=0;i<24;++i){
        audio_voices[i]={};
        SPU_VOICES[i].volumeLeft=0;SPU_VOICES[i].volumeRight=0;
        SPU_VOICES[i].sampleStartAddr=audio_addresses[0];SPU_VOICES[i].sampleRepeatAddr=audio_addresses[0];
        SPU_VOICES[i].adsrLo=0x000f;SPU_VOICES[i].adsrHi=0;
    }
    audio_upload_address=address;audio_ready=true;return true;
}
inline void audio_keyoff(int voice){if(voice<16)SPU_KEY_OFF_LOW=1u<<voice;else SPU_KEY_OFF_HIGH=1u<<(voice-16);}
void AudioSource::stop(){
#ifdef EPOK_HAS_SEQUENCES
    SequenceLock lock;
    sequence_stop(this);
#endif
    music_stop(this);
    for(int i=0;i<24;++i)if(audio_voices[i].owner==this){audio_keyoff(i);audio_voices[i].owner=nullptr;}
}
bool AudioSource::is_playing() const {
#ifdef EPOK_HAS_SEQUENCES
    SequenceLock lock;
    if(sequence_is_playing(this))return true;
#endif
    if(music_is_playing(this))return true;for(auto& v:audio_voices)if(v.owner==this)return true;return false;
}
inline void audio_parameters(int i){
    const auto& voice=audio_voices[i];const auto& source=*voice.owner;
    int volume=source.volume.raw();if(volume<0)volume=0;if(volume>4095)volume=4095;
    volume=int(uint32_t(volume)*transition_audio_gain()/4096);
    SPU_VOICES[i].volumeLeft=volume*4;SPU_VOICES[i].volumeRight=volume*4;
    int raw=source.pitch.raw();if(raw<1024)raw=1024;if(raw>16384)raw=16384;
    int pitch=audio_bank[voice.clip].rate*raw/44100;
    if(pitch<1)pitch=1;if(pitch>0x3fff)pitch=0x3fff;
    SPU_VOICES[i].sampleRate=uint16_t(pitch);
}
void AudioSource::play(){
#ifdef EPOK_HAS_SEQUENCES
    SequenceLock lock;
#endif
    if(!enabled || !audio_ready || clip<0 || size_t(clip)>=audio_count)return;
#ifdef EPOK_HAS_SEQUENCES
    if(audio_bank[clip].sequence){sequence_play(this);return;}
    sequence_stop(this);
#endif
    if(audio_bank[clip].music){music_play(this);return;}
    if(!audio_bank[clip].data || !audio_bank[clip].size)return;
    stop();int chosen=-1;
    for(int i=0;i<24;++i)if(!audio_voices[i].owner){chosen=i;break;}
    if(chosen<0){
        for(int i=0;i<24;++i){auto& voice=audio_voices[i];if(voice.owner->priority>priority)continue;
            if(chosen<0 || voice.owner->priority<audio_voices[chosen].owner->priority ||
                (voice.owner->priority==audio_voices[chosen].owner->priority && voice.started<audio_voices[chosen].started))chosen=i;
        }
    }
    if(chosen<0)return;
#ifdef EPOK_HAS_SEQUENCES
    sequence_voice_stolen(chosen);
#endif
    audio_keyoff(chosen);
    // Allow the SPU to consume key-off before reusing the hardware voice.
    for(volatile int wait=0;wait<2048;wait=wait+1){}
    audio_voices[chosen]={this,audio_frame,clip};
#ifdef EPOK_HAS_SEQUENCES
    audio_voices[chosen].priority=priority;
#endif
    auto& v=SPU_VOICES[chosen];v.sampleStartAddr=audio_addresses[clip];v.sampleRepeatAddr=audio_addresses[clip]+2;
    v.adsrLo=0x000f;v.adsrHi=0;
    audio_parameters(chosen);
    if(chosen<16)SPU_KEY_ON_LOW=1u<<chosen;else SPU_KEY_ON_HIGH=1u<<(chosen-16);
}
inline void audio_tick(){
#ifdef EPOK_HAS_SEQUENCES
    SequenceLock lock;
    sequence_update_sources();
#endif
    if(!audio_ready)return;
    ++audio_frame;
    uint32_t ended=HW_U16(0x1f801d9c)|(uint32_t(HW_U16(0x1f801d9e))<<16);
    for(int i=0;i<24;++i){auto& voice=audio_voices[i];if(!voice.owner)continue;
#ifdef EPOK_HAS_SEQUENCES
        if(voice.sequence>=0)continue;
        voice.priority=voice.owner->priority;
#endif
        if(!voice.owner->enabled || voice.owner->clip!=voice.clip || (!audio_bank[voice.clip].looping && audio_frame-voice.started>1 && (ended&(1u<<i)))){
            audio_keyoff(i);voice.owner=nullptr;
        }else audio_parameters(i);
    }
}
}
