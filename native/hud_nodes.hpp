#pragma once
#include "hud_commands.hpp"

static std::vector<epok::ActorData> epok_hud_entities(const EpokHudNode* nodes,uint32_t count){
    std::vector<epok::ActorData> entities(count);
    for(uint32_t i=0;i<count;++i){
        const auto& n=nodes[i];auto& e=entities[i];e.parent=n.parent;
        e.alive=n.flags&1;e.active=n.flags&2;e.canvas.enabled=n.flags&4;e.canvas.focused=int16_t(n.canvas_focused);
        e.canvas.screen_overlay=n.flags&16384;
        e.rect=epok_hud_rect(n.rect);e.rect.enabled=n.flags&8;
        e.image.enabled=n.flags&16;e.image.texture=n.texture;
        e.image.tiling=n.tiling==1?epok::ImageTiling::Tile:n.tiling==2?epok::ImageTiling::TileFit:epok::ImageTiling::None;
        e.text.enabled=n.flags&32;e.text.wrap=n.flags&128;e.text.set_text(n.text);
        e.text.font=n.font;e.text.align=n.align==1?epok::TextAlign::Center:n.align==2?epok::TextAlign::Right:epok::TextAlign::Left;
        e.progress.enabled=n.flags&64;e.progress.value=epok::Fixed(n.progress[0],epok::Fixed::RAW);
        e.layout_element.enabled=n.flags&256;e.layout_element.horizontal=uint8_t(n.layout_element[0]);e.layout_element.vertical=uint8_t(n.layout_element[1]);
        e.layout_element.stretch=epok::Fixed(n.layout_element[4],epok::Fixed::RAW);
        e.layout_container.enabled=n.flags&512;e.layout_container.kind=epok::LayoutKind(uint8_t(n.layout_container[0]));e.layout_container.columns=uint8_t(n.layout_container[7]);
        e.focusable.enabled=n.flags&1024;e.focusable.order=uint8_t(n.focusable[5]);
        e.focusable.kind=n.flags&2048?epok::FocusKind::Toggle:epok::FocusKind::Button;e.focusable.checked=n.flags&4096;e.focusable.outline=n.flags&8192;
        e.progress.step=epok::Fixed(n.progress[7],epok::Fixed::RAW);
        for(int k=0;k<2;++k){e.layout_element.minimum[k]=epok::Fixed(n.layout_element[2+k],epok::Fixed::RAW);e.layout_container.spacing[k]=epok::Fixed(n.layout_container[1+k],epok::Fixed::RAW);}
        for(int k=0;k<3;++k){e.image.color[k]=uint8_t(n.image_color[k]);e.text.color[k]=uint8_t(n.text_color[k]);e.progress.color[k]=uint8_t(n.progress[1+k]);e.progress.background[k]=uint8_t(n.progress[4+k]);e.focusable.highlight[k]=uint8_t(n.focusable[6+k]);}
        for(int k=0;k<4;++k){e.image.region[k]=uint16_t(n.region[k]);e.image.borders[k]=uint16_t(n.borders[k]);e.layout_container.padding[k]=epok::Fixed(n.layout_container[3+k],epok::Fixed::RAW);e.focusable.neighbors[k]=int16_t(n.focusable[1+k]);}
    }
    return entities;
}
