#pragma once
#include <stdint.h>
namespace epok {
// One cooked glyph of an authored font. The field order matches
// src/font_asset.rs::GlyphMetric, which header_fragment emits positionally.
// `u`/`v` are atlas texels; `x_offset`/`y_offset` place the drawn box relative
// to the pen on the baseline, y growing downward, exactly as ab_glyph reports
// the outline bounds at import.
struct GlyphMetric { uint32_t codepoint; uint8_t u,v,width,height,advance; int8_t x_offset,y_offset; };
// A 4bpp glyph atlas with its sixteen-entry CLUT. `x`,`y`,`clut_x`,`clut_y` are
// VRAM placement assigned by the export allocator, not by the baker; `baseline`
// is the ascent from the top of a line, so a glyph draws at baseline+y_offset.
struct Font { const uint16_t* pixels; const GlyphMetric* metrics; const uint16_t* palette; uint16_t count,width,height,line_height,baseline,x,y,clut_x,clut_y; };
// Metrics are sorted by codepoint at import, so a miss costs log2(count) reads.
inline const GlyphMetric* find_glyph(const Font& font,uint32_t codepoint) {
    if(!font.metrics)return nullptr;
    uint16_t low=0,high=font.count;
    while(low<high){const uint16_t mid=uint16_t(low+(high-low)/2);const uint32_t c=font.metrics[mid].codepoint;if(c==codepoint)return &font.metrics[mid];if(c<codepoint)low=uint16_t(mid+1);else high=mid;}
    return nullptr;
}
// One UTF-8 scalar from `text`, advancing `n` past it. A malformed or truncated
// sequence yields '?' and consumes one byte, so a bad string can neither loop
// nor read past the terminator.
inline uint32_t utf8_scalar(const char* text,size_t& n) {
    const uint8_t a=uint8_t(text[n++]);
    if(a<0x80)return a;
    const uint8_t b=uint8_t(text[n]);
    if((b&0xc0)!=0x80)return '?';
    if(a>=0xc2&&a<=0xdf){++n;return uint32_t(a&31)<<6|uint32_t(b&63);}
    const uint8_t c=uint8_t(text[n+1]);
    if((c&0xc0)!=0x80)return '?';
    if(a>=0xe0&&a<=0xef){n+=2;return uint32_t(a&15)<<12|uint32_t(b&63)<<6|uint32_t(c&63);}
    const uint8_t d=uint8_t(text[n+2]);
    if(a>=0xf0&&a<=0xf4&&(d&0xc0)==0x80){n+=3;return uint32_t(a&7)<<18|uint32_t(b&63)<<12|uint32_t(c&63)<<6|uint32_t(d&63);}
    return '?';
}
// The cell mapping src/bitmap_font.rs bakes into the built-in atlas: ASCII
// 32..126 in order, then the sixteen Spanish extras, and anything else draws
// '?'. Font index -1 is this atlas; only authored fonts go through `Font`.
inline int builtin_cell(uint32_t codepoint) {
    if(codepoint>=32&&codepoint<=126)return int(codepoint-32);
    static constexpr uint32_t extra[16]={0xe1,0xe9,0xed,0xf3,0xfa,0xfc,0xf1,0xc1,0xc9,0xcd,0xd3,0xda,0xdc,0xd1,0xbf,0xa1};
    for(int i=0;i<16;++i)if(extra[i]==codepoint)return 95+i;
    return '?'-32;
}
}
