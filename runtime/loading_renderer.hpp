#pragma once
#include "transition.hpp"
#include "loading-image.hh"
#include "frame_clear.hpp"
#include "psyqo/gpu.hh"
#include "psyqo/fragments.hh"
#include "psyqo/primitives/rectangles.hh"
#include "psyqo/primitives/sprites.hh"
#include "psyqo/primitives/quads.hh"

namespace epok {
class LoadingRenderer {
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Sprite> letters[2][96];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad> images[2];
    const LoadingImage* uploaded=nullptr;
public:
    void draw(psyqo::GPU& gpu) {
        if(!transition.loading())return;
        const auto& options=transition.options.loading;
        const unsigned parity=gpu.getParity();
        const int right=display_width-display_width/20,bottom=display_height-display_height/20;
        const char* text=transition.phase==TransitionPhase::Failed?"Load failed - START to retry":options.text;
        size_t count=0;while(text&&text[count]&&count<95)++count;
        const int columns=(display_width-display_width/10)/8;
        const int rows=int((count+columns-1)/columns);
        const auto* image=options.image?options.image:&default_loading_image;
        if(image->pixels&&image->width&&image->height&&image->width<=64&&image->height<=64&&image->width%2==0){
            if(uploaded!=image){gpu.uploadToVRAM(image->pixels,{{{.x=960,.y=384}},{{.w=int16_t(image->width),.h=int16_t(image->height)}}});uploaded=image;}
            auto& fragment=images[parity];auto& q=fragment.primitive;
            const int top=bottom-(rows?rows*16+4:0)-image->height,left=right-image->width;
            q.pointA={{.x=int16_t(left),.y=int16_t(top)}};q.pointB={{.x=int16_t(right),.y=int16_t(top)}};
            q.pointC={{.x=int16_t(left),.y=int16_t(top+image->height)}};q.pointD={{.x=int16_t(right),.y=int16_t(top+image->height)}};
            q.uvA.u=0;q.uvA.v=128;q.uvB.u=image->width-1;q.uvB.v=128;
            q.uvC.u=0;q.uvC.v=128+image->height-1;q.uvD.u=image->width-1;q.uvD.v=128+image->height-1;
            q.tpage.setPageX(15).setPageY(1).set(psyqo::Prim::TPageAttr::Tex16Bits);
            q.setColor({{.r=128,.g=128,.b=128}}).setOpaque();gpu.chain(fragment);
        }
        // Wrap long messages inside the TV safe area instead of drawing offscreen.
        auto& page=pages[parity];page.primitive.attr.setPageX(15).setPageY(1).set(psyqo::Prim::TPageAttr::Tex4Bits).setDithering(false);configure_display_field<display_interlaced>(page.primitive.attr);gpu.chain(page);
        for(size_t n=0;n<count;++n){
            const int row=int(n)/columns,col=int(n)%columns;
            const int row_length=int(count)-row*columns<columns?int(count)-row*columns:columns;
            unsigned c=uint8_t(text[n]);if(c<32||c>126)c='?';c-=32;
            auto& f=letters[parity][n];auto& p=f.primitive;
            p.position={{.x=int16_t(right-row_length*8+col*8),.y=int16_t(bottom-(rows-row)*16)}};p.size={{.w=8,.h=16}};
            p.texInfo.u=(c%32)*8;p.texInfo.v=192+(c/32)*16;p.texInfo.clut=psyqo::PrimPieces::ClutIndex(60,448);
            p.setColor({{.r=uint8_t((unsigned(options.color[0])+1)/2),.g=uint8_t((unsigned(options.color[1])+1)/2),.b=uint8_t((unsigned(options.color[2])+1)/2)}}).setOpaque();gpu.chain(f);
        }
        transition.presented=true;
    }
};
}
