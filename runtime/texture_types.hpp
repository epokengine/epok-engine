#pragma once
#include <stdint.h>
#include <stddef.h>
namespace epok {
enum class BlendMode {Cutout,Average,Add,Subtract,AddQuarter};
struct Texture {
    uint16_t width,height,x,y,word_width,clut_x,clut_y;
    const uint16_t* pixels;const uint16_t* palette;uint16_t bits=8;
    uint32_t stream_page=UINT32_MAX;uint16_t stream_offset=0;
};
inline constexpr bool texture_present(const Texture& t){return t.pixels||t.stream_page!=UINT32_MAX;}
inline constexpr uint16_t texture_colors(const Texture& t){return t.bits==4?16:256;}
inline constexpr uint16_t texture_base_u(const Texture& t){return uint16_t((t.x%64)*(t.bits==4?4:2));}
inline constexpr uint16_t texture_packed_uv(const Texture& t,int32_t u,int32_t v){
    u=u<0?0:u>4096?4096:u;v=v<0?0:v>4096?4096:v;
    return uint16_t((texture_base_u(t)+u*(t.width-1)/4096)|((t.y%256+v*(t.height-1)/4096)<<8));
}
// Immutable generated banks know their VRAM placement before compilation.
// One word packs CLUT (high half), base TPAGE (low half) and presence (bit 31).
// CLUT addresses use only 15 bits. Palette CONTENTS are intentionally not baked:
// palette animation can update VRAM without invalidating these addresses.
template<size_t N> struct BakedTextureTable { uint32_t words[N]{}; };
template<size_t N>
constexpr BakedTextureTable<N> bake_texture_packets(const Texture (&bank)[N]) {
    BakedTextureTable<N> out{};
    for(size_t i=0;i<N;++i){
        const auto& t=bank[i];if(!texture_present(t))continue;
        const uint32_t clut=uint16_t((t.clut_y<<6)|(t.clut_x/16));
        const uint32_t page=((t.x/64)&15)|(((t.y/256)&1)<<4)
            |(t.bits==4?0:0x80);
        out.words[i]=0x80000000u|(clut<<16)|page;
    }
    return out;
}
struct BakedTexturePacket { uint32_t clut16,tpage16;bool textured; };
template<bool Interlaced,bool Dithering>
inline BakedTexturePacket baked_texture_packet(const uint32_t* bank,size_t count,
    uint16_t index,BlendMode blend) {
    const uint32_t word=bank&&size_t(index)<count?bank[index]:0;
    const uint32_t abr=blend==BlendMode::Cutout?0:uint32_t(blend)-1;
    constexpr uint32_t mode=(Dithering?0x02000000u:0)|(Interlaced?0:0x04000000u);
    return {word&0x7fff0000u,(word<<16)|(abr<<21)|mode,(word>>31)!=0};
}
}
