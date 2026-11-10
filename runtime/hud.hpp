#pragma once
#include "hud_core.hpp"
#include "hud-config.hh"
#include "hud-font.hh"
#include "fonts.hh"
#include "texture.hpp"
#include "psyqo/primitives/rectangles.hh"
#include "psyqo/primitives/sprites.hh"
#include "psyqo/primitives/quads.hh"
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
#include "common/hardware/counters.h"
#endif

namespace epok {
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
// Diagnostic builds only. Times are hardware counter-1 scanlines; the peak is
// one complete HUD draw, so its phase values belong to the same invocation.
struct HudProfileDetail {
    uint32_t draw=0,parity=0,retained=0,key_scanlines=0,layout_scanlines=0,emit_scanlines=0,total_scanlines=0;
};
inline HudProfileDetail hud_profile_detail,hud_profile_peak;
#endif
class HudRenderer {
    psyqo::GPU* output=nullptr;
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> rectangles[2][hud_rectangle_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad> images[2][hud_rectangle_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Sprite> glyphs[2][hud_glyph_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2][hud_text_budget];
    // Rotation leaves the rectangle/sprite classes behind: GP0 0b011 rasterizes
    // axis-aligned only, so a rotated element draws from these polygon pools.
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Quad> rotated_quads[2][hud_rotated_budget];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedQuad> rotated_textured[2][hud_rotated_budget];
    // GP0 no-op bookends delimit each parity's block so an unchanged layout is
    // re-chained in one call; the links between the retained fragments survive
    // because only a rebuild writes them.
    struct Nop { uint32_t command=0; };
    psyqo::Fragments::SimpleFragment<Nop> bookends[2][2];
    uint32_t retained_key[2]={};bool retained[2]={};HudStats retained_stats[2];
    static constexpr size_t node_capacity=objects.size();
    hud_core::NodeCache<node_capacity> nodes;
    psyqo::Fragments::SimpleFragment<Nop> node_bookends[2][node_capacity][2];
    uint32_t node_keys[node_capacity]={};
    using ImageReuse=hud_core::ImageReuseCache<8>;
    static_assert(sizeof(ImageReuse)+sizeof(ActorData*)+sizeof(size_t)<=1024,"Image packet reuse must stay within its1KiB state budget");
    ImageReuse image_reuse;
    const ActorData* current_entities=nullptr;size_t current_count=0;
    size_t used=0,texts=0,rects=0,letters=0,pictures=0,turned=0,turned_textured=0;
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
    HudProfileDetail profile_work;
    uint16_t profile_layout_begin=0,profile_emit_begin=0;
    static void publish_profile(HudProfileDetail& destination,HudProfileDetail value){
        const uint32_t draw=value.draw;value.draw=0;
        destination.draw=0;asm volatile("":::"memory");
        destination=value;asm volatile("":::"memory");
        destination.draw=draw;
    }
    void profile_finished(uint16_t begin){
        profile_work.total_scanlines=uint16_t(COUNTERS[1].value-begin);
        publish_profile(hud_profile_detail,profile_work);
        if(profile_work.total_scanlines>hud_profile_peak.total_scanlines)publish_profile(hud_profile_peak,profile_work);
    }
#endif
    static uint32_t mix(uint32_t h,uint32_t v){return hud_core::render_key_mix(h,v);}
    // Every input the fragment build reads: hierarchy, canvas/rect layout,
    // image, progress and text state, plus the texture bank.
    template<size_t N>uint32_t layout_key(const std::array<ActorData,N>& entities,size_t count){
        uint32_t h=2166136261u;h=mix(h,uint32_t(count));
        h=mix(h,uint32_t(reinterpret_cast<uintptr_t>(texture_assets)));
        h=mix(h,uint32_t(reinterpret_cast<uintptr_t>(font_assets)));
        for(size_t i=0;i<count;++i){
            node_keys[i]=hud_core::render_node_key(entities[i],reinterpret_cast<uintptr_t>(texture_assets),reinterpret_cast<uintptr_t>(font_assets));
            h=mix(h,node_keys[i]);
        }
        return h;
    }
public:
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
    void layout_finished(){
        const uint16_t now=COUNTERS[1].value;
        profile_work.layout_scanlines=uint16_t(now-profile_layout_begin);
        profile_emit_begin=now;
    }
#endif
    using NodeCounts=typename hud_core::NodeCache<node_capacity>::Counts;
    NodeCounts counts()const{return {{uint32_t(used),uint32_t(texts),uint32_t(letters),uint32_t(pictures),uint32_t(turned),uint32_t(turned_textured)}};}
    bool begin_node(int owner,const hud_core::Rect& rect,const hud_core::Affine2& transform,const uint8_t* tint,bool outlined,HudStats& stats){
        uint32_t key=node_keys[owner];
        for(auto value:{rect.x,rect.y,rect.w,rect.h,transform.c,transform.s,transform.tx,transform.ty})key=mix(key,uint32_t(value.raw()));
        key=mix(key,uint32_t(tint[0])|uint32_t(tint[1])<<8|uint32_t(tint[2])<<16|uint32_t(outlined)<<24);
        auto offsets=counts();const unsigned parity=output->getParity();
        image_reuse.capture(parity,size_t(owner),current_entities,current_count,rect,transform,uint32_t(used),
                            reinterpret_cast<uintptr_t>(texture_assets),reinterpret_cast<uintptr_t>(font_assets));
        if(nodes.begin(size_t(owner),key,offsets,stats)){
            used=offsets.values[0];texts=offsets.values[1];letters=offsets.values[2];pictures=offsets.values[3];turned=offsets.values[4];turned_textured=offsets.values[5];
            image_reuse.end(parity,size_t(owner),uint32_t(used));
            output->chain(&node_bookends[parity][owner][0],&node_bookends[parity][owner][1]);return true;
        }
        output->chain(node_bookends[parity][owner][0]);return false;
    }
    void end_node(int owner,const HudStats& stats){nodes.end(size_t(owner),counts(),stats);image_reuse.end(output->getParity(),size_t(owner),uint32_t(used));output->chain(node_bookends[output->getParity()][owner][1]);}
    bool texture_size(int id,int& width,int& height){const auto* t=texture(id);if(!t)return false;width=t->width;height=t->height;return true;}
    template<class Rectangle> void write_rectangle(unsigned parity,size_t index,const Rectangle& rectangle){
        auto& value=rectangles[parity][index].primitive;
        value.position={{.x=int16_t(rectangle.x0),.y=int16_t(rectangle.y0)}};
        value.size={{.w=int16_t(rectangle.x1-rectangle.x0),.h=int16_t(rectangle.y1-rectangle.y0)}};
        value.setColor(psyqo::Color{{.r=rectangle.color[0],.g=rectangle.color[1],.b=rectangle.color[2]}}).setOpaque();
    }
    void rectangle(int,int x0,int y0,int x1,int y1,const uint8_t* color){
        const unsigned parity=output->getParity();const size_t index=used++;
        write_rectangle(parity,index,hud_core::ImageRectangle{x0,y0,x1,y1,{color[0],color[1],color[2]}});output->chain(rectangles[parity][index]);
    }
    void image(int,int id,int x0,int y0,int x1,int y1,int u0,int v0,int u1,int v1,const uint8_t* color){
        const auto* t=texture(id);auto& f=images[output->getParity()][pictures++];auto& q=f.primitive;
        q.pointA={{.x=int16_t(x0),.y=int16_t(y0)}};q.pointB={{.x=int16_t(x1),.y=int16_t(y0)}};
        q.pointC={{.x=int16_t(x0),.y=int16_t(y1)}};q.pointD={{.x=int16_t(x1),.y=int16_t(y1)}};
        const auto base_u=texture_base_u(*t);
        q.uvA.u=base_u+u0;q.uvA.v=v0+(t->y&255);q.uvB.u=base_u+u1;q.uvB.v=v0+(t->y&255);
        q.uvC.u=base_u+u0;q.uvC.v=v1+(t->y&255);q.uvD.u=base_u+u1;q.uvD.v=v1+(t->y&255);
        q.clutIndex=texture_clut(*t);q.tpage=texture_page(*t,BlendMode::Cutout);
        q.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(f);
    }
    // Font index -1 is the built-in atlas, resident at (960,448) with its CLUT
    // at (60,448); an authored font brings its own placement from the export
    // allocator. Both are 4bpp, so one TPage per text element still serves.
    const Font* font(int index){return font_assets&&index>=0&&size_t(index)<font_count?&font_assets[index]:nullptr;}
    void begin_text(int index){
        const Font* f=font(index);
        auto& page=pages[output->getParity()][texts++];
        page.primitive.attr.setPageX(f?f->x/64:15).setPageY(f?f->y/256:1).set(psyqo::Prim::TPageAttr::Tex4Bits).setDithering(false);configure_display_field<display_interlaced>(page.primitive.attr);output->chain(page);
    }
    // Source coordinates arrive as atlas texels; the page origin turns them into
    // the texel offsets within the page begin_text() chained.
    void glyph(int,int index,int u,int v,int x0,int y0,int x1,int y1,const uint8_t* color){
        const Font* f=font(index);auto& fragment=glyphs[output->getParity()][letters++];auto& p=fragment.primitive;
        p.position={{.x=int16_t(x0),.y=int16_t(y0)}};p.size={{.w=int16_t(x1-x0),.h=int16_t(y1-y0)}};
        p.texInfo.u=uint8_t(u+(f?(f->x%64)*4:0));p.texInfo.v=uint8_t(v+(f?f->y%256:192));
        p.texInfo.clut=f?psyqo::PrimPieces::ClutIndex(uint16_t(f->clut_x/16),f->clut_y):psyqo::PrimPieces::ClutIndex(60,448);
        p.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(fragment);
    }
    // Rotated fills, images and glyphs. The corner arrays arrive in the A/B/C/D
    // Z order the polygon class wants, so nothing is reordered here.
    void quad(int,const int* x,const int* y,const uint8_t* color){
        auto& f=rotated_quads[output->getParity()][turned++];auto& q=f.primitive;
        q.pointA={{.x=int16_t(x[0]),.y=int16_t(y[0])}};q.pointB={{.x=int16_t(x[1]),.y=int16_t(y[1])}};
        q.pointC={{.x=int16_t(x[2]),.y=int16_t(y[2])}};q.pointD={{.x=int16_t(x[3]),.y=int16_t(y[3])}};
        q.setColor(psyqo::Color{{.r=color[0],.g=color[1],.b=color[2]}}).setOpaque();output->chain(f);
    }
    void textured_quad(int,int id,const int* x,const int* y,const int* u,const int* v,const uint8_t* color){
        const auto* t=texture(id);if(!t)return;
        auto& f=rotated_textured[output->getParity()][turned_textured++];auto& q=f.primitive;
        q.pointA={{.x=int16_t(x[0]),.y=int16_t(y[0])}};q.pointB={{.x=int16_t(x[1]),.y=int16_t(y[1])}};
        q.pointC={{.x=int16_t(x[2]),.y=int16_t(y[2])}};q.pointD={{.x=int16_t(x[3]),.y=int16_t(y[3])}};
        q.uvA.u=u[0];q.uvA.v=v[0]+(t->y&255);q.uvB.u=u[1];q.uvB.v=v[1]+(t->y&255);
        q.uvC.u=u[2];q.uvC.v=v[2]+(t->y&255);q.uvD.u=u[3];q.uvD.v=v[3]+(t->y&255);
        q.clutIndex=texture_clut(*t);q.tpage=texture_page(*t,BlendMode::Cutout);
        q.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(f);
    }
    // The same page, CLUT and atlas texels glyph() uses; only the primitive
    // class differs, and a TexturedQuad carries its own page rather than
    // inheriting the one begin_text() chained.
    void glyph_quad(int,int index,const int* x,const int* y,const int* u,const int* v,const uint8_t* color){
        const Font* font_of=font(index);
        auto& f=rotated_textured[output->getParity()][turned_textured++];auto& q=f.primitive;
        q.pointA={{.x=int16_t(x[0]),.y=int16_t(y[0])}};q.pointB={{.x=int16_t(x[1]),.y=int16_t(y[1])}};
        q.pointC={{.x=int16_t(x[2]),.y=int16_t(y[2])}};q.pointD={{.x=int16_t(x[3]),.y=int16_t(y[3])}};
        const int base_u=font_of?int((font_of->x%64)*4):0,base_v=font_of?int(font_of->y%256):192;
        q.uvA.u=uint8_t(base_u+u[0]);q.uvA.v=uint8_t(base_v+v[0]);q.uvB.u=uint8_t(base_u+u[1]);q.uvB.v=uint8_t(base_v+v[1]);
        q.uvC.u=uint8_t(base_u+u[2]);q.uvC.v=uint8_t(base_v+v[2]);q.uvD.u=uint8_t(base_u+u[3]);q.uvD.v=uint8_t(base_v+v[3]);
        q.clutIndex=font_of?psyqo::PrimPieces::ClutIndex(uint16_t(font_of->clut_x/16),font_of->clut_y):psyqo::PrimPieces::ClutIndex(60,448);
        psyqo::PrimPieces::TPageAttr page;
        page.setPageX(font_of?font_of->x/64:15).setPageY(font_of?font_of->y/256:1).set(psyqo::Prim::TPageAttr::Tex4Bits).setDithering(false);
        configure_display_field<display_interlaced>(page);q.tpage=page;
        q.setColor({{.r=uint8_t((unsigned(color[0])+1)/2),.g=uint8_t((unsigned(color[1])+1)/2),.b=uint8_t((unsigned(color[2])+1)/2)}}).setOpaque();output->chain(f);
    }
    void initialize(psyqo::GPU& gpu){gpu.uploadToVRAM(hud_font_pixels,{{{.x=960,.y=448}},{{.w=64,.h=64}}});fonts_initialize(gpu,font_assets,font_count);}
    template<size_t N>void draw(psyqo::GPU& gpu,std::array<ActorData,N>& entities,size_t count) {
        const unsigned parity=gpu.getParity();
        bool allow_image_reuse=true;
#ifdef EPOK_HUD_IMAGE_REUSE_DIAGNOSTICS
        auto& diagnostic=hud_core::image_reuse_diagnostics;
        ++diagnostic.draws;diagnostic.state_bytes=sizeof(ImageReuse)+sizeof(current_entities)+sizeof(current_count);
        allow_image_reuse=diagnostic.enabled;
        if(diagnostic.invalidate_next){invalidate();diagnostic.invalidate_next=false;}
#endif
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
        const uint16_t profile_begin=COUNTERS[1].value;
        profile_work={hud_profile_detail.draw+1,parity};
#endif
        const uint32_t key=layout_key(entities,count);
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
        profile_work.key_scanlines=uint16_t(COUNTERS[1].value-profile_begin);
#endif
        if(retained[parity]&&retained_key[parity]==key){
#ifdef EPOK_HUD_IMAGE_REUSE_DIAGNOSTICS
            ++diagnostic.cached_draws;
#endif
            gpu.chain(&bookends[parity][0],&bookends[parity][1]);
            hud_stats=retained_stats[parity];
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
            profile_work.retained=1;profile_finished(profile_begin);
#endif
            return;
        }
        if(allow_image_reuse&&retained[parity]&&!retained_stats[parity].dropped){
            ImageReuse::Patch patches[8];unsigned count_of_patches=0;
            if(image_reuse.prepare(parity,entities.data(),count,node_keys,reinterpret_cast<uintptr_t>(texture_assets),reinterpret_cast<uintptr_t>(font_assets),
                                   display_width,display_height,patches,count_of_patches)){
                for(unsigned n=0;n<count_of_patches;++n){const auto& patch=patches[n];
                    for(unsigned i=0;i<patch.rectangles.count;++i)write_rectangle(parity,patch.first+i,patch.rectangles.values[i]);
                    // The packets now carry a different appearance from the
                    // node's last full compiler key. Force its next full emit.
                    nodes.invalidate_node(parity,patch.owner);
                    image_reuse.applied(parity,patch,node_keys[patch.owner]);
                }
                retained_key[parity]=key;hud_stats=retained_stats[parity];
#ifdef EPOK_HUD_IMAGE_REUSE_DIAGNOSTICS
                ++diagnostic.patched_draws;diagnostic.patched_images+=count_of_patches;
#endif
                gpu.chain(&bookends[parity][0],&bookends[parity][1]);
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
                profile_work.retained=1;profile_work.emit_scanlines=uint16_t(COUNTERS[1].value-profile_begin)-profile_work.key_scanlines;profile_finished(profile_begin);
#endif
                return;
            }
        }
        used=texts=rects=letters=pictures=turned=turned_textured=0;hud_stats={};
#ifdef EPOK_HUD_IMAGE_REUSE_DIAGNOSTICS
        ++diagnostic.full_compiles;
#endif
        nodes.rebuild(parity);
        image_reuse.begin(parity);current_entities=entities.data();current_count=count;
        gpu.chain(bookends[parity][0]);
        int first[N],next[N];Fixed measured[N][2];hud_core::Rect rects[N];hud_core::Affine2 transforms[N];
        output=&gpu;
        hud_core::Compiler compiler(*this,display_width,display_height,{hud_layout_budget,hud_rectangle_budget,hud_text_budget,hud_glyph_budget,hud_rotated_budget});
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
        profile_layout_begin=COUNTERS[1].value;
#endif
        compiler.draw(entities.data(),count,first,next,measured,rects,transforms);
        gpu.chain(bookends[parity][1]);
        hud_stats=compiler.stats;
        retained[parity]=true;retained_key[parity]=key;retained_stats[parity]=hud_stats;
        image_reuse.seal(parity,entities.data(),count,node_keys,reinterpret_cast<uintptr_t>(texture_assets),reinterpret_cast<uintptr_t>(font_assets));
#if defined(EPOK_PROFILE_DETAIL) && defined(__mips__)
        profile_work.emit_scanlines=uint16_t(COUNTERS[1].value-profile_emit_begin);
        profile_finished(profile_begin);
#endif
    }
    // Scene switches rebuild both parities; slot reuse alone is covered by the key.
    void invalidate(){retained[0]=retained[1]=false;nodes.invalidate();image_reuse.invalidate();}
};
}
