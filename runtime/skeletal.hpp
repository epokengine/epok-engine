#pragma once
#include "epok.hpp"
#include "affine.hpp"
namespace epok {
// Shared scratch, reused between characters. No allocation and no per-frame vertex cache on disc.
namespace skeletal_detail {
inline Affine<Fixed> pose_matrix(const BonePose& p) {
    Fixed x(p.rotation[0],Fixed::RAW),y(p.rotation[1],Fixed::RAW),z(p.rotation[2],Fixed::RAW),w(p.rotation[3],Fixed::RAW);
    Fixed two=2.0,one=1.0;
    Affine<Fixed> m;
    m.values[0][0]=one-two*(y*y+z*z);m.values[0][1]=two*(x*y-z*w);m.values[0][2]=two*(x*z+y*w);
    m.values[1][0]=two*(x*y+z*w);m.values[1][1]=one-two*(x*x+z*z);m.values[1][2]=two*(y*z-x*w);
    m.values[2][0]=two*(x*z-y*w);m.values[2][1]=two*(y*z+x*w);m.values[2][2]=one-two*(x*x+y*y);
    for(int r=0;r<3;++r){for(int c=0;c<3;++c)m.values[r][c]*=Fixed(p.scale[c],Fixed::RAW);m.values[r][3]=Fixed(int32_t(p.translation[r])*16,Fixed::RAW);}
    return m;
}
struct Scratch {
    Affine<Fixed> bones[64];
    int16_t vertices[512][3];
    MeshGeometry geometry{};
    void pose(const SkeletalMesh& model, const Animator& animator) {
        const AnimationClip* clip=animator.clip>=0 && size_t(animator.clip)<model.clip_count?&model.clips[animator.clip]:nullptr;
        uint32_t frame=animator.ticks/2;
        if(clip){uint32_t length=clip->frames>1?clip->frames-1:1;if(animator.looping)frame%=length;else if(frame>=clip->frames)frame=clip->frames-1;}
        for(size_t i=0;i<model.bone_count;++i){
            const auto& bone=model.bones[i];const BonePose* p=&bone.bind;
            if(clip){const auto& track=clip->tracks[i];p=&track.poses[track.constant?0:frame];}
            auto local=pose_matrix(*p);bones[i]=bone.parent<0?local:bones[bone.parent].compose(local);
        }
        int32_t lo[3]={32767,32767,32767},hi[3]={-32768,-32768,-32768};
        for(size_t i=0;i<model.geometry->vertex_count;++i){
            Fixed local[3],out[3];for(int c=0;c<3;++c)local[c]=Fixed(model.geometry->vertices[i][c],Fixed::RAW);
            bones[model.vertex_bones[i]].point(local,out);
            for(int c=0;c<3;++c){int32_t v=out[c].raw();if(v< -32768)v=-32768;if(v>32767)v=32767;vertices[i][c]=int16_t(v);if(v<lo[c])lo[c]=v;if(v>hi[c])hi[c]=v;}
        }
        geometry=*model.geometry;geometry.vertices=vertices;
        for(int c=0;c<3;++c){geometry.center[c]=(lo[c]+hi[c])/2;geometry.extent[c]=(hi[c]-lo[c]+1)/2+2;}
    }
};
inline Scratch scratch;
}
}
