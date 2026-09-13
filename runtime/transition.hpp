#pragma once
#include <stdint.h>
#include <stddef.h>
#if __has_include("transition-config.hh")
#include "transition-config.hh"
#endif
#ifndef EPOK_FADE_OUT_MS
#define EPOK_FADE_OUT_MS 300
#define EPOK_FADE_IN_MS 300
#define EPOK_LOADING_TEXT "Now loading..."
#endif

namespace epok {
// Resident RGB555 pixels; max 64 x 64. The descriptor and pixels must outlive
// the transition, unlike text which request_scene copies immediately.
struct LoadingImage { const uint16_t* pixels=nullptr; uint16_t width=0,height=0; };
struct LoadingOptions {
    const char* text=EPOK_LOADING_TEXT;
    const LoadingImage* image=nullptr;
    uint8_t color[3]={255,255,255};
};
struct TransitionOptions {
    uint16_t fade_out_ms=EPOK_FADE_OUT_MS, fade_in_ms=EPOK_FADE_IN_MS;
    LoadingOptions loading;
};
enum class TransitionPhase : uint8_t { Idle, FadeOut, Loading, FadeIn, Failed };
struct TransitionState {
    TransitionPhase phase=TransitionPhase::Idle;
    TransitionOptions options;
    char text[96]={};
    uint32_t started=0;
    uint16_t audio_gain=4096;
    uint8_t opacity=0;
    bool presented=false, boot=false;
    void begin(const TransitionOptions& value,uint32_t now,bool initial=false) {
        options=value;
        size_t n=0;while(value.loading.text&&value.loading.text[n]&&n<95){text[n]=value.loading.text[n];++n;}text[n]=0;
        options.loading.text=text;
        started=now;boot=initial;presented=false;
        phase=initial?TransitionPhase::Loading:TransitionPhase::FadeOut;
        opacity=initial?255:0;audio_gain=initial?0:4096;
    }
    bool busy() const {return phase!=TransitionPhase::Idle;}
    bool loading() const {return phase==TransitionPhase::Loading||phase==TransitionPhase::Failed;}
    void advance(uint32_t now) {
        if(phase!=TransitionPhase::FadeOut&&phase!=TransitionPhase::FadeIn)return;
        const bool outgoing=phase==TransitionPhase::FadeOut;
        const uint32_t duration=uint32_t(outgoing?options.fade_out_ms:options.fade_in_ms)*1000;
        const uint32_t elapsed=now-started;
        const uint32_t progress=!duration||elapsed>=duration?4096:uint32_t(uint64_t(elapsed)*4096/duration);
        audio_gain=uint16_t(outgoing?4096-progress:progress);
        opacity=uint8_t((uint32_t(4096-audio_gain)*255+2048)/4096);
        if(progress==4096){phase=outgoing?TransitionPhase::Loading:TransitionPhase::Idle;presented=false;}
    }
    void loaded(uint32_t now) {phase=TransitionPhase::FadeIn;started=now;boot=false;}
    void fail() {phase=TransitionPhase::Failed;opacity=255;audio_gain=0;}
};
inline TransitionState transition;
// Read-only to gameplay. The transition gain never overwrites AudioSource.volume.
inline uint16_t transition_audio_gain(){return transition.audio_gain;}
}
