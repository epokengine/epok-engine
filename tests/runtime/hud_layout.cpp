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
#include "../../runtime/hud_focus.hpp"

using namespace epok;
using hud_core::Affine2;
using hud_core::Rect;

namespace {
struct Recorder {
    // Every emitter writes one row. The axis-aligned kinds fill box/source; the
    // rotated kinds fill corner/texel, so one comparison covers both paths.
    struct Draw {
        int kind = 0, owner = -1, subject = -1;
        int x0 = 0, y0 = 0, x1 = 0, y1 = 0, u0 = 0, v0 = 0, u1 = 0, v1 = 0;
        int corner[8] = {}, texel[8] = {};
        uint8_t color[3] = {};
        bool operator==(const Draw&) const = default;
    };
    std::vector<Draw> commands;
    int texture_width = 0, texture_height = 0;
    bool texture_size(int id, int& w, int& h) {
        if (id < 0 || texture_width <= 0) return false;
        w = texture_width; h = texture_height; return true;
    }
    void rectangle(int owner, int x0, int y0, int x1, int y1, const uint8_t* c) {
        commands.push_back({0, owner, -1, x0, y0, x1, y1, 0, 0, 0, 0, {}, {}, {c[0], c[1], c[2]}});
    }
    void image(int owner, int id, int x0, int y0, int x1, int y1, int u0, int v0, int u1, int v1, const uint8_t* c) {
        commands.push_back({1, owner, id, x0, y0, x1, y1, u0, v0, u1, v1, {}, {}, {c[0], c[1], c[2]}});
    }
    void begin_text() {}
    void glyph(int owner, unsigned character, int x0, int y0, int x1, int y1, int u, int v, const uint8_t* c) {
        commands.push_back({2, owner, int(character), x0, y0, x1, y1, u, v, 0, 0, {}, {}, {c[0], c[1], c[2]}});
    }
    void quad(int owner, const int* x, const int* y, const uint8_t* c) { turned(3, owner, -1, x, y, nullptr, nullptr, c); }
    void textured_quad(int owner, int id, const int* x, const int* y, const int* u, const int* v, const uint8_t* c) { turned(4, owner, id, x, y, u, v, c); }
    void glyph_quad(int owner, unsigned character, const int* x, const int* y, const int* u, const int* v, const uint8_t* c) { turned(5, owner, int(character), x, y, u, v, c); }
private:
    void turned(int kind, int owner, int subject, const int* x, const int* y, const int* u, const int* v, const uint8_t* c) {
        Draw draw;
        draw.kind = kind; draw.owner = owner; draw.subject = subject;
        for (int i = 0; i < 4; ++i) { draw.corner[i * 2] = x[i]; draw.corner[i * 2 + 1] = y[i]; }
        if (u && v) for (int i = 0; i < 4; ++i) { draw.texel[i * 2] = u[i]; draw.texel[i * 2 + 1] = v[i]; }
        for (int i = 0; i < 3; ++i) draw.color[i] = c[i];
        commands.push_back(draw);
    }
};

constexpr size_t capacity = 40;
constexpr int screen_width = 320, screen_height = 240;
ActorData objects[capacity];
int first[capacity], next[capacity];
Fixed measured[capacity][2];
Rect rects[capacity];
Affine2 transforms[capacity];
Recorder sink;

void reset() {
    for (auto& object : objects) object = ActorData{};
    for (auto& rect : rects) rect = Rect{};
    sink.commands.clear();
    sink.texture_width = sink.texture_height = 0;
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
HudStats run(size_t count, bool emit, unsigned layout_budget = 64, unsigned rectangle_budget = 256, unsigned rotated_budget = 128) {
    sink.commands.clear();
    hud_core::Compiler compiler(sink, screen_width, screen_height, {layout_budget, rectangle_budget, 64, 1024, rotated_budget});
    if (emit) compiler.draw(objects, count, first, next, measured, rects, transforms);
    else compiler.layout(objects, count, first, next, measured, rects, transforms);
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


// A 200x100 panel over a 64x64 source, with one texture registered.
void tiled_panel(ImageTiling tiling) {
    canvas_with(1, 200, 100);
    sink.texture_width = sink.texture_height = 64;
    objects[1].image.enabled = true;
    objects[1].image.texture = 0;
    objects[1].image.tiling = tiling;
}

// (k) Tile repeats at the source's own size and clips the last column and row.
void tile_repeats_at_texel_size_and_clips_the_remainder() {
    tiled_panel(ImageTiling::Tile);
    const auto stats = run(2, true);
    assert(stats.images == 8 && stats.dropped == 0);
    assert(sink.commands.size() == 8);
    int full_columns = 0, cut_columns = 0, full_rows = 0, cut_rows = 0, bottom_source = 0;
    for (const auto& draw : sink.commands) {
        assert(draw.kind == 1);
        (draw.x1 - draw.x0 == 64 ? full_columns : cut_columns) += 1;
        (draw.y1 - draw.y0 == 64 ? full_rows : cut_rows) += 1;
        // The row that does not fit is cut at the top, so it shows the source's
        // bottom rows rather than a squashed whole tile.
        if (draw.y1 - draw.y0 == 36 && draw.v0 == 28 && draw.v1 == 63) ++bottom_source;
        assert(draw.x1 - draw.x0 == 64 || draw.x1 - draw.x0 == 8);
        assert(draw.y1 - draw.y0 == 64 || draw.y1 - draw.y0 == 36);
    }
    assert(full_columns == 6 && cut_columns == 2);
    assert(full_rows == 4 && cut_rows == 4);
    assert(bottom_source == 4);
}

// (l) TileFit rounds the count per axis and scales the tiles to fill exactly.
void tile_fit_rounds_the_count_and_leaves_no_gap() {
    tiled_panel(ImageTiling::TileFit);
    const auto stats = run(2, true);
    assert(stats.images == 6 && stats.dropped == 0);
    assert(sink.commands.size() == 6);
    int width = 0, height = 0;
    for (size_t i = 0; i < sink.commands.size(); ++i) {
        const auto& draw = sink.commands[i];
        // Every tile carries the whole source region; only its size differs.
        assert(draw.u0 == 0 && draw.u1 == 63 && draw.v0 == 0 && draw.v1 == 63);
        if (i < 3) width += draw.x1 - draw.x0;
        if (i % 3 == 0) height += draw.y1 - draw.y0;
    }
    assert(width == 200 && height == 100);
}

// (m) With nine-slice borders only the centre piece tiles.
void nine_slice_tiles_only_its_centre() {
    tiled_panel(ImageTiling::Tile);
    for (auto& border : objects[1].image.borders) border = 8;
    const auto stats = run(2, true);
    // Eight stretched frame pieces, plus a 184x84 centre over a 48x48 source.
    assert(stats.images == 16 && stats.dropped == 0);
    // The centre source spans u 8..55 and v 8..55; a tile of it ends at v 55,
    // which no stretched frame piece does.
    int centre_tiles = 0;
    for (const auto& draw : sink.commands)
        if (draw.u0 == 8 && draw.v1 == 55) ++centre_tiles;
    assert(centre_tiles == 8);
}

// (n) The rectangle budget stops the run of tiles and counts one drop.
void tiling_stops_when_the_rectangle_budget_runs_out() {
    tiled_panel(ImageTiling::Tile);
    const auto stats = run(2, true, 64, 5);
    assert(stats.images == 5 && stats.dropped == 1);
    assert(sink.commands.size() == 5);
}

// (o) A press edge follows the neighbour table, and nothing else moves focus.
void the_dpad_follows_the_neighbour_table() {
    canvas_with(3, 40, 16);
    objects[0].canvas.focused = 1;
    for (size_t i = 1; i <= 3; ++i) objects[i].focusable.enabled = true;
    objects[1].focusable.neighbors[1] = 2;
    objects[2].focusable.neighbors[1] = 3;
    const uint32_t right = 1u << unsigned(Button::Right), left = 1u << unsigned(Button::Left);
    hud_focus_update(objects, 4, right);
    assert(objects[0].canvas.focused == 2);
    // The third element is inactive, so the link to it is refused.
    objects[3].active = false;
    hud_focus_update(objects, 4, right);
    assert(objects[0].canvas.focused == 2);
    objects[3].active = true;
    hud_focus_update(objects, 4, right);
    assert(objects[0].canvas.focused == 3);
    // No left neighbour: focus stays where it is rather than clearing.
    hud_focus_update(objects, 4, left);
    assert(objects[0].canvas.focused == 3);
    // A frame with no direction edge changes nothing.
    hud_focus_update(objects, 4, 0);
    assert(objects[0].canvas.focused == 3);
}

// (p) The focused element's colours are multiplied by its highlight.
void focus_multiplies_the_colour_it_already_draws() {
    canvas_with(1, 40, 16);
    objects[0].canvas.focused = 1;
    objects[1].image.enabled = true;
    objects[1].image.color[0] = 200; objects[1].image.color[1] = 100; objects[1].image.color[2] = 50;
    objects[1].focusable.enabled = true;
    objects[1].focusable.highlight[0] = 128; objects[1].focusable.highlight[1] = 255; objects[1].focusable.highlight[2] = 64;
    run(2, true);
    assert(sink.commands.size() == 1 && sink.commands[0].kind == 0);
    assert(sink.commands[0].color[0] == 100 && sink.commands[0].color[1] == 100 && sink.commands[0].color[2] == 12);
    // Focus elsewhere and the authored bytes reach the sink untouched.
    objects[0].canvas.focused = -1;
    run(2, true);
    assert(sink.commands[0].color[0] == 200 && sink.commands[0].color[1] == 100 && sink.commands[0].color[2] == 50);
}

bool near(int value, int expected) {
    if (value >= expected - 1 && value <= expected + 1) return true;
    std::printf("  corner mismatch: actual %d expected %d\n", value, expected);
    return false;
}

// (q) A quarter turn about the pivot swaps the rect's axes on screen.
void a_quarter_turn_maps_the_corners_about_the_pivot() {
    canvas_with(1, 100, 40);
    objects[1].rect.rotation = 90.0;
    objects[1].image.enabled = true;
    const auto stats = run(2, true);
    assert(stats.rotated == 1 && stats.rectangles == 0 && stats.dropped == 0);
    assert(sink.commands.size() == 1 && sink.commands[0].kind == 3);
    // The rect is 100x40 centred on the 320x240 canvas, so the pivot is 160,120
    // and the corners land forty wide and a hundred tall around it.
    const int expected[8] = {140, 170, 140, 70, 180, 170, 180, 70};
    for (int i = 0; i < 8; ++i) assert(near(sink.commands[0].corner[i], expected[i]));
    // Layout itself is untouched: the axis-aligned rect is what it always was.
    assert(same(rects[1], 110, 100, 100, 40));
}

// (r) A child of a rotated panel turns with the panel.
void a_child_inherits_its_parents_rotation() {
    canvas_with(1, 100, 40);
    objects[1].rect.rotation = 90.0;
    objects[2].parent = 1;
    objects[2].rect.enabled = true;
    objects[2].rect.size[0] = 20.0;
    objects[2].rect.size[1] = 10.0;
    objects[2].image.enabled = true;
    const auto stats = run(3, true);
    assert(stats.rotated == 1);
    // The child carries the panel's transform, not one of its own.
    assert(transforms[2].c.raw() == transforms[1].c.raw() && transforms[2].s.raw() == transforms[1].s.raw());
    assert(transforms[2].tx.raw() == transforms[1].tx.raw() && transforms[2].ty.raw() == transforms[1].ty.raw());
    // Its own rect is the centred 20x10 box; turned a quarter about 160,120 the
    // top-left corner lands five left and ten below the pivot.
    assert(same(rects[2], 150, 115, 20, 10));
    const int expected[8] = {155, 130, 155, 110, 165, 130, 165, 110};
    for (int i = 0; i < 8; ++i) assert(near(sink.commands[0].corner[i], expected[i]));
}

// (s) The identity fast path is the old one, packet for packet. A full turn is
// still a rotation the author asked for, and it must come out unrotated.
void an_identity_transform_emits_exactly_the_old_primitives() {
    canvas_with(2, 120, 40);
    sink.texture_width = sink.texture_height = 32;
    objects[1].image.enabled = true;
    objects[1].image.texture = 0;
    objects[2].progress.enabled = true;
    objects[2].text.enabled = true;
    objects[2].text.set_text("HP 42");
    const auto plain = run(3, true);
    const auto before = sink.commands;
    assert(plain.rotated == 0 && !before.empty());
    for (const auto& draw : before) assert(draw.kind < 3);
    objects[1].rect.rotation = 360.0;
    objects[2].rect.rotation = -360.0;
    const auto turned = run(3, true);
    assert(turned.rotated == 0);
    assert(turned.rectangles == plain.rectangles && turned.images == plain.images);
    assert(turned.glyphs == plain.glyphs && turned.texts == plain.texts);
    assert(sink.commands == before);
}

// (t) Rotated primitives come out of their own pool and drop on their own.
void the_rotated_budget_drops_on_its_own() {
    canvas_with(3, 40, 16);
    for (size_t i = 1; i <= 3; ++i) {
        objects[i].image.enabled = true;
        objects[i].rect.rotation = 30.0;
    }
    const auto stats = run(4, true, 64, 256, 2);
    assert(stats.rotated == 2 && stats.dropped == 1 && stats.rectangles == 0);
    assert(sink.commands.size() == 2);
}

// (u) The editor writes these aggregates positionally, so a member added
// anywhere but the end would still compile there and mean something else here.
void the_exported_aggregates_keep_their_field_order() {
    const Image image{true, {1, 2, 3}, -1, {4, 5, 6, 7}, {8, 9, 10, 11}, ImageTiling::TileFit};
    assert(image.tiling == ImageTiling::TileFit && image.borders[3] == 11 && image.region[0] == 4);
    const RectTransform rect{true, {0.0, 0.0}, {1.0, 1.0}, {0.5, 0.5}, {2.0, 3.0}, {4.0, 5.0}, 45.0};
    assert(rect.rotation.raw() == 45 * 4096 && rect.size[1].raw() == 5 * 4096);
    const Focusable focus{true, {0, 1, 2, 3}, 4, {5, 6, 7}};
    assert(focus.order == 4 && focus.highlight[2] == 7 && focus.neighbors[3] == 3);
    const Canvas canvas{true, 9};
    assert(canvas.focused == 9);
}

// The reflected component identities are the ones the editor mirrors.
static_assert(FocusableComponent::static_class_id == 0x35443a7ad04b3989ull, "FocusableComponent identity");
static_assert(sizeof(ImageTiling) == 1, "ImageTiling must fit the reflected 32-bit enum rule trivially");
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
    tile_repeats_at_texel_size_and_clips_the_remainder();
    tile_fit_rounds_the_count_and_leaves_no_gap();
    nine_slice_tiles_only_its_centre();
    tiling_stops_when_the_rectangle_budget_runs_out();
    the_dpad_follows_the_neighbour_table();
    focus_multiplies_the_colour_it_already_draws();
    a_quarter_turn_maps_the_corners_about_the_pivot();
    a_child_inherits_its_parents_rotation();
    an_identity_transform_emits_exactly_the_old_primitives();
    the_rotated_budget_drops_on_its_own();
    the_exported_aggregates_keep_their_field_order();
    std::puts("Runtime HUD layout, tiling, focus and rotation tests passed.");
}
