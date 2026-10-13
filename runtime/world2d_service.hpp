#pragma once
// World2D service (World2D initiative, phase 1): the collision world behind Collider2D and
// CharacterBody2D, trigger dispatch, Camera2D following, sprite animation and the native
// 2D renderer. Only the generated application includes this header; it installs
// `world2d::hooks` so components reach it without depending on it.
//
// Capacities come from the cooked scene header (defaults below):
//   EPOK_WORLD2D_COLLIDERS     colliders resolved per step (CollisionWorld2D capacity)
//   EPOK_WORLD2D_TRIGGER_PAIRS remembered overlapping trigger pairs
//   EPOK_WORLD2D_DRAW_ITEMS    sprites resolved for drawing per frame
//   EPOK_WORLD2D_DRAW_ITEMS    sprites drawn per frame
//   EPOK_WORLD2D_ARENA_BYTES   GPU packet bytes per frame parity
// Exceeding any of them drops the extra work and counts it in `world2d::stats`.
//
// Rendering lives in world2d_render.hpp; this header has no GPU dependency, so host tests
// exercise the collision, trigger, camera and animation step directly.
#include "world2d_components.hpp"

#ifndef EPOK_WORLD2D_COLLIDERS
#define EPOK_WORLD2D_COLLIDERS 64
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
};
inline Stats stats;

// Stable LSD radix sort of draw items by their 32-bit key. Passes whose byte is equal for
// every item are skipped, so a scene drawn on one z level costs two passes at most.
struct DrawItem {
    uint32_t key;
    uint16_t index;
};
inline void sort_draw_items(DrawItem* items, DrawItem* scratch, size_t count) {
    if (count < 2) return;
    DrawItem* from = items;
    DrawItem* to = scratch;
    for (int shift = 0; shift < 32; shift += 8) {
        uint16_t buckets[257] = {};
        for (size_t i = 0; i < count; ++i) ++buckets[((from[i].key >> shift) & 255) + 1];
        bool trivial = false;
        for (int b = 1; b <= 256; ++b)
            if (buckets[b] == count) trivial = true;
        if (trivial) continue;
        for (int b = 0; b < 256; ++b) buckets[b + 1] += buckets[b];
        for (size_t i = 0; i < count; ++i) to[buckets[(from[i].key >> shift) & 255]++] = from[i];
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
    explicit Projector(const Camera2D& camera) {
        sine = sin_degrees(-camera.rotation);
        cosine = cos_degrees(-camera.rotation);
        scale = camera2d_pixels_per_unit(camera);
        camera2d_viewport_center(camera, center);
        origin[0] = camera.position[0];
        origin[1] = camera.position[1];
        identity = !camera.rotation.raw() && scale.raw() == 4096;
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

class Service {
    CollisionWorld2D<EPOK_WORLD2D_COLLIDERS, EPOK_WORLD2D_TRIGGER_PAIRS> world;
    Collider2DComponent* slots[EPOK_WORLD2D_COLLIDERS] = {};
    size_t slot_count = 0;
    uint32_t synced_tick = 0xffffffffu;

    static bool owner_active(ActorComponent& component) { return is_active(&component); }
    static ObjectId owner_of(Collider2DComponent* collider) {
        auto* owner = collider ? collider->get_owner() : nullptr;
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

public:
    // ---- collision ---------------------------------------------------------------------
    void sync(bool force = false) {
        if (!force && synced_tick == time.ticks) return;
        synced_tick = time.ticks;
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
        slot_count = count;
        stats.colliders = uint32_t(count);
        stats.dropped_colliders = dropped;
    }
    bool move(ActorComponent& body, const Fixed* delta, uint32_t mask, MoveResult2D& result) {
        sync();
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
        result = world.move_and_slide_2d(size_t(collider->slot), delta, mask & collider->shape.layer_mask);
        return displace_owner(body, result.displacement);
    }
    ObjectId actor_at(int index) const {
        return index >= 0 && size_t(index) < slot_count ? owner_of(slots[index]) : ObjectId{};
    }
    SpatialHit2D raycast(const Fixed* origin, const Fixed* delta, uint32_t mask, bool triggers) {
        sync();
        return world.raycast2d(origin, delta, mask, -1, triggers);
    }
    size_t overlap(const Aabb2D& box, ObjectId* actors, size_t capacity, uint32_t mask, bool triggers) {
        sync();
        uint16_t indices[EPOK_WORLD2D_COLLIDERS];
        const size_t total = world.overlap(box, indices, EPOK_WORLD2D_COLLIDERS, mask, -1, triggers);
        for (size_t i = 0; actors && i < total && i < capacity && i < EPOK_WORLD2D_COLLIDERS; ++i) actors[i] = actor_at(indices[i]);
        return total;
    }

    // ---- simulation step ---------------------------------------------------------------
    // After the level tick: animate, resolve this step's colliders and triggers, then
    // move the cameras to where the actors ended up.
    template<class LevelType> void step(Fixed dt, LevelType* level) {
        sprites.each([&](Sprite2DComponent& sprite) { sprite.advance(dt); });
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
        synced_tick = 0xffffffffu;
        colliders.each([](Collider2DComponent& collider) { collider.slot = -1; });
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
}


// ---- sprite resolution -------------------------------------------------------------------
// One sprite resolved for drawing: screen corners in the sprite's own frame (A top-left,
// B top-right, C bottom-left, D bottom-right), its texel rectangle, colour and blend.
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
};
// Resolves every visible sprite through the active camera into `draws`, and writes the
// back-to-front order into `items` (`scratch` is sort workspace). `texture_size(index,
// width, height)` reports a texture's size and returns false when it is absent.
// Returns the number of sprites to draw; extra sprites are counted as dropped.
template<class TextureSize>
size_t collect_sprites(SpriteDraw* draws, DrawItem* items, DrawItem* scratch, size_t capacity, TextureSize&& texture_size) {
    stats.sprites = stats.culled = stats.dropped = 0;
    if (!sprites.head) return 0;
    Camera2D camera;
    if (auto* active = service.active_camera()) camera = active->view();
    const Projector projector(camera);
    size_t count = 0;
    sprites.each([&](Sprite2DComponent& sprite) {
        ++stats.sprites;
        if (!sprite.visible || !is_active(&sprite)) return;
        uint16_t width = 0, height = 0;
        if (!texture_size(int(sprite.texture), width, height)) { ++stats.culled; return; }
        uint16_t source[4];
        if (!sprite.source_rect(width, height, source)) { ++stats.culled; return; }
        Affine2D matrix;
        if (!world_of(sprite, matrix)) return;
        const Fixed w(int32_t(source[2]) * 4096, Fixed::RAW), h(int32_t(source[3]) * 4096, Fixed::RAW);
        const Fixed left = (sprite.centered ? -(w / 2) : Fixed(0.0)) + sprite.offset[0];
        const Fixed top = (sprite.centered ? -(h / 2) : Fixed(0.0)) + sprite.offset[1];
        const Fixed local[4][2] = {{left, top}, {left + w, top}, {left, top + h}, {left + w, top + h}};
        SpriteDraw draw;
        int16_t min_x = 32767, min_y = 32767, max_x = -32768, max_y = -32768;
        for (int i = 0; i < 4; ++i) {
            Fixed world[2], screen[2];
            matrix.point(local[i], world);
            projector.point(world, screen);
            draw.x[i] = screen_pixel(screen[0]);
            draw.y[i] = screen_pixel(screen[1]);
            if (draw.x[i] < min_x) min_x = draw.x[i];
            if (draw.x[i] > max_x) max_x = draw.x[i];
            if (draw.y[i] < min_y) min_y = draw.y[i];
            if (draw.y[i] > max_y) max_y = draw.y[i];
        }
        if (max_x <= 0 || max_y <= 0 || min_x >= display_width || min_y >= display_height) { ++stats.culled; return; }
        // The GPU rejects primitives wider than 1023 or taller than 511 pixels.
        if (max_x - min_x > 1023 || max_y - min_y > 511) { ++stats.dropped; return; }
        if (count >= capacity) { ++stats.dropped; return; }
        const auto* owner_root = root_of(sprite);
        int32_t z = int32_t(sprite.z_index) + (owner_root ? int32_t(owner_root->transform.draw_order) : 0);
        z = z < -32768 ? -32768 : z > 32767 ? 32767 : z;
        draw.key = uint32_t(z + 32768) << 16;
        if (sprite.y_sort) {
            const int32_t row = (matrix.t[1].raw() >> 12) + 32768;
            draw.key |= uint32_t(row < 1 ? 1 : row > 65535 ? 65535 : row);
        }
        for (int k = 0; k < 4; ++k) draw.source[k] = source[k];
        for (int k = 0; k < 3; ++k) draw.color[k] = sprite.modulate[k];
        draw.texture = sprite.texture;
        draw.blend = sprite.blend;
        draw.flip_h = sprite.flip_h;
        draw.flip_v = sprite.flip_v;
        const bool axis_aligned = matrix.m[0][0].raw() == 4096 && matrix.m[1][1].raw() == 4096 && !matrix.m[0][1].raw() &&
                                  !matrix.m[1][0].raw();
        draw.rectangle = projector.identity && axis_aligned && !sprite.flip_h && !sprite.flip_v;
        draws[count] = draw;
        items[count] = DrawItem{draw.key, uint16_t(count)};
        ++count;
    });
    sort_draw_items(items, scratch, count);
    return count;
}
}  // namespace world2d
}  // namespace epok
