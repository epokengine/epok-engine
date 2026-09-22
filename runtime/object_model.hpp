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
#define EPOK_INCLUDE_FROM_OBJECT_MODEL 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_OBJECT_MODEL
#include <new>
#include <stddef.h>
#include <stdint.h>
#include <type_traits>

#if defined(__clang__) && defined(EPOK_REFLECTION)
// Declarative default component on an actor field: EPOK_COMPONENT(Root, Name="Body").
#define EPOK_COMPONENT(...) __attribute__((annotate("EPOK_COMPONENT:" #__VA_ARGS__)))
#else
#define EPOK_COMPONENT(...)
#endif

namespace epok {
class Object;
class Actor;
class ActorComponent;
class Level;
class World;
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
struct EPOK_VALUE(Id="74e6240c-40ea-40a5-ae85-e54d9d92c7ad") ObjectBatch8 {
    uint32_t total=0,count=0;
    ObjectId item0{},item1{},item2{},item3{},item4{},item5{},item6{},item7{};
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
    // Legacy hand-written tables remain conservative. Cooked tables detect
    // inherited no-op callbacks at compile time, including inherited overrides.
    bool component_tick=true,component_frame=true;
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
    uint16_t pending_releases = 0;
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
        auto* record = slot(id);
        if (!record || !record->instance || !record->type) return nullptr;
        // The cook validates each class family. Base-family queries dominate
        // ticking and activation; they need no virtual call or ancestry walk.
        if constexpr (std::is_same_v<T, Object>) {}
        else if constexpr (std::is_same_v<T, Actor>) {
            if (record->type->family != ObjectFamily::Actor) return nullptr;
        } else if constexpr (std::is_same_v<T, ActorComponent>) {
            if (record->type->family != ObjectFamily::Component) return nullptr;
        } else if constexpr (std::is_same_v<T, Level>) {
            if (record->type->family != ObjectFamily::Level) return nullptr;
        } else if constexpr (std::is_same_v<T, World>) {
            if (record->type->family != ObjectFamily::World) return nullptr;
        } else if (!object_class_is_a(record->type->id, T::static_class_id)) return nullptr;
        return static_cast<T*>(record->instance);
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
        ++pending_releases;
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
        if (dispatch || !pending_releases) return;
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
            --pending_releases;
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
    EPOK_FUNCTION(BlueprintEvent) virtual void tick(Fixed delta_seconds) { (void)delta_seconds; }
    EPOK_FUNCTION(BlueprintEvent) virtual void end_play(EndPlayReason end_play_reason) { (void)end_play_reason; }
    EPOK_FUNCTION(BlueprintEvent) virtual void on_enable() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_disable() {}
    EPOK_FUNCTION(BlueprintEvent, Id="5c57d8df-b3f0-42f9-a46e-77ce16248221") virtual void on_frame(uint32_t frame_microseconds) {(void)frame_microseconds;}
    virtual void frame_update(uint32_t frame_microseconds) {on_frame(frame_microseconds);}
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
    // Reflected identity/hierarchy readers. Every authoring provider (C++, Blueprint,
    // Lua) inherits them from this base instead of bridging a private builtin.
    EPOK_FUNCTION(BlueprintPure, Id="8e2697a0-4267-4923-808b-62acb68d5f3e") ObjectId level_id() const { return m_level; }
    EPOK_FUNCTION(BlueprintPure, Id="d8217460-caff-411c-b7bc-f0ebb34b1c83") ObjectId root_id() const { return m_root; }
    EPOK_FUNCTION(BlueprintPure, Id="146a6de0-328f-4273-bc7e-ba379ff26ad7") ObjectId logical_parent() const { return m_logical_parent; }
    EPOK_FUNCTION(BlueprintPure, Id="3538d7f8-8886-4437-a7ab-41afa18aad93") ObjectId component_id(size_t index) const { return index < actor_component_capacity ? m_components[index] : ObjectId{}; }
    EPOK_FUNCTION(BlueprintPure, Id="3092c8e4-43ff-47f5-ab0c-290ba4685978") size_t component_count() const { return m_component_count; }
    // Self flag only; Level::actor_active() folds the logical parent chain.
    EPOK_FUNCTION(BlueprintPure, Id="66f0ed92-0e29-498a-aff0-1102dd0c9783") bool active() const { return m_active; }
    // Folded activation with the on_enable/on_disable fan-out, and deferred destruction
    // inside a dispatch scope: the same Level paths the legacy Blueprint nodes take.
    EPOK_FUNCTION(BlueprintCallable, Id="6eb5a977-358e-45a2-a8ab-317646d124f2") void set_active(bool active);
    EPOK_FUNCTION(BlueprintCallable, Id="cf849e67-22a1-4838-b593-774689e03195") void destroy();
    EPOK_FUNCTION(BlueprintPure, Id="9771750e-0866-439b-acdc-7c76c271e46e") bool wants_tick() const { return m_wants_tick; }
    EPOK_FUNCTION(BlueprintCallable, Id="ffe58520-ad64-4c77-a2ce-85c0d3ba18dc") void set_wants_tick(bool value) { m_wants_tick = value; }
protected:
    friend class Level;
    char m_name[33] = {};
    ActorData* m_data=nullptr;
    ObjectId m_level, m_root, m_logical_parent;
    ObjectId m_components[actor_component_capacity] = {};
    uint8_t m_component_count = 0;
    // Immutable callback capabilities, refreshed only when component order or
    // ownership changes. Avoid resolving every inert render/collider component
    // just to rediscover that it has no tick or frame callback.
    uint8_t m_tick_components = 0, m_frame_components = 0;
    bool m_actor_tick = true, m_actor_frame = true;
    bool m_active = true;
    bool m_wants_tick = true;
    bool m_begun = false, m_ended = false, m_doomed = false;
};

class EPOK_CLASS(Abstract, Blueprintable, Family=Component, Domain=None, Owners=World3D|World2D|UI, Id="2e5021ee-d14d-4d77-9112-455f29d639d2") ActorComponent : public Object {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("2e5021ee-d14d-4d77-9112-455f29d639d2");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    EPOK_FUNCTION(BlueprintEvent) virtual void begin_play() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void tick(Fixed delta_seconds) { (void)delta_seconds; }
    EPOK_FUNCTION(BlueprintEvent) virtual void end_play(EndPlayReason end_play_reason) { (void)end_play_reason; }
    EPOK_FUNCTION(BlueprintEvent) virtual void on_enable() {}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_disable() {}
    EPOK_FUNCTION(BlueprintEvent, Id="6422355a-f3d5-48c5-b516-22c46dd375c9") virtual void on_frame(uint32_t frame_microseconds) {(void)frame_microseconds;}
    // Runtime hooks, not reflected. frame_update runs once per rendered frame even while
    // paused; attach_slot exposes the spatial parent of components that have one.
    void timeline_sync(uint64_t, bool) override {}
    virtual void blueprint_observe() {}
    virtual void frame_update(uint32_t frame_microseconds) {on_frame(frame_microseconds);}
    virtual ObjectId* attach_slot() { return nullptr; }
    // Forward-only collision hook. The Level does not own collision; the collision
    // service calls dispatch_trigger(Level&, ...) which fans the event out to the
    // owner's components. Nothing in the object model generates trigger events.
    EPOK_FUNCTION(BlueprintEvent, Id="45c82d77-d076-43fe-b947-a4eed670f46b") virtual void trigger_event(ObjectId other,TriggerPhase phase) {(void)other;(void)phase;}
    virtual void on_trigger(DataHandle other, TriggerPhase phase) {(void)other;(void)phase;}
    // False while a service still holds this component's storage; the registry then
    // keeps the (already dead) slot quarantined instead of returning it to the pool.
    virtual bool releasable() const { return true; }
    EPOK_FUNCTION(BlueprintPure, Id="845721eb-1ae9-49f4-b207-607875ce46ef") ObjectId owner_id() const { return m_owner; }
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

// Access-restricted or overloaded user hooks must remain callable through the
// virtual base interface. When detection is ambiguous, conservatively dispatch.
template<class T,class=void> struct ComponentCallbacks {
    static constexpr bool tick=true,frame=true;
};
template<class T> struct ComponentCallbacks<T,std::void_t<decltype(&T::tick),
    decltype(&T::frame_update),decltype(&T::on_frame)>> {
    static constexpr bool tick=!std::is_same_v<decltype(&T::tick),decltype(&ActorComponent::tick)>;
    static constexpr bool frame=!std::is_same_v<decltype(&T::frame_update),decltype(&ActorComponent::frame_update)> ||
        !std::is_same_v<decltype(&T::on_frame),decltype(&ActorComponent::on_frame)>;
};
template<class T,class=void> struct ActorCallbacks {static constexpr bool tick=true,frame=true;};
template<class T> struct ActorCallbacks<T,std::void_t<decltype(&T::tick),
    decltype(&T::frame_update),decltype(&T::on_frame)>> {
    static constexpr bool tick=!std::is_same_v<decltype(&T::tick),decltype(&Actor::tick)>;
    static constexpr bool frame=!std::is_same_v<decltype(&T::frame_update),decltype(&Actor::frame_update)> ||
        !std::is_same_v<decltype(&T::on_frame),decltype(&Actor::on_frame)>;
};

inline bool attach_component(ObjectId child,ObjectId parent);

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
    EPOK_FUNCTION(BlueprintPure, Id="62b05cbc-6533-4ef5-ab4a-10799392885d") Transform local_transform() const {return transform?*transform:local;}
    EPOK_FUNCTION(BlueprintCallable, Id="6401cab7-d0e1-4da9-9fd7-e0f93a9af82f") void set_local_transform(Transform value) {if(transform)*transform=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="d8980f8d-c4be-4124-a3a6-c76b514d0cd3") void set_local_position(Fixed x,Fixed y,Fixed z) {if(transform){transform->position[0]=x;transform->position[1]=y;transform->position[2]=z;}}
    EPOK_FUNCTION(BlueprintCallable, Id="47515dbf-aa6f-4b37-8e4d-cefdfe350da1") void set_local_rotation(Fixed x,Fixed y,Fixed z) {if(transform){transform->rotation[0]=x;transform->rotation[1]=y;transform->rotation[2]=z;}}
    EPOK_FUNCTION(BlueprintCallable, Id="7ea295a2-7bfa-4a52-8ff2-53d521cb6b0c") void set_local_scale(Fixed x,Fixed y,Fixed z) {if(transform){transform->scale[0]=x;transform->scale[1]=y;transform->scale[2]=z;}}
    EPOK_FUNCTION(BlueprintPure, Id="206bc783-8e14-4454-aa80-0477ae7dc20a") WorldAffineSample world_affine() const {return gameplay_world_affine(slot);}
    EPOK_FUNCTION(BlueprintCallable, Id="b54be7db-02d4-46c8-a924-5016d0901e99") bool attach_to(ObjectId parent) {return attach_component(id(),parent);}
    EPOK_FUNCTION(BlueprintCallable, Id="abbb446c-b23c-4b67-925a-034148e4d9c0") void teleport(Fixed x,Fixed y,Fixed z) {set_local_position(x,y,z);reset_motion_interpolation();}
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
    EPOK_FUNCTION(BlueprintCallable, Id="35d86c50-a113-4d96-afb3-13ac190366d6") void set_position(Fixed x,Fixed y) {transform.position[0]=x;transform.position[1]=y;}
    EPOK_FUNCTION(BlueprintCallable, Id="aa6a1bc3-3b47-426e-b637-2df0934b69d4") void set_rotation(Fixed value) {transform.rotation=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="1a8868ba-9090-40f1-ade0-0bdf5d878e71") void set_scale(Fixed x,Fixed y) {transform.scale[0]=x;transform.scale[1]=y;}
    EPOK_FUNCTION(BlueprintPure, Id="954493ae-978f-4396-b6c9-e0e27f97ebbe") Fixed position_x() const {return transform.position[0];}
    EPOK_FUNCTION(BlueprintPure, Id="2bb96472-0dbf-46a4-9c88-6564bd3368d0") Fixed position_y() const {return transform.position[1];}
    EPOK_FUNCTION(BlueprintPure, Id="cf9ed1af-c5e9-4228-b81a-dd27db5f4dc3") Fixed rotation() const {return transform.rotation;}
    EPOK_FUNCTION(BlueprintCallable, Id="6fc034d2-686c-40d7-b447-08d76abfd5ab") bool attach_to(ObjectId parent) {return attach_component(id(),parent);}
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
    EPOK_FUNCTION(BlueprintPure, Id="eb82c905-50c1-42dd-8d7f-df0b1ebc173a") bool enabled() const {return rect&&rect->enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="3c6b4722-b31d-40c5-8f83-0add66ac31c8") void set_enabled(bool value) {if(rect)rect->enabled=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="c5628299-4e0c-4ba6-89a2-8d289865a550") void set_position(Fixed x,Fixed y) {if(rect){rect->position[0]=x;rect->position[1]=y;}}
    EPOK_FUNCTION(BlueprintCallable, Id="45b9ad2e-b86b-464f-b303-c54353bc5f4d") void set_size(Fixed x,Fixed y) {if(rect){rect->size[0]=x;rect->size[1]=y;}}
    EPOK_FUNCTION(BlueprintCallable, Id="1eca293e-859c-4462-bc73-0b67d42ab4d6") void set_anchors(Fixed min_x,Fixed min_y,Fixed max_x,Fixed max_y) {if(rect){rect->anchor_min[0]=min_x;rect->anchor_min[1]=min_y;rect->anchor_max[0]=max_x;rect->anchor_max[1]=max_y;}}
    EPOK_FUNCTION(BlueprintCallable, Id="b1345048-7372-4bd1-b1e1-10ec4c33e7fc") void set_pivot(Fixed x,Fixed y) {if(rect){rect->pivot[0]=x;rect->pivot[1]=y;}}
    EPOK_FUNCTION(BlueprintPure, Id="f21d1b01-6933-433b-a970-4d6c812380b3") Fixed rotation() const {return rect?rect->rotation:Fixed(0.0);}
    // Degrees about the pivot. Layout is unchanged; only the emitted primitives turn.
    EPOK_FUNCTION(BlueprintCallable, Id="f83ea29d-442e-47f0-bf15-c44681a73aac") void set_rotation(Fixed value) {if(rect)rect->rotation=value;}
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
    EPOK_FUNCTION(BlueprintPure, Id="012b5766-939d-4755-b18b-a262cce20486") bool enabled() const { return source&&source->enabled; }
    EPOK_FUNCTION(BlueprintCallable, Id="883763d2-faba-462f-a5e7-cafcc15aa17e") void set_enabled(bool value) {if(source){source->enabled=value;if(!value)source->stop();}}
    EPOK_FUNCTION(BlueprintPure, Id="f5afcdbb-dd4d-45de-9bea-1243707147e5") int32_t clip() const {return source?source->clip:-1;}
    EPOK_FUNCTION(BlueprintCallable, Id="c75782d6-2772-4d29-b864-f1d1790d1d26") void set_clip(int32_t value) {if(source&&source->clip!=value){source->stop();source->clip=value;}}
    EPOK_FUNCTION(BlueprintPure, Id="85028699-06fa-4ace-9bb4-9b4f10f7665e") Fixed volume() const {return source?source->volume:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="9fdfc008-5493-4659-a63a-cba2822602fc") void set_volume(Fixed value) {if(source)source->volume=value<0.0?Fixed(0.0):value>1.0?Fixed(1.0):value;}
    EPOK_FUNCTION(BlueprintPure, Id="cfa327ac-f4a2-43d0-a88d-1881b96a23bc") Fixed pitch() const {return source?source->pitch:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="3b263a70-c72d-41e8-86c7-676d68cb2874") void set_pitch(Fixed value) {if(source)source->pitch=value<0.25?Fixed(0.25):value>4.0?Fixed(4.0):value;}
    EPOK_FUNCTION(BlueprintPure, Id="b597e8ba-58bf-48d6-b7d5-9eca0455b209") uint32_t priority() const {return source?source->priority:0;}
    EPOK_FUNCTION(BlueprintCallable, Id="b3ec45ef-1e8f-4f36-a951-c01b045537a7") void set_priority(uint32_t value) {if(source)source->priority=uint8_t(value>255?255:value);}
    EPOK_FUNCTION(BlueprintPure, Id="6b6c0d55-e8ac-4249-9fe9-4f53ae07784b") bool play_on_start() const {return source&&source->play_on_start;}
    EPOK_FUNCTION(BlueprintCallable, Id="53cb39bd-7174-4fa3-b537-fc924632fe8c") void set_play_on_start(bool value) {if(source)source->play_on_start=value;}
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
    ActorData* entity_slot() const { auto* owner=const_cast<Mesh3DComponent*>(this)->get_owner();return owner?owner->data():nullptr; }
    EPOK_FUNCTION(BlueprintPure, Id="0f7800c8-0ea2-488a-90da-e6a60083edb4") uint32_t vertex_count() const {
        const auto* data=entity_slot();const auto* geometry=data&&data->animator.model?data->animator.model->geometry:data?data->geometry:nullptr;return geometry?uint32_t(geometry->vertex_count):0;
    }
    EPOK_FUNCTION(BlueprintPure, Id="2f1dce0f-f29d-44bd-b8f9-c455540516f8") uint32_t bone_count() const {
        const auto* data=entity_slot();return data&&data->animator.model?uint32_t(data->animator.model->bone_count):0;
    }
    EPOK_FUNCTION(BlueprintPure, Id="45e1ce94-0ed3-4557-a7d5-30bd670d8097") uint32_t clip_count() const {
        const auto* data=entity_slot();return data&&data->animator.model?uint32_t(data->animator.model->clip_count):0;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="047cbafb-5024-44d8-92a3-4939b31de81d") bool play_clip(uint32_t clip,bool looping) {
        auto* data=entity_slot();return data&&clip<=0x7fffffffu&&data->animator.play(int(clip),looping);
    }
    EPOK_FUNCTION(BlueprintCallable, Id="74c0cd75-108d-486c-bccb-68b0a85b1a9a") void pause_animation() {auto* data=entity_slot();if(data)data->animator.pause();}
    EPOK_FUNCTION(BlueprintCallable, Id="a6ef5510-6cb3-418d-be3c-c1e5f4e56776") void resume_animation() {auto* data=entity_slot();if(data)data->animator.resume();}
    EPOK_FUNCTION(BlueprintCallable, Id="a127e39c-84a2-4618-b6e3-b08f8d466ae3") void stop_animation() {auto* data=entity_slot();if(data)data->animator.stop();}
    EPOK_FUNCTION(BlueprintPure, Id="3ea752b4-6bf6-4e45-831c-b05524ecdc46") uint32_t clip_frames(uint32_t clip) const {
        const auto* data=entity_slot();return data&&data->animator.model&&clip<data->animator.model->clip_count?uint32_t(data->animator.model->clips[clip].frames):0;
    }
    // One loop of a clip, measured in the animator's own two-per-frame ticks.
    // Fixed so a manually driven cycle can be advanced by a fractional amount
    // and wrapped against this length without leaving the Fixed vocabulary.
    EPOK_FUNCTION(BlueprintPure, Id="857927dd-9229-44d3-8556-b392f18f3567") Fixed clip_loop_ticks(uint32_t clip) const {
        const uint32_t frames=clip_frames(clip);return Fixed(int32_t((frames>1?frames-1:1)*2),0);
    }
    // Manual playback position, in the same ticks. It is what a game drives
    // when the cycle has to follow something other than real time -- a
    // locomotion blend following ground speed, for instance -- and it pairs
    // with pause_animation() so the animator stops advancing on its own.
    EPOK_FUNCTION(BlueprintCallable, Id="b0b30f1a-a82b-4806-bb2a-eb7392bb6607") void set_animation_position(Fixed ticks) {
        auto* data=entity_slot();if(!data)return;const int32_t value=ticks.raw()/4096;data->animator.ticks=uint32_t(value<0?0:value);
    }
    EPOK_FUNCTION(BlueprintPure, Id="3ec24d35-4613-4ffb-b50a-0ea205e901b1") SkeletalPlaybackState playback_state() const {
        SkeletalPlaybackState result;const auto* data=entity_slot();if(!data||!data->animator.model)return result;
        const auto& animator=data->animator;result.valid=true;result.enabled=animator.enabled;result.playing=animator.playing;result.looping=animator.looping;result.clip=animator.clip;result.ticks=animator.ticks;
        if(animator.clip>=0&&size_t(animator.clip)<animator.model->clip_count){const auto frames=animator.model->clips[animator.clip].frames;const uint32_t length=frames>1?frames-1:1;result.sampled_frame=animator.looping?(animator.ticks/2)%length:(animator.ticks/2<frames?animator.ticks/2:frames?frames-1:0);}return result;
    }
    EPOK_FUNCTION(BlueprintCallable, Capability=skeletal-vertex-query, Id="ac3cac16-6202-4c15-93ed-9a5fb2721ca1") VertexSample sample_vertex(uint32_t vertex,PoseKind pose,CoordinateSpace space) {return skeletal_sample_vertex(entity_slot(),vertex,pose,space);}
    EPOK_FUNCTION(BlueprintCallable, Capability=skeletal-vertex-query, Id="2a2f91f6-3591-46e3-8ced-ea5a41ce565a") VertexSamples4 sample_vertices(VertexIndexBatch4 indices,PoseKind pose,CoordinateSpace space) {return skeletal_sample_vertices(entity_slot(),indices,pose,space);}
    EPOK_FUNCTION(BlueprintCallable, Capability=skeletal-bone-query, Id="7c6e66dd-df5c-4195-adb0-e66c7ee4587b") BoneSample sample_bone(uint32_t bone,PoseKind pose,CoordinateSpace space) {return skeletal_sample_bone(entity_slot(),bone,pose,space);}
    EPOK_FUNCTION(BlueprintPure, Id="d186f5be-bb2d-42cb-936c-e792a7be6247") MaterialSnapshot material_state() const {
        MaterialSnapshot result;const auto* data=entity_slot();if(!data)return result;const auto& value=data->material;result.valid=true;result.unlit=value.unlit;result.red=value.color[0];result.green=value.color[1];result.blue=value.color[2];result.texture=uint32_t(value.texture);result.blend=uint32_t(value.blend);result.depth_bias=value.depth_bias;result.uv_x=Fixed(value.uv_scroll[0],Fixed::RAW);result.uv_y=Fixed(value.uv_scroll[1],Fixed::RAW);return result;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="e3a012fc-ee1e-4201-b22e-c0159df503f7") void set_material_color(uint32_t red,uint32_t green,uint32_t blue) {auto* data=entity_slot();if(data){data->material.color[0]=uint8_t(red>255?255:red);data->material.color[1]=uint8_t(green>255?255:green);data->material.color[2]=uint8_t(blue>255?255:blue);}}
    EPOK_FUNCTION(BlueprintCallable, Id="97078826-e1ef-4ec4-9d24-33151d2f68b9") void set_material_texture(int32_t texture) {auto* data=entity_slot();if(data)data->material.texture=texture;}
    EPOK_FUNCTION(BlueprintCallable, Id="99f6b202-f672-49fb-bea8-3778b5b26b0c") void set_material_unlit(bool unlit) {auto* data=entity_slot();if(data)data->material.unlit=unlit;}
    EPOK_FUNCTION(BlueprintCallable, Id="ad3a142b-e4b7-4e19-b35a-ed0a57fae64a") void set_material_blend(BlendMode blend) {auto* data=entity_slot();if(data)data->material.blend=blend;}
    EPOK_FUNCTION(BlueprintCallable, Id="6f111802-066c-44aa-bd5b-c63ec08b45cb") void set_material_depth_bias(int32_t value) {auto* data=entity_slot();if(data)data->material.depth_bias=int16_t(value<-32768?-32768:value>32767?32767:value);}
    EPOK_FUNCTION(BlueprintCallable, Id="736a9982-99f0-45f7-ad00-bc938a3c1786") void set_uv_scroll(Fixed x,Fixed y) {auto* data=entity_slot();if(data){data->material.uv_scroll[0]=x.raw();data->material.uv_scroll[1]=y.raw();}}
    EPOK_FUNCTION(BlueprintPure, Id="29d828aa-a390-443b-9628-36a8661f76d0") uint32_t quad_count() const {const auto* data=entity_slot();const auto* geometry=data&&data->animator.model?data->animator.model->geometry:data?data->geometry:nullptr;return geometry?uint32_t(geometry->quad_count):0;}
    EPOK_FUNCTION(BlueprintPure, Id="c476252f-20d0-47aa-bda9-a0d09f490b4d") bool streamed() const {const auto* data=entity_slot();const auto* geometry=data&&data->animator.model?data->animator.model->geometry:data?data->geometry:nullptr;return geometry&&geometry->stream_page!=0xffffffffu;}
    EPOK_FUNCTION(BlueprintPure, Capability=mesh-streaming, Id="a9a09e6d-2393-41f9-abfa-5cd51b484983") MeshDataState geometry_state() const {const auto* data=entity_slot();return mesh_geometry_state(data?data->geometry:nullptr);}
    EPOK_FUNCTION(BlueprintCallable, Capability=mesh-streaming, AsyncRequest, Id="ca928c49-6773-433a-8fbf-94979698ab40") bool request_geometry() {const auto* data=entity_slot();return request_mesh_geometry(data?data->geometry:nullptr);}
    EPOK_FUNCTION(BlueprintCallable, Capability=mesh-streaming, Id="2a9adf07-389e-4455-8d45-a1d87ae5f32f") MeshVertexSample sample_geometry_vertex(uint32_t vertex,CoordinateSpace space) {return sample_mesh_vertex(entity_slot(),vertex,space);}
    EPOK_FUNCTION(BlueprintPure, Id="a120a95e-3961-4855-b058-1b63e98f1213") bool lighting_enabled() const {const auto* data=entity_slot();return data&&data->lighting.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="3e7046c9-fe54-403e-b82d-2c4da8ec6946") void set_lighting_enabled(bool value) {auto* data=entity_slot();if(data)data->lighting.enabled=value;}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=sprite, Id="0b68656b-364b-441f-ad86-ddc0408e80a0") Sprite3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("0b68656b-364b-441f-ad86-ddc0408e80a0");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<Sprite3DComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="cbcb2992-f37c-4bb2-b099-0843ebbc81a8") SpritePlaybackState playback_state() const {SpritePlaybackState result;const auto* data=entity_slot();if(!data)return result;const auto& animator=data->sprite_animator;result.valid=true;result.enabled=animator.enabled;result.playing=animator.playing;result.completed=animator.completed;result.clip=animator.clip;result.frame=animator.frame;result.clip_count=animator.clip_count;result.pending_events=animator.event_count;result.dropped_events=animator.dropped_events;return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="2d45b9ac-969a-4ee1-8afa-c2516c4e48f5") bool play_clip(uint32_t clip) {auto* data=entity_slot();return data&&clip<=0xffffu&&data->sprite_animator.play(uint16_t(clip));}
    EPOK_FUNCTION(BlueprintCallable, Id="b22cf961-2e2f-49ef-ab17-460779926861") void pause_animation() {auto* data=entity_slot();if(data)data->sprite_animator.pause();}
    EPOK_FUNCTION(BlueprintCallable, Id="5ffc422d-bc57-4bb4-9909-615533655743") void resume_animation() {auto* data=entity_slot();if(data)data->sprite_animator.resume();}
    EPOK_FUNCTION(BlueprintCallable, Id="3fc41b61-85c1-4a93-a3d3-50f553028e63") uint32_t poll_event() {auto* data=entity_slot();uint16_t event=0;return data&&data->sprite_animator.poll_event(event)?event:0;}
    EPOK_FUNCTION(BlueprintCallable, Id="f5b6ff77-26f8-4142-92f7-e27ba8e5f646") bool take_completion() {auto* data=entity_slot();return data&&data->sprite_animator.take_completion();}
    EPOK_FUNCTION(BlueprintPure, Id="67c2aaf1-400d-46ef-9629-50e5bde990e8") bool enabled() const {const auto* data=entity_slot();return data&&data->sprite.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="fd1516be-0dc8-4812-b70f-5b707df972b6") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->sprite.enabled=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="49748389-94aa-4a76-ab76-1bbc63836c3c") void set_texture(int32_t value) {auto* data=entity_slot();if(data)data->sprite.texture=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="e0a080ba-34ee-44e6-ae72-598503e23060") void set_size(Fixed x,Fixed y) {auto* data=entity_slot();if(data){data->sprite.size[0]=x;data->sprite.size[1]=y;}}
    EPOK_FUNCTION(BlueprintCallable, Id="7dc105f5-e92e-4e71-a53a-19d99c069dde") void set_flip(bool x,bool y) {auto* data=entity_slot();if(data){data->sprite.flip_x=x;data->sprite.flip_y=y;}}
    EPOK_FUNCTION(BlueprintCallable, Id="b7cbde1f-44f3-4480-8acd-4cfe934f4397") void set_color(uint32_t red,uint32_t green,uint32_t blue) {auto* data=entity_slot();if(data){data->sprite.color[0]=uint8_t(red>255?255:red);data->sprite.color[1]=uint8_t(green>255?255:green);data->sprite.color[2]=uint8_t(blue>255?255:blue);}}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=camera, Id="9fe2abff-f285-435d-976d-825c5db5420a") Camera3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("9fe2abff-f285-435d-976d-825c5db5420a");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<Camera3DComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="c847011b-6678-416b-8821-b6e7462e2bf8") bool enabled() const {const auto* data=entity_slot();return data&&(data->camera||data->camera_settings.enabled);}
    EPOK_FUNCTION(BlueprintCallable, Id="d0e959f6-f593-40a8-ab06-350478b79a68") void set_enabled(bool value) {auto* data=entity_slot();if(data){data->camera=value;data->camera_settings.enabled=value;}}
    EPOK_FUNCTION(BlueprintPure, Id="ac1b7ccf-5f66-4262-a68e-c8ad425cc175") Fixed field_of_view() const {const auto* data=entity_slot();return data?data->camera_settings.field_of_view:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="69851a74-a83c-4f4f-8b15-c8faf9b76dd4") void set_field_of_view(Fixed value) {auto* data=entity_slot();if(data)data->camera_settings.field_of_view=value<1.0?Fixed(1.0):value>179.0?Fixed(179.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="7f08b9a7-3873-4f45-aae0-b11a78c59257") bool make_active() {return set_active_camera(entity_slot());}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=light, Id="e18d296c-5820-4557-9386-8b12b9ca37a2") Light3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("e18d296c-5820-4557-9386-8b12b9ca37a2");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<Light3DComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="93482c0f-d8fd-40c3-b506-38686fd21a36") bool enabled() const {const auto* data=entity_slot();return data&&data->light.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="c39fab3a-d50e-4f4b-84e7-089805c15d49") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->light.enabled=value;}
    EPOK_FUNCTION(BlueprintPure, Id="3020a7cf-2b91-4d65-916f-86fb22145016") Fixed intensity() const {const auto* data=entity_slot();return data?data->light.intensity:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="4caf5ff0-af40-4f31-884c-47efdb4956cc") void set_intensity(Fixed value) {auto* data=entity_slot();if(data)data->light.intensity=value<0.0?Fixed(0.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="25f782ce-f53b-4eb2-a3bc-b6cf3cf6d8aa") void set_range(Fixed value) {auto* data=entity_slot();if(data)data->light.range=value<0.0?Fixed(0.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="7af83e00-f5a8-488f-af89-504b74e7e372") void set_color(uint32_t red,uint32_t green,uint32_t blue) {auto* data=entity_slot();if(data){data->light.color[0]=uint8_t(red>255?255:red);data->light.color[1]=uint8_t(green>255?255:green);data->light.color[2]=uint8_t(blue>255?255:blue);}}
    EPOK_FUNCTION(BlueprintCallable, Id="00eb6cb5-b5fb-465b-8aa3-056adf6ad038") void set_type(LightType value) {auto* data=entity_slot();if(data)data->light.type=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="c6acdbff-6961-48ea-afda-f5f36733d50a") void set_mode(LightMode value) {auto* data=entity_slot();if(data)data->light.mode=value;}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=collider, Id="e8431f94-e526-4d7d-aace-8c8fae7955e6") Collider3DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("e8431f94-e526-4d7d-aace-8c8fae7955e6");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<Collider3DComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="a794af03-7f2c-4de2-8211-cfcbe0aa05e4") bool enabled() const {const auto* data=entity_slot();return data&&data->collider.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="67d0ff7c-9d3d-440f-acba-5331147adb09") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->collider.enabled=value;}
    EPOK_FUNCTION(BlueprintPure, Id="9c100782-860e-428a-8b15-5b7e89716b1b") bool trigger() const {const auto* data=entity_slot();return data&&data->collider.trigger;}
    EPOK_FUNCTION(BlueprintCallable, Id="ed541c10-273f-4d93-a12a-c64234837b54") void set_trigger(bool value) {auto* data=entity_slot();if(data)data->collider.trigger=value;}
    EPOK_FUNCTION(BlueprintPure, Id="e282e657-8d99-4b9a-9681-136c4661e72d") uint32_t layer() const {const auto* data=entity_slot();return data?data->collider.layer:0;}
    EPOK_FUNCTION(BlueprintCallable, Id="7170c50e-054d-46f8-ae7c-f820407745f4") void set_layer(uint32_t value) {auto* data=entity_slot();if(data)data->collider.layer=value;}
    EPOK_FUNCTION(BlueprintPure, Id="79415044-4a35-47a7-9789-a4a19272c796") uint32_t mask() const {const auto* data=entity_slot();return data?data->collider.mask:0;}
    EPOK_FUNCTION(BlueprintCallable, Id="54229688-e3fd-433a-b111-a9218291ab78") void set_mask(uint32_t value) {auto* data=entity_slot();if(data)data->collider.mask=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="14cacb6a-ff12-43b3-b918-909c6613735b") void set_center(Fixed x,Fixed y,Fixed z) {auto* data=entity_slot();if(data){data->collider.center[0]=x;data->collider.center[1]=y;data->collider.center[2]=z;}}
    EPOK_FUNCTION(BlueprintCallable, Id="9336d04c-300a-4bda-b9a6-301e775277d0") void set_half_extents(Fixed x,Fixed y,Fixed z) {auto* data=entity_slot();if(data){data->collider.half_extents[0]=x<0.0?-x:x;data->collider.half_extents[1]=y<0.0?-y:y;data->collider.half_extents[2]=z<0.0?-z:z;}}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=canvas, Id="f2cfb26b-af53-4a46-9d91-1debea72e01b") CanvasComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("f2cfb26b-af53-4a46-9d91-1debea72e01b");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<CanvasComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="00aeb910-bc8e-41cd-9a73-0fe2eef92a29") bool enabled() const {const auto* data=entity_slot();return data&&data->canvas.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="45fd9028-92b6-463b-bd03-f34787624b73") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->canvas.enabled=value;}
    // The focused element, as an actor index into the scene's objects array, or
    // -1. Authored initial focus and runtime focus are this one field.
    EPOK_FUNCTION(BlueprintPure, Id="6c7c7c84-562b-438c-a4a0-ea89dbd7e8aa") int32_t focused() const {const auto* data=entity_slot();return data?int32_t(data->canvas.focused):-1;}
    EPOK_FUNCTION(BlueprintCallable, Id="06a3f570-b233-4740-bb87-17fd87a4a52b") void set_focused(int value) {auto* data=entity_slot();if(data)data->canvas.focused=int16_t(value<-1?-1:value>32767?32767:value);}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=image, Id="d04c78d6-23bd-40d7-88f1-b1afc1b05b5b") ImageComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("d04c78d6-23bd-40d7-88f1-b1afc1b05b5b");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<ImageComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="8701ea5a-0d4e-43bc-af6b-691bddcfd82d") bool enabled() const {const auto* data=entity_slot();return data&&data->image.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="3f06fcef-66ca-48f1-8883-ea12d42d786c") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->image.enabled=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="39996407-2091-4a70-b2e1-2bbf508f77e9") void set_texture(int32_t value) {auto* data=entity_slot();if(data)data->image.texture=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="7d9da0a9-2ae2-403b-9f40-5061bfd12e47") void set_color(uint32_t red,uint32_t green,uint32_t blue) {auto* data=entity_slot();if(data){data->image.color[0]=uint8_t(red>255?255:red);data->image.color[1]=uint8_t(green>255?255:green);data->image.color[2]=uint8_t(blue>255?255:blue);}}
    EPOK_FUNCTION(BlueprintCallable, Id="d4550792-9e1d-4454-b65d-1b4b86d04880") void set_region(uint32_t x,uint32_t y,uint32_t width,uint32_t height) {auto* data=entity_slot();if(data){data->image.region[0]=uint16_t(x>65535?65535:x);data->image.region[1]=uint16_t(y>65535?65535:y);data->image.region[2]=uint16_t(width>65535?65535:width);data->image.region[3]=uint16_t(height>65535?65535:height);}}
    // Nine-slice borders in source pixels: left, top, right, bottom.
    EPOK_FUNCTION(BlueprintCallable, Id="afbae136-9f2c-4c7f-b845-a9cbdd225530") void set_borders(int left,int top,int right,int bottom) {auto* data=entity_slot();if(!data)return;const int values[4]={left,top,right,bottom};for(int i=0;i<4;++i)data->image.borders[i]=uint16_t(values[i]<0?0:values[i]>65535?65535:values[i]);}
    EPOK_FUNCTION(BlueprintPure, Id="7d9e1551-488f-446b-8ac5-8dea96b96fee") int32_t tiling() const {const auto* data=entity_slot();return data?int32_t(data->image.tiling):0;}
    // 0 None, 1 Tile, 2 TileFit; anything else leaves the image stretched.
    EPOK_FUNCTION(BlueprintCallable, Id="e0c70a72-1587-4c3e-bfc1-8835c634162f") void set_tiling(int value) {auto* data=entity_slot();if(data)data->image.tiling=value==1?ImageTiling::Tile:value==2?ImageTiling::TileFit:ImageTiling::None;}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=text, Id="fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34") TextComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<TextComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="75d0c7d3-e56e-43cd-a5e2-476c1265b2b7") bool enabled() const {const auto* data=entity_slot();return data&&data->text.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="e687c218-94b4-4db8-ad06-7a14cfec7ad9") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->text.enabled=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="60b81db3-178c-4639-95ce-bfc9b9c47847") void set_number(int32_t value) {auto* data=entity_slot();if(!data)return;char text[16]={};char reversed[16]={};uint32_t magnitude=value<0?uint32_t(-int64_t(value)):uint32_t(value);uint32_t count=0;do{reversed[count++]=char('0'+magnitude%10);magnitude/=10;}while(magnitude&&count<15);uint32_t out=0;if(value<0)text[out++]='-';while(count)text[out++]=reversed[--count];text[out]=0;data->text.set_text(text);}
    EPOK_FUNCTION(BlueprintCallable, Id="9d40fb20-8d06-4ea1-afba-5c743418b301") void set_unsigned(uint32_t value) {auto* data=entity_slot();if(!data)return;char text[16]={};char reversed[16]={};uint32_t count=0;do{reversed[count++]=char('0'+value%10);value/=10;}while(value&&count<15);uint32_t out=0;while(count)text[out++]=reversed[--count];text[out]=0;data->text.set_text(text);}
    EPOK_FUNCTION(BlueprintCallable, Id="f8757d62-0492-45fe-a27f-0a018532f842") void clear_text() {auto* data=entity_slot();if(data)data->text.value[0]=0;}
    EPOK_FUNCTION(BlueprintCallable, Id="cbcc41a3-936a-49c0-b55e-12dbbe869c7e") bool set_text_word(uint32_t index,uint32_t packed) {auto* data=entity_slot();if(!data||index>=128)return false;char* output=data->text.value+index*4;for(uint32_t i=0;i<4;++i){const uint8_t value=uint8_t(packed>>(i*8));output[i]=value&&value<128?char(value):value?'?':0;}data->text.value[511]=0;return true;}
    EPOK_FUNCTION(BlueprintPure, Id="694cc674-49fb-4c9f-8851-b655b1b8fd45") uint32_t text_word(uint32_t index) const {const auto* data=entity_slot();if(!data||index>=128)return 0;uint32_t result=0;for(uint32_t i=0;i<4;++i)result|=uint32_t(uint8_t(data->text.value[index*4+i]))<<(i*8);return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="0e61f050-7634-473f-a3bf-85d4067af329") void set_color(uint32_t red,uint32_t green,uint32_t blue) {auto* data=entity_slot();if(data){data->text.color[0]=uint8_t(red>255?255:red);data->text.color[1]=uint8_t(green>255?255:green);data->text.color[2]=uint8_t(blue>255?255:blue);}}
    EPOK_FUNCTION(BlueprintCallable, Id="bdcfb111-a742-4059-96a7-d10dd8182c88") void set_wrap(bool value) {auto* data=entity_slot();if(data)data->text.wrap=value;}
    EPOK_FUNCTION(BlueprintPure, Id="26daebdb-28d8-496c-87ee-f4d4bf80c839") int32_t align() const {const auto* data=entity_slot();return data?int32_t(data->text.align):0;}
    // 0 Left, 1 Center, 2 Right; anything else leaves the lines left-aligned.
    EPOK_FUNCTION(BlueprintCallable, Id="c2a8c760-4d90-4e2e-8900-a747a7065366") void set_align(int value) {auto* data=entity_slot();if(data)data->text.align=value==1?TextAlign::Center:value==2?TextAlign::Right:TextAlign::Left;}
    EPOK_FUNCTION(BlueprintPure, Id="cd1ac0c4-58af-4241-b0df-fdc8ab8c5ebc") int32_t font_index() const {const auto* data=entity_slot();return data?int32_t(data->text.font):-1;}
    // The cooked font this label draws from, in export order; -1 is the
    // built-in 8x16 atlas. An index no font answers falls back to the built-in.
    EPOK_FUNCTION(BlueprintCallable, Id="4fc0a479-e9e3-4042-ae30-4559eace268b") void set_font_index(int value) {auto* data=entity_slot();if(data)data->text.font=value<0?-1:value;}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=progress, Id="28bf5245-5d80-4cba-a77d-1d74479ac276") ProgressBarComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("28bf5245-5d80-4cba-a77d-1d74479ac276");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<ProgressBarComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="243e056c-8774-476a-94c1-f9381299f418") bool enabled() const {const auto* data=entity_slot();return data&&data->progress.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="9b695075-9675-4dd1-9097-14c1616c51aa") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->progress.enabled=value;}
    EPOK_FUNCTION(BlueprintPure, Id="77e4c531-5d42-470c-954e-937d5204f664") Fixed value() const {const auto* data=entity_slot();return data?data->progress.value:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="d9ce66cc-cf09-4e8d-b76c-a1793dd980e0") void set_value(Fixed value) {auto* data=entity_slot();if(data)data->progress.value=value<0.0?Fixed(0.0):value>1.0?Fixed(1.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="0afbb6fa-16ca-455b-aa47-5cc6cb719527") void set_colors(uint32_t red,uint32_t green,uint32_t blue,uint32_t background_red,uint32_t background_green,uint32_t background_blue) {auto* data=entity_slot();if(data){data->progress.color[0]=uint8_t(red>255?255:red);data->progress.color[1]=uint8_t(green>255?255:green);data->progress.color[2]=uint8_t(blue>255?255:blue);data->progress.background[0]=uint8_t(background_red>255?255:background_red);data->progress.background[1]=uint8_t(background_green>255?255:background_green);data->progress.background[2]=uint8_t(background_blue>255?255:background_blue);}}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=layout, Id="c1eae9de-bc3d-4fea-9b85-ddbc8aeb8bb1") LayoutElementComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("c1eae9de-bc3d-4fea-9b85-ddbc8aeb8bb1");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<LayoutElementComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="fc293045-6c2d-4027-8079-eee4b7ffd19e") bool enabled() const {const auto* data=entity_slot();return data&&data->layout_element.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="f014afb1-7b58-402f-9b40-dabd4095a375") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->layout_element.enabled=value;}
    EPOK_FUNCTION(BlueprintPure, Id="eca949ba-094f-4d9f-8201-1ad05f9d547b") int32_t horizontal_flags() const {const auto* data=entity_slot();return data?int32_t(data->layout_element.horizontal):0;}
    EPOK_FUNCTION(BlueprintCallable, Id="17dd57e1-9df3-4e5d-b265-a3d0eff0f8c2") void set_horizontal_flags(int32_t value) {auto* data=entity_slot();if(data)data->layout_element.horizontal=uint8_t(value<0?0:value>255?255:value);}
    EPOK_FUNCTION(BlueprintPure, Id="3a8d2c1e-96b2-46fb-8b6f-20e0df8cb1e8") int32_t vertical_flags() const {const auto* data=entity_slot();return data?int32_t(data->layout_element.vertical):0;}
    EPOK_FUNCTION(BlueprintCallable, Id="46451cc1-fb1b-4c2d-b90f-4f29dfd18ecf") void set_vertical_flags(int32_t value) {auto* data=entity_slot();if(data)data->layout_element.vertical=uint8_t(value<0?0:value>255?255:value);}
    EPOK_FUNCTION(BlueprintPure, Id="fdbb5ebf-f358-4c29-b922-0014ec72a113") Fixed minimum_width() const {const auto* data=entity_slot();return data?data->layout_element.minimum[0]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintPure, Id="79059823-26f2-4f4a-8555-cafcacf39897") Fixed minimum_height() const {const auto* data=entity_slot();return data?data->layout_element.minimum[1]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="5afa583a-6bcf-40e9-b141-d82faa8f27c3") void set_minimum(Fixed width,Fixed height) {auto* data=entity_slot();if(data){data->layout_element.minimum[0]=width<0.0?Fixed(0.0):width;data->layout_element.minimum[1]=height<0.0?Fixed(0.0):height;}}
    EPOK_FUNCTION(BlueprintPure, Id="d012d9ea-a6ab-44d5-a774-df8ec13efa7e") Fixed stretch() const {const auto* data=entity_slot();return data?data->layout_element.stretch:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="89587c7a-96f4-4729-885c-3d6bbecd72d8") void set_stretch(Fixed value) {auto* data=entity_slot();if(data)data->layout_element.stretch=value<0.0?Fixed(0.0):value;}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=layout, Id="0215c00b-c65f-49d5-924e-ac205dd0c8e0") LayoutContainerComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("0215c00b-c65f-49d5-924e-ac205dd0c8e0");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<LayoutContainerComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="179dd638-cfaf-42d9-925e-28fcc2fa4c8d") bool enabled() const {const auto* data=entity_slot();return data&&data->layout_container.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="4600df3a-968b-4537-a752-0ec56455e093") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->layout_container.enabled=value;}
    EPOK_FUNCTION(BlueprintPure, Id="c13f53c0-f45d-4a81-b057-582839522c10") int32_t kind() const {const auto* data=entity_slot();return data?int32_t(data->layout_container.kind):0;}
    EPOK_FUNCTION(BlueprintCallable, Id="c0604b5f-8010-49b3-a83f-7597b1f5cfa2") void set_kind(LayoutKind value) {auto* data=entity_slot();if(data)data->layout_container.kind=value;}
    EPOK_FUNCTION(BlueprintPure, Id="29b86b2c-cd4d-4a9c-aee4-0197bfc8ddbb") Fixed spacing_x() const {const auto* data=entity_slot();return data?data->layout_container.spacing[0]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintPure, Id="6e3951d7-315b-4765-b1d2-1835d2f0f708") Fixed spacing_y() const {const auto* data=entity_slot();return data?data->layout_container.spacing[1]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="01e8a277-95d7-49dd-8ac0-8e33276df388") void set_spacing(Fixed x,Fixed y) {auto* data=entity_slot();if(data){data->layout_container.spacing[0]=x<0.0?Fixed(0.0):x;data->layout_container.spacing[1]=y<0.0?Fixed(0.0):y;}}
    EPOK_FUNCTION(BlueprintPure, Id="51a0e4ba-11c5-44c8-aad1-cfe098031ddc") Fixed padding_left() const {const auto* data=entity_slot();return data?data->layout_container.padding[0]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintPure, Id="a8de539e-567d-4a85-afe0-285a58d66c9a") Fixed padding_top() const {const auto* data=entity_slot();return data?data->layout_container.padding[1]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintPure, Id="11ff649d-92ce-4795-8ba5-20f48e0b6c6a") Fixed padding_right() const {const auto* data=entity_slot();return data?data->layout_container.padding[2]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintPure, Id="da44743b-fd56-4ac4-8d01-6a5487c56039") Fixed padding_bottom() const {const auto* data=entity_slot();return data?data->layout_container.padding[3]:Fixed(0.0);}
    EPOK_FUNCTION(BlueprintCallable, Id="c548107e-9579-42f9-8f0c-ee0d99144a7d") void set_padding(Fixed left,Fixed top,Fixed right,Fixed bottom) {auto* data=entity_slot();if(!data)return;const Fixed values[4]={left,top,right,bottom};for(int i=0;i<4;++i)data->layout_container.padding[i]=values[i]<0.0?Fixed(0.0):values[i];}
    EPOK_FUNCTION(BlueprintPure, Id="69869fdd-afa9-4661-9ade-b5f30385b5d0") int32_t columns() const {const auto* data=entity_slot();return data?int32_t(data->layout_container.columns):0;}
    EPOK_FUNCTION(BlueprintCallable, Id="c3cd2262-8a8b-42c7-b3bc-3e4761cc7098") void set_columns(int32_t value) {auto* data=entity_slot();if(data)data->layout_container.columns=uint8_t(value<1?1:value>255?255:value);}
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=focus, Id="63836cf7-47ce-4174-ad2c-6b3b4266414d") FocusableComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("63836cf7-47ce-4174-ad2c-6b3b4266414d");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<FocusableComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="ad4b5019-074b-49cc-8e05-746012c25281") bool enabled() const {const auto* data=entity_slot();return data&&data->focusable.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="0783fbe7-7ba7-4159-a0c5-c6e657bae03d") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->focusable.enabled=value;}
    // Direction 0 left, 1 right, 2 up, 3 down; the value is an actor index into
    // the scene's objects array, or -1 for no neighbour that way.
    EPOK_FUNCTION(BlueprintPure, Id="a2806533-3f89-4ccb-a434-79c99bcc182e") int32_t neighbor(int dir) const {const auto* data=entity_slot();return data&&dir>=0&&dir<4?int32_t(data->focusable.neighbors[dir]):-1;}
    EPOK_FUNCTION(BlueprintCallable, Id="e67141fb-61f8-4444-af85-0037e7b150cf") void set_neighbor(int dir,int actor) {auto* data=entity_slot();if(data&&dir>=0&&dir<4)data->focusable.neighbors[dir]=int16_t(actor<-1?-1:actor>32767?32767:actor);}
    EPOK_FUNCTION(BlueprintPure, Id="3cab80d1-3b2b-4007-a1b4-ce73582dfd00") int32_t order() const {const auto* data=entity_slot();return data?int32_t(data->focusable.order):0;}
    EPOK_FUNCTION(BlueprintCallable, Id="e487a3bb-b41f-4f3e-9654-fd8690b91000") void set_order(int value) {auto* data=entity_slot();if(data)data->focusable.order=uint8_t(value<0?0:value>255?255:value);}
    EPOK_FUNCTION(BlueprintCallable, Id="b1a87ee4-d338-4d13-b97b-5912a5c8d526") void set_highlight(int red,int green,int blue) {auto* data=entity_slot();if(!data)return;const int values[3]={red,green,blue};for(int i=0;i<3;++i)data->focusable.highlight[i]=uint8_t(values[i]<0?0:values[i]>255?255:values[i]);}
    // True while the canvas above this element points at it. Focus lives on the
    // canvas, so the answer is a walk up the slot parents, not a flag here.
    EPOK_FUNCTION(BlueprintPure, Id="6beb822a-bfd3-4152-bd2d-2825eb42c9fd") bool is_focused() const {
        const auto* data=entity_slot();const int self=entity_index(data);if(self<0)return false;
        // Slots are contiguous, so the array base is this slot minus its index.
        const ActorData* base=data-self;
        for(int current=self,depth=0;current>=0&&depth<33;++depth){
            const auto& node=base[current];
            if(node.canvas.enabled)return node.canvas.focused==self;
            current=node.parent;
        }
        return false;
    }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=particles, Id="1d067605-c408-40b8-b2c2-718b8cf0c601") ParticleEmitterComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c601");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<ParticleEmitterComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="30e02f03-0c0d-4875-85e1-3423af5156fd") ParticleEmitterState state() const {ParticleEmitterState result;const auto* data=entity_slot();if(!data)return result;const auto& emitter=data->particle_emitter;result.valid=true;result.enabled=emitter.enabled;result.playing=emitter.playing;result.continuous=emitter.continuous;result.pending=emitter.pending;result.max_particles=emitter.max_particles;return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="e2f09d77-b27c-42a1-8e2c-8b3138819d34") void play() {auto* data=entity_slot();if(data)data->particle_emitter.play();}
    EPOK_FUNCTION(BlueprintCallable, Id="a5fac700-06a4-4dd9-a6a2-ac9b920c2dac") void stop() {auto* data=entity_slot();if(data)data->particle_emitter.stop();}
    EPOK_FUNCTION(BlueprintCallable, Id="3ca01647-47a6-4f54-a4b5-3ca3021b6783") void burst(uint32_t count) {auto* data=entity_slot();if(data)data->particle_emitter.burst(uint16_t(count>256?256:count));}
    EPOK_FUNCTION(BlueprintCallable, Id="4394060a-88bc-45fa-8895-f7f58c6c3846") void set_enabled(bool value) {auto* data=entity_slot();if(data){data->particle_emitter.enabled=value;if(!value)data->particle_emitter.stop();}}
    EPOK_FUNCTION(BlueprintCallable, Id="d74618dc-1e6c-4dd3-af09-a8640d315cd9") void set_rate(Fixed value) {auto* data=entity_slot();if(data)data->particle_emitter.rate=value<0.0?Fixed(0.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="6a8c1193-55af-4a63-b6f0-50b5091e6b08") void set_lifetime(Fixed value) {auto* data=entity_slot();if(data)data->particle_emitter.lifetime=value<0.0?Fixed(0.0):value;}
    EPOK_FUNCTION(BlueprintCallable, Id="c9a87858-341d-4fac-bef0-dd83cd9140d4") void set_max_particles(uint32_t value) {auto* data=entity_slot();if(data)data->particle_emitter.max_particles=uint16_t(value>256?256:value);}
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
    ActorData* entity_slot() const {auto* owner=const_cast<PaletteAnimatorComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="ba716145-f679-420d-9081-dd6b4e8b0b2c") PaletteAnimationState state() const {PaletteAnimationState result;const auto* data=entity_slot();if(!data)return result;const auto& value=data->palette_animator;result.valid=true;result.enabled=value.enabled;result.reverse=value.reverse;result.texture=uint32_t(value.texture);result.first=value.first;result.last=value.last;result.offset=value.offset;result.speed=value.speed;return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="f590d75b-157c-4b05-9218-d0f506998e4b") void configure(int32_t texture,uint32_t first,uint32_t last,Fixed speed,bool reverse) {auto* data=entity_slot();if(!data)return;auto& value=data->palette_animator;value.texture=texture;value.first=uint8_t(first>255?255:first);value.last=uint8_t(last>255?255:last);value.speed=speed<0.0?Fixed(0.0):speed;value.reverse=reverse;value.enabled=value.first>0&&value.last>value.first;value.reset();}
    EPOK_FUNCTION(BlueprintCallable, Id="a3a8b122-7f9b-4cf5-8c95-b55fce5a881c") void set_enabled(bool enabled) {auto* data=entity_slot();if(data)data->palette_animator.enabled=enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="e93e4745-783d-454d-b1f0-094d426949b4") void reset() {auto* data=entity_slot();if(data)data->palette_animator.reset();}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=shadow, Id="1d067605-c408-40b8-b2c2-718b8cf0c605") BlobShadowComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1d067605-c408-40b8-b2c2-718b8cf0c605");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    ActorData* entity_slot() const {auto* owner=const_cast<BlobShadowComponent*>(this)->get_owner();return owner?owner->data():nullptr;}
    EPOK_FUNCTION(BlueprintPure, Id="db3a27d7-5c08-42dd-9cb3-d76c1f49a751") bool enabled() const {const auto* data=entity_slot();return data&&data->blob_shadow.enabled;}
    EPOK_FUNCTION(BlueprintCallable, Id="5d5ac6bc-6440-42fa-ae16-8c1d6a5d3175") void set_enabled(bool value) {auto* data=entity_slot();if(data)data->blob_shadow.enabled=value;}
    EPOK_FUNCTION(BlueprintCallable, Id="d75bdf7e-c768-4e73-9d6d-d63961070389") void configure(Fixed radius,Fixed strength,Fixed distance) {auto* data=entity_slot();if(data){data->blob_shadow.radius=radius<0.0?Fixed(0.0):radius;data->blob_shadow.strength=strength<0.0?Fixed(0.0):strength>1.0?Fixed(1.0):strength;data->blob_shadow.distance=distance<0.0?Fixed(0.0):distance;}}
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
        refresh_component_callbacks(owner);
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
        if(actor_type){actor.m_actor_tick=actor_type->component_tick;actor.m_actor_frame=actor_type->component_frame;}
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
        refresh_component_callbacks(actor);
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
        refresh_component_callbacks(owner);
    }
    void refresh_component_callbacks(Actor& owner) {
        static_assert(actor_component_capacity<=8,"Widen the callback masks with the component capacity");
        owner.m_tick_components=owner.m_frame_components=0;
        for(size_t c=0;c<owner.m_component_count;++c){
            const auto* type=m_registry->class_of(owner.m_components[c]);
            if(!type)continue;
            if(type->component_tick)owner.m_tick_components|=uint8_t(1u<<c);
            if(type->component_frame)owner.m_frame_components|=uint8_t(1u<<c);
        }
    }
    bool order_components(Actor& actor,const ObjectId* ids,size_t count) {
        if(count!=actor.m_component_count)return false;
        for(size_t i=0;i<count;++i) {
            auto* component=m_registry->resolve<ActorComponent>(ids[i]);
            if(!component || component->owner_id()!=actor.id())return false;
            for(size_t j=0;j<i;++j)if(ids[j]==ids[i])return false;
        }
        for(size_t i=0;i<count;++i)actor.m_components[i]=ids[i];
        refresh_component_callbacks(actor);
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
        if(!actor->m_tick_components&&(!actor->m_wants_tick||!actor->m_actor_tick))return;
        if (!actor_active(*actor)) return;
        ObjectDispatchScope scope(*m_registry);
        for (size_t c = 0; c < actor->m_component_count; ++c) {
            if (actor->m_doomed) return;
            if (actor->m_tick_components & (1u<<c))
                if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->tick(delta);
        }
        if (!actor->m_doomed && actor->m_wants_tick && actor->m_actor_tick) actor->tick(delta);
    }
    void frame_update_actor(ObjectId id, uint32_t elapsed) {
        auto* actor = m_registry->resolve<Actor>(id);
        if (!actor || actor->m_doomed || !actor->m_begun) return;
        if(!actor->m_frame_components&&!actor->m_actor_frame)return;
        if (!actor_active(*actor)) return;
        ObjectDispatchScope scope(*m_registry);
        for (size_t c = 0; c < actor->m_component_count; ++c) {
            if (actor->m_doomed) return;
            if (actor->m_frame_components & (1u<<c))
                if (auto* component = m_registry->resolve<ActorComponent>(actor->m_components[c])) component->frame_update(elapsed);
        }
        if(!actor->m_doomed&&actor->m_actor_frame)actor->frame_update(elapsed);
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
// The reflected Actor operations reach the Level through the registry, exactly like
// epok::bp::api::set_active/destroy. Without an active registry or Level they do nothing.
inline void Actor::set_active(bool active) {
    auto* level = active_object_registry ? active_object_registry->resolve<Level>(m_level) : nullptr;
    if (level) level->set_active(id(), active);
}
inline void Actor::destroy() {
    auto* level = active_object_registry ? active_object_registry->resolve<Level>(m_level) : nullptr;
    if (level) level->destroy_actor(id());
}
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
    auto* other_data=other.get();
    const ObjectId other_id=other_data&&other_data->owner?other_data->owner->id():ObjectId{};
    ObjectDispatchScope scope(*registry);
    for (size_t i = 0; i < owner->component_count(); ++i)
        if (auto* component = registry->resolve<ActorComponent>(owner->component_id(i))) {
            // The legacy bindings table wins: a component wrapping a Behaviour that
            // table already notified is skipped, so nothing is delivered twice.
            component->on_trigger(other, phase);
            component->trigger_event(other_id, phase);
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
