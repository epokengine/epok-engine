#pragma once
// World2D components (World2D initiative, phase 1): the authored 2D sprite, camera,
// collider and character body of an Actor2D.
//
// Units follow world2d.hpp: Q12 pixels, X right, Y down, degrees.
//
// Registration. Every component links itself into an intrusive list when the Level binds
// it to its owner (`level_attached`, a hook scripts cannot override) and unlinks when it
// is destroyed, so a Blueprint or Lua subclass that overrides begin_play() without
// calling its parent still renders and collides. Lists
// carry no capacity and no storage: a game without 2D content pays nothing for them.
// Activation is evaluated where the lists are consumed (an inactive owner neither draws
// nor collides) instead of being mirrored into the lists.
//
// Services. The renderer, the collision world and the camera follower live in
// world2d_service.hpp, which only the generated application includes. That service
// installs `world2d::hooks`; without it (host HUD preview, reflection) every query
// below fails gracefully and reports nothing.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d.hpp"
#include <stddef.h>
#include <stdint.h>

namespace epok {
class Sprite2DComponent;
class Camera2DComponent;
class Collider2DComponent;
class CharacterBody2DComponent;
class TileMapLayerComponent;

namespace world2d {
// Intrusive list link. Copying a component (bank reset assigns default state) never
// copies its links, so a copied value cannot corrupt the list it does not belong to.
template<class T> struct Link {
    T* prev = nullptr;
    T* next = nullptr;
    bool linked = false;
    Link() = default;
    Link(const Link&) {}
    Link& operator=(const Link&) { return *this; }
};
template<class T> struct List {
    T* head = nullptr;
    T* tail = nullptr;
    uint16_t count = 0;
    void add(T* item) {
        auto& link = item->link2d;
        if (link.linked) return;
        link.prev = tail;
        link.next = nullptr;
        if (tail) tail->link2d.next = item;
        else head = item;
        tail = item;
        link.linked = true;
        ++count;
    }
    void remove(T* item) {
        auto& link = item->link2d;
        if (!link.linked) return;
        if (link.prev) link.prev->link2d.next = link.next;
        else head = link.next;
        if (link.next) link.next->link2d.prev = link.prev;
        else tail = link.prev;
        link.prev = link.next = nullptr;
        link.linked = false;
        --count;
    }
    // Safe against the visited item unlinking itself.
    template<class F> void each(F&& visit) const {
        for (T* item = head; item;) {
            T* next = item->link2d.next;
            visit(*item);
            item = next;
        }
    }
};
inline List<Sprite2DComponent> sprites;
inline List<Camera2DComponent> cameras;
inline List<Collider2DComponent> colliders;
inline List<CharacterBody2DComponent> bodies;

// The tile map cell behind a collision index, installed by the service when tile maps
// collide: false when `index` is an ordinary collider.
inline bool (*tile_cell_at)(int index, int32_t& x, int32_t& y) = nullptr;

// The root SceneComponent2D of a component's owner, or null.
inline SceneComponent2D* root_of(ActorComponent& component) {
    auto* owner = component.get_owner();
    auto* registry = active_object_registry;
    return owner && registry ? registry->resolve<SceneComponent2D>(owner->root_id()) : nullptr;
}
inline const SceneComponent2D* resolve_scene2d(ObjectId id) {
    return active_object_registry ? active_object_registry->resolve<SceneComponent2D>(id) : nullptr;
}
// World matrix of the owner's root, following SceneComponent2D attachments.
inline bool world_of(ActorComponent& component, Affine2D& out) {
    auto* root = root_of(component);
    return root && world_matrix_2d(*root, [](ObjectId id) { return resolve_scene2d(id); }, out);
}
inline Fixed step_seconds() { return Fixed(time.delta_raw, Fixed::RAW); }
// The first component of type T on `actor`, or null. C++ convenience for gameplay code:
// `auto* body = world2d::find<CharacterBody2DComponent>(get_owner());`
template<class T> T* find(Actor* actor) {
    auto* registry = active_object_registry;
    if (!actor || !registry) return nullptr;
    for (size_t i = 0; i < actor->component_count(); ++i)
        if (auto* component = registry->resolve<T>(actor->component_id(i))) return component;
    return nullptr;
}
}  // namespace world2d

// ---- Sprite2D --------------------------------------------------------------------------
//
// One textured quad, optionally animated by cooked frame clips (a still sprite and a
// frame-animated sprite in one component). The source rectangle is, in priority order:
//   1. the current frame of the playing clip, when the component has clips;
//   2. cell `frame` of an `hframes` x `vframes` grid over `region` (sprite sheets);
//   3. `region` itself (width/height 0 = the rest of the texture).
// The quad is `region` texels large at scale 1, centred on the owner when `centered`,
// then moved by `offset`. Flips mirror the texture, not the transform.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=sprite2d, Id="892ac5c9-eae8-48b8-853a-db93d20a537b") Sprite2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("892ac5c9-eae8-48b8-853a-db93d20a537b");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::sprites.add(this); }
    ~Sprite2DComponent() override { world2d::sprites.remove(this); }
    world2d::Link<Sprite2DComponent> link2d;

    // Authored state. The editor cooks it; gameplay changes it through the functions below.
    int32_t texture = -1;
    // An actor with a SubViewport2D: its picture replaces the texture.
    ObjectId viewport;
    uint16_t region[4] = {0, 0, 0, 0};
    uint16_t hframes = 1, vframes = 1;
    Fixed offset[2] = {0.0, 0.0};
    uint8_t modulate[3] = {255, 255, 255};
    BlendMode blend = BlendMode::Cutout;
    int16_t z_index = 0;
    bool centered = true, flip_h = false, flip_v = false, visible = true, y_sort = false;
    // Cooked frame clips (SpriteFrames). Regions are absolute texel rectangles.
    const SpriteClip* clips = nullptr;
    uint16_t clip_count = 0, animation = 0, frame = 0;
    Fixed speed_scale = 1.0, elapsed = 0.0;
    bool playing = false, finished = false;
    uint16_t pending_event = 0;

    // Shows the picture of the SubViewport2D on `actor` (an empty id: the texture again).
    EPOK_FUNCTION(BlueprintCallable, Id="7b2febaf-a185-4023-a7b9-07608bbde42e") void set_viewport(ObjectId actor) { viewport = actor; }
    EPOK_FUNCTION(BlueprintCallable, Id="232abd11-a1eb-4041-ada7-20a45d58b72e") void play(uint32_t clip) {
        if (!clips || clip >= clip_count || !clips[clip].frame_count) return;
        if (animation != clip || !playing) { animation = uint16_t(clip); frame = 0; elapsed = 0.0; }
        playing = true;
        finished = false;
        pending_event = clips[clip].frames[0].event;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="fea6d041-fb0c-4d99-b727-fe61f18e05cf") void stop() { playing = false; frame = 0; elapsed = 0.0; }
    EPOK_FUNCTION(BlueprintCallable, Id="505ef953-32be-4801-ab78-195f296907f4") void pause() { playing = false; }
    EPOK_FUNCTION(BlueprintPure, Id="6811ce30-6cb9-400b-84ab-0a5ae6280f16") bool is_playing() const { return playing; }
    EPOK_FUNCTION(BlueprintPure, Id="9273ca33-59a4-48f0-835a-438465b03d32") uint32_t current_animation() const { return animation; }
    EPOK_FUNCTION(BlueprintPure, Id="676e8560-c1d9-4526-8a0b-ebcadd814855") uint32_t animation_count() const { return clip_count; }
    EPOK_FUNCTION(BlueprintPure, Id="3cb78e69-a80c-454a-af7f-2d233a003f19") uint32_t current_frame() const { return frame; }
    EPOK_FUNCTION(BlueprintCallable, Id="f7bb899d-2229-45a8-b8d4-e29694d34b44") void set_frame(uint32_t value) {
        const uint32_t count = clips && animation < clip_count ? clips[animation].frame_count : uint32_t(hframes) * vframes;
        frame = uint16_t(count ? value % count : 0);
        elapsed = 0.0;
    }
    // True once after a non-looping clip reached its last frame.
    EPOK_FUNCTION(BlueprintCallable, Id="8c41cbdf-defd-4d02-92a9-a2b944c47460") bool take_animation_finished() { const bool value = finished; finished = false; return value; }
    // The event id of the last frame entered (0 = none), reported once.
    EPOK_FUNCTION(BlueprintCallable, Id="171a45be-30cd-4af2-84f6-eaf2f80f86fb") uint32_t take_frame_event() { const uint32_t value = pending_event; pending_event = 0; return value; }
    EPOK_FUNCTION(BlueprintCallable, Id="2b02af07-89e5-4f53-8bcb-bd44ee9cddac") void set_speed_scale(Fixed value) { speed_scale = value < 0.0 ? Fixed(0.0) : value; }
    EPOK_FUNCTION(BlueprintCallable, Id="480f44c0-403e-4090-b367-9e7f848cedd0") void set_flip_h(bool value) { flip_h = value; }
    EPOK_FUNCTION(BlueprintCallable, Id="cfe54c72-ffce-4a25-807f-0ba9440bbf28") void set_flip_v(bool value) { flip_v = value; }
    EPOK_FUNCTION(BlueprintPure, Id="138989d7-aa4a-41ff-91cc-e5b3dbc8f9d5") bool is_flipped_h() const { return flip_h; }
    EPOK_FUNCTION(BlueprintCallable, Id="eb824e25-17d0-4a66-9d65-12d156d2856e") void set_visible(bool value) { visible = value; }
    EPOK_FUNCTION(BlueprintPure, Id="c8269240-09cc-4a93-afc9-bdff322af638") bool is_visible() const { return visible; }
    EPOK_FUNCTION(BlueprintCallable, Id="7701c7a1-509d-4394-afb1-fb68c31ccabf") void set_modulate(uint32_t red, uint32_t green, uint32_t blue) {
        modulate[0] = uint8_t(red > 255 ? 255 : red);
        modulate[1] = uint8_t(green > 255 ? 255 : green);
        modulate[2] = uint8_t(blue > 255 ? 255 : blue);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="33b3264a-0cab-40c4-afce-17fbab99a900") void set_z_index(int32_t value) {
        z_index = int16_t(value < -4096 ? -4096 : value > 4096 ? 4096 : value);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="fb3e1d8c-c21e-4930-a80f-974b8262b5b7") void set_texture_region(uint32_t x, uint32_t y, uint32_t width, uint32_t height) {
        region[0] = uint16_t(x); region[1] = uint16_t(y); region[2] = uint16_t(width); region[3] = uint16_t(height);
    }
    // C++ convenience: play a clip by its authored name.
    bool play_named(const char* name) {
        for (uint16_t i = 0; clips && name && i < clip_count; ++i) {
            const char* a = clips[i].name;
            const char* b = name;
            while (a && *a && *a == *b) { ++a; ++b; }
            if (a && !*a && !*b) { play(i); return true; }
        }
        return false;
    }
    // Advances the playing clip by one simulation step. Called by the World2D service.
    void advance(Fixed dt) {
        if (!playing || !clips || animation >= clip_count) return;
        const auto& clip = clips[animation];
        if (!clip.frame_count) return;
        int64_t scaled = int64_t(dt.raw()) * speed_scale.raw() / 4096;
        if (scaled > 4096) scaled = 4096;
        elapsed += Fixed(int32_t(scaled), Fixed::RAW);
        for (int guard = 0; guard < 64; ++guard) {
            if (frame >= clip.frame_count) frame = 0;
            const Fixed duration = clip.frames[frame].duration.raw() > 0 ? clip.frames[frame].duration : Fixed(1, Fixed::RAW);
            if (elapsed < duration) break;
            elapsed -= duration;
            if (frame + 1 < clip.frame_count) ++frame;
            else if (clip.looping) frame = 0;
            else { playing = false; finished = true; elapsed = 0.0; break; }
            if (clip.frames[frame].event) pending_event = clip.frames[frame].event;
        }
    }
    // The texel rectangle drawn this frame, given the texture size. False when empty.
    bool source_rect(uint16_t texture_width, uint16_t texture_height, uint16_t* out) const {
        if (clips && animation < clip_count && clips[animation].frame_count) {
            const auto& f = clips[animation].frames[frame < clips[animation].frame_count ? frame : 0];
            for (int k = 0; k < 4; ++k) out[k] = f.region[k];
        } else {
            out[0] = region[0];
            out[1] = region[1];
            out[2] = region[2] ? region[2] : uint16_t(texture_width > region[0] ? texture_width - region[0] : 0);
            out[3] = region[3] ? region[3] : uint16_t(texture_height > region[1] ? texture_height - region[1] : 0);
            const uint16_t columns = hframes ? hframes : 1, rows = vframes ? vframes : 1;
            if (columns > 1 || rows > 1) {
                const uint16_t cell = uint16_t(frame % (columns * rows));
                out[2] = uint16_t(out[2] / columns);
                out[3] = uint16_t(out[3] / rows);
                out[0] = uint16_t(out[0] + (cell % columns) * out[2]);
                out[1] = uint16_t(out[1] + (cell / columns) * out[3]);
            }
        }
        return out[2] && out[3] && uint32_t(out[0]) + out[2] <= texture_width && uint32_t(out[1]) + out[3] <= texture_height;
    }
};

// ---- Camera2D --------------------------------------------------------------------------
//
// Follows its owner's world position. The service resolves, once per simulation step:
//   drag margins (the target may wander inside a dead zone) -> limits (the view never
//   shows outside them) -> smoothing (exponential approach at `smoothing_speed` per
//   second) -> shake. `offset` shifts the view after the limits are applied.
// The first current camera in creation order is the one the renderer uses; make_current()
// clears the flag on every other camera.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=camera2d, Id="7245e90e-d900-4976-b738-0f95caa1d15d") Camera2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("7245e90e-d900-4976-b738-0f95caa1d15d");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::cameras.add(this); }
    ~Camera2DComponent() override { world2d::cameras.remove(this); }
    world2d::Link<Camera2DComponent> link2d;

    bool current = true;
    Fixed zoom = 1.0;
    Fixed offset[2] = {0.0, 0.0};
    bool rotating = false;
    bool smoothing = false;
    Fixed smoothing_speed = 5.0;
    bool limits_enabled = false;
    // Left, top, right, bottom world pixels the view must stay inside.
    Fixed limits[4] = {-65536.0, -65536.0, 65536.0, 65536.0};
    bool drag_enabled = false;
    // Left, top, right, bottom fractions (0..1) of the half view the target may move freely.
    Fixed drag_margins[4] = {0.2, 0.2, 0.2, 0.2};
    // Clear colour used when no 3D camera paints the background.
    uint8_t background[3] = {33, 40, 52};

    // Resolved state, owned by the service.
    Fixed position[2] = {0.0, 0.0};
    Fixed rotation = 0.0;
    bool initialized = false;
    Fixed shake_amplitude = 0.0, shake_time = 0.0, shake_duration = 0.0;
    uint32_t shake_seed = 0x2545f491u;

    EPOK_FUNCTION(BlueprintCallable, Id="f5590c52-5ad4-46f9-bcff-be79109d29f0") void make_current() {
        world2d::cameras.each([&](Camera2DComponent& other) { other.current = &other == this; });
    }
    EPOK_FUNCTION(BlueprintPure, Id="9e6147c7-7048-4e17-813a-301e2588c5c9") bool is_current() const { return current; }
    EPOK_FUNCTION(BlueprintCallable, Id="c366f02f-7c45-463d-bdd9-08dbccf00c97") void set_zoom(Fixed value) {
        zoom = value < Fixed(0.0625) ? Fixed(0.0625) : value > Fixed(64.0) ? Fixed(64.0) : value;
    }
    EPOK_FUNCTION(BlueprintPure, Id="1effe962-5e26-476d-8235-84afd7cff4ad") Fixed get_zoom() const { return zoom; }
    EPOK_FUNCTION(BlueprintCallable, Id="5755f869-ab67-4550-a959-2247ae36c962") void set_offset(Fixed x, Fixed y) { offset[0] = x; offset[1] = y; }
    EPOK_FUNCTION(BlueprintCallable, Id="fa07a399-1b7c-41c0-a47b-1b817d51f740") void set_limits(Fixed left, Fixed top, Fixed right, Fixed bottom) {
        limits[0] = left; limits[1] = top; limits[2] = right; limits[3] = bottom;
        limits_enabled = true;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="b81982f6-1b48-445b-a77e-c9cf2aa7391b") void clear_limits() { limits_enabled = false; }
    // Shakes the view by up to `amplitude` pixels, fading out over `seconds`.
    EPOK_FUNCTION(BlueprintCallable, Id="d0f9124a-b803-4cf4-a7d1-cb6c95a75373") void shake(Fixed amplitude, Fixed seconds) {
        shake_amplitude = amplitude < 0.0 ? Fixed(0.0) : amplitude;
        shake_duration = shake_time = seconds < 0.0 ? Fixed(0.0) : seconds;
    }
    // Jump to the target this step instead of smoothing towards it.
    EPOK_FUNCTION(BlueprintCallable, Id="9078f738-489a-490e-8526-8c007c38b2c0") void reset_smoothing() { initialized = false; }
    EPOK_FUNCTION(BlueprintPure, Id="5d303d2d-985f-4e9e-ba7a-d78f3431ccf5") Fixed screen_center_x() const { return position[0] + offset[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="826cccdb-b798-440a-b07e-b8034090c85d") Fixed screen_center_y() const { return position[1] + offset[1]; }

    // One simulation step towards `target` (the owner's world position).
    void follow(const Fixed* target, Fixed dt) {
        const Fixed half[2] = {Fixed(display_width * 2048, Fixed::RAW) / zoom, Fixed(display_height * 2048, Fixed::RAW) / zoom};
        Fixed desired[2] = {target[0], target[1]};
        if (drag_enabled && initialized) {
            for (int k = 0; k < 2; ++k) {
                const Fixed low = position[k] - half[k] * drag_margins[k], high = position[k] + half[k] * drag_margins[k + 2];
                desired[k] = position[k];
                if (target[k] < low) desired[k] = position[k] - (low - target[k]);
                if (target[k] > high) desired[k] = position[k] + (target[k] - high);
            }
        }
        if (limits_enabled)
            for (int k = 0; k < 2; ++k) {
                const Fixed low = limits[k] + half[k], high = limits[k + 2] - half[k];
                if (low > high) desired[k] = (limits[k] + limits[k + 2]) / 2;
                else if (desired[k] < low) desired[k] = low;
                else if (desired[k] > high) desired[k] = high;
            }
        if (!initialized || !smoothing) {
            position[0] = desired[0];
            position[1] = desired[1];
            initialized = true;
        } else {
            Fixed weight = smoothing_speed * dt;
            if (weight > 1.0) weight = 1.0;
            for (int k = 0; k < 2; ++k) position[k] += (desired[k] - position[k]) * weight;
        }
        if (shake_time > 0.0) {
            shake_time -= dt;
            if (shake_time < 0.0) shake_time = 0.0;
        }
    }
    // The Camera2D the renderer projects through.
    Camera2D view() {
        Camera2D camera;
        camera.position[0] = position[0] + offset[0];
        camera.position[1] = position[1] + offset[1];
        if (shake_time > 0.0 && shake_duration > 0.0 && shake_amplitude > 0.0) {
            const Fixed strength = shake_amplitude * (shake_time / shake_duration);
            for (int k = 0; k < 2; ++k) {
                shake_seed = shake_seed * 1664525u + 1013904223u;
                const int32_t unit = int32_t((shake_seed >> 8) & 8191) - 4096;  // [-1, 1) in Q12
                camera.position[k] += strength * Fixed(unit, Fixed::RAW);
            }
        }
        camera.zoom = zoom;
        camera.rotation = rotating ? rotation : Fixed(0.0);
        return camera;
    }
};

// ---- Collider2D ------------------------------------------------------------------------
//
// One box or circle placed at the owner's world position plus `shape.offset` (the offset
// turns and scales with the owner; box extents scale but stay axis-aligned). Solid
// colliders block CharacterBody2D motion; triggers report overlaps to their actor's
// `overlap_2d` event and to its components' `trigger_event`.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=collider2d, Id="aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe") Collider2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::colliders.add(this); }
    ~Collider2DComponent() override { world2d::colliders.remove(this); }
    world2d::Link<Collider2DComponent> link2d;

    Collider2D shape;
    // Index in the collision world for the current step, or -1. Owned by the service.
    int16_t slot = -1;

    EPOK_FUNCTION(BlueprintCallable, Id="99fbe1d8-dc81-4ac3-bfef-1b5970b96cb1") void set_enabled(bool value) { shape.enabled = value; }
    EPOK_FUNCTION(BlueprintPure, Id="03ec8e24-8614-4ab3-b113-f02221c9ea0e") bool is_enabled() const { return shape.enabled; }
    EPOK_FUNCTION(BlueprintPure, Id="0fe7a436-bfe2-4216-911f-8332041e933a") bool is_trigger() const { return shape.trigger; }
    EPOK_FUNCTION(BlueprintCallable, Id="f9615887-a3c4-4337-b6eb-f561ec40da2e") void set_collision_layer(uint32_t value) { shape.layer = value; }
    EPOK_FUNCTION(BlueprintCallable, Id="27d8deb2-317a-493e-9bd9-6e22031cc37d") void set_collision_mask(uint32_t value) { shape.layer_mask = value; }
    EPOK_FUNCTION(BlueprintPure, Id="46a3be84-51da-4e25-9bfb-f46483b7ab17") uint32_t collision_layer() const { return shape.layer; }
    EPOK_FUNCTION(BlueprintPure, Id="e5ba9e63-c9e9-4695-bf13-d89f57be83ef") uint32_t collision_mask() const { return shape.layer_mask; }
    // Full box size in pixels (width, height), centred on the shape offset.
    EPOK_FUNCTION(BlueprintCallable, Id="2e974d3c-49a4-475e-86a3-0ab76fd933de") void set_box_size(Fixed width, Fixed height) {
        shape.shape = Collider2D::Box;
        shape.half_extents[0] = (width < 0.0 ? -width : width) / 2;
        shape.half_extents[1] = (height < 0.0 ? -height : height) / 2;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="cf11aec7-17ba-492d-934e-3c3d3b2c2cb9") void set_circle_radius(Fixed radius) {
        shape.shape = Collider2D::Circle;
        shape.radius = radius < 0.0 ? -radius : radius;
    }
};

// ---- CharacterBody2D -------------------------------------------------------------------
//
// The move_and_slide contract on the bounded World2D solver: `velocity` is in pixels
// per second, `up_direction` defaults to screen up (0, -1), and the body moves through its
// actor's first solid Collider2D. Motion is resolved one axis at a time against
// axis-aligned shapes, so a wall stops horizontal motion while gravity keeps applying.
// Floor snapping keeps a grounded body on the floor when it walks off a small step.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=body2d, Id="8a092515-6374-4792-985c-9ff9a967a82d") CharacterBody2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("8a092515-6374-4792-985c-9ff9a967a82d");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::bodies.add(this); }
    ~CharacterBody2DComponent() override { world2d::bodies.remove(this); }
    world2d::Link<CharacterBody2DComponent> link2d;

    Fixed velocity[2] = {0.0, 0.0};
    Fixed up_direction[2] = {0.0, -1.0};
    Fixed floor_max_angle = 45.0;
    Fixed floor_snap_length = 4.0;
    uint32_t collision_mask = 0xffffffffu;
    // Floating bodies (top-down games) treat every collision as a wall.
    bool floating = false;

    bool on_floor = false, on_wall = false, on_ceiling = false;
    Fixed last_motion[2] = {0.0, 0.0};
    // The velocity the last move_and_slide lost against what it hit, and the actor of the
    // wall it hit (characters push rigid bodies with them).
    Fixed blocked_velocity[2] = {0.0, 0.0};
    ObjectId wall_collider;
    ObjectId last_collider;
    // Cell of the last tile hit, when the last collision was a TileMapLayer.
    int32_t last_tile[2] = {0, 0};
    bool last_hit_tile = false;

    EPOK_FUNCTION(BlueprintCallable, Id="8cbd83fc-9005-4378-b295-17b7db2423c2") void set_velocity(Fixed x, Fixed y) { velocity[0] = x; velocity[1] = y; }
    EPOK_FUNCTION(BlueprintPure, Id="d6100b9e-dd13-4083-a4eb-f8e71657d540") Fixed velocity_x() const { return velocity[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="a9dd10c9-5330-432b-a1a8-e045f3143111") Fixed velocity_y() const { return velocity[1]; }
    EPOK_FUNCTION(BlueprintPure, Id="33d90de2-b12c-4a7d-8382-ef868816781d") bool is_on_floor() const { return on_floor; }
    EPOK_FUNCTION(BlueprintPure, Id="15c29f50-91d6-49ce-8e44-f3bdb4bc35ab") bool is_on_wall() const { return on_wall; }
    EPOK_FUNCTION(BlueprintPure, Id="14163a37-c1cc-460d-9248-2e03e8cc723e") bool is_on_ceiling() const { return on_ceiling; }
    EPOK_FUNCTION(BlueprintPure, Id="69face40-226b-4f78-adb9-ed49e2531e2f") ObjectId last_collision_actor() const { return last_collider; }
    EPOK_FUNCTION(BlueprintPure, Id="c7262c81-371a-43f0-91d5-ab34543035b1") Fixed last_motion_x() const { return last_motion[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="ea8cd1c9-186a-4add-98db-f2515356314c") Fixed last_motion_y() const { return last_motion[1]; }
    // Whether the last collision was a tile of a TileMapLayer (last_collision_actor is its actor).
    EPOK_FUNCTION(BlueprintPure, Id="2430b34e-f480-40ea-a461-33642fc4a076") bool last_collision_is_tile() const { return last_hit_tile; }
    // Cell column / row of that tile.
    EPOK_FUNCTION(BlueprintPure, Id="5901e1e6-c254-4a4f-a73d-82f57ef9758b") int32_t last_tile_x() const { return last_tile[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="319ef726-f36f-42fd-a4a9-a8b95aa208bb") int32_t last_tile_y() const { return last_tile[1]; }

    // Moves by velocity * step, slides along what it hits, zeroes the velocity components
    // pushing into the obstacle and refreshes the floor/wall/ceiling state. Returns whether
    // anything was hit. Call it from tick: it uses the simulation step.
    EPOK_FUNCTION(BlueprintCallable, Id="e8bdb3c0-41ed-42c9-9f8b-d994db924f9a") bool move_and_slide() {
        const Fixed dt = world2d::step_seconds();
        const Fixed delta[2] = {velocity[0] * dt, velocity[1] * dt};
        const bool was_on_floor = on_floor;
        MoveResult2D result;
        on_floor = on_wall = on_ceiling = false;
        last_motion[0] = last_motion[1] = 0.0;
        blocked_velocity[0] = blocked_velocity[1] = 0.0;
        if (!move(delta, result)) return false;
        bool hit = classify(result, true);
        // Snap: stay grounded across small drops while not moving away from the floor.
        const Fixed away = velocity[0] * up_direction[0] + velocity[1] * up_direction[1];
        if (!floating && was_on_floor && !on_floor && away <= 0.0 && floor_snap_length > 0.0) {
            const Fixed probe[2] = {-up_direction[0] * floor_snap_length, -up_direction[1] * floor_snap_length};
            MoveResult2D snap;
            if (move(probe, snap)) {
                if (classify(snap, false) && on_floor) hit = true;
                else {
                    const Fixed back[2] = {-snap.displacement[0], -snap.displacement[1]};
                    MoveResult2D undo;
                    move(back, undo);
                    last_motion[0] -= snap.displacement[0];
                    last_motion[1] -= snap.displacement[1];
                }
            }
        }
        return hit;
    }
    // Moves by an explicit displacement (pixels) and stops at the first obstacle.
    EPOK_FUNCTION(BlueprintCallable, Id="ea7c6ee9-4dca-4b45-b8dd-79607b2b38b0") bool move_and_collide(Fixed dx, Fixed dy) {
        const Fixed delta[2] = {dx, dy};
        MoveResult2D result;
        last_motion[0] = last_motion[1] = 0.0;
        return move(delta, result) && result.blocked;
    }

private:
    bool move(const Fixed* delta, MoveResult2D& result) {
        if (!world2d::hooks.move || !world2d::hooks.move(*this, delta, collision_mask, result)) return false;
        last_motion[0] += result.displacement[0];
        last_motion[1] += result.displacement[1];
        return true;
    }
    // Floor when the contact normal is within floor_max_angle of up_direction.
    bool classify(const MoveResult2D& result, bool stop_velocity) {
        bool hit = false;
        const Fixed limit = cos_degrees(floor_max_angle);
        for (int axis = 0; axis < 2; ++axis) {
            if (result.blocker[axis] < 0) continue;
            hit = true;
            if (world2d::hooks.hit_actor) last_collider = world2d::hooks.hit_actor(result.blocker[axis]);
            last_hit_tile = world2d::tile_cell_at && world2d::tile_cell_at(result.blocker[axis], last_tile[0], last_tile[1]);
            const Fixed n[2] = {axis == 0 ? result.normal[0] : Fixed(0.0), axis == 1 ? result.normal[1] : Fixed(0.0)};
            const Fixed facing = n[0] * up_direction[0] + n[1] * up_direction[1];
            if (!floating && facing >= limit) on_floor = true;
            else if (!floating && facing <= -limit) on_ceiling = true;
            else {
                on_wall = true;
                if (world2d::hooks.hit_actor) wall_collider = world2d::hooks.hit_actor(result.blocker[axis]);
            }
            if (stop_velocity && ((velocity[axis] > 0.0 && n[axis] < 0.0) || (velocity[axis] < 0.0 && n[axis] > 0.0))) {
                blocked_velocity[axis] = velocity[axis];
                velocity[axis] = 0.0;
            }
        }
        return hit;
    }
};
}  // namespace epok

// Tile maps, parallax, canvas layers and the content nodes build on the components above.
#include "world2d_tiles.hpp"
#include "world2d_content.hpp"
#include "world2d_skeleton.hpp"
#include "world2d_navigation.hpp"
#include "world2d_physics.hpp"
#include "world2d_viewports.hpp"
