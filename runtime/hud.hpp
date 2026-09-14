#pragma once
#include "hud_core.hpp"
#include "hud-config.hh"
#include "hud-font.hh"
#include "texture.hpp"
#include "psyqo/primitives/rectangles.hh"
#include "psyqo/primitives/sprites.hh"
#include "psyqo/primitives/quads.hh"

namespace epok {
class HudRenderer {
    psyqo::GPU* output=nullptr;
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> rectangles[2][hud_rectangle_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad> images[2][hud_rectangle_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Sprite> glyphs[2][hud_glyph_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2][hud_text_budget];
    // GP0 no-op bookends delimit each parity's block so an unchanged layout is
    // re-chained in one call; the links between the retained fragments survive
    // because only a rebuild writes them.
    struct Nop { uint32_t command=0; };
    psyqo::Fragments::SimpleFragment<Nop> bookends[2][2];
    uint32_t retained_key[2]={};bool retained[2]={};HudStats retained_stats[2];
    size_t used=0,texts=0,rects=0,letters=0,pictures=0;
    static uint32_t mix(uint32_t h,uint32_t v){return (h^v)*16777619u;}
    // Every input the fragment build reads: hierarchy, canvas/rect layout,
    // image, progress and text state, plus the texture bank.
    template<size_t N>static uint32_t layout_key(const std::array<ActorData,N>& entities,size_t count){
        uint32_t h=2166136261u;h=mix(h,uint32_t(count));h=mix(h,uint32_t(reinterpret_cast<uintptr_t>(texture_assets)));
        for(size_t i=0;i<count;++i){
            const auto& e=entities[i];
            h=mix(h,uint32_t(e.alive)|uint32_t(e.active)<<1|uint32_t(e.canvas.enabled)<<2|uint32_t(e.rect.enabled)<<3|uint32_t(e.parent+1)<<8);
            if(!e.canvas.enabled&&!e.rect.enabled)continue;
            for(int k=0;k<2;++k){h=mix(h,uint32_t(e.rect.anchor_min[k].raw()));h=mix(h,uint32_t(e.rect.anchor_max[k].raw()));h=mix(h,uint32_t(e.rect.pivot[k].raw()));h=mix(h,uint32_t(e.rect.position[k].raw()));h=mix(h,uint32_t(e.rect.size[k].raw()));}
            h=mix(h,uint32_t(e.image.enabled)|uint32_t(e.image.color[0])<<8|uint32_t(e.image.color[1])<<16|uint32_t(e.image.color[2])<<24);h=mix(h,uint32_t(e.image.texture));
            for(int k=0;k<4;++k)h=mix(h,uint32_t(e.image.region[k])|uint32_t(e.image.borders[k])<<16);
            h=mix(h,uint32_t(e.progress.enabled)|uint32_t(e.progress.color[0])<<8|uint32_t(e.progress.color[1])<<16|uint32_t(e.progress.color[2])<<24);
            h=mix(h,uint32_t(e.progress.value.raw()));h=mix(h,uint32_t(e.progress.background[0])|uint32_t(e.progress.background[1])<<8|uint32_t(e.progress.background[2])<<16);
            h=mix(h,uint32_t(e.text.enabled)|uint32_t(e.text.wrap)<<1|uint32_t(e.text.color[0])<<8|uint32_t(e.text.color[1])<<16|uint32_t(e.text.color[2])<<24);
            if(e.text.enabled)for(const char* c=e.text.value;*c;++c)h=mix(h,uint8_t(*c));
        }
        return h;
    }
public:
    bool texture_size(int id,int& width,int& height){const auto* t=texture(id);if(!t)return false;width=t->width;height=t->height;return true;}
    void rectangle(int,int x0,int y0,int x1,int y1,const uint8_t* color){
        auto& f=rectangles[output->getParity()][used++];
        f.primitive.position={{.x=int16_t(x0),.y=int16_t(y0)}};
        f.primitive.size={{.w=int16_t(x1-x0),.h=int16_t(y1-y0)}};
        f.primitive.setColor(psyqo::Color{{.r=color[0],.g=color[1],.b=color[2]}}).setOpaque();output->chain(f);
    }
    void image(int,int id,int x0,int y0,int x1,int y1,int u0,int v0,int u1,int v1,const uint8_t* color){
        const auto* t=texture(id);auto& f=images[output->getParity()][pictures++];auto& q=f.primitive;
        q.pointA={{.x=int16_t(x0),.y=int16_t(y0)}};q.pointB={{.x=int16_t(x1),.y=int16_t(y0)}};
        q.pointC={{.x=int16_t(x0),.y=int16_t(y1)}};q.pointD={{.x=int16_t(x1),.y=int16_t(y1)}};
        q.uvA.u=u0;q.uvA.v=v0+(t->y&255);q.uvB.u=u1;q.uvB.v=v0+(t->y&255);
        q.uvC.u=u0;q.uvC.v=v1+(t->y&255);q.uvD.u=u1;q.uvD.v=v1+(t->y&255);
        q.clutIndex=texture_clut(*t);q.tpage=texture_page(*t,BlendMode::Cutout);
        q.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(f);
    }
    void begin_text(){
        auto& page=pages[output->getParity()][texts++];
        page.primitive.attr.setPageX(15).setPageY(1).set(psyqo::Prim::TPageAttr::Tex4Bits).setDithering(false);configure_display_field<display_interlaced>(page.primitive.attr);output->chain(page);
    }
    void glyph(int,unsigned c,int x0,int y0,int x1,int y1,int u,int v,const uint8_t* color){
        unsigned i=c-32;auto& f=glyphs[output->getParity()][letters++];auto& p=f.primitive;
        p.position={{.x=int16_t(x0),.y=int16_t(y0)}};p.size={{.w=int16_t(x1-x0),.h=int16_t(y1-y0)}};
        p.texInfo.u=(i%32)*8+u;p.texInfo.v=192+(i/32)*16+v;p.texInfo.clut=psyqo::PrimPieces::ClutIndex(60,448);
        p.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(f);
    }
    void initialize(psyqo::GPU& gpu){gpu.uploadToVRAM(hud_font_pixels,{{{.x=960,.y=448}},{{.w=64,.h=64}}});}
    template<size_t N>void draw(psyqo::GPU& gpu,std::array<ActorData,N>& entities,size_t count) {
        const unsigned parity=gpu.getParity();
        const uint32_t key=layout_key(entities,count);
        if(retained[parity]&&retained_key[parity]==key){
            gpu.chain(&bookends[parity][0],&bookends[parity][1]);
            hud_stats=retained_stats[parity];
            return;
        }
        used=texts=rects=letters=pictures=0;hud_stats={};
        gpu.chain(bookends[parity][0]);
        int first[N],next[N];
        output=&gpu;
        hud_core::Compiler compiler(*this,display_width,display_height,{hud_layout_budget,hud_rectangle_budget,hud_text_budget,hud_glyph_budget});
        compiler.draw(entities.data(),count,first,next);
        gpu.chain(bookends[parity][1]);
        hud_stats=compiler.stats;
        retained[parity]=true;retained_key[parity]=key;retained_stats[parity]=hud_stats;
    }
    // Scene switches rebuild both parities; slot reuse alone is covered by the key.
    void invalidate(){retained[0]=retained[1]=false;}
};
}
