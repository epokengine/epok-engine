#pragma once
#include "epok.hpp"
#include <array>
#include <stddef.h>
#include <stdint.h>

// Retained GPU packets for static and unlit animated meshes (Project Settings > Rendering >
// Retained Packets). Every quad of a retained object has two preferred fragment
// slots in each parity buffer. Only visible chunks claim them for this frame;
// the dynamic path may borrow all other slots. A rebuild writes the words that do not change
// per frame: command byte and corner colours, page UVs, palette and page
// attributes. Each frame then writes only screen coordinates, links the
// fragment into the ordering table and, when fog touches the quad, its three
// colour words. PsyQo transfers the other parity's chain while a frame is
// built, so a rebuild touches only the parity being built and each parity
// keeps its own key of the inputs baked into its packets.
namespace epok {
// An imported editable mesh owns per-face materials. The actor tint can still
// have its default lit flag even when every face explicitly opts out.
inline bool mesh_faces_unlit(const MeshGeometry* geometry) {
    if(!geometry)return false;
    for(auto* mesh=geometry;mesh;mesh=mesh->next){
        if(!mesh->editable || (mesh->quad_count && !mesh->quads))return false;
        for(size_t q=0;q<mesh->quad_count;++q)if(!mesh->quads[q].material.unlit)return false;
    }
    return true;
}
// Generated topology and face materials are immutable. A new model/scene
// invalidates the pointer; actor tint and lighting remain live renderer inputs.
struct MeshMaterialCache {
    const MeshGeometry* geometry=nullptr;
    bool unlit=false,uniform=false;
    uint32_t uniform_color=0;
    bool faces_unlit(const MeshGeometry* source) {
        if(source!=geometry){
            geometry=source;unlit=mesh_faces_unlit(source);uniform=unlit;
            bool have=false;
            if(unlit)for(auto* mesh=source;mesh;mesh=mesh->next)for(size_t q=0;q<mesh->quad_count;++q){
                const auto& face=mesh->quads[q];
                const uint32_t color=uint32_t(face.material.color[0])|uint32_t(face.material.color[1])<<8|uint32_t(face.material.color[2])<<16;
                if(!have){uniform_color=color;have=true;}
                if(face.vertex_colors||color!=uniform_color)uniform=false;
            }
            uniform&=have;
        }
        return unlit;
    }
};
inline bool retain_mesh_packets(bool skeletal,bool object_lit,bool baked_colors) {
    return !skeletal || (!object_lit && !baked_colors);
}
struct RetainedKey {
    const MeshGeometry* geometry=nullptr;const Texture* bank=nullptr;const uint8_t (*baked)[3]=nullptr;size_t baked_count=0;
    uint32_t generation=0,light_signature=0;int32_t basis[9]={};
    // All-zero defaults keep the pool in .bss; `valid` gates every comparison.
    uint8_t color[3]={};bool unlit=false;int texture=0;BlendMode blend=BlendMode(0);int16_t depth_bias=0;int32_t uv_scroll[2]={};
    bool lighting_enabled=false;ReceiveLighting receive=ReceiveLighting(0);bool valid=false;
    bool same_attributes(const RetainedKey& o) const {
        if(geometry!=o.geometry||bank!=o.bank||baked!=o.baked||baked_count!=o.baked_count||generation!=o.generation||light_signature!=o.light_signature)return false;
        for(int i=0;i<9;++i)if(basis[i]!=o.basis[i])return false;
        return unlit==o.unlit&&texture==o.texture&&blend==o.blend&&depth_bias==o.depth_bias&&uv_scroll[0]==o.uv_scroll[0]&&uv_scroll[1]==o.uv_scroll[1]&&lighting_enabled==o.lighting_enabled&&receive==o.receive;
    }
    bool operator==(const RetainedKey& o) const {
        if(!valid||!o.valid||!same_attributes(o))return false;
        for(int c=0;c<3;++c)if(color[c]!=o.color[c])return false;
        return true;
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
    uint8_t valid_parities=0,chunk_dynamic=0,gpu_colors=0;
    uint32_t attribute_revision=0; // first record only; UV/command source epoch
};
// Per-frame counters: retained objects drawn and packet rebuilds.
struct RetainedStats {uint32_t objects=0,rebuilds=0,build_scanlines=0,quads=0,slot_top=0,dynamic_low=0;};
inline RetainedStats retained_stats;

// Packet cache addresses are not reservations of this frame's GPU budget.
// Claim a visible chunk atomically before rebuilding/linking any of its packets.
// Dynamic packets descend through the unclaimed gaps. A later chunk whose
// preferred range was already borrowed falls back to dynamic emission.
template<size_t Capacity> class FramePacketSlots {
    static_assert(Capacity > 0 && Capacity <= UINT32_MAX);
    std::array<uint32_t,(Capacity+31)/32> claimed{};
    uint32_t next=Capacity;
    uint32_t available_begin=0;
    static uint32_t mask(uint32_t first,uint32_t end,uint32_t word) {
        const uint32_t base=word*32;
        const uint32_t lo=first>base?first-base:0;
        const uint32_t hi=end-base<32?end-base:32;
        return (UINT32_MAX<<lo)&(UINT32_MAX>>(32-hi));
    }
public:
    void begin_frame(){claimed.fill(0);next=Capacity;available_begin=0;}
    uint32_t dynamic_low() const{return next;}
    bool range_claimed(uint32_t first,uint32_t count) const {
        if(!count||first>=Capacity||count>Capacity-first)return false;
        const uint32_t end=first+count,last=(end-1)/32;
        for(uint32_t w=first/32;w<=last;++w){
            const auto bits=mask(first,end,w);
            if((claimed[w]&bits)!=bits)return false;
        }
        return true;
    }
    // A pass emitting many packets may bump through the current free run
    // [run_floor(), dynamic_low()) itself, exactly as allocate_dynamic(_pair)
    // would, and store its new low end before any other allocation.
    uint32_t run_floor() const{return available_begin;}
    void set_dynamic_low(uint32_t low){next=low;}
    bool claim(uint32_t first,uint32_t count){
        if(!count||first>=next||count>next-first)return false;
        const uint32_t end=first+count,last=(end-1)/32;
        for(uint32_t w=first/32;w<=last;++w)
            if(claimed[w]&mask(first,end,w))return false;
        for(uint32_t w=first/32;w<=last;++w)claimed[w]|=mask(first,end,w);
        if(end>available_begin)available_begin=end;
        return true;
    }
    uint32_t allocate_dynamic(){
        // The common path is the same bump allocation as a cache-free frame.
        // Inspect the bitmap only when crossing a claimed range, not per triangle.
        if(next>available_begin)return --next;
        return next_dynamic_run();
    }
    // GT4/G4 need 52/36 bytes including DMA, versus 40/28 for GT3/G3.
    // Reserve two ADJACENT slots atomically before writing a larger packet.
    // Two independent allocate_dynamic() calls could straddle a retained range.
    uint32_t allocate_dynamic_pair(){
        if(next>available_begin&&next-available_begin>=2){next-=2;return next;}
        return next_dynamic_pair();
    }
private:
    // Bitmap traversal is needed only at a retained reservation boundary.
    // Keeping it out of every inlined emitter bounds the R3000 I-cache working
    // set; the common case remains the same checked decrement above.
#if defined(__GNUC__)
    __attribute__((noinline,cold))
#endif
    uint32_t next_dynamic_run(){
        while(next&&(claimed[(next-1)/32]&(1u<<((next-1)&31))))--next;
        available_begin=next;
        while(available_begin&&!(claimed[(available_begin-1)/32]&(1u<<((available_begin-1)&31))))--available_begin;
        return next>available_begin?--next:Capacity;
    }
#if defined(__GNUC__)
    __attribute__((noinline,cold))
#endif
    uint32_t next_dynamic_pair(){
        uint32_t scan=next,run=0;
        while(scan){
            --scan;
            if(claimed[scan/32]&(1u<<(scan&31)))run=0;
            else if(++run==2){
                next=available_begin=scan;
                while(available_begin&&!(claimed[(available_begin-1)/32]&(1u<<((available_begin-1)&31))))--available_begin;
                return scan;
            }
        }
        // Failure does not consume a remaining singleton needed by a triangle.
        return Capacity;
    }
};
template<size_t Objects,size_t QuadCapacity> class RetainedGeometry {
public:
    struct State {RetainedKey key[2];const MeshGeometry* geometry=nullptr;uint32_t first_quad=0,quad_count=0,capacity=0,attribute_revision=0;uint8_t attribute_parity=0;bool allocated=false,any_dynamic=false,stream_validated=false,packets_invalid[2]={};};
private:
    std::array<State,Objects> states{};
    std::array<RetainedQuad,QuadCapacity> records{};
    uint32_t next_quad=0;
public:
    static constexpr size_t quad_capacity=QuadCapacity;
    void reset(){for(auto& s:states)s=State{};next_quad=0;}
    // A reused entity slot keeps its resident range, including across LODs.
    void forget(size_t index){if(index<Objects){states[index].key[0].valid=false;states[index].key[1].valid=false;states[index].packets_invalid[0]=states[index].packets_invalid[1]=true;}}
    void prepare_key(size_t index,unsigned parity,const RetainedKey& key,bool allow_recolor){
        auto& s=states[index];if(s.key[parity]==key)return;
        const bool recolor=allow_recolor&&!s.any_dynamic&&!s.packets_invalid[parity]&&
            s.key[parity].geometry&&s.key[parity].same_attributes(key);
        if(!s.attribute_revision||!s.key[s.attribute_parity].same_attributes(key)){
            if(!++s.attribute_revision){s.attribute_revision=1;for(uint32_t q=0;q<s.quad_count;++q)records[s.first_quad+q].attribute_revision=0;}
        }
        if(s.key[1-parity].valid&&!(s.key[1-parity]==key))s.key[1-parity].valid=false;
        s.key[parity]=key;s.attribute_parity=uint8_t(parity);s.any_dynamic=false;s.packets_invalid[parity]=false;
        const uint8_t bit=uint8_t(1u<<parity),pending=uint8_t(bit<<2);
        for(uint32_t q=0;q<s.quad_count;++q){
            auto& r=records[s.first_quad+q];
            // Only an owned packet and a chunk with the exact current command,
            // UV and material epoch can keep those words during a tint change.
            if(recolor&&(r.valid_parities&bit)&&r.attribute_revision==s.attribute_revision&&!r.chunk_dynamic)r.valid_parities|=pending;
            else r.valid_parities&=uint8_t(~(bit|pending));
        }
    }
    // First preferred fragment slot beyond the cache (not a per-frame reservation).
    uint32_t slot_top() const {return next_quad*2;}
    // `limit_slot` is the packet-buffer capacity. Ranges describe cache addresses;
    // FramePacketSlots separately arbitrates their use in the current frame.
    // Freed metadata ranges are not reclaimed until reset().
    bool allocate(size_t index,const MeshGeometry* geometry,size_t quads,uint32_t limit_slot){
        if(index>=Objects||!quads)return false;
        auto& s=states[index];
        if(s.allocated&&s.geometry==geometry&&s.quad_count==quads)return true;
        if(s.allocated&&quads<=s.capacity){
            const auto first=s.first_quad,capacity=s.capacity;
            s=State{};s.geometry=geometry;s.first_quad=first;s.quad_count=uint32_t(quads);s.capacity=capacity;s.allocated=true;
            return true;
        }
        if(next_quad+quads>QuadCapacity||(next_quad+quads)*2>limit_slot)return false;
        s=State{};s.geometry=geometry;s.first_quad=next_quad;s.quad_count=s.capacity=uint32_t(quads);s.allocated=true;
        next_quad+=uint32_t(quads);
        return true;
    }
    // Dynamic writes can replace dormant cached packets. Invalidate their keys
    // after rendering, even for invisible objects; leave the other parity alone.
    // The descending allocator's low watermark is conservative across claimed gaps.
    void invalidate_borrowed(unsigned parity,uint32_t first_slot){
        if(first_slot>=slot_top())return;
        for(auto& s:states)if(s.allocated&&(s.first_quad+s.quad_count)*2>first_slot){
            s.key[parity&1].valid=false;s.packets_invalid[parity&1]=true;
        }
    }
    // A descending allocation can cross protected chunks without writing them.
    // Keep their attributes and invalidate only the dormant chunks it borrowed.
    template<size_t Capacity,class Borrowed>
    void invalidate_borrowed(unsigned parity,uint32_t first_slot,
                             const FramePacketSlots<Capacity>& slots,Borrowed borrowed){
        if(first_slot>=slot_top())return;
        parity&=1;const uint8_t bits=uint8_t((1u<<parity)|(4u<<parity));
        for(size_t index=0;index<Objects;++index){
            auto& s=states[index];
            if(!s.allocated||(s.first_quad+s.quad_count)*2<=first_slot)continue;
            uint32_t total=0;
            for(auto* mesh=s.geometry;mesh;mesh=mesh->next)total+=uint32_t(mesh->quad_count);
            if(total!=s.quad_count){
                s.key[parity].valid=false;s.packets_invalid[parity]=true;
                borrowed(index,0);continue;
            }
            uint32_t offset=0;
            for(auto* mesh=s.geometry;mesh;mesh=mesh->next){
                const uint32_t count=uint32_t(mesh->quad_count),first=(s.first_quad+offset)*2;
                if(count&&first+count*2>first_slot&&!slots.range_claimed(first,count*2)){
                    records[s.first_quad+offset].valid_parities&=uint8_t(~bits);
                    borrowed(index,offset);
                }
                offset+=count;
            }
        }
    }
    State& state(size_t index){return states[index];}
    RetainedQuad* quads(size_t index){return records.data()+states[index].first_quad;}
    uint32_t first_slot(size_t index) const {return states[index].first_quad*2;}
};
}
