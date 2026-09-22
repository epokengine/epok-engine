#pragma once
#include "epok.hpp"
#include "affine.hpp"
#include "lighting.hpp"
#include "texture.hpp"
#include "sprite_math.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/primitives/triangles.hh"
namespace epok {
namespace sprite_detail {
inline Affine<Fixed> plane(const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view){
    if(sprite.orientation==SpriteOrientation::Fixed)return world;
    auto result=Affine<Fixed>::identity();
    lighting_detail::Vector right={{view.values[0][0].raw(),view.values[0][1].raw(),view.values[0][2].raw()}},up={{view.values[1][0].raw(),view.values[1][1].raw(),view.values[1][2].raw()}};
    if(sprite.orientation==SpriteOrientation::Upright){right.v[1]=0;up={{0,4096,0}};}
    right=lighting_detail::normalize(right);up=lighting_detail::normalize(up);if(!right.v[0]&&!right.v[1]&&!right.v[2])right={{4096,0,0}};
    for(int axis=0;axis<2;++axis){uint64_t squared=0;for(int r=0;r<3;++r){int64_t v=world.values[r][axis].raw();squared+=v*v;}auto scale=lighting_detail::sqrt64(squared);for(int r=0;r<3;++r)result.values[r][axis]=Fixed(int32_t(int64_t(axis==0?right.v[r]:up.v[r])*scale/4096),Fixed::RAW);}
    for(int r=0;r<3;++r){result.values[r][2]=Fixed(int32_t((int64_t(right.v[(r+1)%3])*up.v[(r+2)%3]-int64_t(right.v[(r+2)%3])*up.v[(r+1)%3])/4096),Fixed::RAW);result.values[r][3]=world.values[r][3];}
    return result;
}
// A bounded per-renderer cache of camera-facing bases. The full uncached plane
// above remains the reference for Q12 scale, shear and negative-value rounding.
class PlaneCache {
    Affine<Fixed> previous_view,unit_model[2],unit_view[2];
    bool has_view=false,valid[2]={};
public:
    void clear(){has_view=false;valid[0]=valid[1]=false;}
    void transform(const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view,
                   Affine<Fixed>& model,Affine<Fixed>& camera){
        if(sprite.orientation==SpriteOrientation::Fixed){model=world;camera=view.compose(world);return;}
        bool changed=!has_view;
        if(!changed)for(int r=0;r<3;++r)for(int c=0;c<4;++c)
            changed|=previous_view.values[r][c].raw()!=view.values[r][c].raw();
        if(changed){previous_view=view;has_view=true;valid[0]=valid[1]=false;}
        const unsigned index=sprite.orientation==SpriteOrientation::Upright?1:0;
        if(!valid[index]){
            unit_model[index]=plane(sprite,Affine<Fixed>::identity(),view);
            unit_view[index]=view.compose(unit_model[index]);valid[index]=true;
        }
        model=unit_model[index];bool unit=true;
        for(int axis=0;axis<2;++axis){
            uint64_t squared=0;for(int r=0;r<3;++r){const int64_t value=world.values[r][axis].raw();squared+=value*value;}
            if(squared==uint64_t(4096)*4096)continue;
            unit=false;const auto scale=lighting_detail::sqrt64(squared);
            for(int r=0;r<3;++r)model.values[r][axis]=Fixed(int32_t(int64_t(unit_model[index].values[r][axis].raw())*scale/4096),Fixed::RAW);
        }
        Fixed position[3];for(int r=0;r<3;++r){position[r]=world.values[r][3];model.values[r][3]=position[r];}
        if(unit){
            camera=unit_view[index];Fixed translated[3];view.point(position,translated);
            for(int r=0;r<3;++r)camera.values[r][3]=translated[r];
        }else camera=view.compose(model);
    }
};
}
// Fixed double-buffered primitive storage. Sprites and particles share this budget.
template<size_t Capacity=2048>class SpriteRenderer {
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TexturedTriangle> primitives[2][Capacity];
    size_t used=0;
    bool dithering=false;
    sprite_detail::PlaneCache planes;
public:
    void begin(bool dither=false){used=0;dithering=dither;sprite_stats={};planes.clear();}
    // LightingRenderer::shade(owner,objects,world,true) must prepare GTE before a lit draw.
    template<class Table>void draw(int parity,Table& table,const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view,bool receive_lighting=true){
        if(!sprite.enabled)return;const auto* tex=texture(sprite.texture);if(!tex){++sprite_stats.culled;return;}
        Affine<Fixed> model,mv;planes.transform(sprite,world,view,model,mv);
        for(int r=0;r<2;++r)for(int c=0;c<4;++c)mv.values[r][c]*=projection_focal;
        if(sprite.region[0]>=tex->width||sprite.region[1]>=tex->height){++sprite_stats.culled;return;}
        auto w=sprite.region[2]?sprite.region[2]:tex->width-sprite.region[0],h=sprite.region[3]?sprite.region[3]:tex->height-sprite.region[1];
        if(!w||!h||uint32_t(sprite.region[0])+w>tex->width||uint32_t(sprite.region[1])+h>tex->height){++sprite_stats.culled;return;}
        struct Clip {int32_t p[3];int32_t uv[2];};
        Clip buffers[2][12];int from=0,count=4;
        static constexpr int xy[4][2]={{0,1},{1,1},{1,0},{0,0}};
        for(int i=0;i<4;++i){Fixed local[3]={(Fixed(xy[i][0]*4096,Fixed::RAW)-sprite.pivot[0])*sprite.size[0],(Fixed(xy[i][1]*4096,Fixed::RAW)-sprite.pivot[1])*sprite.size[1],0.0},p[3];mv.point(local,p);for(int c=0;c<3;++c)buffers[0][i].p[c]=p[c].raw();
            const int u=sprite.flip_x?1-xy[i][0]:xy[i][0],v=sprite.flip_y?xy[i][1]:1-xy[i][1];buffers[0][i].uv[0]=(sprite.region[0]+u*(w-1))*4096;buffers[0][i].uv[1]=(tex->y%256+sprite.region[1]+v*(h-1))*4096;}
        bool clipped=false,inside=true;
        for(int i=0;i<4;++i){const auto* p=buffers[0][i].p;inside&=sprite_detail::interior(p[0],p[1],p[2]);}
        // An interior quad is already the exact output of all six clip passes.
        // Boundary quads retain the complete original clipper and rounding.
        for(int plane=0;!inside&&plane<6&&count>0;++plane){auto distance=[&](const Clip& v)->int64_t{switch(plane){case 0:return int64_t(v.p[2])-1024;case 1:return 128*4096-1-int64_t(v.p[2]);case 2:return int64_t(v.p[2])+v.p[0];case 3:return int64_t(v.p[2])-v.p[0];case 4:return 3*int64_t(v.p[2])+4*int64_t(v.p[1]);default:return 3*int64_t(v.p[2])-4*int64_t(v.p[1]);}};
            int next=0;auto previous=buffers[from][count-1];auto pd=distance(previous);
            for(int i=0;i<count;++i){const auto current=buffers[from][i];auto cd=distance(current);if((pd>=0)!=(cd>=0)){clipped=true;Clip v;int64_t fraction=pd*65536/(pd-cd);for(int c=0;c<3;++c)v.p[c]=previous.p[c]+(int64_t(current.p[c])-previous.p[c])*fraction/65536;for(int c=0;c<2;++c)v.uv[c]=previous.uv[c]+(int64_t(current.uv[c])-previous.uv[c])*fraction/65536;if(plane==0)v.p[2]=1024;if(plane==1)v.p[2]=128*4096-1;if(next<12)buffers[1-from][next++]=v;}if(cd>=0&&next<12)buffers[1-from][next++]=current;else if(cd<0)clipped=true;previous=current;pd=cd;}
            count=next;from=1-from;
        }
        if(count<3){++sprite_stats.culled;return;}if(clipped)++sprite_stats.clipped;
        MeshQuad face{};face.normal[2]=-4096;face.material.unlit=sprite.unlit;Material tint{};tint.unlit=sprite.unlit;for(int c=0;c<3;++c){face.material.color[c]=255;tint.color[c]=sprite.color[c];}
        // At most 12 vertices, each clipped to [1024, 128*4096): sum fits i32.
        int32_t depth_sum=0;for(int i=0;i<count;++i)depth_sum+=buffers[from][i].p[2];
        auto color=fog_color(lighting_detail::mesh_shade(model,face,tint,receive_lighting),int32_t(depth_sum/count));color.r=uint8_t((uint16_t(color.r)+1)/2);color.g=uint8_t((uint16_t(color.g)+1)/2);color.b=uint8_t((uint16_t(color.b)+1)/2);
        psyqo::Vertex points[12];for(int i=0;i<count;++i){const auto& p=buffers[from][i];points[i]={{.x=int16_t(display_width/2+sprite_detail::project_ratio(p.p[0],display_width/2,p.p[2])),.y=int16_t(display_height/2-sprite_detail::project_ratio(p.p[1],display_height*2/3,p.p[2]))}};}
        ++sprite_stats.submitted;
        // One depth bucket for the complete quad avoids diagonal ordering seams.
        int depth=int(depth_sum/(count*1024))+sprite.depth_bias;if(depth<0)depth=0;if(depth>511)depth=511;
        for(int i=1;i+1<count;++i){int64_t area=int64_t(points[i].x-points[0].x)*(points[i+1].y-points[0].y)-int64_t(points[i].y-points[0].y)*(points[i+1].x-points[0].x);if(!area)continue;if(used>=Capacity){++sprite_stats.dropped;continue;}auto& f=primitives[parity][used++];auto& p=f.primitive;p.setColor(color);if(sprite.blend==BlendMode::Cutout)p.setOpaque();else p.setSemiTrans();p.pointA=points[0];p.pointB=points[i];p.pointC=points[i+1];p.uvA={.u=uint8_t(buffers[from][0].uv[0]/4096),.v=uint8_t(buffers[from][0].uv[1]/4096)};p.uvB={.u=uint8_t(buffers[from][i].uv[0]/4096),.v=uint8_t(buffers[from][i].uv[1]/4096)};p.uvC={.u=uint8_t(buffers[from][i+1].uv[0]/4096),.v=uint8_t(buffers[from][i+1].uv[1]/4096)};p.clutIndex=texture_clut(*tex);p.tpage=texture_page(*tex,sprite.blend,dithering);table.insert(f,depth);++sprite_stats.triangles;sprite_stats.estimated_pixels+=uint32_t((area<0?-area:area)/2);}
    }
};
}
