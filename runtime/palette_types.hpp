#pragma once
#include "psyqo/fixed-point.hh"
#include <stdint.h>
namespace epok {
using Fixed=psyqo::FixedPoint<12>;
struct PaletteAnimator {
    bool enabled=false,reverse=false;int texture=-1;
    uint8_t first=1,last=2;Fixed speed=8.0;
    uint32_t remainder=0;uint16_t offset=0;
    void reset(){remainder=0;offset=0;}
    void advance(Fixed dt) {
        if(!enabled||first==0||last<=first||speed.raw()<=0||dt.raw()<=0)return;
        const uint32_t count=uint32_t(last)-first+1;
        uint64_t elapsed=uint64_t(uint32_t(dt.raw()))*uint32_t(speed.raw())+remainder;
        offset=uint16_t((offset+elapsed/(4096u*4096u))%count);
        remainder=uint32_t(elapsed%(4096u*4096u));
    }
    uint16_t source_index(uint16_t index)const {
        if(!enabled||first==0||last<=first||index<first||index>last)return index;
        const uint16_t count=uint16_t(last)-first+1;
        const uint16_t shift=reverse?uint16_t((count-offset%count)%count):uint16_t(offset%count);
        return uint16_t(first+(index-first+shift)%count);
    }
    void apply(const uint16_t* source,uint16_t* destination)const {
        for(uint16_t i=0;i<256;++i)destination[i]=source[source_index(i)];
    }
};
struct PaletteStats {uint32_t uploads=0,bytes=0,conflicts=0;};
inline PaletteStats palette_stats;
}
