#pragma once
#include "visibility_track.hpp"
namespace epok {
inline bool rail_pending_cut() {
  const auto* data=active_visibility_track;const auto& s=rail_navigation_state;
  return data&&data->navigation&&s.pending&&s.crossing&&
      s.pending_link<data->navigation->link_count&&data->navigation->links[s.pending_link].load_cut;
}
// This controller consumes world-space player position, never pad input. It is
// reusable with native/Blueprint actors and independent of the page transport.
inline bool rail_set_link_enabled(size_t index,bool enabled) {
  const auto* data=active_visibility_track;
  if(!data||!data->navigation||index>=data->navigation->link_count)return false;
  auto& bits=rail_navigation_state.enabled[index/32];const auto mask=1u<<(index%32);
  if(enabled)bits|=mask;else bits&=~mask;return true;
}
inline int64_t rail_projection(const int32_t* player,const VisibilityTrackSample& sample,const RailParameter& p) {
  return ((int64_t(player[0])-sample.position[0])*p.axis[0]+
          (int64_t(player[2])-sample.position[2])*p.axis[1])/4096;
}
inline int64_t rail_error(int64_t value,const RailParameter& p) {
  return value<p.limits[0]?int64_t(p.limits[0])-value:value>p.limits[1]?value-p.limits[1]:0;
}
inline void rail_prepare(uint16_t sample,uint16_t route,bool crossing,bool forward) {
  const auto& data=*active_visibility_track;const auto& nav=*data.navigation;auto& s=rail_navigation_state;
  for(size_t w=0;w<data.word_count;++w)data.next[w]=crossing?0:nav.anchor[w];
  if(crossing){
    // Each route starts with a full checkpoint. No traversal through unrelated
    // routes or O(level-size) rebuild when following a graph link.
    for(size_t i=nav.routes[route].first;i<=sample;++i)visibility_track_apply(data.next,data,i);
  }else{
    const size_t current=visibility_track_stats.sample;
    visibility_track_apply(data.next,data,sample>current?sample:current);
  }
  s.pending=true;s.next_sample=sample;s.next_route=route;s.crossing=crossing;s.next_forward=forward;
  visibility_page_selection.next=sample;
}
template<class Ready,class Camera>
inline void advance_rail_navigation(uint32_t elapsed,const int32_t* player,Ready&& ready,Camera&& camera) {
  const auto* ptr=active_visibility_track;if(!ptr||!ptr->navigation)return;
  const auto& data=*ptr;const auto& nav=*data.navigation;auto& s=rail_navigation_state;
  for(unsigned i=0;i<3;++i)s.target_position[i]=player[i];
  size_t current=visibility_track_stats.sample;
  if(!visibility_track_started){
    if(!ready(false)){++visibility_track_stats.waiting_frames;const auto a=data.sample(current);camera(a,a,0);return;}
    visibility_track_started=true;elapsed=0;
  }
  const auto& route=nav.routes[s.route];const auto p=nav.parameter(current);
  const auto a=data.sample(current);
  const int64_t d0=rail_projection(player,a,p);
  if(!s.pending && elapsed){
    uint16_t selected=uint16_t(current),selected_route=s.route;
    bool crossing=false,forward=s.forward;uint16_t selected_link=0xffff;
    int64_t best=rail_error(d0,p);
    const bool auto_forward=p.speed>=0?s.forward:!s.forward;
    for(int direction=-1;direction<=1;direction+=2){
      const bool next_forward=direction>0;
      if(route.automatic && next_forward!=auto_forward)continue;
      const int candidate=int(current)+direction;
      if(candidate>=route.first&&candidate<route.first+route.count){
        const auto error=rail_error(rail_projection(player,data.sample(candidate),p),p);
        if((route.automatic&&p.speed)||error<best){
          best=error;selected=uint16_t(candidate);forward=next_forward;
          crossing=false;selected_route=s.route;selected_link=0xffff;
        }
      }else if(route.automatic||best>0){
        // Only outgoing links at this endpoint participate. Never snap to a
        // spatially close route that is not in the authored graph.
        for(size_t l=route.link_first;l<route.link_first+route.link_count;++l){
          const auto& link=nav.links[l];
          if(link.from_end!=next_forward||!(s.enabled[l/32]&(1u<<(l%32))))continue;
          const auto& r=nav.routes[link.route];
          const uint16_t endpoint=uint16_t(r.first+(link.to_end?r.count-1:0));
          const auto look=uint16_t(endpoint+(r.count>1?(link.to_end?-1:1):0));
          const auto error=rail_error(rail_projection(player,data.sample(look),p),p);
          if((route.automatic&&p.speed)||error<best){
            best=error;selected=endpoint;selected_route=link.route;crossing=true;forward=!link.to_end;selected_link=uint16_t(l);
            if(route.automatic)break;
          }
        }
      }
    }
    if(selected!=current||selected_route!=s.route){rail_prepare(selected,selected_route,crossing,forward);s.pending_link=selected_link;}
  }
  if(s.pending&&s.crossing){
    const auto& r=nav.routes[s.next_route];
    const uint16_t look=uint16_t(s.next_sample+(r.count>1?(s.next_forward?1:-1):0));
    if((s.pending_link!=0xffff&&!(s.enabled[s.pending_link/32]&(1u<<(s.pending_link%32))))||
       (!route.automatic&&rail_error(rail_projection(player,data.sample(look),p),p)>=rail_error(d0,p)))s.pending=false;
  }
  if(s.pending){
    const auto b=data.sample(s.next_sample);
    uint32_t goal=4096;
    if(!route.automatic&&!s.crossing){
      const int64_t d1=rail_projection(player,b,p);
      const int64_t value=d0+(d1-d0)*s.fraction/4096;
      if(!rail_error(value,p))goal=s.fraction;
      else if(d0==d1)goal=0;
      else{
        const int64_t bound=value<p.limits[0]?p.limits[0]:p.limits[1];
        const int64_t solved=(bound-d0)*4096/(d1-d0);
        goal=solved<0?0:solved>4096?4096:uint32_t(solved);
      }
    }
    // A reversal may cancel a queued, still-unread page without exposing it.
    if(!goal&&!s.fraction){s.pending=false;}
    else if(elapsed){
      if(!ready(true)){++visibility_track_stats.waiting_frames;}
      else{
        const uint64_t speed=p.speed<0?-int64_t(p.speed):p.speed;
        uint32_t step=uint32_t((uint64_t(elapsed>100000?100000:elapsed)*speed)/1000000);
        if(s.crossing)step=4096;
        if(goal<s.fraction){
          ++s.reversals;s.fraction-=s.fraction-goal<step?s.fraction-goal:step;
        }else s.fraction+=goal-s.fraction<step?goal-s.fraction:step;
        for(size_t w=0;w<data.word_count;++w)data.current[w]=nav.anchor[w]|(s.fraction?data.next[w]:0);
        visibility_page_selection.extra=s.fraction?s.next_sample:0xffff;
        if(s.fraction==4096){
          for(size_t w=0;w<data.word_count;++w)nav.anchor[w]=data.current[w]=data.next[w];
          visibility_page_selection.current=s.next_sample;visibility_page_selection.extra=0xffff;
          if(s.crossing){++s.transitions;s.forward=s.next_forward;}
          s.route=s.next_route;s.fraction=0;s.pending=false;
          current=s.next_sample;visibility_track_stats.sample=uint32_t(current);++visibility_track_stats.advances;
        }else if(!s.fraction&&!goal)s.pending=false;
      }
    }
  }
  camera(data.sample(current),data.sample(s.pending?s.next_sample:current),s.fraction);
}
}
