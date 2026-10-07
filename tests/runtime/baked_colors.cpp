#include "baked_draw_types.hpp"
#include <cassert>
#include <cstdio>

int main(){
    static uint32_t palette[65536];
    for(unsigned i=0;i<65536;++i)palette[i]=(i*2654435761u)&0xffffffu;
    epok::BakedDrawFace face{};
    uint32_t state=0x51a9471u;
    auto next=[&](){state=state*1664525u+1013904223u;return state;};
    unsigned checked=0;
    for(unsigned sample=0;sample<10000;++sample){
        face.color=sample<2?sample*0xffffffffu:next();
        face.shaded=sample<2?~face.color:next();
        face.flags=uint16_t(next());
        for(unsigned indexed=0;indexed<2;++indexed){
            face.flags=uint16_t((face.flags&~0x10u)|(indexed?0x10u:0));
            for(unsigned part=0;part<4;++part){
                uint32_t result[5]={0,0,0,0,0xdeadbeef};
                epok::baked_ordered_colors(face,part==2,indexed?palette:nullptr,result);
                const unsigned corners[]={part==2?1u:0u,part==2?2u:1u,part==2?3u:2u,3u};
                for(unsigned c=0;c<4;++c)
                    assert(result[c]==epok::baked_corner_color(face,corners[c],indexed?palette:nullptr));
                assert(result[4]==0xdeadbeef);++checked;
            }
        }
    }
    std::printf("Baked colors: %u uniform/indexed ordered layouts match scalar reference, including 0/65535 indices.\n",checked);
}
