#pragma once
// Object / Actor / Component class model (actor-architecture initiative).
//
// This header is included at the end of epok.hpp so the Clang extractor reflects the
// native bases automatically. The MIPS compiler sees plain C++20: no RTTI, exceptions,
// heap allocation or standard containers. Stable class identities are fixed by
// knowledge/initiatives/actor-architecture/design.md; a Rust unit test checks them.
//
// Declaration order follows C++ completeness requirements: the component classes precede
// Actor3D/Actor2D/UIActor because those actors embed their root component by value. The
// annotations, names and Ids are the contract; their order in the file is not.
#include "epok.hpp"
#include <new>
#include <stddef.h>
#include <stdint.h>

#if defined(__clang__) && defined(EPOK_REFLECTION)
// Declarative default component on an actor field: EPOK_COMPONENT(Root, Name="Body").
#define EPOK_COMPONENT(...) __attribute__((annotate("EPOK_COMPONENT:" #__VA_ARGS__)))
#else
#define EPOK_COMPONENT(...)
#endif

namespace epok {
class Object;
struct NativeComponentDefault {
    Object* component=nullptr;
    const char* name=nullptr;
    bool root=false;
    int16_t attach_parent=-1;
    uint64_t class_id=0;
};
// Compact runtime identity. Index 0xffff is null. Generations never revive a reused slot.
struct ObjectId {
    uint16_t index = 0xffff;
    uint16_t generation = 0;
    constexpr bool valid() const { return index != 0xffff && generation != 0; }
    Object* get() const;
    constexpr bool operator==(const ObjectId& other) const {
        return index == other.index && generation == other.generation;
    }
    constexpr bool operator!=(const ObjectId& other) const { return !(*this == other); }
};
enum class ObjectState : uint8_t { Unused, Reserved, Initialized, Playing, EndingPlay, Destroyed };
enum class EndPlayReason : uint8_t { Destroyed, LevelUnloaded, Quit };
enum class ObjectDomain : uint8_t { None, World3D, World2D, UI };
enum class ObjectFamily : uint8_t { Object, Actor, Component, World, Level };
// 2D world transform: XY position, rotation about the view axis, per-axis scale and a
// stable draw order. Q12 units match the 3D world; screen conversion belongs to Camera2D.
struct Transform2D {
    Fixed position[2] = {0.0, 0.0};
    Fixed rotation = 0.0;
    Fixed scale[2] = {1.0, 1.0};
    int16_t draw_order = 0;
};

// Class descriptor flags. The cook mirrors the reflected placement/component contract.
enum ObjectClassFlags : uint8_t {
    ObjectClassAbstract = 1,
    ObjectClassPlaceable = 2,
    ObjectClassSpawnable = 4,
    ObjectClassSceneManaged = 8,
    ObjectClassRoot = 16,
    ObjectClassMultiple = 32,
};
// Owner domain masks for components. None never appears in an owners mask.
constexpr uint8_t object_domain_bit(ObjectDomain domain) {
    return domain == ObjectDomain::None ? uint8_t(0) : uint8_t(1u << (uint8_t(domain) - 1u));
}

// Compact class identity: the first eight bytes of SHA-256("uuid"), little endian. This
// reproduces crate::blueprint_refs::compact_id exactly, so the C++ constant and the value
// the Rust cook emits are computed from the same UUID by the same algorithm and can never
// drift. Everything below is constexpr; nothing survives into the MIPS image.
namespace detail {
constexpr uint32_t sha256_k[64] = {
    0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
    0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
    0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
    0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
    0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
    0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
    0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
    0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u,
};
constexpr uint32_t sha256_rotr(uint32_t value, unsigned bits) {
    return uint32_t((value >> bits) | (value << (32u - bits)));
}
// Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).
constexpr uint64_t compact_class_id(const char* text) {
    unsigned length = 0;
    while (text[length]) ++length;
    unsigned char block[64] = {};
    for (unsigned i = 0; i < length && i < 55u; ++i) block[i] = uint8_t(text[i]);
    block[length < 55u ? length : 55u] = 0x80;
    const uint64_t bits = uint64_t(length) * 8u;
    for (unsigned i = 0; i < 8; ++i) block[63 - i] = uint8_t((bits >> (8u * i)) & 0xffu);
    uint32_t h[8] = {0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
                     0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u};
    uint32_t w[64] = {};
    for (unsigned i = 0; i < 16; ++i)
        w[i] = uint32_t(block[i * 4] << 24) | uint32_t(block[i * 4 + 1] << 16) |
               uint32_t(block[i * 4 + 2] << 8) | uint32_t(block[i * 4 + 3]);
    for (unsigned i = 16; i < 64; ++i) {
        const uint32_t s0 = sha256_rotr(w[i - 15], 7) ^ sha256_rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
        const uint32_t s1 = sha256_rotr(w[i - 2], 17) ^ sha256_rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
        w[i] = uint32_t(w[i - 16] + s0 + w[i - 7] + s1);
    }
    uint32_t a = h[0], b = h[1], c = h[2], d = h[3], e = h[4], f = h[5], g = h[6], x = h[7];
    for (unsigned i = 0; i < 64; ++i) {
        const uint32_t s1 = sha256_rotr(e, 6) ^ sha256_rotr(e, 11) ^ sha256_rotr(e, 25);
        const uint32_t ch = (e & f) ^ (~e & g);
        const uint32_t t1 = uint32_t(x + s1 + ch + sha256_k[i] + w[i]);
        const uint32_t s0 = sha256_rotr(a, 2) ^ sha256_rotr(a, 13) ^ sha256_rotr(a, 22);
        const uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
        const uint32_t t2 = uint32_t(s0 + maj);
        x = g; g = f; f = e; e = uint32_t(d + t1); d = c; c = b; b = a; a = uint32_t(t1 + t2);
    }
    h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += x;
    uint64_t value = 0;
    for (unsigned i = 0; i < 8; ++i) {
        const uint32_t word = h[i / 4];
        const uint64_t byte = uint64_t((word >> (24u - 8u * (i % 4u))) & 0xffu);
        value |= byte << (8u * i);
    }
    return value;
}
}  // namespace detail

class Object;
// Cooked class table. Storage comes from the class's ObjectPool through acquire/release;
// create/destroy expose the raw placement-new / virtual destructor pair for callers that
// own the memory (an actor's embedded default components, or a future arena).
struct ClassDescriptor {
    uint64_t id = 0;
    uint64_t parent = 0;
    ObjectFamily family = ObjectFamily::Object;
    ObjectDomain domain = ObjectDomain::None;
    uint8_t owners_mask = 0;
    uint8_t flags = 0;
    Object* (*create)(void* storage) = nullptr;
    void (*destroy)(Object* instance) = nullptr;
    size_t size = 0;
    size_t align = 0;
    Object* (*acquire)() = nullptr;
    void (*release)(Object* instance) = nullptr;
    size_t default_component_count=0;
    NativeComponentDefault (*default_component)(Object&,size_t)=nullptr;
};
// Emitted by the Rust cook next to the scene banks. Host tests provide their own table.
extern const ClassDescriptor object_classes[];
extern const size_t object_class_count;
inline const ClassDescriptor* find_object_class(uint64_t id) {
    if (id) for (size_t i = 0; i < object_class_count; ++i)
        if (object_classes[i].id == id) return &object_classes[i];
    return nullptr;
}
// Ancestry walk bounded by the table size; the cook rejects cycles before emitting.
inline bool object_class_is_a(uint64_t child, uint64_t parent) {
    if (!parent) return false;
    for (size_t depth = 0; child && depth <= object_class_count; ++depth) {
        if (child == parent) return true;
        const auto* info = find_object_class(child);
        if (!info) return false;
        child = info->parent;
    }
    return false;
}

class EPOK_CLASS(Abstract, Family=Object, Id="26a54c0d-ca81-41ca-aecc-d0346a6357d2") Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("26a54c0d-ca81-41ca-aecc-d0346a6357d2");
    virtual ~Object() = default;
    // Runtime-owned identity hooks; not an authoring API.
    virtual uint64_t class_id() const { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    virtual void timeline_sync(uint64_t, bool) {}
    ObjectId id() const { return m_id; }
    ObjectState state() const { return m_state; }
    bool is_a(uint64_t parent) const { return object_class_is_a(class_id(), parent); }
protected:
    friend struct ObjectRegistry;
    friend class Level;
    ObjectId m_id;
    ObjectState m_state = ObjectState::Unused;
    uint64_t m_runtime_class_id = 0;
};

// Placement-new into caller storage; destroy runs the virtual destructor of the concrete
// type, so a base pointer never slices.
template<class T> Object* object_construct(void* storage) { return new (storage) T(); }
inline void object_destruct(Object* instance) { if (instance) instance->~Object(); }

// Fixed typed storage for one concrete class. Mirrors bp::TypedPool: static storage, a
// used flag per slot, no heap and no construction until a slot is claimed.
template<class T, size_t N> struct ObjectPool {
    static_assert(N > 0 && N <= 256, "Object pool capacity must stay bounded");
    alignas(T) inline static unsigned char values[N][sizeof(T)] = {};
    inline static Object* handles[N] = {};
    inline static bool used[N] = {};
    static constexpr size_t capacity = N;
    static constexpr size_t storage_bytes = sizeof(values) + sizeof(handles) + sizeof(used);
    static Object* acquire() {
        for (size_t i = 0; i < N; ++i) if (!used[i]) {
            used[i] = true;
            handles[i] = object_construct<T>(values[i]);
            return handles[i];
        }
        return nullptr;
    }
    static void release(Object* instance) {
        for (size_t i = 0; i < N; ++i) if (used[i] && handles[i] == instance) {
            object_destruct(instance);
            handles[i] = nullptr;
            used[i] = false;
            return;
        }
    }
    static size_t live() {
        size_t count = 0;
        for (size_t i = 0; i < N; ++i) if (used[i]) ++count;
        return count;
    }
};

struct ObjectSlot {
    Object* instance = nullptr;
    const ClassDescriptor* type = nullptr;
    uint16_t generation = 0;
    ObjectState state = ObjectState::Unused;
    bool used = false;
    bool releasing = false;
    bool owns_storage = false;
};
struct ObjectStats { uint32_t alive = 0, peak = 0, rejected = 0, spawned = 0, deferred = 0, quarantined = 0; };

// ---- service quarantine ------------------------------------------------------------
// Some services retain the address of a component's storage across asynchronous
// callbacks. The XA music service is the one that exists today: `music_active` and
// `music_requested` (runtime/music.hpp) keep an AudioSource* while the CD driver's
// lookup/stop completions are still in flight, which is exactly why
// `allocate_actor_data` refuses to reuse a legacy slot whose `audio` is one of them.
// Component-owned storage needs the same rule, so the registry asks this predicate
// before handing released storage back to its pool.
//
// The audio service is linked after this header, so the cooked build installs the
// predicate instead of this header naming `music_active` (which would force it to be
// declared `extern` before runtime/music.hpp defines it as an inline variable). With
// no hook installed nothing is quarantined and release is immediate.
inline bool (*audio_source_retained)(const AudioSource*) = nullptr;
// Defined after AudioComponent; `false` for every object that owns no retained storage.
inline bool object_storage_quarantined(Object* instance);

// ---- trigger delivery filter -------------------------------------------------------

// Slot table over a fixed capacity. The table itself is a view so the capacity can come
// from a template default here or from the cook later without changing call sites.
struct ObjectRegistry {
    ObjectSlot* slots = nullptr;
    uint16_t capacity = 0;
    uint16_t dispatch = 0;
    ObjectStats stats;
    constexpr ObjectRegistry(ObjectSlot* table, uint16_t count) : slots(table), capacity(count) {}
    ObjectRegistry(const ObjectRegistry&) = delete;
    ObjectRegistry& operator=(const ObjectRegistry&) = delete;

    static uint16_t next_generation(uint16_t value) { ++value; return value ? value : uint16_t(1); }
    ObjectSlot* slot(ObjectId id) {
        if (!id.valid() || id.index >= capacity) return nullptr;
        auto& record = slots[id.index];
        if (!record.used || record.releasing || record.generation != id.generation) return nullptr;
        return &record;
    }
    const ObjectSlot* slot(ObjectId id) const { return const_cast<ObjectRegistry*>(this)->slot(id); }
    Object* get(ObjectId id) {
        auto* record = slot(id);
        return record ? record->instance : nullptr;
    }
    // Generation and class checked; the runtime class must derive from T's static class.
    template<class T> T* resolve(ObjectId id) {
        Object* instance = get(id);
        if (!instance) return nullptr;
        if (!object_class_is_a(instance->class_id(), T::static_class_id)) return nullptr;
        return static_cast<T*>(instance);
    }
    template<class T> const T* resolve(ObjectId id) const { return const_cast<ObjectRegistry*>(this)->resolve<T>(id); }
    const ClassDescriptor* class_of(ObjectId id) {
        auto* record = slot(id);
        return record ? record->type : nullptr;
    }

    // Pool-backed construction of a concrete class. Abstract classes are never created.
    ObjectId acquire(const ClassDescriptor& type) {
        if ((type.flags & ObjectClassAbstract) || !type.acquire || !type.release) { ++stats.rejected; return {}; }
        ObjectSlot* free_slot = nullptr;
        uint16_t index = 0;
        for (uint16_t i = 0; i < capacity; ++i) if (!slots[i].used) { free_slot = &slots[i]; index = i; break; }
        if (!free_slot) { ++stats.rejected; return {}; }
        Object* instance = type.acquire();
        if (!instance) { ++stats.rejected; return {}; }
        return install(*free_slot, index, instance, type, true);
    }
    // Adopt storage owned by somebody else (an actor's embedded default component, the
    // Level itself). Release never destroys it.
    ObjectId adopt(Object& instance, const ClassDescriptor& type) {
        ObjectSlot* free_slot = nullptr;
        uint16_t index = 0;
        for (uint16_t i = 0; i < capacity; ++i) if (!slots[i].used) { free_slot = &slots[i]; index = i; break; }
        if (!free_slot) { ++stats.rejected; return {}; }
        return install(*free_slot, index, &instance, type, false);
    }
    // Invalidates the handle immediately; storage is quarantined while callbacks run.
    bool release(ObjectId id) {
        auto* record = slot(id);
        if (!record) return false;
        record->state = ObjectState::Destroyed;
        if (record->instance) record->instance->m_state = ObjectState::Destroyed;
        record->releasing = true;
        record->generation = next_generation(record->generation);
        if (stats.alive) --stats.alive;
        if (!dispatch) finish_release();
        return true;
    }
    // Called when the last dispatch scope unwinds: no callback can still be on the stack
    // of a destroyed object, so its typed storage returns to the pool. Storage a service
    // still points at stays quarantined; the handle is already dead either way, so the
    // only effect is that the slot and the pool entry are not reused yet. Call again
    // (`collect_quarantined`) once the service releases it.
    void finish_release() {
        if (dispatch) return;
        stats.quarantined = 0;  // gauge, not a counter: slots still held by a service.
        for (uint16_t i = 0; i < capacity; ++i) {
            auto& record = slots[i];
            if (!record.used || !record.releasing) continue;
            if (object_storage_quarantined(record.instance)) { ++stats.quarantined; continue; }
            if (record.owns_storage && record.type && record.type->release) record.type->release(record.instance);
            record.instance = nullptr;
            record.type = nullptr;
            record.state = ObjectState::Unused;
            record.used = false;
            record.releasing = false;
            record.owns_storage = false;
        }
    }
    // Retry the quarantined slots. The scene bank calls it once per frame, next to the
    // point where the legacy path re-checks music_active before reusing a slot.
    void collect_quarantined() { finish_release(); }
    size_t live() const {
        size_t count = 0;
        for (uint16_t i = 0; i < capacity; ++i) if (slots[i].used && !slots[i].releasing) ++count;
        return count;
    }
private:
    ObjectId install(ObjectSlot& record, uint16_t index, Object* instance, const ClassDescriptor& type, bool owned) {
        record.generation = record.generation ? record.generation : uint16_t(1);
        record.instance = instance;
        record.type = &type;
        record.state = ObjectState::Reserved;
        record.used = true;
        record.releasing = false;
        record.owns_storage = owned;
        instance->m_id = ObjectId{index, record.generation};
        instance->m_state = ObjectState::Reserved;
        instance->m_runtime_class_id = type.id;
        ++stats.alive;
        ++stats.spawned;
        if (stats.alive > stats.peak) stats.peak = stats.alive;
        return instance->m_id;
    }
};
// Default capacity for a cooked build; tests instantiate a small table.
template<size_t Capacity = 256> struct ObjectRegistryStorage : ObjectRegistry {
    static_assert(Capacity > 0 && Capacity < 0xffff, "Slot indices are 16 bit with 0xffff reserved");
    ObjectSlot table[Capacity] = {};
    ObjectRegistryStorage() : ObjectRegistry(table, uint16_t(Capacity)) {}
};
// Storage of a destroyed object stays quarantined until every nested callback returns.
struct ObjectDispatchScope {
    ObjectRegistry& registry;
    explicit ObjectDispatchScope(ObjectRegistry& value) : registry(value) { ++registry.dispatch; }
    ~ObjectDispatchScope() { if (registry.dispatch) --registry.dispatch; registry.finish_release(); }
    ObjectDispatchScope(const ObjectDispatchScope&) = delete;
    ObjectDispatchScope& operator=(const ObjectDispatchScope&) = delete;
};
// Registry bound by the active Level. World ownership arrives with the World wiring phase.
inline ObjectRegistry* active_object_registry = nullptr;
inline Object* ObjectId::get() const {return active_object_registry ? active_object_registry->get(*this) : nullptr;}

class ActorComponent;
inline constexpr size_t actor_component_capacity = 8;
inline constexpr size_t level_actor_capacity = 64;
inline constexpr size_t level_pending_capacity = 16;
inline constexpr size_t object_hierarchy_depth = 16;

class EPOK_CLASS(Abstract, Blueprintable, Family=Actor, Domain=None, Id="6e6efc67-66c4-4dae-90f8-7c8c4e612dea") Actor : public Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("6e6efc67-66c4-4dae-90f8-7c8c4e612dea");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_FUNCTION(BlueprintEvent) virtual void begin_play() {}
    virtual void blueprint_observe() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void tick(Fixed) {}
    EPOK_FUNCTION(BlueprintEvent) virtual void end_play(EndPlayReason) {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_enable() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_disable() {}
    // Runtime hook, not reflected: the declarative root component embedded in the actor.
    virtual ActorComponent* default_root() { return nullptr; }
    // Legacy adapter: the canonical slot behind the root scene component, or nullptr.
    ActorData* data() const { return m_data; }
    void bind_data(ActorData& value) { m_data=&value; value.owner=this; }
    const char* name() const { return m_name; }
    void set_name(const char* value) {
        size_t i = 0;
        if (value) for (; i < 32 && value[i]; ++i) m_name[i] = value[i];
        m_name[i] = 0;
    }
    ObjectId level_id() const { return m_level; }
    ObjectId root_id() const { return m_root; }
    ObjectId logical_parent() const { return m_logical_parent; }
    ObjectId component_id(size_t index) const { return index < actor_component_capacity ? m_components[index] : ObjectId{}; }
    size_t component_count() const { return m_component_count; }
    // Self flag only; Level::actor_active() folds the logical parent chain.
    bool active() const { return m_active; }
    bool wants_tick() const { return m_wants_tick; }
    void set_wants_tick(bool value) { m_wants_tick = value; }
protected:
    friend class Level;
    char m_name[33] = {};
    ActorData* m_data=nullptr;
    ObjectId m_level, m_root, m_logical_parent;
    ObjectId m_components[actor_component_capacity] = {};
    uint8_t m_component_count = 0;
    bool m_active = true;
    bool m_wants_tick = true;
    bool m_begun = false, m_ended = false, m_doomed = false;
};

class EPOK_CLASS(Abstract, Blueprintable, Family=Component, Domain=None, Owners=World3D|World2D|UI, Id="2e5021ee-d14d-4d77-9112-455f29d639d2") ActorComponent : public Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("2e5021ee-d14d-4d77-9112-455f29d639d2");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_FUNCTION(BlueprintEvent) virtual void begin_play() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void tick(Fixed) {}
    EPOK_FUNCTION(BlueprintEvent) virtual void end_play(EndPlayReason) {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_enable() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_disable() {}
    // Runtime hooks, not reflected. frame_update runs once per rendered frame even while
    // paused; attach_slot exposes the spatial parent of components that have one.
    virtual void timeline_sync(uint64_t, bool) {}
    virtual void blueprint_observe() {}
    virtual void frame_update(uint32_t) {}
    virtual ObjectId* attach_slot() { return nullptr; }
    // Forward-only collision hook. The Level does not own collision; the collision
    // service calls dispatch_trigger(Level&, ...) which fans the event out to the
    // owner's components. Nothing in the object model generates trigger events.
    virtual void on_trigger(DataHandle, TriggerPhase) {}
    // False while a service still holds this component's storage; the registry then
    // keeps the (already dead) slot quarantined instead of returning it to the pool.
    virtual bool releasable() const { return true; }
    ObjectId owner_id() const { return m_owner; }
    Actor* get_owner();
    const char* name() const { return m_name; }
    void set_name(const char* value) {
        size_t i = 0;
        if (value) for (; i < 32 && value[i]; ++i) m_name[i] = value[i];
        m_name[i] = 0;
    }
protected:
    friend class Level;
    ObjectId m_owner;
    char m_name[33] = {};
    bool m_begun = false, m_ended = false;
};

class EPOK_CLASS(Blueprintable, Root, Domain=World3D, Owners=World3D, Id="ed73d249-b6cb-4a3c-a0e8-696de55e286f") SceneComponent3D : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("ed73d249-b6cb-4a3c-a0e8-696de55e286f");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ObjectId* attach_slot() override { return &attach_parent; }
    // View over the canonical slot transform when the owner is backed by a legacy entity.
    Transform* transform = nullptr;
    ObjectId attach_parent;
    // Canonical storage: the legacy entity slot owns the transform and everything that
    // already reads it (collision, rendering, motion interpolation) keeps working.
    void bind_slot(ActorData& value) { slot = &value; transform = &value.transform; }
    // No legacy slot: the component owns the transform.
    void bind_local() { slot = nullptr; transform = &local; }
    ActorData* entity_slot() const { return slot; }
    Transform local = {{0.0, 0.0, 0.0}, {0.0, 0.0, 0.0}, {1.0, 1.0, 1.0}};
protected:
    ActorData* slot = nullptr;
};
class EPOK_CLASS(Blueprintable, Root, Domain=World2D, Owners=World2D, Id="27887770-a779-4a16-863c-5abd32786cfa") SceneComponent2D : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("27887770-a779-4a16-863c-5abd32786cfa");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ObjectId* attach_slot() override { return &attach_parent; }
    // 2D actors own their transform: no legacy slot carries a Transform2D, and no
    // fictitious 3D transform is created for them.
    Transform2D transform;
    ObjectId attach_parent;
};
class EPOK_CLASS(Abstract, Blueprintable, Domain=UI, Owners=UI, Id="83bffb60-2c33-4be1-9041-8c8f4c395b86") UIComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("83bffb60-2c33-4be1-9041-8c8f4c395b86");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Root, Domain=UI, Owners=UI, Id="dc805165-6c65-48dc-8ff8-4a638a5d21df") RectTransformComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("dc805165-6c65-48dc-8ff8-4a638a5d21df");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ObjectId* attach_slot() override { return &attach_parent; }
    RectTransform* rect = nullptr;
    ObjectId attach_parent;
    void bind_slot(ActorData& value) { slot = &value; rect = &value.rect; }
    void bind_local() { slot = nullptr; rect = &local; }
    ActorData* entity_slot() const { return slot; }
    RectTransform local;
protected:
    ActorData* slot = nullptr;
};
// Shared audio without a transform. Forwards to the existing audio service.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D|UI, Cardinality=Multiple, Capability=audio, Id="7f0eb028-5301-4ac7-b93b-5665fab12b20") AudioComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("7f0eb028-5301-4ac7-b93b-5665fab12b20");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    AudioSource* source = nullptr;
    EPOK_FUNCTION(Callable) void play() { if (source) source->play(); }
    EPOK_FUNCTION(Callable) void stop() { if (source) source->stop(); }
    EPOK_FUNCTION(Pure) bool is_playing() const { return source && source->is_playing(); }
    // Bind to the legacy slot's AudioSource, or to component-owned storage.
    void bind_slot(ActorData& value) { source = &value.audio; m_slot = &value; }
    void bind_local() { source = &local; m_slot = nullptr; }
    ActorData* entity_slot() const { return m_slot; }
    // True only for component-owned storage. A slot-backed component is a *view* over a
    // legacy AudioSource that the scene bank and lifecycle.hpp already drive.
    bool owns_source() const { return source && source == &local; }
    AudioSource local;

    // play_on_start policy. Exactly one path starts a given AudioSource:
    //   * slot-backed: the scene bank's bank-load loop and bp::activate_spawn_audio
    //     (runtime/lifecycle.hpp) start it. begin_play here must NOT play, or a migrated
    //     entity would be heard twice.
    //   * component-owned: no legacy path knows about `local`, so begin_play starts it
    //     when play_on_start is set, the source is enabled and the owner is active.
    void begin_play() override {
        if (!source || !source->enabled || !source->play_on_start) return;
        if (!owner_active()) return;
        if (!source->is_playing()) source->play();
    }
    // Deactivation and teardown stop both kinds: lifecycle.hpp::set_active stops a legacy
    // slot's audio through the slot table, and stopping an already stopped source is a
    // no-op, so the two paths are idempotent rather than conflicting.
    void on_disable() override { if (source && source->is_playing()) source->stop(); }
    void end_play(EndPlayReason) override { if (source && source->is_playing()) source->stop(); }
    // Component-owned storage stays quarantined while an asynchronous consumer (the XA
    // music service) still points at it, mirroring allocate_actor_data's legacy-slot rule.
    // Slot storage belongs to the scene bank, so this component never holds it back.
    bool releasable() const override {
        if (!owns_source() || !audio_source_retained) return true;
        return !audio_source_retained(&local);
    }
private:
    bool owner_active() const;
    ActorData* m_slot = nullptr;
};
// Native components register their ownership with the Actor lifecycle.
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=mesh, Id="580f99b1-c905-4f96-b34f-807c51335ba0") Mesh3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("580f99b1-c905-4f96-b34f-807c51335ba0");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=sprite, Id="0b68656b-364b-441f-ad86-ddc0408e80a0") Sprite3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("0b68656b-364b-441f-ad86-ddc0408e80a0");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=camera, Id="9fe2abff-f285-435d-976d-825c5db5420a") Camera3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("9fe2abff-f285-435d-976d-825c5db5420a");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=light, Id="e18d296c-5820-4557-9386-8b12b9ca37a2") Light3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("e18d296c-5820-4557-9386-8b12b9ca37a2");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=collider, Id="e8431f94-e526-4d7d-aace-8c8fae7955e6") Collider3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("e8431f94-e526-4d7d-aace-8c8fae7955e6");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=canvas, Id="f2cfb26b-af53-4a46-9d91-1debea72e01b") CanvasComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("f2cfb26b-af53-4a46-9d91-1debea72e01b");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=image, Id="d04c78d6-23bd-40d7-88f1-b1afc1b05b5b") ImageComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("d04c78d6-23bd-40d7-88f1-b1afc1b05b5b");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=text, Id="fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34") TextComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=progress, Id="28bf5245-5d80-4cba-a77d-1d74479ac276") ProgressBarComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("28bf5245-5d80-4cba-a77d-1d74479ac276");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=particles, Id="1d067605-c408-40b8-b2c2-718b8cf0c601") ParticleEmitterComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c601");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D|UI, Capability=timeline, Id="1d067605-c408-40b8-b2c2-718b8cf0c602") TimelineComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c602");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=effect, Id="1d067605-c408-40b8-b2c2-718b8cf0c603") ParticleEffectComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c603");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=palette, Id="1d067605-c408-40b8-b2c2-718b8cf0c604") PaletteAnimatorComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c604");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=shadow, Id="1d067605-c408-40b8-b2c2-718b8cf0c605") BlobShadowComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c605");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};

class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World3D, Id="fc24ce9b-558c-49de-bc35-e040f350e486") Actor3D : public Actor {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("fc24ce9b-558c-49de-bc35-e040f350e486");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_COMPONENT(Root, Name="Root") SceneComponent3D root;
    ActorComponent* default_root() override { return &root; }
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World2D, Id="5308054e-0aaa-4d53-963b-440cf0c71916") Actor2D : public Actor {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("5308054e-0aaa-4d53-963b-440cf0c71916");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_COMPONENT(Root, Name="Root") SceneComponent2D root;
    ActorComponent* default_root() override { return &root; }
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=UI, Id="b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f") UIActor : public Actor {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_COMPONENT(Root, Name="Root") RectTransformComponent root;
    ActorComponent* default_root() override { return &root; }
};
// One per loaded Level. The loader creates it; it is never placed or spawned by content.
class EPOK_CLASS(Blueprintable, SceneManaged, Id="b4c08aa0-fa85-4abf-8f45-7501e1c8a040") SceneScriptActor : public Actor {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("b4c08aa0-fa85-4abf-8f45-7501e1c8a040");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
};

// One spawn request. The name must outlive the call; a deferred request keeps the pointer
// until the batch flushes, so cooked content passes the scene bank's string literals.
struct ActorSpawnRequest {
    const ClassDescriptor* type = nullptr;
    const char* name = nullptr;
    bool active = true;
    ObjectId logical_parent;
};

// One queued spawn/destroy request, executed when the current batch finishes.
struct LevelPendingOp {
    ObjectId actor;
    ActorSpawnRequest request;
    EndPlayReason reason = EndPlayReason::Destroyed;
    bool destroy = false;
};

class EPOK_CLASS(Abstract, Family=Level, Id="b4683321-83e2-4b90-bd87-bb314f9eda2e") Level : public Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("b4683321-83e2-4b90-bd87-bb314f9eda2e");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }

    // Binds the slot table and registers the Level itself. Also publishes the registry as
    // the process-wide one used by Actor::entity()/ActorComponent::get_owner().
    bool bind(ObjectRegistry& value) {
        m_registry = &value;
        active_object_registry = &value;
        const auto* type = find_object_class(class_id());
        if (!type) return false;
        return value.adopt(*this, *type).valid();
    }
    ObjectRegistry* registry() const { return m_registry; }
    ObjectStats stats() const { return m_registry ? m_registry->stats : ObjectStats{}; }
    size_t actor_count() const { return m_actor_count; }
    ObjectId actor_at(size_t index) const { return index < m_actor_count ? m_actors[index] : ObjectId{}; }
    ObjectId scene_script() const { return m_scene_script; }
    bool set_logical_parent(ObjectId child,ObjectId parent) {
        auto* actor=m_registry?m_registry->resolve<Actor>(child):nullptr;
        if(!actor)return false;
        ObjectId current=parent;
        for(size_t depth=0;current.valid();++depth) {
            if(current==child || depth>=level_actor_capacity)return false;
            const auto* ancestor=m_registry->resolve<Actor>(current);
            if(!ancestor || ancestor->level_id()!=actor->level_id())return false;
            current=ancestor->logical_parent();
        }
        actor->m_logical_parent=parent;return true;
    }

    // ---- spawning ------------------------------------------------------------------
    // Preparation hook, run once per reserved actor after its defaults, root, owner and
    // registration are in place and *before* any `begin_play` of this batch (design.md
    // section 6, between steps 2 and 5). A cooked Level uses it to add the authored
    // components, bind the legacy slot, apply attachment, resolve persistent references
    // and write the property overrides, so an actor's own `begin_play` graph observes
    // its per-instance configuration rather than its class defaults.
    //
    // A function pointer rather than a virtual: a nested batch (a spawn deferred out of
    // a callback and flushed at the end of this one) passes no hook and is therefore
    // never mistaken for a row of the cooked table being loaded.
    using ActorPrepareFn = bool (*)(Level&, Actor&, size_t batch_index);

    // All or nothing. On any failure every reservation of this batch is released, so a
    // half-built actor never keeps orphan components.
    size_t spawn_batch(const ActorSpawnRequest* requests, size_t count, ObjectId* out,
                       ActorPrepareFn prepare = nullptr) {
        if (!m_registry || !requests || !count) return 0;
        ObjectId reserved[level_actor_capacity] = {};
        if (count > level_actor_capacity || m_actor_count + count > level_actor_capacity) {
            ++m_registry->stats.rejected;
            return 0;
        }
        // 1. reserve every instance.
        bool ok = true;
        for (size_t i = 0; i < count && ok; ++i) {
            if (!requests[i].type) { ++m_registry->stats.rejected; ok = false; break; }
            reserved[i] = m_registry->acquire(*requests[i].type);
            if (!reserved[i].valid()) ok = false;
        }
        // 2. defaults: names, owners, roots.
        for (size_t i = 0; i < count && ok; ++i) {
            auto* actor = m_registry->resolve<Actor>(reserved[i]);
            if (!actor) { ok = false; break; }
            actor->m_level = id();
            actor->m_active = requests[i].active;
            actor->m_logical_parent = requests[i].logical_parent;
            actor->set_name(requests[i].name);
            if (!install_default_root(*actor)) { ok = false; break; }
        }
        if (!ok) {
            for (size_t i = 0; i < count; ++i) if (reserved[i].valid()) release_actor_storage(reserved[i]);
            return 0;
        }
        // 3. persistent references resolve to ObjectId in the prepare hook below (4b);
        //    no native class carries one.
        // 4. initialize and register.
        for (size_t i = 0; i < count; ++i) {
            auto* actor = m_registry->resolve<Actor>(reserved[i]);
            set_state(reserved[i], ObjectState::Initialized);
            for (size_t c = 0; c < actor->m_component_count; ++c) set_state(actor->m_components[c], ObjectState::Initialized);
            m_actors[m_actor_count++] = reserved[i];
            if (out) out[i] = reserved[i];
        }
        // 4b. cooked preparation: authored components, legacy slot binding, attachment,
        //     persistent reference resolution and property overrides. Everything below
        //     -- begin_play, on_enable, tick, the scene script -- therefore observes a
        //     fully configured actor.
        if (prepare) {
            for(size_t i=0;i<count && ok;++i) {
                auto* actor=m_registry->resolve<Actor>(reserved[i]);
                ok=actor && prepare(*this,*actor,i);
            }
            if(!ok) {for(size_t i=0;i<count;++i){forget_actor(reserved[i]);release_actor_storage(reserved[i]);if(out)out[i]={};}return 0;}
        }
        // 5. begin_play: components first, then the owning actor, in batch order.
        for (size_t i = 0; i < count; ++i) begin_play_actor(reserved[i]);
        // 6. the scene script begins after the initial level actors.
        begin_play_scene_script();
        flush_pending();
        return count;
    }
    virtual ObjectId spawn_actor(const ClassDescriptor& type, const char* name, ObjectId logical_parent = {}) {
        ActorSpawnRequest request;
        request.type = &type;
        request.name = name;
        request.logical_parent = logical_parent;
        if(m_registry && m_registry->dispatch) {defer_spawn(request);return {};}
        ObjectId result;
        return spawn_batch(&request, 1, &result) ? result : ObjectId{};
    }
    // The scene script is created by the loader; it is never part of the actor table.
    ObjectId create_scene_script(const ClassDescriptor& type, const char* name) {
        if (!m_registry || m_scene_script.valid()) return {};
        const ObjectId id = m_registry->acquire(type);
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor) { if (id.valid()) release_actor_storage(id); return {}; }
        actor->m_level = this->id();
        actor->set_name(name);
        if (!install_default_root(*actor)) { release_actor_storage(id); return {}; }
        set_state(id, ObjectState::Initialized);
        for (size_t c = 0; c < actor->m_component_count; ++c) set_state(actor->m_components[c], ObjectState::Initialized);
        m_scene_script = id;
        return id;
    }

    // ---- destruction ---------------------------------------------------------------
    // A request made inside a callback marks the actor immediately (it receives no further
    // events) and runs the teardown when the current batch finishes.
    bool destroy_actor(ObjectId actor, EndPlayReason reason = EndPlayReason::Destroyed) {
        if (!m_registry) return false;
        auto* instance = m_registry->resolve<Actor>(actor);
        if (!instance || instance->m_doomed) return false;
        instance->m_doomed = true;
        if (m_registry->dispatch) {
            if (m_pending_count >= level_pending_capacity) { ++m_registry->stats.rejected; return false; }
            auto& pending = m_pending[m_pending_count++];
            pending = LevelPendingOp{};
            pending.destroy = true;
            pending.actor = actor;
            pending.reason = reason;
            ++m_registry->stats.deferred;
            return true;
        }
        tear_down_actor(actor, reason);
        flush_pending();
        return true;
    }
    // Exit order: scene script first while the actors are still alive, then every actor.
    void end_play_all(EndPlayReason reason) {
        if (!m_registry) return;
        if (m_scene_script.valid()) {
            tear_down_actor(m_scene_script, reason);
            m_scene_script = ObjectId{};
        }
        while (m_actor_count) tear_down_actor(m_actors[m_actor_count - 1], reason);
        m_pending_count = 0;
    }

    // ---- ticking -------------------------------------------------------------------
    void tick(Fixed delta) {
        if (!m_registry) return;
        ObjectId snapshot[level_actor_capacity] = {};
        const size_t count = m_actor_count;
        for (size_t i = 0; i < count; ++i) snapshot[i] = m_actors[i];
        for (size_t i = 0; i < count; ++i) tick_actor(snapshot[i], delta);
        tick_actor(m_scene_script, delta);
        flush_pending();
    }
    // Once per rendered frame, even while paused (legacy Behaviour parity).
    void frame_update(uint32_t elapsed) {
        if (!m_registry) return;
        ObjectId snapshot[level_actor_capacity] = {};
        const size_t count = m_actor_count;
        for (size_t i = 0; i < count; ++i) snapshot[i] = m_actors[i];
        for (size_t i = 0; i < count; ++i) frame_update_actor(snapshot[i], elapsed);
        frame_update_actor(m_scene_script, elapsed);
        flush_pending();
    }

    // ---- activation ----------------------------------------------------------------
    bool actor_active(const Actor& actor) const {
        const Actor* current = &actor;
        for (size_t depth = 0; current && depth < object_hierarchy_depth; ++depth) {
            if (!current->m_active || current->m_doomed) return false;
            if (!current->m_logical_parent.valid()) return true;
            current = const_cast<ObjectRegistry*>(m_registry)->resolve<Actor>(current->m_logical_parent);
        }
        return current != nullptr;
    }
    bool set_active(ObjectId actor, bool active) {
        if (!m_registry) return false;
        auto* instance = m_registry->resolve<Actor>(actor);
        if (!instance || instance->m_doomed) return false;
        bool before[level_actor_capacity] = {};
        for (size_t i = 0; i < m_actor_count; ++i) {
            auto* other = m_registry->resolve<Actor>(m_actors[i]);
            before[i] = other && actor_active(*other);
        }
        instance->m_active = active;
        if(instance->m_data)instance->m_data->active=active;
        for (size_t i = 0; i < m_actor_count; ++i) {
            auto* other = m_registry->resolve<Actor>(m_actors[i]);
            if (!other || !other->m_begun) continue;
            const bool after = actor_active(*other);
            if (before[i] == after) continue;
            ObjectDispatchScope scope(*m_registry);
            if (after) {
                for (size_t c = 0; c < other->m_component_count; ++c)
                    if (auto* component = m_registry->resolve<ActorComponent>(other->m_components[c])) component->on_enable();
                other->on_enable();
            } else {
                for (size_t c = 0; c < other->m_component_count; ++c)
                    if (auto* component = m_registry->resolve<ActorComponent>(other->m_components[c])) component->on_disable();
                other->on_disable();
            }
        }
        flush_pending();
        return true;
    }

    // ---- components ----------------------------------------------------------------
    template<class T> T* add_component(Actor& owner, const char* name = nullptr) {
        if (!m_registry) return nullptr;
        const auto* type = find_object_class(T::static_class_id);
        const auto* owner_type = find_object_class(owner.class_id());
        if (!type || !owner_type || !accepts_component(owner, *owner_type, *type)) { ++m_registry->stats.rejected; return nullptr; }
        const ObjectId id = m_registry->acquire(*type);
        auto* component = m_registry->resolve<T>(id);
        if (!component) { if (id.valid()) m_registry->release(id); return nullptr; }
        attach_component_record(owner, *component, name);
        set_state(id, ObjectState::Initialized);
        // A component added after the owner began play begins immediately, once.
        if (owner.m_begun) {
            ObjectDispatchScope scope(*m_registry);
            begin_play_component(*component);
            if (actor_active(owner)) component->on_enable();
        }
        return component;
    }
    template<class T> T* get_component(const Actor& owner) const {
        if (!m_registry) return nullptr;
        auto* registry = const_cast<ObjectRegistry*>(m_registry);
        for (size_t i = 0; i < owner.m_component_count; ++i)
            if (auto* component = registry->resolve<T>(owner.m_components[i])) return component;
        return nullptr;
    }
    template<class T> size_t get_components(const Actor& owner, T** out, size_t capacity) const {
        if (!m_registry) return 0;
        auto* registry = const_cast<ObjectRegistry*>(m_registry);
        size_t found = 0;
        for (size_t i = 0; i < owner.m_component_count; ++i)
            if (auto* component = registry->resolve<T>(owner.m_components[i])) {
                if (out && found < capacity) out[found] = component;
                ++found;
            }
        return found;
    }
    bool remove_component(Actor& owner, ObjectId component) {
        if (!m_registry) return false;
        auto* instance = m_registry->resolve<ActorComponent>(component);
        const auto* owner_type = find_object_class(owner.class_id());
        if (!instance || instance->m_owner != owner.id()) { ++m_registry->stats.rejected; return false; }
        // The root of a spatial actor defines its transform contract and cannot be removed.
        if (component == owner.m_root && owner_type && owner_type->domain != ObjectDomain::None) {
            ++m_registry->stats.rejected;
            return false;
        }
        size_t index = actor_component_capacity;
        for (size_t i = 0; i < owner.m_component_count; ++i) if (owner.m_components[i] == component) { index = i; break; }
        if (index == actor_component_capacity) { ++m_registry->stats.rejected; return false; }
        {
            ObjectDispatchScope scope(*m_registry);
            if (owner.m_begun && actor_active(owner)) instance->on_disable();
            end_play_component(*instance, EndPlayReason::Destroyed);
        }
        for (size_t i = index + 1; i < owner.m_component_count; ++i) owner.m_components[i - 1] = owner.m_components[i];
        owner.m_components[--owner.m_component_count] = ObjectId{};
        if (component == owner.m_root) owner.m_root = ObjectId{};
        m_registry->release(component);
        return true;
    }

protected:
    ObjectRegistry* m_registry = nullptr;
    ObjectId m_actors[level_actor_capacity] = {};
    ObjectId m_scene_script;
    size_t m_actor_count = 0;
    LevelPendingOp m_pending[level_pending_capacity] = {};
    size_t m_pending_count = 0;
    bool m_flushing = false;

    void set_state(ObjectId id, ObjectState state) {
        if (auto* slot = m_registry->slot(id)) {
            slot->state = state;
            if (slot->instance) slot->instance->m_state = state;
        }
    }
    bool install_default_root(Actor& actor) {
        const auto* actor_type=find_object_class(actor.class_id());
        if(actor_type && actor_type->default_component && actor_type->default_component_count) {
            if(actor_type->default_component_count>actor_component_capacity)return false;
            for(size_t c=0;c<actor_type->default_component_count;++c) {
                const auto entry=actor_type->default_component(actor,c);
                const auto* type=entry.component?find_object_class(entry.class_id?entry.class_id:entry.component->class_id()):nullptr;
                if(!type || !accepts_component(actor,*actor_type,*type))return false;
                auto id=m_registry->adopt(*entry.component,*type);
                auto* component=m_registry->resolve<ActorComponent>(id);
                if(!component)return false;
                attach_component_record(actor,*component,entry.name);
                if(entry.root) {if(actor.m_root.valid())return false;actor.m_root=id;}
                if(auto* root=m_registry->resolve<SceneComponent3D>(id))root->bind_local();
                if(auto* rect=m_registry->resolve<RectTransformComponent>(id))rect->bind_local();
                if(auto* audio=m_registry->resolve<AudioComponent>(id))audio->bind_local();
            }
            for(size_t c=0;c<actor_type->default_component_count;++c) {
                const auto entry=actor_type->default_component(actor,c);
                if(entry.attach_parent<0)continue;
                if(size_t(entry.attach_parent)>=actor.m_component_count)return false;
                auto* component=m_registry->resolve<ActorComponent>(actor.m_components[c]);
                if(!component || !component->attach_slot())return false;
                *component->attach_slot()=actor.m_components[size_t(entry.attach_parent)];
            }
            return actor_type->domain==ObjectDomain::None || actor.m_root.valid();
        }
        auto* root = actor.default_root();
        if (!root) return true;
        const auto* type = find_object_class(root->class_id());
        if (!type) return false;
        const ObjectId id = m_registry->adopt(*root, *type);
        if (!id.valid()) return false;
        if (auto* scene = m_registry->resolve<SceneComponent3D>(id)) scene->bind_local();
        if (auto* rect = m_registry->resolve<RectTransformComponent>(id)) rect->bind_local();
        root->m_owner = actor.id();
        root->set_name("Root");
        actor.m_root = id;
        actor.m_components[actor.m_component_count++] = id;
        return true;
    }
    bool accepts_component(const Actor& owner, const ClassDescriptor& owner_type, const ClassDescriptor& type) const {
        if (type.family != ObjectFamily::Component || (type.flags & ObjectClassAbstract)) return false;
        if (owner.m_component_count >= actor_component_capacity) return false;
        // Owner domain must be in the component's owners mask.
        const uint8_t bit = object_domain_bit(owner_type.domain);
        if (type.owners_mask && !(type.owners_mask & bit)) return false;
        if (!type.owners_mask && owner_type.domain != ObjectDomain::None) return false;
        if (!(type.flags & ObjectClassMultiple)) {
            auto* registry = const_cast<ObjectRegistry*>(m_registry);
            for (size_t i = 0; i < owner.m_component_count; ++i) {
                const auto* existing = registry->class_of(owner.m_components[i]);
                if (existing && existing->id == type.id) return false;
            }
        }
        return true;
    }
    void attach_component_record(Actor& owner, ActorComponent& component, const char* name) {
        component.m_owner = owner.id();
        component.set_name(name);
        owner.m_components[owner.m_component_count++] = component.id();
    }
    bool order_components(Actor& actor,const ObjectId* ids,size_t count) {
        if(count!=actor.m_component_count)return false;
        for(size_t i=0;i<count;++i) {
            auto* component=m_registry->resolve<ActorComponent>(ids[i]);
            if(!component || component->owner_id()!=actor.id())return false;
            for(size_t j=0;j<i;++j)if(ids[j]==ids[i])return false;
        }
        for(size_t i=0;i<count;++i)actor.m_components[i]=ids[i];
        return true;
    }
    void begin_play_component(ActorComponent& component) {
        if (component.m_begun) return;
        component.m_begun = true;
        set_state(component.id(), ObjectState::Playing);
        component.begin_play();
    }
    void end_play_component(ActorComponent& component, EndPlayReason reason) {
        if (!component.m_begun || component.m_ended) return;
        component.m_ended = true;
        set_state(component.id(), ObjectState::EndingPlay);
        component.end_play(reason);
    }
    void begin_play_actor(ObjectId id) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor || actor->m_begun || actor->m_doomed) return;
        ObjectDispatchScope scope(*m_registry);
        for (size_t c = 0; c < actor->m_component_count; ++c)
            if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) begin_play_component(*component);
        if (actor->m_doomed) return;
        actor->m_begun = true;
        set_state(id, ObjectState::Playing);
        actor->begin_play();
        if (!actor->m_doomed && actor_active(*actor)) {
            for (size_t c = 0; c < actor->m_component_count; ++c)
                if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->on_enable();
            actor->on_enable();
        }
    }
    void begin_play_scene_script() {
        if (!m_scene_script.valid()) return;
        begin_play_actor(m_scene_script);
    }
    void tick_actor(ObjectId id, Fixed delta) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor || actor->m_doomed || !actor->m_begun) return;
        if (!actor_active(*actor)) return;
        ObjectDispatchScope scope(*m_registry);
        for (size_t c = 0; c < actor->m_component_count; ++c) {
            if (actor->m_doomed) return;
            if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->tick(delta);
        }
        if (!actor->m_doomed && actor->m_wants_tick) actor->tick(delta);
    }
    void frame_update_actor(ObjectId id, uint32_t elapsed) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor || actor->m_doomed || !actor->m_begun) return;
        if (!actor_active(*actor)) return;
        ObjectDispatchScope scope(*m_registry);
        for (size_t c = 0; c < actor->m_component_count; ++c) {
            if (actor->m_doomed) return;
            if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->frame_update(elapsed);
        }
    }
    // Releases an unstarted reservation: no gameplay event ever ran on it.
    void release_actor_storage(ObjectId id) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (actor) for (size_t c = 0; c < actor->m_component_count; ++c) m_registry->release(actor->m_components[c]);
        if(actor && actor->data()) {
            auto* data=actor->data();
            data->alive=false;data->active=false;data->owner=nullptr;
            ++data->generation;if(!data->generation)data->generation=1;
        }
        m_registry->release(id);
    }
    // Deactivate, component end_play, actor end_play, then storage.
    void tear_down_actor(ObjectId id, EndPlayReason reason) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor) { forget_actor(id); return; }
        actor->m_doomed = true;
        // Logical children belong to the placement; remove them before releasing
        // their parent so no child retains a dead parent or spatial root.
        ObjectId children[level_actor_capacity]={};size_t child_count=0;
        for(size_t i=0;i<m_actor_count;++i) {
            auto* child=m_registry->resolve<Actor>(m_actors[i]);
            if(child&&child->logical_parent()==id)children[child_count++]=child->id();
        }
        for(size_t i=0;i<child_count;++i)tear_down_actor(children[i],reason);
        if (actor->m_begun && !actor->m_ended) {
            ObjectDispatchScope scope(*m_registry);
            if (actor->m_active) {
                for (size_t c = 0; c < actor->m_component_count; ++c)
                    if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->on_disable();
                actor->on_disable();
            }
            for (size_t c = 0; c < actor->m_component_count; ++c)
                if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) end_play_component(*component, reason);
            actor->m_ended = true;
            set_state(id, ObjectState::EndingPlay);
            actor->end_play(reason);
        }
        for (size_t c = 0; c < actor->m_component_count; ++c) m_registry->release(actor->m_components[c]);
        actor->m_component_count = 0;
        if (auto* data=actor->data()) {
            data->audio.stop(); data->alive=false; data->active=false; data->owner=nullptr;
            ++data->generation; if (!data->generation) data->generation=1;
        }
        m_registry->release(id);
        forget_actor(id);
    }
    void forget_actor(ObjectId id) {
        for (size_t i = 0; i < m_actor_count; ++i) if (m_actors[i] == id) {
            for (size_t j = i + 1; j < m_actor_count; ++j) m_actors[j - 1] = m_actors[j];
            m_actors[--m_actor_count] = ObjectId{};
            return;
        }
    }
    bool defer_spawn(const ActorSpawnRequest& request) {
        if (m_pending_count >= level_pending_capacity) { ++m_registry->stats.rejected; return false; }
        auto& pending = m_pending[m_pending_count++];
        pending = LevelPendingOp{};
        pending.request = request;
        ++m_registry->stats.deferred;
        return true;
    }
    // Runs when the current batch finishes: a deferred spawn never receives the events of
    // the batch it was created in, and a deferred destroy has already stopped dispatching.
    void flush_pending() {
        if (m_flushing || !m_registry || m_registry->dispatch) return;
        m_flushing = true;
        for (size_t round = 0; round < 8 && m_pending_count; ++round) {
            LevelPendingOp queue[level_pending_capacity] = {};
            const size_t count = m_pending_count;
            for (size_t i = 0; i < count; ++i) queue[i] = m_pending[i];
            m_pending_count = 0;
            for (size_t i = 0; i < count; ++i) {
                if (queue[i].destroy) tear_down_actor(queue[i].actor, queue[i].reason);
                else if (queue[i].request.type) {
                    ObjectId created;
                    const ActorSpawnRequest request = queue[i].request;
                    created=spawn_actor(*request.type,request.name,request.logical_parent);
                }
            }
        }
        m_flushing = false;
    }
};

class EPOK_CLASS(Abstract, Family=World, Id="ab2d72b4-fcf6-4b30-8494-43fc0a7cf46c") World : public Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("ab2d72b4-fcf6-4b30-8494-43fc0a7cf46c");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    // Service context. Level/World wiring into main.cpp and the scene banks is a later phase.
    Level* level = nullptr;
    ObjectRegistry* registry = nullptr;
    void bind(Level& value, ObjectRegistry& table) { level = &value; registry = &table; }
};

// ---- out-of-line definitions -------------------------------------------------------
inline Actor* ActorComponent::get_owner() {
    return active_object_registry ? active_object_registry->resolve<Actor>(m_owner) : nullptr;
}
inline bool object_storage_quarantined(Object* instance) {
    if (!instance || !object_class_is_a(instance->class_id(), ActorComponent::static_class_id)) return false;
    return !static_cast<ActorComponent*>(instance)->releasable();
}
// Folds the logical parent chain, exactly like the deactivation path, so audio owned by
// an actor under an inactive parent does not start on begin_play.
inline bool AudioComponent::owner_active() const {
    auto* registry = active_object_registry;
    if (!registry) return false;
    auto* owner = const_cast<AudioComponent*>(this)->get_owner();
    if (!owner) return false;
    auto* level = registry->resolve<Level>(owner->level_id());
    return level ? level->actor_active(*owner) : owner->active();
}
// Collision is owned by the spatial services, not by the Level: they resolve the legacy
// slot to its actor and call this, which fans the event out to the owner's components in
// registration order. Inactive, unstarted and doomed actors receive nothing.
inline size_t dispatch_trigger(Level& level, ObjectId actor, DataHandle other, TriggerPhase phase) {
    auto* registry = level.registry();
    if (!registry) return 0;
    auto* owner = registry->resolve<Actor>(actor);
    if (!owner || !level.actor_active(*owner)) return 0;
    size_t delivered = 0;
    ObjectDispatchScope scope(*registry);
    for (size_t i = 0; i < owner->component_count(); ++i)
        if (auto* component = registry->resolve<ActorComponent>(owner->component_id(i))) {
            // The legacy bindings table wins: a component wrapping a Behaviour that
            // table already notified is skipped, so nothing is delivered twice.
            component->on_trigger(other, phase);
            ++delivered;
        }
    return delivered;
}
inline bool is_active(Object* value) {
    if(!value || !active_object_registry)return false;
    Actor* actor=active_object_registry->resolve<Actor>(value->id());
    if(!actor) {auto* component=active_object_registry->resolve<ActorComponent>(value->id());actor=component?component->get_owner():nullptr;}
    if(!actor)return false;
    auto* level=active_object_registry->resolve<Level>(actor->level_id());
    return level ? level->actor_active(*actor) : actor->active();
}
// Spatial attachment between components of the same domain. Logical actor parenting never
// inherits matrices; only this attachment does.
inline bool attach_component(ObjectId child, ObjectId parent) {
    auto* registry = active_object_registry;
    if (!registry) return false;
    auto* component = registry->resolve<ActorComponent>(child);
    if (!component || !component->attach_slot()) { if (registry) ++registry->stats.rejected; return false; }
    if (!parent.valid()) { *component->attach_slot() = ObjectId{}; return true; }
    auto* target = registry->resolve<ActorComponent>(parent);
    const auto* child_type = registry->class_of(child);
    const auto* parent_type = registry->class_of(parent);
    if (!target || !target->attach_slot() || !child_type || !parent_type ||
        child_type->domain == ObjectDomain::None || child_type->domain != parent_type->domain ||
        child == parent) {
        ++registry->stats.rejected;
        return false;
    }
    // Cycle detection: walking up from the candidate parent must not reach the child.
    ObjectId walker = parent;
    for (size_t depth = 0; walker.valid() && depth < object_hierarchy_depth; ++depth) {
        if (walker == child) { ++registry->stats.rejected; return false; }
        auto* step = registry->resolve<ActorComponent>(walker);
        auto* slot = step ? step->attach_slot() : nullptr;
        if (!slot) break;
        walker = *slot;
    }
    *component->attach_slot() = parent;
    return true;
}
}
