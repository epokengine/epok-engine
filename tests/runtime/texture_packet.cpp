#include "texture_types.hpp"
#include "psyqo/primitives/common.hh"
#include <cassert>
#include <cstdio>
#include <cstring>
#include <initializer_list>

template<bool Interlaced,bool Dither> void verify(){
    using namespace epok;
    uint16_t pixels=0,palette=0;
    unsigned checked=0;
    for(unsigned x=0;x<1024;x+=2)for(unsigned y:{0u,1u,255u,256u,511u})
    for(unsigned cx:{0u,16u,640u,1008u})for(unsigned cy:{0u,1u,255u,256u,511u})
    for(unsigned bits:{4u,8u}){
        Texture banks[2]={{4,1,uint16_t(x),uint16_t(y),2,uint16_t(cx),uint16_t(cy),&pixels,&palette,uint16_t(bits)}, {}};
        const auto baked=bake_texture_packets(banks);
        // Resident and streamed descriptors must encode the same packet words.
        banks[0].pixels=nullptr;banks[0].stream_page=0;
        const auto streamed=bake_texture_packets(banks);
        assert(streamed.words[0]==baked.words[0]&&baked.words[1]==0);
        for(unsigned blend=0;blend<5;++blend){
            psyqo::PrimPieces::TPageAttr p;
            p.setPageX(x/64).setPageY(y/256)
                .set(bits==4?psyqo::Prim::TPageAttr::Tex4Bits:psyqo::Prim::TPageAttr::Tex8Bits)
                .set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(blend?blend-1:0)).setDithering(Dither);
            if constexpr(Interlaced)p.disableDisplayArea();else p.enableDisplayArea();
            const psyqo::PrimPieces::ClutIndex clut(cx/16,cy);
            uint16_t cb,pb;std::memcpy(&cb,&clut,2);std::memcpy(&pb,&p,2);
            const auto packet=baked_texture_packet<Interlaced,Dither>(baked.words,2,0,BlendMode(blend));
            assert(packet.textured&&packet.clut16==(uint32_t(cb)<<16)&&packet.tpage16==(uint32_t(pb)<<16));
            ++checked;
        }
        for(uint16_t index:{uint16_t(1),uint16_t(2),uint16_t(65535)})
            assert((!baked_texture_packet<Interlaced,Dither>(baked.words,2,index,BlendMode::Average).textured));
        assert((!baked_texture_packet<Interlaced,Dither>(nullptr,2,0,BlendMode::Cutout).textured));
        assert((!baked_texture_packet<Interlaced,Dither>(baked.words,0,0,BlendMode::Cutout).textured));
    }
    std::printf("Baked texture packets: %u PsyQo comparisons; interlaced=%d dither=%d.\n",checked,Interlaced,Dither);
}
int main(){
    // Constant initialization is required; no constructors or warmup loop at runtime.
    constexpr epok::Texture bank[]={{4,1,0,0,2,0,0,nullptr,nullptr,4,0,0},{}};
    constexpr auto table=epok::bake_texture_packets(bank);
    static_assert(table.words[0]==0x80000000u&&table.words[1]==0);
    static_assert(sizeof(table)==8);
    verify<false,false>();verify<false,true>();verify<true,false>();verify<true,true>();
}
