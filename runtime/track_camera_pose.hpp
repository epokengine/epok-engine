#pragma once
#include "visibility_track.hpp"
namespace epok {
struct TrackCameraPose {int32_t position[3],rotation[3],fov;};
inline TrackCameraPose interpolate_track_camera(const VisibilityTrackSample& a,const VisibilityTrackSample& b,uint32_t fraction) {
    TrackCameraPose out{};
    out.fov=int32_t(int64_t(a.fov)+(int64_t(b.fov)-a.fov)*fraction/4096);
    for(unsigned c=0;c<3;++c){
        out.position[c]=int32_t(int64_t(a.position[c])+(int64_t(b.position[c])-a.position[c])*fraction/4096);
        int32_t angle=int32_t((int64_t(b.rotation[c])-a.rotation[c])%(360*4096));
        if(angle>180*4096)angle-=360*4096;if(angle< -180*4096)angle+=360*4096;
        out.rotation[c]=int32_t(int64_t(a.rotation[c])+int64_t(angle)*fraction/4096);
    }
    return out;
}
inline void aim_track_camera(TrackCameraPose& pose,uint32_t elapsed,const int32_t* target,uint32_t previous_transitions) {
    const auto* track=active_visibility_track;
    if(!track||!track->navigation||!track->navigation->aim||!visibility_track_started)return;
    const auto& nav=*track->navigation;const auto& state=rail_navigation_state;
    const bool cut=state.transitions!=previous_transitions&&state.pending_link<nav.link_count&&nav.links[state.pending_link].load_cut;
    const uint16_t current=uint16_t(visibility_track_stats.sample),next=state.pending?state.next_sample:current;
    advance_rail_aim(*nav.aim,rail_aim_state,elapsed,pose.position,target,pose.rotation,current,next,state.fraction,cut);
}
}
