#pragma once
#include <stdint.h>
namespace epok {
// Post-HUD fade to black. 0 is clear, 255 is opaque black. Deliberately
// survives scene-bank switches so games can hide the activation frame.
inline uint8_t screen_fade=0;
struct FogEnvironment {bool enabled=false;int32_t start=12*4096,end=40*4096;uint8_t color[3]={64,77,102};};
inline FogEnvironment fog_environment;
template<class Color>inline Color fog_color(Color color,int32_t depth){
    if(!fog_environment.enabled||depth<=fog_environment.start)return color;
    int32_t amount=depth>=fog_environment.end?4096:int32_t(int64_t(depth-fog_environment.start)*4096/(fog_environment.end-fog_environment.start));
    color.r=uint8_t((int32_t(color.r)*(4096-amount)+int32_t(fog_environment.color[0])*amount+2048)/4096);color.g=uint8_t((int32_t(color.g)*(4096-amount)+int32_t(fog_environment.color[1])*amount+2048)/4096);color.b=uint8_t((int32_t(color.b)*(4096-amount)+int32_t(fog_environment.color[2])*amount+2048)/4096);return color;
}
struct UvVertex {int32_t camera[3],color[3],uv[2];};
// Split at texture wrap boundaries before converting UV to 8-bit page pixels.
// At most four cells and bounded eight-vertex clip buffers, no heap allocation.
template<class Emit>void scroll_triangle(const UvVertex* input,const int32_t* speed,uint32_t ticks,Emit emit){
    if(!speed[0]&&!speed[1]){emit(input[0],input[1],input[2]);return;}
    int32_t offset[2];UvVertex base[3];for(int c=0;c<2;++c){offset[c]=int32_t(int64_t(speed[c])*(ticks%245760)/60%4096);if(offset[c]<0)offset[c]+=4096;}
    for(int i=0;i<3;++i){base[i]=input[i];for(int c=0;c<2;++c){auto u=base[i].uv[c];base[i].uv[c]=(u<0?0:u>4096?4096:u)+offset[c];}}
    for(int x=0;x<=int(offset[0]>0);++x)for(int y=0;y<=int(offset[1]>0);++y){UvVertex buffers[2][8];for(int i=0;i<3;++i)buffers[0][i]=base[i];int from=0,count=3;int32_t origin[2]={x*4096,y*4096};
        for(int plane=0;plane<4&&count;++plane){const int axis=plane/2;auto distance=[&](const UvVertex& v){return plane%2?origin[axis]+4096-v.uv[axis]:v.uv[axis]-origin[axis];};int next=0;auto previous=buffers[from][count-1];int32_t pd=distance(previous);
            for(int i=0;i<count;++i){auto current=buffers[from][i];int32_t cd=distance(current);if((pd>=0)!=(cd>=0)){int32_t t=int32_t(int64_t(pd)*65536/(pd-cd));UvVertex v;for(int c=0;c<3;++c){v.camera[c]=previous.camera[c]+int32_t((int64_t(current.camera[c])-previous.camera[c])*t/65536);v.color[c]=previous.color[c]+int32_t(int64_t(current.color[c]-previous.color[c])*t/65536);}for(int c=0;c<2;++c)v.uv[c]=previous.uv[c]+int32_t(int64_t(current.uv[c]-previous.uv[c])*t/65536);v.uv[axis]=origin[axis]+(plane%2?4096:0);if(next<8)buffers[1-from][next++]=v;}if(cd>=0&&next<8)buffers[1-from][next++]=current;previous=current;pd=cd;}count=next;from=1-from;
        }
        for(int i=0;i<count;++i)for(int c=0;c<2;++c)buffers[from][i].uv[c]-=origin[c];
        for(int i=1;i+1<count;++i)emit(buffers[from][0],buffers[from][i],buffers[from][i+1]);
    }
}
}
