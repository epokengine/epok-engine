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

// The minimap: the walkable cells of the dungeon's plane as rectangles, in cells from
// the minimap grid's corner. A room's rectangles carry its index; a corridor's carry
// -1 and the rooms it opens into, which show it dimly once they are explored.
struct MapRect {
    uint8_t x,y,w,h;
    int8_t room;
    int8_t near[2];
    uint8_t unused;
};
// Visible shows the box; follow keeps the player in its middle at `cell` pixels a cell
// instead of fitting the dungeon; everything skips the fog; full fills the screen;
// right and bottom measure the box's corner from those screen edges.
enum MinimapFlag : uint8_t { minimap_visible=1, minimap_follow=2, minimap_everything=4, minimap_full=8, minimap_right=16, minimap_bottom=32 };
enum MinimapColor : uint8_t { color_room, color_corridor, color_seen, color_background, color_player, color_key, color_lock, color_goal, minimap_colors };
struct Minimap {
    const MapRect* rects=nullptr;
    uint16_t count=0;
    uint8_t columns=0,rows=0;
    // The grid's corner and one cell's step on the dungeon's plane, world raw values:
    // (x, z) in 3D, (x, y) pixels in 2D.
    int32_t corner[2]={0,0},step[2]={1,1};
    // Runtime object slots: the player, each item's actor and each lock's door; -1 when
    // there is none.
    int16_t player=-1;
    const int16_t* item_actors=nullptr;
    const int16_t* lock_actors=nullptr;
    // Explored rectangles, one bit each; rooms explored, one bit each.
    uint32_t* explored=nullptr;
    // Where it draws: the box's corner and size in pixels, pixels per cell when it
    // follows the player, and MinimapFlag bits.
    int16_t x=0,y=0,width=0,height=0;
    uint8_t cell=2,flags=0;
    uint8_t colors[minimap_colors][3]={};
};
inline Minimap minimap{};
inline uint32_t rooms_explored[2]={0,0};
inline void reset_minimap(const Minimap& value) {
    minimap=value;rooms_explored[0]=rooms_explored[1]=0;
    if(minimap.explored)for(uint16_t i=0;i<(minimap.count+31)/32;++i)minimap.explored[i]=0;
}
inline bool rect_explored(uint16_t i) { return minimap.explored&&(minimap.explored[i>>5]>>(i&31)&1); }
inline bool room_explored(int32_t room) { return room>=0&&room<64&&(rooms_explored[room>>5]>>(room&31)&1); }
// Marks a room and every rectangle of it explored.
inline void explore_room(int32_t room) {
    if(room<0||room>=int32_t(map.room_count)||room>=64)return;
    rooms_explored[room>>5]|=1u<<(room&31);
    for(uint16_t i=0;i<minimap.count;++i)if(minimap.rects[i].room==room)minimap.explored[i>>5]|=1u<<(i&31);
}
inline void explore_everything() {
    for(int32_t r=0;r<int32_t(map.room_count)&&r<64;++r)rooms_explored[r>>5]|=1u<<(r&31);
    if(minimap.explored)for(uint16_t i=0;i<minimap.count;++i)minimap.explored[i>>5]|=1u<<(i&31);
}
inline void forget_everything() {
    rooms_explored[0]=rooms_explored[1]=0;
    if(minimap.explored)for(uint16_t i=0;i<(minimap.count+31)/32;++i)minimap.explored[i]=0;
}
inline int32_t floor_divide(int32_t a,int32_t b) { int32_t q=a/b;if((a%b!=0)&&((a<0)!=(b<0)))--q;return q; }
// The minimap cell holding a world point; false outside the grid.
inline bool map_cell(const int32_t* point,int32_t& cx,int32_t& cy) {
    if(!minimap.count||!minimap.step[0]||!minimap.step[1])return false;
    const int axis=map.two_d?1:2;
    cx=floor_divide(point[0]-minimap.corner[0],minimap.step[0]);
    cy=floor_divide(point[axis]-minimap.corner[1],minimap.step[1]);
    return cx>=0&&cy>=0&&cx<minimap.columns&&cy<minimap.rows;
}
// The rectangle holding a minimap cell, or -1.
inline int32_t rect_at(int32_t cx,int32_t cy) {
    for(uint16_t i=0;i<minimap.count;++i){
        const auto& r=minimap.rects[i];
        if(cx>=r.x&&cy>=r.y&&cx<r.x+r.w&&cy<r.y+r.h)return i;
    }
    return -1;
}
// Explores where a point stands: its rectangle, and its whole room the first time.
inline void explore_at(const int32_t* point) {
    int32_t cx,cy;
    if(!map_cell(point,cx,cy))return;
    const int32_t i=rect_at(cx,cy);
    if(i<0)return;
    minimap.explored[i>>5]|=1u<<(i&31);
    const int32_t room=minimap.rects[i].room;
    if(room>=0&&!room_explored(room))explore_room(room);
}
}
