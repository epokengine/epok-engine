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
//   EPOK_WORLD2D_PARTICLES     2D particles alive at once, shared by every emitter
//   EPOK_WORLD2D_TRIGGER_PAIRS remembered overlapping trigger pairs
//   EPOK_WORLD2D_DRAW_ITEMS    sprites, tile layers and repeated copies drawn per frame
//   EPOK_WORLD2D_ARENA_BYTES   GPU packet bytes per frame parity
//   EPOK_WORLD2D_NAV_CELLS     cells of the largest navigation grid (A* scratch)
//   EPOK_WORLD2D_NAV_OPEN      A* open-list entries
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
#ifndef EPOK_WORLD2D_PARTICLES
#define EPOK_WORLD2D_PARTICLES 64
#endif
#ifndef EPOK_WORLD2D_TRIGGER_PAIRS
#define EPOK_WORLD2D_TRIGGER_PAIRS 64
#endif
#ifndef EPOK_WORLD2D_DRAW_ITEMS
#define EPOK_WORLD2D_DRAW_ITEMS 256
#endif
#ifndef EPOK_WORLD2D_NAV_CELLS
#define EPOK_WORLD2D_NAV_CELLS 1
#endif
#ifndef EPOK_WORLD2D_NAV_OPEN
#define EPOK_WORLD2D_NAV_OPEN 1
#endif

namespace epok {
namespace world2d {

struct Stats {
    uint32_t sprites = 0, drawn = 0, rectangles = 0, quads = 0, pages = 0, culled = 0, dropped = 0;
    uint32_t arena_bytes = 0, colliders = 0, dropped_colliders = 0, trigger_pairs = 0, dropped_trigger_pairs = 0;
    uint32_t tile_layers = 0, tiles = 0, tile_probes = 0, dropped_tile_probes = 0, tile_edits = 0, dropped_tile_edits = 0;
    uint32_t particles = 0, dropped_particles = 0, shapes = 0;
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
    struct Particle {
        Particles2DComponent* owner = nullptr;
        Fixed position[2] = {0.0, 0.0}, velocity[2] = {0.0, 0.0};
        Fixed age = 0.0, life = 1.0;
    };
    Particle particles[EPOK_WORLD2D_PARTICLES] = {};
public:
    // Grid A* scratch for navigation agents (see world2d_navigation.hpp).
    uint16_t nav_cost[EPOK_WORLD2D_NAV_CELLS] = {};
    uint8_t nav_info[EPOK_WORLD2D_NAV_CELLS] = {};
    uint32_t nav_heap[EPOK_WORLD2D_NAV_OPEN] = {};
private:

    // Uniform [0, 1) from the emitter's own sequence, so emitters stay reproducible.
    static Fixed random_unit(Particles2DComponent& emitter) {
        emitter.seed = emitter.seed * 1664525u + 1013904223u;
        return Fixed(int32_t((emitter.seed >> 12) & 4095), Fixed::RAW);
    }
    static Fixed lerp(Fixed a, Fixed b, Fixed t) { return a + detail::mul_2d(b - a, t); }
    void spawn(Particles2DComponent& e, const Affine2D& matrix) {
        Particle* slot = nullptr;
        for (auto& p : particles)
            if (!p.owner) { slot = &p; break; }
        if (!slot) { ++stats.dropped_particles; return; }
        Fixed local[2] = {0.0, 0.0};
        if (e.shape == EmissionShape2D::Rectangle) {
            for (int k = 0; k < 2; ++k) local[k] = detail::mul_2d(random_unit(e) * 2 - Fixed(1.0), e.extents[k]);
        } else if (e.shape == EmissionShape2D::Circle) {
            const Fixed angle = random_unit(e) * 360;
            const Fixed radius = detail::mul_2d(random_unit(e), e.extents[0]);
            local[0] = detail::mul_2d(cos_degrees(angle), radius);
            local[1] = detail::mul_2d(sin_degrees(angle), radius);
        }
        const Fixed angle = e.direction + detail::mul_2d(random_unit(e) * 2 - Fixed(1.0), e.spread);
        const Fixed speed = lerp(e.speed[0], e.speed[1], random_unit(e));
        Fixed velocity[2] = {detail::mul_2d(cos_degrees(angle), speed), detail::mul_2d(sin_degrees(angle), speed)};
        if (!e.local_coords) {
            // Launch in the emitter's world orientation.
            Fixed world[2];
            matrix.point(local, world);
            local[0] = world[0];
            local[1] = world[1];
            const Fixed vx = velocity[0], vy = velocity[1];
            const Fixed sx = length_2d(matrix.m[0][0], matrix.m[1][0]), sy = length_2d(matrix.m[0][1], matrix.m[1][1]);
            velocity[0] = (sx.raw() ? detail::mul_2d(matrix.m[0][0] / sx, vx) : vx) + (sy.raw() ? detail::mul_2d(matrix.m[0][1] / sy, vy) : Fixed(0.0));
            velocity[1] = (sx.raw() ? detail::mul_2d(matrix.m[1][0] / sx, vx) : Fixed(0.0)) + (sy.raw() ? detail::mul_2d(matrix.m[1][1] / sy, vy) : vy);
        }
        slot->owner = &e;
        slot->position[0] = local[0];
        slot->position[1] = local[1];
        slot->velocity[0] = velocity[0];
        slot->velocity[1] = velocity[1];
        slot->age = 0.0;
        slot->life = e.lifetime.raw() > 0 ? e.lifetime : Fixed(1.0);
        ++e.alive;
        ++e.emitted;
    }
    void simulate_particles(Fixed dt) {
        stats.particles = 0;
        for (auto& p : particles) {
            if (!p.owner) continue;
            p.age += dt;
            if (p.age >= p.life) {
                if (p.owner->alive) --p.owner->alive;
                p.owner = nullptr;
                continue;
            }
            auto& e = *p.owner;
            Fixed keep = 1.0;
            if (e.damping.raw() > 0) {
                keep = Fixed(1.0) - e.damping * dt;
                if (keep < 0.0) keep = 0.0;
            }
            for (int k = 0; k < 2; ++k) {
                p.velocity[k] = detail::mul_2d(p.velocity[k] + e.gravity[k] * dt, keep);
                p.position[k] += p.velocity[k] * dt;
            }
            ++stats.particles;
        }
        emitters.each([&](Particles2DComponent& e) {
            // A finished one-shot burst reports once, when its last particle is gone.
            if (!e.emitting) {
                if (e.one_shot && e.emitted && !e.alive) {
                    e.finished = true;
                    e.emitted = 0;
                }
                return;
            }
            Affine2D matrix;
            if (!is_active(&e) || !world_of(e, matrix)) return;
            const uint16_t amount = e.amount ? e.amount : 1;
            const Fixed life = e.lifetime.raw() > 0 ? e.lifetime : Fixed(1.0);
            if (e.explosiveness >= 1.0) {
                // Bursts: the whole amount at once, once per lifetime.
                e.accumulator -= dt;
                if (e.accumulator <= 0.0) {
                    for (uint16_t i = 0; i < amount && e.alive < amount; ++i) spawn(e, matrix);
                    e.accumulator = life;
                    if (e.one_shot) e.emitting = false;
                }
                return;
            }
            e.accumulator += dt * int32_t(amount) / life;
            if (e.accumulator > Fixed(int32_t(amount) * 4096, Fixed::RAW)) e.accumulator = Fixed(int32_t(amount) * 4096, Fixed::RAW);
            while (e.accumulator >= 1.0 && e.alive < amount) {
                e.accumulator -= 1.0;
                spawn(e, matrix);
                if (e.one_shot && e.emitted >= amount) {
                    e.emitting = false;
                    break;
                }
            }
        });
    }

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

    // ---- particles ---------------------------------------------------------------------
    void release(Particles2DComponent& emitter) {
        for (auto& p : particles)
            if (p.owner == &emitter) p.owner = nullptr;
        emitter.alive = 0;
    }
    // Visits (position, age fraction) of every live particle of `emitter`, oldest slots
    // first. Positions are world pixels, or emitter-local with local coordinates.
    template<class Visit> void each_particle_of(const Particles2DComponent& emitter, Visit&& visit) const {
        for (const auto& p : particles)
            if (p.owner == &emitter) visit(p.position, p.life.raw() > 0 ? p.age / p.life : Fixed(0.0));
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
    // ---- navigation ----------------------------------------------------------------------
    // Agents follow their target actor, plan when asked (one path per step for the whole
    // map, so a crowd never stalls a frame), then walk their path.
    void navigate(Fixed dt) {
        bool planned = false;
        nav_agents.each([&](NavigationAgent2DComponent& agent) {
            if (!owner_active(agent)) return;
            SceneComponent2D* root = root_of(agent);
            Affine2D matrix;
            if (!root || !world_of(agent, matrix)) return;
            if (agent.has_target && agent.target_actor.valid()) {
                SceneComponent2D* other = actor_root(agent.target_actor);
                Affine2D there;
                if (!other || !root_world(*other, there)) {
                    agent.stop();
                    return;
                }
                agent.target[0] = there.t[0];
                agent.target[1] = there.t[1];
                agent.clock += dt;
                const Fixed moved = length_2d(agent.target[0] - agent.planned_for[0], agent.target[1] - agent.planned_for[1]);
                const Fixed threshold = agent.arrive_distance > Fixed(4.0) ? agent.arrive_distance : Fixed(4.0);
                if (agent.clock >= agent.repath_interval && moved > threshold) agent.dirty = true;
            }
            if (agent.has_target && agent.dirty && !planned) {
                plan(agent, matrix.t);
                planned = true;
            }
            follow(agent, matrix.t, dt, *root);
        });
    }
    static void plan(NavigationAgent2DComponent& agent, const Fixed* at) {
        agent.dirty = false;
        agent.clock = 0.0;
        agent.planned_for[0] = agent.target[0];
        agent.planned_for[1] = agent.target[1];
        agent.path_count = agent.path_index = 0;
        agent.reachable = false;
        NavigationRegion2DComponent* region = nullptr;
        nav_regions.each([&](NavigationRegion2DComponent& r) {
            if (!region && owner_active(r) && r.covers(at)) region = &r;
        });
        bool reached = false;
        const size_t count = region ? region->find_path(at, agent.target, agent.path, nav_path_limit, reached) : 0;
        agent.path_count = uint8_t(count);
        agent.reachable = reached;
        agent.finished = count == 0;
    }
    static void follow(NavigationAgent2DComponent& agent, const Fixed* at, Fixed dt, SceneComponent2D& root) {
        agent.velocity[0] = agent.velocity[1] = 0.0;
        if (!agent.has_target || agent.finished) return;
        while (agent.path_index < agent.path_count &&
               length_2d(agent.path[agent.path_index][0] - at[0], agent.path[agent.path_index][1] - at[1]) <= agent.arrive_distance)
            ++agent.path_index;
        if (agent.path_index >= agent.path_count) {
            if (agent.path_count && !agent.dirty) {
                agent.finished = true;
                if (agent.reachable) agent.arrived = true;
            }
            return;
        }
        const Fixed dx = agent.path[agent.path_index][0] - at[0], dy = agent.path[agent.path_index][1] - at[1];
        const int64_t distance = length_2d(dx, dy).raw();
        if (distance <= 0) return;
        // Never overshoot the waypoint within one step.
        int64_t speed = agent.max_speed.raw();
        if (dt.raw() > 0 && int64_t(speed) * dt.raw() / 4096 > distance) speed = distance * 4096 / dt.raw();
        agent.velocity[0] = Fixed(int32_t(int64_t(dx.raw()) * speed / distance), Fixed::RAW);
        agent.velocity[1] = Fixed(int32_t(int64_t(dy.raw()) * speed / distance), Fixed::RAW);
        if (!agent.drives_body) return;
        if (auto* body = find<CharacterBody2DComponent>(agent.get_owner())) {
            body->set_velocity(agent.velocity[0], agent.velocity[1]);
            body->move_and_slide();
        } else {
            const Fixed next[2] = {at[0] + agent.velocity[0] * dt, at[1] + agent.velocity[1] * dt};
            set_world_position(root, next);
        }
    }

    // ---- rigid bodies --------------------------------------------------------------------
    static RigidBody2DComponent* rigid_of(ObjectId actor) {
        auto* registry = active_object_registry;
        auto* owner = registry && actor.valid() ? registry->resolve<Actor>(actor) : nullptr;
        return owner ? find<RigidBody2DComponent>(owner) : nullptr;
    }
    static Fixed times(Fixed value, int64_t q12) { return detail::clamp_raw_2d(int64_t(value.raw()) * q12 / 4096); }
    static Fixed magnitude(Fixed v) { return v.raw() < 0 ? -v : v; }
    static bool position_of(ActorComponent& component, Fixed* out) {
        Affine2D m;
        if (!world_of(component, m)) return false;
        out[0] = m.t[0];
        out[1] = m.t[1];
        return true;
    }
    // A one-dimensional collision along `axis`, with the lower restitution of the two.
    static void exchange(RigidBody2DComponent& a, RigidBody2DComponent& b, int axis) {
        Fixed pa[2], pb[2];
        if (!position_of(a, pa) || !position_of(b, pb)) return;
        const int32_t towards = pb[axis] > pa[axis] ? 1 : -1;
        const int64_t u1 = a.velocity[axis].raw(), u2 = b.velocity[axis].raw();
        if ((u1 - u2) * towards <= 0) return;  // already separating
        const int64_t m1 = a.mass.raw() > 0 ? a.mass.raw() : 1, m2 = b.mass.raw() > 0 ? b.mass.raw() : 1;
        const int64_t e = (a.bounce < b.bounce ? a.bounce : b.bounce).raw();
        const int64_t p = m1 * u1 + m2 * u2, total = m1 + m2;
        a.velocity[axis] = detail::clamp_raw_2d((p - m2 * e * (u1 - u2) / 4096) / total);
        b.velocity[axis] = detail::clamp_raw_2d((p + m1 * e * (u1 - u2) / 4096) / total);
        b.wake();
    }
    void simulate_bodies(Fixed dt) {
        // Characters push the pushable bodies they walk into; heavier ones move slower.
        bodies.each([&](CharacterBody2DComponent& character) {
            if (!character.on_wall || !owner_active(character) || !character.blocked_velocity[0].raw()) return;
            RigidBody2DComponent* rigid = rigid_of(character.wall_collider);
            Fixed pc[2], pr[2];
            if (!rigid || rigid->frozen || !rigid->pushable || !position_of(character, pc) || !position_of(*rigid, pr)) return;
            if ((pr[0] > pc[0]) != (character.blocked_velocity[0] > 0.0)) return;
            const int64_t mass = rigid->mass.raw() > 4096 ? rigid->mass.raw() : 4096;
            rigid->velocity[0] = Fixed(int32_t(int64_t(character.blocked_velocity[0].raw()) * 3 * 4096 / (4 * mass)), Fixed::RAW);
            rigid->wake();
        });
        // Springs pull before the bodies integrate.
        joints.each([&](Joint2DComponent& joint) {
            if (joint.enabled && joint.kind == JointKind2D::Spring && owner_active(joint)) spring(joint);
        });
        rigid_bodies.each([&](RigidBody2DComponent& body) {
            if (!owner_active(body)) return;
            if (body.frozen || body.sleeping) {
                body.force[0] = body.force[1] = 0.0;
                return;
            }
            const int64_t inverse = body.inverse_mass();
            for (int k = 0; k < 2; ++k) {
                const Fixed accel = body.gravity[k] + times(body.force[k], inverse);
                body.velocity[k] += accel * dt;
                if (body.linear_damping.raw() > 0) body.velocity[k] -= body.velocity[k] * body.linear_damping * dt;
                body.force[k] = 0.0;
            }
            const Fixed delta[2] = {body.velocity[0] * dt, body.velocity[1] * dt};
            MoveResult2D result;
            if (!move(body, delta, body.collision_mask, result)) return;
            bool hit = false;
            Fixed impact = 0.0;
            for (int axis = 0; axis < 2; ++axis) {
                if (result.blocker[axis] < 0) continue;
                hit = true;
                const Fixed normal_speed = magnitude(body.velocity[axis]);
                if (normal_speed > impact) impact = normal_speed;
                RigidBody2DComponent* other = rigid_of(actor_at(result.blocker[axis]));
                if (other && other != &body && !other->frozen) {
                    exchange(body, *other, axis);
                    continue;
                }
                // Bounce off a static obstacle; slow resting bounces settle to zero.
                Fixed bounced = -times(body.velocity[axis], body.bounce.raw());
                if (magnitude(bounced) < Fixed(16.0)) bounced = 0.0;
                body.velocity[axis] = bounced;
                // Friction along the surface, in proportion to how hard it was hit.
                const int other_axis = 1 - axis;
                const Fixed grip = times(normal_speed, body.friction.raw() + times(body.friction, body.bounce.raw()).raw());
                if (magnitude(body.velocity[other_axis]) <= grip) body.velocity[other_axis] = 0.0;
                else body.velocity[other_axis] -= body.velocity[other_axis] > 0.0 ? grip : -grip;
            }
            // Circles roll along what they stand on (visual: the node turns).
            const Collider2DComponent* shape = solid_collider_of(body);
            SceneComponent2D* root = root_of(body);
            if (body.rolls && shape && root && shape->shape.shape == Collider2D::Circle && shape->shape.radius.raw() > 0) {
                // Rolling without slipping: degrees per second = speed / radius * 57.2958.
                if (result.blocker[1] >= 0)
                    body.angular_velocity = detail::clamp_raw_2d(int64_t(body.velocity[0].raw()) * 234684 / shape->shape.radius.raw());
                root->transform.rotation = Fixed(wrap_degrees_raw(root->transform.rotation.raw() + (body.angular_velocity * dt).raw()), Fixed::RAW);
            }
            if (hit && !body.touching) {
                body.contact = true;
                body.impact = impact;
            }
            body.touching = hit;
            // Sleep after half a second of resting contact.
            const bool still = magnitude(body.velocity[0]) < Fixed(3.0) && magnitude(body.velocity[1]) < Fixed(3.0);
            if (body.can_sleep && hit && still) {
                body.rest_clock += dt;
                if (body.rest_clock > Fixed(0.5)) {
                    body.sleeping = true;
                    body.velocity[0] = body.velocity[1] = 0.0;
                }
            } else {
                body.rest_clock = 0.0;
            }
        });
        // Rods and ropes hold their length, a few passes so chains settle.
        for (int pass = 0; pass < 4; ++pass)
            joints.each([&](Joint2DComponent& joint) {
                if (joint.enabled && joint.kind != JointKind2D::Spring && owner_active(joint)) constrain(joint);
            });
    }
    // The two ends of a joint: positions, bodies (null for a fixed end) and whether valid.
    bool joint_ends(Joint2DComponent& joint, Fixed* pa, Fixed* pb, RigidBody2DComponent*& a, RigidBody2DComponent*& b) {
        b = rigid_of(joint.body_b);
        if (!b || !position_of(*b, pb)) return false;
        a = rigid_of(joint.body_a);
        if (a) return position_of(*a, pa);
        if (joint.body_a.valid()) {
            SceneComponent2D* other = actor_root(joint.body_a);
            Affine2D m;
            if (!other || !root_world(*other, m)) return false;
            pa[0] = m.t[0];
            pa[1] = m.t[1];
            return true;
        }
        return position_of(joint, pa);
    }
    void spring(Joint2DComponent& joint) {
        Fixed pa[2], pb[2];
        RigidBody2DComponent *a, *b;
        if (!joint_ends(joint, pa, pb, a, b)) return;
        const Fixed dx = pb[0] - pa[0], dy = pb[1] - pa[1];
        const int64_t d = length_2d(dx, dy).raw();
        if (d <= 0) return;
        // Stretch times stiffness, plus damping of the closing speed, along the joint.
        const Fixed va[2] = {a ? a->velocity[0] : Fixed(0.0), a ? a->velocity[1] : Fixed(0.0)};
        const int64_t closing = (int64_t(b->velocity[0].raw() - va[0].raw()) * dx.raw() + int64_t(b->velocity[1].raw() - va[1].raw()) * dy.raw()) / d;
        const int64_t pull = (d - joint.length.raw()) * joint.stiffness.raw() / 4096 + closing * joint.damping.raw() / 4096;
        const Fixed f[2] = {detail::clamp_raw_2d(pull * dx.raw() / d), detail::clamp_raw_2d(pull * dy.raw() / d)};
        b->apply_force(-f[0], -f[1]);
        if (a) a->apply_force(f[0], f[1]);
    }
    void constrain(Joint2DComponent& joint) {
        Fixed pa[2], pb[2];
        RigidBody2DComponent *a, *b;
        if (!joint_ends(joint, pa, pb, a, b)) return;
        const Fixed dx = pb[0] - pa[0], dy = pb[1] - pa[1];
        const int64_t d = length_2d(dx, dy).raw();
        if (d <= 0) return;
        const int64_t stretch = d - joint.length.raw();
        if (joint.kind == JointKind2D::Rope && stretch <= 0) return;
        const int64_t wa = a ? a->inverse_mass() : 0, wb = b->inverse_mass();
        if (wa + wb <= 0) return;
        // Move both ends along the joint, in proportion to their inverse masses.
        const Fixed n[2] = {Fixed(int32_t(int64_t(dx.raw()) * 4096 / d), Fixed::RAW), Fixed(int32_t(int64_t(dy.raw()) * 4096 / d), Fixed::RAW)};
        const int64_t share_b = wb * 4096 / (wa + wb), share_a = 4096 - share_b;
        Fixed next_b[2], next_a[2];
        for (int k = 0; k < 2; ++k) {
            next_b[k] = pb[k] - detail::clamp_raw_2d(stretch * n[k].raw() / 4096 * share_b / 4096);
            next_a[k] = pa[k] + detail::clamp_raw_2d(stretch * n[k].raw() / 4096 * share_a / 4096);
        }
        if (SceneComponent2D* root = root_of(*b)) set_world_position(*root, next_b);
        if (a && wa)
            if (SceneComponent2D* root = root_of(*a)) set_world_position(*root, next_a);
        // Remove the relative speed along the joint, so it holds without bouncing.
        const Fixed va[2] = {a ? a->velocity[0] : Fixed(0.0), a ? a->velocity[1] : Fixed(0.0)};
        const int64_t along = (int64_t(b->velocity[0].raw() - va[0].raw()) * n[0].raw() + int64_t(b->velocity[1].raw() - va[1].raw()) * n[1].raw()) / 4096;
        if (joint.kind == JointKind2D::Rope && along <= 0) return;
        for (int k = 0; k < 2; ++k) {
            b->velocity[k] -= detail::clamp_raw_2d(along * n[k].raw() / 4096 * share_b / 4096);
            if (a && wa) a->velocity[k] += detail::clamp_raw_2d(along * n[k].raw() / 4096 * share_a / 4096);
        }
        if (stretch > 64 || stretch < -64) {
            b->wake();
            if (a) a->wake();
        }
    }

    template<class LevelType> void step(Fixed dt, LevelType* level) {
        sprites.each([&](Sprite2DComponent& sprite) { sprite.advance(dt); });
        if (nav_agents.head) navigate(dt);
        tile_clock += dt;
        if (tile_clock > Fixed(65536.0)) tile_clock -= Fixed(65536.0);
        // Path followers move first, so this step's collisions and triggers see them, and
        // carry the bodies standing on them.
        followers.each([&](PathFollow2DComponent& follower) {
            if (!owner_active(follower)) return;
            Fixed moved[2];
            if (!follower.advance(dt, moved) || (!moved[0].raw() && !moved[1].raw())) return;
            Actor* owner = follower.get_owner();
            const ObjectId id = owner ? owner->id() : ObjectId{};
            bodies.each([&](CharacterBody2DComponent& body) {
                if (body.get_owner() != owner && body.on_floor && body.last_collider == id) displace_owner(body, moved);
            });
        });
        // Skeletons pose their bones (and solve IK) before the colliders are gathered.
        skeletons.each([&](Skeleton2DComponent& skeleton) {
            if (owner_active(skeleton)) skeleton.advance(dt);
        });
        if (rigid_bodies.head) simulate_bodies(dt);
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
        remotes.each([](RemoteTransform2DComponent& remote) {
            if (is_active(&remote)) remote.push();
        });
        // Lines through nodes follow them once everything has moved this step.
        lines.each([](Line2DComponent& line) {
            if (line.follow_count && is_active(&line)) line.follow_nodes();
        });
        if (notifiers.head || audio_players.head) {
            Camera2D camera;
            if (auto* active = active_camera()) camera = active->view();
            const Projector projector(camera);
            notifiers.each([&](VisibleOnScreenNotifier2DComponent& notifier) { notice(notifier, projector); });
            const Fixed listener[2] = {camera.position[0], camera.position[1]};
            const Fixed half = camera.zoom.raw() > 0 ? Fixed(int32_t(display_width) * 2048, Fixed::RAW) / camera.zoom : Fixed(160.0);
            audio_players.each([&](AudioPlayer2DComponent& player) {
                Affine2D matrix;
                if (is_active(&player) && world_of(player, matrix)) player.spatialize(matrix.t, listener, half);
            });
        }
        simulate_particles(dt);
    }
    // Whether a notifier's rectangle is inside the view; reports changes and, when asked,
    // deactivates its owner off screen.
    static void notice(VisibleOnScreenNotifier2DComponent& notifier, const Projector& projector) {
        Affine2D matrix;
        if (!world_of(notifier, matrix)) return;
        const Fixed* r = notifier.rect;
        const Fixed corners[4][2] = {{r[0], r[1]}, {r[0] + r[2], r[1]}, {r[0], r[1] + r[3]}, {r[0] + r[2], r[1] + r[3]}};
        int32_t bounds[4] = {0x7fffffff, 0x7fffffff, -0x7fffffff, -0x7fffffff};
        for (const auto& corner : corners) {
            Fixed world[2], screen[2];
            matrix.point(corner, world);
            projector.point(world, screen);
            const int32_t x = screen[0].raw() >> 12, y = screen[1].raw() >> 12;
            if (x < bounds[0]) bounds[0] = x;
            if (y < bounds[1]) bounds[1] = y;
            if (x > bounds[2]) bounds[2] = x;
            if (y > bounds[3]) bounds[3] = y;
        }
        const bool on = bounds[2] >= 0 && bounds[3] >= 0 && bounds[0] < display_width && bounds[1] < display_height;
        if (notifier.known && on == notifier.on_screen) return;
        if (notifier.known) (on ? notifier.entered : notifier.exited) = true;
        notifier.on_screen = on;
        notifier.known = true;
        if (notifier.disable_when_hidden)
            if (Actor* owner = notifier.get_owner()) owner->set_active(on);
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
        for (auto& p : particles) p.owner = nullptr;
        emitters.each([](Particles2DComponent& e) { e.alive = 0; });
        stats = {};
    }
};
inline Service service;

inline void install() {
    nav_scratch = NavScratch{service.nav_cost, service.nav_info, service.nav_heap, EPOK_WORLD2D_NAV_CELLS, EPOK_WORLD2D_NAV_OPEN};
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
    release_particles = [](Particles2DComponent& emitter) { service.release(emitter); };
}


// ---- draw list ----------------------------------------------------------------------------
enum class DrawKind : uint8_t { Sprite, Tiles, Line, Polygon, Particles };
// One entry of the frame's draw list: a sprite resolved to screen corners (A top-left,
// B top-right, C bottom-left, D bottom-right in the sprite's own frame), its texel
// rectangle, colour and blend; or a whole tile layer, line, polygon or particle emitter,
// expanded by the renderer (`each_tile`, `each_line_quad`, `each_polygon_triangle`,
// `each_particle`).
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
    // Entries the renderer expands (tile layers, lines, polygons, particles): the
    // component, whether it is drawn in screen pixels, and the screen shift of a repeated
    // parallax copy (plus a screen canvas offset). `color` is then the canvas tint.
    DrawKind kind = DrawKind::Sprite;
    const ActorComponent* component = nullptr;
    bool screen = false;
    int16_t shift[2] = {0, 0};
};
// One quad of a Line2D (corners as in SpriteDraw) with a colour per corner.
struct ShapeQuad {
    int16_t x[4], y[4];
    uint8_t color[4][3];
};
// One triangle of a Polygon2D, with texels when the polygon is textured.
struct ShapeTriangle {
    int16_t x[3], y[3];
    uint16_t u[3], v[3];
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
// The camera of the view being collected: the render pass's, else the active one.
inline bool view_camera(Camera2D& out) {
    if (render_pass.camera) {
        out = *render_pass.camera;
        return true;
    }
    if (auto* active = service.active_camera()) {
        out = active->view();
        return true;
    }
    return false;
}
}  // namespace world2d
inline bool SubViewport2DComponent::view(Camera2D& out) const {
    Camera2DComponent* source = camera.valid() ? world2d::camera_on(camera) : world2d::service.active_camera();
    if (!source) return false;
    out = source->view();
    out.viewport[0] = out.viewport[1] = 0;
    out.viewport[2] = int16_t(width);
    out.viewport[3] = int16_t(height);
    return true;
}
namespace world2d {
template<class TextureSize>
size_t collect_sprites(SpriteDraw* draws, DrawItem* items, DrawItem* scratch, size_t capacity, TextureSize&& texture_size) {
    stats.sprites = stats.culled = stats.dropped = stats.tile_layers = stats.tiles = 0;
    if (!sprites.head && !tile_layers.head && !lines.head && !polygons.head && !emitters.head) return 0;
    Camera2D camera;
    view_camera(camera);
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
        if (!canvas.visible || !render_pass.accepts(canvas)) return;
        uint16_t width = 0, height = 0;
        const SubViewport2DComponent* picture = sprite.viewport.valid() ? subviewport_on(sprite.viewport) : nullptr;
        if (picture) {
            width = picture->width;
            height = picture->height;
        } else if (!texture_size(int(sprite.texture), width, height)) {
            ++stats.culled;
            return;
        }
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
        draw.texture = picture ? -2 : sprite.texture;
        if (picture) draw.component = picture;
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
        else if (!(bounds[2] <= render_pass.clip[0] || bounds[3] <= render_pass.clip[1] ||
                   bounds[0] >= render_pass.clip[0] + render_pass.clip[2] || bounds[1] >= render_pass.clip[1] + render_pass.clip[3]))
            emit(0, 0);
        if (!drawn) ++stats.culled;
    });
    tile_layers.each([&](TileMapLayerComponent& layer) {
        if (!layer.visible || !layer.tileset || !is_active(&layer)) return;
        const Canvas canvas = canvas_of(layer);
        if (!canvas.visible || !render_pass.accepts(canvas)) return;
        ++stats.tile_layers;
        SpriteDraw draw;
        draw.kind = DrawKind::Tiles;
        draw.component = &layer;
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
    // Lines, polygons and particle emitters: one entry each, on their canvas.
    auto shape = [&](ActorComponent& component, DrawKind kind, int16_t z, BlendMode blend, int32_t texture) {
        const Canvas canvas = canvas_of(component);
        if (!canvas.visible || !render_pass.accepts(canvas)) return;
        SpriteDraw draw;
        draw.kind = kind;
        draw.component = &component;
        draw.screen = canvas.screen;
        draw.blend = blend;
        draw.texture = texture;
        if (const uint8_t* by = tints.of(canvas.layer))
            for (int k = 0; k < 3; ++k) draw.color[k] = by[k];
        if (canvas.screen) {
            draw.shift[0] = screen_pixel(canvas.offset[0]);
            draw.shift[1] = screen_pixel(canvas.offset[1]);
        }
        const auto* owner_root = root_of(component);
        draw.key = z_key(int32_t(z) + (owner_root ? int32_t(owner_root->transform.draw_order) : 0));
        ++stats.shapes;
        const bool repeats = canvas.parallax && (canvas.parallax->repeat_size[0] > 0.0 || canvas.parallax->repeat_size[1] > 0.0);
        Affine2D matrix;
        if (!repeats || kind == DrawKind::Particles || !world_of(component, matrix)) {
            push(draw, canvas.layer);
            return;
        }
        // A repeating line or polygon: copies across the screen, from its points' bounds.
        const Projector& projector = canvas.screen ? screen_projector : world_projector;
        int16_t bounds[4] = {32767, 32767, -32768, -32768};
        auto grow = [&](const Fixed* local) {
            Fixed world[2], screen[2];
            matrix.point(local, world);
            projector.point(world, screen);
            const int16_t x = int16_t(screen_pixel(screen[0]) + draw.shift[0]), y = int16_t(screen_pixel(screen[1]) + draw.shift[1]);
            if (x < bounds[0]) bounds[0] = x;
            if (x > bounds[2]) bounds[2] = x;
            if (y < bounds[1]) bounds[1] = y;
            if (y > bounds[3]) bounds[3] = y;
        };
        if (kind == DrawKind::Line) {
            const auto& line = static_cast<Line2DComponent&>(component);
            for (uint16_t i = 0; i < line.point_count && i < line2d_max_points; ++i) grow(line.points[i]);
        } else {
            const auto& polygon = static_cast<Polygon2DComponent&>(component);
            for (uint16_t i = 0; i < polygon.vertex_count && polygon.vertices; ++i) grow(polygon.vertices + 2 * i);
        }
        repeat_copies(canvas, projector, bounds, [&](int16_t dx, int16_t dy) {
            SpriteDraw copy = draw;
            copy.shift[0] = int16_t(draw.shift[0] + dx);
            copy.shift[1] = int16_t(draw.shift[1] + dy);
            push(copy, canvas.layer);
        });
    };
    stats.shapes = 0;
    lines.each([&](Line2DComponent& line) {
        if (line.visible && line.point_count >= 2 && is_active(&line)) shape(line, DrawKind::Line, line.z_index, line.blend, -1);
    });
    polygons.each([&](Polygon2DComponent& polygon) {
        if (polygon.visible && polygon.triangle_count && is_active(&polygon))
            shape(polygon, DrawKind::Polygon, polygon.z_index, polygon.blend, polygon.texture);
    });
    emitters.each([&](Particles2DComponent& emitter) {
        if (emitter.visible && emitter.alive && is_active(&emitter))
            shape(emitter, DrawKind::Particles, emitter.z_index, emitter.blend, emitter.texture);
    });
    sort_draw_items(items, scratch, count);
    return count;
}

// The projection an expanded entry draws through, and its owner's world matrix.
inline bool entry_space(const SpriteDraw& entry, Camera2D& camera, Affine2D& matrix) {
    if (!entry.component) return false;
    if (entry.screen) {
        camera.position[0] = Fixed(int32_t(display_width) * 2048, Fixed::RAW);
        camera.position[1] = Fixed(int32_t(display_height) * 2048, Fixed::RAW);
    } else {
        view_camera(camera);
    }
    return world_of(*const_cast<ActorComponent*>(entry.component), matrix);
}
inline uint8_t mix(uint8_t a, uint8_t b, Fixed t, uint8_t tint) {
    const int32_t value = int32_t(a) + ((int32_t(b) - int32_t(a)) * t.raw()) / 4096;
    return uint8_t((value * int32_t(tint) + 127) / 255);
}
// The quads of a Line2D entry, one per segment, extended by half the width at inner
// joints so corners close.
template<class Visit> inline void each_line_quad(const SpriteDraw& entry, Visit&& visit) {
    if (entry.kind != DrawKind::Line) return;
    const auto& line = *static_cast<const Line2DComponent*>(entry.component);
    Camera2D camera;
    Affine2D matrix;
    if (!entry_space(entry, camera, matrix)) return;
    const Projector projector(camera);
    const uint16_t count = line.point_count > line2d_max_points ? line2d_max_points : line.point_count;
    const uint16_t segments = uint16_t(line.closed ? count : count - 1);
    if (count < 2) return;
    // Screen-space half width: the owner's scale (largest axis) and the camera's zoom.
    const Fixed sx = length_2d(matrix.m[0][0], matrix.m[1][0]), sy = length_2d(matrix.m[0][1], matrix.m[1][1]);
    const Fixed half = detail::mul_2d(detail::mul_2d(line.width, sx > sy ? sx : sy), projector.scale) / 2;
    if (half.raw() <= 0) return;
    for (uint16_t i = 0; i < segments; ++i) {
        const uint16_t j = uint16_t((i + 1) % count);
        Fixed a[2], b[2];
        matrix.point(line.points[i], a);
        matrix.point(line.points[j], b);
        projector.point(a, a);
        projector.point(b, b);
        const Fixed dx = b[0] - a[0], dy = b[1] - a[1];
        const Fixed length = length_2d(dx, dy);
        if (!length.raw()) continue;
        const Fixed ux = dx / length, uy = dy / length;
        const Fixed nx = detail::mul_2d(-uy, half), ny = detail::mul_2d(ux, half);
        // Square caps at inner joints overlap the next segment, so corners have no gaps.
        const bool open_start = !line.closed && i == 0, open_end = !line.closed && j == count - 1;
        if (!open_start) { a[0] -= detail::mul_2d(ux, half); a[1] -= detail::mul_2d(uy, half); }
        if (!open_end) { b[0] += detail::mul_2d(ux, half); b[1] += detail::mul_2d(uy, half); }
        ShapeQuad quad;
        const Fixed corners[4][2] = {{a[0] + nx, a[1] + ny}, {b[0] + nx, b[1] + ny}, {a[0] - nx, a[1] - ny}, {b[0] - nx, b[1] - ny}};
        for (int k = 0; k < 4; ++k) {
            quad.x[k] = int16_t(screen_pixel(corners[k][0]) + entry.shift[0]);
            quad.y[k] = int16_t(screen_pixel(corners[k][1]) + entry.shift[1]);
        }
        const Fixed t0 = Fixed(int32_t(i) * 4096 / segments, Fixed::RAW), t1 = Fixed(int32_t(i + 1) * 4096 / segments, Fixed::RAW);
        for (int c = 0; c < 3; ++c) {
            quad.color[0][c] = quad.color[2][c] = mix(line.begin_color[c], line.end_color[c], t0, entry.color[c]);
            quad.color[1][c] = quad.color[3][c] = mix(line.begin_color[c], line.end_color[c], t1, entry.color[c]);
        }
        int16_t min_x = 32767, max_x = -32768, min_y = 32767, max_y = -32768;
        for (int k = 0; k < 4; ++k) {
            if (quad.x[k] < min_x) min_x = quad.x[k];
            if (quad.x[k] > max_x) max_x = quad.x[k];
            if (quad.y[k] < min_y) min_y = quad.y[k];
            if (quad.y[k] > max_y) max_y = quad.y[k];
        }
        if (max_x <= 0 || max_y <= 0 || min_x >= display_width || min_y >= display_height) continue;
        if (max_x - min_x > 1023 || max_y - min_y > 511) continue;
        visit(quad);
    }
}
// The triangles of a Polygon2D entry.
template<class Visit> inline void each_polygon_triangle(const SpriteDraw& entry, Visit&& visit) {
    if (entry.kind != DrawKind::Polygon) return;
    const auto& polygon = *static_cast<const Polygon2DComponent*>(entry.component);
    if (!polygon.vertices || !polygon.triangles) return;
    Camera2D camera;
    Affine2D matrix;
    if (!entry_space(entry, camera, matrix)) return;
    const Projector projector(camera);
    for (uint16_t t = 0; t < polygon.triangle_count; ++t) {
        ShapeTriangle triangle;
        int16_t min_x = 32767, max_x = -32768, min_y = 32767, max_y = -32768;
        bool valid = true;
        for (int k = 0; k < 3; ++k) {
            const uint16_t index = polygon.triangles[3 * t + k];
            if (index >= polygon.vertex_count) { valid = false; break; }
            Fixed world[2], screen[2];
            matrix.point(polygon.vertices + 2 * index, world);
            projector.point(world, screen);
            triangle.x[k] = int16_t(screen_pixel(screen[0]) + entry.shift[0]);
            triangle.y[k] = int16_t(screen_pixel(screen[1]) + entry.shift[1]);
            triangle.u[k] = polygon.texels ? polygon.texels[2 * index] : 0;
            triangle.v[k] = polygon.texels ? polygon.texels[2 * index + 1] : 0;
            if (triangle.x[k] < min_x) min_x = triangle.x[k];
            if (triangle.x[k] > max_x) max_x = triangle.x[k];
            if (triangle.y[k] < min_y) min_y = triangle.y[k];
            if (triangle.y[k] > max_y) max_y = triangle.y[k];
        }
        if (!valid || max_x <= 0 || max_y <= 0 || min_x >= display_width || min_y >= display_height) continue;
        if (max_x - min_x > 1023 || max_y - min_y > 511) continue;
        visit(triangle);
    }
}
// The particles of an emitter entry as sprites (texel rectangle, colour, corners).
template<class Visit>
inline void each_particle(const SpriteDraw& entry, uint16_t texture_width, uint16_t texture_height, Visit&& visit) {
    if (entry.kind != DrawKind::Particles) return;
    const auto& emitter = *static_cast<const Particles2DComponent*>(entry.component);
    Camera2D camera;
    Affine2D matrix;
    if (!entry_space(entry, camera, matrix)) return;
    const Projector projector(camera);
    uint16_t region[4] = {emitter.region[0], emitter.region[1], emitter.region[2], emitter.region[3]};
    if (!region[2]) region[2] = uint16_t(texture_width > region[0] ? texture_width - region[0] : 0);
    if (!region[3]) region[3] = uint16_t(texture_height > region[1] ? texture_height - region[1] : 0);
    const uint16_t frames = emitter.hframes ? emitter.hframes : 1;
    const uint16_t cell = uint16_t(region[2] / frames);
    if (!cell || !region[3] || uint32_t(region[0]) + region[2] > texture_width || uint32_t(region[1]) + region[3] > texture_height) return;
    service.each_particle_of(emitter, [&](const Fixed* position, Fixed t) {
        Fixed world[2] = {position[0], position[1]};
        if (emitter.local_coords) matrix.point(position, world);
        Fixed screen[2];
        projector.point(world, screen);
        const Fixed size = detail::mul_2d(emitter.scale[0] + detail::mul_2d(emitter.scale[1] - emitter.scale[0], t), projector.scale);
        SpriteDraw draw;
        uint16_t frame = uint16_t((t.raw() * frames) >> 12);
        if (frame >= frames) frame = uint16_t(frames - 1);
        draw.source[0] = uint16_t(region[0] + frame * cell);
        draw.source[1] = region[1];
        draw.source[2] = cell;
        draw.source[3] = region[3];
        const Fixed half_w = detail::mul_2d(Fixed(int32_t(cell) * 2048, Fixed::RAW), size);
        const Fixed half_h = detail::mul_2d(Fixed(int32_t(region[3]) * 2048, Fixed::RAW), size);
        const int16_t left = int16_t(screen_pixel(screen[0] - half_w) + entry.shift[0]), right = int16_t(screen_pixel(screen[0] + half_w) + entry.shift[0]);
        const int16_t top = int16_t(screen_pixel(screen[1] - half_h) + entry.shift[1]), bottom = int16_t(screen_pixel(screen[1] + half_h) + entry.shift[1]);
        if (right <= 0 || bottom <= 0 || left >= display_width || top >= display_height || right <= left || bottom <= top) return;
        draw.x[0] = draw.x[2] = left;
        draw.x[1] = draw.x[3] = right;
        draw.y[0] = draw.y[1] = top;
        draw.y[2] = draw.y[3] = bottom;
        for (int k = 0; k < 3; ++k) draw.color[k] = mix(emitter.start_color[k], emitter.end_color[k], t, entry.color[k]);
        draw.texture = emitter.texture;
        draw.blend = emitter.blend;
        draw.rectangle = right - left == int16_t(cell) && bottom - top == int16_t(region[3]);
        visit(draw);
    });
}

// Every visible tile of a tile-layer entry, in row order. `texture_width/height` bound the
// atlas; tiles whose frame falls outside it are skipped.
template<class Visit>
inline void each_tile(const SpriteDraw& entry, uint16_t texture_width, uint16_t texture_height, Visit&& visit) {
    if (entry.kind != DrawKind::Tiles || !entry.component) return;
    const auto* layer = static_cast<const TileMapLayerComponent*>(entry.component);
    if (!layer->tileset) return;
    Camera2D camera;
    if (!entry.screen) view_camera(camera);
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
