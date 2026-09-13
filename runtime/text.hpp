#pragma once
#include <stddef.h>
#include <stdint.h>
namespace epok {
struct Text {
    bool enabled=false;
    uint8_t color[3]={255,255,255};
    // UTF-8 is converted once at assignment to the built-in atlas encoding.
    char value[512]={};
    bool wrap=true;
    void set_text(const char* text) {
        size_t n=0;
        while(text && *text && n<511) {
            unsigned c=uint8_t(*text++);
            if(c>=0xc2 && c<=0xdf && (uint8_t(*text)&0xc0)==0x80) {
                c=((c&31)<<6)|(uint8_t(*text++)&63);
            } else if(c>=128) { c='?'; }
            static constexpr uint16_t extra[]={0xe1,0xe9,0xed,0xf3,0xfa,0xfc,0xf1,0xc1,0xc9,0xcd,0xd3,0xda,0xdc,0xd1,0xbf,0xa1};
            unsigned encoded='?';
            if(c=='\n' || (c>=32 && c<=126)) encoded=c;
            else for(unsigned i=0;i<sizeof(extra)/sizeof(extra[0]);++i) if(extra[i]==c) encoded=127+i;
            value[n++]=char(encoded);
        }
        value[n]=0;
    }
};
struct HudStats {uint32_t rectangles=0,glyphs=0,texts=0,images=0,dropped=0;};
inline HudStats hud_stats;
}
