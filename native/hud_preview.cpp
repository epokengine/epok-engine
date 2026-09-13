#include "hud_commands.hpp"
#include <algorithm>

extern "C" uint32_t epok_hud_compile(const EpokHudNode* nodes,uint32_t count,const int32_t* dimensions,uint32_t textures,int32_t width,int32_t height,const uint32_t* budget,EpokHudCommand* output,uint32_t capacity,uint32_t* stats){
    EpokHudSink sink;sink.dimensions.assign(dimensions,dimensions+textures*2);
    std::vector<epok::Entity> entities(count);
    for(uint32_t i=0;i<count;++i){
        const auto& n=nodes[i];auto& e=entities[i];e.parent=n.parent;
        e.alive=n.flags&1;e.active=n.flags&2;e.canvas.enabled=n.flags&4;
        e.rect=epok_hud_rect(n.rect);e.rect.enabled=n.flags&8;
        e.image.enabled=n.flags&16;e.image.texture=n.texture;
        e.text.enabled=n.flags&32;e.text.wrap=n.flags&128;e.text.set_text(n.text);
        e.progress.enabled=n.flags&64;e.progress.value=epok::Fixed(n.progress[0],epok::Fixed::RAW);
        for(int k=0;k<3;++k){e.image.color[k]=uint8_t(n.image_color[k]);e.text.color[k]=uint8_t(n.text_color[k]);e.progress.color[k]=uint8_t(n.progress[1+k]);e.progress.background[k]=uint8_t(n.progress[4+k]);}
        for(int k=0;k<4;++k){e.image.region[k]=uint16_t(n.region[k]);e.image.borders[k]=uint16_t(n.borders[k]);}
    }
    std::vector<int> first(count),next(count);
    epok::hud_core::Compiler compiler(sink,width,height,{budget[0],budget[1],budget[2],budget[3]});
    compiler.draw(entities.data(),count,first.data(),next.data());
    auto s=compiler.stats;stats[0]=s.rectangles;stats[1]=s.glyphs;stats[2]=s.texts;stats[3]=s.images;stats[4]=s.dropped;
    uint32_t written=std::min(capacity,uint32_t(sink.commands.size()));
    std::copy_n(sink.commands.data(),written,output);return written;
}
extern "C" void epok_hud_resolve(const int32_t* parent,const int32_t* rect,int32_t* output){
    using epok::Fixed;
    auto r=epok::hud_core::resolve({Fixed(parent[0],Fixed::RAW),Fixed(parent[1],Fixed::RAW),Fixed(parent[2],Fixed::RAW),Fixed(parent[3],Fixed::RAW)},epok_hud_rect(rect));
    output[0]=r.x.raw();output[1]=r.y.raw();output[2]=r.w.raw();output[3]=r.h.raw();
}
