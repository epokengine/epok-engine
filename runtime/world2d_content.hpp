#pragma once
// World2D content nodes (World2D initiative, phase 3): Path2D and PathFollow2D, Line2D,
// Polygon2D, Particles2D, AudioPlayer2D, VisibleOnScreenNotifier2D and RemoteTransform2D.
//
// Every shape is in its owner's local pixels and follows the owner's world transform.
// Cooked shapes (path samples, polygon triangles) are constant; Line2D keeps a small
// editable point list so gameplay can bend ropes and aim lasers. The World2D service
// advances followers, particles, notifiers, remote transforms and 2D audio once per
// simulation step; the renderer expands lines, polygons and particles at draw time.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_tiles.hpp"

namespace epok {
inline constexpr size_t line2d_follow_limit = 16;
class Path2DComponent;
class PathFollow2DComponent;
class Line2DComponent;
class Polygon2DComponent;
class Particles2DComponent;
class AudioPlayer2DComponent;
class VisibleOnScreenNotifier2DComponent;
class RemoteTransform2DComponent;

namespace world2d {
inline List<Path2DComponent> paths;
inline List<PathFollow2DComponent> followers;
inline List<Line2DComponent> lines;
inline List<Polygon2DComponent> polygons;
inline List<Particles2DComponent> emitters;
inline List<AudioPlayer2DComponent> audio_players;
inline List<VisibleOnScreenNotifier2DComponent> notifiers;
inline List<RemoteTransform2DComponent> remotes;

// Particles live in the service's pool; an emitter releases its own when it goes away.
inline void (*release_particles)(Particles2DComponent&) = nullptr;

// Integer square root of a non-negative 64-bit value.
inline int64_t isqrt64(int64_t value) {
    if (value <= 0) return 0;
    int64_t root = 0;
    for (int64_t bit = int64_t(1) << 62; bit; bit >>= 2) {
        if (value >= root + bit) {
            value -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
    }
    return root;
}
// Angle of a vector in Q12 degrees, from +X towards +Y (clockwise on screen), in
// (-180, 180]. A rational approximation, within a third of a degree.
inline Fixed angle_degrees(Fixed x, Fixed y) {
    const int32_t raw_x = x.raw(), raw_y = y.raw();
    const int32_t ax = raw_x < 0 ? -raw_x : raw_x, ay = raw_y < 0 ? -raw_y : raw_y;
    const int32_t large = ax > ay ? ax : ay, small = ax > ay ? ay : ax;
    if (!large) return Fixed(0, Fixed::RAW);
    const int32_t ratio = int32_t(int64_t(small) * 4096 / large);
    // Degrees of atan(ratio) for ratio in [0, 1].
    int32_t angle = int32_t(int64_t(ratio) * (45 * 4096 + 16 * (4096 - ratio)) / 4096);
    if (ay > ax) angle = 90 * 4096 - angle;
    if (raw_x < 0) angle = 180 * 4096 - angle;
    if (raw_y < 0) angle = -angle;
    return Fixed(angle, Fixed::RAW);
}
// Length of a Q12 vector, in Q12.
inline Fixed length_2d(Fixed x, Fixed y) {
    return Fixed(int32_t(isqrt64(int64_t(x.raw()) * x.raw() + int64_t(y.raw()) * y.raw())), Fixed::RAW);
}
// Moves the owner's root so its world position becomes `world`, through the inverse of
// the parent chain's linear part when the root is attached.
inline bool set_world_position(SceneComponent2D& root, const Fixed* world) {
    Fixed local[2] = {world[0], world[1]};
    if (root.attach_parent.valid()) {
        const SceneComponent2D* parent = resolve_scene2d(root.attach_parent);
        Affine2D matrix;
        if (parent && world_matrix_2d(*parent, [](ObjectId id) { return resolve_scene2d(id); }, matrix)) {
            const int64_t a = matrix.m[0][0].raw(), b = matrix.m[0][1].raw(), c = matrix.m[1][0].raw(), d = matrix.m[1][1].raw();
            const int64_t determinant = (a * d - b * c) / 4096;
            if (!determinant) return false;
            const int64_t x = int64_t(world[0].raw()) - matrix.t[0].raw(), y = int64_t(world[1].raw()) - matrix.t[1].raw();
            local[0] = detail::clamp_raw_2d((d * x - b * y) / determinant);
            local[1] = detail::clamp_raw_2d((a * y - c * x) / determinant);
        }
    }
    root.transform.position[0] = local[0];
    root.transform.position[1] = local[1];
    transform2d_clamp(root.transform);
    return true;
}
}  // namespace world2d

// ---- Path2D -------------------------------------------------------------------------------
//
// A polyline (curves are sampled by the editor) with its cumulative lengths, in the
// owner's local pixels. Distances along the path are in those same pixels.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=path2d, Id="fb44120d-2561-4ac3-943a-333fef57fcd0") Path2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("fb44120d-2561-4ac3-943a-333fef57fcd0");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::paths.add(this); }
    ~Path2DComponent() override { world2d::paths.remove(this); }
    world2d::Link<Path2DComponent> link2d;

    const Fixed* points = nullptr;   // x, y pairs
    const Fixed* lengths = nullptr;  // distance from the first point to each point
    uint16_t point_count = 0;
    bool closed = false;

    EPOK_FUNCTION(BlueprintPure, Id="2f2ee36c-abdf-4c57-a16d-b39935e8d5ca") Fixed get_length() const {
        return point_count && lengths ? lengths[point_count - 1] : Fixed(0.0);
    }
    // The local point and unit direction at `distance` along the path, clamped (or
    // wrapped, for a closed path) to its length.
    bool sample_local(Fixed distance, Fixed* point, Fixed* direction) const {
        if (!points || !lengths || !point_count) return false;
        const Fixed total = get_length();
        if (point_count == 1 || total.raw() <= 0) {
            point[0] = points[0];
            point[1] = points[1];
            direction[0] = 1.0;
            direction[1] = 0.0;
            return true;
        }
        if (closed) {
            int32_t d = distance.raw() % total.raw();
            if (d < 0) d += total.raw();
            distance = Fixed(d, Fixed::RAW);
        } else if (distance < 0.0) {
            distance = 0.0;
        } else if (distance > total) {
            distance = total;
        }
        uint16_t low = 0, high = uint16_t(point_count - 1);
        while (high - low > 1) {
            const uint16_t mid = uint16_t((low + high) / 2);
            if (lengths[mid] <= distance) low = mid;
            else high = mid;
        }
        const Fixed span = lengths[high] - lengths[low];
        const Fixed* a = points + 2 * low;
        const Fixed* b = points + 2 * high;
        const Fixed t = span.raw() > 0 ? (distance - lengths[low]) / span : Fixed(0.0);
        point[0] = a[0] + detail::mul_2d(b[0] - a[0], t);
        point[1] = a[1] + detail::mul_2d(b[1] - a[1], t);
        const Fixed dx = b[0] - a[0], dy = b[1] - a[1];
        const Fixed length = world2d::length_2d(dx, dy);
        direction[0] = length.raw() ? dx / length : Fixed(1.0);
        direction[1] = length.raw() ? dy / length : Fixed(0.0);
        return true;
    }
    // World position at `distance` along the path.
    EPOK_FUNCTION(BlueprintPure, Id="3b8d4b7f-7cf1-432a-9733-608e71e15c35") Fixed sample_x(Fixed distance) const { return sample_world(distance, 0); }
    EPOK_FUNCTION(BlueprintPure, Id="c0734e40-6c31-46c9-8769-d548bb36b172") Fixed sample_y(Fixed distance) const { return sample_world(distance, 1); }

private:
    Fixed sample_world(Fixed distance, int axis) const {
        Fixed local[2], direction[2], world[2];
        Affine2D matrix;
        if (!sample_local(distance, local, direction)) return 0.0;
        if (!world2d::world_of(const_cast<Path2DComponent&>(*this), matrix)) return local[axis];
        matrix.point(local, world);
        return world[axis];
    }
};

// ---- PathFollow2D -------------------------------------------------------------------------
//
// Moves its actor along a Path2D: the path of `path` (another actor), or of the actor it
// is attached to. `progress` is the distance travelled in pixels; `speed` advances it every
// simulation step. Bodies standing on a moving actor's solid collider ride along.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=path2d, Id="2977d643-f30d-43c1-a6cc-15dc8cddf158") PathFollow2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("2977d643-f30d-43c1-a6cc-15dc8cddf158");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::followers.add(this); }
    ~PathFollow2DComponent() override { world2d::followers.remove(this); }
    world2d::Link<PathFollow2DComponent> link2d;

    ObjectId path;
    Fixed progress = 0.0, speed = 0.0;
    Fixed offset[2] = {0.0, 0.0};  // along / across the path
    bool looping = true;           // wrap at the ends (always, on a closed path)
    bool ping_pong = false;        // or turn back at the ends
    bool rotates = false;          // turn the actor to the path's direction
    bool moving = true;

    EPOK_FUNCTION(BlueprintCallable, Id="f88385ab-1b01-4b64-bd86-ddd822f2cef7") void set_progress(Fixed value) { progress = value; }
    EPOK_FUNCTION(BlueprintPure, Id="f95f945d-6164-4bbc-8263-1d4f2d408125") Fixed get_progress() const { return progress; }
    // Progress as a fraction (0..1) of the path's length.
    EPOK_FUNCTION(BlueprintCallable, Id="613a8591-954b-4081-ab15-cc4940ad6afd") void set_progress_ratio(Fixed value) {
        if (auto* p = resolve_path()) progress = detail::mul_2d(value, p->get_length());
    }
    EPOK_FUNCTION(BlueprintPure, Id="8ff65ae5-c640-4055-ad6a-01b7466a3ecc") Fixed get_progress_ratio() const {
        auto* p = resolve_path();
        const Fixed total = p ? p->get_length() : Fixed(0.0);
        return total.raw() > 0 ? progress / total : Fixed(0.0);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="ada216fd-ae30-41e4-aaf7-7a66a5b18df3") void set_speed(Fixed value) { speed = value; }
    EPOK_FUNCTION(BlueprintPure, Id="9fa3b4ea-072b-45b5-8948-48fb983c4e63") Fixed get_speed() const { return speed; }
    EPOK_FUNCTION(BlueprintCallable, Id="ead6cee2-ea3e-43d2-836c-2fd37c37ddfc") void set_moving(bool value) { moving = value; }
    EPOK_FUNCTION(BlueprintPure, Id="738cac73-c907-41be-bcdc-0ca789325858") bool is_moving() const { return moving; }

    // The Path2D this follower uses.
    Path2DComponent* resolve_path() const {
        auto* registry = active_object_registry;
        if (!registry) return nullptr;
        Actor* target = path.valid() ? registry->resolve<Actor>(path) : nullptr;
        if (!target) {
            auto* root = world2d::root_of(const_cast<PathFollow2DComponent&>(*this));
            const SceneComponent2D* parent = root && root->attach_parent.valid() ? world2d::resolve_scene2d(root->attach_parent) : nullptr;
            target = parent ? const_cast<SceneComponent2D*>(parent)->get_owner() : nullptr;
        }
        return target ? world2d::find<Path2DComponent>(target) : nullptr;
    }
    // One simulation step: advance, then place the actor. Writes the world displacement
    // of the actor into `moved`. False when there is no path.
    bool advance(Fixed dt, Fixed* moved) {
        moved[0] = moved[1] = 0.0;
        auto* p = resolve_path();
        auto* root = world2d::root_of(*this);
        if (!p || !root) return false;
        const Fixed total = p->get_length();
        if (moving && speed.raw()) {
            progress += speed * dt;
            if (total.raw() > 0 && !p->closed) {
                if (ping_pong) {
                    if (progress > total) { progress = total - (progress - total); speed = -speed; }
                    if (progress < 0.0) { progress = -progress; speed = -speed; }
                } else if (looping) {
                    int32_t d = progress.raw() % total.raw();
                    if (d < 0) d += total.raw();
                    progress = Fixed(d, Fixed::RAW);
                } else if (progress > total) {
                    progress = total;
                } else if (progress < 0.0) {
                    progress = 0.0;
                }
            }
        }
        Fixed local[2], direction[2];
        if (!p->sample_local(progress + offset[0], local, direction)) return false;
        // Across the path: the direction turned a quarter towards +Y (screen down).
        local[0] -= detail::mul_2d(direction[1], offset[1]);
        local[1] += detail::mul_2d(direction[0], offset[1]);
        Affine2D matrix;
        Fixed world[2] = {local[0], local[1]};
        if (world2d::world_of(*p, matrix)) matrix.point(local, world);
        Affine2D before;
        Fixed start[2] = {root->transform.position[0], root->transform.position[1]};
        if (world2d::world_of(*this, before)) { start[0] = before.t[0]; start[1] = before.t[1]; }
        if (!world2d::set_world_position(*root, world)) return false;
        if (rotates) root->transform.rotation = world2d::angle_degrees(direction[0], direction[1]);
        moved[0] = world[0] - start[0];
        moved[1] = world[1] - start[1];
        return true;
    }
};

// ---- Line2D -------------------------------------------------------------------------------
//
// A polyline of up to `line2d_max_points` points drawn `width` pixels wide, its colour
// blending from `begin_color` to `end_color`. Gameplay may move, add and clear points.
inline constexpr uint16_t line2d_max_points = 32;
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=line2d, Id="22d9a692-8fa0-4009-b7e3-a2d73755cd9a") Line2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("22d9a692-8fa0-4009-b7e3-a2d73755cd9a");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::lines.add(this); }
    ~Line2DComponent() override { world2d::lines.remove(this); }
    world2d::Link<Line2DComponent> link2d;

    Fixed points[line2d_max_points][2] = {};
    uint16_t point_count = 0;
    // Nodes the line runs through, in order: the service moves the points onto them
    // every step (ropes, chains, beams between moving things).
    ObjectId follow[line2d_follow_limit];
    uint8_t follow_count = 0;
    Fixed width = 2.0;
    uint8_t begin_color[3] = {255, 255, 255}, end_color[3] = {255, 255, 255};
    BlendMode blend = BlendMode::Cutout;
    int16_t z_index = 0;
    bool closed = false, visible = true;

    EPOK_FUNCTION(BlueprintPure, Id="10a74921-30e0-4dd1-b12a-0bf4749c67a9") uint32_t get_point_count() const { return point_count; }
    EPOK_FUNCTION(BlueprintCallable, Id="d3056d1d-264f-46d1-8896-bf7b242633d9") bool add_point(Fixed x, Fixed y) {
        if (point_count >= line2d_max_points) return false;
        points[point_count][0] = x;
        points[point_count][1] = y;
        ++point_count;
        return true;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="73dd9878-41f3-435f-a68e-8af9719c3b78") void set_point(uint32_t index, Fixed x, Fixed y) {
        if (index < point_count) { points[index][0] = x; points[index][1] = y; }
    }
    EPOK_FUNCTION(BlueprintPure, Id="120c891f-a821-45a1-84fe-f7ac515bd890") Fixed get_point_x(uint32_t index) const { return index < point_count ? points[index][0] : Fixed(0.0); }
    EPOK_FUNCTION(BlueprintPure, Id="d263e68e-1a43-4f33-b385-008c28bf25ae") Fixed get_point_y(uint32_t index) const { return index < point_count ? points[index][1] : Fixed(0.0); }
    EPOK_FUNCTION(BlueprintCallable, Id="e024b382-fce7-4e26-9a3f-f36bf814a082") void remove_point(uint32_t index) {
        if (index >= point_count) return;
        for (uint32_t i = index; i + 1 < point_count; ++i) { points[i][0] = points[i + 1][0]; points[i][1] = points[i + 1][1]; }
        --point_count;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="5934f674-c2a8-4074-b21c-c6b9c257685f") void clear_points() { point_count = 0; }
    // Stops following nodes: the points stay where they are and are the line's own again.
    EPOK_FUNCTION(BlueprintCallable, Id="15b0ab21-b5e2-42ae-bdac-84e1a53c73a0") void stop_following() { follow_count = 0; }
    // Moves the points onto the followed nodes. Called by the service every step.
    void follow_nodes() {
        if (!follow_count) return;
        Affine2D matrix;
        if (!world2d::world_of(*this, matrix)) return;
        // Inverse of the owner's world transform, to bring the nodes into local pixels.
        const int64_t a = matrix.m[0][0].raw(), b = matrix.m[0][1].raw(), c = matrix.m[1][0].raw(), d = matrix.m[1][1].raw();
        const int64_t determinant = (a * d - b * c) / 4096;
        if (!determinant) return;
        uint16_t count = 0;
        for (uint8_t i = 0; i < follow_count && count < line2d_max_points; ++i) {
            auto* registry = active_object_registry;
            auto* actor = registry && follow[i].valid() ? registry->resolve<Actor>(follow[i]) : nullptr;
            auto* root = actor ? registry->resolve<SceneComponent2D>(actor->root_id()) : nullptr;
            Affine2D there;
            if (!root || !world_matrix_2d(*root, [](ObjectId id) { return world2d::resolve_scene2d(id); }, there)) continue;
            const int64_t x = int64_t(there.t[0].raw()) - matrix.t[0].raw(), y = int64_t(there.t[1].raw()) - matrix.t[1].raw();
            points[count][0] = detail::clamp_raw_2d((d * x - b * y) / determinant);
            points[count][1] = detail::clamp_raw_2d((a * y - c * x) / determinant);
            ++count;
        }
        point_count = count;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="5fa86eb7-f3aa-4922-9303-a2394a2d7943") void set_width(Fixed value) { width = value < 0.0 ? Fixed(0.0) : value; }
    EPOK_FUNCTION(BlueprintCallable, Id="007cb579-204e-449f-93c2-64888ce4d1c7") void set_colors(uint32_t red, uint32_t green, uint32_t blue, uint32_t end_red, uint32_t end_green, uint32_t end_blue) {
        const uint32_t values[6] = {red, green, blue, end_red, end_green, end_blue};
        for (int k = 0; k < 3; ++k) {
            begin_color[k] = uint8_t(values[k] > 255 ? 255 : values[k]);
            end_color[k] = uint8_t(values[k + 3] > 255 ? 255 : values[k + 3]);
        }
    }
    EPOK_FUNCTION(BlueprintCallable, Id="3d0e5363-4aa1-4e68-8ea1-218f4b7a8ad8") void set_visible(bool value) { visible = value; }
};

// ---- Polygon2D ----------------------------------------------------------------------------
//
// A filled polygon, triangulated by the editor: `color` alone, or a texture mapped by the
// cooked per-vertex texels and multiplied by `color`.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=polygon2d, Id="4fd4cdc4-46bb-4707-9328-40c8881c97b7") Polygon2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("4fd4cdc4-46bb-4707-9328-40c8881c97b7");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::polygons.add(this); }
    ~Polygon2DComponent() override { world2d::polygons.remove(this); }
    world2d::Link<Polygon2DComponent> link2d;

    const Fixed* vertices = nullptr;      // x, y pairs
    const uint16_t* texels = nullptr;     // u, v pairs, when textured
    const uint16_t* triangles = nullptr;  // three vertex indices each
    uint16_t vertex_count = 0, triangle_count = 0;
    int32_t texture = -1;
    uint8_t color[3] = {255, 255, 255};
    BlendMode blend = BlendMode::Cutout;
    int16_t z_index = 0;
    bool visible = true;

    EPOK_FUNCTION(BlueprintCallable, Id="f287161d-6852-4955-b685-36cfa507fdb7") void set_color(uint32_t red, uint32_t green, uint32_t blue) {
        color[0] = uint8_t(red > 255 ? 255 : red);
        color[1] = uint8_t(green > 255 ? 255 : green);
        color[2] = uint8_t(blue > 255 ? 255 : blue);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="d9c61596-eec3-419f-b831-64c401a49f69") void set_visible(bool value) { visible = value; }
};

// ---- Particles2D --------------------------------------------------------------------------
//
// A CPU particle emitter: `amount` particles live for `lifetime` seconds, spawned evenly
// (explosiveness 0) or all at once (1) from a point, rectangle or circle, launched in
// `direction` (degrees, 0 = right, 90 = down) within `spread` at a random speed between
// `speed[0]` and `speed[1]`, pulled by `gravity`, slowed by `damping`, scaled and tinted
// from start to end. Each particle draws `region` of `texture` (or one cell of a sheet).
enum class EmissionShape2D : uint8_t { Point, Rectangle, Circle };
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=particles2d, Id="79892328-8fd3-43c8-a03b-cfa57b483c64") Particles2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("79892328-8fd3-43c8-a03b-cfa57b483c64");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::emitters.add(this); }
    ~Particles2DComponent() override {
        world2d::emitters.remove(this);
        if (world2d::release_particles) world2d::release_particles(*this);
    }
    world2d::Link<Particles2DComponent> link2d;

    bool emitting = true, one_shot = false, local_coords = false, visible = true;
    uint16_t amount = 16;
    Fixed lifetime = 1.0, explosiveness = 0.0;
    Fixed direction = -90.0, spread = 30.0;
    Fixed speed[2] = {40.0, 80.0};
    Fixed gravity[2] = {0.0, 98.0};
    Fixed damping = 0.0;
    Fixed scale[2] = {1.0, 1.0};  // start, end
    uint8_t start_color[3] = {255, 255, 255}, end_color[3] = {255, 255, 255};
    EmissionShape2D shape = EmissionShape2D::Point;
    Fixed extents[2] = {8.0, 8.0};  // half size of the rectangle; [0] is the circle radius
    int32_t texture = -1;
    uint16_t region[4] = {0, 0, 0, 0};
    uint16_t hframes = 1;           // frames across `region`, played over the lifetime
    BlendMode blend = BlendMode::Add;
    int16_t z_index = 0;

    // Runtime state, owned by the service.
    Fixed accumulator = 0.0;
    uint16_t emitted = 0, alive = 0;
    uint32_t seed = 0x9e3779b9u;
    bool finished = false;

    EPOK_FUNCTION(BlueprintCallable, Id="c065c5a1-a1cc-42cb-aaf2-4d83d0df397c") void set_emitting(bool value) {
        if (value && !emitting) { emitted = 0; finished = false; accumulator = 0.0; }
        emitting = value;
    }
    EPOK_FUNCTION(BlueprintPure, Id="913b7bae-dceb-4f58-865d-d548e7f35a82") bool is_emitting() const { return emitting; }
    // Starts a one-shot burst again (or restarts a stream).
    EPOK_FUNCTION(BlueprintCallable, Id="47ceca40-1328-4a97-98be-331c3d1d18f8") void restart() {
        emitted = 0;
        finished = false;
        accumulator = 0.0;
        emitting = true;
    }
    EPOK_FUNCTION(BlueprintPure, Id="e48d5ad4-cb2b-48eb-8a7c-554ce7119f14") uint32_t get_alive_count() const { return alive; }
    // True once after a one-shot burst's last particle died.
    EPOK_FUNCTION(BlueprintCallable, Id="fe088a1c-26b4-4506-a6d6-7129f97379d0") bool take_finished() {
        const bool value = finished;
        finished = false;
        return value;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="6318135f-4451-47ce-95dc-dbc76da88e4d") void set_visible(bool value) { visible = value; }
};

// ---- AudioPlayer2D ------------------------------------------------------------------------
//
// Plays a clip whose loudness and stereo position follow the current camera: full volume
// at the centre of the view, fading to silence at `max_distance` pixels, panned by how far
// left or right of the view's centre it is.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=audio, Id="41b14829-fc1d-429b-abe4-f49c68592e7c") AudioPlayer2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("41b14829-fc1d-429b-abe4-f49c68592e7c");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::audio_players.add(this); }
    ~AudioPlayer2DComponent() override { world2d::audio_players.remove(this); }
    world2d::Link<AudioPlayer2DComponent> link2d;

    AudioSource source;
    Fixed volume = 1.0;          // before distance attenuation
    Fixed max_distance = 400.0;  // pixels
    Fixed attenuation = 1.0;     // exponent of the fade: 1 linear, 2 faster near the edge
    Fixed panning = 1.0;         // 0 keeps the clip centred, 1 pans fully at the screen edge
    bool autoplay = false;

    EPOK_FUNCTION(BlueprintCallable, Id="dcaba641-5682-40e9-a409-9578210df9fb") void play() { source.enabled = true; source.play(); }
    EPOK_FUNCTION(BlueprintCallable, Id="2b4cc2ab-5cd5-4bbf-9262-75549b04ce20") void stop() { source.stop(); }
    EPOK_FUNCTION(BlueprintPure, Id="9e7fb5b8-0465-481f-b858-0cdbb6fd0a77") bool is_playing() const { return source.is_playing(); }
    EPOK_FUNCTION(BlueprintCallable, Id="85e93796-94e3-4319-9282-71b7a5f54133") void set_volume(Fixed value) { volume = value < 0.0 ? Fixed(0.0) : value > 1.0 ? Fixed(1.0) : value; }
    EPOK_FUNCTION(BlueprintCallable, Id="2af5b672-d742-40ea-bc7a-de927f95a047") void set_pitch(Fixed value) { source.pitch = value < 0.25 ? Fixed(0.25) : value > 4.0 ? Fixed(4.0) : value; }
    EPOK_FUNCTION(BlueprintCallable, Id="5ac52e75-1687-4003-9eb0-76c0079caa19") void set_clip(int32_t value) {
        if (source.clip != value) { source.stop(); source.clip = value; }
    }
    void begin_play() override {
        if (autoplay && source.enabled && is_active(this) && !source.is_playing()) source.play();
    }
    void on_disable() override { if (source.is_playing()) source.stop(); }
    void end_play(EndPlayReason) override { if (source.is_playing()) source.stop(); }
    bool releasable() const override { return !audio_source_retained || !audio_source_retained(&source); }
    // Loudness and pan for a listener at `listener` (world pixels).
    void spatialize(const Fixed* here, const Fixed* listener, Fixed half_width) {
        const Fixed dx = here[0] - listener[0], dy = here[1] - listener[1];
        Fixed gain = 1.0;
        if (max_distance.raw() > 0) {
            const Fixed distance = world2d::length_2d(dx, dy);
            Fixed fall = distance >= max_distance ? Fixed(0.0) : Fixed(1.0) - distance / max_distance;
            // Integer part of the exponent: fall^attenuation (1, 2, 3...).
            Fixed shaped = fall;
            for (int k = 2; k <= 4 && Fixed(int32_t(k * 4096), Fixed::RAW) <= attenuation; ++k) shaped = detail::mul_2d(shaped, fall);
            gain = shaped;
        }
        source.volume = detail::mul_2d(volume, gain);
        Fixed pan = half_width.raw() > 0 ? detail::mul_2d(dx / half_width, panning) : Fixed(0.0);
        if (pan > 1.0) pan = 1.0;
        if (pan < -1.0) pan = -1.0;
        source.pan = int8_t(pan.raw() * 127 / 4096);
    }
};

// ---- VisibleOnScreenNotifier2D ------------------------------------------------------------
//
// Tracks whether `rect` (owner-local x, y, width, height) is inside the current view.
// Gameplay polls `is_on_screen` or the one-shot `take_screen_entered` / `take_screen_exited`.
// With `disable_when_hidden`, the owner is deactivated off screen and activated again when
// it returns: enemies that only run near the player.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=notifier2d, Id="8dd738bf-e8e5-4064-afb7-9700388cff08") VisibleOnScreenNotifier2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("8dd738bf-e8e5-4064-afb7-9700388cff08");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::notifiers.add(this); }
    ~VisibleOnScreenNotifier2DComponent() override { world2d::notifiers.remove(this); }
    world2d::Link<VisibleOnScreenNotifier2DComponent> link2d;

    Fixed rect[4] = {-10.0, -10.0, 20.0, 20.0};
    bool disable_when_hidden = false;
    bool on_screen = false, known = false, entered = false, exited = false;

    EPOK_FUNCTION(BlueprintPure, Id="b0e11307-3127-497b-8549-28975991e211") bool is_on_screen() const { return on_screen; }
    EPOK_FUNCTION(BlueprintCallable, Id="d55bce03-50fb-4398-bb02-b591ead4841d") bool take_screen_entered() { const bool v = entered; entered = false; return v; }
    EPOK_FUNCTION(BlueprintCallable, Id="b1ca555b-dff3-4cca-bd26-662164ccfdd6") bool take_screen_exited() { const bool v = exited; exited = false; return v; }
};

// ---- RemoteTransform2D --------------------------------------------------------------------
//
// Copies this actor's world position, rotation and/or scale onto `target` every
// simulation step, so a node can follow another without being attached to it.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=remote2d, Id="118db825-4d5a-4595-9c4e-6d889a54a7d3") RemoteTransform2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("118db825-4d5a-4595-9c4e-6d889a54a7d3");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::remotes.add(this); }
    ~RemoteTransform2DComponent() override { world2d::remotes.remove(this); }
    world2d::Link<RemoteTransform2DComponent> link2d;

    ObjectId target;
    bool update_position = true, update_rotation = true, update_scale = true;

    EPOK_FUNCTION(BlueprintCallable, Id="160e83b7-2867-4337-9d27-78f7f2229eb4") void set_target(ObjectId value) { target = value; }
    EPOK_FUNCTION(BlueprintPure, Id="eeb69d41-56a5-4b7a-8c2a-ca836ed6a5bc") ObjectId get_target() const { return target; }
    // Pushes the transform once. Called by the service every simulation step.
    void push() {
        auto* registry = active_object_registry;
        auto* actor = registry && target.valid() ? registry->resolve<Actor>(target) : nullptr;
        auto* root = actor ? registry->resolve<SceneComponent2D>(actor->root_id()) : nullptr;
        Affine2D matrix;
        if (!root || !world2d::world_of(*this, matrix)) return;
        if (update_position) world2d::set_world_position(*root, matrix.t);
        // Rotation and scale of this actor's chain, applied relative to the target's parent.
        Fixed rotation = 0.0;
        const SceneComponent2D* node = world2d::root_of(*this);
        for (size_t depth = 0; node && depth < world2d_depth_limit; ++depth) {
            rotation += node->transform.rotation;
            node = node->attach_parent.valid() ? world2d::resolve_scene2d(node->attach_parent) : nullptr;
        }
        Fixed parent_rotation = 0.0;
        node = root->attach_parent.valid() ? world2d::resolve_scene2d(root->attach_parent) : nullptr;
        for (size_t depth = 0; node && depth < world2d_depth_limit; ++depth) {
            parent_rotation += node->transform.rotation;
            node = node->attach_parent.valid() ? world2d::resolve_scene2d(node->attach_parent) : nullptr;
        }
        if (update_rotation) root->transform.rotation = rotation - parent_rotation;
        if (update_scale) {
            const SceneComponent2D* own = world2d::root_of(*this);
            if (own) {
                root->transform.scale[0] = own->transform.scale[0];
                root->transform.scale[1] = own->transform.scale[1];
            }
        }
        transform2d_clamp(root->transform);
    }
};
}  // namespace epok
