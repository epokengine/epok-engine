#pragma once
// Host-only ADSR fitting/audition. Rates follow the PSX SPU register layout.
// The console uses the actual hardware envelope, not this model.
#include <cstdint>
#include <cmath>
#include <algorithm>
namespace epok::native_music {
struct Envelope {
    uint16_t lo=0,hi=0;int level=0;uint32_t wait=0;uint8_t phase=0;
    void key_on(uint16_t a,uint16_t b){lo=a;hi=b;level=0;wait=0;phase=0;}
    void key_off(){if(phase<3){phase=3;wait=0;}}
    bool done()const{return phase==4;}
    void advance(uint32_t frames){
        while(frames && phase<4){
            if(wait){const auto n=std::min(wait,frames);wait-=n;frames-=n;if(!frames)break;}
            unsigned shift,step;bool down,exponential;
            if(phase==0){shift=(lo>>10)&31;step=(lo>>8)&3;down=false;exponential=lo&0x8000;}
            else if(phase==1){shift=(lo>>4)&15;step=0;down=exponential=true;}
            else if(phase==2){shift=(hi>>8)&31;step=(hi>>6)&3;down=hi&0x4000;exponential=hi&0x8000;
                if(shift==31 && step==3)return; // infinite sustain
            }else{shift=hi&31;step=0;down=true;exponential=hi&32;}
            const uint32_t period=1u<<(shift>11?shift-11:0);
            int change=(down?(-8+int(step)):(7-int(step)))*(1<<(shift<11?11-shift:0));
            wait=period;
            if(exponential && !down && level>0x6000)wait*=4;
            if(exponential && down)change=std::min(-1,(change*level)>>15);
            level=std::clamp(level+change,0,32767);
            if(phase==0 && level==32767){phase=1;wait=0;}
            else if(phase==1 && level<=std::min(32767,(int(lo&15)+1)*2048)){phase=2;wait=0;}
            else if(down && level==0 && phase>=2){phase=4;wait=0;}
        }
    }
};
inline double rate_duration(unsigned shift,unsigned step,bool down,bool exponential,int target){
    int level=down?32767:0;uint64_t frames=0;
    for(unsigned i=0;i<100000;++i){
        unsigned period=1u<<(shift>11?shift-11:0);
        int change=(down?(-8+int(step)):(7-int(step)))*(1<<(shift<11?11-shift:0));
        if(exponential && !down && level>0x6000)period*=4;
        if(exponential && down)change=std::min(-1,(change*level)>>15);
        frames+=period;level=std::clamp(level+change,0,32767);
        if((down && level<=target) || (!down && level>=target))return double(frames)*1000000.0/44100;
    }
    return 1e15;
}
struct EnvelopeFit {uint16_t lo=0,hi=0;double error=0;};
inline EnvelopeFit fit_envelope(uint32_t attack,uint32_t hold,uint32_t decay,uint16_t sustain,uint32_t release){
    EnvelopeFit out;double best=1e100;
    auto score=[](double actual,double desired){return std::abs(std::log((actual+23)/(desired+23)));};
    for(unsigned rate=0;rate<127;++rate){const double e=score(rate_duration(rate>>2,rate&3,false,false,32767),attack);
        if(e<best){best=e;out.lo=uint16_t(rate<<8);}}
    const unsigned sl=unsigned(std::clamp((int(sustain)+1024)/2048-1,0,15));out.lo|=uint16_t(sl);
    const int target=std::min(32767,int(sl+1)*2048);best=1e100;
    // Very fast exponential steps can overshoot the chosen sustain level by
    // 50%. Prefer <=1/64-scale steps over that permanent loudness error. Full
    // sustain has no audible decay/hold at all: use the smallest decay step.
    for(unsigned rate=target==32767?15:5;rate<16;++rate){const double e=score(rate_duration(rate,0,true,true,target),double(hold)+decay);
        if(e<best){best=e;out.lo=uint16_t((out.lo&0xff0f)|(rate<<4));}}
    // Freeze sustain. A source with zero sustain continues decaying to silence.
    out.hi=0x1fc0;
    if(sustain<1024)out.hi=uint16_t(0xc000|(((out.lo>>4)&15)<<8));
    best=1e100;
    for(unsigned rate=0;rate<31;++rate){const double e=score(rate_duration(rate,0,true,true,0),release);
        if(e<best){best=e;out.hi=uint16_t((out.hi&~63u)|32|rate);}}
    out.error=best;return out;
}
}
