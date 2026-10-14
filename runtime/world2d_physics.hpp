#pragma once
// World2D bounded rigid bodies and joints (World2D initiative, phase 4).
//
// A RigidBody2D moves its node under gravity, forces and impulses, using the node's
// solid Collider2D as its shape. The World2D service sweeps every awake body through the
// same collision world as move_and_slide (tiles and colliders, no tunnelling), bounces it
// off what it hits with restitution and Coulomb-style friction, exchanges momentum with
// the rigid bodies it runs into, lets characters push the pushable ones, rolls circles
// along floors and puts resting bodies to sleep. Boxes keep their rotation: the solver is
// axis-aligned, which keeps it exact and stable in Q12 for the few dozen bodies a scene
// holds.
//
// A Joint2D ties a rigid body to another one, or to the joint node's position: a rod
// (fixed distance), a rope (at most the distance) or a spring (stiffness and damping).
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_navigation.hpp"

namespace epok {
class RigidBody2DComponent;
class Joint2DComponent;

namespace world2d {
inline List<RigidBody2DComponent> rigid_bodies;
inline List<Joint2DComponent> joints;
}  // namespace world2d

enum class JointKind2D : uint8_t { Rod = 0, Rope = 1, Spring = 2 };

// ---- RigidBody2D ---------------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=physics2d, Id="5f046abe-757c-4499-96bb-35461de71053") RigidBody2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("5f046abe-757c-4499-96bb-35461de71053");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::rigid_bodies.add(this); }
    ~RigidBody2DComponent() override { world2d::rigid_bodies.remove(this); }
    world2d::Link<RigidBody2DComponent> link2d;

    // Authored.
    Fixed mass = 1.0, bounce = 0.2, friction = 0.5, linear_damping = 0.0;
    Fixed gravity[2] = {0.0, 720.0};
    uint32_t collision_mask = 0xffffffffu;
    bool frozen = false, pushable = true, can_sleep = true, rolls = true;
    // State.
    Fixed velocity[2] = {}, force[2] = {}, angular_velocity = 0.0, rest_clock = 0.0;
    bool sleeping = false, touching = false, contact = false;
    Fixed impact = 0.0;

    EPOK_FUNCTION(BlueprintCallable, Id="d85cf67c-fb5b-4187-8bdf-43fd72dd64a9") void apply_impulse(Fixed x, Fixed y) {
        if (frozen || mass.raw() <= 0) return;
        velocity[0] += Fixed(int32_t(int64_t(x.raw()) * 4096 / mass.raw()), Fixed::RAW);
        velocity[1] += Fixed(int32_t(int64_t(y.raw()) * 4096 / mass.raw()), Fixed::RAW);
        wake();
    }
    // A force for the next simulation step only (call it every tick to keep pushing).
    EPOK_FUNCTION(BlueprintCallable, Id="20386ec2-0809-4c1c-891b-fcdd60bd90d0") void apply_force(Fixed x, Fixed y) {
        force[0] += x;
        force[1] += y;
        wake();
    }
    EPOK_FUNCTION(BlueprintCallable, Id="916a893c-cc95-47bc-b0fa-221fbdc6a2a3") void set_velocity(Fixed x, Fixed y) {
        velocity[0] = x;
        velocity[1] = y;
        wake();
    }
    EPOK_FUNCTION(BlueprintPure, Id="d77904c0-2687-4a7c-b6f6-67a3e8afe0bc") Fixed get_velocity_x() const { return velocity[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="37e41687-753c-4837-a790-d7944b391b62") Fixed get_velocity_y() const { return velocity[1]; }
    EPOK_FUNCTION(BlueprintPure, Id="5fed34a7-989c-429a-ad3e-46ffec10d4e6") Fixed get_angular_velocity() const { return angular_velocity; }
    EPOK_FUNCTION(BlueprintPure, Id="4a2f34ca-c313-4a2c-be74-bc58840f2898") bool is_sleeping() const { return sleeping; }
    EPOK_FUNCTION(BlueprintCallable, Id="943f2475-fd54-4718-9359-e5886bb84646") void wake() {
        sleeping = false;
        rest_clock = 0.0;
    }
    // A frozen body stays where it is and blocks like a static one.
    EPOK_FUNCTION(BlueprintCallable, Id="79f98a77-e0ce-48ad-9646-858f51c88ff3") void set_frozen(bool value) {
        frozen = value;
        if (frozen) velocity[0] = velocity[1] = 0.0;
        wake();
    }
    EPOK_FUNCTION(BlueprintPure, Id="a1b0b6f0-2204-4291-8df0-b64f34f9e422") bool is_frozen() const { return frozen; }
    // True once when the body starts touching something; `get_impact` is how hard (px/s).
    EPOK_FUNCTION(BlueprintCallable, Id="40d9af79-f7ec-4eb5-b7d6-831d3ead94c1") bool take_contact() {
        const bool was = contact;
        contact = false;
        return was;
    }
    EPOK_FUNCTION(BlueprintPure, Id="5b7918a3-c981-4a52-85bf-72c124162844") Fixed get_impact() const { return impact; }
    EPOK_FUNCTION(BlueprintPure, Id="d682b2e3-d2a7-4a38-8490-dff5c166d78e") bool is_touching() const { return touching; }
    // Inverse mass in Q12 per unit: 0 for a frozen (immovable) body.
    int64_t inverse_mass() const { return frozen || mass.raw() <= 0 ? 0 : int64_t(4096) * 4096 / mass.raw(); }
};

// ---- Joint2D -------------------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Cardinality=Multiple, Capability=physics2d, Id="033cc190-e131-43f7-a4ea-54dc87817c1e") Joint2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("033cc190-e131-43f7-a4ea-54dc87817c1e");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::joints.add(this); }
    ~Joint2DComponent() override { world2d::joints.remove(this); }
    world2d::Link<Joint2DComponent> link2d;

    // `body_a` empty: the joint node's own position is the fixed end.
    ObjectId body_a, body_b;
    JointKind2D kind = JointKind2D::Rod;
    Fixed length = 32.0, stiffness = 40.0, damping = 2.0;
    bool enabled = true;

    EPOK_FUNCTION(BlueprintCallable, Id="64a7e48d-01b7-4134-ba08-0aaf75f8ab44") void set_enabled(bool value) { enabled = value; }
    EPOK_FUNCTION(BlueprintPure, Id="60f7dd18-7cec-4036-90f3-a35311f87af9") bool is_enabled() const { return enabled; }
    EPOK_FUNCTION(BlueprintCallable, Id="394d08a8-9b38-4b26-bf31-75c6aa56e9fc") void set_length(Fixed value) { length = value.raw() < 0 ? Fixed(0.0) : value; }
    EPOK_FUNCTION(BlueprintPure, Id="3b974dcf-b143-488c-a154-e8f1b341f8c2") Fixed get_length() const { return length; }
    // Cuts the joint for good: the rope snaps.
    EPOK_FUNCTION(BlueprintCallable, Id="57cb417b-3b7f-4861-b6b1-c259853460d7") void break_joint() {
        enabled = false;
        body_a = body_b = ObjectId{};
    }
};
}  // namespace epok
