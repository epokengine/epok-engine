#pragma once
// World2D cut-out skeletons (World2D initiative, phase 4).
//
// A Skeleton2D animates a list of bone actors: ordinary 2D actors attached to each other,
// each usually drawing one Sprite2D part (head, arm, leg). Bones are the actors' own root
// transforms, so the attachment hierarchy, the editor's move/rotate tools and every 2D
// component keep working on them. An animation is a list of poses at increasing times;
// each pose holds a rotation and a position for every bone, and the skeleton blends the
// two poses around the current time (rotations along the shorter arc).
//
// Up to two-bone IK chains run after the animation: they turn an upper and a lower bone
// so a tip actor (a hand, a foot) reaches a target actor, bending to the chosen side.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_content.hpp"

namespace epok {
class Skeleton2DComponent;

namespace world2d {
inline List<Skeleton2DComponent> skeletons;
inline constexpr size_t skeleton2d_bone_limit = 24;
inline constexpr size_t skeleton2d_ik_limit = 2;
inline constexpr int32_t degrees_turn_raw = 360 * 4096;

// An angle in raw Q12 degrees brought into (-180, 180].
inline int32_t wrap_degrees_raw(int32_t raw) {
    raw %= degrees_turn_raw;
    if (raw > degrees_turn_raw / 2) raw -= degrees_turn_raw;
    if (raw <= -degrees_turn_raw / 2) raw += degrees_turn_raw;
    return raw;
}
// The root transform of an actor, or null.
inline SceneComponent2D* actor_root(ObjectId actor) {
    auto* registry = active_object_registry;
    auto* owner = registry && actor.valid() ? registry->resolve<Actor>(actor) : nullptr;
    return owner ? registry->resolve<SceneComponent2D>(owner->root_id()) : nullptr;
}
inline bool root_world(const SceneComponent2D& root, Affine2D& out) {
    return world_matrix_2d(root, [](ObjectId id) { return resolve_scene2d(id); }, out);
}
}  // namespace world2d

// One animation: `key_count` poses at increasing `times` (seconds), each `bone_count`
// (rotation, x, y) triples in the skeleton's bone order. A looping animation blends its
// last pose back into the first across `length`.
struct SkeletonClip2D {
    const Fixed* times = nullptr;
    const Fixed* poses = nullptr;
    uint8_t key_count = 0;
    bool looping = true;
    Fixed length = 0.0;
};
// A two-bone IK chain: `upper` and `lower` index the skeleton's bones; `tip` is the actor
// at the end of the lower bone and `target` the actor it reaches for.
struct SkeletonIk2D {
    uint8_t upper = 0, lower = 0;
    ObjectId tip, target;
    bool bend_positive = true, enabled = true;
};

// ---- Skeleton2D ---------------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=skeleton2d, Id="b933c773-9256-4b9b-98ff-ef7900a02a9e") Skeleton2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("b933c773-9256-4b9b-98ff-ef7900a02a9e");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::skeletons.add(this); }
    ~Skeleton2DComponent() override { world2d::skeletons.remove(this); }
    world2d::Link<Skeleton2DComponent> link2d;

    // Cooked: bone actors, their rest pose (rotation, x, y), animations and IK chains.
    ObjectId bones[world2d::skeleton2d_bone_limit];
    const Fixed* rest = nullptr;
    uint8_t bone_count = 0;
    const SkeletonClip2D* clips = nullptr;
    uint8_t clip_count = 0;
    SkeletonIk2D ik[world2d::skeleton2d_ik_limit];
    uint8_t ik_count = 0;
    // Playback.
    int8_t clip = -1;
    bool playing = false, finished = false;
    Fixed time = 0.0, speed = 1.0;

    // Starts animation `index` from its beginning; an index out of range stops.
    EPOK_FUNCTION(BlueprintCallable, Id="b5c8094d-da41-4e39-b70e-d51267a57399") void play(int32_t index) {
        if (index < 0 || index >= clip_count) {
            stop();
            return;
        }
        clip = int8_t(index);
        time = 0.0;
        playing = true;
        finished = false;
        apply(time);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="fb5090b7-c9cb-4af5-83c7-dec3548f379d") void stop() { playing = false; }
    EPOK_FUNCTION(BlueprintPure, Id="160eb267-0dd8-45df-863a-6b44c229d0a1") bool is_playing() const { return playing; }
    EPOK_FUNCTION(BlueprintPure, Id="ad42b804-8e9e-41c6-92a7-03bfdf0a56a6") int32_t get_animation() const { return clip; }
    EPOK_FUNCTION(BlueprintPure, Id="f29033db-c3c1-43ff-b796-fa7465882254") int32_t get_animation_count() const { return clip_count; }
    EPOK_FUNCTION(BlueprintCallable, Id="331edf75-e1c7-4355-9a7f-e694825ea4c4") void set_speed(Fixed value) { speed = value; }
    EPOK_FUNCTION(BlueprintPure, Id="f1c57837-3327-4e92-8fc1-28fd8e8dfb29") Fixed get_speed() const { return speed; }
    // Jumps the current animation to `seconds` and poses the bones at once.
    EPOK_FUNCTION(BlueprintCallable, Id="e6b7948a-f0e4-405c-9c86-900510f54bb0") void seek(Fixed seconds) {
        if (clip < 0 || clip >= clip_count) return;
        time = wrap_time(seconds);
        apply(time);
    }
    EPOK_FUNCTION(BlueprintPure, Id="804ff1d0-9a04-4b77-811d-6032e18118e5") Fixed get_time() const { return time; }
    // True once after a non-looping animation reaches its end.
    EPOK_FUNCTION(BlueprintCallable, Id="3ac01d30-fc89-4f89-9553-b1bff9a41924") bool take_finished() {
        const bool was = finished;
        finished = false;
        return was;
    }
    // Stops and puts every bone back in its rest pose.
    EPOK_FUNCTION(BlueprintCallable, Id="98b91016-cd28-4495-a139-0b2d0f9244b7") void reset_pose() {
        playing = false;
        if (rest) pose(rest);
    }
    EPOK_FUNCTION(BlueprintPure, Id="d0f0718f-9ada-4b8c-a940-a4ffe5de9196") int32_t get_bone_count() const { return bone_count; }
    EPOK_FUNCTION(BlueprintPure, Id="7bdeae2a-fdc6-4736-a6d2-8a84db6e2dda") ObjectId get_bone(int32_t index) const {
        return index >= 0 && index < bone_count ? bones[index] : ObjectId{};
    }
    EPOK_FUNCTION(BlueprintCallable, Id="fd37af96-b662-45b3-afaa-573b7936ecf9") void set_ik_enabled(int32_t chain, bool value) {
        if (chain >= 0 && chain < ik_count) ik[chain].enabled = value;
    }
    EPOK_FUNCTION(BlueprintPure, Id="45066adc-29eb-4c38-b8e2-c51f9ba931bf") bool is_ik_enabled(int32_t chain) const {
        return chain >= 0 && chain < ik_count && ik[chain].enabled;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="04d94f51-8e1c-4f5a-8bc0-e3dda50ae3b1") void set_ik_target(int32_t chain, ObjectId actor) {
        if (chain >= 0 && chain < ik_count) ik[chain].target = actor;
    }

    // Called by the service every simulation step: advances the animation, poses the
    // bones, then solves the IK chains.
    void advance(Fixed dt) {
        if (playing && clip >= 0 && clip < clip_count) {
            const SkeletonClip2D& c = clips[clip];
            Fixed next = time + dt * speed;
            if (!c.looping && c.length.raw() > 0 && (next >= c.length || next.raw() < 0)) {
                next = next.raw() < 0 ? Fixed(0.0) : c.length;
                playing = false;
                finished = true;
            }
            time = wrap_time(next);
            apply(time);
        }
        for (uint8_t i = 0; i < ik_count; ++i) solve(ik[i]);
    }
    // Poses the bones at `seconds` of the current animation.
    void apply(Fixed seconds) {
        if (clip < 0 || clip >= clip_count) return;
        const SkeletonClip2D& c = clips[clip];
        if (!c.key_count || !c.times || !c.poses) return;
        const Fixed* first = c.poses;
        const Fixed* last = c.poses + size_t(c.key_count - 1) * bone_count * 3;
        if (c.key_count == 1) {
            pose(first);
            return;
        }
        uint8_t k = 0;
        while (k + 1 < c.key_count && c.times[k + 1] <= seconds) ++k;
        const Fixed* a;
        const Fixed* b;
        int32_t span, into;
        if (seconds < c.times[0]) {
            if (!c.looping) {
                pose(first);
                return;
            }
            // Before the first key of a loop: still blending out of the last one.
            a = last;
            b = first;
            span = c.length.raw() - c.times[c.key_count - 1].raw() + c.times[0].raw();
            into = c.length.raw() - c.times[c.key_count - 1].raw() + seconds.raw();
        } else if (k + 1 >= c.key_count) {
            if (!c.looping) {
                pose(last);
                return;
            }
            a = last;
            b = first;
            span = c.length.raw() - c.times[k].raw() + c.times[0].raw();
            into = seconds.raw() - c.times[k].raw();
        } else {
            a = c.poses + size_t(k) * bone_count * 3;
            b = a + size_t(bone_count) * 3;
            span = c.times[k + 1].raw() - c.times[k].raw();
            into = seconds.raw() - c.times[k].raw();
        }
        const int32_t f = span > 0 ? int32_t(int64_t(into < 0 ? 0 : into > span ? span : into) * 4096 / span) : 0;
        Fixed blended[3];
        for (uint8_t i = 0; i < bone_count; ++i) {
            const Fixed* pa = a + size_t(i) * 3;
            const Fixed* pb = b + size_t(i) * 3;
            const int32_t turn = world2d::wrap_degrees_raw(pb[0].raw() - pa[0].raw());
            blended[0] = Fixed(world2d::wrap_degrees_raw(pa[0].raw() + int32_t(int64_t(turn) * f / 4096)), Fixed::RAW);
            for (int axis = 1; axis < 3; ++axis)
                blended[axis] = Fixed(pa[axis].raw() + int32_t(int64_t(pb[axis].raw() - pa[axis].raw()) * f / 4096), Fixed::RAW);
            set_bone(i, blended);
        }
    }

private:
    Fixed wrap_time(Fixed seconds) const {
        const SkeletonClip2D& c = clips[clip];
        if (c.length.raw() <= 0) return Fixed(0.0);
        if (!c.looping) return seconds.raw() < 0 ? Fixed(0.0) : seconds > c.length ? c.length : seconds;
        int32_t raw = seconds.raw() % c.length.raw();
        if (raw < 0) raw += c.length.raw();
        return Fixed(raw, Fixed::RAW);
    }
    void set_bone(uint8_t index, const Fixed* value) {
        auto* root = world2d::actor_root(bones[index]);
        if (!root) return;
        root->transform.rotation = value[0];
        root->transform.position[0] = value[1];
        root->transform.position[1] = value[2];
        transform2d_clamp(root->transform);
    }
    void pose(const Fixed* values) {
        for (uint8_t i = 0; i < bone_count; ++i) set_bone(i, values + size_t(i) * 3);
    }
    static Fixed distance(const Fixed* a, const Fixed* b) { return world2d::length_2d(b[0] - a[0], b[1] - a[1]); }
    // Scales are always positive (mirroring is a sprite flip), so turning a bone by an
    // angle turns its world direction by the same angle.
    static void turn(SceneComponent2D& root, int32_t delta_raw) {
        root.transform.rotation = Fixed(world2d::wrap_degrees_raw(root.transform.rotation.raw() + delta_raw), Fixed::RAW);
    }
    void solve(const SkeletonIk2D& chain) {
        if (!chain.enabled || chain.upper >= bone_count || chain.lower >= bone_count) return;
        auto* upper = world2d::actor_root(bones[chain.upper]);
        auto* lower = world2d::actor_root(bones[chain.lower]);
        auto* tip = world2d::actor_root(chain.tip);
        auto* target = world2d::actor_root(chain.target);
        if (!upper || !lower || !tip || !target) return;
        Affine2D wu, wl, wt, wg;
        if (!world2d::root_world(*upper, wu) || !world2d::root_world(*lower, wl) || !world2d::root_world(*tip, wt) ||
            !world2d::root_world(*target, wg))
            return;
        const int64_t a = distance(wu.t, wl.t).raw(), b = distance(wl.t, wt.t).raw();
        if (a <= 0 || b <= 0) return;
        int64_t d = distance(wu.t, wg.t).raw();
        // Out of reach: the chain points straight at the target; too close: fully folded.
        const int64_t far = a + b - 1, near = (a > b ? a - b : b - a) + 1;
        if (d > far) d = far;
        if (d < near) d = near;
        // Law of cosines for the angle at the upper joint, in Q12.
        int64_t cosine = (a * a + d * d - b * b) * 4096 / (2 * a * d);
        if (cosine > 4096) cosine = 4096;
        if (cosine < -4096) cosine = -4096;
        const int64_t sine = world2d::isqrt64(int64_t(4096) * 4096 - cosine * cosine);
        const int32_t opening = world2d::angle_degrees(Fixed(int32_t(cosine), Fixed::RAW), Fixed(int32_t(sine), Fixed::RAW)).raw();
        const int32_t base = world2d::angle_degrees(wg.t[0] - wu.t[0], wg.t[1] - wu.t[1]).raw();
        const int32_t wanted = base + (chain.bend_positive ? opening : -opening);
        const int32_t current = world2d::angle_degrees(wl.t[0] - wu.t[0], wl.t[1] - wu.t[1]).raw();
        turn(*upper, world2d::wrap_degrees_raw(wanted - current));
        transform2d_clamp(upper->transform);
        if (!world2d::root_world(*lower, wl) || !world2d::root_world(*tip, wt)) return;
        const int32_t reach = world2d::angle_degrees(wg.t[0] - wl.t[0], wg.t[1] - wl.t[1]).raw();
        const int32_t along = world2d::angle_degrees(wt.t[0] - wl.t[0], wt.t[1] - wl.t[1]).raw();
        turn(*lower, world2d::wrap_degrees_raw(reach - along));
        transform2d_clamp(lower->transform);
    }
};
}  // namespace epok
