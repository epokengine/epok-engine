#pragma once
// Blueprint-facing helpers for the Object/Actor/Component runtime model.
//
// Generated Blueprint classes whose family is Actor or Component need a handful of
// null-safe adapters: the legacy entity slot behind an actor's root scene component,
// the EntityHandle the Blueprint node ABI speaks, the owner of a component and a
// bounded actor spawn. Keeping them here (instead of in the generated text) means the
// null checks exist once, are compiled for MIPS with the rest of the runtime and can be
// unit-tested by the host harness.
//
// No RTTI, no exceptions, no heap. Every accessor returns a null identity rather than
// dereferencing a stale handle.
#include "object_model.hpp"

namespace epok::bp {

// Legacy slot behind the actor's root scene component. Null for Actor2D/UIActor, whose
// roots carry their own transform storage, and for an actor that has no root yet.
inline Entity* actor_entity(Actor* actor) { return actor ? actor->entity() : nullptr; }

// Blueprint node ABI handle for the actor's legacy slot. A null slot yields a null
// handle, which every epok::bp::api entry point already treats as "do nothing".
inline EntityHandle actor_handle(Actor* actor) {
    Entity* slot = actor_entity(actor);
    return slot ? epok::handle(slot) : EntityHandle{};
}

// Owning actor of a component, and its handle/slot, for Component-family Blueprints.
inline Actor* component_owner(ActorComponent* component) {
    return component ? component->get_owner() : nullptr;
}
inline ObjectId component_owner_id(ActorComponent* component) {
    Actor* owner = component_owner(component);
    return owner ? owner->id() : ObjectId{};
}
inline Entity* component_entity(ActorComponent* component) {
    return actor_entity(component_owner(component));
}
inline EntityHandle component_handle(ActorComponent* component) {
    return actor_handle(component_owner(component));
}

// Transform alias target. The fallback keeps a reference alias well formed when the
// actor has no legacy slot; generated code still emits an explicit skip before using
// the alias, so the fallback is never observed by a running graph.
inline Transform& object_transform_fallback() {
    static Transform fallback{};
    return fallback;
}
inline Transform& actor_transform(Actor* actor) {
    Entity* slot = actor_entity(actor);
    return slot ? slot->transform : object_transform_fallback();
}
inline Transform& component_transform(ActorComponent* component) {
    return actor_transform(component_owner(component));
}

// Bounded actor spawn for the SpawnActor node. The class must be Actor family and
// concrete; the cook rejects anything else at compile time, and this rejects a stale
// or unknown compact id at run time.
inline ObjectId spawn_actor(Actor* context, uint64_t class_id, ObjectId logical_parent) {
    if (!context || !active_object_registry) return ObjectId{};
    const ClassDescriptor* type = find_object_class(class_id);
    if (!type || type->family != ObjectFamily::Actor) return ObjectId{};
    Level* level = active_object_registry->resolve<Level>(context->level_id());
    return level ? level->spawn_actor(*type, nullptr, logical_parent) : ObjectId{};
}

}  // namespace epok::bp
