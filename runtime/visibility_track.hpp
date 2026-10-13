#pragma once
#include <stdint.h>
#include <stddef.h>
#include "rail_navigation_types.hpp"
#include "ordered_draw_types.hpp"

namespace epok {
// A baked camera path and XOR changes to a per-cooked-face visibility bitset.
// Both current and next masks are resident; geometry remains in the page pool.
struct VisibilityTrackSample {
  int32_t position[3], rotation[3];
  int32_t fov;
  uint32_t change_offset;
  uint16_t change_bytes;
  bool cut;
};
struct PackedVisibilityTrackSample {
  int16_t position[3];uint16_t origin;
  uint16_t rotation[3],fov;
  uint32_t change_offset;uint16_t change_bytes;bool cut;
};
static_assert(sizeof(PackedVisibilityTrackSample)==24);
struct VisibilityPageSet { const uint32_t* pages; uint16_t count; };
struct VisibilityTrackData {
  const VisibilityTrackSample* samples;
  const uint8_t* changes;
  uint32_t* current;
  uint32_t* next;
  uint16_t sample_count, word_count, camera;
  uint32_t sample_us;
  bool cull_backfaces;
  const RailNavigationData* navigation=nullptr;
  const PackedVisibilityTrackSample* packed=nullptr;
  const int32_t (*origins)[3]=nullptr;
  const int32_t* scalars=nullptr;
  // Optional dictionary of exact page sets, cooked after final archive packing.
  // Covers tracked chunks only. Mixed banks merge untracked actors' ordinary
  // bounds-based requests at runtime; their presence does not disable this bake.
  const VisibilityPageSet* page_sets=nullptr;
  const uint16_t* page_samples=nullptr;
  const OrderedDrawData* ordered=nullptr;
  VisibilityTrackSample sample(size_t index) const {
    if(!packed)return samples[index];
    const auto& p=packed[index];VisibilityTrackSample out{};
    for(unsigned i=0;i<3;++i){out.position[i]=origins[p.origin][i]+p.position[i];out.rotation[i]=scalars[p.rotation[i]];}
    out.fov=scalars[p.fov];out.change_offset=p.change_offset;out.change_bytes=p.change_bytes;out.cut=p.cut;return out;
  }
  bool is_cut(size_t index) const {return packed?packed[index].cut:samples[index].cut;}
};
inline const VisibilityTrackData* active_visibility_track=nullptr;
struct VisibilityTrackStats { uint32_t sample=0, advances=0, waiting_frames=0, loops=0; };
inline VisibilityTrackStats visibility_track_stats;
inline uint32_t visibility_track_elapsed=0;
inline bool visibility_track_started=false;
// These cursors follow mask writes, not camera statistics: during a cut's ready
// callback the camera sample has advanced but `next` still names its destination.
struct VisibilityPageSelection { uint16_t current=0,next=0,extra=0xffff; };
inline VisibilityPageSelection visibility_page_selection;
inline bool visibility_has_page_sets() {
  return active_visibility_track&&active_visibility_track->page_sets;
}
template<class Visit> inline void visibility_visit_pages(bool next,Visit&& visit) {
  const auto& data=*active_visibility_track;const auto& s=visibility_page_selection;
  const auto append=[&](uint16_t sample){
    const auto& set=data.page_sets[data.page_samples[sample]];
    for(size_t i=0;i<set.count;++i)visit(set.pages[i]);
  };
  append(next?s.next:s.current);
  if(!next&&s.extra!=0xffff&&s.extra!=s.current)append(s.extra);
}

// Optional read-ahead for a deterministic playback path. Current/next pages
// are selected separately and always retain priority. Do not guess a gameplay
// rail branch or read through a camera cut; cap traversal even for tiny samples.
template<class Visit>
inline void visibility_visit_lookahead_pages(uint32_t horizon_us, Visit&& visit) {
  const auto* data = active_visibility_track;
  if (!data || !data->page_sets || data->navigation || !data->sample_us || data->sample_count < 3) return;
  uint32_t steps = horizon_us / data->sample_us;
  if (steps >= data->sample_count) steps = data->sample_count - 1;
  if (steps > 64) steps = 64;
  uint16_t sample = visibility_page_selection.current;
  for (uint32_t distance = 1; distance <= steps; ++distance) {
    if (++sample == data->sample_count) sample = 0;
    if (data->is_cut(sample)) break;
    if (distance < 2) continue;
    const auto& set = data->page_sets[data->page_samples[sample]];
    for (size_t i = 0; i < set.count; ++i) visit(set.pages[i], distance);
  }
}
inline uint32_t visibility_varint(const uint8_t*& p) {
  uint32_t value=0;unsigned shift=0;uint8_t byte;
  do{byte=*p++;value|=uint32_t(byte&127)<<shift;shift+=7;}while(byte&128);
  return value;
}
// Toggle a cooked contiguous run a word at a time. width is always 1..32:
// neither shift can be 32, including a full aligned word or its last bit.
inline void visibility_xor_run(uint32_t* words,uint32_t first,uint32_t count) {
  while(count){
    const uint32_t bit=first&31;
    const uint32_t available=32-bit;
    const uint32_t width=count<available?count:available;
    words[first>>5]^=(~uint32_t(0)>>(32-width))<<bit;
    first+=width;count-=width;
  }
}
inline void visibility_track_apply(uint32_t* words,const VisibilityTrackData& data,size_t sample) {
  const auto offset=data.packed?data.packed[sample].change_offset:data.samples[sample].change_offset;
  const auto length=data.packed?data.packed[sample].change_bytes:data.samples[sample].change_bytes;
  const uint8_t* p=data.changes+offset;const uint8_t* end=p+length;uint32_t cursor=0;
  while(p<end){const auto code=visibility_varint(p);cursor+=code>>1;
    if(!(code&1)){words[cursor>>5]^=uint32_t(1)<<(cursor&31);++cursor;continue;}
    const uint32_t count=visibility_varint(p)+2;
    visibility_xor_run(words,cursor,count);cursor+=count;
  }
}
inline void set_visibility_track(const VisibilityTrackData* data) {
  active_visibility_track=data;visibility_track_stats={};visibility_track_elapsed=0;visibility_track_started=false;
  visibility_page_selection={};
  rail_navigation_state={};
  rail_aim_state={};
  if(!data)return;
  for(size_t w=0;w<data->word_count;++w)data->current[w]=data->next[w]=0;
  if(data->navigation){
    const auto& nav=*data->navigation;auto& state=rail_navigation_state;
    state.route=state.next_route=nav.start_route;
    state.next_sample=nav.routes[nav.start_route].first;
    visibility_track_stats.sample=state.next_sample;
    visibility_page_selection.current=visibility_page_selection.next=state.next_sample;
    visibility_track_apply(data->current,*data,state.next_sample);
    for(size_t w=0;w<data->word_count;++w)nav.anchor[w]=data->next[w]=data->current[w];
    for(size_t i=0;i<nav.link_count;++i)if(nav.links[i].enabled)state.enabled[i/32]|=1u<<(i%32);
    return;
  }
  visibility_track_apply(data->current,*data,0);
  for(size_t w=0;w<data->word_count;++w)data->next[w]=data->current[w];
  if(data->sample_count>1)visibility_track_apply(data->next,*data,1);
  visibility_page_selection.next=data->sample_count>1?1:0;
}
inline bool visibility_face(const uint32_t* mask,size_t face) {
  return !mask || (mask[face>>5]&(uint32_t(1)<<(face&31)));
}
template<class Quad> inline size_t visibility_vertices(const uint32_t* faces,const Quad* quads,size_t count,uint32_t (&vertices)[16]) {
  for(auto& word:vertices)word=0;
  for(size_t q=0;q<count;++q)if(visibility_face(faces,q))for(unsigned c=0;c<4;++c){
    const auto v=quads[q].indices[c];vertices[v>>5]|=uint32_t(1)<<(v&31);
  }
  size_t count_vertices=0;for(auto word:vertices)while(word){word&=word-1;++count_vertices;}
  return count_vertices;
}
inline bool visibility_chunk(const uint32_t* mask,size_t faces) {
  if(!mask)return true;
  for(size_t w=0;w<(faces+31)/32;++w)if(mask[w])return true;
  return false;
}
inline const uint32_t* visibility_next_mask(const uint32_t* current) {
  return current&&active_visibility_track ? active_visibility_track->next+(current-active_visibility_track->current) : current;
}
inline bool visibility_prefetch_next() {
  const auto* track=active_visibility_track;if(!track)return false;
  return track->navigation?rail_navigation_state.pending:
    !track->is_cut((visibility_track_stats.sample+1)%track->sample_count);
}
// Readiness is supplied by the transport. No camera advance can expose a page
// before its verified read completes. Looping rebuilds the first full mask.
template<class Ready,class Camera>
inline void advance_visibility_track(uint32_t elapsed,Ready&& ready,Camera&& camera) {
  const auto* data=active_visibility_track;if(!data)return;
  size_t current=visibility_track_stats.sample;
  if(!visibility_track_started){
    if(!ready(false)){++visibility_track_stats.waiting_frames;const auto a=data->sample(current);camera(a,a,0);return;}
    visibility_track_started=true;elapsed=0;
  }
  const size_t next=(current+1)%data->sample_count;
  const bool cut=data->is_cut(next);
  if(!cut&&!ready(true)){++visibility_track_stats.waiting_frames;elapsed=0;}
  visibility_track_elapsed+=elapsed;
  if(visibility_track_elapsed>=data->sample_us && (cut||ready(true))){
    visibility_track_elapsed=0;current=next;visibility_track_stats.sample=uint32_t(current);++visibility_track_stats.advances;
    if(!current)++visibility_track_stats.loops;
    for(size_t w=0;w<data->word_count;++w)data->current[w]=data->next[w];
    visibility_page_selection.current=visibility_page_selection.next;
    if(cut)visibility_track_started=ready(true);
    if(current+1<data->sample_count)visibility_track_apply(data->next,*data,current+1);
    else {for(size_t w=0;w<data->word_count;++w)data->next[w]=0;visibility_track_apply(data->next,*data,0);}
    visibility_page_selection.next=uint16_t((current+1)%data->sample_count);
  }
  const auto a=data->sample(current),b=data->sample((current+1)%data->sample_count);
  const uint32_t fraction=b.cut?0:uint32_t(uint64_t(visibility_track_elapsed)*4096/data->sample_us);
  camera(a,b,fraction>4096?4096:fraction);
}
}
#include "rail_navigation.hpp"
