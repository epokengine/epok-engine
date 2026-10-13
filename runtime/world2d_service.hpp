#pragma once
// World2D service (World2D initiative): the collision world behind Collider2D,
// CharacterBody2D and tile maps, trigger dispatch, Camera2D following, parallax, sprite
// and tile animation, runtime tile edits and the draw list shared by the native 2D
// renderer and the PC runner. Only the generated application includes this header; it
// installs `world2d::hooks` so components reach it without depending on it.
//
// Capacities come from the cooked scene header (defaults below):
//   EPOK_WORLD2D_COLLIDERS     colliders resolved per step (CollisionWorld2D capacity)
//   EPOK_WORLD2D_TILE_PROBES   tiles one movement or ray may test against
//   EPOK_WORLD2D_TILE_EDITS    runtime tile edits remembered for the loaded map
//   EPOK_WORLD2D_TRIGGER_PAIRS remembered overlapping trigger pairs
//   EPOK_WORLD2D_DRAW_ITEMS    sprites, tile layers and repeated copies drawn per frame
//   EPOK_WORLD2D_ARENA_BYTES   GPU packet bytes per frame parity
// Exceeding any of them drops the extra work and counts it in `world2d::stats`.
//
// Tile collision. A movement or ray gathers the solid tiles it can touch (the cells under
// the sweep, or the cells a ray crosses in order) into temporary collision entries after
// the colliders, so tiles and colliders resolve through one solver: tiles block exactly
// like boxes, one-way tiles like one-way platforms, and seams between neighbouring tiles
// never catch a sliding body.
//
// Rendering lives in world2d_render.hpp; this header has no GPU dependency, so host tests
// exercise the collision, trigger, camera, parallax and animation step directly.
#include "world2d_components.hpp"

#ifndef EPOK_WORLD2D_COLLIDERS
#define EPOK_WORLD2D_COLLIDERS 64
#endif
#ifndef EPOK_WORLD2D_TILE_PROBES
#define EPOK_WORLD2D_TILE_PROBES 48
#endif
#ifndef EPOK_WORLD2D_TILE_EDITS
#define EPOK_WORLD2D_TILE_EDITS 128
#endif
#ifndef EPOK_WORLD2D_TRIGGER_PAIRS
#define EPOK_WORLD2D_TRIGGER_PAIRS 64
#endif
#ifndef EPOK_WORLD2D_DRAW_ITEMS
#define EPOK_WORLD2D_DRAW_ITEMS 256
#endif

namespace epok {
namespace world2d {

struct Stats {
    uint32_t sprites = 0, drawn = 0, rectangles = 0, quads = 0, pages = 0, culled = 0, dropped = 0;
    uint32_t arena_bytes = 0, colliders = 0, dropped_colliders = 0, trigger_pairs = 0, dropped_trigger_pairs = 0;
    uint32_t tile_layers = 0, tiles = 0, tile_probes = 0, dropped_tile_probes = 0, tile_edits = 0, dropped_tile_edits = 0;
};
inline Stats stats;

// Stable LSD radix sort of draw items by canvas layer, then their 32-bit key. Passes whose
// byte is equal for every item are skipped, so a scene drawn on one z level and one canvas
// costs two passes at most.
struct DrawItem {
    uint32_t key;
    uint16_t index;
    uint8_t layer;
};
inline void sort_draw_items(DrawItem* items, DrawItem* scratch, size_t count) {
    if (count < 2) return;
    DrawItem* from = items;
    DrawItem* to = scratch;
    for (int pass = 0; pass < 5; ++pass) {
        auto byte = [pass](const DrawItem& item) { return pass < 4 ? (item.key >> (pass * 8)) & 255 : uint32_t(item.layer); };
        uint16_t buckets[257] = {};
        for (size_t i = 0; i < count; ++i) ++buckets[byte(from[i]) + 1];
        bool trivial = false;
        for (int b = 1; b <= 256; ++b)
            if (buckets[b] == count) trivial = true;
        if (trivial) continue;
        for (int b = 0; b < 256; ++b) buckets[b + 1] += buckets[b];
        for (size_t i = 0; i < count; ++i) to[buckets[byte(from[i])]++] = from[i];
        DrawItem* swap = from;
        from = to;
        to = swap;
    }
    if (from != items)
        for (size_t i = 0; i < count; ++i) items[i] = from[i];
}

// Camera projection with the rotation resolved once per frame (world2d.hpp's
// world_to_screen recomputes the sine and cosine at every call).
struct Projector {
    Fixed sine = 0.0, cosine = 1.0, scale = 1.0, center[2] = {0.0, 0.0}, origin[2] = {0.0, 0.0};
    bool identity = true;
    Camera2D camera;
    explicit Projector(const Camera2D& view) : camera(view) {
        sine = sin_degrees(-view.rotation);
        cosine = cos_degrees(-view.rotation);
        scale = camera2d_pixels_per_unit(view);
        camera2d_viewport_center(view, center);
        origin[0] = view.position[0];
        origin[1] = view.position[1];
        identity = !view.rotation.raw() && scale.raw() == 4096;
    }
    // A projection that maps world pixels straight to screen pixels (screen canvases).
    static Projector screen() {
        Camera2D view;
        view.position[0] = Fixed(int32_t(display_width) * 2048, Fixed::RAW);
        view.position[1] = Fixed(int32_t(display_height) * 2048, Fixed::RAW);
        return Projector(view);
    }
    void point(const Fixed* world, Fixed* screen) const {
        const Fixed dx = world[0] - origin[0], dy = world[1] - origin[1];
        if (identity) {
            screen[0] = center[0] + dx;
            screen[1] = center[1] + dy;
            return;
        }
        const Fixed rx = detail::mul_2d(dx, cosine) - detail::mul_2d(dy, sine);
        const Fixed ry = detail::mul_2d(dx, sine) + detail::mul_2d(dy, cosine);
        screen[0] = center[0] + detail::mul_2d(rx, scale);
        screen[1] = center[1] + detail::mul_2d(ry, scale);
    }
};
inline int16_t screen_pixel(Fixed value) {
    int32_t v = (value.raw() + 2048) >> 12;
    if (v < -32768) v = -32768;
    if (v > 32767) v = 32767;
    return int16_t(v);
}

// World box of one tile's collision shape, honouring the cell's flip and transpose flags.
// A cell maps screen to texels by transposing first (when set), then mirroring the texel
// axes: flip_h mirrors the tile's columns, flip_v its rows.
inline bool tile_box(const TileInfo& tile, uint16_t value, const Fixed* cell_origin, uint16_t width, uint16_t height,
                     Aabb2D& out) {
    if (tile.collision == TileCollision::None) return false;
    int32_t x = 0, y = 0, w = width, h = height;
    if (tile.collision == TileCollision::Rect) {
        x = tile.rect[0];
        y = tile.rect[1];
        w = tile.rect[2];
        h = tile.rect[3];
        // Flips mirror the tile's texels, then a transpose swaps its axes (the renderer
        // samples the same way, so the shape stays on the pixels it was drawn for).
        if (value & tile_flip_h) x = int32_t(width) - x - w;
        if (value & tile_flip_v) y = int32_t(height) - y - h;
        if (value & tile_transpose) {
            int32_t t = x; x = y; y = t;
            t = w; w = h; h = t;
        }
        if (w <= 0 || h <= 0) return false;
    }
    out.min[0] = cell_origin[0] + Fixed(x * 4096, Fixed::RAW);
    out.min[1] = cell_origin[1] + Fixed(y * 4096, Fixed::RAW);
    out.max[0] = out.min[0] + Fixed(w * 4096, Fixed::RAW);
    out.max[1] = out.min[1] + Fixed(h * 4096, Fixed::RAW);
    return true;
}
inline int64_t floor_div64(int64_t value, int64_t size) {
    return value >= 0 ? value / size : -((-value + size - 1) / size);
}

class Service {
    static constexpr size_t probe_base = EPOK_WORLD2D_COLLIDERS;
    CollisionWorld2D<EPOK_WORLD2D_COLLIDERS + EPOK_WORLD2D_TILE_PROBES, EPOK_WORLD2D_TRIGGER_PAIRS> world;
    Collider2DComponent* slots[EPOK_WORLD2D_COLLIDERS] = {};
    size_t slot_count = 0;
    uint32_t synced_tick = 0xffffffffu;
    struct Probe {
        TileMapLayerComponent* layer;
        int32_t x, y;
    };
    Probe probes[EPOK_WORLD2D_TILE_PROBES] = {};
    size_t probe_count = 0;
    struct TileEdit {
        uint16_t layer;
        int16_t y, x;
        uint16_t value;
    };
    TileEdit edits[EPOK_WORLD2D_TILE_EDITS] = {};
    size_t edit_count = 0;

    static bool owner_active(ActorComponent& component) { return is_active(&component); }
    static ObjectId owner_of(ActorComponent* component) {
        auto* owner = component ? component->get_owner() : nullptr;
        return owner ? owner->id() : ObjectId{};
    }
    // World centre and scaled extents of one collider.
    static bool entry_for(Collider2DComponent& collider, ColliderEntry2D& entry) {
        Affine2D matrix;
        if (!world_of(collider, matrix)) return false;
        entry.collider = collider.shape;
        Fixed center[2];
        matrix.point(collider.shape.offset, center);
        entry.collider.offset[0] = entry.collider.offset[1] = 0.0;
        for (int k = 0; k < 2; ++k) {
            entry.position[k] = center[k];
            // Column length of the linear part: the owner's scale on that axis.
            const int64_t a = matrix.m[0][k].raw(), b = matrix.m[1][k].raw();
            int64_t length = a * a + b * b, root = 4096;
            if (length != int64_t(4096) * 4096) {
                root = 0;
                for (int64_t bit = int64_t(1) << 30; bit; bit >>= 1)
                    if ((root + bit) * (root + bit) <= length) root += bit;
            }
            entry.collider.half_extents[k] = Fixed(int32_t(int64_t(collider.shape.half_extents[k].raw()) * root / 4096), Fixed::RAW);
        }
        if (entry.collider.shape == Collider2D::Circle) {
            const Fixed sx = entry.collider.half_extents[0], sy = entry.collider.half_extents[1];
            const Fixed base = collider.shape.half_extents[0].raw() ? collider.shape.half_extents[0] : Fixed(1.0);
            const Fixed factor = (sx > sy ? sx : sy) / base;
            entry.collider.radius = collider.shape.radius * factor;
        }
        entry.generation = collider.id().generation;
        entry.active = true;
        return true;
    }
    // Sum of the rotations along the owner's attachment chain, in degrees.
    static Fixed world_rotation(ActorComponent& component) {
        Fixed total = 0.0;
        const SceneComponent2D* node = root_of(component);
        for (size_t depth = 0; node && depth < world2d_depth_limit; ++depth) {
            total += node->transform.rotation;
            node = node->attach_parent.valid() ? resolve_scene2d(node->attach_parent) : nullptr;
        }
        return total;
    }
    static Collider2DComponent* solid_collider_of(ActorComponent& body) {
        auto* owner = body.get_owner();
        Collider2DComponent* found = nullptr;
        colliders.each([&](Collider2DComponent& collider) {
            if (!found && collider.get_owner() == owner && !collider.shape.trigger && collider.shape.enabled) found = &collider;
        });
        return found;
    }
    // A world-space displacement applied to the owner's root, through the inverse of the
    // linear part of the root's parent chain when the root is attached.
    static bool displace_owner(ActorComponent& body, const Fixed* delta) {
        auto* root = root_of(body);
        if (!root) return false;
        Fixed local[2] = {delta[0], delta[1]};
        if (root->attach_parent.valid()) {
            const SceneComponent2D* parent = resolve_scene2d(root->attach_parent);
            Affine2D matrix;
            if (parent && world_matrix_2d(*parent, [](ObjectId id) { return resolve_scene2d(id); }, matrix)) {
                const int64_t a = matrix.m[0][0].raw(), b = matrix.m[0][1].raw(), c = matrix.m[1][0].raw(), d = matrix.m[1][1].raw();
                const int64_t determinant = (a * d - b * c) / 4096;
                if (!determinant) return false;
                const int64_t x = delta[0].raw(), y = delta[1].raw();
                local[0] = detail::clamp_raw_2d((d * x - b * y) / determinant);
                local[1] = detail::clamp_raw_2d((a * y - c * x) / determinant);
            }
        }
        root->transform.position[0] += local[0];
        root->transform.position[1] += local[1];
        transform2d_clamp(root->transform);
        return true;
    }

    // ---- tile probes -------------------------------------------------------------------
    static bool collides(TileMapLayerComponent& layer, uint32_t mask) {
        return layer.collision_enabled && (layer.collision_layer & mask) && layer.tileset && owner_active(layer);
    }
    void clear_probes() {
        for (size_t i = 0; i < probe_count; ++i) world.disable(probe_base + i);
        if (probe_count) world.shrink();
        probe_count = 0;
    }
    // Adds the collision box of cell (x, y) as a temporary entry. False when the probe table
    // is full.
    bool probe(TileMapLayerComponent& layer, const Fixed* origin, int32_t x, int32_t y, const Aabb2D* filter) {
        const uint16_t value = layer.cell(x, y);
        const TileInfo* tile = layer.tile_of(value);
        if (!tile) return true;
        const uint16_t w = layer.tile_width(), h = layer.tile_height();
        const Fixed corner[2] = {origin[0] + Fixed(x * int32_t(w) * 4096, Fixed::RAW), origin[1] + Fixed(y * int32_t(h) * 4096, Fixed::RAW)};
        Aabb2D box;
        if (!tile_box(*tile, value, corner, w, h, box)) return true;
        if (filter && !aabb2d_touch(*filter, box)) return true;
        if (probe_count >= EPOK_WORLD2D_TILE_PROBES) {
            ++stats.dropped_tile_probes;
            return false;
        }
        ColliderEntry2D entry;
        entry.collider.shape = Collider2D::Box;
        entry.collider.half_extents[0] = (box.max[0] - box.min[0]) / 2;
        entry.collider.half_extents[1] = (box.max[1] - box.min[1]) / 2;
        entry.collider.layer = layer.collision_layer;
        entry.collider.one_way = (tile->flags & tile_one_way) != 0;
        entry.position[0] = box.min[0] + entry.collider.half_extents[0];
        entry.position[1] = box.min[1] + entry.collider.half_extents[1];
        entry.generation = layer.id().generation;
        world.set(probe_base + probe_count, entry);
        probes[probe_count++] = Probe{&layer, x, y};
        ++stats.tile_probes;
        return true;
    }
    // Every solid tile of `layer` touching `box`.
    void probe_layer(TileMapLayerComponent& layer, const Aabb2D& box, uint32_t mask) {
        Fixed origin[2];
        if (!collides(layer, mask) || !layer.origin(origin)) return;
        const int32_t w = layer.tile_width(), h = layer.tile_height();
        int32_t x0 = floor_cells(box.min[0].raw() - origin[0].raw(), w), x1 = floor_cells(box.max[0].raw() - origin[0].raw(), w);
        int32_t y0 = floor_cells(box.min[1].raw() - origin[1].raw(), h), y1 = floor_cells(box.max[1].raw() - origin[1].raw(), h);
        // A single sweep never needs more than a screen of cells.
        if (x1 - x0 > 63) x1 = x0 + 63;
        if (y1 - y0 > 63) y1 = y0 + 63;
        for (int32_t y = y0; y <= y1; ++y)
            for (int32_t x = x0; x <= x1; ++x)
                if (!probe(layer, origin, x, y, &box)) return;
    }
    void probe_box(const Aabb2D& box, uint32_t mask) {
        tile_layers.each([&](TileMapLayerComponent& layer) { probe_layer(layer, box, mask); });
    }
    // The cells a segment crosses, nearest first (grid traversal), per layer.
    void probe_ray(const Fixed* start, const Fixed* delta, uint32_t mask) {
        tile_layers.each([&](TileMapLayerComponent& layer) {
            Fixed origin[2];
            if (!collides(layer, mask) || !layer.origin(origin)) return;
            const int64_t size[2] = {int64_t(layer.tile_width()) * 4096, int64_t(layer.tile_height()) * 4096};
            const int64_t one = int64_t(1) << 24;
            int64_t cell[2], end[2], step[2], next[2], span[2];
            for (int k = 0; k < 2; ++k) {
                const int64_t p = int64_t(start[k].raw()) - origin[k].raw(), d = delta[k].raw();
                cell[k] = floor_div64(p, size[k]);
                end[k] = floor_div64(p + d, size[k]);
                step[k] = d > 0 ? 1 : d < 0 ? -1 : 0;
                if (!d) {
                    next[k] = span[k] = INT64_MAX;
                    continue;
                }
                const int64_t boundary = (d > 0 ? cell[k] + 1 : cell[k]) * size[k];
                next[k] = (boundary - p) * one / d;
                span[k] = size[k] * one / (d > 0 ? d : -d);
            }
            for (int guard = 0; guard < 512; ++guard) {
                if (!probe(layer, origin, int32_t(cell[0]), int32_t(cell[1]), nullptr)) return;
                if (cell[0] == end[0] && cell[1] == end[1]) return;
                const int axis = next[0] < next[1] ? 0 : 1;
                if (next[axis] > one) return;
                cell[axis] += step[axis];
                next[axis] += span[axis];
            }
        });
    }

    // ---- runtime tile edits ------------------------------------------------------------
    static bool edit_before(const TileEdit& edit, uint16_t layer, int16_t y, int16_t x) {
        if (edit.layer != layer) return edit.layer < layer;
        if (edit.y != y) return edit.y < y;
        return edit.x < x;
    }
    size_t edit_lower(uint16_t layer, int16_t y, int16_t x) const {
        size_t low = 0, high = edit_count;
        while (low < high) {
            const size_t mid = (low + high) / 2;
            if (edit_before(edits[mid], layer, y, x)) low = mid + 1;
            else high = mid;
        }
        return low;
    }
    static bool cell_in_range(int32_t x, int32_t y) { return x >= -32768 && x <= 32767 && y >= -32768 && y <= 32767; }

public:
    // ---- collision ---------------------------------------------------------------------
    void sync(bool force = false) {
        if (!force && synced_tick == time.ticks) return;
        synced_tick = time.ticks;
        clear_probes();
        size_t count = 0;
        uint32_t dropped = 0;
        colliders.each([&](Collider2DComponent& collider) {
            collider.slot = -1;
            if (!collider.shape.enabled || !owner_active(collider)) return;
            if (count >= EPOK_WORLD2D_COLLIDERS) { ++dropped; return; }
            ColliderEntry2D entry;
            if (!entry_for(collider, entry)) return;
            world.set(count, entry);
            slots[count] = &collider;
            collider.slot = int16_t(count);
            ++count;
        });
        for (size_t i = count; i < slot_count; ++i) { world.disable(i); slots[i] = nullptr; }
        if (count < slot_count) world.shrink();
        slot_count = count;
        stats.colliders = uint32_t(count);
        stats.dropped_colliders = dropped;
    }
    bool move(ActorComponent& body, const Fixed* delta, uint32_t mask, MoveResult2D& result) {
        sync();
        clear_probes();
        auto* collider = solid_collider_of(body);
        if (!collider) {
            // No collider: move freely, like a body without shapes.
            result = MoveResult2D{};
            result.displacement[0] = delta[0];
            result.displacement[1] = delta[1];
            return displace_owner(body, delta);
        }
        ColliderEntry2D entry;
        if (collider->slot < 0 || !entry_for(*collider, entry)) return false;
        world.set(size_t(collider->slot), entry);  // the body may have moved since sync
        const uint32_t filter = mask & collider->shape.layer_mask;
        if (tile_layers.head) {
            Aabb2D swept = collider2d_bounds(entry.collider, entry.position);
            for (int k = 0; k < 2; ++k) {
                if (delta[k] > 0.0) swept.max[k] += delta[k];
                else swept.min[k] += delta[k];
            }
            probe_box(swept, filter);
        }
        result = world.move_and_slide_2d(size_t(collider->slot), delta, filter);
        return displace_owner(body, result.displacement);
    }
    ObjectId actor_at(int index) const {
        if (index >= 0 && size_t(index) < slot_count) return owner_of(slots[index]);
        if (index >= int(probe_base) && size_t(index) < probe_base + probe_count) return owner_of(probes[index - probe_base].layer);
        return ObjectId{};
    }
    bool tile_at(int index, int32_t& x, int32_t& y) const {
        if (index < int(probe_base) || size_t(index) >= probe_base + probe_count) return false;
        x = probes[index - probe_base].x;
        y = probes[index - probe_base].y;
        return true;
    }
    SpatialHit2D raycast(const Fixed* origin, const Fixed* delta, uint32_t mask, bool triggers) {
        sync();
        clear_probes();
        if (tile_layers.head) probe_ray(origin, delta, mask);
        return world.raycast2d(origin, delta, mask, -1, triggers);
    }
    size_t overlap(const Aabb2D& box, ObjectId* actors, size_t capacity, uint32_t mask, bool triggers) {
        sync();
        clear_probes();
        uint16_t indices[EPOK_WORLD2D_COLLIDERS];
        size_t total = world.overlap(box, indices, EPOK_WORLD2D_COLLIDERS, mask, -1, triggers);
        for (size_t i = 0; actors && i < total && i < capacity && i < EPOK_WORLD2D_COLLIDERS; ++i) actors[i] = actor_at(indices[i]);
        // Each tile map whose solid tiles overlap the box counts once, as its actor.
        tile_layers.each([&](TileMapLayerComponent& layer) {
            clear_probes();
            probe_layer(layer, box, mask);
            bool overlapping = false;
            for (size_t i = 0; i < probe_count && !overlapping; ++i) {
                const Aabb2D* tile = world.bounds(probe_base + i);
                overlapping = tile && aabb2d_overlap(box, *tile);
            }
            if (!overlapping) return;
            if (actors && total < capacity) actors[total] = owner_of(&layer);
            ++total;
        });
        clear_probes();
        return total;
    }

    // ---- runtime tile edits ------------------------------------------------------------
    void edit_lookup(const TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t& value) const {
        if (!cell_in_range(x, y)) return;
        const size_t at = edit_lower(layer.slot, int16_t(y), int16_t(x));
        if (at < edit_count && edits[at].layer == layer.slot && edits[at].y == y && edits[at].x == x) value = edits[at].value;
    }
    bool edit_store(TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t value) {
        if (!cell_in_range(x, y)) return false;
        const size_t at = edit_lower(layer.slot, int16_t(y), int16_t(x));
        const bool exists = at < edit_count && edits[at].layer == layer.slot && edits[at].y == y && edits[at].x == x;
        if (value == layer.cooked_cell(x, y)) {
            // Back to the authored tile: the edit is no longer needed.
            if (exists) {
                for (size_t i = at; i + 1 < edit_count; ++i) edits[i] = edits[i + 1];
                --edit_count;
                --layer.edit_count;
            }
            stats.tile_edits = uint32_t(edit_count);
            return true;
        }
        if (exists) {
            edits[at].value = value;
            return true;
        }
        if (edit_count >= EPOK_WORLD2D_TILE_EDITS) {
            ++stats.dropped_tile_edits;
            return false;
        }
        for (size_t i = edit_count; i > at; --i) edits[i] = edits[i - 1];
        edits[at] = TileEdit{layer.slot, int16_t(y), int16_t(x), value};
        ++edit_count;
        ++layer.edit_count;
        stats.tile_edits = uint32_t(edit_count);
        return true;
    }
    void edit_forget(TileMapLayerComponent& layer) {
        size_t kept = 0;
        for (size_t i = 0; i < edit_count; ++i)
            if (edits[i].layer != layer.slot) edits[kept++] = edits[i];
        edit_count = kept;
        layer.edit_count = 0;
        stats.tile_edits = uint32_t(edit_count);
    }

    // ---- simulation step ---------------------------------------------------------------
    // After the level tick: animate, resolve this step's colliders and triggers, move the
    // cameras to where the actors ended up, then scroll the parallax layers after them.
    template<class LevelType> void step(Fixed dt, LevelType* level) {
        sprites.each([&](Sprite2DComponent& sprite) { sprite.advance(dt); });
        tile_clock += dt;
        if (tile_clock > Fixed(65536.0)) tile_clock -= Fixed(65536.0);
        sync(true);
        world.update_triggers([&](const TriggerEvent& event) {
            const ObjectId a = actor_at(event.first), b = actor_at(event.second);
            if (level) {
                dispatch(*level, a, b, event.phase);
                dispatch(*level, b, a, event.phase);
            }
        });
        stats.dropped_trigger_pairs = world.dropped_trigger_pairs;
        cameras.each([&](Camera2DComponent& camera) {
            if (!owner_active(camera)) return;
            Affine2D matrix;
            if (!world_of(camera, matrix)) return;
            camera.follow(matrix.t, dt);
            // The view turns with its owner only when `rotating` is set.
            camera.rotation = camera.rotating ? world_rotation(camera) : Fixed(0.0);
        });
        if (parallaxes.head) {
            Fixed center[2] = {0.0, 0.0};
            if (auto* camera = active_camera()) {
                center[0] = camera->screen_center_x();
                center[1] = camera->screen_center_y();
            }
            parallaxes.each([&](Parallax2DComponent& parallax) {
                if (owner_active(parallax)) parallax.follow(center, dt);
            });
        }
    }
    template<class LevelType> static void dispatch(LevelType& level, ObjectId actor, ObjectId other, TriggerPhase phase) {
        auto* registry = level.registry();
        if (!registry) return;
        auto* owner = registry->template resolve<Actor>(actor);
        if (!owner || !level.actor_active(*owner)) return;
        ObjectDispatchScope scope(*registry);
        owner->overlap_2d(other, phase);
        for (size_t i = 0; i < owner->component_count(); ++i)
            if (auto* component = registry->template resolve<ActorComponent>(owner->component_id(i)))
                component->trigger_event(other, phase);
    }
    Camera2DComponent* active_camera() {
        Camera2DComponent* found = nullptr;
        cameras.each([&](Camera2DComponent& camera) {
            if (!found && camera.current && owner_active(camera)) found = &camera;
        });
        return found;
    }
    // Background colour of the active camera, or null to keep the engine default.
    const uint8_t* background() {
        auto* camera = active_camera();
        return camera ? camera->background : nullptr;
    }
    void reset() {
        world.clear();
        for (auto& slot : slots) slot = nullptr;
        slot_count = 0;
        probe_count = 0;
        synced_tick = 0xffffffffu;
        colliders.each([](Collider2DComponent& collider) { collider.slot = -1; });
        edit_count = 0;
        tile_layers.each([](TileMapLayerComponent& layer) { layer.edit_count = 0; });
        tile_clock = 0.0;
        stats = {};
    }
};
inline Service service;

inline void install() {
    hooks.move = [](ActorComponent& body, const Fixed* delta, uint32_t mask, MoveResult2D& result) {
        return service.move(body, delta, mask, result);
    };
    hooks.raycast = [](const Fixed* origin, const Fixed* delta, uint32_t mask, bool triggers) {
        return service.raycast(origin, delta, mask, triggers);
    };
    hooks.hit_actor = [](int index) { return service.actor_at(index); };
    hooks.overlap = [](const Aabb2D& box, ObjectId* actors, size_t capacity, uint32_t mask, bool triggers) {
        return service.overlap(box, actors, capacity, mask, triggers);
    };
    tile_cell_at = [](int index, int32_t& x, int32_t& y) { return service.tile_at(index, x, y); };
    tile_hooks.lookup = [](const TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t& value) {
        service.edit_lookup(layer, x, y, value);
    };
    tile_hooks.store = [](TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t value) {
        return service.edit_store(layer, x, y, value);
    };
    tile_hooks.forget = [](TileMapLayerComponent& layer) { service.edit_forget(layer); };
}


// ---- draw list ----------------------------------------------------------------------------
// One entry of the frame's draw list: a sprite resolved to screen corners (A top-left,
// B top-right, C bottom-left, D bottom-right in the sprite's own frame), its texel
// rectangle, colour and blend; or a whole tile layer (`tiles`), expanded by the renderer.
// Shared by the PSX renderer and the native PC runner, so both draw the same frame.
struct SpriteDraw {
    uint32_t key = 0;
    int16_t x[4] = {}, y[4] = {};
    uint16_t source[4] = {};
    int32_t texture = -1;
    uint8_t color[3] = {255, 255, 255};
    BlendMode blend = BlendMode::Cutout;
    bool flip_h = false, flip_v = false;
    // Unrotated, unscaled, unflipped at zoom 1: a GP0 rectangle reproduces it exactly.
    bool rectangle = false;
    // Tile layer entries: the layer, whether it is drawn in screen pixels, and the screen
    // shift of a repeated parallax copy (plus a screen canvas offset).
    const TileMapLayerComponent* tiles = nullptr;
    bool screen = false;
    int16_t shift[2] = {0, 0};
};
// One tile resolved for drawing (corners as in SpriteDraw).
struct TileDraw {
    int16_t x[4], y[4];
    uint16_t u, v, w, h;
    bool flip_h, flip_v, transpose;
    // Unflipped and drawn one texel per pixel: a GP0 rectangle reproduces it exactly.
    bool rectangle;
};

inline void tint(uint8_t* color, const uint8_t* by) {
    if (!by) return;
    for (int k = 0; k < 3; ++k) color[k] = uint8_t((unsigned(color[k]) * by[k] + 127) / 255);
}
// Screen copies of a drawable under a repeating Parallax2D: calls `visit(dx, dy)` for every
// shift (in screen pixels) whose copy of `bounds` (min x, min y, max x, max y) is on screen.
template<class Visit>
inline void repeat_copies(const Canvas& canvas, const Projector& projector, const int16_t* bounds, Visit&& visit) {
    const Parallax2DComponent* parallax = canvas.parallax;
    int32_t period[2] = {0, 0};
    for (int k = 0; k < 2; ++k)
        if (parallax && parallax->repeat_size[k] > 0.0)
            period[k] = (detail::mul_2d(parallax->repeat_size[k], projector.scale).raw() + 2048) >> 12;
    int32_t range[2][2] = {{0, 0}, {0, 0}};
    const int32_t limit[2] = {int32_t(display_width), int32_t(display_height)};
    for (int k = 0; k < 2; ++k) {
        if (period[k] < 4) continue;
        // Copies n with bounds + n * period overlapping [0, limit).
        range[k][0] = -int32_t(floor_div64(int64_t(bounds[k + 2]), period[k])) - 1;
        range[k][1] = int32_t(floor_div64(int64_t(limit[k] - bounds[k]), period[k])) + 1;
        if (range[k][1] - range[k][0] > 64) range[k][1] = range[k][0] + 64;
    }
    for (int32_t j = range[1][0]; j <= range[1][1]; ++j)
        for (int32_t i = range[0][0]; i <= range[0][1]; ++i) {
            const int32_t dx = i * period[0], dy = j * period[1];
            if (bounds[2] + dx <= 0 || bounds[0] + dx >= limit[0] || bounds[3] + dy <= 0 || bounds[1] + dy >= limit[1]) continue;
            if (dx < -32768 || dx > 32767 || dy < -32768 || dy > 32767) continue;
            visit(int16_t(dx), int16_t(dy));
        }
}
// The CanvasModulate2D tint of each canvas, resolved once per frame.
struct Tints {
    int8_t layers[4] = {};
    const uint8_t* colors[4] = {};
    uint8_t count = 0;
    Tints() {
        canvas_modulates.each([&](CanvasModulate2DComponent& modulate) {
            if (count >= 4 || !is_active(&modulate)) return;
            const int8_t layer = canvas_of(modulate).layer;
            for (uint8_t i = 0; i < count; ++i)
                if (layers[i] == layer) return;
            layers[count] = layer;
            colors[count++] = modulate.color;
        });
    }
    const uint8_t* of(int8_t layer) const {
        for (uint8_t i = 0; i < count; ++i)
            if (layers[i] == layer) return colors[i];
        return nullptr;
    }
};
inline uint32_t z_key(int32_t z) {
    z = z < -32768 ? -32768 : z > 32767 ? 32767 : z;
    return uint32_t(z + 32768) << 16;
}

// Resolves every visible sprite and tile layer through the active camera (or the screen,
// for a screen canvas) into `draws`, and writes the back-to-front order into `items`
// (`scratch` is sort workspace). `texture_size(index, width, height)` reports a texture's
// size and returns false when it is absent. Returns the number of entries to draw; extra
// entries are counted as dropped.
template<class TextureSize>
size_t collect_sprites(SpriteDraw* draws, DrawItem* items, DrawItem* scratch, size_t capacity, TextureSize&& texture_size) {
    stats.sprites = stats.culled = stats.dropped = stats.tile_layers = stats.tiles = 0;
    if (!sprites.head && !tile_layers.head) return 0;
    Camera2D camera;
    if (auto* active = service.active_camera()) camera = active->view();
    const Projector world_projector(camera);
    const Projector screen_projector = Projector::screen();
    const Tints tints;
    size_t count = 0;
    auto push = [&](const SpriteDraw& draw, int8_t layer) {
        if (count >= capacity) {
            ++stats.dropped;
            return;
        }
        draws[count] = draw;
        items[count] = DrawItem{draw.key, uint16_t(count), uint8_t(int16_t(layer) + 128)};
        ++count;
    };
    sprites.each([&](Sprite2DComponent& sprite) {
        ++stats.sprites;
        if (!sprite.visible || !is_active(&sprite)) return;
        const Canvas canvas = canvas_of(sprite);
        if (!canvas.visible) return;
        uint16_t width = 0, height = 0;
        if (!texture_size(int(sprite.texture), width, height)) { ++stats.culled; return; }
        uint16_t source[4];
        if (!sprite.source_rect(width, height, source)) { ++stats.culled; return; }
        Affine2D matrix;
        if (!world_of(sprite, matrix)) return;
        const Projector& projector = canvas.screen ? screen_projector : world_projector;
        const Fixed w(int32_t(source[2]) * 4096, Fixed::RAW), h(int32_t(source[3]) * 4096, Fixed::RAW);
        const Fixed left = (sprite.centered ? -(w / 2) : Fixed(0.0)) + sprite.offset[0];
        const Fixed top = (sprite.centered ? -(h / 2) : Fixed(0.0)) + sprite.offset[1];
        const Fixed local[4][2] = {{left, top}, {left + w, top}, {left, top + h}, {left + w, top + h}};
        SpriteDraw draw;
        int16_t bounds[4] = {32767, 32767, -32768, -32768};
        for (int i = 0; i < 4; ++i) {
            Fixed world[2], screen[2];
            matrix.point(local[i], world);
            if (canvas.screen) {
                world[0] += canvas.offset[0];
                world[1] += canvas.offset[1];
            }
            projector.point(world, screen);
            draw.x[i] = screen_pixel(screen[0]);
            draw.y[i] = screen_pixel(screen[1]);
            if (draw.x[i] < bounds[0]) bounds[0] = draw.x[i];
            if (draw.x[i] > bounds[2]) bounds[2] = draw.x[i];
            if (draw.y[i] < bounds[1]) bounds[1] = draw.y[i];
            if (draw.y[i] > bounds[3]) bounds[3] = draw.y[i];
        }
        // The GPU rejects primitives wider than 1023 or taller than 511 pixels.
        if (bounds[2] - bounds[0] > 1023 || bounds[3] - bounds[1] > 511) { ++stats.dropped; return; }
        const auto* owner_root = root_of(sprite);
        draw.key = z_key(int32_t(sprite.z_index) + (owner_root ? int32_t(owner_root->transform.draw_order) : 0));
        if (sprite.y_sort) {
            const int32_t row = (matrix.t[1].raw() >> 12) + 32768;
            draw.key |= uint32_t(row < 1 ? 1 : row > 65535 ? 65535 : row);
        }
        for (int k = 0; k < 4; ++k) draw.source[k] = source[k];
        for (int k = 0; k < 3; ++k) draw.color[k] = sprite.modulate[k];
        tint(draw.color, tints.of(canvas.layer));
        draw.texture = sprite.texture;
        draw.blend = sprite.blend;
        draw.flip_h = sprite.flip_h;
        draw.flip_v = sprite.flip_v;
        const bool axis_aligned = matrix.m[0][0].raw() == 4096 && matrix.m[1][1].raw() == 4096 && !matrix.m[0][1].raw() &&
                                  !matrix.m[1][0].raw();
        draw.rectangle = projector.identity && axis_aligned && !sprite.flip_h && !sprite.flip_v;
        bool drawn = false;
        auto emit = [&](int16_t dx, int16_t dy) {
            SpriteDraw copy = draw;
            for (int i = 0; i < 4; ++i) {
                copy.x[i] = int16_t(copy.x[i] + dx);
                copy.y[i] = int16_t(copy.y[i] + dy);
            }
            push(copy, canvas.layer);
            drawn = true;
        };
        if (canvas.parallax && (canvas.parallax->repeat_size[0] > 0.0 || canvas.parallax->repeat_size[1] > 0.0))
            repeat_copies(canvas, projector, bounds, emit);
        else if (!(bounds[2] <= 0 || bounds[3] <= 0 || bounds[0] >= display_width || bounds[1] >= display_height))
            emit(0, 0);
        if (!drawn) ++stats.culled;
    });
    tile_layers.each([&](TileMapLayerComponent& layer) {
        if (!layer.visible || !layer.tileset || !is_active(&layer)) return;
        const Canvas canvas = canvas_of(layer);
        if (!canvas.visible) return;
        ++stats.tile_layers;
        SpriteDraw draw;
        draw.tiles = &layer;
        draw.screen = canvas.screen;
        draw.texture = layer.texture;
        draw.blend = layer.blend;
        for (int k = 0; k < 3; ++k) draw.color[k] = layer.modulate[k];
        tint(draw.color, tints.of(canvas.layer));
        const auto* owner_root = root_of(layer);
        draw.key = z_key(int32_t(layer.z_index) + (owner_root ? int32_t(owner_root->transform.draw_order) : 0));
        if (canvas.screen) {
            draw.shift[0] = screen_pixel(canvas.offset[0]);
            draw.shift[1] = screen_pixel(canvas.offset[1]);
        }
        const bool repeats = canvas.parallax && (canvas.parallax->repeat_size[0] > 0.0 || canvas.parallax->repeat_size[1] > 0.0);
        Fixed origin[2];
        if (!repeats || !layer.grid_size[0] || !layer.origin(origin)) {
            push(draw, canvas.layer);
            return;
        }
        // Repeated copies of the cooked area cover the screen; tiles cull themselves.
        const Projector& projector = canvas.screen ? screen_projector : world_projector;
        const int32_t span[2] = {int32_t(layer.tile_width()) << tile_chunk_shift, int32_t(layer.tile_height()) << tile_chunk_shift};
        int16_t bounds[4] = {32767, 32767, -32768, -32768};
        for (int i = 0; i < 4; ++i) {
            const int32_t cx = int32_t(layer.grid_origin[0]) + (i & 1) * int32_t(layer.grid_size[0]);
            const int32_t cy = int32_t(layer.grid_origin[1]) + (i >> 1) * int32_t(layer.grid_size[1]);
            const Fixed world[2] = {origin[0] + Fixed(cx * span[0] * 4096, Fixed::RAW), origin[1] + Fixed(cy * span[1] * 4096, Fixed::RAW)};
            Fixed screen[2];
            projector.point(world, screen);
            const int16_t x = int16_t(screen_pixel(screen[0]) + draw.shift[0]), y = int16_t(screen_pixel(screen[1]) + draw.shift[1]);
            if (x < bounds[0]) bounds[0] = x;
            if (x > bounds[2]) bounds[2] = x;
            if (y < bounds[1]) bounds[1] = y;
            if (y > bounds[3]) bounds[3] = y;
        }
        repeat_copies(canvas, projector, bounds, [&](int16_t dx, int16_t dy) {
            SpriteDraw copy = draw;
            copy.shift[0] = int16_t(draw.shift[0] + dx);
            copy.shift[1] = int16_t(draw.shift[1] + dy);
            push(copy, canvas.layer);
        });
    });
    sort_draw_items(items, scratch, count);
    return count;
}

// Every visible tile of a tile-layer entry, in row order. `texture_width/height` bound the
// atlas; tiles whose frame falls outside it are skipped.
template<class Visit>
inline void each_tile(const SpriteDraw& entry, uint16_t texture_width, uint16_t texture_height, Visit&& visit) {
    const TileMapLayerComponent* layer = entry.tiles;
    if (!layer || !layer->tileset) return;
    Camera2D camera;
    if (!entry.screen)
        if (auto* active = service.active_camera()) camera = active->view();
    const Projector projector = entry.screen ? Projector::screen() : Projector(camera);
    Fixed origin[2];
    if (!layer->origin(origin)) return;
    const int32_t w = layer->tile_width(), h = layer->tile_height();
    // The visible cell range: the screen rectangle mapped back into the layer's grid.
    Fixed corners[4][2] = {{0.0, 0.0}, {Fixed(int32_t(display_width) * 4096, Fixed::RAW), 0.0}, {0.0, Fixed(int32_t(display_height) * 4096, Fixed::RAW)},
                           {Fixed(int32_t(display_width) * 4096, Fixed::RAW), Fixed(int32_t(display_height) * 4096, Fixed::RAW)}};
    int32_t range[4] = {0x7fffffff, 0x7fffffff, -0x7fffffff, -0x7fffffff};
    for (auto& corner : corners) {
        Fixed screen[2] = {corner[0] - Fixed(int32_t(entry.shift[0]) * 4096, Fixed::RAW), corner[1] - Fixed(int32_t(entry.shift[1]) * 4096, Fixed::RAW)};
        Fixed world[2];
        screen_to_world(projector.camera, screen, world);
        const int32_t cx = floor_cells(world[0].raw() - origin[0].raw(), w), cy = floor_cells(world[1].raw() - origin[1].raw(), h);
        if (cx < range[0]) range[0] = cx;
        if (cy < range[1]) range[1] = cy;
        if (cx > range[2]) range[2] = cx;
        if (cy > range[3]) range[3] = cy;
    }
    --range[0];
    --range[1];
    // A zoomed-out view is bounded so one layer can never stall a frame.
    if (range[2] - range[0] > 127) range[2] = range[0] + 127;
    if (range[3] - range[1] > 127) range[3] = range[1] + 127;
    const bool identity = projector.identity;
    Fixed first[2];
    projector.point(origin, first);
    const int32_t sx0 = screen_pixel(first[0]) + entry.shift[0], sy0 = screen_pixel(first[1]) + entry.shift[1];
    const Fixed clock = tile_clock;
    for (int32_t cy = range[1]; cy <= range[3]; ++cy)
        for (int32_t cx = range[0]; cx <= range[2]; ++cx) {
            const uint16_t value = layer->cell(cx, cy);
            const TileInfo* tile = layer->tile_of(value);
            if (!tile) continue;
            TileDraw draw;
            draw.u = tile->u;
            draw.v = tile->v;
            draw.w = uint16_t(w);
            draw.h = uint16_t(h);
            if (tile->frames > 1 && tile->frame_seconds.raw() > 0) {
                const uint32_t frame = uint32_t(clock.raw() / tile->frame_seconds.raw()) % tile->frames;
                const uint32_t column = tile->columns ? frame % tile->columns : frame, row = tile->columns ? frame / tile->columns : 0;
                draw.u = uint16_t(draw.u + column * tile->stride[0]);
                draw.v = uint16_t(draw.v + row * tile->stride[1]);
            }
            if (uint32_t(draw.u) + draw.w > texture_width || uint32_t(draw.v) + draw.h > texture_height) continue;
            draw.flip_h = (value & tile_flip_h) != 0;
            draw.flip_v = (value & tile_flip_v) != 0;
            draw.transpose = (value & tile_transpose) != 0;
            draw.rectangle = identity && !draw.flip_h && !draw.flip_v && !draw.transpose;
            if (identity) {
                const int32_t x = sx0 + cx * w, y = sy0 + cy * h;
                if (x + w <= 0 || y + h <= 0 || x >= display_width || y >= display_height) continue;
                if (x < -32768 || x + w > 32767 || y < -32768 || y + h > 32767) continue;
                draw.x[0] = draw.x[2] = int16_t(x);
                draw.x[1] = draw.x[3] = int16_t(x + w);
                draw.y[0] = draw.y[1] = int16_t(y);
                draw.y[2] = draw.y[3] = int16_t(y + h);
            } else {
                int16_t min_x = 32767, min_y = 32767, max_x = -32768, max_y = -32768;
                for (int i = 0; i < 4; ++i) {
                    const Fixed world[2] = {origin[0] + Fixed((cx * w + (i & 1) * w) * 4096, Fixed::RAW),
                                            origin[1] + Fixed((cy * h + (i >> 1) * h) * 4096, Fixed::RAW)};
                    Fixed screen[2];
                    projector.point(world, screen);
                    draw.x[i] = int16_t(screen_pixel(screen[0]) + entry.shift[0]);
                    draw.y[i] = int16_t(screen_pixel(screen[1]) + entry.shift[1]);
                    if (draw.x[i] < min_x) min_x = draw.x[i];
                    if (draw.x[i] > max_x) max_x = draw.x[i];
                    if (draw.y[i] < min_y) min_y = draw.y[i];
                    if (draw.y[i] > max_y) max_y = draw.y[i];
                }
                if (max_x <= 0 || max_y <= 0 || min_x >= display_width || min_y >= display_height) continue;
            }
            ++stats.tiles;
            visit(draw);
        }
}
}  // namespace world2d
}  // namespace epok
