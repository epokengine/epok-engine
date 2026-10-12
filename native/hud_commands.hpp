#pragma once
#include "hud_core.hpp"
#include "hud_preview.h"
#include <vector>

struct EpokHudSink {
    std::vector<EpokHudCommand> commands;
    std::vector<int32_t> dimensions;
    // Cooked fonts in index order. The metrics they point at are owned by
    // `glyphs`, so the vector must not be resized after `fonts` is built.
    std::vector<epok::Font> fonts;
    std::vector<epok::GlyphMetric> glyphs;
    bool texture_size(int id,int& w,int& h){
        if(id<0||size_t(id)*2+1>=dimensions.size())return false;
        w=dimensions[size_t(id)*2];h=dimensions[size_t(id)*2+1];return w>0&&h>0;
    }
    const epok::Font* font(int index){return index>=0&&size_t(index)<fonts.size()?&fonts[size_t(index)]:nullptr;}
    void rectangle(int owner,int x0,int y0,int x1,int y1,const uint8_t* c){commands.push_back({{0,owner,-1,x0,y0,x1,y1,0,0,0,0,c[0],c[1],c[2],0,0}});}
    void image(int owner,int id,int x0,int y0,int x1,int y1,int u0,int v0,int u1,int v1,const uint8_t* c){commands.push_back({{1,owner,id,x0,y0,x1,y1,u0,v0,u1,v1,c[0],c[1],c[2],0,0}});}
    void begin_text(int){}
    void glyph(int owner,int index,int u,int v,int x0,int y0,int x1,int y1,const uint8_t* color){commands.push_back({{2,owner,index,x0,y0,x1,y1,u,v,0,0,color[0],color[1],color[2],0,0}});}
    // The rotated kinds carry four screen corners and, when textured, four
    // source corners; the colour stays where the axis-aligned kinds keep it.
    void quad(int owner,const int* x,const int* y,const uint8_t* c){commands.push_back(rotated(3,owner,-1,x,y,nullptr,nullptr,c));}
    void textured_quad(int owner,int id,const int* x,const int* y,const int* u,const int* v,const uint8_t* c){commands.push_back(rotated(4,owner,id,x,y,u,v,c));}
    void glyph_quad(int owner,int index,const int* x,const int* y,const int* u,const int* v,const uint8_t* c){commands.push_back(rotated(5,owner,index,x,y,u,v,c));}
    // Rebuild the runtime descriptors from the flat wire table. VRAM placement
    // is absent there: nothing on this side draws from real VRAM.
    void load_fonts(const int32_t* table,uint32_t words){
        fonts.clear();glyphs.clear();
        if(!table)return;
        uint32_t at=0,total=0;
        for(uint32_t scan=0;scan+5<=words;){const uint32_t count=uint32_t(table[scan]);if(scan+5+count*8>words)break;total+=count;scan+=5+count*8;}
        glyphs.reserve(total);
        while(at+5<=words){
            const uint32_t count=uint32_t(table[at]);
            if(at+5+count*8>words)break;
            epok::Font font{};
            font.count=uint16_t(count);font.width=uint16_t(table[at+1]);font.height=uint16_t(table[at+2]);
            font.line_height=uint16_t(table[at+3]);font.baseline=uint16_t(table[at+4]);
            const size_t base=glyphs.size();
            for(uint32_t i=0;i<count;++i){
                const int32_t* m=table+at+5+i*8;
                glyphs.push_back({uint32_t(m[0]),uint8_t(m[1]),uint8_t(m[2]),uint8_t(m[3]),uint8_t(m[4]),uint8_t(m[5]),int8_t(m[6]),int8_t(m[7])});
            }
            font.metrics=glyphs.data()+base;
            fonts.push_back(font);
            at+=5+count*8;
        }
    }
private:
    static EpokHudCommand rotated(int kind,int owner,int subject,const int* x,const int* y,const int* u,const int* v,const uint8_t* c){
        EpokHudCommand command{};
        command.v[0]=kind;command.v[1]=owner;command.v[2]=subject;
        for(int i=0;i<4;++i){command.v[3+i*2]=x[i];command.v[4+i*2]=y[i];}
        command.v[11]=c[0];command.v[12]=c[1];command.v[13]=c[2];
        if(u&&v)for(int i=0;i<4;++i){command.v[14+i*2]=u[i];command.v[15+i*2]=v[i];}
        return command;
    }
};
inline epok::RectTransform epok_hud_rect(const int32_t* r){
    epok::RectTransform result;result.enabled=true;
    for(int i=0;i<2;++i){
        result.anchor_min[i]=epok::Fixed(r[i],epok::Fixed::RAW);result.anchor_max[i]=epok::Fixed(r[2+i],epok::Fixed::RAW);
        result.pivot[i]=epok::Fixed(r[4+i],epok::Fixed::RAW);result.position[i]=epok::Fixed(r[6+i],epok::Fixed::RAW);result.size[i]=epok::Fixed(r[8+i],epok::Fixed::RAW);
    }
    result.rotation=epok::Fixed(r[10],epok::Fixed::RAW);
    return result;
}
