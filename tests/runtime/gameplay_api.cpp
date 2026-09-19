#include <cassert>
#include <cstdio>
#include "../../runtime/epok.hpp"
#include "../../runtime/retained.hpp"

namespace epok {
const ClassDescriptor object_classes[1] = {};
const size_t object_class_count = 0;
ActorData* DataHandle::get() const { return nullptr; }
DataHandle handle(const ActorData*) { return {}; }
bool is_active(const ActorData*) { return false; }
bool skeletal_world_point(const ActorData&,const Fixed* model,Fixed* world) {
    if(!model||!world)return false;
    world[0]=model[0]+Fixed(10.0);
    world[1]=model[1]+Fixed(20.0);
    world[2]=model[2]+Fixed(30.0);
    return true;
}
}

static epok::BonePose identity_pose() {
    epok::BonePose value{};
    value.rotation[3]=4096;
    value.scale[0]=value.scale[1]=value.scale[2]=4096;
    return value;
}

static void skeletal_queries() {
    int16_t vertices[2][3]={{0,0,0},{8192,0,0}};
    epok::MeshGeometry geometry{};
    geometry.vertices=vertices;
    geometry.vertex_count=2;
    const uint8_t vertex_bones[2]={0,0};
    const uint16_t portable_to_cooked[2]={1,0};
    const epok::Bone bones[1]={{-1,identity_pose()}};
    epok::SkeletalMesh rigid{};
    rigid.geometry=&geometry;
    rigid.vertex_bones=vertex_bones;
    rigid.portable_to_cooked=portable_to_cooked;
    rigid.bones=bones;
    rigid.bone_count=1;
    rigid.storage=epok::SkeletalStorage::CpuRigid;
    epok::ActorData actor{};
    actor.animator.enabled=true;
    actor.animator.model=&rigid;
    epok::ActorData other=actor;

    epok::ResourceLibrary::clear_skeletal_queries();
    auto model=epok::skeletal_sample_vertex(&actor,0,epok::PoseKind::Bind,epok::CoordinateSpace::Model);
    auto world=epok::skeletal_sample_vertex(&other,1,epok::PoseKind::Bind,epok::CoordinateSpace::World);
    assert(world.success&&world.position[0].raw()==10*4096&&world.position[1].raw()==20*4096);
    // A second actor/query cannot mutate the first owned result, and model-space
    // zero remains a successful position rather than a failure sentinel.
    assert(model.success&&model.position[0].raw()==8192&&model.position[1].raw()==0);
    auto invalid=epok::skeletal_sample_vertex(&actor,2,epok::PoseKind::Bind,epok::CoordinateSpace::Model);
    assert(!invalid.success&&invalid.error==epok::SkeletalError::InvalidVertex);

    epok::VertexIndexBatch4 indices{};
    indices.count=4;
    indices.index0=0;
    indices.index1=0;
    indices.index2=1;
    indices.index3=9;
    auto batch=epok::skeletal_sample_vertices(&actor,indices,epok::PoseKind::Bind,epok::CoordinateSpace::Model);
    assert(batch.count==4&&batch.total==4);
    assert(batch.sample0.success&&batch.sample1.success&&batch.sample2.success);
    assert(batch.sample0.position[0].raw()==batch.sample1.position[0].raw());
    assert(batch.sample2.position[0].raw()==0);
    assert(!batch.sample3.success&&batch.sample3.error==epok::SkeletalError::InvalidVertex);
    auto stats=epok::ResourceLibrary::skeletal_queries();
    assert(stats.calls==4&&stats.vertices==7&&stats.failures==2);

    // Portable IDs survive many-to-one cooking, including IDs beyond the
    // cooked geometry count (weapon sockets use authored vertex indices).
    const uint16_t duplicates[4]={1,0,1,0};
    rigid.portable_to_cooked=duplicates;rigid.portable_vertex_count=4;
    auto duplicate=epok::skeletal_sample_vertex(&actor,2,epok::PoseKind::Bind,epok::CoordinateSpace::Model);
    assert(duplicate.success&&duplicate.position[0].raw()==8192);
    indices.index0=2;indices.index1=3;indices.index2=4;indices.index3=2;
    batch=epok::skeletal_sample_vertices(&actor,indices,epok::PoseKind::Bind,epok::CoordinateSpace::Model);
    assert(batch.sample0.success&&batch.sample0.position[0].raw()==8192);
    assert(batch.sample1.success&&batch.sample1.position[0].raw()==0);
    assert(!batch.sample2.success&&batch.sample2.error==epok::SkeletalError::InvalidVertex);
    assert(batch.sample3.success&&batch.sample3.position[0].raw()==8192);
    rigid.portable_to_cooked=portable_to_cooked;rigid.portable_vertex_count=0;

    const uint8_t encoded[6]={0,0,0,1,0,0};
    const uint32_t seek[1]={0};
    const epok::VertexFrame frame[1]={{0,seek,false}};
    const epok::BonePose poses[1]={identity_pose()};
    const epok::BoneTrack tracks[1]={{poses,true}};
    const epok::AnimationClip clips[1]={{tracks,frame,encoded,1,"Idle"}};
    int16_t portable_vertices[2][3]={{8192,0,0},{0,0,0}};
    epok::MeshGeometry baked_geometry=geometry;
    baked_geometry.vertices=portable_vertices;
    epok::SkeletalMesh baked=rigid;
    baked.geometry=&baked_geometry;
    baked.portable_to_cooked=nullptr;
    baked.clips=clips;
    baked.clip_count=1;
    baked.storage=epok::SkeletalStorage::BakedVertices;
    actor.animator.model=&baked;
    actor.animator.clip=0;
    auto baked_vertex=epok::skeletal_sample_vertex(&actor,1,epok::PoseKind::Current,epok::CoordinateSpace::Model);
    assert(baked_vertex.success&&baked_vertex.position[0].raw()==16);
    auto bone=epok::skeletal_sample_bone(&actor,0,epok::PoseKind::Current,epok::CoordinateSpace::Model);
    assert(bone.success&&bone.parent==-1&&bone.basis_x[0].raw()==4096);
}

static void skeletal_matrix_fast_paths() {
    using epok::Fixed;
    uint32_t rng=7;
    auto next=[&](){rng=rng*1664525u+1013904223u;return rng;};
    for(int sample=0;sample<10000;++sample){
        auto p=identity_pose();
        for(int k=0;k<4;++k)p.rotation[k]=int16_t(int(next()%8193)-4096);
        if(sample%3==0)p.rotation[0]=p.rotation[1]=p.rotation[2]=0;
        for(int k=0;k<3;++k){p.translation[k]=int16_t(next());p.scale[k]=sample%2?4096:int16_t(next());}
        Fixed x(p.rotation[0],Fixed::RAW),y(p.rotation[1],Fixed::RAW),z(p.rotation[2],Fixed::RAW),w(p.rotation[3],Fixed::RAW),two=2.0,one=1.0;
        epok::Affine<Fixed> ref;
        ref.values[0][0]=one-two*(y*y+z*z);ref.values[0][1]=two*(x*y-z*w);ref.values[0][2]=two*(x*z+y*w);
        ref.values[1][0]=two*(x*y+z*w);ref.values[1][1]=one-two*(x*x+z*z);ref.values[1][2]=two*(y*z-x*w);
        ref.values[2][0]=two*(x*z-y*w);ref.values[2][1]=two*(y*z+x*w);ref.values[2][2]=one-two*(x*x+y*y);
        for(int r=0;r<3;++r){for(int c=0;c<3;++c)ref.values[r][c]*=Fixed(p.scale[c],Fixed::RAW);ref.values[r][3]=Fixed(int32_t(p.translation[r])*16,Fixed::RAW);}
        const auto result=epok::skeletal_detail::pose_matrix(p);
        for(int r=0;r<3;++r)for(int c=0;c<4;++c)assert(result.values[r][c].raw()==ref.values[r][c].raw());
    }
}

static void animated_packet_retention() {
    assert(epok::retain_mesh_packets(true,false,false));
    assert(!epok::retain_mesh_packets(true,true,false));
    assert(!epok::retain_mesh_packets(true,false,true));
    assert(epok::retain_mesh_packets(false,true,true));
    epok::RetainedGeometry<3,4> pool;
    epok::MeshGeometry geometry{};
    epok::MeshQuad faces[2]{};
    geometry.editable=true;geometry.quads=faces;geometry.quad_count=2;
    faces[0].material.unlit=faces[1].material.unlit=true;
    assert(epok::mesh_faces_unlit(&geometry));
    faces[1].material.unlit=false;assert(!epok::mesh_faces_unlit(&geometry));
    faces[1].material.unlit=true;
    epok::MeshGeometry tail{};geometry.next=&tail;
    assert(!epok::mesh_faces_unlit(&geometry));geometry.next=nullptr;
    assert(!epok::mesh_faces_unlit(nullptr));
    epok::MeshMaterialCache materials;
    assert(!materials.faces_unlit(nullptr));
    assert(materials.faces_unlit(&geometry));
    assert(materials.faces_unlit(&geometry));
    assert(!materials.faces_unlit(&tail)); // a different model invalidates
    assert(materials.faces_unlit(&geometry));
    materials={}; // scene/bank storage may reuse the same address
    faces[1].material.unlit=false;
    assert(!materials.faces_unlit(&geometry));
    faces[1].material.unlit=true;
    assert(pool.allocate(0,&geometry,3,8));
    assert(pool.slot_top()==6);
    assert(pool.allocate(0,&geometry,3,8)); // new poses do not allocate again
    assert(pool.slot_top()==6);
    assert(!pool.allocate(1,&geometry,2,8)); // safe dynamic fallback
    auto& state=pool.state(0);
    state.key[0].valid=state.key[1].valid=true;
    assert(state.key[0]==state.key[1]);
    ++state.key[0].color[0];assert(!(state.key[0]==state.key[1]));
    pool.forget(0);assert(!state.key[0].valid&&!state.key[1].valid);
}

static void persistent_utilities() {
    auto tween=epok::UtilityLibrary::tween_start(0.0,10.0,2.0,epok::Ease::SmoothStep);
    auto halfway=epok::UtilityLibrary::tween_advance(tween,1.0);
    assert(halfway.value.raw()==5*4096&&!halfway.completed&&halfway.state.running);
    auto finished=epok::UtilityLibrary::tween_advance(halfway.state,1.0);
    assert(finished.value.raw()==10*4096&&finished.completed&&!finished.state.running);
    auto cancelled=epok::UtilityLibrary::tween_cancel(finished.state);
    assert(!cancelled.running&&!cancelled.completion_pending);

    auto queue=epok::UtilityLibrary::event_queue_clear();
    for(uint32_t kind=1;kind<=4;++kind){
        auto emitted=epok::UtilityLibrary::event_queue_emit(queue,kind,int32_t(kind*10),{});
        assert(emitted.accepted);
        queue=emitted.state;
    }
    auto overflow=epok::UtilityLibrary::event_queue_emit(queue,5,50,{});
    assert(!overflow.accepted&&overflow.state.count==4&&overflow.state.dropped==1);
    auto polled=epok::UtilityLibrary::event_queue_poll(overflow.state);
    assert(polled.valid&&polled.event.kind==1&&polled.event.value==10);
    assert(polled.state.count==3&&polled.state.dropped==1);
}

int main() {
    epok::input.reset();
    auto disconnected=epok::InputLibrary::axis(epok::Axis::LeftX,0);
    assert(!disconnected.connected&&!disconnected.analog&&disconnected.value.raw()==0);

    epok::input.sample(0,true,uint16_t(1u<<uint8_t(epok::Button::Cross)),true,255,0,128,128);
    epok::input.begin_tick();
    auto horizontal=epok::InputLibrary::axis(epok::Axis::LeftX,0);
    auto vertical=epok::InputLibrary::axis(epok::Axis::LeftY,0);
    assert(horizontal.connected&&horizontal.analog&&horizontal.value.raw()==4096);
    assert(vertical.value.raw()==4096);
    assert(epok::InputLibrary::held(epok::Button::Cross,0));
    assert(epok::InputLibrary::pressed(epok::Button::Cross,0));
    assert(!epok::InputLibrary::axis(epok::Axis(9),0).value.raw());
    assert(!epok::InputLibrary::connected(2));

    epok::time.reset(10);
    epok::time.advance(16677);
    epok::time.begin_tick();
    auto clock=epok::TimeLibrary::snapshot();
    assert(!clock.paused&&clock.simulation_ticks==1&&clock.fixed_delta.raw()==68);
    epok::TimeLibrary::set_paused(true);
    assert(epok::TimeLibrary::paused());
    assert(epok::TimeLibrary::snapshot().paused);

    auto sum=epok::MathLibrary::add({1.0,2.0,3.0},{4.0,5.0,6.0});
    assert(sum.x.raw()==5*4096&&sum.y.raw()==7*4096&&sum.z.raw()==9*4096);
    assert(epok::MathLibrary::clamp(4.0,1.0,3.0).raw()==3*4096);

    epok::MemoryCardLibrary::clear_staged_payload();
    assert(epok::MemoryCardLibrary::set_staged_word(0,0x78563412u));
    assert(epok::MemoryCardLibrary::set_staged_word(1023,0xa5a5a5a5u));
    assert(!epok::MemoryCardLibrary::set_staged_word(1024,1));
    assert(epok::MemoryCardLibrary::staged_word(0)==0x78563412u);
    assert(epok::MemoryCardLibrary::staged_word(1023)==0xa5a5a5a5u);
    assert(epok::MemoryCardLibrary::staged_word(1024)==0);

    epok::FocusLibrary::clear();
    auto focus=epok::FocusLibrary::snapshot();
    assert(!focus.valid&&focus.count==0&&!focus.current.valid());
    skeletal_queries();
    skeletal_matrix_fast_paths();
    animated_packet_retention();
    persistent_utilities();
    std::puts("Gameplay libraries: typed records, input/time, math, bounded payload and focus pass.");
}
