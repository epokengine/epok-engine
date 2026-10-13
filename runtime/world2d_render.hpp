#pragma once
// World2D renderer (World2D initiative). Sprites and tile layers are drawn after the 3D
// world and before the HUD, back to front by canvas layer, then a stable key: z-index (the
// owner's draw_order plus the drawable's z_index) first, then, for y-sorted sprites, the
// world Y of their origin; equal keys keep creation order.
//
// An unrotated, unscaled, unflipped sprite seen through an unrotated camera at zoom 1 is
// emitted as a GP0 rectangle (`Sprite`: exact texels, less setup); everything else becomes
// a textured quad. Tiles follow the same rule, and 8 x 8 and 16 x 16 tiles use the GPU's
// fixed-size rectangles, the cheapest primitive it has.
//
// Capacities come from the cooked scene header (defaults below):
//   EPOK_WORLD2D_DRAW_ITEMS    sprites, tile layers and parallax copies drawn per frame
//   EPOK_WORLD2D_ARENA_BYTES   GPU packet bytes per frame parity
// Packets are placed in a per-parity byte arena, so rectangles, quads and page changes
// share one budget; exceeding it drops the extra sprites and tiles and counts them.
//
// SubViewport2D pictures are drawn first, each into its VRAM rectangle (drawing area and
// offset moved there, then the texture cache flushed), so the views drawn after sample
// this frame's picture. A SplitScreen2D draws the world once per half, each clipped to
// its half, then the screen canvases once over both.
#include "world2d_service.hpp"
#include "texture.hpp"
#include "frame_clear.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/control.hh"
#include "psyqo/primitives/quads.hh"
#include "psyqo/primitives/sprites.hh"
#include "psyqo/primitives/triangles.hh"
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
    uint32_t current_page = 0xffffffffu;

    template<class Fragment> Fragment* allocate() {
        constexpr size_t size = (sizeof(Fragment) + 3) & ~size_t(3);
        if (used + size > EPOK_WORLD2D_ARENA_BYTES) { ++stats.dropped; return nullptr; }
        auto* fragment = new (arena[parity] + used) Fragment();
        used += size;
        return fragment;
    }
    static uint16_t page_word(const Texture& t, BlendMode blend) {
        return uint16_t(((t.x / 64) & 15) | (((t.y / 256) & 1) << 4) | ((blend == BlendMode::Cutout ? 0 : int(blend) - 1) << 5) |
                        ((t.bits == 4 ? 0 : t.bits == 8 ? 1 : 2) << 7));
    }
    // Untextured primitives draw their colour as is (textured ones treat 128 as 1.0).
    static psyqo::Color flat_color(const uint8_t* color) { return {{.r = color[0], .g = color[1], .b = color[2]}}; }
    static psyqo::Color color_of(const uint8_t* color) {
        return {{.r = uint8_t((unsigned(color[0]) + 1) / 2), .g = uint8_t((unsigned(color[1]) + 1) / 2),
                 .b = uint8_t((unsigned(color[2]) + 1) / 2)}};
    }
    // Rectangles draw with the current texture page: change it only when it differs.
    bool use_page(psyqo::GPU& gpu, uint16_t page) {
        if (page == current_page) return true;
        auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage>>();
        if (!fragment) return false;
        fragment->primitive.attr.setPageX(page & 15).setPageY((page >> 4) & 1)
            .set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>((page >> 5) & 3))
            .set(((page >> 7) & 3) == 2   ? psyqo::Prim::TPageAttr::Tex16Bits
                 : ((page >> 7) & 3) == 1 ? psyqo::Prim::TPageAttr::Tex8Bits
                                           : psyqo::Prim::TPageAttr::Tex4Bits)
            .setDithering(false);
        configure_display_field<display_interlaced>(fragment->primitive.attr);
        gpu.chain(*fragment);
        current_page = page;
        ++stats.pages;
        return true;
    }
    // A fixed-size or sized GP0 rectangle with exact texels.
    template<class Prim> bool rectangle(psyqo::GPU& gpu, const Texture& t, BlendMode blend, const uint8_t* color, int16_t x, int16_t y,
                                        uint16_t u, uint16_t v, uint16_t w, uint16_t h) {
        auto* fragment = allocate<psyqo::Fragments::SimpleFragment<Prim>>();
        if (!fragment) return false;
        auto& p = fragment->primitive;
        p.setColor(color_of(color));
        if (blend == BlendMode::Cutout) p.setOpaque();
        else p.setSemiTrans();
        p.position = {{.x = x, .y = y}};
        if constexpr (requires { p.size; }) p.size = {{.w = int16_t(w), .h = int16_t(h)}};
        p.texInfo.u = uint8_t(texture_base_u(t) + u);
        p.texInfo.v = uint8_t((t.y & 255) + v);
        p.texInfo.clut = texture_clut(t);
        gpu.chain(*fragment);
        ++stats.rectangles;
        return true;
    }
    // A textured quad: corners A, B, C, D and the texel rectangle mapped onto them.
    bool quad(psyqo::GPU& gpu, const Texture& t, BlendMode blend, const uint8_t* color, const int16_t* x, const int16_t* y,
              uint16_t u, uint16_t v, uint16_t w, uint16_t h, bool flip_h, bool flip_v, bool transpose) {
        auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad>>();
        if (!fragment) return false;
        auto& q = fragment->primitive;
        q.pointA = {{.x = x[0], .y = y[0]}};
        q.pointB = {{.x = x[1], .y = y[1]}};
        q.pointC = {{.x = x[2], .y = y[2]}};
        q.pointD = {{.x = x[3], .y = y[3]}};
        // The GPU omits a polygon's last column and row, so the span is a full texel count.
        auto clamp_uv = [](int value) { return uint8_t(value < 0 ? 0 : value > 255 ? 255 : value); };
        const int base_u = texture_base_u(t), base_v = t.y & 255;
        const int u0 = base_u + u, u1 = u0 + w, v0 = base_v + v, v1 = v0 + h;
        const int ul = flip_h ? u1 - 1 : u0, ur = flip_h ? u0 - 1 : u1;
        const int vt = flip_v ? v1 - 1 : v0, vb = flip_v ? v0 - 1 : v1;
        // Corner texels before a transpose: A (ul, vt), B (ur, vt), C (ul, vb), D (ur, vb).
        // Transposing swaps the texture's axes, which exchanges the B and C texels.
        q.uvA.u = clamp_uv(ul); q.uvA.v = clamp_uv(vt);
        q.uvD.u = clamp_uv(ur); q.uvD.v = clamp_uv(vb);
        if (transpose) {
            q.uvB.u = clamp_uv(ul); q.uvB.v = clamp_uv(vb);
            q.uvC.u = clamp_uv(ur); q.uvC.v = clamp_uv(vt);
        } else {
            q.uvB.u = clamp_uv(ur); q.uvB.v = clamp_uv(vt);
            q.uvC.u = clamp_uv(ul); q.uvC.v = clamp_uv(vb);
        }
        q.clutIndex = texture_clut(t);
        q.tpage = texture_page(t, blend);
        q.setColor(color_of(color));
        if (blend == BlendMode::Cutout) q.setOpaque();
        else q.setSemiTrans();
        gpu.chain(*fragment);
        current_page = page_word(t, blend);  // a textured polygon also sets the drawing texpage
        ++stats.quads;
        return true;
    }
    // Untextured primitives take their semi-transparency mode from the drawing page.
    static uint16_t blend_page(BlendMode blend) { return uint16_t((blend == BlendMode::Cutout ? 0 : int(blend) - 1) << 5); }
    void lines(psyqo::GPU& gpu, const SpriteDraw& d) {
        if (d.blend != BlendMode::Cutout && !use_page(gpu, blend_page(d.blend))) return;
        each_line_quad(d, [&](const ShapeQuad& shape) {
            auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudQuad>>();
            if (!fragment) return;
            auto& q = fragment->primitive;
            q.pointA = {{.x = shape.x[0], .y = shape.y[0]}};
            q.pointB = {{.x = shape.x[1], .y = shape.y[1]}};
            q.pointC = {{.x = shape.x[2], .y = shape.y[2]}};
            q.pointD = {{.x = shape.x[3], .y = shape.y[3]}};
            q.setColorA(flat_color(shape.color[0]));
            q.setColorB(flat_color(shape.color[1]));
            q.setColorC(flat_color(shape.color[2]));
            q.setColorD(flat_color(shape.color[3]));
            if (d.blend == BlendMode::Cutout) q.setOpaque();
            else q.setSemiTrans();
            gpu.chain(*fragment);
            ++stats.quads;
            ++stats.drawn;
        });
    }
    void polygon(psyqo::GPU& gpu, const SpriteDraw& d) {
        const auto& shape = *static_cast<const Polygon2DComponent*>(d.component);
        uint8_t color[3];
        for (int k = 0; k < 3; ++k) color[k] = uint8_t((unsigned(shape.color[k]) * d.color[k] + 127) / 255);
        const Texture* t = shape.texels ? texture(int(d.texture)) : nullptr;
        if (!t && d.blend != BlendMode::Cutout && !use_page(gpu, blend_page(d.blend))) return;
        each_polygon_triangle(d, [&](const ShapeTriangle& tri) {
            if (t) {
                auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedTriangle>>();
                if (!fragment) return;
                auto& p = fragment->primitive;
                p.pointA = {{.x = tri.x[0], .y = tri.y[0]}};
                p.pointB = {{.x = tri.x[1], .y = tri.y[1]}};
                p.pointC = {{.x = tri.x[2], .y = tri.y[2]}};
                const int base_u = texture_base_u(*t), base_v = t->y & 255;
                auto uv = [](int value) { return uint8_t(value < 0 ? 0 : value > 255 ? 255 : value); };
                p.uvA.u = uv(base_u + tri.u[0]); p.uvA.v = uv(base_v + tri.v[0]);
                p.uvB.u = uv(base_u + tri.u[1]); p.uvB.v = uv(base_v + tri.v[1]);
                p.uvC.u = uv(base_u + tri.u[2]); p.uvC.v = uv(base_v + tri.v[2]);
                p.clutIndex = texture_clut(*t);
                p.tpage = texture_page(*t, d.blend);
                p.setColor(color_of(color));
                if (d.blend == BlendMode::Cutout) p.setOpaque();
                else p.setSemiTrans();
                gpu.chain(*fragment);
                current_page = page_word(*t, d.blend);
            } else {
                auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::Triangle>>();
                if (!fragment) return;
                auto& p = fragment->primitive;
                p.pointA = {{.x = tri.x[0], .y = tri.y[0]}};
                p.pointB = {{.x = tri.x[1], .y = tri.y[1]}};
                p.pointC = {{.x = tri.x[2], .y = tri.y[2]}};
                p.setColor(flat_color(color));
                if (d.blend == BlendMode::Cutout) p.setOpaque();
                else p.setSemiTrans();
                gpu.chain(*fragment);
            }
            ++stats.drawn;
        });
    }
    // A resolved sprite (or particle): an exact rectangle when possible, else a quad.
    void sprite(psyqo::GPU& gpu, const SpriteDraw& d, const Texture& t) {
        bool ok;
        if (d.rectangle) {
            if (!use_page(gpu, page_word(t, d.blend))) return;
            ok = rectangle<psyqo::Prim::Sprite>(gpu, t, d.blend, d.color, d.x[0], d.y[0], d.source[0], d.source[1], d.source[2], d.source[3]);
        } else {
            ok = quad(gpu, t, d.blend, d.color, d.x, d.y, d.source[0], d.source[1], d.source[2], d.source[3], d.flip_h, d.flip_v, false);
        }
        if (ok) ++stats.drawn;
    }
    void tiles(psyqo::GPU& gpu, const SpriteDraw& d, const Texture& t) {
        const uint16_t page = page_word(t, d.blend);
        each_tile(d, t.width, t.height, [&](const TileDraw& tile) {
            bool ok;
            if (tile.rectangle) {
                if (!use_page(gpu, page)) return;
                if (tile.w == 16 && tile.h == 16)
                    ok = rectangle<psyqo::Prim::Sprite16x16>(gpu, t, d.blend, d.color, tile.x[0], tile.y[0], tile.u, tile.v, 16, 16);
                else if (tile.w == 8 && tile.h == 8)
                    ok = rectangle<psyqo::Prim::Sprite8x8>(gpu, t, d.blend, d.color, tile.x[0], tile.y[0], tile.u, tile.v, 8, 8);
                else
                    ok = rectangle<psyqo::Prim::Sprite>(gpu, t, d.blend, d.color, tile.x[0], tile.y[0], tile.u, tile.v, tile.w, tile.h);
            } else {
                ok = quad(gpu, t, d.blend, d.color, tile.x, tile.y, tile.u, tile.v, tile.w, tile.h, tile.flip_h, tile.flip_v,
                          tile.transpose);
            }
            if (ok) ++stats.drawn;
        });
    }

    // A SubViewport2D's picture as a 15-bit texture.
    static const Texture* picture_of(const SpriteDraw& d, Texture& out) {
        const auto* picture = static_cast<const SubViewport2DComponent*>(d.component);
        if (!picture) return nullptr;
        out = Texture{};
        out.width = picture->width;
        out.height = picture->height;
        out.x = picture->target_x;
        out.y = picture->target_y;
        out.word_width = picture->width;
        out.bits = 16;
        return &out;
    }
    template<class Prim> bool control(psyqo::GPU& gpu, const Prim& primitive) {
        auto* fragment = allocate<psyqo::Fragments::SimpleFragment<Prim>>();
        if (!fragment) return false;
        fragment->primitive = primitive;
        gpu.chain(*fragment);
        return true;
    }
    // Drawing area (absolute VRAM) and drawing offset.
    bool scissor(psyqo::GPU& gpu, int16_t x, int16_t y, int16_t w, int16_t h, int16_t ox, int16_t oy) {
        psyqo::Prim::Scissor s;
        s.start = psyqo::Prim::DrawingAreaStart(psyqo::Vertex{{.x = x, .y = y}});
        s.end = psyqo::Prim::DrawingAreaEnd(psyqo::Vertex{{.x = int16_t(x + w), .y = int16_t(y + h)}});
        s.offset = psyqo::Prim::DrawingOffset(psyqo::Vertex{{.x = ox, .y = oy}});
        return control(gpu, s);
    }
    // A flat rectangle in drawing coordinates.
    void panel(psyqo::GPU& gpu, int16_t x, int16_t y, int16_t w, int16_t h, const uint8_t* color) {
        auto* fragment = allocate<psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle>>();
        if (!fragment) return;
        fragment->primitive.position = {{.x = x, .y = y}};
        fragment->primitive.size = {{.w = w, .h = h}};
        fragment->primitive.setColor(flat_color(color)).setOpaque();
        gpu.chain(*fragment);
    }
    // Collects the current render pass and chains its packets.
    void view(psyqo::GPU& gpu) {
        const size_t count = collect_sprites(draws, items, scratch, EPOK_WORLD2D_DRAW_ITEMS, [](int index, uint16_t& w, uint16_t& h) {
            const Texture* t = texture(index);
            if (!t) return false;
            w = t->width;
            h = t->height;
            return true;
        });
        for (size_t i = 0; i < count; ++i) {
            const SpriteDraw& d = draws[items[i].index];
            if (d.kind == DrawKind::Line) {
                lines(gpu, d);
                continue;
            }
            if (d.kind == DrawKind::Polygon) {
                polygon(gpu, d);
                continue;
            }
            Texture picture;
            const Texture* t = d.texture == -2 ? picture_of(d, picture) : texture(int(d.texture));
            if (!t) continue;
            if (d.kind == DrawKind::Tiles) tiles(gpu, d, *t);
            else if (d.kind == DrawKind::Particles) each_particle(d, t->width, t->height, [&](const SpriteDraw& p) { sprite(gpu, p, *t); });
            else sprite(gpu, d, *t);
        }
    }

public:
    // Builds and chains this frame's 2D packets. Call after the 3D ordering table and
    // before the HUD.
    void draw(psyqo::GPU& gpu) {
        parity = gpu.getParity();
        used = 0;
        current_page = 0xffffffffu;
        stats.drawn = stats.rectangles = stats.quads = stats.pages = 0;
        // Chained packets draw into the framebuffer after the one on screen.
        const int16_t base = display_interlaced ? int16_t(0) : int16_t(parity ? 0 : 256);
        const int16_t width = int16_t(display_width), height = int16_t(display_height);
        RenderPass main;
        bool moved = false;
        subviewports.each([&](SubViewport2DComponent& picture) {
            if (picture.hide_layer && picture.layer != every_layer) main.hide(int8_t(picture.layer));
            if (!picture.enabled || !is_active(&picture) || !picture.width || !picture.height) return;
            if (picture.countdown) {
                --picture.countdown;
                return;
            }
            picture.countdown = uint8_t(picture.interval > 1 ? picture.interval - 1 : 0);
            Camera2D through;
            if (!picture.view(through)) return;
            psyqo::Prim::FastFill fill;
            fill.setColor(flat_color(picture.background));
            fill.rect = psyqo::Rect{int16_t(picture.target_x), int16_t(picture.target_y), int16_t(picture.width), int16_t(picture.height)};
            if (!control(gpu, fill)) return;
            scissor(gpu, int16_t(picture.target_x), int16_t(picture.target_y), int16_t(picture.width), int16_t(picture.height),
                    int16_t(picture.target_x), int16_t(picture.target_y));
            RenderPass pass;
            pass.camera = &through;
            pass.screen = false;
            pass.only_layer = picture.layer;
            pass.clip[2] = int16_t(picture.width);
            pass.clip[3] = int16_t(picture.height);
            render_pass = pass;
            current_page = 0xffffffffu;
            view(gpu);
            moved = true;
        });
        if (moved) {
            control(gpu, psyqo::Prim::FlushCache{});
            scissor(gpu, 0, base, width, height, 0, base);
            current_page = 0xffffffffu;
        }
        if (SplitScreen2DComponent* split = active_split()) {
            for (int i = 0; i < 2; ++i) {
                Camera2D through;
                if (!split->view(i, through)) continue;
                const int16_t* r = through.viewport;
                scissor(gpu, r[0], int16_t(base + r[1]), r[2], r[3], 0, base);
                if (Camera2DComponent* camera = camera_on(split->cameras[i])) panel(gpu, r[0], r[1], r[2], r[3], camera->background);
                RenderPass pass = main;
                pass.camera = &through;
                pass.screen = false;
                for (int k = 0; k < 4; ++k) pass.clip[k] = r[k];
                render_pass = pass;
                view(gpu);
            }
            scissor(gpu, 0, base, width, height, 0, base);
            if (split->divider) {
                int16_t first[4];
                split->half(0, first);
                if (split->side_by_side) panel(gpu, first[2], 0, int16_t(split->divider), height, split->divider_color);
                else panel(gpu, 0, first[3], width, int16_t(split->divider), split->divider_color);
            }
            main.world = false;
        }
        render_pass = main;
        view(gpu);
        render_pass = RenderPass{};
        stats.arena_bytes = uint32_t(used);
    }
};
inline Renderer renderer;
}  // namespace world2d
}  // namespace epok
