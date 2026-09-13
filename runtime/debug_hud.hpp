#pragma once
#include "debug-hud.hh"

#if EPOK_DEBUG_FPS || EPOK_DEBUG_CPU || EPOK_DEBUG_GTE || EPOK_DEBUG_GPU || EPOK_DEBUG_SPU
#include <array>
#include "psyqo/gpu.hh"
#include "psyqo/kernel.hh"
#include "psyqo/fragments.hh"
#include "psyqo/primitives/rectangles.hh"
#include "psyqo/primitives/sprites.hh"
#include "common/hardware/counters.h"
#include "frame_clear.hpp"
#include "audio.hpp"

namespace epok::debug_hud {
constexpr unsigned bars = EPOK_DEBUG_CPU + EPOK_DEBUG_GTE + EPOK_DEBUG_GPU + EPOK_DEBUG_SPU;
constexpr unsigned glyphs = EPOK_DEBUG_FPS * 8 + bars * 3;
struct State {
    // Existing HUD font/CLUT in VRAM; fixed double-buffered packets, no heap.
    std::array<psyqo::Fragments::SimpleFragment<psyqo::Prim::Sprite>, glyphs> letters[2];
    std::array<psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle>, bars * 2 + 1> rectangles[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2];
    uint16_t frame_start = 0;
#if EPOK_DEBUG_FPS
    uint32_t sample_vblank = 0, frames = 0, fps_tenths = 0;
#endif
#if EPOK_DEBUG_GTE
    uint32_t geometry_lines = 0;
#endif
#if EPOK_DEBUG_GPU
    volatile uint16_t dma_start = 0, dma_lines = 0;
    volatile bool dma_pending = false;
    bool dma_registered = false;
#endif
#if EPOK_DEBUG_SPU
    uint32_t spu_bytes = 4096;
#endif
};
inline State state;
static_assert(sizeof(State) < 2048, "Debug overlay must stay below 2 KiB of packet/state RAM");
inline void initialize(psyqo::GPU& gpu) {
#if EPOK_DEBUG_FPS
    state.sample_vblank = gpu.getFrameCount();
#endif
#if EPOK_DEBUG_GPU
    if (state.dma_registered) return;
    state.dma_registered = true;
    // PsyQo's own handler runs first. This observes command DMA completion,
    // NOT rasterizer utilization. No logging/allocation/waits inside the ISR.
    psyqo::Kernel::registerDmaEvent(psyqo::Kernel::DMA::GPU, [&gpu]() {
        if (state.dma_pending && !gpu.isChainTransferring()) {
            state.dma_lines = uint16_t(COUNTERS[1].value - state.dma_start);
            state.dma_pending = false;
        }
    });
#endif
#if EPOK_DEBUG_SPU
    state.spu_bytes = audio_ready ? audio_upload_address : 4096;
#endif
}
inline void begin(psyqo::GPU& gpu) {
    state.frame_start = COUNTERS[1].value;
#if EPOK_DEBUG_GTE
    state.geometry_lines = 0;
#endif
#if EPOK_DEBUG_GPU
    state.dma_pending = false;
    state.dma_lines = 0;
    state.dma_start = state.frame_start;
    state.dma_pending = gpu.isChainTransferring();
    // Completion may have interrupted the previous read before assignment.
    if (!gpu.isChainTransferring()) state.dma_pending = false;
#endif
#if EPOK_DEBUG_FPS
    ++state.frames;
    const uint32_t elapsed = gpu.getFrameCount() - state.sample_vblank;
    if (elapsed >= 30) {
        state.fps_tenths = state.frames * 600 / elapsed;
        state.frames = 0;
        state.sample_vblank = gpu.getFrameCount();
    }
#endif
}
inline void geometry(uint16_t started, bool gte) {
#if EPOK_DEBUG_GTE
    // CPU preparation + GTE projection of mesh vertices, not GTE busy cycles.
    if (gte) state.geometry_lines += uint16_t(COUNTERS[1].value - started);
#endif
}
inline void draw(psyqo::GPU& gpu) {
    const uint32_t cpu_lines = uint16_t(COUNTERS[1].value - state.frame_start);
    const unsigned parity = gpu.getParity();
    const int left = display_width / 20, bottom = display_height - display_height / 20;
    constexpr int width = 184, row_height = 16;
    int top = bottom - int(bars + EPOK_DEBUG_FPS) * row_height;
    unsigned rect = 0, letter = 0;
    auto rectangle = [&](int x, int y, int w, int h, psyqo::Color color) {
        auto& f = state.rectangles[parity][rect++];
        f.primitive.position = {{.x=int16_t(x), .y=int16_t(y)}};
        f.primitive.size = {{.w=int16_t(w), .h=int16_t(h)}};
        f.primitive.setColor(color).setOpaque();
        gpu.chain(f);
    };
    auto text = [&](const char* s, int y) {
        int x = left + 4;
        for (; *s && letter < glyphs; ++s, x += 8) {
            const unsigned c = unsigned(*s) - 32;
            auto& f = state.letters[parity][letter++]; auto& p = f.primitive;
            p.position = {{.x=int16_t(x), .y=int16_t(y)}}; p.size = {{.w=8, .h=16}};
            p.texInfo.u = (c % 32) * 8; p.texInfo.v = 192 + (c / 32) * 16;
            p.texInfo.clut = psyqo::PrimPieces::ClutIndex(60, 448);
            p.setColor({{.r=128, .g=128, .b=128}}).setOpaque(); gpu.chain(f);
        }
    };
    auto bar = [&](const char* label, uint32_t value, uint32_t limit) {
        text(label, top);
        rectangle(left + 36, top + 5, width - 40, 6, {{.r=40,.g=40,.b=48}});
        const int filled = int((value > limit ? limit : value) * (width - 40) / limit);
        if (filled) rectangle(left + 36, top + 5, filled, 6,
            value >= limit ? psyqo::Color{{.r=240,.g=64,.b=48}} : psyqo::Color{{.r=64,.g=208,.b=120}});
        top += row_height;
    };
    auto& page = state.pages[parity];
    page.primitive.attr.setPageX(15).setPageY(1).set(psyqo::Prim::TPageAttr::Tex4Bits).setDithering(false);
    configure_display_field<display_interlaced>(page.primitive.attr); gpu.chain(page);
    rectangle(left, top, width, bottom - top, {{.r=8,.g=8,.b=16}});
#if EPOK_DEBUG_FPS
    const unsigned fps = state.fps_tenths > 999 ? 999 : state.fps_tenths;
    char label[] = "FPS 00.0";
    label[4] = '0' + fps / 100; label[5] = '0' + (fps / 10) % 10; label[7] = '0' + fps % 10;
    text(label, top); top += row_height;
#endif
    // Time bars use one NTSC vblank budget (~263 scanlines / 16.7 ms).
    // Saturation is red; CPU includes waits inside Scene::frame, excludes flip.
#if EPOK_DEBUG_CPU
    bar("CPU", cpu_lines, 263);
#endif
#if EPOK_DEBUG_GTE
    bar("GTE", state.geometry_lines, 263);
#endif
#if EPOK_DEBUG_GPU
    const uint32_t dma = state.dma_pending ? uint16_t(COUNTERS[1].value - state.dma_start) : state.dma_lines;
    bar("GPU", dma, 263);
#endif
#if EPOK_DEBUG_SPU
    bar("SPU", state.spu_bytes, 512 * 1024);
#endif
}
} // namespace epok::debug_hud
#else
namespace epok::debug_hud {
inline void initialize(psyqo::GPU&) {}
inline void begin(psyqo::GPU&) {}
inline void geometry(uint16_t, bool) {}
inline void draw(psyqo::GPU&) {}
}
#endif
