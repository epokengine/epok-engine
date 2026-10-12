#pragma once
#include "texture_types.hpp"
#include "font_types.hpp"
#include "display.hh"
#include "frame_clear.hpp"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/common.hh"
namespace epok {
inline const Texture* texture(int index){return texture_assets&&index>=0&&size_t(index)<texture_count&&texture_present(texture_assets[index])?&texture_assets[index]:nullptr;}
inline psyqo::PrimPieces::TPageAttr texture_page(const Texture& t,BlendMode blend,bool dithering=false){psyqo::PrimPieces::TPageAttr p;p.setPageX(t.x/64).setPageY(t.y/256).set(t.bits==4?psyqo::Prim::TPageAttr::Tex4Bits:psyqo::Prim::TPageAttr::Tex8Bits).set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(blend==BlendMode::Cutout?0:int(blend)-1)).setDithering(dithering);configure_display_field<display_interlaced>(p);return p;}
inline psyqo::PrimPieces::ClutIndex texture_clut(const Texture& t){return {uint16_t(t.clut_x/16),t.clut_y};}
inline psyqo::PrimPieces::UVCoords texture_uv(const Texture& t,int32_t u,int32_t v){auto uv=texture_packed_uv(t,u,v);return {.u=uint8_t(uv),.v=uint8_t(uv>>8)};}
// Authored fonts are 4bpp, so a row is a quarter of its pixel width in VRAM
// words, and the CLUT is sixteen entries rather than 256. They are resident for
// the whole run: the allocator places them after the textures of every bank.
inline void fonts_initialize(psyqo::GPU& gpu,const Font* fonts,size_t count){for(size_t i=0;fonts&&i<count;++i){const auto& f=fonts[i];if(!f.pixels)continue;gpu.uploadToVRAM(f.pixels,{{{.x=int16_t(f.x),.y=int16_t(f.y)}},{{.w=int16_t(f.width/4),.h=int16_t(f.height)}}});gpu.uploadToVRAM(f.palette,{{{.x=int16_t(f.clut_x),.y=int16_t(f.clut_y)}},{{.w=16,.h=1}}});}}
}
