#include "rail_navigation.hpp"
#include <cassert>
#include <cstdio>
using namespace epok;
int main(){
  VisibilityTrackSample samples[7]{};
  const int x[]={0,1,2,2,3,2,4};
  const uint32_t offsets[]={0,1,3,5,6,8,9};
  const uint16_t lengths[]={1,2,2,1,2,1,2};
  for(unsigned i=0;i<7;++i){samples[i].position[0]=x[i]*4096;samples[i].change_offset=offsets[i];samples[i].change_bytes=lengths[i];}
  const uint8_t changes[]={0,1,0,3,0,6,7,0,10,11,0};
  uint32_t current[1]{},next[1]{},anchor[1]{};
  RailRoute routes[]={{0,3,0,2,false},{3,2,2,1,false},{5,2,3,0,false}};
  RailLink links[]={{1,true,false,true},{2,true,false,false},{0,false,true,true}};
  RailParameter parameters[7];for(auto& p:parameters)p={{4096,0},{0,0},10*4096};
  RailNavigationData nav{routes,links,parameters,anchor,3,3,1,0};
  VisibilityTrackData track{samples,changes,current,next,7,1,0,100000,false,&nav};
  const uint32_t pages[]={11,12,13,14,15,16,17};
  VisibilityPageSet page_sets[7];uint16_t page_samples[7];
  for(unsigned i=0;i<7;++i){page_sets[i]={pages+i,1};page_samples[i]=uint16_t(i);}
  track.page_sets=page_sets;track.page_samples=page_samples;
  auto check_pages=[&](){
    for(unsigned future=0;future<2;++future){
      uint32_t mask=0;visibility_visit_pages(future,[&](uint32_t page){assert(page>=11&&page<=17);mask|=1u<<(page-11);});
      assert(mask==(future?next[0]:current[0]));
    }
  };
  bool current_ready=false,next_ready=false;
  int32_t player[3]{},camera_x=-1;
  auto ready=[&](bool n){check_pages();return n?next_ready:current_ready;};
  auto camera=[&](const auto& a,const auto& b,uint32_t f){check_pages();camera_x=a.position[0]+int32_t(int64_t(b.position[0]-a.position[0])*f/4096);};
  auto advance=[&](uint32_t us){advance_rail_navigation(us,player,ready,camera);};
  set_visibility_track(&track);
  advance(100000);assert(!visibility_track_started&&current[0]==1&&camera_x==0);
  current_ready=true;advance(100000);assert(visibility_track_started);
  // No player movement -> no sample progression or automatic looping.
  for(int i=0;i<20;++i)advance(100000);
  assert(visibility_track_stats.advances==0&&!rail_navigation_state.pending);
  player[0]=4096;advance(50000);
  assert(rail_navigation_state.pending&&next[0]==2&&current[0]==1&&camera_x==0);
  // Reverse while a read is outstanding: cancel, without exposing the new mask.
  player[0]=0;advance(50000);
  assert(!rail_navigation_state.pending&&current[0]==1&&camera_x==0);
  player[0]=4096;next_ready=true;advance(50000);
  assert(camera_x==2048&&current[0]==3&&anchor[0]==1);
  // Pause preserves the interpolated pose and both visible sets.
  advance(0);assert(camera_x==2048&&current[0]==3);
  player[0]=0;advance(50000);assert(camera_x==0&&current[0]==1);
  player[0]=2*4096;advance(100000);advance(100000);
  assert(visibility_track_stats.sample==2&&current[0]==4&&anchor[0]==4);
  // Target must traverse a declared neighbor. Reads and masks remain atomic.
  player[0]=3*4096;next_ready=false;advance(100000);
  assert(rail_navigation_state.pending&&rail_navigation_state.crossing&&next[0]==8&&current[0]==4);
  assert(rail_set_link_enabled(0,false));advance(100000);
  assert(!rail_navigation_state.pending&&rail_navigation_state.route==0);
  assert(!rail_set_link_enabled(512,true));
  rail_set_link_enabled(0,true);next_ready=true;advance(100000);
  assert(rail_navigation_state.route==1&&visibility_track_stats.sample==3&&current[0]==8);
  advance(100000);assert(visibility_track_stats.sample==4&&current[0]==16);
  // Reverse deltas and entry through the destination's final endpoint.
  player[0]=0;advance(100000);assert(visibility_track_stats.sample==3&&current[0]==8);
  advance(100000);assert(rail_navigation_state.route==0&&visibility_track_stats.sample==2&&current[0]==4);
  advance(100000);assert(visibility_track_stats.sample==1&&current[0]==2);
  advance(100000);assert(visibility_track_stats.sample==0&&current[0]==1);
  // Disabled branch stays inaccessible; enabling it selects by player position.
  player[0]=4*4096;advance(100000);advance(100000);
  rail_set_link_enabled(0,false);advance(100000);assert(rail_navigation_state.route==0);
  rail_set_link_enabled(1,true);advance(100000);assert(rail_navigation_state.route==2&&current[0]==32);
  advance(100000);assert(visibility_track_stats.sample==6&&current[0]==64);
  // Full checkpoint at a nonzero start route, automatic mode, signed speed.
  nav.start_route=2;routes[2].automatic=true;set_visibility_track(&track);
  assert(visibility_track_stats.sample==5&&current[0]==32);
  advance(100000);advance(100000);assert(visibility_track_stats.sample==6&&current[0]==64);
  parameters[6].speed=-10*4096;advance(100000);assert(visibility_track_stats.sample==5&&current[0]==32);
  parameters[5].speed=-10*4096;advance(100000);assert(visibility_track_stats.sample==5); // no direction bounce
  // Discontinuous links request an atomic destination load. Read failure or a
  // delayed completion must not publish its camera/mask; the transport may
  // replace unpinned source pages while the previous framebuffer is displayed.
  nav.start_route=0;routes[0].automatic=true;links[0].load_cut=true;set_visibility_track(&track);
  advance(100000);advance(100000);advance(100000);
  next_ready=false;advance(100000);assert(rail_pending_cut()&&visibility_track_stats.sample==2&&current[0]==4);
  const auto before=camera_x;advance(100000);assert(rail_pending_cut()&&camera_x==before&&current[0]==4);
  next_ready=true;advance(100000);assert(!rail_pending_cut()&&visibility_track_stats.sample==3&&current[0]==8);
  assert(rail_navigation_state.target_position[0]==player[0]);
  // Differential stress: reversals, fractional pauses, delayed readiness and
  // graph restrictions must keep the cooked page cursor equal to actual masks.
  routes[0].automatic=false;routes[2].automatic=false;
  for(auto& p:parameters)p.speed=10*4096;
  set_visibility_track(&track);uint32_t random=12345;
  for(unsigned i=0;i<20000;++i){
    random=random*1664525u+1013904223u;
    player[0]=int32_t((random>>8)%24576)-4096;
    next_ready=(random&7)!=0;
    rail_set_link_enabled((random>>5)%3,(random&16)!=0);
    advance((random&3)?17000:0);
  }
  const int16_t axes[][2]={{4096,0},{-4096,0}};
  const int32_t settings[][3]={{0,0,10*4096},{-1024,2048,-30*4096}};
  const PackedRailParameter packed_params[]={{1,1},{0,0}};
  auto compact_nav=nav;compact_nav.parameters=nullptr;compact_nav.packed=packed_params;compact_nav.axes=axes;compact_nav.settings=settings;
  const auto decoded=compact_nav.parameter(0);
  assert(decoded.axis[0]==-4096&&decoded.axis[1]==0&&decoded.limits[0]==-1024&&decoded.limits[1]==2048&&decoded.speed==-30*4096);
  set_visibility_track(nullptr);assert(!visibility_prefetch_next());
  std::puts("rail navigation: stop, forward/reverse, delayed reads, union masks, pause, links, restrictions, branches, checkpoints and automatic mode passed");
}
