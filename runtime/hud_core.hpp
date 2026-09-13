#pragma once
#include "epok.hpp"

namespace epok::hud_core {
// Shared by the PSX packet writer and the native preview. No GPU, heap, or
// floating-point operations: layout, clipping, UVs, text and budgets agree.
struct Rect { Fixed x=0.0,y=0.0,w=0.0,h=0.0; };
struct Budget { unsigned layouts,rectangles,texts,glyphs; };
inline int pixel(Fixed v) { auto r=v.raw();return int((int64_t(r)+(r>=0?2048:-2048))/4096); }
inline int clamp(int v,int hi) { return v<0?0:v>hi?hi:v; }
inline Rect resolve(Rect parent,const RectTransform& r) {
    Fixed output[4],pos[2]={parent.x,parent.y},size[2]={parent.w,parent.h};
    for(int i=0;i<2;++i){
        Fixed extent=size[i]*(r.anchor_max[i]-r.anchor_min[i])+r.size[i];
        if(extent.raw()<0)extent=0.0;
        output[i+2]=extent;
        output[i]=pos[i]+size[i]*(r.anchor_min[i]+(r.anchor_max[i]-r.anchor_min[i])*r.pivot[i])+r.position[i]-extent*r.pivot[i];
    }
    return {output[0],output[1],output[2],output[3]};
}
template<class Sink> class Compiler {
    Sink& sink; int width,height; Budget budget; unsigned layouts=0; int owner=-1;
    bool primitive_available(){if(stats.rectangles+stats.images<budget.rectangles)return true;++stats.dropped;return false;}
    void fill(Rect r,const uint8_t* color){
        int x0=clamp(pixel(r.x),width),x1=clamp(pixel(r.x+r.w),width);
        int y0=clamp(height-pixel(r.y+r.h),height),y1=clamp(height-pixel(r.y),height);
        if(x1<=x0||y1<=y0||!primitive_available())return;
        ++stats.rectangles;sink.rectangle(owner,x0,y0,x1,y1,color);
    }
    void picture(Rect r,const Image& image){
        int texture_width=0,texture_height=0;
        if(!sink.texture_size(image.texture,texture_width,texture_height)){fill(r,image.color);return;}
        int x=pixel(r.x),y=height-pixel(r.y+r.h),w=pixel(r.w),h=pixel(r.h);
        if(w<=0||h<=0)return;
        int x0=clamp(x,width),x1=clamp(x+w,width),y0=clamp(y,height),y1=clamp(y+h,height);
        if(x1<=x0||y1<=y0||!primitive_available())return;
        int u=image.region[0],v=image.region[1];
        int tw=image.region[2]?image.region[2]:texture_width-u,th=image.region[3]?image.region[3]:texture_height-v;
        if(tw<=0||th<=0||u+tw>texture_width||v+th>texture_height)return;
        if(image.borders[0]||image.borders[1]||image.borders[2]||image.borders[3]){
            int left=image.borders[0],top=image.borders[1],right=image.borders[2],bottom=image.borders[3];
            if(left+right>tw||top+bottom>th)return;
            int dl=left,dr=right,dt=top,db=bottom;
            if(dl+dr>w){dl=left*w/(left+right);dr=w-dl;}
            if(dt+db>h){dt=top*h/(top+bottom);db=h-dt;}
            const int sx[4]={0,left,tw-right,tw},sy[4]={0,top,th-bottom,th};
            const int dx[4]={0,dl,w-dr,w},dy[4]={0,dt,h-db,h};
            for(int row=0;row<3;++row)for(int col=0;col<3;++col){
                if(dx[col+1]<=dx[col]||dy[row+1]<=dy[row]||sx[col+1]<=sx[col]||sy[row+1]<=sy[row])continue;
                Image part=image;for(auto& border:part.borders)border=0;
                part.region[0]=uint16_t(u+sx[col]);part.region[1]=uint16_t(v+sy[row]);part.region[2]=uint16_t(sx[col+1]-sx[col]);part.region[3]=uint16_t(sy[row+1]-sy[row]);
                picture({Fixed((x+dx[col])*4096,Fixed::RAW),Fixed((height-y-dy[row+1])*4096,Fixed::RAW),Fixed((dx[col+1]-dx[col])*4096,Fixed::RAW),Fixed((dy[row+1]-dy[row])*4096,Fixed::RAW)},part);
            }
            return;
        }
        ++stats.images;
        sink.image(owner,image.texture,x0,y0,x1,y1,u+(x0-x)*tw/w,v+(y0-y)*th/h,u+(x1-x)*tw/w-1,v+(y1-y)*th/h-1,image.color);
    }
    void label(Rect r,const Text& text){
        if(stats.texts>=budget.texts){++stats.dropped;return;}
        int columns=pixel(r.w)/8,rows=pixel(r.h)/16;if(columns<=0||rows<=0)return;
        ++stats.texts;sink.begin_text();
        int col=0,row=0,left=pixel(r.x),top=height-pixel(r.y+r.h);
        for(size_t n=0;n<511&&text.value[n];++n){
            unsigned c=uint8_t(text.value[n]);
            if(c==10){col=0;++row;continue;}
            if(col>=columns&&text.wrap){col=0;++row;}
            if(row>=rows)break;
            int x=left+col*8,y=top+row*16;++col;
            if(col>columns||x>=width||y>=height||x+8<=0||y+16<=0)continue;
            if(stats.glyphs>=budget.glyphs){++stats.dropped;break;}
            if(c<32||c>142)c='?';
            int x0=clamp(x,width),y0=clamp(y,height),x1=clamp(x+8,width),y1=clamp(y+16,height);
            ++stats.glyphs;sink.glyph(owner,c,x0,y0,x1,y1,x0-x,y0-y,text.color);
        }
    }
    void visit(Entity* entities,size_t count,int index,Rect parent,const int* first,const int* next,int depth){
        if(depth>32)return;
        auto& e=entities[index];if(!e.alive||!e.active)return;
        Rect r=parent;
        if(e.rect.enabled){if(layouts++>=budget.layouts){++stats.dropped;return;}r=resolve(parent,e.rect);}
        owner=index;
        if(e.image.enabled)picture(r,e.image);
        if(e.progress.enabled){fill(r,e.progress.background);auto part=r;auto value=e.progress.value;if(value.raw()<0)value=0.0;if(value.raw()>4096)value=1.0;part.w*=value;fill(part,e.progress.color);}
        if(e.text.enabled)label(r,e.text);
        for(int child=first[index];child>=0;child=next[child])if(entities[child].rect.enabled)visit(entities,count,child,r,first,next,depth+1);
    }
public:
    HudStats stats;
    Compiler(Sink& sink,int width,int height,Budget budget):sink(sink),width(width),height(height),budget(budget){}
    void draw(Entity* entities,size_t count,int* first,int* next){
        stats={};layouts=0;
        for(size_t i=0;i<count;++i)first[i]=next[i]=-1;
        for(size_t i=count;i-->0;){int p=entities[i].parent;if(p>=0&&size_t(p)<count){next[i]=first[p];first[p]=int(i);}}
        for(size_t i=0;i<count;++i)if(entities[i].canvas.enabled&&entities[i].parent<0)visit(entities,count,int(i),{0.0,0.0,Fixed(width*4096,Fixed::RAW),Fixed(height*4096,Fixed::RAW)},first,next,0);
    }
};
}
