#include "hud_commands.hpp"
#include <algorithm>

static std::vector<epok::ActorData> epok_hud_entities(const EpokHudNode* nodes,uint32_t count){
    std::vector<epok::ActorData> entities(count);
    for(uint32_t i=0;i<count;++i){
        const auto& n=nodes[i];auto& e=entities[i];e.parent=n.parent;
        e.alive=n.flags&1;e.active=n.flags&2;e.canvas.enabled=n.flags&4;e.canvas.focused=int16_t(n.canvas_focused);
        e.rect=epok_hud_rect(n.rect);e.rect.enabled=n.flags&8;
        e.image.enabled=n.flags&16;e.image.texture=n.texture;
        e.image.tiling=n.tiling==1?epok::ImageTiling::Tile:n.tiling==2?epok::ImageTiling::TileFit:epok::ImageTiling::None;
        e.text.enabled=n.flags&32;e.text.wrap=n.flags&128;e.text.set_text(n.text);
        e.progress.enabled=n.flags&64;e.progress.value=epok::Fixed(n.progress[0],epok::Fixed::RAW);
        e.layout_element.enabled=n.flags&256;e.layout_element.horizontal=uint8_t(n.layout_element[0]);e.layout_element.vertical=uint8_t(n.layout_element[1]);
        e.layout_element.stretch=epok::Fixed(n.layout_element[4],epok::Fixed::RAW);
        e.layout_container.enabled=n.flags&512;e.layout_container.kind=epok::LayoutKind(uint8_t(n.layout_container[0]));e.layout_container.columns=uint8_t(n.layout_container[7]);
        e.focusable.enabled=n.flags&1024;e.focusable.order=uint8_t(n.focusable[5]);
        for(int k=0;k<2;++k){e.layout_element.minimum[k]=epok::Fixed(n.layout_element[2+k],epok::Fixed::RAW);e.layout_container.spacing[k]=epok::Fixed(n.layout_container[1+k],epok::Fixed::RAW);}
        for(int k=0;k<3;++k){e.image.color[k]=uint8_t(n.image_color[k]);e.text.color[k]=uint8_t(n.text_color[k]);e.progress.color[k]=uint8_t(n.progress[1+k]);e.progress.background[k]=uint8_t(n.progress[4+k]);e.focusable.highlight[k]=uint8_t(n.focusable[6+k]);}
        for(int k=0;k<4;++k){e.image.region[k]=uint16_t(n.region[k]);e.image.borders[k]=uint16_t(n.borders[k]);e.layout_container.padding[k]=epok::Fixed(n.layout_container[3+k],epok::Fixed::RAW);e.focusable.neighbors[k]=int16_t(n.focusable[1+k]);}
    }
    return entities;
}
extern "C" uint32_t epok_hud_compile(const EpokHudNode* nodes,uint32_t count,const int32_t* dimensions,uint32_t textures,int32_t width,int32_t height,const uint32_t* budget,EpokHudCommand* output,uint32_t capacity,uint32_t* stats){
    EpokHudSink sink;sink.dimensions.assign(dimensions,dimensions+textures*2);
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
extern "C" uint32_t epok_hud_layout(const EpokHudNode* nodes,uint32_t count,int32_t width,int32_t height,int32_t* out_rects){
    auto entities=epok_hud_entities(nodes,count);
    std::vector<int> first(count),next(count);
    std::vector<epok::Fixed> measured(size_t(count)*2);std::vector<epok::hud_core::Rect> rects(count);
    std::vector<epok::hud_core::Affine2> transforms(count);
    auto* sizes=reinterpret_cast<epok::Fixed(*)[2]>(measured.data());
    EpokHudSink sink;
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
