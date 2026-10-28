#pragma once
#include "epok.hpp"

// Project targeting policy is isolated from camera and movement. The demo uses
// a name prefix as its lightweight target marker; this can later be replaced by
// a gameplay tag/component without changing the controller modes.
namespace third_person_targeting {
bool candidate(const epok::ActorData* value);
int64_t horizontal_distance_squared(const epok::Transform& from,const epok::ActorData& target);
bool nearby(const epok::Transform& from,const epok::ActorData& target,epok::Fixed range);
bool visible(const epok::Transform& from,const epok::ActorData& target,const epok::SpatialQueryBatch* queries=nullptr);
epok::DataHandle nearest(const epok::Transform& from,epok::Fixed range,const epok::SpatialQueryBatch* queries=nullptr);
epok::DataHandle next(const epok::Transform& from,const epok::DataHandle& selected,epok::Fixed range,const epok::SpatialQueryBatch* queries=nullptr);
}
