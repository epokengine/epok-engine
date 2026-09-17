#pragma once
#define EPOK_INCLUDE_FROM_SKELETAL 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_SKELETAL
#include "affine.hpp"
namespace epok {
// Shared scratch, reused between visible characters. RigidGte never writes vertices;
// BakedVertices decodes a single frame here, and CpuRigid is the lit-material fallback.
// The editor's table generator sizes every emitted array with these exact
// numbers (see src/skeletal_compile.rs). Assert the MIPS layout here so a
// change to any runtime struct fails the target build instead of silently
// invalidating the reported budget.
#ifdef __mips__
static_assert(sizeof(BonePose)==20,"BonePose layout changed; update the editor's size constants");
static_assert(sizeof(Bone)==22,"Bone layout changed; update the editor's size constants");
static_assert(sizeof(BoneTrack)==8,"BoneTrack layout changed; update the editor's size constants");
static_assert(sizeof(VertexFrame)==12,"VertexFrame layout changed; update the editor's size constants");
static_assert(sizeof(AnimationClip)==20,"AnimationClip layout changed; update the editor's size constants");
static_assert(sizeof(Material)==24,"Material layout changed; update the editor's size constants");
static_assert(sizeof(MeshQuad)==72,"MeshQuad layout changed; update the editor's size constants");
static_assert(sizeof(MeshGeometry)==72,"MeshGeometry layout changed; update the editor's size constants");
static_assert(sizeof(SkeletalMesh)==36,"SkeletalMesh layout changed; update the editor's size constants");
static_assert(sizeof(Animator)==20,"Animator layout changed; update the editor's size constants");
static_assert(sizeof(Affine<Fixed>)==48,"Affine<Fixed> layout changed; update the editor's size constants");
#endif

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
#ifdef __mips__
static_assert(sizeof(Scratch)==64*48+512*6+72,"Shared skeletal scratch layout changed; update the editor's size constants");
#endif
inline Scratch scratch;
}

namespace skeletal_query_detail {
inline SkeletalQueryStats& stats(){static SkeletalQueryStats value;return value;}
inline bool valid_pose(PoseKind pose){return pose==PoseKind::Bind||pose==PoseKind::Current;}
inline bool valid_space(CoordinateSpace space){return space==CoordinateSpace::Model||space==CoordinateSpace::World;}
inline const BonePose* selected_pose(const SkeletalMesh& model,const Animator& animator,size_t bone,PoseKind kind) {
    if(!model.bones||bone>=model.bone_count)return nullptr;
    if(kind==PoseKind::Bind)return &model.bones[bone].bind;
    if(animator.clip<0||size_t(animator.clip)>=model.clip_count)return &model.bones[bone].bind;
    const auto& clip=model.clips[animator.clip];
    if(!clip.tracks)return nullptr;
    const auto& track=clip.tracks[bone];
    if(!track.poses)return nullptr;
    return &track.poses[track.constant?0:skeletal_detail::Scratch::frame_index(&clip,animator)];
}
inline bool bone_matrix(const SkeletalMesh& model,const Animator& animator,uint32_t bone,PoseKind pose,Affine<Fixed>& output) {
    if(!model.bones||bone>=model.bone_count||model.bone_count>64)return false;
    uint16_t chain[64];size_t count=0;int current=int(bone);
    while(current>=0&&size_t(current)<model.bone_count&&count<64){
        for(size_t i=0;i<count;++i)if(chain[i]==uint16_t(current))return false;
        chain[count++]=uint16_t(current);current=model.bones[current].parent;
    }
    if(current>=0)return false;
    stats().bones+=uint32_t(count);output=Affine<Fixed>::identity();
    while(count){const size_t index=chain[--count];const auto* value=selected_pose(model,animator,index,pose);if(!value)return false;output=output.compose(skeletal_detail::pose_matrix(*value));}
    return true;
}
struct BatchScratch {Affine<Fixed> bones[64];};
inline BatchScratch& batch_scratch(){static BatchScratch value;return value;}
inline bool pose_all(const SkeletalMesh& model,const Animator& animator,PoseKind pose) {
    if(!model.bones||model.bone_count>64)return false;auto& output=batch_scratch();
    for(size_t index=0;index<model.bone_count;++index){const auto* value=selected_pose(model,animator,index,pose);if(!value)return false;const auto local=skeletal_detail::pose_matrix(*value);const int parent=model.bones[index].parent;if(parent>=int(index))return false;output.bones[index]=parent<0?local:output.bones[parent].compose(local);}stats().bones+=uint32_t(model.bone_count);return true;
}
inline bool baked_vertex(const SkeletalMesh& model,const Animator& animator,uint32_t vertex,PoseKind pose,Fixed* output) {
    if(!model.geometry||vertex>=model.geometry->vertex_count)return false;
    const auto* bind=model.geometry->vertices[vertex];
    if(pose==PoseKind::Bind||animator.clip<0||size_t(animator.clip)>=model.clip_count){for(int c=0;c<3;++c)output[c]=Fixed(bind[c],Fixed::RAW);return true;}
    const auto& clip=model.clips[animator.clip];
    if(!clip.vertex_frames||!clip.vertex_data){for(int c=0;c<3;++c)output[c]=Fixed(bind[c],Fixed::RAW);return true;}
    const auto& frame=clip.vertex_frames[skeletal_detail::Scratch::frame_index(&clip,animator)];
    uint32_t first=0;const uint8_t* source=clip.vertex_data+frame.offset;
    if(!frame.raw&&frame.seek){const uint32_t block=vertex/16;first=block*16;source=clip.vertex_data+frame.seek[block];}
    if(frame.raw){source+=size_t(vertex)*6;for(int c=0;c<3;++c){const int16_t value=int16_t(uint16_t(source[0])|(uint16_t(source[1])<<8));source+=2;output[c]=Fixed(value,Fixed::RAW);}stats().decoded_bytes+=6;return true;}
    const uint8_t* decoded_begin=source;
    for(uint32_t i=first;i<=vertex;++i)for(int c=0;c<3;++c){const int8_t delta=int8_t(*source++);int16_t value;if(delta==-128){value=int16_t(uint16_t(source[0])|(uint16_t(source[1])<<8));source+=2;}else value=int16_t(int32_t(model.geometry->vertices[i][c])+int32_t(delta)*16);if(i==vertex)output[c]=Fixed(value,Fixed::RAW);}
    stats().decoded_bytes+=uint32_t(source-decoded_begin);
    return true;
}
inline bool model_vertex(const SkeletalMesh& model,const Animator& animator,uint32_t portable,PoseKind pose,Fixed* output,SkeletalError& error) {
    if(!model.geometry||portable>=model.geometry->vertex_count){error=SkeletalError::InvalidVertex;return false;}
    if(model.storage==SkeletalStorage::BakedVertices){if(!baked_vertex(model,animator,portable,pose,output)){error=SkeletalError::MissingPoseData;return false;}return true;}
    const uint32_t cooked=model.portable_to_cooked?model.portable_to_cooked[portable]:portable;
    if(cooked>=model.geometry->vertex_count||!model.vertex_bones){error=SkeletalError::MissingPoseData;return false;}
    const uint32_t bone=model.vertex_bones[cooked];Affine<Fixed> matrix;
    if(!bone_matrix(model,animator,bone,pose,matrix)){error=SkeletalError::MissingPoseData;return false;}
    Fixed local[3];for(int c=0;c<3;++c)local[c]=Fixed(model.geometry->vertices[cooked][c],Fixed::RAW);matrix.point(local,output);return true;
}
inline bool apply_space(const ActorData& entity,CoordinateSpace space,Fixed* position,SkeletalError& error) {
    if(space==CoordinateSpace::Model)return true;
    Fixed transformed[3];if(!skeletal_world_point(entity,position,transformed)){error=SkeletalError::WorldUnavailable;return false;}
    for(int c=0;c<3;++c)position[c]=transformed[c];return true;
}
}

inline VertexSample skeletal_sample_vertex_impl(const ActorData* entity,uint32_t vertex,PoseKind pose,CoordinateSpace space,bool account_request) {
    VertexSample result;if(account_request){++skeletal_query_detail::stats().calls;++skeletal_query_detail::stats().vertices;}
    if(!entity||!entity->animator.model){result.error=SkeletalError::MissingModel;++skeletal_query_detail::stats().failures;return result;}
    if(!skeletal_query_detail::valid_pose(pose)){result.error=SkeletalError::MissingPoseData;++skeletal_query_detail::stats().failures;return result;}
    if(!skeletal_query_detail::valid_space(space)){result.error=SkeletalError::InvalidCoordinateSpace;++skeletal_query_detail::stats().failures;return result;}
    const auto& model=*entity->animator.model;
    if(!skeletal_query_detail::model_vertex(model,entity->animator,vertex,pose,result.position,result.error)){++skeletal_query_detail::stats().failures;return result;}
    if(!skeletal_query_detail::apply_space(*entity,space,result.position,result.error)){++skeletal_query_detail::stats().failures;return result;}
    const auto* clip=entity->animator.clip>=0&&size_t(entity->animator.clip)<model.clip_count?&model.clips[entity->animator.clip]:nullptr;
    result.sampled_frame=skeletal_detail::Scratch::frame_index(clip,entity->animator);result.error=SkeletalError::None;result.success=true;return result;
}

inline VertexSample skeletal_sample_vertex(const ActorData* entity,uint32_t vertex,PoseKind pose,CoordinateSpace space) {
    return skeletal_sample_vertex_impl(entity,vertex,pose,space,true);
}

inline VertexSamples4 skeletal_sample_vertices(const ActorData* entity,VertexIndexBatch4 indices,PoseKind pose,CoordinateSpace space) {
    VertexSamples4 output;++skeletal_query_detail::stats().calls;output.total=indices.count;output.count=indices.count<4?indices.count:4;skeletal_query_detail::stats().vertices+=output.count;
    const uint32_t values[4]={indices.index0,indices.index1,indices.index2,indices.index3};VertexSample* samples[4]={&output.sample0,&output.sample1,&output.sample2,&output.sample3};
    if(!entity||!entity->animator.model||!skeletal_query_detail::valid_pose(pose)||!skeletal_query_detail::valid_space(space)){
        for(uint32_t i=0;i<output.count;++i)*samples[i]=skeletal_sample_vertex_impl(entity,values[i],pose,space,false);return output;
    }
    const auto& model=*entity->animator.model;
    if(model.storage!=SkeletalStorage::BakedVertices&&model.geometry&&model.bones&&model.vertex_bones&&model.bone_count<=64){
        Animator sampled=entity->animator;if(pose==PoseKind::Bind)sampled.clip=-1;
        if(!skeletal_query_detail::pose_all(model,sampled,pose)){for(uint32_t i=0;i<output.count;++i){samples[i]->error=SkeletalError::MissingPoseData;++skeletal_query_detail::stats().failures;}return output;}
        const auto* clip=entity->animator.clip>=0&&size_t(entity->animator.clip)<model.clip_count?&model.clips[entity->animator.clip]:nullptr;
        const uint32_t frame=skeletal_detail::Scratch::frame_index(clip,entity->animator);
        for(uint32_t i=0;i<output.count;++i){
            bool repeated=false;for(uint32_t previous=0;previous<i;++previous)if(values[previous]==values[i]){*samples[i]=*samples[previous];repeated=true;break;}if(repeated)continue;
            if(values[i]>=model.geometry->vertex_count){samples[i]->error=SkeletalError::InvalidVertex;++skeletal_query_detail::stats().failures;continue;}
            const uint32_t cooked=model.portable_to_cooked?model.portable_to_cooked[values[i]]:values[i];if(cooked>=model.geometry->vertex_count){samples[i]->error=SkeletalError::InvalidVertex;++skeletal_query_detail::stats().failures;continue;}const uint32_t bone=model.vertex_bones[cooked];if(bone>=model.bone_count){samples[i]->error=SkeletalError::MissingPoseData;++skeletal_query_detail::stats().failures;continue;}Fixed local[3];for(int c=0;c<3;++c)local[c]=Fixed(model.geometry->vertices[cooked][c],Fixed::RAW);skeletal_query_detail::batch_scratch().bones[bone].point(local,samples[i]->position);if(!skeletal_query_detail::apply_space(*entity,space,samples[i]->position,samples[i]->error)){++skeletal_query_detail::stats().failures;continue;}samples[i]->sampled_frame=frame;samples[i]->error=SkeletalError::None;samples[i]->success=true;
        }
    }else for(uint32_t i=0;i<output.count;++i){bool repeated=false;for(uint32_t previous=0;previous<i;++previous)if(values[previous]==values[i]){*samples[i]=*samples[previous];repeated=true;break;}if(!repeated)*samples[i]=skeletal_sample_vertex_impl(entity,values[i],pose,space,false);}
    return output;
}

inline BoneSample skeletal_sample_bone(const ActorData* entity,uint32_t bone,PoseKind pose,CoordinateSpace space) {
    BoneSample result;++skeletal_query_detail::stats().calls;
    if(!entity||!entity->animator.model){result.error=SkeletalError::MissingModel;++skeletal_query_detail::stats().failures;return result;}
    if(!skeletal_query_detail::valid_pose(pose)){result.error=SkeletalError::MissingPoseData;++skeletal_query_detail::stats().failures;return result;}
    if(!skeletal_query_detail::valid_space(space)){result.error=SkeletalError::InvalidCoordinateSpace;++skeletal_query_detail::stats().failures;return result;}
    const auto& model=*entity->animator.model;if(!model.bones||bone>=model.bone_count){result.error=SkeletalError::InvalidBone;++skeletal_query_detail::stats().failures;return result;}
    Affine<Fixed> matrix;if(!skeletal_query_detail::bone_matrix(model,entity->animator,bone,pose,matrix)){result.error=SkeletalError::MissingPoseData;++skeletal_query_detail::stats().failures;return result;}
    if(space==CoordinateSpace::World){Fixed world_origin[3],point[3],world_point[3],translation[3];for(int row=0;row<3;++row)translation[row]=matrix.values[row][3];if(!skeletal_world_point(*entity,translation,world_origin)){result.error=SkeletalError::WorldUnavailable;++skeletal_query_detail::stats().failures;return result;}for(int column=0;column<3;++column){for(int row=0;row<3;++row)point[row]=translation[row]+matrix.values[row][column];if(!skeletal_world_point(*entity,point,world_point)){result.error=SkeletalError::WorldUnavailable;++skeletal_query_detail::stats().failures;return result;}for(int row=0;row<3;++row)matrix.values[row][column]=world_point[row]-world_origin[row];}for(int row=0;row<3;++row)matrix.values[row][3]=world_origin[row];}
    for(int row=0;row<3;++row){result.basis_x[row]=matrix.values[row][0];result.basis_y[row]=matrix.values[row][1];result.basis_z[row]=matrix.values[row][2];result.position[row]=matrix.values[row][3];}
    result.parent=model.bones[bone].parent;const auto* clip=entity->animator.clip>=0&&size_t(entity->animator.clip)<model.clip_count?&model.clips[entity->animator.clip]:nullptr;result.sampled_frame=skeletal_detail::Scratch::frame_index(clip,entity->animator);result.error=SkeletalError::None;result.success=true;return result;
}
}
