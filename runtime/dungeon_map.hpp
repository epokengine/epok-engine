#pragma once
#include <stdint.h>
#include "dungeon_random.hpp"

namespace epok::dungeon {
// What the editor baked about the map's generated dungeon, for the Dungeon gameplay
// queries. Positions are world Fixed raw values (Q12): x, y, z in 3D, and pixels in x
// and y with z 0 in 2D. Tables are constant data written by the cook; `reset` points
// the active map at them when the map loads, or clears them when it has no dungeon.
struct RoomData {
    int32_t minimum[3],maximum[3],center[3];
    int16_t level;
    uint16_t path,first_neighbour,first_spot;
    uint8_t flags,depth,to_goal,neighbours,spots;
};
// A key, item, locked door or spot: where it is and which room it belongs to.
struct PointData {
    int32_t position[3];
    int16_t room;
    uint16_t lock;
};
enum RoomFlag : uint8_t { spawn_room=1, goal_room=2, key_room=4, item_room=8, locked_room=16 };
struct Map {
    const RoomData* rooms=nullptr;
    uint16_t room_count=0;
    const uint8_t* neighbours=nullptr;
    const PointData* spots=nullptr;
    const PointData* items=nullptr;
    uint16_t item_count=0;
    const PointData* locks=nullptr;
    uint16_t lock_count=0;
    int16_t spawn=-1,goal=-1;
    uint32_t seed=0;
    bool two_d=false;
};
inline Map map{};
inline void reset(const Map& value) { map=value; }

inline const RoomData* room(int32_t index) {
    return index>=0&&index<int32_t(map.room_count)?&map.rooms[index]:nullptr;
}
// The room whose bounds hold the point, in the dungeon's plane; -1 in corridors and
// outside. Rooms never overlap on that plane, whatever their level.
inline int32_t room_at(const int32_t* point) {
    const int axis=map.two_d?1:2;
    for(uint16_t i=0;i<map.room_count;++i){
        const auto& r=map.rooms[i];
        if(point[0]>=r.minimum[0]&&point[0]<=r.maximum[0]&&point[axis]>=r.minimum[axis]&&point[axis]<=r.maximum[axis])return i;
    }
    return -1;
}
// Breadth-first over the room links. Writes each reached room's predecessor towards
// `from` and returns the number of steps to `to`, or -1 when it cannot be reached.
inline int32_t walk(int32_t from,int32_t to,int8_t* previous) {
    if(!room(from)||!room(to))return -1;
    constexpr uint16_t most=64;
    int8_t steps[most];
    for(uint16_t i=0;i<most;++i){steps[i]=-1;previous[i]=-1;}
    uint8_t queue[most];uint16_t head=0,tail=0;
    steps[from]=0;queue[tail++]=uint8_t(from);
    while(head<tail){
        const uint8_t at=queue[head++];
        if(at==to)return steps[at];
        const auto& r=map.rooms[at];
        for(uint8_t n=0;n<r.neighbours;++n){
            const uint8_t next=map.neighbours[r.first_neighbour+n];
            if(next<most&&steps[next]<0){steps[next]=int8_t(steps[at]+1);previous[next]=int8_t(at);queue[tail++]=next;}
        }
    }
    return -1;
}
// The neighbour of `from` on a shortest way to `to`: `to` itself when they are linked,
// `from` when they are the same room, -1 when there is no way.
inline int32_t next_room(int32_t from,int32_t to) {
    int8_t previous[64];
    const int32_t steps=walk(from,to,previous);
    if(steps<0)return -1;
    if(steps==0)return from;
    int32_t at=to;
    while(previous[at]!=from)at=previous[at];
    return at;
}
// A gameplay random number below `below`, the same for the same stream, index and
// dungeon seed on every run.
inline uint32_t random(uint32_t stream,uint32_t index,uint32_t below) {
    return Random::stream(map.seed^(stream*0x85ebca6bu),index).below(below);
}
}
