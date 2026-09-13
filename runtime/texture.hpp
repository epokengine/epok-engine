#pragma once
#include "texture_types.hpp"
#include "display.hh"
#include "frame_clear.hpp"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/common.hh"
namespace epok {
inline const Texture* texture(int index){return texture_assets&&index>=0&&size_t(index)<texture_count&&texture_assets[index].pixels?&texture_assets[index]:nullptr;}
inline psyqo::PrimPieces::TPageAttr texture_page(const Texture& t,BlendMode blend){psyqo::PrimPieces::TPageAttr p;p.setPageX(t.x/64).setPageY(t.y/256).set(psyqo::Prim::TPageAttr::Tex8Bits).set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(blend==BlendMode::Cutout?0:int(blend)-1)).setDithering(false);configure_display_field<display_interlaced>(p);return p;}
inline psyqo::PrimPieces::ClutIndex texture_clut(const Texture& t){return {uint16_t(t.clut_x/16),t.clut_y};}
inline psyqo::PrimPieces::UVCoords texture_uv(const Texture& t,int32_t u,int32_t v){u=u<0?0:u>4096?4096:u;v=v<0?0:v>4096?4096:v;return {.u=uint8_t(u*(t.width-1)/4096),.v=uint8_t(t.y%256+v*(t.height-1)/4096)};}
inline void textures_initialize(psyqo::GPU& gpu){for(size_t i=0;texture_assets&&i<texture_count;++i){const auto& t=texture_assets[i];if(!t.pixels)continue;gpu.uploadToVRAM(t.pixels,{{{.x=int16_t(t.x),.y=int16_t(t.y)}},{{.w=int16_t(t.word_width),.h=int16_t(t.height)}}});gpu.uploadToVRAM(t.palette,{{{.x=int16_t(t.clut_x),.y=int16_t(t.clut_y)}},{{.w=256,.h=1}}});}}
}
