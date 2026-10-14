#pragma once
// World2D tile worlds and canvas layers (World2D initiative, phase 2): cooked tile sets,
// TileMapLayer, Parallax2D, CanvasLayer2D and CanvasModulate2D.
//
// Tile maps. A TileMapLayer draws one cooked TileSet atlas on a grid anchored at its
// owner's world position (the owner's rotation and scale are ignored: a tile map is
// always axis-aligned and one texel per pixel at zoom 1). Cells live in 16 x 16 chunks;
// only chunks that hold a tile are cooked, and a small grid of chunk indices covers the
// used area. A cell value is 0 when empty, otherwise the tile index plus one in the low
// 13 bits and the flip/transpose flags in the high bits.
//
// Runtime edits. Cooked cells are constant. `set_cell` and `erase_cell` record overrides
// in a bounded table owned by the World2D service (EPOK_WORLD2D_TILE_EDITS entries for the
// whole map); loading a map clears them, so a restarted level shows its authored tiles.
//
// Canvas layers. Every drawable belongs to the canvas of its nearest CanvasLayer2D
// ancestor (through attachments), or to the default world canvas (layer 0). Layers draw
// in ascending order; a layer that does not follow the viewport is drawn in screen pixels,
// untouched by the Camera2D. A CanvasModulate2D tints everything on its canvas.
//
// Parallax. A Parallax2D moves its actor, and therefore everything attached to it, by
// (camera - reference) * (1 - scroll_scale) plus its autoscroll; with a repeat size the
// renderer repeats its drawables to cover the screen.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_components.hpp"

namespace epok {
class TileMapLayerComponent;
class Parallax2DComponent;
class CanvasLayer2DComponent;
class CanvasModulate2DComponent;

enum class TileCollision : uint8_t { None, Full, Rect };
inline constexpr uint8_t tile_one_way = 1;

// One tile of a cooked TileSet.
struct TileInfo {
    uint8_t atlas[2] = {0, 0};      // column, row in the atlas grid
    uint16_t u = 0, v = 0;          // top-left texel of the first animation frame
    TileCollision collision = TileCollision::None;
    uint8_t flags = 0;              // tile_one_way
    uint8_t rect[4] = {0, 0, 0, 0}; // collision rectangle inside the tile (x, y, w, h)
    uint8_t frames = 1;             // animation frames, laid out to the right of the tile
    uint8_t columns = 0;            // frames per atlas row before wrapping; 0 = never wraps
    uint16_t stride[2] = {0, 0};    // texels between two frames (tile size plus separation)
    Fixed frame_seconds = 0.0;
};
struct TileSetData {
    uint16_t tile_width = 16, tile_height = 16;
    uint16_t tile_count = 0;
    const TileInfo* tiles = nullptr;
    // `custom_count` integers per tile (custom data layers), tile-major.
    uint16_t custom_count = 0;
    const int32_t* custom = nullptr;
};

inline constexpr int tile_chunk_shift = 4;
inline constexpr int32_t tile_chunk_cells = 1 << (2 * tile_chunk_shift);
inline constexpr uint16_t tile_index_mask = 0x1fff;
inline constexpr uint16_t tile_flip_h = 0x2000, tile_flip_v = 0x4000, tile_transpose = 0x8000;

namespace world2d {
inline List<TileMapLayerComponent> tile_layers;
inline List<Parallax2DComponent> parallaxes;
inline List<CanvasLayer2DComponent> canvas_layers;
inline List<CanvasModulate2DComponent> canvas_modulates;
inline uint16_t tile_layer_serial = 0;

// Installed by the service: runtime cell edits live there, so a build without 2D content
// carries no edit table. Without them a layer shows exactly its cooked cells.
struct TileHooks {
    // Overrides `value` with the edit of (x, y), when one exists.
    void (*lookup)(const TileMapLayerComponent&, int32_t, int32_t, uint16_t&) = nullptr;
    // Records an edit; false when the edit table is full.
    bool (*store)(TileMapLayerComponent&, int32_t, int32_t, uint16_t) = nullptr;
    void (*forget)(TileMapLayerComponent&) = nullptr;
};
inline TileHooks tile_hooks;
// Seconds of animation time shared by every animated tile, advanced by the service.
inline Fixed tile_clock = 0.0;

// Floor division of a Q12 distance by a whole number of pixels.
inline int32_t floor_cells(int32_t raw, int32_t pixels) {
    const int32_t size = pixels * 4096;
    if (size <= 0) return 0;
    return raw >= 0 ? raw / size : -((-raw + size - 1) / size);
}
}  // namespace world2d

// ---- TileMapLayer -----------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=tilemap2d, Id="66f7568f-41eb-4d1b-8fbf-1cb55848a9ac") TileMapLayerComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("66f7568f-41eb-4d1b-8fbf-1cb55848a9ac");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override {
        world2d::tile_layers.add(this);
        if (!slot) {
            if (!++world2d::tile_layer_serial) ++world2d::tile_layer_serial;
            slot = world2d::tile_layer_serial;
        }
    }
    ~TileMapLayerComponent() override {
        world2d::tile_layers.remove(this);
        if (edit_count && world2d::tile_hooks.forget) world2d::tile_hooks.forget(*this);
    }
    world2d::Link<TileMapLayerComponent> link2d;

    // Authored state, cooked by the editor.
    const TileSetData* tileset = nullptr;
    int32_t texture = -1;
    const uint16_t* cells = nullptr;   // tile_chunk_cells values per cooked chunk
    const int16_t* grid = nullptr;     // chunk index per grid slot, -1 when empty
    int16_t grid_origin[2] = {0, 0};   // chunk coordinates of grid slot 0
    uint16_t grid_size[2] = {0, 0};
    uint8_t modulate[3] = {255, 255, 255};
    BlendMode blend = BlendMode::Cutout;
    int16_t z_index = 0;
    bool visible = true, collision_enabled = true;
    uint32_t collision_layer = 1;
    // Runtime identity in the edit table and the number of edits it holds there.
    uint16_t slot = 0, edit_count = 0;

    // The cooked value of a cell, ignoring runtime edits.
    uint16_t cooked_cell(int32_t x, int32_t y) const {
        if (!grid || !cells) return 0;
        const int32_t cx = (x >> tile_chunk_shift) - grid_origin[0], cy = (y >> tile_chunk_shift) - grid_origin[1];
        if (cx < 0 || cy < 0 || cx >= int32_t(grid_size[0]) || cy >= int32_t(grid_size[1])) return 0;
        const int16_t chunk = grid[cy * int32_t(grid_size[0]) + cx];
        if (chunk < 0) return 0;
        const int32_t mask = (1 << tile_chunk_shift) - 1;
        return cells[int32_t(chunk) * tile_chunk_cells + ((y & mask) << tile_chunk_shift) + (x & mask)];
    }
    // The value of a cell, runtime edits included.
    uint16_t cell(int32_t x, int32_t y) const {
        uint16_t value = cooked_cell(x, y);
        if (edit_count && world2d::tile_hooks.lookup) world2d::tile_hooks.lookup(*this, x, y, value);
        return value;
    }
    const TileInfo* tile_of(uint16_t value) const {
        const uint16_t index = value & tile_index_mask;
        return index && tileset && index <= tileset->tile_count ? &tileset->tiles[index - 1] : nullptr;
    }
    // World pixels of the grid origin (cell 0, 0's top-left corner). False without a root.
    bool origin(Fixed* out) const {
        Affine2D matrix;
        if (!world2d::world_of(const_cast<TileMapLayerComponent&>(*this), matrix)) return false;
        out[0] = matrix.t[0];
        out[1] = matrix.t[1];
        return true;
    }
    uint16_t tile_width() const { return tileset && tileset->tile_width ? tileset->tile_width : 16; }
    uint16_t tile_height() const { return tileset && tileset->tile_height ? tileset->tile_height : 16; }

    EPOK_FUNCTION(BlueprintPure, Id="786e7084-75c5-45b4-a3e1-39bab5730ff2") bool is_cell_empty(int32_t x, int32_t y) const { return !tile_of(cell(x, y)); }
    // Atlas column of the cell's tile, or -1 when the cell is empty.
    EPOK_FUNCTION(BlueprintPure, Id="8c9bca7e-35e2-4ff5-bd65-fe937868deb5") int32_t get_cell_atlas_x(int32_t x, int32_t y) const {
        const auto* tile = tile_of(cell(x, y));
        return tile ? int32_t(tile->atlas[0]) : -1;
    }
    // Atlas row of the cell's tile, or -1 when the cell is empty.
    EPOK_FUNCTION(BlueprintPure, Id="0e2e8dab-e522-4dd5-873f-301bb2bd4a2c") int32_t get_cell_atlas_y(int32_t x, int32_t y) const {
        const auto* tile = tile_of(cell(x, y));
        return tile ? int32_t(tile->atlas[1]) : -1;
    }
    // Whether the cell's tile blocks bodies (full, rectangle or one-way collision).
    EPOK_FUNCTION(BlueprintPure, Id="07420b11-617a-495f-9077-b22229d080b6") bool is_cell_solid(int32_t x, int32_t y) const {
        const auto* tile = tile_of(cell(x, y));
        return tile && tile->collision != TileCollision::None;
    }
    // Custom data layer `layer` of the cell's tile; 0 for an empty cell or unknown layer.
    EPOK_FUNCTION(BlueprintPure, Id="a358d378-1e3f-4c64-af3b-f33a2ff13d60") int32_t get_cell_custom_data(int32_t x, int32_t y, uint32_t layer) const {
        const uint16_t index = cell(x, y) & tile_index_mask;
        if (!index || !tileset || index > tileset->tile_count || layer >= tileset->custom_count || !tileset->custom) return 0;
        return tileset->custom[uint32_t(index - 1) * tileset->custom_count + layer];
    }
    // Places the atlas tile (atlas_x, atlas_y) in a cell. False when the tile set has no such
    // tile or the map's runtime edit table is full.
    EPOK_FUNCTION(BlueprintCallable, Id="a84d5bcc-fc20-4036-ba42-a31a7ed18e18") bool set_cell(int32_t x, int32_t y, int32_t atlas_x, int32_t atlas_y) {
        if (!tileset) return false;
        for (uint16_t i = 0; i < tileset->tile_count; ++i)
            if (tileset->tiles[i].atlas[0] == atlas_x && tileset->tiles[i].atlas[1] == atlas_y) return store(x, y, uint16_t(i + 1));
        return false;
    }
    // Same as set_cell, keeping the flip flags: flip_h, flip_v and transpose.
    EPOK_FUNCTION(BlueprintCallable, Id="0fdcae03-9d2f-4404-b1ea-93426f02af16") bool set_cell_flipped(int32_t x, int32_t y, int32_t atlas_x, int32_t atlas_y, bool flip_h, bool flip_v, bool transpose) {
        if (!tileset) return false;
        const uint16_t flags = uint16_t((flip_h ? tile_flip_h : 0) | (flip_v ? tile_flip_v : 0) | (transpose ? tile_transpose : 0));
        for (uint16_t i = 0; i < tileset->tile_count; ++i)
            if (tileset->tiles[i].atlas[0] == atlas_x && tileset->tiles[i].atlas[1] == atlas_y) return store(x, y, uint16_t((i + 1) | flags));
        return false;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="c2919b78-83a1-4718-a39b-2971991c796f") bool erase_cell(int32_t x, int32_t y) { return store(x, y, 0); }
    // The cell column containing world X / the cell row containing world Y.
    EPOK_FUNCTION(BlueprintPure, Id="c2391270-85de-4522-b5f7-f8fcc380ba7a") int32_t world_to_map_x(Fixed world_x) const {
        Fixed o[2] = {0.0, 0.0};
        origin(o);
        return world2d::floor_cells(world_x.raw() - o[0].raw(), tile_width());
    }
    EPOK_FUNCTION(BlueprintPure, Id="21f588c9-e000-4a81-8714-1f07206d9b74") int32_t world_to_map_y(Fixed world_y) const {
        Fixed o[2] = {0.0, 0.0};
        origin(o);
        return world2d::floor_cells(world_y.raw() - o[1].raw(), tile_height());
    }
    // World X / Y of a cell's centre.
    EPOK_FUNCTION(BlueprintPure, Id="87332fd2-2bd7-4701-811d-8393609847ea") Fixed map_to_world_x(int32_t x) const {
        Fixed o[2] = {0.0, 0.0};
        origin(o);
        return o[0] + Fixed(x * int32_t(tile_width()) * 4096 + int32_t(tile_width()) * 2048, Fixed::RAW);
    }
    EPOK_FUNCTION(BlueprintPure, Id="e24d9b1f-31f0-4ca4-9f91-c1682ac4341e") Fixed map_to_world_y(int32_t y) const {
        Fixed o[2] = {0.0, 0.0};
        origin(o);
        return o[1] + Fixed(y * int32_t(tile_height()) * 4096 + int32_t(tile_height()) * 2048, Fixed::RAW);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="0c7255d7-ddff-432f-a4f5-372c15df5c27") void set_visible(bool value) { visible = value; }
    EPOK_FUNCTION(BlueprintPure, Id="f609a114-983b-479b-a2d0-f5e6c2fdc398") bool is_visible() const { return visible; }
    EPOK_FUNCTION(BlueprintCallable, Id="4d309fe9-f4bf-46b4-980d-db9681bb6922") void set_collision_enabled(bool value) { collision_enabled = value; }
    EPOK_FUNCTION(BlueprintCallable, Id="8a88d0ee-54d2-4e2d-9a6c-bcde505efb99") void set_modulate(uint32_t red, uint32_t green, uint32_t blue) {
        modulate[0] = uint8_t(red > 255 ? 255 : red);
        modulate[1] = uint8_t(green > 255 ? 255 : green);
        modulate[2] = uint8_t(blue > 255 ? 255 : blue);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="49db49cb-34b8-46bc-9182-705c4989d024") void set_z_index(int32_t value) {
        z_index = int16_t(value < -4096 ? -4096 : value > 4096 ? 4096 : value);
    }

private:
    bool store(int32_t x, int32_t y, uint16_t value) {
        return world2d::tile_hooks.store && world2d::tile_hooks.store(*this, x, y, value);
    }
};

// ---- Parallax2D -------------------------------------------------------------------------
//
// Moves its actor so the layer scrolls at `scroll_scale` times the camera's speed: 1 moves
// with the world, 0 stays on screen, 0.5 is a distant layer. `reference` is the camera
// centre the authored layout was placed for (the editor cooks the map's starting view),
// so the game shows the layer where it was placed when the camera is there.
// `autoscroll` adds a constant drift in pixels per second. With a repeat size the layer's
// drawables repeat every `repeat_size` pixels on that axis, filling the screen.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=parallax2d, Id="24128b3f-e0c0-4617-9154-55f2cb577ff4") Parallax2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("24128b3f-e0c0-4617-9154-55f2cb577ff4");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::parallaxes.add(this); }
    ~Parallax2DComponent() override { world2d::parallaxes.remove(this); }
    world2d::Link<Parallax2DComponent> link2d;

    Fixed scroll_scale[2] = {1.0, 1.0};
    Fixed repeat_size[2] = {0.0, 0.0};
    Fixed autoscroll[2] = {0.0, 0.0};
    Fixed reference[2] = {0.0, 0.0};
    // Resolved state, owned by the service.
    Fixed base[2] = {0.0, 0.0}, drift[2] = {0.0, 0.0};
    bool initialized = false;

    EPOK_FUNCTION(BlueprintCallable, Id="6d941a43-e30e-4edf-ba7e-80d33c88b9dc") void set_scroll_scale(Fixed x, Fixed y) { scroll_scale[0] = x; scroll_scale[1] = y; }
    EPOK_FUNCTION(BlueprintCallable, Id="96c2e7b6-8317-4e43-8e70-7f6d0366e5b0") void set_autoscroll(Fixed x, Fixed y) { autoscroll[0] = x; autoscroll[1] = y; }
    EPOK_FUNCTION(BlueprintCallable, Id="5a09f5d1-2906-43c9-bc94-5a370dfb4042") void set_repeat_size(Fixed x, Fixed y) {
        repeat_size[0] = x < 0.0 ? Fixed(0.0) : x;
        repeat_size[1] = y < 0.0 ? Fixed(0.0) : y;
    }
    // One simulation step: `camera` is the active camera's centre (or 0, 0 without one).
    void follow(const Fixed* camera, Fixed dt) {
        auto* root = world2d::root_of(*this);
        if (!root) return;
        if (!initialized) {
            base[0] = root->transform.position[0];
            base[1] = root->transform.position[1];
            drift[0] = drift[1] = 0.0;
            initialized = true;
        }
        for (int k = 0; k < 2; ++k) {
            drift[k] += autoscroll[k] * dt;
            // A repeating layer only needs its drift modulo one period.
            if (repeat_size[k] > 0.0) {
                const int32_t period = repeat_size[k].raw();
                int32_t value = drift[k].raw() % period;
                if (value < 0) value += period;
                drift[k] = Fixed(value, Fixed::RAW);
            }
            const Fixed follow = (camera[k] - reference[k]) * (Fixed(1.0) - scroll_scale[k]);
            root->transform.position[k] = base[k] + follow + drift[k];
        }
        transform2d_clamp(root->transform);
    }
};

// ---- CanvasLayer2D ----------------------------------------------------------------------
//
// Puts its actor and everything attached to it on canvas `layer`: layers draw in ascending
// order and the default world canvas is layer 0. Unless it follows the viewport, the
// layer is drawn in screen pixels (a world position of 0, 0 is the screen's top-left
// corner, shifted by `offset`) and ignores the Camera2D: HUD sprites, fixed backgrounds.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=canvas2d, Id="bd384e4b-5903-485c-b114-5ffb6a330538") CanvasLayer2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("bd384e4b-5903-485c-b114-5ffb6a330538");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::canvas_layers.add(this); }
    ~CanvasLayer2DComponent() override { world2d::canvas_layers.remove(this); }
    world2d::Link<CanvasLayer2DComponent> link2d;

    int8_t layer = 1;
    Fixed offset[2] = {0.0, 0.0};
    bool follow_viewport = false;
    bool visible = true;

    EPOK_FUNCTION(BlueprintCallable, Id="fa31c08c-8328-4ca1-b974-1fe0268963f6") void set_layer(int32_t value) { layer = int8_t(value < -64 ? -64 : value > 64 ? 64 : value); }
    EPOK_FUNCTION(BlueprintPure, Id="9001123d-07a1-4b00-a6e6-c755becdd9b3") int32_t get_layer() const { return layer; }
    EPOK_FUNCTION(BlueprintCallable, Id="07cfc937-d5ff-4b87-8e1f-09d1d447be1a") void set_offset(Fixed x, Fixed y) { offset[0] = x; offset[1] = y; }
    EPOK_FUNCTION(BlueprintCallable, Id="16c698ed-9f18-4ebb-ac68-b260c0c4bf34") void set_visible(bool value) { visible = value; }
    EPOK_FUNCTION(BlueprintPure, Id="bba393bd-dbea-414a-8736-103d85ec856d") bool is_visible() const { return visible; }
    EPOK_FUNCTION(BlueprintCallable, Id="08977ad7-6887-4685-8c2d-b60d0baa8180") void set_follow_viewport(bool value) { follow_viewport = value; }
};

// ---- CanvasModulate2D -------------------------------------------------------------------
//
// Multiplies the colour of everything drawn on its canvas (255 leaves it unchanged): night
// scenes, flashes, fades of a whole layer. The first active one on a canvas wins.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=canvas2d, Id="8611633e-7385-4b5e-9f25-92d524dd2005") CanvasModulate2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("8611633e-7385-4b5e-9f25-92d524dd2005");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::canvas_modulates.add(this); }
    ~CanvasModulate2DComponent() override { world2d::canvas_modulates.remove(this); }
    world2d::Link<CanvasModulate2DComponent> link2d;

    uint8_t color[3] = {255, 255, 255};

    EPOK_FUNCTION(BlueprintCallable, Id="795a16b2-f1a3-48bb-84a8-3adfb0fa7c27") void set_color(uint32_t red, uint32_t green, uint32_t blue) {
        color[0] = uint8_t(red > 255 ? 255 : red);
        color[1] = uint8_t(green > 255 ? 255 : green);
        color[2] = uint8_t(blue > 255 ? 255 : blue);
    }
};

namespace world2d {
// The canvas a drawable belongs to, resolved through its owner's attachment chain.
struct Canvas {
    int8_t layer = 0;
    bool screen = false;           // drawn in screen pixels, ignoring the camera
    Fixed offset[2] = {0.0, 0.0};  // screen-space shift of a screen canvas
    bool visible = true;
    const Parallax2DComponent* parallax = nullptr;
};
// Walks from `component`'s owner up its attachments. The nearest CanvasLayer2D decides the
// canvas; the nearest Parallax2D below it decides the repetition.
inline Canvas canvas_of(ActorComponent& component) {
    Canvas canvas;
    if (!canvas_layers.head && !parallaxes.head) return canvas;
    const SceneComponent2D* node = root_of(component);
    Actor* owner = component.get_owner();
    for (size_t depth = 0; owner && depth < world2d_depth_limit; ++depth) {
        if (!canvas.parallax)
            parallaxes.each([&](Parallax2DComponent& parallax) {
                if (!canvas.parallax && parallax.get_owner() == owner && is_active(&parallax)) canvas.parallax = &parallax;
            });
        bool found = false;
        canvas_layers.each([&](CanvasLayer2DComponent& layer) {
            if (found || layer.get_owner() != owner || !is_active(&layer)) return;
            found = true;
            canvas.layer = layer.layer;
            canvas.screen = !layer.follow_viewport;
            canvas.offset[0] = layer.offset[0];
            canvas.offset[1] = layer.offset[1];
            canvas.visible = layer.visible;
        });
        if (found || !node || !node->attach_parent.valid()) break;
        node = resolve_scene2d(node->attach_parent);
        owner = node ? const_cast<SceneComponent2D*>(node)->get_owner() : nullptr;
    }
    return canvas;
}
}  // namespace world2d
}  // namespace epok
