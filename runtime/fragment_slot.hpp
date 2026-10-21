#pragma once
#include <new>
#include <stdint.h>

namespace epok {
// A frame slot is allocated once for either packet type. Double buffering and
// the retained-slot allocator already keep its GPU lifetime disjoint. Sharing
// that storage avoids reserving two complete packet arenas for the same slots.
template<class Plain,class Textured> union FragmentSlot {
    Plain plain;
    Textured textured;
    FragmentSlot() {}
    Plain& make_plain(){return *::new(static_cast<void*>(&plain)) Plain;}
    Textured& make_textured(){return *::new(static_cast<void*>(&textured)) Textured;}
};
// The retained packet's command, colors and UVs already exist. Update only XY
// and DMA links, using the same GT3/G3 word layout as PsyQo's typed fragments.
template<bool Textured,class Head> inline void update_retained_triangle(uint32_t* packet,uint32_t a,uint32_t b,uint32_t c,Head& bucket){
    packet[2]=a;packet[Textured?5:4]=b;packet[Textured?8:6]=c;
    packet[0]=((Textured?9u:6u)<<24)|(bucket&0xffffffu);
    bucket=uint32_t(reinterpret_cast<uintptr_t>(packet)&0xffffffu);
}
template<bool Textured> inline void update_retained_uniform_color(uint32_t* packet,uint32_t command,uint32_t color){
    packet[1]=(Textured?command:0x30000000u)|color;
    packet[Textured?4:3]=color;packet[Textured?7:5]=color;
}
}
