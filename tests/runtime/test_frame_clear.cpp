#include <cassert>
#include <cstring>
#include <vector>
#include <array>
#include <utility>
// Use the SDK's host chain links; the GPU primitive payloads remain unchanged.
#define PS1_PC_PORT 1
#include "frame_clear.hpp"

// Record real PsyQo primitive packets; no GPU driver or display is simulated.
struct Recorder {
    unsigned parity = 0;
    int width, height;
    std::vector<uint32_t> words;
    unsigned getParity() const { return parity; }
    void getNextClear(psyqo::Prim::FastFill& fill, psyqo::Color color) {
        assert(height == 240); // A 480i clear must never request a second buffer.
        fill.setColor(color);
        fill.rect = psyqo::Rect{0, int16_t(parity ? 0 : 256), int16_t(width), 240};
    }
    template<class Fragment> void chain(Fragment& fragment) {
        auto start = words.size();
        words.resize(start + sizeof(fragment.primitive) / sizeof(uint32_t));
        std::memcpy(words.data() + start, &fragment.primitive, sizeof(fragment.primitive));
    }
};

template<int Width, int Height> void check_mode() {
    epok::FrameClear<Width, Height> clear;
    Recorder gpu{0, Width, Height};
    for (unsigned field = 0; field < 4; ++field) {
        gpu.parity = field & 1;
        gpu.words.clear();
        clear.draw(gpu, psyqo::Color{{.r=33,.g=40,.b=52}});
        assert(gpu.words.size() == 4);
        assert(gpu.words[0] == (Height == 480 ? 0xe1000000u : 0xe1000400u));
        assert(gpu.words[1] == ((Height == 480 ? 0x60000000u : 0x02000000u) | 0x342821u));
        const unsigned y = Height == 480 ? 0 : gpu.parity ? 0 : 256;
        assert(gpu.words[2] == (y << 16));
        assert(gpu.words[3] == (unsigned(Height) << 16 | Width));
        assert(y + Height <= 512); // No vertical wrapping across VRAM.
    }
}

int main() {
    check_mode<256,240>(); check_mode<320,240>(); check_mode<368,240>();
    check_mode<512,240>(); check_mode<640,240>();
    check_mode<256,480>(); check_mode<320,480>(); check_mode<368,480>();
    check_mode<512,480>(); check_mode<640,480>();
    psyqo::PrimPieces::TPageAttr page;
    page.setPageX(15).setPageY(1).set(psyqo::Prim::TPageAttr::Tex4Bits)
        .set(psyqo::Prim::TPageAttr::FullBackSubFullFront).enableDisplayArea();
    epok::configure_display_field<true>(page);
    assert(!page.isDisplayAreaEnabled());
    assert(page.getPageX() == 15 && page.getPageY() == 1);
    assert(page.getSemiTrans() == psyqo::Prim::TPageAttr::FullBackSubFullFront);
    epok::configure_display_field<false>(page);
    assert(page.isDisplayAreaEnabled());
}
