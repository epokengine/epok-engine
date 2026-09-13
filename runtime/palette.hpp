#pragma once
#include "palette_types.hpp"
#include "texture.hpp"
#include "epok.hpp"
namespace epok {
class PaletteRenderer {
    struct State {const uint16_t* source=nullptr;uint16_t offset=0;uint8_t first=0,last=0;};
    State previous[32];
    alignas(4) uint16_t scratch[256];
public:
    void clear(){for(auto& state:previous)state=State{};palette_stats={};}
    // Call before chaining this frame's geometry. Blocking uploads retain no
    // borrowed scratch pointer; wait once before changing any in-flight CLUT.
    template<class Objects> void upload(psyqo::GPU& gpu,const Objects& objects,size_t count) {
        State desired[32];const Texture* descriptors[32]={};bool controlled[32]={};
        palette_stats={};
        for(size_t i=0;texture_assets&&i<texture_count;++i) {
            const auto& texture=texture_assets[i];
            if(!texture.pixels||!texture.palette||texture.clut_y<480||texture.clut_y>=512)continue;
            auto slot=texture.clut_y-480;descriptors[slot]=&texture;desired[slot].source=texture.palette;
        }
        for(size_t i=0;i<count;++i) {
            const auto& object=objects[i];const auto& animator=object.palette_animator;
            if(!animator.enabled||animator.first==0||animator.last<=animator.first||!is_active_slot(i))continue;
            const auto* asset=texture(animator.texture);if(!asset||asset->clut_y<480||asset->clut_y>=512)continue;
            auto slot=asset->clut_y-480;if(controlled[slot]){++palette_stats.conflicts;continue;}controlled[slot]=true;
            auto& state=desired[slot];state.first=animator.first;state.last=animator.last;
            const uint16_t length=uint16_t(animator.last)-animator.first+1;
            state.offset=animator.reverse?uint16_t((length-animator.offset%length)%length):uint16_t(animator.offset%length);
        }
        bool waited=false;
        for(unsigned slot=0;slot<32;++slot) {
            const auto& state=desired[slot];auto& old=previous[slot];
            if(!state.source){old=State{};continue;}
            if(state.source==old.source&&state.offset==old.offset&&state.first==old.first&&state.last==old.last)continue;
            for(unsigned i=0;i<256;++i) {
                unsigned source=i;
                if(state.first&&i>=state.first&&i<=state.last)source=state.first+(i-state.first+state.offset)%(state.last-state.first+1);
                scratch[i]=state.source[source];
            }
            if(!waited){gpu.waitChainIdle();waited=true;}
            const auto& texture=*descriptors[slot];
            gpu.uploadToVRAM(scratch,{{{.x=int16_t(texture.clut_x),.y=int16_t(texture.clut_y)}},{{.w=256,.h=1}}});
            old=state;++palette_stats.uploads;palette_stats.bytes+=512;
        }
    }
};
}
