#pragma once
#include <stdint.h>
namespace epok {
enum class BlendMode {Cutout,Average,Add,Subtract,AddQuarter};
struct Texture {uint16_t width,height,x,y,word_width,clut_x,clut_y;const uint16_t* pixels;const uint16_t* palette;};
}
