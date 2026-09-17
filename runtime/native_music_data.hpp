#pragma once
// EPSQ v3: editor-compiled SPU commands. No MIDI parser or SoundFont synthesis
// is needed to execute this stream. Times are absolute microseconds per loop.
#include <cstdint>
#include <cstddef>
namespace epok::native_music {
enum Op : uint8_t { Group, Start, Release, Cut, Pitch, Left, Right, Send, Loop, End, Mark };
struct Command { uint32_t time; uint8_t op, lane; uint16_t value; };
struct Tone { uint16_t sample,pitch,left,right,adsr1,adsr2; uint8_t loop,send; uint16_t reserved; };
static_assert(sizeof(Command)==8 && sizeof(Tone)==16);
inline uint16_t u16(const uint8_t* p){return uint16_t(p[0])|uint16_t(p[1])<<8;}
inline uint32_t u32(const uint8_t* p){return u16(p)|uint32_t(u16(p+2))<<16;}
struct View {
    const uint8_t* data=nullptr;uint32_t size=0;
    uint32_t count()const{return u32(data+12);}
    uint16_t tones()const{return uint16_t(u32(data+20));}
    const Command* commands()const{return reinterpret_cast<const Command*>(data+40);}
    const Tone* patches()const{return reinterpret_cast<const Tone*>(data+40+count()*8);}
    bool valid(uint16_t samples)const{
        if(!data || uintptr_t(data)%4 || size<40 || size>256*1024 || data[0]!='E' || data[1]!='P' || data[2]!='S' || data[3]!='Q' || u16(data+4)!=3 || u16(data+6)!=40 ||
            !u16(data+10) || u16(data+10)>24 || !count() || count()>32768 || !u32(data+20) || u32(data+20)>65535 ||
            uint64_t(40)+uint64_t(count())*8+uint64_t(u32(data+20))*16!=size)return false;
        bool marked=false,ended=false;uint32_t mark=0;uint16_t group=0;uint32_t lanes=0,active=0;
        for(uint32_t n=0;n<count();++n){const auto& e=commands()[n];
            if(ended || (n && e.time<commands()[n-1].time) || (group && (e.op!=Start || e.time!=commands()[n-1].time)))return false;
            if(e.op==Group){if(e.lane>=128 || !e.value || e.value>24)return false;group=e.value;lanes=0;}
            else if(e.op==Start){if(!group || e.lane>=24 || e.value>=tones() || ((lanes|active)&(1u<<e.lane)))return false;--group;lanes|=1u<<e.lane;active|=1u<<e.lane;}
            else if(e.op==Mark){if(marked || e.lane || e.value)return false;marked=true;mark=e.time;}
            else if(e.op==Loop){if(!marked || e.time<=mark || e.lane || e.value || active)return false;ended=true;}
            else if(e.op==End){if(marked || e.lane || e.value || active)return false;ended=true;}
            else if(e.op>Mark || e.lane>=24 ||
                ((e.op==Cut || e.op==Release) && e.value) || (e.op==Pitch && (!e.value || e.value>0x3fff)) ||
                ((e.op==Left || e.op==Right) && e.value>0x3fff) || (e.op==Send && e.value>1))return false;
            else {if(!(active&(1u<<e.lane)))return false;if(e.op==Cut)active&=~(1u<<e.lane);}
        }
        if(!ended || group)return false;
        for(unsigned n=0;n<tones();++n){const auto& t=patches()[n];if(t.sample>=samples || !t.pitch || t.pitch>0x3fff ||
            t.left>0x3fff || t.right>0x3fff || (t.loop!=0 && t.loop!=1 && t.loop!=3) || t.send>1 || t.reserved || (t.adsr2&0x2000))return false;}
        return true;
    }
};
}
