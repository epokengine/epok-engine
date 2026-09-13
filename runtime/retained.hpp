#pragma once
#include "epok.hpp"
#include <array>
#include <stddef.h>
#include <stdint.h>

// Retained GPU packets for static meshes (Project Settings > Rendering >
// Retained Packets). Every quad of a retained object owns two fixed fragment
// slots in each parity buffer. A rebuild writes the words that do not change
// per frame: command byte and corner colours, page UVs, palette and page
// attributes. Each frame then writes only screen coordinates, links the
// fragment into the ordering table and, when fog touches the quad, its three
// colour words. PsyQo transfers the other parity's chain while a frame is
// built, so a rebuild touches only the parity being built and each parity
// keeps its own key of the inputs baked into its packets.
namespace epok {
struct RetainedKey {
    const MeshGeometry* geometry=nullptr;const Texture* bank=nullptr;const uint8_t (*baked)[3]=nullptr;size_t baked_count=0;
    uint32_t generation=0,light_signature=0;int32_t basis[9]={};
    // All-zero defaults keep the pool in .bss; `valid` gates every comparison.
    uint8_t color[3]={};bool unlit=false;int texture=0;BlendMode blend=BlendMode(0);int16_t depth_bias=0;int32_t uv_scroll[2]={};
    bool lighting_enabled=false;ReceiveLighting receive=ReceiveLighting(0);bool valid=false;
    bool operator==(const RetainedKey& o) const {
        if(!valid||!o.valid)return false;
        if(geometry!=o.geometry||bank!=o.bank||baked!=o.baked||baked_count!=o.baked_count||generation!=o.generation||light_signature!=o.light_signature)return false;
        for(int i=0;i<9;++i)if(basis[i]!=o.basis[i])return false;
        for(int c=0;c<3;++c)if(color[c]!=o.color[c])return false;
        return unlit==o.unlit&&texture==o.texture&&blend==o.blend&&depth_bias==o.depth_bias&&uv_scroll[0]==o.uv_scroll[0]&&uv_scroll[1]==o.uv_scroll[1]&&lighting_enabled==o.lighting_enabled&&receive==o.receive;
    }
};
struct RetainedQuad {
    uint32_t shaded[4]={};   // fog-free 0..255 corner colours
    uint32_t command=0;      // GPU command byte with the transparency bit
    int16_t depth_bias=0;
    uint8_t textured=0;
    uint8_t dynamic=0;       // UV scrolling: the per-frame path draws it instead
    uint8_t fogged=0;        // per-parity bit: colour words currently hold fogged values
    // Only the first record of a chunk uses these two flags. They permit lazy
    // packet rebuilds without acquiring payloads for invisible streamed chunks.
    uint8_t valid_parities=0,chunk_dynamic=0,pad=0;
};
// Per-frame counters: retained objects drawn and packet rebuilds.
struct RetainedStats {uint32_t objects=0,rebuilds=0;};
inline RetainedStats retained_stats;
template<size_t Objects,size_t QuadCapacity> class RetainedGeometry {
public:
    struct State {RetainedKey key[2];const MeshGeometry* geometry=nullptr;uint32_t first_quad=0,quad_count=0;bool allocated=false,any_dynamic=false,stream_validated=false;};
private:
    std::array<State,Objects> states{};
    std::array<RetainedQuad,QuadCapacity> records{};
    uint32_t next_quad=0;
public:
    static constexpr size_t quad_capacity=QuadCapacity;
    void reset(){for(auto& s:states)s=State{};next_quad=0;}
    // A reused entity slot keeps its quads; a different geometry reallocates.
    void forget(size_t index){if(index<Objects){states[index].key[0].valid=false;states[index].key[1].valid=false;}}
    // First fragment slot beyond the pool; the per-frame path allocates downwards from the capacity.
    uint32_t slot_top() const {return next_quad*2;}
    // `limit_slot` is the lowest slot the per-frame path used this frame, which
    // a new allocation must not reach. Freed ranges are not reclaimed until reset().
    bool allocate(size_t index,const MeshGeometry* geometry,size_t quads,uint32_t limit_slot){
        if(index>=Objects||!quads)return false;
        auto& s=states[index];
        if(s.allocated&&s.geometry==geometry&&s.quad_count==quads)return true;
        if(next_quad+quads>QuadCapacity||(next_quad+quads)*2>limit_slot)return false;
        s=State{};s.geometry=geometry;s.first_quad=next_quad;s.quad_count=uint32_t(quads);s.allocated=true;
        next_quad+=uint32_t(quads);
        return true;
    }
    State& state(size_t index){return states[index];}
    RetainedQuad* quads(size_t index){return records.data()+states[index].first_quad;}
    uint32_t first_slot(size_t index) const {return states[index].first_quad*2;}
};
}
