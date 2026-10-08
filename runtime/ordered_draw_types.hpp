#pragma once
#include <stdint.h>
namespace epok {
inline constexpr uint32_t ordered_draw_abi=3;
// Published with performance_stats at the end of the same completed frame.
// Packet counts differ from triangle-equivalent buffer-slot accounting.
struct OrderedDrawStats {
    uint32_t frame=0,candidates=0,packets=0,quads=0,cpu_direct=0,clipped_candidates=0,page_switches=0;
    uint32_t origin_changes=0,anchored_vertices=0,anchor_rejections=0;
    uint32_t sample=0xffffffffu,frame_microseconds=0;
    int32_t camera_origin[3]{},camera_basis[12]{},row_scale[2]{};
};
static_assert(sizeof(OrderedDrawStats)==116);
inline OrderedDrawStats ordered_draw_stats;
struct OrderedDrawBlock { uint32_t page; uint16_t offset,first; };
static_assert(sizeof(OrderedDrawBlock)==8);
// Sorted native record ranges; centers are cooked from immutable Q8 bounds.
// These are projection blocks, not new streaming pages or visibility groups.
// Compact blocks store vertices as Q8 offsets from the center; low/high bound
// those offsets so one check per block proves every vertex fits the GTE.
// The trailing fields are renderer state, not cooked data: the block's offset
// for the current camera lattice, computed at most once per epoch (a frame or
// a projection re-anchor). state = epoch << 1 | (every vertex fits).
struct OrderedProjectionBlock {
    uint32_t first;uint16_t count,flags;int32_t center[3];int16_t low[3],high[3];
    uint32_t state,xy;int32_t sum[3];
};
static_assert(sizeof(OrderedProjectionBlock)==52);
inline constexpr uint16_t ordered_projection_compact=1;
// A face's projection block index sits above its two topology bits.
inline uint32_t ordered_face_block(uint16_t topology) { return topology>>2; }
// Exact Q12 world position of a stored vertex, for the CPU and validation
// paths: (relative + center) * 16 + fraction reproduces the cooked Q12 value.
inline void ordered_vertex_world(const OrderedProjectionBlock& block,const uint8_t* entry,int32_t* world) {
    if(!(block.flags&ordered_projection_compact)){
        const auto* q12=reinterpret_cast<const int32_t*>(entry);
        for(unsigned c=0;c<3;++c)world[c]=q12[c];
        return;
    }
    const auto* words=reinterpret_cast<const uint32_t*>(entry);
    const int32_t relative[3]={int16_t(words[0]),int16_t(words[0]>>16),int16_t(words[1])};
    for(unsigned c=0;c<3;++c)
        world[c]=int32_t(uint32_t(relative[c]+block.center[c])*16u+((words[1]>>(16+4*c))&15u));
}
struct OrderedDrawData {
    const OrderedDrawBlock* blocks;
    const uint16_t* sample_blocks;
    const uint8_t* actors;
    uint16_t block_count,sample_count,capacity,actor_count;
    OrderedProjectionBlock* projections=nullptr;
    uint32_t projection_count=0;
};
inline const OrderedProjectionBlock* ordered_projection_block(const OrderedDrawData* data,uint32_t token) {
    if(!data||!data->projections)return nullptr;
    token&=~3u;uint32_t lo=0,hi=data->projection_count;
    while(lo<hi){const uint32_t mid=lo+(hi-lo)/2;
        if(data->projections[mid].first<=token)lo=mid+1;else hi=mid;}
    if(!lo)return nullptr;
    const auto& block=data->projections[lo-1];
    return (token-block.first)/4<block.count?&block:nullptr;
}
}
