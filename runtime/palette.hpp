#pragma once
#include "palette_types.hpp"
#include "texture.hpp"
#include "epok.hpp"
namespace epok {
class PaletteRenderer {
    // Cache only animated textures, by descriptor, not by a reserved CLUT row.
    // Packed CLUTs can share any row and contain either 16 or 256 colors.
    struct State {const Texture* asset=nullptr;uint16_t offset=0;uint8_t first=0,last=0;bool seen=false;};
    State previous[texture_count?texture_count:1];
    size_t previous_count=0;
    alignas(4) uint16_t scratch[256];
    void write(psyqo::GPU& gpu,const Texture& asset,const uint16_t* colors,bool& waited){
        if(!waited){gpu.waitChainIdle();waited=true;}
        const auto count=texture_colors(asset);
        gpu.uploadToVRAM(colors,{{{.x=int16_t(asset.clut_x),.y=int16_t(asset.clut_y)}},{{.w=int16_t(count),.h=1}}});
        ++palette_stats.uploads;palette_stats.bytes+=count*2;
    }
public:
    void clear(){previous_count=0;palette_stats={};}
    // Call before chaining this frame's geometry. Blocking uploads retain no
    // borrowed scratch pointer; wait once before changing any in-flight CLUT.
    template<class Objects> void upload(psyqo::GPU& gpu,const Objects& objects,size_t count) {
        palette_stats={};
        for(size_t i=0;i<previous_count;++i)previous[i].seen=false;
        bool waited=false;
        for(size_t i=0;i<count;++i) {
            const auto& object=objects[i];const auto& animator=object.palette_animator;
            if(!animator.enabled||animator.first==0||animator.last<=animator.first||!is_active_slot(i))continue;
            const auto* asset=texture(animator.texture);if(!asset||!asset->palette)continue;
            size_t slot=0;while(slot<previous_count&&previous[slot].asset!=asset)++slot;
            if(slot==previous_count){previous[previous_count++]=State{.asset=asset};}
            auto& old=previous[slot];
            if(old.seen){++palette_stats.conflicts;continue;}
            old.seen=true;
            const uint16_t length=uint16_t(animator.last)-animator.first+1;
            const auto offset=animator.reverse?uint16_t((length-animator.offset%length)%length):uint16_t(animator.offset%length);
            if(offset==old.offset&&animator.first==old.first&&animator.last==old.last)continue;
            const auto colors=texture_colors(*asset);
            for(unsigned color=0;color<colors;++color) {
                unsigned source=color;
                if(color>=animator.first&&color<=animator.last)source=animator.first+(color-animator.first+offset)%length;
                // A 4-bit palette's omitted 240 entries were all zero before
                // cooking; dynamic animation ranges must not read beyond it.
                scratch[color]=source<colors?asset->palette[source]:0;
            }
            write(gpu,*asset,scratch,waited);
            old.offset=offset;old.first=animator.first;old.last=animator.last;
        }
        for(size_t i=0;i<previous_count;){
            if(previous[i].seen){++i;continue;}
            const auto& asset=*previous[i].asset;
            write(gpu,asset,asset.palette,waited); // restore a disabled/removed controller once
            previous[i]=previous[--previous_count];
        }
    }
};
}
