#pragma once
#include "epok.hpp"

// Geometry adapters; selected policies and typed bindings own targeting rules.
namespace third_person_targeting {
int64_t horizontal_distance_squared(const epok::Transform& from,const epok::ActorData& target);
bool nearby(const epok::Transform& from,const epok::ActorData& target,epok::Fixed range);
bool visible(const epok::Transform& from,const epok::ActorData& target,const epok::SpatialQueryBatch* queries=nullptr);
}
