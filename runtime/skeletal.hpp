#pragma once
#include "epok.hpp"
#include "affine.hpp"
namespace epok {
// Shared scratch, reused between visible characters. RigidGte never writes vertices;
// BakedVertices decodes a single frame here, and CpuRigid is the lit-material fallback.
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
    static uint32_t frame_index(const AnimationClip* clip,const Animator& animator) {
        if(!clip)return 0;
        uint32_t frame=animator.ticks/2;
        uint32_t length=clip->frames>1?clip->frames-1:1;
        if(animator.looping)frame%=length;
        else if(frame>=clip->frames)frame=clip->frames-1;
        return frame;
    }
    void pose_bones(const SkeletalMesh& model,const Animator& animator) {
        const AnimationClip* clip=animator.clip>=0 && size_t(animator.clip)<model.clip_count?&model.clips[animator.clip]:nullptr;
        uint32_t frame=frame_index(clip,animator);
        for(size_t i=0;i<model.bone_count;++i){
            const auto& bone=model.bones[i];const BonePose* p=&bone.bind;
            if(clip&&clip->tracks){const auto& track=clip->tracks[i];p=&track.poses[track.constant?0:frame];}
            auto local=pose_matrix(*p);bones[i]=bone.parent<0?local:bones[bone.parent].compose(local);
        }
    }
    void skin_vertices(const SkeletalMesh& model) {
        for(size_t i=0;i<model.geometry->vertex_count;++i){
            Fixed local[3],out[3];for(int c=0;c<3;++c)local[c]=Fixed(model.geometry->vertices[i][c],Fixed::RAW);
            bones[model.vertex_bones[i]].point(local,out);
            for(int c=0;c<3;++c){int32_t v=out[c].raw();if(v< -32768)v=-32768;if(v>32767)v=32767;vertices[i][c]=int16_t(v);}
        }
        geometry=*model.geometry;geometry.vertices=vertices;
    }
    void decode_vertices(const SkeletalMesh& model,const Animator& animator) {
        geometry=*model.geometry;
        const AnimationClip* clip=animator.clip>=0 && size_t(animator.clip)<model.clip_count?&model.clips[animator.clip]:nullptr;
        if(!clip||!clip->vertex_frames||!clip->vertex_data){return;}
        const auto& encoded=clip->vertex_frames[frame_index(clip,animator)];
        const uint8_t* source=clip->vertex_data+encoded.offset;
        for(size_t i=0;i<model.geometry->vertex_count;++i){
            for(int c=0;c<3;++c){
                if(encoded.raw){vertices[i][c]=int16_t(uint16_t(source[0])|(uint16_t(source[1])<<8));source+=2;continue;}
                int8_t delta=int8_t(*source++);
                if(delta==-128){vertices[i][c]=int16_t(uint16_t(source[0])|(uint16_t(source[1])<<8));source+=2;}
                else vertices[i][c]=int16_t(int32_t(model.geometry->vertices[i][c])+int32_t(delta)*16);
            }
        }
        geometry.vertices=vertices;
    }
    void pose(const SkeletalMesh& model,const Animator& animator) {
        if(model.storage==SkeletalStorage::BakedVertices){decode_vertices(model,animator);return;}
        pose_bones(model,animator);skin_vertices(model);
    }
};
inline Scratch scratch;
}
}
