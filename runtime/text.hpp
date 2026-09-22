#pragma once
#include <stddef.h>
#include <stdint.h>
namespace epok {
// Horizontal alignment of each measured line inside the text rect. Pinned:
// saved Blueprint graphs store it, so the type only ever grows at the end.
enum class TextAlign : uint8_t { Left=0, Center=1, Right=2 };
struct Text {
    bool enabled=false;
    uint8_t color[3]={255,255,255};
    // UTF-8, verbatim. Decoding to a glyph happens at draw time, against the
    // built-in atlas or the authored font this element names.
    char value[512]={};
    bool wrap=true;
    // -1 is the built-in 8x16 atlas; anything else indexes the cooked fonts.
    int font=-1;
    TextAlign align=TextAlign::Left;
    void set_text(const char* text) {
        size_t n=0;
        while(text&&*text&&n<511)value[n++]=*text++;
        value[n]=0;
    }
};
// `rotated` counts quads emitted through the rotated path; they come out of
// hud_rotated_budget rather than the rectangle and glyph pools.
struct HudStats {uint32_t rectangles=0,glyphs=0,texts=0,images=0,dropped=0,rotated=0;};
inline HudStats hud_stats;
}
