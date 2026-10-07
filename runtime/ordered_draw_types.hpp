#pragma once
#include <stdint.h>
namespace epok {
inline constexpr uint32_t ordered_draw_abi=2;
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
struct OrderedProjectionBlock {uint32_t first,count;int32_t center[3];};
static_assert(sizeof(OrderedProjectionBlock)==20);
struct OrderedDrawData {
    const OrderedDrawBlock* blocks;
    const uint16_t* sample_blocks;
    const uint8_t* actors;
    uint16_t block_count,sample_count,capacity,actor_count;
    const OrderedProjectionBlock* projections=nullptr;
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
