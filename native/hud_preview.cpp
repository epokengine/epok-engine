#include "hud_commands.hpp"
#include <algorithm>

#include "hud_nodes.hpp"

extern "C" void epok_hud_text_storage(uint32_t* sizes){sizes[0]=sizeof(epok::Text);sizes[1]=sizeof(epok::Text::value);}

extern "C" uint32_t epok_hud_compile(const EpokHudNode* nodes,uint32_t count,const int32_t* dimensions,uint32_t textures,int32_t width,int32_t height,const uint32_t* budget,EpokHudCommand* output,uint32_t capacity,uint32_t* stats,const int32_t* font_table,uint32_t font_words){
    EpokHudSink sink;sink.dimensions.assign(dimensions,dimensions+textures*2);sink.load_fonts(font_table,font_words);
    auto entities=epok_hud_entities(nodes,count);
    std::vector<int> first(count),next(count);
    std::vector<epok::Fixed> measured(size_t(count)*2);std::vector<epok::hud_core::Rect> rects(count);
    std::vector<epok::hud_core::Affine2> transforms(count);
    auto* sizes=reinterpret_cast<epok::Fixed(*)[2]>(measured.data());
    epok::hud_core::Compiler compiler(sink,width,height,{budget[0],budget[1],budget[2],budget[3],budget[4]});
    compiler.draw(entities.data(),count,first.data(),next.data(),sizes,rects.data(),transforms.data());
    auto s=compiler.stats;stats[0]=s.rectangles;stats[1]=s.glyphs;stats[2]=s.texts;stats[3]=s.images;stats[4]=s.dropped;stats[5]=s.rotated;
    uint32_t written=std::min(capacity,uint32_t(sink.commands.size()));
    std::copy_n(sink.commands.data(),written,output);return written;
}
extern "C" uint32_t epok_hud_layout(const EpokHudNode* nodes,uint32_t count,int32_t width,int32_t height,int32_t* out_rects,const int32_t* font_table,uint32_t font_words){
    auto entities=epok_hud_entities(nodes,count);
    std::vector<int> first(count),next(count);
    std::vector<epok::Fixed> measured(size_t(count)*2);std::vector<epok::hud_core::Rect> rects(count);
    std::vector<epok::hud_core::Affine2> transforms(count);
    auto* sizes=reinterpret_cast<epok::Fixed(*)[2]>(measured.data());
    // A label's intrinsic size is measured from its own font, so the arrange
    // pass needs the same table the command compiler gets.
    EpokHudSink sink;sink.load_fonts(font_table,font_words);
    // The editor viewport answers "where is this element", so nothing may be
    // dropped for budget: one layout slot per node is always enough.
    epok::hud_core::Compiler compiler(sink,width,height,{count,0,0,0,0});
    compiler.layout(entities.data(),count,first.data(),next.data(),sizes,rects.data(),transforms.data());
    for(uint32_t i=0;i<count;++i){
        const auto& r=rects[i];int32_t* out=out_rects+size_t(i)*12;
        out[0]=r.x.raw();out[1]=r.y.raw();out[2]=r.w.raw();out[3]=r.h.raw();
        // The four corners the element really occupies, so the viewport can draw
        // a rotated outline and place its handles on it.
        const epok::Fixed cx[4]={r.x,r.x+r.w,r.x,r.x+r.w},cy[4]={r.y+r.h,r.y+r.h,r.y,r.y};
        for(int k=0;k<4;++k){epok::Fixed x,y;epok::hud_core::apply(transforms[i],cx[k],cy[k],x,y);out[4+k*2]=x.raw();out[5+k*2]=y.raw();}
    }
    return count;
}
extern "C" void epok_hud_resolve(const int32_t* parent,const int32_t* rect,int32_t* output){
    using epok::Fixed;
    auto r=epok::hud_core::resolve({Fixed(parent[0],Fixed::RAW),Fixed(parent[1],Fixed::RAW),Fixed(parent[2],Fixed::RAW),Fixed(parent[3],Fixed::RAW)},epok_hud_rect(rect));
    output[0]=r.x.raw();output[1]=r.y.raw();output[2]=r.w.raw();output[3]=r.h.raw();
}
