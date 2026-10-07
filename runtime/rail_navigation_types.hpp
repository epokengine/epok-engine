#pragma once
#include <stdint.h>
#include <stddef.h>
#include "rail_camera_aim.hpp"
namespace epok {
struct RailRoute { uint16_t first,count,link_first,link_count; bool automatic; };
struct RailLink { uint16_t route; bool from_end,to_end,enabled,load_cut; };
struct RailParameter { int16_t axis[2]; int32_t limits[2],speed; };
struct PackedRailParameter { uint16_t axis,settings; };
struct RailNavigationData {
  const RailRoute* routes;const RailLink* links;const RailParameter* parameters;
  uint32_t* anchor;uint16_t route_count,link_count,target,start_route;
  const PackedRailParameter* packed=nullptr;
  const int16_t (*axes)[2]=nullptr;
  const int32_t (*settings)[3]=nullptr;
  const RailAimData* aim=nullptr;
  RailParameter parameter(size_t sample) const {
    if(!packed)return parameters[sample];
    const auto& p=packed[sample];const auto* a=axes[p.axis];const auto* s=settings[p.settings];
    return {{a[0],a[1]},{s[0],s[1]},s[2]};
  }
};
struct RailNavigationState {
  uint16_t route=0,next_route=0,next_sample=0,pending_link=0xffff;
  uint32_t fraction=0,transitions=0,reversals=0;
  bool pending=false,crossing=false,forward=true,next_forward=true;
  uint32_t enabled[16]{};
  int32_t target_position[3]{}; // Last consumed world-space Q12 position; diagnostics only.
};
inline RailNavigationState rail_navigation_state;
static_assert(offsetof(RailNavigationState,target_position)==88&&sizeof(RailNavigationState)==100);
}
