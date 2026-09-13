#pragma once
// Cooked actor tables and the process-wide Object/Actor/Component state.
//
// The Rust cook emits one `ActorTable` per scene bank (see `project::actor_table_body`)
// plus the capacity macro below. This header owns the two pieces of state the generated
// banks and main.cpp share: the slot table (`object_registry`) and the `Level`
// (`level`). Keeping them here means main.cpp sees exactly one owner, and a bank that
// contains no actors still links against the same symbols.
//
// No RTTI, exceptions, heap or standard containers: this compiles for MIPS with the rest
// of the runtime and for the host test harness.
#include "object_model.hpp"

// Cooked capacity: max over banks of (actors + components + 32 dynamic). The generated
// scene bank header defines it before including this file; the fallback keeps standalone
// translation units (host tests, syntax probes) compiling.
#ifndef EPOK_OBJECT_REGISTRY_CAPACITY
#define EPOK_OBJECT_REGISTRY_CAPACITY 64
#endif

// Lets scene_service.hpp tear the level down only in a build that actually has one.
#define EPOK_ACTOR_TABLES 1

namespace epok {

// ---- cooked row shapes -------------------------------------------------------------
// One authored component of one actor. Indices are table-local and 16 bit; -1 is null.
struct ActorComponentRecord {
    uint64_t class_id = 0;
    const char* name = nullptr;
    bool root = false;
    // Component index inside the same actor whose component this one attaches to.
    int16_t attach_parent = -1;
    // Legacy entity slot backing a root scene component, or -1 for component-owned
    // storage. Only a root record ever carries one.
    int16_t legacy_slot = -1;
};
// One authored actor. `apply` writes the cooked property overrides; it is null when the
// actor and its components override nothing.
struct ActorRecord {
    uint64_t class_id = 0;
    const char* name = nullptr;
    bool active = true;
    int16_t logical_parent = -1;   // actor index in this table
    int16_t attach_actor = -1;     // actor index this actor's root attaches to
    int16_t attach_component = -1; // component index inside that actor, -1 = its root
    const ActorComponentRecord* components = nullptr;
    size_t component_count = 0;
    void (*apply)(ObjectRegistry&, Actor&, const ObjectId*) = nullptr;
};
// One resolved map-scoped reference of the bank's scene Blueprint.
//
// P6 lowers every persisted ActorRef/ComponentRef/EntityRef default of a map's own
// Blueprint to the null identity and hands the resolution to the cook. A row names the
// owning class, the member inside it and the target as a *table index*, never as a UUID
// and never as a name: the loader turns it into an ObjectId (or an EntityHandle) before
// the scene script's begin_play, so nothing is ever looked up by name at tick.
enum class SceneRefKind : uint8_t { Actor = 0, Component = 1, Entity = 2 };
struct SceneReferenceRecord {
    uint64_t class_id = 0;   // compact id of the owning class (the cooked scene script)
    uint64_t member = 0;     // compact id of the persisted member the writer switches on
    SceneRefKind kind = SceneRefKind::Actor;
    int16_t actor = -1;      // actor table index, for Actor and Component targets
    int16_t component = -1;  // component index inside that actor, for Component targets
    int16_t entity = -1;     // legacy entity slot index, for Entity targets
};
// One scene bank. `scene_script_class` is 0 when the map authored no scene Blueprint; the
// loader then instantiates the base epok::SceneScriptActor, so every bank has exactly one.
//
// The reference fields are trailing and default to "none", so a bank that resolves no
// map-scoped reference emits exactly the three-field initializer it always did.
struct ActorTable {
    const ActorRecord* actors = nullptr;
    size_t count = 0;
    uint64_t scene_script_class = 0;
    const SceneReferenceRecord* references = nullptr;
    size_t reference_count = 0;
    // Generated writer: assigns the resolved identity to the member the record names.
    // Typed member access needs the C++ class, so the cook emits the switch; the table
    // stays plain data.
    void (*bind_reference)(ObjectRegistry&, Actor&, uint64_t member, ObjectId target,
                           EntityHandle entity) = nullptr;
};

// Counters read from RAM by tools/profile_runtime.py. Plain uint32 fields in declaration
// order, like the other *_stats structs the profiler understands.
struct ActorStats {
    uint32_t alive = 0, peak = 0, rejected = 0, spawned = 0, deferred = 0;
    uint32_t actors = 0, components = 0, scene_scripts = 0, banks_loaded = 0;
};
inline ActorStats actor_stats;

// ---- level ------------------------------------------------------------------------
// Level specialization that knows how to instantiate a cooked table.
//
// Actor identity, names, the active flag, the logical parent, the declarative root and
// registration all come from Level::spawn_batch, unchanged. What this class adds is the
// cooked half: the authored component set (by class id rather than by C++ type), the
// legacy slot binding, spatial attachment and the generated property overrides.
class SceneLevel : public Level {
public:
    // Binds the slot table once. Calling it again is a no-op, so a scene transition
    // reuses the same registry and the same Level identity.
    bool ensure_bound(ObjectRegistry& registry) {
        if (m_registry == &registry) { active_object_registry = &registry; return true; }
        return bind(registry);
    }

    // Adds a component named by its cooked class id. Mirrors Level::add_component<T>
    // (same owner-domain, cardinality, abstractness and capacity rules) without needing
    // the C++ type at the call site. begin_play is deferred to finish_components().
    ObjectId add_component_by_class(Actor& owner, const ClassDescriptor& type, const char* name) {
        if (!m_registry) return {};
        const auto* owner_type = find_object_class(owner.class_id());
        if (!owner_type || !accepts_component(owner, *owner_type, type)) {
            ++m_registry->stats.rejected;
            return {};
        }
        const ObjectId id = m_registry->acquire(type);
        auto* component = m_registry->resolve<ActorComponent>(id);
        if (!component) {
            if (id.valid()) m_registry->release(id);
            return {};
        }
        attach_component_record(owner, *component, name);
        set_state(id, ObjectState::Initialized);
        return id;
    }

    // Loads one cooked bank. Returns the number of actors instantiated.
    //
    // Order (design.md section 6; the P10 deviation is closed):
    //   1. Level::spawn_batch -> reserve, defaults, roots, owners, register;
    //   2. the prepare hook, per actor of the batch: authored components, legacy slot
    //      binding, attachment and the generated property overrides;
    //   3. begin_play -- components first, then the owning actor -- so an actor's own
    //      begin_play already sees its per-instance overrides and components;
    //   4. the bank's single SceneScriptActor: created, its map-scoped references bound,
    //      then begin_play.
    // The scene script therefore always observes fully configured actors.
    size_t load_bank(const ActorTable& table, Entity* slots, size_t slot_count) {
        if (!m_registry) return 0;
        ObjectId spawned[level_actor_capacity] = {};
        size_t count = table.count;
        if (count > level_actor_capacity) { ++m_registry->stats.rejected; return 0; }
        // Context for prepare_cooked_actor(), which the base Level calls back with the
        // batch-local index only.
        m_table = &table;
        m_spawned = spawned;
        m_slots = slots;
        m_slot_count = slot_count;
        m_base = 0;
        m_count = count;
        for (size_t i = 0; i < level_actor_capacity; ++i) m_attached[i] = false;
        // The cook emits every actor after its logical parent. Spawning runs one batch
        // per generation so a child's request can carry the parent's ObjectId, which
        // Level::spawn_batch only knows how to set at reservation time.
        size_t done = 0;
        while (done < count) {
            ActorSpawnRequest requests[level_actor_capacity] = {};
            size_t group = 0;
            while (done + group < count) {
                const ActorRecord& record = table.actors[done + group];
                if (record.logical_parent >= 0 && size_t(record.logical_parent) >= done) break;
                requests[group].type = find_object_class(record.class_id);
                requests[group].name = record.name;
                requests[group].active = record.active;
                requests[group].logical_parent =
                    record.logical_parent >= 0 ? spawned[size_t(record.logical_parent)] : ObjectId{};
                ++group;
            }
            if (!group) { ++m_registry->stats.rejected; break; }
            m_base = done;
            m_count = done + group;
            if (spawn_batch(requests, group, spawned + done, &prepare_cooked_actor) != group) break;
            done += group;
        }
        count = done;
        m_count = count;
        // Attachment to an actor the cook emitted in a later generation could not be
        // resolved while that actor had no ObjectId yet; it is applied here instead.
        for (size_t i = 0; i < count; ++i)
            if (!m_attached[i]) attach_to_actor(table.actors[i], i, count);
        create_bank_scene_script(table);
        m_table = nullptr;
        m_spawned = nullptr;
        m_slots = nullptr;
        m_slot_count = m_base = m_count = 0;
        ++actor_stats.banks_loaded;
        refresh_stats();
        return count;
    }

    // Cooked scene script for the bank. Exactly one per loaded level: the cooked class
    // when the map authored a scene Blueprint, the runtime base otherwise.
    ObjectId create_bank_scene_script(const ActorTable& table) {
        if (!m_registry || scene_script().valid()) return {};
        const ClassDescriptor* type = find_object_class(table.scene_script_class);
        if (!type || type->family != ObjectFamily::Actor || (type->flags & ObjectClassAbstract))
            type = find_object_class(SceneScriptActor::static_class_id);
        if (!type) return {};
        const ObjectId id = create_scene_script(*type, "SceneScript");
        if (id.valid()) {
            ++actor_stats.scene_scripts;
            // Persistent references are ObjectIds before begin_play, never names at tick.
            bind_scene_references(table, id);
            begin_play_scene_script();
        }
        return id;
    }

    // Binds the bank's resolved map-scoped references into the scene script instance.
    // Every target is a table index the cook resolved by UUID and scope (P6); this turns
    // it into a live identity and hands it to the generated writer.
    size_t bind_scene_references(const ActorTable& table, ObjectId owner) {
        if (!m_registry || !table.references || !table.bind_reference) return 0;
        auto* instance = m_registry->resolve<Actor>(owner);
        if (!instance) return 0;
        size_t bound = 0;
        for (size_t i = 0; i < table.reference_count; ++i) {
            const SceneReferenceRecord& record = table.references[i];
            // A row belongs to the class it was cooked for; a stale one binds nothing.
            if (record.class_id && !object_class_is_a(instance->class_id(), record.class_id)) continue;
            ObjectId target;
            EntityHandle entity;
            switch (record.kind) {
            case SceneRefKind::Actor:
                if (record.actor >= 0 && size_t(record.actor) < m_count && m_spawned)
                    target = m_spawned[size_t(record.actor)];
                break;
            case SceneRefKind::Component:
                if (record.actor >= 0 && size_t(record.actor) < m_count && m_spawned)
                    if (auto* owner_actor = m_registry->resolve<Actor>(m_spawned[size_t(record.actor)]))
                        target = record.component >= 0 ? owner_actor->component_id(size_t(record.component))
                                                       : owner_actor->root_id();
                break;
            case SceneRefKind::Entity:
                if (record.entity >= 0 && m_slots && size_t(record.entity) < m_slot_count)
                    entity = handle(&m_slots[size_t(record.entity)]);
                break;
            }
            table.bind_reference(*m_registry, *instance, record.member, target, entity);
            ++bound;
        }
        return bound;
    }

    // Snapshot of the registry and level counters for tools/profile_runtime.py.
    void refresh_stats() {
        if (!m_registry) return;
        const ObjectStats& source = m_registry->stats;
        actor_stats.alive = source.alive;
        actor_stats.peak = source.peak;
        actor_stats.rejected = source.rejected;
        actor_stats.spawned = source.spawned;
        actor_stats.deferred = source.deferred;
        actor_stats.actors = uint32_t(m_actor_count);
        uint32_t components = 0;
        for (size_t i = 0; i < m_actor_count; ++i)
            if (auto* actor = m_registry->resolve<Actor>(m_actors[i]))
                components += uint32_t(actor->component_count());
        actor_stats.components = components;
    }

private:
    // Level::spawn_batch preparation hook. Runs after the actor's defaults, root, owner
    // and registration, and before any begin_play of the batch.
    static void prepare_cooked_actor(Level& level, Actor& actor, size_t batch_index) {
        static_cast<SceneLevel&>(level).configure_actor(actor, batch_index);
    }

    // Cooked configuration of one actor: authored components, legacy slot binding,
    // spatial attachment and property overrides.
    void configure_actor(Actor& instance, size_t batch_index) {
        if (!m_table || !m_spawned) return;
        const size_t index = m_base + batch_index;
        if (index >= m_table->count) return;
        const ActorRecord& record = m_table->actors[index];
        Actor* actor = &instance;
        Entity* slots = m_slots;
        const size_t slot_count = m_slot_count;
        ObjectId components[actor_component_capacity] = {};
        size_t mapped = 0;
        for (size_t c = 0; c < record.component_count && c < actor_component_capacity; ++c) {
            const ActorComponentRecord& entry = record.components[c];
            ObjectId id;
            if (entry.root) {
                // The declarative root already exists; the cook only names and binds it.
                id = actor->root_id();
                if (auto* component = m_registry->resolve<ActorComponent>(id)) component->set_name(entry.name);
            } else if (const auto* type = find_object_class(entry.class_id)) {
                id = add_component_by_class(*actor, *type, entry.name);
            } else {
                ++m_registry->stats.rejected;
            }
            components[mapped++] = id;
            bind_legacy_slot(id, entry.legacy_slot, slots, slot_count);
        }
        for (size_t c = 0; c < mapped; ++c) {
            const ActorComponentRecord& entry = record.components[c];
            if (entry.attach_parent >= 0 && size_t(entry.attach_parent) < mapped)
                attach_component(components[c], components[size_t(entry.attach_parent)]);
        }
        m_attached[index] = attach_to_actor(record, index, m_count);
        // The generated overrides land before begin_play, so an actor's own begin_play
        // graph reads its per-instance values and not the class defaults.
        if (record.apply) record.apply(*m_registry, *actor, components);
    }

    // Spatial attachment of this actor's root to another actor of the same bank. Returns
    // false when the target has no identity yet (it belongs to a later spawn generation),
    // in which case load_bank retries once every actor exists.
    bool attach_to_actor(const ActorRecord& record, size_t index, size_t count) {
        if (record.attach_actor < 0 || size_t(record.attach_actor) >= count ||
            size_t(record.attach_actor) == index)
            return true;
        auto* actor = m_registry->resolve<Actor>(m_spawned[index]);
        auto* target = m_registry->resolve<Actor>(m_spawned[size_t(record.attach_actor)]);
        if (!actor || !target) return false;
        ObjectId parent = target->root_id();
        if (record.attach_component >= 0)
            parent = target->component_id(size_t(record.attach_component));
        // A rejected attachment (domain mismatch, cycle) is counted once by
        // attach_component and is not worth retrying: both actors already exist.
        attach_component(actor->root_id(), parent);
        return true;
    }

    // Canonical storage for a root backed by a legacy entity: collision, rendering and
    // motion interpolation keep reading exactly the same memory.
    void bind_legacy_slot(ObjectId id, int16_t slot, Entity* slots, size_t slot_count) {
        if (slot < 0 || !slots || size_t(slot) >= slot_count) return;
        Entity& entity = slots[size_t(slot)];
        if (auto* scene = m_registry->resolve<SceneComponent3D>(id)) scene->bind_slot(entity);
        else if (auto* rect = m_registry->resolve<RectTransformComponent>(id)) rect->bind_slot(entity);
        else if (auto* audio = m_registry->resolve<AudioComponent>(id)) audio->bind_slot(entity);
    }

    // Loader context for prepare_cooked_actor(), valid only for the duration of
    // load_bank(). The base Level hands the hook a batch-local index, so the bank, the
    // identities spawned so far and the legacy slot table live here.
    const ActorTable* m_table = nullptr;
    const ObjectId* m_spawned = nullptr;
    Entity* m_slots = nullptr;
    size_t m_slot_count = 0, m_base = 0, m_count = 0;
    bool m_attached[level_actor_capacity] = {};
};

// Process-wide state. One owner for the whole image.
inline ObjectRegistryStorage<EPOK_OBJECT_REGISTRY_CAPACITY> object_registry;
inline SceneLevel level;
inline constexpr size_t object_registry_capacity = EPOK_OBJECT_REGISTRY_CAPACITY;

// Called by the generated load_bank_N() before the legacy initialize_scripts().
inline size_t load_actor_bank(const ActorTable& table, Entity* slots, size_t slot_count) {
    level.ensure_bound(object_registry);
    return level.load_bank(table, slots, slot_count);
}
// Called by scene_tick() before the legacy binding teardown, while the actors and their
// legacy slots are still alive.
inline void unload_actor_bank() {
    level.end_play_all(EndPlayReason::LevelUnloaded);
    level.refresh_stats();
}
// Retries the slots an asynchronous service still points at. main.cpp calls it once per
// frame, next to the point where the legacy path re-checks music_active.
inline void collect_object_quarantine() { object_registry.collect_quarantined(); }

// ---- trigger fan-out ---------------------------------------------------------------
// The collision service reports legacy slot indices; the object model speaks ObjectIds.
// An actor participates in a trigger event when its *root* component is bound to the
// colliding slot -- that root is the canonical transform the collision world read.
inline ObjectId actor_for_slot(const Entity* slot) {
    if (!slot) return {};
    for (size_t i = 0; i < level.actor_count(); ++i) {
        const ObjectId id = level.actor_at(i);
        auto* actor = object_registry.resolve<Actor>(id);
        if (!actor) continue;
        const ObjectId root = actor->root_id();
        if (auto* scene = object_registry.resolve<SceneComponent3D>(root)) {
            if (scene->entity_slot() == slot) return id;
        } else if (auto* rect = object_registry.resolve<RectTransformComponent>(root)) {
            if (rect->entity_slot() == slot) return id;
        }
    }
    return {};
}
// One event, one delivery: the collision service calls this once per colliding slot, and
// epok::dispatch_trigger skips any component the legacy `bindings` table already
// notified (see component_trigger_filtered, installed by scene_service.hpp).
inline size_t dispatch_slot_trigger(EntityHandle self, EntityHandle other, TriggerPhase phase) {
    const Entity* slot = self.get();
    if (!slot) return 0;
    const ObjectId actor = actor_for_slot(slot);
    return actor.valid() ? dispatch_trigger(level, actor, other, phase) : 0;
}

}  // namespace epok
