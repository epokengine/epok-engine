// Host contract tests for the Object/Actor/Component runtime (actor-architecture P2/P3).
// Compiles the real runtime/object_model.hpp through epok.hpp; only the audio service and
// the class table are stubbed, exactly as the cooked build would provide them.
#include <cassert>
#include <cstdio>
#include <cstring>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/epok.hpp"

namespace epok {
// Audio service stub: the component forwards to it and nothing else here plays sound.
unsigned audio_plays = 0, audio_stops = 0;
bool audio_playing = false;
void AudioSource::play() { ++audio_plays; audio_playing = true; }
void AudioSource::stop() { ++audio_stops; audio_playing = false; }
bool AudioSource::is_playing() const { return audio_playing; }
}
using namespace epok;

// ---- event trace -------------------------------------------------------------------
namespace {
char trace[8192];
size_t trace_length = 0;
void note(const char* first, const char* second = "") {
    for (const char* text : {first, second})
        while (*text && trace_length + 2 < sizeof(trace)) trace[trace_length++] = *text++;
    if (trace_length + 2 < sizeof(trace)) trace[trace_length++] = ';';
    trace[trace_length] = 0;
}
void trace_reset() { trace_length = 0; trace[0] = 0; }
bool trace_is(const char* expected) {
    if (std::strcmp(trace, expected) == 0) return true;
    std::printf("  trace mismatch\n    actual:   %s\n    expected: %s\n", trace, expected);
    return false;
}

unsigned actor_destructors = 0, component_destructors = 0;
// Counters live outside the objects: pool storage is reused after end_play, so reading a
// destroyed actor would be undefined behaviour.
unsigned actor_ends = 0, component_ends = 0;
EndPlayReason actor_reason = EndPlayReason::Quit, component_reason = EndPlayReason::Quit;
bool suicide_on_begin = false;
unsigned chain_spawns = 0;
Level* level = nullptr;

// ---- test classes ------------------------------------------------------------------
struct ProbeRoot : SceneComponent3D {
    static constexpr uint64_t static_class_id = 0x2001;
    uint64_t class_id() const override { return static_class_id; }
    ~ProbeRoot() override { ++component_destructors; }
    void begin_play() override { note(name(), ".begin"); }
    void tick(Fixed) override { note(name(), ".tick"); }
    void end_play(EndPlayReason) override { note(name(), ".end"); }
    void on_enable() override { note(name(), ".enable"); }
    void on_disable() override { note(name(), ".disable"); }
};
// Owners=World3D|World2D|UI, Cardinality=Single.
struct ProbeComponent : ActorComponent {
    static constexpr uint64_t static_class_id = 0x2002;
    uint64_t class_id() const override { return static_class_id; }
    unsigned begins = 0;
    void begin_play() override { ++begins; note(name(), ".begin"); }
    void tick(Fixed) override { note(name(), ".tick"); }
    void end_play(EndPlayReason value) override { ++component_ends; component_reason = value; note(name(), ".end"); }
    void on_enable() override { note(name(), ".enable"); }
    void on_disable() override { note(name(), ".disable"); }
};
struct ProbeActor : Actor3D {
    static constexpr uint64_t static_class_id = 0x1001;
    uint64_t class_id() const override { return static_class_id; }
    ~ProbeActor() override { ++actor_destructors; }
    ProbeRoot probe_root;
    ActorComponent* default_root() override { return &probe_root; }
    unsigned begins = 0, ticks = 0, enables = 0, disables = 0;
    ObjectId destroy_on_tick;
    bool spawn_on_tick = false;
    void begin_play() override;
    void tick(Fixed) override;
    void end_play(EndPlayReason value) override { ++actor_ends; actor_reason = value; note(name(), ".end"); }
    void on_enable() override { ++enables; note(name(), ".enable"); }
    void on_disable() override { ++disables; note(name(), ".disable"); }
};
struct ProbeScript : SceneScriptActor {
    static constexpr uint64_t static_class_id = 0x1004;
    uint64_t class_id() const override { return static_class_id; }
    void begin_play() override { note(name(), ".begin"); }
    void tick(Fixed) override { note(name(), ".tick"); }
    void end_play(EndPlayReason) override { note(name(), ".end"); }
};
struct Probe2D : Actor2D {
    static constexpr uint64_t static_class_id = 0x1002;
    uint64_t class_id() const override { return static_class_id; }
};
struct ProbeUI : UIActor {
    static constexpr uint64_t static_class_id = 0x1003;
    uint64_t class_id() const override { return static_class_id; }
};
}  // namespace

// ---- cooked class table ------------------------------------------------------------
namespace epok {
const ClassDescriptor object_classes[] = {
    {Object::static_class_id, 0, ObjectFamily::Object, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Object), alignof(Object), nullptr, nullptr},
    {Actor::static_class_id, Object::static_class_id, ObjectFamily::Actor, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Actor), alignof(Actor), nullptr, nullptr},
    {Actor3D::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::World3D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), nullptr, nullptr, sizeof(Actor3D), alignof(Actor3D), nullptr, nullptr},
    {Actor2D::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::World2D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), nullptr, nullptr, sizeof(Actor2D), alignof(Actor2D), nullptr, nullptr},
    {UIActor::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::UI, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), nullptr, nullptr, sizeof(UIActor), alignof(UIActor), nullptr, nullptr},
    {SceneScriptActor::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::None, 0, ObjectClassSceneManaged, nullptr, nullptr, sizeof(SceneScriptActor), alignof(SceneScriptActor), nullptr, nullptr},
    {ActorComponent::static_class_id, Object::static_class_id, ObjectFamily::Component, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(ActorComponent), alignof(ActorComponent), nullptr, nullptr},
    {SceneComponent3D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World3D, object_domain_bit(ObjectDomain::World3D), ObjectClassRoot, &object_construct<SceneComponent3D>, &object_destruct, sizeof(SceneComponent3D), alignof(SceneComponent3D), &ObjectPool<SceneComponent3D, 4>::acquire, &ObjectPool<SceneComponent3D, 4>::release},
    {SceneComponent2D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, object_domain_bit(ObjectDomain::World2D), ObjectClassRoot, &object_construct<SceneComponent2D>, &object_destruct, sizeof(SceneComponent2D), alignof(SceneComponent2D), &ObjectPool<SceneComponent2D, 4>::acquire, &ObjectPool<SceneComponent2D, 4>::release},
    {UIComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::UI, object_domain_bit(ObjectDomain::UI), ObjectClassAbstract, nullptr, nullptr, sizeof(UIComponent), alignof(UIComponent), nullptr, nullptr},
    {RectTransformComponent::static_class_id, UIComponent::static_class_id, ObjectFamily::Component, ObjectDomain::UI, object_domain_bit(ObjectDomain::UI), ObjectClassRoot, &object_construct<RectTransformComponent>, &object_destruct, sizeof(RectTransformComponent), alignof(RectTransformComponent), &ObjectPool<RectTransformComponent, 4>::acquire, &ObjectPool<RectTransformComponent, 4>::release},
    {AudioComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None,
     uint8_t(object_domain_bit(ObjectDomain::World3D) | object_domain_bit(ObjectDomain::World2D) | object_domain_bit(ObjectDomain::UI)),
     ObjectClassMultiple, &object_construct<AudioComponent>, &object_destruct, sizeof(AudioComponent), alignof(AudioComponent), &ObjectPool<AudioComponent, 4>::acquire, &ObjectPool<AudioComponent, 4>::release},
    {Level::static_class_id, Object::static_class_id, ObjectFamily::Level, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Level), alignof(Level), nullptr, nullptr},
    {World::static_class_id, Object::static_class_id, ObjectFamily::World, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(World), alignof(World), nullptr, nullptr},
    // Test content classes, as the cook would emit them for Blueprint subclasses.
    {ProbeActor::static_class_id, Actor3D::static_class_id, ObjectFamily::Actor, ObjectDomain::World3D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<ProbeActor>, &object_destruct, sizeof(ProbeActor), alignof(ProbeActor), &ObjectPool<ProbeActor, 4>::acquire, &ObjectPool<ProbeActor, 4>::release},
    {Probe2D::static_class_id, Actor2D::static_class_id, ObjectFamily::Actor, ObjectDomain::World2D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<Probe2D>, &object_destruct, sizeof(Probe2D), alignof(Probe2D), &ObjectPool<Probe2D, 2>::acquire, &ObjectPool<Probe2D, 2>::release},
    {ProbeUI::static_class_id, UIActor::static_class_id, ObjectFamily::Actor, ObjectDomain::UI, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<ProbeUI>, &object_destruct, sizeof(ProbeUI), alignof(ProbeUI), &ObjectPool<ProbeUI, 2>::acquire, &ObjectPool<ProbeUI, 2>::release},
    {ProbeScript::static_class_id, SceneScriptActor::static_class_id, ObjectFamily::Actor, ObjectDomain::None, 0, ObjectClassSceneManaged, &object_construct<ProbeScript>, &object_destruct, sizeof(ProbeScript), alignof(ProbeScript), &ObjectPool<ProbeScript, 1>::acquire, &ObjectPool<ProbeScript, 1>::release},
    {ProbeRoot::static_class_id, SceneComponent3D::static_class_id, ObjectFamily::Component, ObjectDomain::World3D, object_domain_bit(ObjectDomain::World3D), ObjectClassRoot, &object_construct<ProbeRoot>, &object_destruct, sizeof(ProbeRoot), alignof(ProbeRoot), &ObjectPool<ProbeRoot, 4>::acquire, &ObjectPool<ProbeRoot, 4>::release},
    {ProbeComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None,
     uint8_t(object_domain_bit(ObjectDomain::World3D) | object_domain_bit(ObjectDomain::World2D) | object_domain_bit(ObjectDomain::UI)),
     0, &object_construct<ProbeComponent>, &object_destruct, sizeof(ProbeComponent), alignof(ProbeComponent), &ObjectPool<ProbeComponent, 4>::acquire, &ObjectPool<ProbeComponent, 4>::release},
};
const size_t object_class_count = sizeof(object_classes) / sizeof(object_classes[0]);
}

namespace {
const ClassDescriptor& class_of(uint64_t id) {
    const auto* found = find_object_class(id);
    assert(found);
    return *found;
}
ObjectRegistryStorage<32> registry_storage;
Level game_level;

void ProbeActor::begin_play() {
    ++begins;
    note(name(), ".begin");
    // The actor already knows its identity here, so it can request its own destruction.
    if (suicide_on_begin) level->destroy_actor(id());
}
void ProbeActor::tick(Fixed) {
    ++ticks;
    note(name(), ".tick");
    if (destroy_on_tick.valid()) level->destroy_actor(destroy_on_tick);
    if (spawn_on_tick) level->spawn_actor(class_of(ProbeActor::static_class_id), "Late");
}

// Rebuilds the level and the registry between cases; every object goes through the
// production teardown path first, so a leak shows up as a non-empty pool.
void reset() {
    if (level && level->registry()) {
        level->end_play_all(EndPlayReason::LevelUnloaded);
        registry_storage.release(level->id());
    }
    game_level = Level{};
    level = &game_level;
    active_object_registry = &registry_storage;
    registry_storage.stats = ObjectStats{};
    assert(game_level.bind(registry_storage));
    trace_reset();
    audio_plays = audio_stops = 0;
    audio_playing = false;
    actor_ends = component_ends = 0;
    suicide_on_begin = false;
    chain_spawns = 0;
    assert((ObjectPool<ProbeActor, 4>::live() == 0));
    assert((ObjectPool<ProbeComponent, 4>::live() == 0));
    assert((ObjectPool<AudioComponent, 4>::live() == 0));
}
ObjectId spawn(uint64_t class_id, const char* name, ObjectId parent = {}) {
    return game_level.spawn_actor(class_of(class_id), name, parent);
}

// 1. Concrete construction/destruction, identity through a base pointer, is_a chains.
void identity_and_storage() {
    reset();
    const unsigned destructors = actor_destructors;
    const ObjectId id = spawn(ProbeActor::static_class_id, "A");
    assert(id.valid());
    Object* base = registry_storage.get(id);
    assert(base && base->class_id() == ProbeActor::static_class_id);
    assert(base->is_a(Actor3D::static_class_id) && base->is_a(Actor::static_class_id) && base->is_a(Object::static_class_id));
    assert(!base->is_a(Actor2D::static_class_id));
    assert(registry_storage.resolve<Actor>(id) && registry_storage.resolve<Actor3D>(id));
    assert(!registry_storage.resolve<ActorComponent>(id));
    assert((ObjectPool<ProbeActor, 4>::live() == 1));
    // Three inheritance levels for a component: RectTransform -> UIComponent -> ActorComponent.
    assert(object_class_is_a(RectTransformComponent::static_class_id, UIComponent::static_class_id));
    assert(object_class_is_a(RectTransformComponent::static_class_id, ActorComponent::static_class_id));
    assert(object_class_is_a(RectTransformComponent::static_class_id, Object::static_class_id));
    assert(!object_class_is_a(RectTransformComponent::static_class_id, Actor::static_class_id));
    assert(!object_class_is_a(0, Object::static_class_id) && !object_class_is_a(Object::static_class_id, 0));
    // Handles die with the object; the reused slot carries a new generation.
    assert(game_level.destroy_actor(id));
    assert(!registry_storage.get(id) && !registry_storage.resolve<Actor>(id));
    assert(actor_destructors == destructors + 1);
    assert((ObjectPool<ProbeActor, 4>::live() == 0));
    const ObjectId again = spawn(ProbeActor::static_class_id, "B");
    assert(again.valid() && again.index == id.index && again.generation != id.generation);
    assert(!registry_storage.get(id) && registry_storage.get(again));
}

// 2. Batch lifecycle: components before actors, scene script last, tick order, end_play once.
void lifecycle_order() {
    reset();
    const ObjectId script = game_level.create_scene_script(class_of(ProbeScript::static_class_id), "Script");
    assert(script.valid());
    ActorSpawnRequest requests[2] = {};
    requests[0].type = &class_of(ProbeActor::static_class_id);
    requests[0].name = "A";
    requests[1].type = &class_of(ProbeActor::static_class_id);
    requests[1].name = "B";
    ObjectId spawned[2] = {};
    trace_reset();
    assert(game_level.spawn_batch(requests, 2, spawned) == 2);
    assert(trace_is("Root.begin;A.begin;Root.enable;A.enable;Root.begin;B.begin;Root.enable;B.enable;Script.begin;"));
    trace_reset();
    game_level.tick(0.25);
    assert(trace_is("Root.tick;A.tick;Root.tick;B.tick;Script.tick;"));
    auto* a = registry_storage.resolve<ProbeActor>(spawned[0]);
    auto* b = registry_storage.resolve<ProbeActor>(spawned[1]);
    assert(a && b && a->ticks == 1 && b->ticks == 1 && a->begins == 1);
    assert(a->state() == ObjectState::Playing);
    trace_reset();
    game_level.end_play_all(EndPlayReason::LevelUnloaded);
    // The scene script ends first, while the actors are still alive.
    assert(trace_is("Script.end;Root.disable;B.disable;Root.end;B.end;Root.disable;A.disable;Root.end;A.end;"));
    assert(game_level.actor_count() == 0);
    assert(registry_storage.live() == 1);   // only the Level itself remains
}

// 3. end_play exactly once with its reason, even if teardown is requested twice.
void end_play_once() {
    reset();
    const ObjectId id = spawn(ProbeActor::static_class_id, "A");
    auto* actor = registry_storage.resolve<ProbeActor>(id);
    auto* extra = game_level.add_component<ProbeComponent>(*actor, "Logic");
    assert(extra);
    assert(game_level.destroy_actor(id, EndPlayReason::Quit));
    assert(!game_level.destroy_actor(id, EndPlayReason::Quit));
    assert(actor_ends == 1 && actor_reason == EndPlayReason::Quit);
    assert(component_ends == 1 && component_reason == EndPlayReason::Quit);   // the Logic component
}

// 4. Destroy and spawn requested inside callbacks are deferred to the end of the batch.
void deferred_mutations() {
    reset();
    const ObjectId a = spawn(ProbeActor::static_class_id, "A");
    const ObjectId b = spawn(ProbeActor::static_class_id, "B");
    auto* first = registry_storage.resolve<ProbeActor>(a);
    assert(first && registry_storage.resolve<ProbeActor>(b));
    first->destroy_on_tick = b;
    trace_reset();
    game_level.tick(0.25);
    // B is marked immediately, so it never ticks; its teardown runs after the tick loop.
    assert(trace_is("Root.tick;A.tick;Root.disable;B.disable;Root.end;B.end;"));
    assert(game_level.stats().deferred == 1);
    assert(!registry_storage.get(b) && game_level.actor_count() == 1);
    first->destroy_on_tick = ObjectId{};

    // A spawn inside a callback never receives the events of the batch it was born in.
    reset();
    const ObjectId root = spawn(ProbeActor::static_class_id, "A");
    registry_storage.resolve<ProbeActor>(root)->spawn_on_tick = true;
    trace_reset();
    game_level.tick(0.25);
    assert(trace_is("Root.tick;A.tick;Root.begin;Late.begin;Root.enable;Late.enable;"));
    assert(game_level.actor_count() == 2 && game_level.stats().deferred == 1);
    registry_storage.resolve<ProbeActor>(root)->spawn_on_tick = false;

    // Destroying itself inside begin_play: no on_enable, no tick, end_play after the batch.
    reset();
    suicide_on_begin = true;
    trace_reset();
    const ObjectId doomed = spawn(ProbeActor::static_class_id, "A");
    assert(trace_is("Root.begin;A.begin;Root.disable;A.disable;Root.end;A.end;"));
    assert(!registry_storage.get(doomed) && actor_ends == 1 && game_level.actor_count() == 0);
    suicide_on_begin = false;
}

// 5. Capacity control: rejections counted, nothing half-built survives a failed batch.
void capacity_and_rollback() {
    reset();
    ObjectId actors[4] = {};
    for (auto& id : actors) { id = spawn(ProbeActor::static_class_id, "Filler"); assert(id.valid()); }
    const uint32_t rejected = game_level.stats().rejected;
    assert(!spawn(ProbeActor::static_class_id, "Overflow").valid());
    assert(game_level.stats().rejected == rejected + 1);
    assert(game_level.actor_count() == 4);
    for (auto id : actors) assert(game_level.destroy_actor(id));
    assert(actor_ends == 4);

    // A batch that fails while installing roots releases every reservation it made.
    ObjectRegistryStorage<3> tiny;
    Level small;
    assert(small.bind(tiny));
    ActorSpawnRequest requests[2] = {};
    requests[0].type = &class_of(ProbeActor::static_class_id);
    requests[0].name = "A";
    requests[1].type = &class_of(ProbeActor::static_class_id);
    requests[1].name = "B";
    ObjectId spawned[2] = {};
    assert(small.spawn_batch(requests, 2, spawned) == 0);
    assert(small.actor_count() == 0 && tiny.live() == 1 && tiny.stats.rejected > 0);
    assert((ObjectPool<ProbeActor, 4>::live() == 0));   // no orphan actors or components
    level = &game_level;
    active_object_registry = &registry_storage;
}

// 6. Components: acceptance, rejections, lookup and removal.
void component_rules() {
    reset();
    const ObjectId a = spawn(ProbeActor::static_class_id, "A");
    const ObjectId two = spawn(Probe2D::static_class_id, "Flat");
    const ObjectId ui = spawn(ProbeUI::static_class_id, "Panel");
    auto* actor = registry_storage.resolve<ProbeActor>(a);
    auto* flat = registry_storage.resolve<Probe2D>(two);
    auto* panel = registry_storage.resolve<ProbeUI>(ui);
    assert(actor && flat && panel);
    // Roots are registered by the defaults step of the batch.
    assert(actor->root_id().valid() && actor->component_count() == 1);
    assert(registry_storage.resolve<SceneComponent3D>(actor->root_id()) == &actor->probe_root);
    assert(registry_storage.resolve<SceneComponent2D>(flat->root_id()) == &flat->root);
    assert(registry_storage.resolve<RectTransformComponent>(panel->root_id()) == &panel->root);
    // UI/2D actors have no fictitious 3D transform.
    assert(!flat->data() && !panel->data() && !actor->data());
    assert(actor->probe_root.transform == &actor->probe_root.local);
    assert(panel->root.rect == &panel->root.local);

    // Audio is accepted by all three domains and is Multiple.
    assert(game_level.add_component<AudioComponent>(*actor, "Voice"));
    assert(game_level.add_component<AudioComponent>(*actor, "Steps"));
    assert(game_level.add_component<AudioComponent>(*flat, "Blip"));
    assert(game_level.add_component<AudioComponent>(*panel, "Click"));
    AudioComponent* sources[4] = {};
    assert(game_level.get_components<AudioComponent>(*actor, sources, 4) == 2);
    assert(game_level.get_component<AudioComponent>(*actor) == sources[0]);
    // Single cardinality: the second instance is rejected.
    assert(game_level.add_component<ProbeComponent>(*actor, "Logic"));
    const uint32_t rejected = game_level.stats().rejected;
    assert(!game_level.add_component<ProbeComponent>(*actor, "Duplicate"));
    // A UI component cannot live on a 3D actor.
    assert(!game_level.add_component<RectTransformComponent>(*actor, "Rect"));
    assert(game_level.stats().rejected == rejected + 2);
    // The root of a spatial actor cannot be removed; an ordinary component can.
    assert(!game_level.remove_component(*actor, actor->root_id()));
    auto* logic = game_level.get_component<ProbeComponent>(*actor);
    assert(logic && logic->begins == 1);
    const ObjectId logic_id = logic->id();
    const unsigned ends_before = component_ends;
    assert(game_level.remove_component(*actor, logic_id));
    assert(component_ends == ends_before + 1 && !registry_storage.get(logic_id));
    assert(!game_level.get_component<ProbeComponent>(*actor));
    assert(!game_level.remove_component(*actor, logic_id));
    // Audio stops when its component ends.
    const unsigned stops = audio_stops;
    sources[0]->bind_local();
    sources[0]->play();
    assert(audio_plays == 1);
    assert(game_level.destroy_actor(a));
    assert(audio_stops > stops);
}

// 7. Activation propagates to components and to logically parented actors.
void activation_propagation() {
    reset();
    const ObjectId parent = spawn(ProbeActor::static_class_id, "P");
    const ObjectId child = spawn(ProbeActor::static_class_id, "C", parent);
    auto* p = registry_storage.resolve<ProbeActor>(parent);
    auto* c = registry_storage.resolve<ProbeActor>(child);
    assert(p && c && c->logical_parent() == parent);
    trace_reset();
    assert(game_level.set_active(parent, false));
    assert(trace_is("Root.disable;P.disable;Root.disable;C.disable;"));
    assert(p->disables == 1 && c->disables == 1);
    trace_reset();
    assert(game_level.set_active(parent, false));   // idempotent
    assert(trace_is(""));
    // An inactive actor does not tick.
    game_level.tick(0.25);
    assert(trace_is(""));
    trace_reset();
    assert(game_level.set_active(parent, true));
    assert(trace_is("Root.enable;P.enable;Root.enable;C.enable;"));
}

// 8. Spatial attachment: same domain only, cycles rejected, logical parenting is separate.
void attachment_rules() {
    reset();
    const ObjectId a = spawn(ProbeActor::static_class_id, "A");
    const ObjectId b = spawn(ProbeActor::static_class_id, "B");
    const ObjectId ui = spawn(ProbeUI::static_class_id, "Panel");
    const ObjectId root_a = registry_storage.resolve<ProbeActor>(a)->root_id();
    const ObjectId root_b = registry_storage.resolve<ProbeActor>(b)->root_id();
    const ObjectId root_ui = registry_storage.resolve<ProbeUI>(ui)->root_id();
    assert(attach_component(root_b, root_a));
    assert(registry_storage.resolve<SceneComponent3D>(root_b)->attach_parent == root_a);
    // Cycle: A under B while B is already under A.
    const uint32_t rejected = game_level.stats().rejected;
    assert(!attach_component(root_a, root_b));
    assert(!attach_component(root_a, root_a));
    // Cross-domain attachment is rejected.
    assert(!attach_component(root_ui, root_a));
    assert(game_level.stats().rejected == rejected + 3);
    // Detaching is always allowed; logical parenting never inherits a matrix.
    assert(attach_component(root_b, ObjectId{}));
    assert(!registry_storage.resolve<SceneComponent3D>(root_b)->attach_parent.valid());
}

void compact_identities() {
    assert(Object::static_class_id == UINT64_C(14343360524917884802));
    assert(Actor::static_class_id == UINT64_C(3359541496846185808));
    assert(Actor3D::static_class_id == UINT64_C(11376955819108322094));
    assert(Actor2D::static_class_id == UINT64_C(14491127478636399617));
    assert(UIActor::static_class_id == UINT64_C(787666360041255473));
    assert(SceneScriptActor::static_class_id == UINT64_C(12838523513215880272));
    assert(ActorComponent::static_class_id == UINT64_C(16618224846313936943));
    assert(SceneComponent3D::static_class_id == UINT64_C(9884174540194706915));
    assert(SceneComponent2D::static_class_id == UINT64_C(87571934073580460));
    assert(UIComponent::static_class_id == UINT64_C(5886126345564491135));
    assert(RectTransformComponent::static_class_id == UINT64_C(6389541649464623131));
    assert(AudioComponent::static_class_id == UINT64_C(5821298606789721782));
    assert(Level::static_class_id == UINT64_C(12599419720711463237));
    assert(World::static_class_id == UINT64_C(6971179517075037215));
}

void size_report() {
    std::printf("Object model sizes (host, %zu-bit pointers):\n", sizeof(void*) * 8);
    std::printf("  Object %zu  Actor %zu  Actor3D %zu  Actor2D %zu  UIActor %zu  SceneScriptActor %zu\n",
                sizeof(Object), sizeof(Actor), sizeof(Actor3D), sizeof(Actor2D), sizeof(UIActor), sizeof(SceneScriptActor));
    std::printf("  ActorComponent %zu  SceneComponent3D %zu  SceneComponent2D %zu  UIComponent %zu  RectTransformComponent %zu\n",
                sizeof(ActorComponent), sizeof(SceneComponent3D), sizeof(SceneComponent2D), sizeof(UIComponent), sizeof(RectTransformComponent));
    std::printf("  AudioComponent %zu  Level %zu  World %zu\n",
                sizeof(AudioComponent), sizeof(Level), sizeof(World));
    std::printf("  ObjectId %zu  ObjectSlot %zu  ClassDescriptor %zu  ObjectRegistryStorage<32> %zu\n",
                sizeof(ObjectId), sizeof(ObjectSlot), sizeof(ClassDescriptor), sizeof(ObjectRegistryStorage<32>));
    std::printf("  pool storage_bytes: Actor3D x4 %zu  SceneComponent3D x4 %zu  AudioComponent x4 %zu\n",
                ObjectPool<Actor3D, 4>::storage_bytes, ObjectPool<SceneComponent3D, 4>::storage_bytes,
                ObjectPool<AudioComponent, 4>::storage_bytes);
}
}  // namespace

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    compact_identities();
    identity_and_storage();
    lifecycle_order();
    end_play_once();
    deferred_mutations();
    capacity_and_rollback();
    component_rules();
    activation_propagation();
    attachment_rules();
    reset();
    size_report();
    std::puts("Object model identity, pools, lifecycle order, deferral, components, attachment tests passed.");
    return 0;
}
