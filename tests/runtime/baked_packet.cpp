#if defined(_WIN32)
#define _CRT_SECURE_NO_WARNINGS
#endif
#include "../../runtime/baked_packet.hpp"
#include <cassert>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <vector>

static void emit(bool textured,bool quad,uint32_t* out,const uint32_t* rgb,const uint32_t* xy,
    epok::BakedTextureWords uv,uint32_t command){
    if(textured){if(quad)epok::BakedPacket<true,true>::write(out,rgb,xy,uv,command);
        else epok::BakedPacket<true,false>::write(out,rgb,xy,uv,command);}
    else {if(quad)epok::BakedPacket<false,true>::write(out,rgb,xy,uv,command);
        else epok::BakedPacket<false,false>::write(out,rgb,xy,uv,command);}
}
int main(){
    static_assert(epok::BakedPacket<false,false>::bytes_with_dma==28);
    static_assert(epok::BakedPacket<false,true>::bytes_with_dma==36);
    static_assert(epok::BakedPacket<true,false>::bytes_with_dma==40);
    static_assert(epok::BakedPacket<true,true>::bytes_with_dma==52);
    const uint32_t rgb[]={0x80b9ff,0,0xffffff,0x123456},xy[]={0xff800100,1,2,3};
    const epok::BakedTextureWords uv{0x12340201,0x40560403,0x08070605};
    const auto second=epok::baked_second_half(uv);
    assert(second.uv_clut==0x12340403&&second.uv_tpage==0x40560605&&second.uv34==0x0807);
    for(bool textured:{false,true})for(bool quad:{false,true})for(uint32_t command:{0x34000000u,0x36000000u}){
        uint32_t out[13];for(auto& w:out)w=0xdeadbeef;
        emit(textured,quad,out,rgb,xy,uv,command);
        const unsigned count=(textured?3:2)*(quad?4:3);
        assert(out[count]==0xdeadbeef&&out[0]==((textured?command:0x30000000u)|(quad?0x08000000u:0)|rgb[0]));
        for(unsigned c=0;c<(quad?4u:3u);++c){const unsigned at=c*(textured?3:2);
            assert((out[at]&0xffffff)==rgb[c]&&out[at+1]==xy[c]);}
    }
    epok::MonotoneDrawBucket<2048> order;
    assert(order.next(300)==300&&order.next(1600)==300&&order.next(200)==200&&order.next(1000)==200);
    assert(order.next(0)==0&&order.next(2047)==0);
    if(const auto* path=std::getenv("EPOK_BAKED_PACKET_FIXTURE")){
        std::ifstream stream(path,std::ios::binary);assert(stream);
        const std::vector<uint8_t> raw{std::istreambuf_iterator<char>(stream),std::istreambuf_iterator<char>()};
        assert(raw.size()>=12&&std::string(raw.begin(),raw.begin()+8)=="EPOKPKT1");size_t cursor=8;
        auto read=[&](){assert(cursor+4<=raw.size());uint32_t v=0;for(unsigned i=0;i<4;++i)v|=uint32_t(raw[cursor++])<<(i*8);return v;};
        const auto count=read();
        for(uint32_t i=0;i<count;++i){
            const auto kind=read(),command=read();uint32_t colors[4],screen[4];
            for(auto& v:colors)v=read();for(auto& v:screen)v=read();
            epok::BakedTextureWords texture{read(),read(),read()};
            const auto words=read();const bool textured=kind&1,quad=kind&2,half=kind&4;
            assert(kind<8&&!(quad&&half)&&words==(textured?3u:2u)*(quad?4u:3u));
            if(half)texture=epok::baked_second_half(texture);
            uint32_t output[13];for(auto& v:output)v=0xdeadbeef;
            emit(textured,quad,output,colors,screen,texture,command);
            for(unsigned w=0;w<12;++w){const auto expected=read();assert(output[w]==expected);}
            assert(output[12]==0xdeadbeef);
        }
        assert(cursor==raw.size());std::cout<<"Baked packets: "<<count<<" complete payloads match original MIPS execution.\n";
    }
    std::cout<<"G3/G4/GT3/GT4 words, raw Gouraud, half-quad UV slots and monotone buckets pass.\n";
}
