#pragma once
#include <cstdint>
#include <cstddef>
#include <vector>
#include "texture_types.hpp"
namespace psyqo {
struct Point {int16_t x,y;};
struct Size {int16_t w,h;};
struct Position {Point point;};
struct Dimensions {Size size;};
struct Rect {Position position;Dimensions dimensions;};
struct GPU {
    struct Upload {int16_t x,y;std::vector<uint16_t> colors;};
    unsigned waits=0;
    std::vector<Upload> uploads;
    void waitChainIdle(){++waits;}
    void uploadToVRAM(const uint16_t* data,Rect r){
        uploads.push_back({r.position.point.x,r.position.point.y,
            std::vector<uint16_t>(data,data+r.dimensions.size.w*r.dimensions.size.h)});
    }
};
}
namespace epok {
inline constexpr size_t texture_count=197;
inline Texture texture_assets[texture_count];
inline bool active_slots[texture_count];
inline bool is_active_slot(size_t i){return active_slots[i];}
inline const Texture* texture(int i){return i>=0&&size_t(i)<texture_count&&texture_present(texture_assets[i])?&texture_assets[i]:nullptr;}
}
