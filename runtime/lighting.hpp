#pragma once
#include "epok.hpp"
#include "affine.hpp"
#include "fixed_math.hpp"
#include "psyqo/gte-registers.hh"
#include "psyqo/gte-kernels.hh"
#include "psyqo/primitives/common.hh"
#include <array>

namespace epok {
extern LightingStats lighting_work;
// No heap allocation, floats, or per-vertex point-light search on the target.
namespace lighting_detail {
inline int32_t clamp(int32_t v,int32_t low,int32_t high){return v<low?low:(v>high?high:v);}
using fixed_math::sqrt64; // One implementation, shared with the easing kernel.
struct Vector { int32_t v[3]={}; };
inline Vector normalize(Vector in){int32_t biggest=0;for(auto c:in.v){int32_t a=c<0?-c:c;if(a>biggest)biggest=a;}while(biggest>16384){biggest>>=1;for(auto& c:in.v)c>>=1;}uint32_t length=0;for(auto c:in.v)length+=c*c;auto n=sqrt64(length);if(!n)return {};for(auto& c:in.v)c=c*4096/int32_t(n);return in;}
inline Vector direction(const Affine<Fixed>& m){return normalize({{-m.values[0][2].raw(),-m.values[1][2].raw(),-m.values[2][2].raw()}});}
inline Vector face_normal(const Affine<Fixed>& m,int face){
    // Cofactor columns implement inverse-transpose even under inherited shear.
    const int axis=face<2?2:(face<4?0:1);const int sign=(face==0 || face==2 || face==5)?-1:1;
    const int a=(axis+1)%3,b=(axis+2)%3;int64_t cross[3];uint64_t biggest=0;
    for(int r=0;r<3;++r){cross[r]=int64_t(m.values[(r+1)%3][a].raw())*m.values[(r+2)%3][b].raw()-int64_t(m.values[(r+2)%3][a].raw())*m.values[(r+1)%3][b].raw();auto absolute=uint64_t(cross[r]<0?-cross[r]:cross[r]);if(absolute>biggest)biggest=absolute;}
    while(biggest>16384){biggest>>=1;for(auto& c:cross)c>>=1;}
    Vector n;for(int r=0;r<3;++r)n.v[r]=sign*int32_t(cross[r]);return normalize(n);
}
struct MeshNormalTransform {
  int64_t cofactors[3][3]={};
  bool initialized=false;
  Vector apply(const Affine<Fixed>& m,const int16_t* normal) {
  if(!initialized){
    for(int axis=0;axis<3;++axis){
      const int a=(axis+1)%3,b=(axis+2)%3;
      for(int r=0;r<3;++r)
        cofactors[r][axis]=(int64_t(m.values[(r+1)%3][a].raw())*m.values[(r+2)%3][b].raw()-int64_t(m.values[(r+2)%3][a].raw())*m.values[(r+1)%3][b].raw())/4096;
    }
    initialized=true;
  }
  int64_t out[3] = {};
  uint64_t biggest = 0;
  for (int axis = 0; axis < 3; ++axis) {
    for (int r = 0; r < 3; ++r) {
      out[r] += cofactors[r][axis] * normal[axis] / 4096;
    }
  }
  for (auto v : out) {
    auto n = uint64_t(v < 0 ? -v : v);
    if (n > biggest)
      biggest = n;
  }
  while (biggest > 16384) {
    biggest >>= 1;
    for (auto &v : out)
      v >>= 1;
  }
  Vector n;
  for (int r = 0; r < 3; ++r)
    n.v[r] = int32_t(out[r]);
  return normalize(n);
  }
};
inline Vector mesh_normal(const Affine<Fixed>& m,const int16_t* normal) {
  MeshNormalTransform transform;return transform.apply(m,normal);
}
// Shades one face with the light registers prepared by LightingRenderer::shade.
// The normal is separate from the face so callers can substitute a runtime one.
inline psyqo::Color mesh_shade_normal(const Affine<Fixed> &world, const int16_t* normal, const Material& face_material,
                               const Material &tint, bool enabled,MeshNormalTransform* transform=nullptr) {
  using namespace psyqo::GTE;
  uint8_t color[3];
  for (int c = 0; c < 3; ++c)
    color[c] = uint16_t(face_material.color[c]) * tint.color[c] / 255;
  if (face_material.unlit || tint.unlit || !enabled)
    return psyqo::Color{{.r = color[0], .g = color[1], .b = color[2]}};
  auto n = transform?transform->apply(world,normal):mesh_normal(world, normal);
  write<Register::VXY0>(uint16_t(n.v[0]) | (uint32_t(uint16_t(n.v[1])) << 16));
  write<Register::VZ0>(uint32_t(n.v[2]));
  Kernels::ncs();
  auto rgb = readRaw<Register::RGB2>();
  ++lighting_work.gte_normals;
  return psyqo::Color{{.r = uint8_t((rgb & 255) * color[0] / 255),
                       .g = uint8_t(((rgb >> 8) & 255) * color[1] / 255),
                       .b = uint8_t(((rgb >> 16) & 255) * color[2] / 255)}};
}
inline psyqo::Color mesh_shade(const Affine<Fixed> &world, const MeshQuad &face,
                               const Material &tint, bool enabled,MeshNormalTransform* transform=nullptr) {
  return mesh_shade_normal(world, face.normal, face.material, tint, enabled, transform);
}
// Lit face whose normal is already in the space of the GTE light matrix (see
// LightingRenderer::localize). Callers handle the unlit cases.
inline psyqo::Color mesh_shade_local(const int16_t* normal, const Material& face_material, const Material& tint) {
  using namespace psyqo::GTE;
  write<Register::VXY0>(uint16_t(normal[0]) | (uint32_t(uint16_t(normal[1])) << 16));
  write<Register::VZ0>(uint32_t(int32_t(normal[2])));
  Kernels::ncs();
  const uint32_t rgb = readRaw<Register::RGB2>();
  ++lighting_work.gte_normals;
  const uint32_t r = uint32_t(face_material.color[0]) * tint.color[0] / 255, g = uint32_t(face_material.color[1]) * tint.color[1] / 255, b = uint32_t(face_material.color[2]) * tint.color[2] / 255;
  return psyqo::Color{{.r = uint8_t((rgb & 255) * r / 255), .g = uint8_t(((rgb >> 8) & 255) * g / 255), .b = uint8_t(((rgb >> 16) & 255) * b / 255)}};
}
inline uint32_t pair(int32_t a,int32_t b){return uint16_t(a)|(uint32_t(uint16_t(b))<<16);}
struct Source {size_t entity;Vector direction;int32_t position[3];};
struct Local {Vector direction;int32_t color[3]={};int selected=-1;bool initialized=false;};
}

template<size_t N> class LightingRenderer {
    using Vector=lighting_detail::Vector;
    std::array<lighting_detail::Source,32> sources{};
    std::array<lighting_detail::Local,N> local_cache{};
    std::array<std::array<Vector,6>,N> normals{};
    std::array<std::array<int32_t,9>,N> bases{};
    std::array<bool,N> have_normals{};
    int32_t light_rows[3][3]={};   // world-space light matrix of the last shade()
    size_t count=0;
public:
    // Hash of the light matrix, colours, ambient and material tint loaded by the
    // last shade(); retained packets rebuild when it changes.
    uint32_t signature=0;
private:
public:
    void reset_owner(size_t index) {
        if(index>=N)return;
        local_cache[index]={};have_normals[index]=false;
    }
    void clear() {
        count=0;
        for(size_t i=0;i<N;++i)reset_owner(i);
    }
    void prepare(const std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t object_count){
        count=0;lighting_work={};
        for(size_t i=0;i<object_count && count<sources.size();++i){const auto& l=objects[i].light;if(!l.enabled || l.mode==LightMode::Baked || !is_active_slot(i))continue;
            auto& s=sources[count++];s.entity=i;s.direction=lighting_detail::direction(world[i]);for(int c=0;c<3;++c)s.position[c]=world[i].values[c][3].raw();
        }lighting_work.active_lights=count;
    }
    std::array<psyqo::Color,6> shade(size_t object,const std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,bool generic=false){
        using namespace lighting_detail;using namespace psyqo::GTE;
        const auto& material=objects[object].material;std::array<psyqo::Color,6> output;
        if(material.unlit || !objects[object].lighting.enabled){for(auto& c:output)c=psyqo::Color{{.r=material.color[0],.g=material.color[1],.b=material.color[2]}};return output;}
        ++lighting_work.lit_objects;
        bool changed=!have_normals[object];
        for(int r=0;r<3;++r)for(int c=0;c<3;++c)if(bases[object][r*3+c]!=world[object].values[r][c].raw())changed=true;
        if(changed && !generic){for(int f=0;f<6;++f)normals[object][f]=face_normal(world[object],f);for(int r=0;r<3;++r)for(int c=0;c<3;++c)bases[object][r*3+c]=world[object].values[r][c].raw();have_normals[object]=true;}

        int best[2]={-1,-1},score[2]={INT32_MIN,INT32_MIN};int32_t strength[32]={};Vector directions[32];
        for(size_t i=0;i<count;++i){const auto& s=sources[i];const auto& l=objects[s.entity].light;int slot=l.type==LightType::Point;int power=clamp(l.intensity.raw(),0,8192);directions[i]=s.direction;
            if(slot){if(!lighting_environment.point_lights)continue;Vector delta;uint64_t square=0;for(int c=0;c<3;++c){delta.v[c]=clamp(s.position[c]-world[object].values[c][3].raw(),-256*4096,256*4096);square+=int64_t(delta.v[c])*delta.v[c];}
                int range=clamp(l.range.raw(),1,128*4096);if(square>=uint64_t(range)*range)continue;int distance=sqrt64(square);int attenuation=4096-uint32_t(distance)*4096/uint32_t(range);
                power=(power*attenuation/4096)*attenuation/4096;directions[i]=normalize(delta);
            }
            if(power<=0)continue;strength[i]=power;int value=clamp(l.priority,-100,100)*16384+power;
            // Hysteresis prevents nearest-light flicker around influence boundaries.
            if(slot && local_cache[object].selected==int(s.entity))value+=410;
            if(value>score[slot]){score[slot]=value;best[slot]=int(i);}
        }
        int32_t lm[3][3]={},cm[3][3]={};
        for(int slot=0;slot<2;++slot){int32_t colors[3]={};Vector d;
            if(best[slot]>=0){int i=best[slot];d=directions[i];const auto& l=objects[sources[i].entity].light;for(int c=0;c<3;++c)colors[c]=int32_t(l.color[c])*strength[i]/255;}
            if(slot){auto& cache=local_cache[object];cache.selected=best[slot]<0?-1:int(sources[best[slot]].entity);
                // Smooth local-light transitions over a few frames; first frame is immediate.
                for(int c=0;c<3;++c){cache.direction.v[c]=cache.initialized?(cache.direction.v[c]*3+d.v[c])/4:d.v[c];cache.color[c]=cache.initialized?(cache.color[c]*3+colors[c])/4:colors[c];colors[c]=cache.color[c];}d=cache.direction;cache.initialized=true;
            }
            for(int c=0;c<3;++c){lm[slot][c]=d.v[c];cm[c][slot]=colors[c];}
        }
        for(int r=0;r<3;++r)for(int c=0;c<3;++c)light_rows[r][c]=lm[r][c];
        {uint32_t sig=2166136261u;auto mix=[&](uint32_t v){sig=(sig^v)*16777619u;};
         for(int r=0;r<3;++r)for(int c=0;c<3;++c){mix(uint32_t(lm[r][c]));mix(uint32_t(cm[r][c]));}
         for(int c=0;c<3;++c){mix(uint32_t(lighting_environment.ambient[c].raw()));mix(material.color[c]);}
         signature=sig?sig:1;}
        write<Register::L11L12>(pair(lm[0][0],lm[0][1]));write<Register::L13L21>(pair(lm[0][2],lm[1][0]));write<Register::L22L23>(pair(lm[1][1],lm[1][2]));write<Register::L31L32>(uint32_t(0));write<Register::L33>(uint32_t(0));
        write<Register::LR1LR2>(pair(cm[0][0],cm[0][1]));write<Register::LR3LG1>(pair(0,cm[1][0]));write<Register::LG2LG3>(pair(cm[1][1],0));write<Register::LB1LB2>(pair(cm[2][0],cm[2][1]));write<Register::LB3>(uint32_t(0));
        write<Register::RBK>(uint32_t(clamp(lighting_environment.ambient[0].raw(),0,4096)));write<Register::GBK>(uint32_t(clamp(lighting_environment.ambient[1].raw(),0,4096)));write<Register::BBK>(uint32_t(clamp(lighting_environment.ambient[2].raw(),0,4096)));
        write<Register::RGB>(uint32_t(material.color[0])|(uint32_t(material.color[1])<<8)|(uint32_t(material.color[2])<<16));
        if(generic)return output;
        for(int f=0;f<6;++f){auto n=normals[object][f];write<Register::VXY0>(pair(n.v[0],n.v[1]));write<Register::VZ0>(uint32_t(n.v[2]));Kernels::ncs();auto rgb=readRaw<Register::RGB2>();output[f]=psyqo::Color{{.r=uint8_t((rgb&255)*material.color[0]/255),.g=uint8_t(((rgb>>8)&255)*material.color[1]/255),.b=uint8_t(((rgb>>16)&255)*material.color[2]/255)}};}
        lighting_work.gte_normals+=6;return output;
    }
    // Rotates the light matrix loaded by shade() into an object's local space
    // when its world basis is a rotation with uniform scale (within 1/256).
    // Face normals then feed NCS directly. Shear or non-uniform scale keeps the
    // registers untouched and returns false for the per-face cofactor path.
    bool localize(const Affine<Fixed>& world) {
        using namespace lighting_detail;using namespace psyqo::GTE;
        int64_t basis[3][3];for(int r=0;r<3;++r)for(int c=0;c<3;++c)basis[r][c]=world.values[r][c].raw();
        int64_t length[3]={};for(int c=0;c<3;++c)for(int r=0;r<3;++r)length[c]+=basis[r][c]*basis[r][c];
        if(length[0]<=0)return false;
        for(int c=1;c<3;++c){const int64_t d=length[c]-length[0];if((d<0?-d:d)*256>length[0])return false;}
        for(int a=0;a<3;++a)for(int b=a+1;b<3;++b){int64_t dot=0;for(int r=0;r<3;++r)dot+=basis[r][a]*basis[r][b];if((dot<0?-dot:dot)*256>length[0])return false;}
        const int32_t scale=int32_t(sqrt64(uint64_t(length[0])));if(scale<=0)return false;
        int32_t q[3][3];for(int r=0;r<3;++r)for(int c=0;c<3;++c)q[r][c]=int32_t(basis[r][c]*4096/scale);
        int32_t local[3][3];
        for(int r=0;r<3;++r)for(int c=0;c<3;++c){int64_t sum=0;for(int k=0;k<3;++k)sum+=int64_t(light_rows[r][k])*q[k][c];local[r][c]=int32_t(clamp(int32_t(sum/4096),-32768,32767));}
        write<Register::L11L12>(pair(local[0][0],local[0][1]));write<Register::L13L21>(pair(local[0][2],local[1][0]));write<Register::L22L23>(pair(local[1][1],local[1][2]));write<Register::L31L32>(pair(local[2][0],local[2][1]));write<Register::L33>(uint32_t(uint16_t(local[2][2])));
        return true;
    }
};
}
