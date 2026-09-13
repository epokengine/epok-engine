#pragma once
#include "blueprint_runtime.hpp"
#include "actor_blueprint.hpp"

namespace epok { extern bool lifecycle_tearing_down; }
namespace epok::bp {
using ClassId = uint64_t;
struct ClassInfo {
    ClassId id = 0, parent = 0;
    Behaviour* (*create)() = nullptr;
    void (*release)(Behaviour*) = nullptr;
    bool (*configure)(EntityHandle) = nullptr;
};
// Cooked ancestry includes non-instantiable entries. Zero is invalid; the cook
// rejects identity collisions and missing/cyclic ancestry before emitting these.
extern const ClassInfo classes[];
extern const size_t class_count;
template<class Type, size_t Capacity = 4> struct TypedPool {
    static_assert(Capacity > 0 && Capacity <= 32);
    inline static Type values[Capacity] = {};
    inline static bool used[Capacity] = {};
    static constexpr size_t storage_bytes = sizeof(values) + sizeof(used);
    static Behaviour* acquire() {
        for (size_t i = 0; i < Capacity; ++i) if (!used[i]) {
            used[i] = true; values[i] = Type{}; return &values[i];
        }
        return nullptr;
    }
    static void release(Behaviour* instance) {
        for (size_t i = 0; i < Capacity; ++i) if (&values[i] == instance && used[i]) {
            values[i] = Type{}; used[i] = false; return;
        }
    }
};
inline const ClassInfo* find_class(ClassId id) {
    if (id) for (size_t i = 0; i < class_count; ++i) if (classes[i].id == id) return &classes[i];
    return nullptr;
}
inline bool class_is_a(ClassId child, ClassId parent) {
    if (!parent) return false;
    for (size_t depth = 0; child && depth <= class_count; ++depth) {
        const auto* info = find_class(child);
        if (!info) return false;
        if (child == parent) return true;
        child = info->parent;
    }
    return false;
}
inline constexpr size_t dynamic_capacity = 32;
struct DynamicBinding {
    Binding binding = {};
    EntityHandle owner;
    EntityHandle batch;
    const ClassInfo* type = nullptr;
    uint32_t serial = 0, calls = 0;
    bool used = false, retiring = false, started = false;
};
inline DynamicBinding dynamic_bindings[dynamic_capacity] = {};
struct SpawnStats { uint32_t alive = 0, peak = 0, rejected = 0, spawned = 0; };
inline SpawnStats spawn_stats;
inline uint32_t binding_serial = 0;
inline void finish_release(DynamicBinding& binding) {
    if (!binding.used || !binding.retiring || binding.calls) return;
    binding.type->release(binding.binding.behaviour);
    binding.used = false; binding.retiring = false;
    if (spawn_stats.alive) --spawn_stats.alive;
}
// Typed storage and its entity slot remain quarantined until all nested
// callbacks return; destroying self inside update must not overwrite its stack.
struct DispatchScope {
    DynamicBinding& binding;
    explicit DispatchScope(DynamicBinding& value) : binding(value) { ++binding.calls; }
    ~DispatchScope() { --binding.calls; finish_release(binding); }
    DispatchScope(const DispatchScope&) = delete;
    DispatchScope& operator=(const DispatchScope&) = delete;
};
inline bool slot_quarantined(size_t index) {
    for (const auto& binding : dynamic_bindings)
        if (binding.used && binding.owner.index == index) return true;
    return false;
}
inline DynamicBinding* find_binding(EntityHandle owner) {
    if (!owner.get()) return nullptr;
    for (auto& binding : dynamic_bindings)
        if (binding.used && !binding.retiring && same_owner(binding.owner, owner)) return &binding;
    return nullptr;
}
Behaviour* authored_behaviour(EntityHandle owner);
ClassId authored_class(EntityHandle owner);
void activate_spawn_audio(EntityHandle root);
inline Behaviour* behaviour(EntityHandle owner) {
    auto* binding = find_binding(owner); return binding ? binding->binding.behaviour : authored_behaviour(owner);
}
inline bool is_a(EntityHandle owner, ClassId parent) {
    const auto* binding = find_binding(owner);
    if(binding)return class_is_a(binding->type->id,parent);
    return class_is_a(authored_class(owner),parent);
}
// Visit only bindings that existed at dispatch start. Generations and serials
// prevent a callback-created replacement from receiving the old slot's event.
template<class Callback> inline void visit(Callback callback) {
    uint32_t snapshot[dynamic_capacity] = {};
    for (size_t i = 0; i < dynamic_capacity; ++i)
        if (dynamic_bindings[i].used && !dynamic_bindings[i].retiring && dynamic_bindings[i].started) snapshot[i] = dynamic_bindings[i].serial;
    for (size_t i = 0; i < dynamic_capacity; ++i) {
        auto& binding = dynamic_bindings[i];
        if (!snapshot[i] || !binding.used || binding.retiring || binding.serial != snapshot[i] || !binding.owner.get()) continue;
        DispatchScope scope(binding);
        callback(binding.binding, binding.owner);
    }
}
inline void retire(const bool* doomed, const bool* active, size_t count) {
    uint32_t snapshot[dynamic_capacity] = {};
    for (size_t i = 0; i < dynamic_capacity; ++i) {
        auto& binding = dynamic_bindings[i];
        if (binding.used && !binding.retiring && binding.owner.index < count && doomed[binding.owner.index]) {
            snapshot[i] = binding.serial;
            binding.retiring = true;
        }
    }
    for (size_t i = 0; i < dynamic_capacity; ++i) {
        auto& binding = dynamic_bindings[i];
        if (!snapshot[i] || !binding.used || binding.serial != snapshot[i]) continue;
        DispatchScope scope(binding);
        binding.binding.behaviour->blueprint_cancel();
        if(binding.started){
            if (active[binding.owner.index]) binding.binding.behaviour->on_disable();
            binding.binding.behaviour->on_destroy();
        }
    }
}
// Reserve binds a fresh typed instance but performs no construction callbacks or
// gameplay dispatch. Template assembly uses one explicit batch before start.
inline EntityHandle reserve(ClassId id, const char* name, Entity* parent = nullptr, EntityHandle batch = {}) {
    const auto* type = find_class(id);
    if (lifecycle_tearing_down || !type || !type->create || !type->release) { increment(spawn_stats.rejected); return {}; }
    DynamicBinding* slot = nullptr;
    for (auto& candidate : dynamic_bindings) if (!candidate.used) { slot = &candidate; break; }
    if (!slot) { increment(spawn_stats.rejected); return {}; }
    // Reserve before invoking a native default constructor, which can reenter
    // engine APIs. This incomplete record has no owner/serial and is unvisitable.
    *slot = {}; slot->used = true;
    auto* instance = type->create();
    if (!instance) { slot->used = false; increment(spawn_stats.rejected); return {}; }
    auto* entity = create_entity(name, parent);
    if (!entity) { type->release(instance); slot->used = false; increment(spawn_stats.rejected); return {}; }
    const EntityHandle owner = handle(entity);
    ++binding_serial; if (!binding_serial) ++binding_serial;
    *slot = {{instance, owner.index, id}, owner, batch.get()?batch:owner, type, binding_serial, 0, true, false, false};
    ++spawn_stats.alive; increment(spawn_stats.spawned);
    if (spawn_stats.alive > spawn_stats.peak) spawn_stats.peak = spawn_stats.alive;
    instance->bind(*entity);
    return owner;
}
inline void start_reserved(EntityHandle owner) {
    auto* slot=find_binding(owner);
    if(!slot||slot->started)return;
    DispatchScope scope(*slot);slot->started=true;
    slot->binding.behaviour->start(owner.get()->transform);
    if(!slot->retiring&&owner.get()&&is_active(owner.get()))slot->binding.behaviour->on_enable();
}
inline void start_batch(EntityHandle root) {
    uint32_t snapshot[dynamic_capacity]={};
    for(size_t i=0;i<dynamic_capacity;++i){const auto& item=dynamic_bindings[i];if(item.used&&!item.retiring&&!item.started&&same_owner(item.batch,root))snapshot[i]=item.serial;}
    start_reserved(root);
    for(size_t i=0;i<dynamic_capacity;++i){auto& item=dynamic_bindings[i];if(snapshot[i]&&item.used&&!item.retiring&&item.serial==snapshot[i])start_reserved(item.owner);}
}
inline EntityHandle spawn(ClassId id, const char* name, Entity* parent = nullptr) {
    auto root=reserve(id,name,parent);
    if(!root.get())return {};
    auto* slot=find_binding(root);
    if(slot->type->configure&&!slot->type->configure(root)){
        if(root.get())destroy_entity(root.get());
        increment(spawn_stats.rejected);return {};
    }
    start_batch(root);
    if(root.get())activate_spawn_audio(root);
    return root.get()?root:EntityHandle{};
}
}
