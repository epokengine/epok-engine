#include <cassert>
#include "audio-runtime/audio.hpp"
namespace epok {
inline int music_starts=0;
void music_play(AudioSource*){++music_starts;}
void music_stop(AudioSource*){}
bool music_is_playing(const AudioSource*){return false;}
}
int main(){
    using namespace epok;
    uint8_t sample[64]={};
    AudioClip clips[]={{nullptr,0,0,false},{sample,64,22050,false},
                       {nullptr,0,0,false},{sample,64,22050,true}};
    assert(audio_initialize(clips,4));
    assert(audio_ready && audio_addresses[1]==512 && audio_addresses[3]==520);
    AudioSource omitted;omitted.clip=0;
    omitted.play();omitted.stop();assert(!omitted.is_playing());
    for(const auto& voice:audio_voices)assert(!voice.owner);
    AudioSource sfx;sfx.clip=1;sfx.play();assert(sfx.is_playing());
    omitted.clip=2;omitted.play();assert(!omitted.is_playing());
    assert(sfx.is_playing() && audio_voices[0].owner==&sfx);
    audio_tick();assert(sfx.is_playing());sfx.stop();assert(!sfx.is_playing());
    // Preserve free-slot, priority and oldest-age selection before adding notes.
    AudioSource crowded[26];
    for(int i=0;i<24;++i){
        crowded[i].clip=3;crowded[i].priority=128;
        audio_tick();crowded[i].play();
        assert(audio_voices[i].owner==&crowded[i]);
    }
    crowded[24].clip=3;crowded[24].priority=0;crowded[24].play();
    assert(!crowded[24].is_playing());
    crowded[24].priority=128;crowded[24].play();
    assert(audio_voices[0].owner==&crowded[24] && !crowded[0].is_playing());
    crowded[25].clip=3;crowded[25].priority=255;crowded[25].play();
    assert(audio_voices[1].owner==&crowded[25] && !crowded[1].is_playing());
    crowded[24].enabled=false;audio_tick();assert(!crowded[24].is_playing());
    crowded[25].clip=1;audio_tick();assert(!crowded[25].is_playing());
    for(auto& source:crowded)source.stop();
    for(const auto& voice:audio_voices)assert(!voice.owner);
    assert(music_starts==0);
    AudioClip silent[]={{nullptr,0,0,false}};
    assert(audio_initialize(silent,1));omitted.clip=0;
    omitted.play();assert(!omitted.is_playing());
    // CD descriptors still take the normal music path.
    AudioClip cd[]={{nullptr,0,37800,true,"M0000000.XA;1"}};
    assert(audio_initialize(cd,1));omitted.play();assert(music_starts==1);
}
