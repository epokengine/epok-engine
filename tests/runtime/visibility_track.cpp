#include "visibility_track.hpp"
#include "streaming_pool.hpp"
#include <cassert>
#include <cstdio>

static void test_xor_runs(){
  // Every alignment, empty/singleton runs, several word boundaries, and
  // nonzero initial masks. Guard words and untouched bits must survive.
  for(uint32_t first=0;first<96;++first)for(uint32_t count=0;count<=96;++count){
    uint32_t actual[8],expected[8],original[8];
    for(unsigned w=0;w<8;++w)actual[w]=expected[w]=original[w]=0xa539c6e7u^(w*0x17293b5du);
    for(uint32_t i=0;i<count;++i)expected[1+((first+i)>>5)]^=1u<<((first+i)&31);
    epok::visibility_xor_run(actual+1,first,count);
    for(unsigned w=0;w<8;++w)assert(actual[w]==expected[w]);
    epok::visibility_xor_run(actual+1,first,count);
    for(unsigned w=0;w<8;++w)assert(actual[w]==original[w]);
  }
  // The decoder must preserve cursor advancement across mixed run/singleton
  // entries, not merely produce the right result for an isolated run.
  const uint8_t changes[]={63,32,0,3,2,124,1,63};
  epok::VisibilityTrackSample sample{};sample.change_bytes=sizeof(changes);
  uint32_t actual[8]{},expected[8]{};
  epok::VisibilityTrackData data{&sample,changes,actual,nullptr,1,8,0,1,true};
  const auto* p=changes;uint32_t cursor=0;
  while(p<changes+sizeof(changes)){
    const auto code=epok::visibility_varint(p);cursor+=code>>1;
    const auto count=(code&1)?epok::visibility_varint(p)+2:1;
    for(uint32_t i=0;i<count;++i,++cursor)expected[cursor>>5]^=1u<<(cursor&31);
  }
  epok::visibility_track_apply(actual,data,0);
  for(unsigned w=0;w<8;++w)assert(actual[w]==expected[w]);
  epok::visibility_track_apply(actual,data,0);
  for(auto word:actual)assert(word==0);
}

int main(){
  test_xor_runs();
  // Sample 0 selects face 0; sample 1 drops 0 and selects 33. Two words
  // exercise chunk/word boundaries and the loop back to the full first mask.
  epok::VisibilityTrackSample samples[2]={
    {{0,0,0},{0,0,0},90*4096,0,1,true},
    {{4096,0,0},{0,0,0},90*4096,1,2,false}
  };
  uint8_t changes[]={0,0,64};uint32_t current[2]{},next[2]{};
  epok::VisibilityTrackData track{samples,changes,current,next,2,2,0,100000,true};
  const uint32_t page_ids[]={7,90000};
  const epok::VisibilityPageSet page_sets[]={{page_ids,1},{page_ids+1,1}};
  const uint16_t page_samples[]={0,1};track.page_sets=page_sets;track.page_samples=page_samples;
  auto check_pages=[&](){
    assert(epok::visibility_has_page_sets());
    for(unsigned future=0;future<2;++future){
      uint32_t actual[2]{};
      epok::visibility_visit_pages(future,[&](uint32_t page){
        assert(page==7||page==90000);actual[page==7?0:1]|=page==7?1:2;
      });
      const auto* expected=future?next:current;
      assert(actual[0]==expected[0]&&actual[1]==expected[1]);
    }
  };
  epok::set_visibility_track(&track);
  assert(epok::visibility_face(current,0)&&!epok::visibility_face(current,33));
  assert(!epok::visibility_face(next,0)&&epok::visibility_face(next,33));
  bool current_ready=false,next_ready=false;uint32_t fraction=999;int32_t camera_x=-1;
  auto ready=[&](bool future){check_pages();return future?next_ready:current_ready;};
  auto camera=[&](const auto& a,const auto&,uint32_t f){check_pages();camera_x=a.position[0];fraction=f;};
  epok::advance_visibility_track(200000,ready,camera);
  assert(!epok::visibility_track_started&&camera_x==0&&fraction==0);
  current_ready=true;
  epok::advance_visibility_track(200000,ready,camera);
  assert(epok::visibility_track_stats.sample==0&&epok::visibility_track_elapsed==0);
  next_ready=true;
  epok::advance_visibility_track(50000,ready,camera);
  assert(epok::visibility_track_stats.sample==0&&fraction==2048);
  epok::advance_visibility_track(50000,ready,camera);
  assert(epok::visibility_track_stats.sample==1&&camera_x==4096&&fraction==0);
  assert(epok::visibility_face(current,33)&&epok::visibility_face(next,0));
  next_ready=false;
  epok::advance_visibility_track(300000,ready,camera);
  assert(epok::visibility_track_stats.sample==0&&epok::visibility_track_stats.loops==1);
  assert(!epok::visibility_track_started);
  current_ready=false;
  epok::advance_visibility_track(100000,ready,camera);
  assert(!epok::visibility_track_started&&fraction==0);
  current_ready=true;next_ready=true;
  epok::advance_visibility_track(0,ready,camera);
  assert(epok::visibility_track_started);
  assert(current[0]==1&&current[1]==0&&next[0]==0&&next[1]==2);
  uint32_t big[2048]{};const uint8_t run[]={1,254,255,3,254,255,7};
  epok::VisibilityTrackSample packed[2]{};packed[0].change_bytes=4;packed[1].change_offset=4;packed[1].change_bytes=3;
  auto large=track;large.samples=packed;large.changes=run;
  epok::visibility_track_apply(big,large,0);
  for(auto word:big)assert(word==0xffffffffu);
  epok::visibility_track_apply(big,large,1);assert(big[2047]==0x7fffffffu);
  struct Quad{uint16_t indices[4];};
  const Quad quads[]={{{0,1,511,511}},{{2,3,4,5}},{{0,31,32,511}}};
  uint32_t face_mask[]={5},vertex_mask[16];
  assert(epok::visibility_vertices(face_mask,quads,3,vertex_mask)==5);
  for(unsigned i=0;i<512;++i)assert(epok::visibility_face(vertex_mask,i)==(i==0||i==1||i==31||i==32||i==511));
  epok::set_visibility_track(nullptr);
  assert(!epok::active_visibility_track&&epok::visibility_face(nullptr,100)&&!epok::visibility_has_page_sets());
  const int32_t origins[][3]={{30000,-30000,7}},scalars[]={0,90*4096};
  const epok::PackedVisibilityTrackSample compressed[]={
    {{-30000,30000,-7},0,{0,0,0},1,0,1,true},
    {{-25904,30000,-7},0,{0,0,0},1,1,2,false}
  };
  auto compact=track;compact.samples=nullptr;compact.packed=compressed;compact.origins=origins;compact.scalars=scalars;
  for(unsigned i=0;i<2;++i){const auto value=compact.sample(i);const auto& expected=samples[i];
    for(unsigned c=0;c<3;++c){assert(value.position[c]==expected.position[c]);assert(value.rotation[c]==expected.rotation[c]);}
    assert(value.fov==expected.fov&&value.cut==expected.cut&&value.change_offset==expected.change_offset&&value.change_bytes==expected.change_bytes);
  }
  epok::set_visibility_track(&compact);assert(current[0]==1&&next[1]==2);
  epok::advance_visibility_track(0,ready,camera);epok::advance_visibility_track(100000,ready,camera);
  assert(epok::visibility_track_stats.sample==1&&camera_x==4096&&current[1]==2);
  epok::set_visibility_track(nullptr);
  // A non-streaming/legacy bank selects the fallback; empty baked samples are
  // valid (no dummy page zero may leak into the working set).
  auto empty=track;const epok::VisibilityPageSet empty_sets[]={{nullptr,0}};
  const uint16_t empty_samples[]={0,0};empty.page_sets=empty_sets;empty.page_samples=empty_samples;
  epok::set_visibility_track(&empty);unsigned visits=0;
  epok::visibility_visit_pages(false,[&](uint32_t){++visits;});
  epok::visibility_visit_pages(true,[&](uint32_t){++visits;});assert(visits==0);
  empty.page_sets=nullptr;empty.page_samples=nullptr;epok::set_visibility_track(&empty);
  assert(!epok::visibility_has_page_sets());epok::set_visibility_track(nullptr);
  epok::VisibilityTrackSample window_samples[6]{};
  window_samples[0].cut = window_samples[4].cut = true;
  const uint32_t window_pages[] = {0,1,2,3,4,5};
  epok::VisibilityPageSet window_sets[6];
  const uint16_t window_indices[] = {0,1,2,3,4,5};
  for (unsigned i = 0; i < 6; ++i) window_sets[i] = {window_pages+i,1};
  auto window = track; window.samples = window_samples; window.sample_count = 6;
  window.page_sets = window_sets; window.page_samples = window_indices;
  epok::set_visibility_track(&window);
  uint32_t seen = 0;
  auto collect = [&](uint32_t page, uint32_t distance) { assert(distance >= 2); seen |= 1u << page; };
  epok::visibility_visit_lookahead_pages(100000, collect); assert(!seen);
  epok::visibility_visit_lookahead_pages(1500000, collect); assert(seen == ((1u<<2)|(1u<<3)));
  // Speculation can occupy only spare slots, preserving even an untracked
  // mandatory page with a numerically worse distance score.
  epok::StreamPageWorkingSet<3> working;
  working.consider(8,0); working.consider(9,100000);
  epok::visibility_visit_lookahead_pages(1500000,[&](uint32_t page,uint32_t distance){
    if (!working.contains(page) && working.count() < 3) working.consider(page,distance);
  });
  assert(working.count() == 3 && working.contains(8) && working.contains(9) && working.contains(2));
  window_samples[4].cut = false; seen = 0;
  epok::visibility_visit_lookahead_pages(UINT32_MAX, collect); assert(seen == 60);
  epok::visibility_page_selection.current = 4; seen = 0;
  epok::visibility_visit_lookahead_pages(1500000, collect); assert(!seen); // Stops at cut 0.
  window_samples[0].cut = false; epok::visibility_page_selection.current = 5;
  epok::visibility_visit_lookahead_pages(1500000, collect); assert(seen == 30); // Wrap, never full cycle.
  epok::RailNavigationData unknown_branch{}; window.navigation = &unknown_branch; seen = 0;
  epok::visibility_visit_lookahead_pages(1500000, collect); assert(!seen);
  window.navigation = nullptr; window.sample_us = 0;
  epok::visibility_visit_lookahead_pages(1500000, collect); assert(!seen);
  epok::VisibilityTrackSample tiny_samples[70]{};
  uint16_t tiny_indices[70]{};
  window.samples = tiny_samples; window.page_samples = tiny_indices;
  window.sample_count = 70; window.sample_us = 1;
  epok::set_visibility_track(&window); unsigned bounded_visits = 0;
  epok::visibility_visit_lookahead_pages(UINT32_MAX,[&](uint32_t,uint32_t distance){
    assert(distance <= 64); ++bounded_visits;
  });
  assert(bounded_visits == 63); // No unbounded traversal at tiny sample intervals.
  epok::set_visibility_track(nullptr);
  std::puts("baked visibility: readiness, bounded spare-slot read-ahead, cuts, rail exclusion, interpolation and looping passed");
}
