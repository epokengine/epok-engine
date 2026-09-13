#pragma once
#include "psyqo/fragments.hh"
#include "psyqo/primitives/control.hh"
#include "psyqo/primitives/misc.hh"
#include "psyqo/primitives/rectangles.hh"

namespace epok {
template<bool Interlaced>
inline void configure_display_field(psyqo::PrimPieces::TPageAttr& attr) {
    // In 480i, preserve the field currently being scanned out. In progressive
    // modes PsyQo instead draws into the other 240-line framebuffer.
    if constexpr (Interlaced) attr.disableDisplayArea();
    else attr.enableDisplayArea();
}

template<int Width, int Height>
class FrameClear {
    psyqo::Fragments::SimpleFragment<psyqo::Prim::FastFill> fills[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> fields[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2];
public:
    template<class Gpu> void draw(Gpu& gpu, psyqo::Color color) {
        const unsigned parity = gpu.getParity();
        auto& page = pages[parity];
        configure_display_field<Height == 480>(page.primitive.attr);
        gpu.chain(page);
        if constexpr (Height == 480) {
            // FastFill ignores field masking and scissor. getNextClear also
            // starts at y=256 in PsyQo's interlaced mode, wrapping past VRAM.
            // A rasterized rectangle obeys field masking and draws at (0,0).
            auto& field = fields[parity];
            field.primitive.position = {{.x=0,.y=0}};
            field.primitive.size = {{.w=Width,.h=Height}};
            field.primitive.setColor(color).setOpaque();
            gpu.chain(field);
        } else {
            auto& fill = fills[parity];
            gpu.getNextClear(fill.primitive, color);
            gpu.chain(fill);
        }
    }
};
}
