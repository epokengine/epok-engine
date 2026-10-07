#include "baked_selection.hpp"
#include <cassert>
#include <cstdio>
using namespace epok;
int main() {
  BakedSelectionCache<2> cache;
  BakedDrawFace faces[96]{};
  faces[0].indices[0]=0;faces[0].indices[1]=31;faces[0].indices[2]=32;faces[0].indices[3]=32;
  faces[95].indices[0]=128;faces[95].indices[1]=255;faces[95].indices[2]=511;faces[95].indices[3]=511;
  uint32_t mask[3]={1,0,0};int first,second;
  const auto& a=cache.get(&first,0,mask,faces,96);
  assert(a.count==3 && a.vertices[0]==0x80000001u && a.vertices[1]==1);
  // A hit never reads the evicted topology page: the cache owns only bit values.
  assert(cache.get(&first,0,mask,nullptr,96).count==3);
  mask[2]=0x80000000u;
  assert(cache.get(&first,0,mask,faces,96).count==6);
  assert(cache.get(&second,2,mask,faces,96).count==6); // deliberate slot collision
  mask[0]=0;assert(cache.get(&first,0,mask,faces,96).count==3);
  mask[2]=0;assert(cache.get(&first,0,mask,faces,96).count==0);
  std::puts("baked selection: exact masks, high indices, collisions, eviction-safe hits passed");
}
