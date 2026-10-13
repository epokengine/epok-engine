#pragma once
// World2D renderer (World2D initiative, phase 1). Sprites are drawn after the 3D world
// and before the HUD, back to front by a stable key: z-index (the owner's draw_order plus
// the sprite's z_index) first, then, for y-sorted sprites, the world Y of their origin;
// equal keys keep creation order. An unrotated, unscaled, unflipped sprite seen through an
// unrotated camera at zoom 1 is emitted as a GP0 rectangle (`Sprite`: exact texels, less
// setup); everything else becomes a textured quad.
//
// Capacities come from the cooked scene header (defaults below):
//   EPOK_WORLD2D_DRAW_ITEMS    sprites drawn per frame
//   EPOK_WORLD2D_ARENA_BYTES   GPU packet bytes per frame parity
// Packets are placed in a per-parity byte arena, so rectangles, quads and page changes
// share one budget; exceeding it drops the extra sprites and counts them.
#include "world2d_service.hpp"
#include "texture.hpp"
#include "frame_clear.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/control.hh"
#include "psyqo/primitives/quads.hh"
#include "psyqo/primitives/sprites.hh"
#include <new>

#ifndef EPOK_WORLD2D_ARENA_BYTES
#define EPOK_WORLD2D_ARENA_BYTES 12288
#endif

namespace epok {
namespace world2d {

// ---- renderer --------------------------------------------------------------------------
class Renderer {
    alignas(4) uint8_t arena[2][EPOK_WORLD2D_ARENA_BYTES];
    SpriteDraw draws[EPOK_WORLD2D_DRAW_ITEMS];
    DrawItem items[EPOK_WORLD2D_DRAW_ITEMS], scratch[EPOK_WORLD2D_DRAW_ITEMS];
    size_t used = 0;
    unsigned parity = 0;

    template<class Fragment> Fragment* allocate() {
        constexpr size_t size = (sizeof(Fragment) + 3) & ~size_t(3);
        if (used + size > EPOK_WORLD2D_ARENA_BYTES) { ++stats.dropped; return nullptr; }
        auto* fragment = new (arena[parity] + used) Fragment();
        used += size;
        return fragment;
    }
    static uint16_t page_word(const Texture& t, BlendMode blend) {
        return uint16_t(((t.x / 64) & 15) | (((t.y / 256) & 1) << 4) | ((blend == BlendMode::Cutout ? 0 : int(blend) - 1) << 5) |
                        ((t.bits == 4 ? 0 : 1) << 7));
    }
    static psyqo::Color color_of(const SpriteDraw& draw) {
        return {{.r = uint8_t((unsigned(draw.color[0]) + 1) / 2), .g = uint8_t((unsigned(draw.color[1]) + 1) / 2),
                 .b = uint8_t((unsigned(draw.color[2]) + 1) / 2)}};
    }

public:
    // Builds and chains this frame's 2D packets. Call after the 3D ordering table and
    // before the HUD.
    void draw(psyqo::GPU& gpu) {
        parity = gpu.getParity();
        used = 0;
        stats.drawn = stats.rectangles = stats.quads = stats.pages = 0;
        const size_t count = collect_sprites(draws, items, scratch, EPOK_WORLD2D_DRAW_ITEMS, [](int index, uint16_t& w, uint16_t& h) {
            const Texture* t = texture(index);
            if (!t) return false;
            w = t->width;
            h = t->height;
            return true;
        });
        uint32_t current_page = 0xffffffffu;
        for (size_t i = 0; i < count; ++i) {
            const SpriteDraw& d = draws[items[i].index];
            const Texture* t = texture(int(d.texture));
            if (!t) continue;
            const uint16_t base_u = texture_base_u(*t), base_v = uint16_t(t->y & 255);
            const uint16_t page = page_word(*t, d.blend);
            if (d.rectangle) {
                if (page != current_page) {
                    auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage>>();
                    if (!fragment) continue;
                    fragment->primitive.attr.setPageX(page & 15).setPageY((page >> 4) & 1)
                        .set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>((page >> 5) & 3))
                        .set((page >> 7) & 1 ? psyqo::Prim::TPageAttr::Tex8Bits : psyqo::Prim::TPageAttr::Tex4Bits)
                        .setDithering(false);
                    configure_display_field<display_interlaced>(fragment->primitive.attr);
                    gpu.chain(*fragment);
                    current_page = page;
                    ++stats.pages;
                }
                auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::Sprite>>();
                if (!fragment) continue;
                auto& p = fragment->primitive;
                p.setColor(color_of(d));
                if (d.blend == BlendMode::Cutout) p.setOpaque();
                else p.setSemiTrans();
                p.position = {{.x = d.x[0], .y = d.y[0]}};
                p.size = {{.w = int16_t(d.source[2]), .h = int16_t(d.source[3])}};
                p.texInfo.u = uint8_t(base_u + d.source[0]);
                p.texInfo.v = uint8_t(base_v + d.source[1]);
                p.texInfo.clut = texture_clut(*t);
                gpu.chain(*fragment);
                ++stats.rectangles;
            } else {
                auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad>>();
                if (!fragment) continue;
                auto& q = fragment->primitive;
                q.pointA = {{.x = d.x[0], .y = d.y[0]}};
                q.pointB = {{.x = d.x[1], .y = d.y[1]}};
                q.pointC = {{.x = d.x[2], .y = d.y[2]}};
                q.pointD = {{.x = d.x[3], .y = d.y[3]}};
                // The GPU omits a polygon's last column and row, so the span is a full texel count.
                auto clamp_uv = [](int value) { return uint8_t(value < 0 ? 0 : value > 255 ? 255 : value); };
                const int u0 = base_u + d.source[0], u1 = u0 + d.source[2], v0 = base_v + d.source[1], v1 = v0 + d.source[3];
                const int ul = d.flip_h ? u1 - 1 : u0, ur = d.flip_h ? u0 - 1 : u1;
                const int vt = d.flip_v ? v1 - 1 : v0, vb = d.flip_v ? v0 - 1 : v1;
                q.uvA.u = clamp_uv(ul); q.uvA.v = clamp_uv(vt);
                q.uvB.u = clamp_uv(ur); q.uvB.v = clamp_uv(vt);
                q.uvC.u = clamp_uv(ul); q.uvC.v = clamp_uv(vb);
                q.uvD.u = clamp_uv(ur); q.uvD.v = clamp_uv(vb);
                q.clutIndex = texture_clut(*t);
                q.tpage = texture_page(*t, d.blend);
                q.setColor(color_of(d));
                if (d.blend == BlendMode::Cutout) q.setOpaque();
                else q.setSemiTrans();
                gpu.chain(*fragment);
                current_page = page;  // a textured polygon also sets the drawing texpage
                ++stats.quads;
            }
            ++stats.drawn;
        }
        stats.arena_bytes = uint32_t(used);
    }
};
inline Renderer renderer;
}  // namespace world2d
}  // namespace epok
