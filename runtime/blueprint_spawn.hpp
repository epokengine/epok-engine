#pragma once
#include "blueprint_runtime.hpp"
#include "actor_blueprint.hpp"
namespace epok::bp {
using ClassId = uint64_t;
inline const ClassDescriptor* find_class(ClassId id) { return find_object_class(id); }
inline bool class_is_a(ClassId child, ClassId parent) { return object_class_is_a(child,parent); }
inline Object* object(ObjectId id) { return active_object_registry ? active_object_registry->get(id) : nullptr; }
inline Object* object(DataHandle value) { auto* data=value.get(); return data ? data->owner : nullptr; }
inline bool is_a(ObjectId id,ClassId type) { auto* value=object(id); return value && class_is_a(value->class_id(),type); }
inline bool is_a(DataHandle id,ClassId type) { auto* value=object(id); return value && class_is_a(value->class_id(),type); }
}
