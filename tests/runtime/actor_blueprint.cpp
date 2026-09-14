// Host contract test for the cooked half of the object model (actor-architecture P5).
//
// It compiles two things the Rust editor emits as text and that no other test sees:
//   * `epok::object_classes[]` / `epok::object_class_count` in the exact shape
//     `blueprint_spawn::object_class_table` writes, including the pool budget assert;
//   * the class shape `blueprint_compile` generates for an Actor- and a Component-family
//     Blueprint, including the null-safe adapters in runtime/actor_blueprint.hpp.
//
// If the emitted text ever stops being valid C++, this fails here rather than in a
// project build on a machine with the MIPS SDK.
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
// The Blueprint continuation machine is compiled into a project only when it has
// Blueprints, exactly as the cook defines it.
#define EPOK_BLUEPRINTS 1
#include "../../runtime/epok.hpp"

#include "../../runtime/actor_blueprint.hpp"
#include "../../runtime/blueprint_runtime.hpp"

namespace epok {
// Audio service stub; nothing here plays sound.
void AudioSource::play() {}
void AudioSource::stop() {}
bool AudioSource::is_playing() const { return false; }

// Scene slot service. A cooked game supplies these from its scene bank; the host suite
// resolves handles against the component-owned ActorData storage of the actors below, which
// is all `epok::bp::actor_handle` needs to be exercised.
static ActorData* slots[8] = {};
static uint16_t slot_count = 0;
DataHandle handle(const ActorData* entity) {
    DataHandle result;
    if (!entity) return result;
    for (uint16_t i = 0; i < slot_count; ++i)
        if (slots[i] == entity) {
            result.index = i;
            result.generation = entity->generation;
            return result;
        }
    if (slot_count < 8) {
        slots[slot_count] = const_cast<ActorData*>(entity);
        result.index = slot_count++;
        result.generation = entity->generation;
    }
    return result;
}
ActorData* DataHandle::get() const {
    return index < slot_count && slots[index]->alive && slots[index]->generation == generation
               ? slots[index]
               : nullptr;
}
bool is_active(const ActorData* entity) { return entity && entity->alive && entity->active; }
}  // namespace epok

using namespace epok;

// ---- generated shape: Actor-family Blueprint ---------------------------------------
// `BP_Goblin : public epok::Actor3D` with an authored begin_play, a synthesized tick
// pump and an end_play that cancels the continuation tasks.
class BP_Goblin : public epok::Actor3D {
public:
    static constexpr uint64_t static_class_id = UINT64_C(0x5001);
    uint64_t class_id() const override { return static_class_id; }
    unsigned begins = 0, ticks = 0, ends = 0;

    virtual void begin_play() override {
        epok::ActorData* epok_self = epok::bp::actor_entity(this);
        (void)epok_self;
        const auto epok_owner = epok_self ? epok::handle(epok_self) : epok::DataHandle{};
        (void)epok_owner;
        ++begins;
        epok::Actor3D::begin_play();
    }
    epok::bp::Continuations<8> epok_tasks;
    void tick(epok::Fixed dt) override {
        epok::ActorData* epok_self = epok::bp::actor_entity(this);
        (void)epok_self;
        const auto epok_owner = epok_self ? epok::handle(epok_self) : epok::DataHandle{};
        (void)epok_owner;
        epok_tasks.advance(dt, epok::blueprint_scene_generation);
        epok::Actor3D::tick(dt);
        epok::bp::Continuation epok_cont;
        while (epok_tasks.poll(epok_cont)) {
        }
        ++ticks;
    }
    void end_play(epok::EndPlayReason reason) override {
        epok_tasks.cancel_all();
        ++ends;
        epok::Actor3D::end_play(reason);
    }
};

// ---- generated shape: Component-family Blueprint ------------------------------------
// `Self` is the component; Get Owner yields the owning Actor as a typed ObjectId.
class BP_Siren : public epok::AudioComponent {
public:
    static constexpr uint64_t static_class_id = UINT64_C(0x5002);
    uint64_t class_id() const override { return static_class_id; }
    epok::ObjectId owner = epok::ObjectId{};
    void begin_play() override { owner = epok::bp::component_owner_id(this); }
};

// ---- cooked class table -------------------------------------------------------------
// The shape `blueprint_spawn::object_class_table` emits: one descriptor per non-Behaviour
// class, compact ids straight from the class identities, pool-backed storage for the
// concrete ones and the placement-new pair for embedded default components.
namespace epok {
#define EPOK_ABSTRACT_ROW(Type, Parent, Family, Domain)                                      \
    {Type::static_class_id, Parent, epok::ObjectFamily::Family, epok::ObjectDomain::Domain, 0, \
     1,  nullptr, nullptr, 0, 0, nullptr, nullptr}
#define EPOK_CONCRETE_ROW(Type, Parent, Family, Domain, Owners, Flags)                       \
    {Type::static_class_id, Parent, epok::ObjectFamily::Family, epok::ObjectDomain::Domain,    \
     Owners, Flags, &epok::object_construct<Type>, &epok::object_destruct, sizeof(Type),       \
     alignof(Type), &epok::ObjectPool<Type, 4>::acquire, &epok::ObjectPool<Type, 4>::release}

inline const ClassDescriptor object_classes[] = {
    EPOK_ABSTRACT_ROW(Object, UINT64_C(0), Object, None),
    EPOK_ABSTRACT_ROW(Actor, Object::static_class_id, Actor, None),
    EPOK_CONCRETE_ROW(Actor3D, Actor::static_class_id, Actor, World3D, 0, 6),
    EPOK_CONCRETE_ROW(BP_Goblin, Actor3D::static_class_id, Actor, World3D, 0, 6),
    EPOK_ABSTRACT_ROW(ActorComponent, Object::static_class_id, Component, None),
    EPOK_CONCRETE_ROW(SceneComponent3D, ActorComponent::static_class_id, Component, World3D, 1, 16),
    EPOK_CONCRETE_ROW(AudioComponent, ActorComponent::static_class_id, Component, None, 7, 32),
    EPOK_CONCRETE_ROW(BP_Siren, AudioComponent::static_class_id, Component, None, 7, 32),
    EPOK_ABSTRACT_ROW(Level, Object::static_class_id, Level, None),
};
[[gnu::used]] inline const size_t object_class_count =
    sizeof(object_classes) / sizeof(object_classes[0]);
static_assert((epok::ObjectPool<BP_Goblin, 4>::storage_bytes +
               epok::ObjectPool<BP_Siren, 4>::storage_bytes +
               epok::ObjectPool<epok::Actor3D, 4>::storage_bytes +
               epok::ObjectPool<epok::AudioComponent, 4>::storage_bytes +
               epok::ObjectPool<epok::SceneComponent3D, 4>::storage_bytes) <= 65536,
              "Object pools exceed the 64 KiB cook limit");
}  // namespace epok

int main() {
#ifdef _MSC_VER
    _CrtSetReportMode(_CRT_ASSERT, _CRTDBG_MODE_FILE);
    _CrtSetReportFile(_CRT_ASSERT, _CRTDBG_FILE_STDERR);
#endif
    // The table is walkable and the ancestry chain the cook emits resolves.
    assert(find_object_class(BP_Goblin::static_class_id) != nullptr);
    assert(object_class_is_a(BP_Goblin::static_class_id, Actor::static_class_id));
    assert(object_class_is_a(BP_Siren::static_class_id, ActorComponent::static_class_id));
    assert(!object_class_is_a(BP_Goblin::static_class_id, BP_Siren::static_class_id));
    assert(find_object_class(BP_Siren::static_class_id)->owners_mask == 7);
    assert((find_object_class(BP_Siren::static_class_id)->flags & 32) != 0);
    // Abstract bases stay walkable but never instantiable.
    assert(find_object_class(Actor::static_class_id)->acquire == nullptr);
    assert((find_object_class(Actor::static_class_id)->flags & 1) != 0);

    // A Blueprint actor runs its whole lifecycle through the cooked descriptor.
    static ObjectRegistryStorage<16> storage;
    ObjectRegistry& registry = storage;
    active_object_registry = &registry;
    Level level;
    assert(level.bind(registry));

    ActorSpawnRequest request;
    request.type = find_object_class(BP_Goblin::static_class_id);
    request.name = "goblin";
    ObjectId spawned;
    assert(level.spawn_batch(&request, 1, &spawned));
    auto* goblin = registry.resolve<BP_Goblin>(spawned);
    assert(goblin && goblin->begins == 1);

    // Until the scene binds a legacy slot behind the root component, the adapter is
    // null and every generated body has to tolerate that. This is the normal state of
    // an Actor2D or a UIActor, which have no 3D slot at all.
    assert(epok::bp::actor_entity(goblin) == nullptr);
    assert(epok::bp::actor_handle(goblin).get() == nullptr);

    static ActorData legacy_slot;
    legacy_slot.alive = true;
    legacy_slot.active = true;
    auto* root = registry.resolve<SceneComponent3D>(goblin->root_id());
    assert(root != nullptr);
    root->bind_slot(legacy_slot);
    goblin->bind_data(legacy_slot);
    assert(epok::bp::actor_entity(goblin) == &legacy_slot);
    assert(epok::bp::actor_handle(goblin).get() == &legacy_slot);
    assert(&epok::bp::actor_transform(goblin) == &legacy_slot.transform);

    // A component Blueprint on that actor resolves its owner.
    auto* siren = level.add_component<BP_Siren>(*goblin, "siren");
    assert(siren && siren->owner == spawned);
    assert(epok::bp::component_owner(siren) == goblin);
    assert(epok::bp::component_entity(siren) == epok::bp::actor_entity(goblin));

    level.tick(Fixed(1.0 / 60.0));
    assert(goblin->ticks == 1);

    // SpawnActor goes through the Level the actor belongs to, and refuses a component id.
    ObjectId child = epok::bp::spawn_actor(goblin, BP_Goblin::static_class_id, spawned);
    assert(registry.resolve<BP_Goblin>(child) != nullptr);
    assert(epok::bp::spawn_actor(goblin, BP_Siren::static_class_id, ObjectId{}) == ObjectId{});
    assert(epok::bp::spawn_actor(nullptr, BP_Goblin::static_class_id, ObjectId{}) == ObjectId{});

    // Null identities never dereference: a detached component has no owner, entity or
    // handle, and the transform alias still yields a usable lvalue.
    BP_Siren detached;
    assert(epok::bp::component_owner(&detached) == nullptr);
    assert(epok::bp::component_entity(&detached) == nullptr);
    assert(epok::bp::component_handle(&detached).get() == nullptr);
    assert(epok::bp::actor_entity(nullptr) == nullptr);
    epok::bp::component_transform(&detached).position[0] = Fixed(1.0);
    assert(epok::bp::actor_transform(nullptr).position[0] == Fixed(1.0));

    level.end_play_all(EndPlayReason::Quit);
    active_object_registry = nullptr;
    std::printf(
        "Actor Blueprint cook: class table shape, generated Actor/Component classes, owner "
        "resolution, bounded spawn and null-safe adapters passed.\n");
    return 0;
}
