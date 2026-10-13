#pragma once
#include "baked_draw_types.hpp"
#include <array>

namespace epok {
// Immutable topology + an exact face mask uniquely determine the vertex set.
// Direct-mapped, bounded cache; collisions rebuild rather than hiding faces.
// Stores values only, never page-pool pointers. Eviction/reload is harmless.
template<size_t Slots> struct BakedSelectionCache {
  struct Entry {
    const void* key=nullptr;
    uint32_t faces[3]={}, vertices[16]={};
    uint16_t count=0;
  };
  std::array<Entry,Slots> entries{};
  const Entry& get(const void* key,size_t slot,const uint32_t* mask,
                   const BakedDrawFace* faces,size_t count) {
    auto& entry=entries[slot%Slots];
    const size_t words=(count+31)/32;
    bool matches=entry.key==key;
    for(size_t w=0;w<words;++w)matches&=entry.faces[w]==mask[w];
    if(matches)return entry;
    entry.key=key;
    for(size_t w=0;w<words;++w)entry.faces[w]=mask[w];
    for(auto& word:entry.vertices)word=0;
    for(size_t q=0;q<count;++q)if(mask[q/32]&(1u<<(q%32)))
      for(auto vertex:faces[q].indices)entry.vertices[vertex/32]|=1u<<(vertex%32);
    size_t vertices=0;for(auto word:entry.vertices)while(word){word&=word-1;++vertices;}
    entry.count=uint16_t(vertices);return entry;
  }
};
}
