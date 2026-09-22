// Host contract tests for the shared HUD measure/arrange/emit passes. The real
// runtime/hud_core.hpp is compiled; only the draw sink is a recorder. HUD space
// is +Y up, so the top edge of a rect is y + h and a Vertical box walks down.
#include <cassert>
#include <cstdio>
#include <vector>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/hud_core.hpp"

using namespace epok;
using hud_core::Rect;

namespace {
struct Recorder {
    struct Draw { int kind, owner, x0, y0, x1, y1; };
    std::vector<Draw> commands;
    bool texture_size(int, int&, int&) { return false; }
    void rectangle(int owner, int x0, int y0, int x1, int y1, const uint8_t*) { commands.push_back({0, owner, x0, y0, x1, y1}); }
    void image(int owner, int, int x0, int y0, int x1, int y1, int, int, int, int, const uint8_t*) { commands.push_back({1, owner, x0, y0, x1, y1}); }
    void begin_text() {}
    void glyph(int owner, unsigned, int x0, int y0, int x1, int y1, int, int, const uint8_t*) { commands.push_back({2, owner, x0, y0, x1, y1}); }
};

constexpr size_t capacity = 40;
constexpr int screen_width = 320, screen_height = 240;
ActorData objects[capacity];
int first[capacity], next[capacity];
Fixed measured[capacity][2];
Rect rects[capacity];
Recorder sink;

void reset() {
    for (auto& object : objects) object = ActorData{};
    for (auto& rect : rects) rect = Rect{};
    sink.commands.clear();
}
// A Canvas root plus `children` empty rect children of it, sized `width` x `height`.
void canvas_with(size_t children, int width, int height) {
    reset();
    objects[0].canvas.enabled = true;
    objects[0].parent = -1;
    for (size_t i = 1; i <= children; ++i) {
        objects[i].parent = 0;
        objects[i].rect.enabled = true;
        objects[i].rect.position[0] = objects[i].rect.position[1] = 0.0;
        objects[i].rect.size[0] = Fixed(width, 0);
        objects[i].rect.size[1] = Fixed(height, 0);
    }
}
HudStats run(size_t count, bool emit, unsigned layout_budget = 64) {
    sink.commands.clear();
    hud_core::Compiler compiler(sink, screen_width, screen_height, {layout_budget, 256, 64, 1024});
    if (emit) compiler.draw(objects, count, first, next, measured, rects);
    else compiler.layout(objects, count, first, next, measured, rects);
    return compiler.stats;
}
bool same(const Rect& r, int x, int y, int w, int h) {
    if (r.x.raw() == x * 4096 && r.y.raw() == y * 4096 && r.w.raw() == w * 4096 && r.h.raw() == h * 4096) return true;
    std::printf("  rect mismatch\n    actual:   %d %d %d %d\n    expected: %d %d %d %d\n",
                r.x.raw() / 4096, r.y.raw() / 4096, r.w.raw() / 4096, r.h.raw() / 4096, x, y, w, h);
    return false;
}
void set_element(size_t index, uint8_t horizontal, uint8_t vertical, double stretch) {
    auto& element = objects[index].layout_element;
    element.enabled = true;
    element.horizontal = horizontal;
    element.vertical = vertical;
    element.stretch = Fixed(int32_t(stretch * 4096), Fixed::RAW);
}
LayoutContainer& set_container(size_t index, LayoutKind kind) {
    auto& container = objects[index].layout_container;
    container.enabled = true;
    container.kind = kind;
    return container;
}

// (a) A subtree without a container resolves exactly as the anchor chain does,
// and the emit pass draws where those rects say.
void no_container_matches_the_anchor_chain() {
    canvas_with(1, 180, 64);
    objects[1].rect.anchor_min[0] = 0.0; objects[1].rect.anchor_min[1] = 1.0;
    objects[1].rect.anchor_max[0] = 0.0; objects[1].rect.anchor_max[1] = 1.0;
    objects[1].rect.pivot[0] = 0.0; objects[1].rect.pivot[1] = 1.0;
    objects[1].rect.position[0] = 12.0; objects[1].rect.position[1] = -12.0;
    objects[2].parent = 1;
    objects[2].rect.enabled = true;
    objects[2].rect.anchor_min[0] = objects[2].rect.anchor_min[1] = 0.0;
    objects[2].rect.anchor_max[0] = objects[2].rect.anchor_max[1] = 1.0;
    objects[2].rect.size[0] = -16.0; objects[2].rect.size[1] = -16.0;
    objects[2].image.enabled = true;
    const Rect screen{0.0, 0.0, Fixed(screen_width, 0), Fixed(screen_height, 0)};
    const Rect panel = hud_core::resolve(screen, objects[1].rect);
    const Rect child = hud_core::resolve(panel, objects[2].rect);
    run(3, true);
    assert(same(rects[0], 0, 0, screen_width, screen_height));
    assert(rects[1].x.raw() == panel.x.raw() && rects[1].y.raw() == panel.y.raw());
    assert(rects[1].w.raw() == panel.w.raw() && rects[1].h.raw() == panel.h.raw());
    assert(rects[2].x.raw() == child.x.raw() && rects[2].y.raw() == child.y.raw());
    assert(rects[2].w.raw() == child.w.raw() && rects[2].h.raw() == child.h.raw());
    // No texture is registered, so the image falls back to a flat fill.
    assert(sink.commands.size() == 1 && sink.commands[0].owner == 2);
    assert(sink.commands[0].x0 == hud_core::pixel(child.x));
    assert(sink.commands[0].y0 == screen_height - hud_core::pixel(child.y + child.h));
}

// (b) Horizontal expanders fill the box and split the leftover by stretch.
void horizontal_expanders_split_the_leftover() {
    canvas_with(4, 20, 10);
    objects[1].rect.size[0] = 240.0; objects[1].rect.size[1] = 60.0;
    set_container(1, LayoutKind::Horizontal);
    for (size_t i = 2; i <= 4; ++i) {
        objects[i].parent = 1;
        set_element(i, 1 | 2, 1, i == 4 ? 2.0 : 1.0);
    }
    run(5, false);
    assert(same(rects[1], 40, 90, 240, 60));
    assert(same(rects[2], 40, 90, 65, 60));
    assert(same(rects[3], 105, 90, 65, 60));
    assert(same(rects[4], 170, 90, 110, 60));
    // Nothing expands: the cells are the measured minimums, packed left.
    for (size_t i = 2; i <= 4; ++i) objects[i].layout_element.enabled = false;
    run(5, false);
    assert(same(rects[2], 40, 90, 20, 60));
    assert(same(rects[3], 60, 90, 20, 60));
    assert(same(rects[4], 80, 90, 20, 60));
}

// (c) +Y is up, so a Vertical box puts its first child at the highest y.
void vertical_puts_the_first_child_at_the_top() {
    canvas_with(3, 50, 20);
    objects[1].rect.size[0] = 200.0; objects[1].rect.size[1] = 120.0;
    auto& container = set_container(1, LayoutKind::Vertical);
    container.spacing[1] = 10.0;
    objects[2].parent = objects[3].parent = 1;
    run(4, false);
    assert(same(rects[1], 60, 60, 200, 120));
    assert(same(rects[2], 60, 160, 200, 20));
    assert(same(rects[3], 60, 130, 200, 20));
    assert(rects[2].y.raw() > rects[3].y.raw());
}

// (d) A two-column grid wraps the third child onto the second row, left column.
void grid_wraps_onto_the_next_row() {
    canvas_with(4, 50, 20);
    objects[1].rect.size[0] = 200.0; objects[1].rect.size[1] = 120.0;
    set_container(1, LayoutKind::Grid).columns = 2;
    for (size_t i = 2; i <= 4; ++i) objects[i].parent = 1;
    run(5, false);
    assert(same(rects[2], 60, 120, 100, 60));
    assert(same(rects[3], 160, 120, 100, 60));
    assert(same(rects[4], 60, 60, 100, 60));
    assert(rects[4].y.raw() < rects[2].y.raw());
}

// (e) Margin hands its child the rect shrunk by left/top/right/bottom padding.
void margin_shrinks_by_its_padding() {
    canvas_with(2, 50, 20);
    objects[1].rect.size[0] = 200.0; objects[1].rect.size[1] = 120.0;
    auto& container = set_container(1, LayoutKind::Margin);
    container.padding[0] = 10.0; container.padding[1] = 20.0;
    container.padding[2] = 30.0; container.padding[3] = 40.0;
    objects[2].parent = 1;
    run(3, false);
    assert(same(rects[2], 70, 100, 160, 60));
    // Its own rect is larger than the padding plus the child, so it floors the measurement.
    assert(measured[1][0].raw() == 200 * 4096 && measured[1][1].raw() == 120 * 4096);
    // A child larger than that rect puts the formula back on top.
    objects[2].rect.size[0] = 300.0;
    objects[2].rect.size[1] = 200.0;
    run(3, false);
    assert(measured[1][0].raw() == 340 * 4096 && measured[1][1].raw() == 260 * 4096);
    assert(same(rects[2], 70, 100, 160, 60));
}

// (f) Center centres a child that does not fill, and still fills one that does.
void center_centres_a_shrinking_child() {
    canvas_with(2, 50, 20);
    objects[1].rect.size[0] = 200.0; objects[1].rect.size[1] = 120.0;
    set_container(1, LayoutKind::Center);
    objects[2].parent = 1;
    set_element(2, 0, 0, 1.0);
    run(3, false);
    assert(same(rects[2], 135, 110, 50, 20));
    set_element(2, 1, 1, 1.0);
    run(3, false);
    assert(same(rects[2], 60, 60, 200, 120));
}

// (g) The size flags decide where a child sits inside the cell it was given.
void size_flags_place_the_child_inside_its_cell() {
    canvas_with(2, 50, 20);
    objects[1].rect.size[0] = 200.0; objects[1].rect.size[1] = 120.0;
    set_container(1, LayoutKind::Horizontal);
    objects[2].parent = 1;
    set_element(2, 1, 4, 1.0);
    run(3, false);
    assert(same(rects[2], 60, 110, 50, 20));
    set_element(2, 1, 8, 1.0);
    run(3, false);
    assert(same(rects[2], 60, 160, 50, 20));
    set_element(2, 1, 1, 1.0);
    run(3, false);
    assert(same(rects[2], 60, 60, 50, 120));
    // Begin alignment on the cross axis is the bottom edge, +Y up.
    set_element(2, 1, 0, 1.0);
    run(3, false);
    assert(same(rects[2], 60, 60, 50, 20));
}

// (h) The layout budget and the depth cap prune exactly as the single pass did.
void the_budget_and_depth_cap_still_prune() {
    canvas_with(3, 40, 20);
    for (size_t i = 1; i <= 3; ++i) objects[i].image.enabled = true;
    assert(run(4, true, 64).dropped == 0);
    assert(sink.commands.size() == 3);
    const auto stats = run(4, true, 2);
    assert(stats.dropped == 1 && sink.commands.size() == 2);
    // A chain deeper than the cap stops at depth 32, counted from the root.
    reset();
    objects[0].canvas.enabled = true;
    objects[0].parent = -1;
    for (size_t i = 1; i < capacity; ++i) {
        objects[i].parent = int(i) - 1;
        objects[i].rect.enabled = true;
        objects[i].rect.size[0] = 10.0;
        objects[i].rect.size[1] = 10.0;
    }
    run(capacity, false);
    assert(rects[32].w.raw() != 0);
    assert(rects[33].w.raw() == 0 && rects[capacity - 1].w.raw() == 0);
}

// (i) An inactive child takes no cell: the list closes up behind it and the box
// measures as if it were not there.
void an_inactive_child_takes_no_cell() {
    canvas_with(4, 50, 20);
    objects[1].rect.size[0] = 200.0;
    objects[1].rect.size[1] = 120.0;
    set_container(1, LayoutKind::Vertical).spacing[1] = 10.0;
    for (size_t i = 2; i <= 4; ++i) objects[i].parent = 1;
    objects[3].active = false;
    run(5, false);
    assert(same(rects[1], 60, 60, 200, 120));
    assert(same(rects[2], 60, 160, 200, 20));
    assert(rects[3].w.raw() == 0 && rects[3].h.raw() == 0);
    // One spacing between the survivors, not two plus the gap the hidden one left.
    assert(same(rects[4], 60, 130, 200, 20));
    // With no rect of its own to floor it, the box measures its live children only.
    objects[1].rect.size[0] = objects[1].rect.size[1] = 0.0;
    run(5, false);
    assert(measured[1][1].raw() == 50 * 4096);
    objects[3].active = true;
    run(5, false);
    assert(measured[1][1].raw() == 80 * 4096);
}

// (j) A nested container is floored by its own rect size like any other child.
void a_nested_container_honours_its_own_rect_size() {
    canvas_with(4, 60, 20);
    objects[1].rect.size[0] = 200.0;
    objects[1].rect.size[1] = 100.0;
    set_container(1, LayoutKind::Horizontal);
    objects[2].parent = 1;
    objects[2].rect.size[0] = 150.0;
    objects[2].rect.size[1] = 0.0;
    set_container(2, LayoutKind::Vertical);
    objects[3].parent = objects[4].parent = 2;
    run(5, false);
    // Content is 60 wide; the box asked for 150 and gets a cell that wide.
    assert(measured[2][0].raw() == 150 * 4096 && measured[2][1].raw() == 40 * 4096);
    assert(same(rects[2], 60, 70, 150, 100));
    assert(same(rects[3], 60, 150, 150, 20));
    assert(same(rects[4], 60, 130, 150, 20));
    objects[2].rect.size[0] = 0.0;
    run(5, false);
    assert(measured[2][0].raw() == 60 * 4096);
    assert(same(rects[2], 60, 70, 60, 100));
}

// The reflected component identities are the ones the editor mirrors.
static_assert(LayoutElementComponent::static_class_id == 0x7e1486803f88bac6ull, "LayoutElementComponent identity");
static_assert(LayoutContainerComponent::static_class_id == 0xf37f18bd6de6c69eull, "LayoutContainerComponent identity");
static_assert(sizeof(LayoutKind) == 1, "LayoutKind must fit the reflected 32-bit enum rule trivially");
}  // namespace

int main() {
    no_container_matches_the_anchor_chain();
    horizontal_expanders_split_the_leftover();
    vertical_puts_the_first_child_at_the_top();
    grid_wraps_onto_the_next_row();
    margin_shrinks_by_its_padding();
    center_centres_a_shrinking_child();
    size_flags_place_the_child_inside_its_cell();
    the_budget_and_depth_cap_still_prune();
    an_inactive_child_takes_no_cell();
    a_nested_container_honours_its_own_rect_size();
    std::puts("Runtime HUD layout container tests passed.");
}
