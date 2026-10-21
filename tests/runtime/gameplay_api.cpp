#include <cassert>
#include <cstdio>
#include "../../runtime/epok.hpp"
#include "../../runtime/retained.hpp"
#include "../../runtime/fixed_math.hpp"
#include "../../runtime/skeletal.hpp"

namespace epok {
const ClassDescriptor object_classes[1] = {};
const size_t object_class_count = 0;
ActorData* DataHandle::get() const { return nullptr; }
DataHandle handle(const ActorData*) { return {}; }
bool is_active(const ActorData*) { return false; }
// No actor resolves on the host, so actors have no world placement.
WorldAffineSample gameplay_world_affine(const ActorData* entity) {
    WorldAffineSample result;
    if(entity)result.success=true;
    return result;
}
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

static void visible_packet_ownership() {
    // Main regression: an invisible cache covers 2,044 slots, but no GPU
    // packet is claimed. All 2,048 dynamic triangles must still fit.
    epok::FramePacketSlots<2048> slots;
    epok::RetainedGeometry<3,1024> cache;
    epok::MeshGeometry geometry{};
    assert(cache.allocate(0,&geometry,1022,2048));
    cache.state(0).key[0].valid=cache.state(0).key[1].valid=true;
    slots.begin_frame();
    for(uint32_t i=2048;i>0;--i)assert(slots.allocate_dynamic()==i-1);
    assert(slots.allocate_dynamic()==2048);
    cache.invalidate_borrowed(0,slots.dynamic_low());
    assert(!cache.state(0).key[0].valid&&cache.state(0).key[1].valid);
    // Invalidation survives invisible frames; only the overwritten parity
    // rebuilds when the object eventually comes back into view.
    slots.begin_frame();
    cache.invalidate_borrowed(1,slots.dynamic_low());
    assert(!cache.state(0).key[0].valid&&cache.state(0).key[1].valid);

    epok::FramePacketSlots<70> mixed;
    mixed.begin_frame();
    assert(mixed.claim(30,5)); // straddles two bitmap words
    assert(!mixed.claim(0,71)&&!mixed.claim(UINT32_MAX,2));
    assert(!mixed.claim(10,21)); // rejection must not claim its prefix
    assert(mixed.claim(10,20));
    assert(!mixed.claim(10,0)&&!mixed.claim(30,5));
    bool seen[70]={};
    for(unsigned n=0;n<45;++n){
        const auto slot=mixed.allocate_dynamic();
        assert(slot<70&&!seen[slot]&&(slot<10||slot>=35));seen[slot]=true;
    }
    assert(mixed.allocate_dynamic()==70);
    mixed.begin_frame();
    assert(mixed.allocate_dynamic()==69);
    assert(!mixed.claim(68,2)); // cannot overwrite already-linked dynamic work
    assert(mixed.claim(0,69));
    assert(mixed.allocate_dynamic()==70);
    mixed.begin_frame();
    assert(mixed.claim(0,70)&&mixed.allocate_dynamic()==70);

    epok::MeshGeometry root{},tail{};root.quad_count=2;root.next=&tail;tail.quad_count=1;
    epok::RetainedGeometry<1,3> chunks;
    epok::FramePacketSlots<8> chunk_slots;
    assert(chunks.allocate(0,&root,3,8));
    auto* chunk_records=chunks.quads(0);
    auto initialize_chunks=[&](){
        chunks.state(0).key[0].valid=true;
        chunk_records[0].valid_parities=chunk_records[2].valid_parities=15;
        chunk_slots.begin_frame();
    };
    uint32_t earliest=UINT32_MAX;
    auto borrowed=[&](size_t index,uint32_t offset){assert(index==0);if(offset<earliest)earliest=offset;};
    initialize_chunks();chunks.invalidate_borrowed(0,5,chunk_slots,borrowed);
    assert(chunks.state(0).key[0].valid&&chunk_records[0].valid_parities==15);
    assert(chunk_records[2].valid_parities==10&&earliest==2);
    initialize_chunks();earliest=UINT32_MAX;
    assert(chunk_slots.claim(0,4)&&chunk_slots.range_claimed(0,4));
    assert(!chunk_slots.range_claimed(0,5)&&!chunk_slots.range_claimed(UINT32_MAX,2));
    chunks.invalidate_borrowed(0,3,chunk_slots,borrowed);
    assert(chunks.state(0).key[0].valid&&chunk_records[0].valid_parities==15);
    assert(chunk_records[2].valid_parities==10&&earliest==2);
    initialize_chunks();earliest=UINT32_MAX;
    chunks.invalidate_borrowed(1,3,chunk_slots,borrowed);
    assert(chunk_records[0].valid_parities==5&&chunk_records[2].valid_parities==5&&earliest==0);

    mixed.begin_frame();
    assert(mixed.claim(67,2)); // Slot 69 alone cannot hold a quad.
    assert(mixed.allocate_dynamic_pair()==65); // 65,66 are adjacent and free.
    assert(!mixed.claim(66,1));
    assert(mixed.allocate_dynamic()==64);
    mixed.begin_frame();
    assert(mixed.claim(1,69));
    assert(mixed.allocate_dynamic_pair()==70);
    assert(mixed.allocate_dynamic()==0); // Failed pair preserves the singleton.

    uint32_t pair_rng=41768;
    for(unsigned frame=0;frame<400;++frame){
        mixed.begin_frame();bool occupied[70]={};uint32_t cursor=70;
        for(unsigned step=0;step<150;++step){
            pair_rng=pair_rng*1664525u+1013904223u;
            const unsigned operation=(pair_rng>>16)%3;
            if(operation==0){
                const uint32_t first=(pair_rng>>8)%76,count=(pair_rng>>20)%40;
                bool expected=count&&first<cursor&&count<=cursor-first;
                if(expected)for(uint32_t s=first;s<first+count;++s)expected&=!occupied[s];
                assert(mixed.claim(first,count)==expected);
                if(expected)for(uint32_t s=first;s<first+count;++s)occupied[s]=true;
            }else if(operation==1){
                uint32_t at=cursor,expected=70;
                while(at){--at;if(!occupied[at]){expected=at;break;}}
                assert(mixed.allocate_dynamic()==expected);
                cursor=expected<70?expected:0;
                if(expected<70)occupied[expected]=true;
            }else{
                uint32_t expected=70;
                for(uint32_t at=cursor;at>=2;--at)if(!occupied[at-1]&&!occupied[at-2]){expected=at-2;break;}
                assert(mixed.allocate_dynamic_pair()==expected);
                if(expected<70){occupied[expected]=occupied[expected+1]=true;cursor=expected;}
            }
        }
    }

    // Differential allocator test, including partial final words, overlaps,
    // exhaustion, arbitrary object ordering and frame resets.
    uint32_t rng=17;
    for(unsigned frame=0;frame<1000;++frame){
        mixed.begin_frame();bool occupied[70]={};uint32_t cursor=70;
        for(unsigned step=0;step<150;++step){
            rng=rng*1664525u+1013904223u;
            if(rng&1){
                const uint32_t first=(rng>>8)%76,count=(rng>>20)%40;
                bool expected=count&&first<cursor&&count<=cursor-first;
                if(expected)for(uint32_t s=first;s<first+count;++s)expected&=!occupied[s];
                assert(mixed.claim(first,count)==expected);
                if(expected)for(uint32_t s=first;s<first+count;++s)occupied[s]=true;
            }else{
                uint32_t expected=70;
                while(cursor){--cursor;if(!occupied[cursor]){expected=cursor;occupied[cursor]=true;break;}}
                assert(mixed.allocate_dynamic()==expected);
            }
        }
    }
}

static void integer_square_roots(){
    auto reference=[](uint64_t v){uint64_t root=0,bit=uint64_t(1)<<62;while(bit>v)bit>>=2;while(bit){if(v>=root+bit){v-=root+bit;root=(root>>1)+bit;}else root>>=1;bit>>=2;}return uint32_t(root);};
    uint64_t seed=0x1528ead;
    for(unsigned i=0;i<200000;++i){
        seed=seed*6364136223846793005ULL+1;
        const uint64_t value=i&1?seed:uint32_t(seed>>32);
        assert(epok::fixed_math::sqrt64(value)==reference(value));
    }
    for(uint64_t n=0;n<=65535;++n){
        assert(epok::fixed_math::sqrt64(n*n)==n);
        if(n)assert(epok::fixed_math::sqrt64(n*n-1)==n-1);
    }
    assert(epok::fixed_math::sqrt64(UINT64_MAX)==UINT32_MAX);
}

static void animated_packet_retention() {
    epok::Bone bones[2]{};bones[0].parent=-1;bones[1].parent=0;
    for(auto& b:bones){b.bind.rotation[3]=4096;for(auto& scale:b.bind.scale)scale=4096;}
    bones[1].bind.translation[1]=256;
    epok::AnimationClip clip{};clip.frames=8;
    epok::SkeletalMesh skeleton{};skeleton.bones=bones;skeleton.bone_count=2;skeleton.clips=&clip;skeleton.clip_count=1;
    epok::Animator animator;animator.clip=0;
    epok::skeletal_detail::Scratch scratch,reference;
    int16_t stops[2]={-1,-1};
    for(unsigned frame=0;frame<40;++frame){
        animator.ticks=uint16_t(frame/2);const auto pitch=frame<20?epok::Fixed(0.0):epok::Fixed(10.0);
        if(frame==30)stops[0]=1;
        const auto rebuilt=scratch.pose_bones(skeleton,animator,0,stops,pitch,frame<35?1:2);
        reference.pose_bones(skeleton,animator,0,stops,pitch);
        for(unsigned b=0;b<2;++b)for(unsigned r=0;r<3;++r)for(unsigned c=0;c<4;++c)assert(scratch.bones[b].values[r][c]==reference.bones[b].values[r][c]);
        if(frame==1)assert(!rebuilt);
        if(frame==0||frame==20||frame==30||frame==35)assert(rebuilt==2);
    }
    // Mutable callers do not opt in: changing a bind pose is observed at once.
    bones[1].bind.translation[1]=512;
    assert(scratch.pose_bones(skeleton,animator,0,stops,0.0)==2);
    assert(scratch.bones[1].values[1][3]==epok::Fixed(2.0));
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
    assert(materials.uniform&&materials.uniform_color==0xffffff);
    assert(materials.faces_unlit(&geometry));
    assert(!materials.faces_unlit(&tail)); // a different model invalidates
    assert(materials.faces_unlit(&geometry));
    materials={}; // scene/bank storage may reuse the same address
    faces[1].material.unlit=false;
    assert(!materials.faces_unlit(&geometry));
    faces[1].material.unlit=true;
    materials={};faces[1].material.color[0]=100;
    assert(materials.faces_unlit(&geometry)&&!materials.uniform);
    materials={};faces[1].material.color[0]=255;faces[1].vertex_colors=true;
    assert(materials.faces_unlit(&geometry)&&!materials.uniform);
    faces[1].vertex_colors=false;
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
    // Tint updates preserve attributes only in initialized, owned chunks.
    pool.reset();assert(pool.allocate(0,&geometry,3,8));
    epok::RetainedKey key;key.geometry=&geometry;key.valid=true;key.generation=1;
    pool.prepare_key(0,0,key,true);pool.prepare_key(0,1,key,true);
    auto* records=pool.quads(0);
    records[0].valid_parities=3;records[0].attribute_revision=state.attribute_revision;
    records[2].valid_parities=3;records[2].attribute_revision=0; // dormant old chunk
    key.color[0]=128;pool.prepare_key(0,0,key,true);
    assert((records[0].valid_parities&5)==5);
    assert(!(records[2].valid_parities&5));
    // A material change and a later return cannot recolor stale shared metadata.
    key.texture=3;pool.prepare_key(0,1,key,true);
    assert(!(records[0].valid_parities&2));
    records[0].attribute_revision=state.attribute_revision;records[0].valid_parities|=2;
    key.texture=-1;pool.prepare_key(0,0,key,true);
    assert(!(records[0].valid_parities&5));
    records[0].attribute_revision=state.attribute_revision;records[0].valid_parities|=1;
    pool.invalidate_borrowed(0,0);key.color[0]=99;pool.prepare_key(0,0,key,true);
    assert(!(records[0].valid_parities&5)); // borrowed UVs require the full build
    records[0].attribute_revision=state.attribute_revision;records[0].valid_parities|=1;
    pool.forget(0);key.color[0]=17;pool.prepare_key(0,0,key,true);
    assert(!(records[0].valid_parities&5));
    // A repeated near/far LOD change reuses this actor's resident range.
    for(unsigned i=0;i<1000;++i){
        assert(pool.allocate(0,&tail,1,8));assert(pool.slot_top()==6);
        assert(pool.allocate(0,&geometry,3,8));assert(pool.slot_top()==6);
        assert(!state.key[0].valid&&!state.key[1].valid);
    }
}

// The scripted fog facade over the raw `fog_environment` store. The accepted range
// is the editor validator's; src/effects.rs checks the same candidates against that
// validator, so neither end of the facade can drift alone.
static void scene_fog_range() {
    epok::fog_environment=epok::FogEnvironment{};
    const auto defaults=epok::SceneLibrary::fog();
    assert(!defaults.enabled&&defaults.start_distance.raw()==12*4096&&defaults.end_distance.raw()==40*4096);
    assert(defaults.red==64&&defaults.green==77&&defaults.blue==102);
    // The record's own defaults are the authored scene defaults, so an untouched
    // Blueprint/Lua pin writes back exactly what the editor would.
    const epok::FogSettings declared;
    assert(declared.enabled==defaults.enabled&&declared.start_distance.raw()==defaults.start_distance.raw());
    assert(declared.end_distance.raw()==defaults.end_distance.raw());
    assert(declared.red==defaults.red&&declared.green==defaults.green&&declared.blue==defaults.blue);

    epok::FogSettings value;
    value.enabled=true;value.start_distance=0.0;value.end_distance=128.0;
    value.red=300;value.green=77;value.blue=0;
    assert(epok::SceneLibrary::set_fog(value));
    assert(epok::fog_environment.enabled&&epok::fog_environment.start==0&&epok::fog_environment.end==128*4096);
    // Channels clamp exactly as a transition's loading color does.
    assert(epok::fog_environment.color[0]==255&&epok::fog_environment.color[1]==77&&epok::fog_environment.color[2]==0);
    const auto applied=epok::SceneLibrary::fog();
    assert(applied.enabled&&applied.start_distance.raw()==0&&applied.end_distance.raw()==128*4096);
    assert(applied.red==255&&applied.green==77&&applied.blue==0);

    // One Q12 step is the narrowest accepted span.
    epok::FogSettings narrow=value;
    narrow.start_distance=5.0;narrow.end_distance=epok::Fixed(5*4096+1,epok::Fixed::RAW);
    assert(epok::SceneLibrary::set_fog(narrow));
    assert(epok::fog_environment.start==5*4096&&epok::fog_environment.end==5*4096+1);

    // A rejected range applies nothing at all: the flag, both distances and the
    // three channels keep the values the last accepted call left behind.
    const epok::FogEnvironment before=epok::fog_environment;
    epok::FogSettings rejected[4]={narrow,narrow,narrow,narrow};
    rejected[0].start_distance=epok::Fixed(-1,epok::Fixed::RAW);
    rejected[1].end_distance=epok::Fixed(128*4096+1,epok::Fixed::RAW);
    rejected[2].end_distance=rejected[2].start_distance;
    rejected[3].end_distance=1.0;
    for(const auto& candidate:rejected){
        epok::FogSettings copy=candidate;
        copy.enabled=!before.enabled;copy.red=1;copy.green=2;copy.blue=3;
        assert(!epok::SceneLibrary::set_fog(copy));
        assert(epok::fog_environment.enabled==before.enabled);
        assert(epok::fog_environment.start==before.start&&epok::fog_environment.end==before.end);
        assert(epok::fog_environment.color[0]==before.color[0]&&epok::fog_environment.color[1]==before.color[1]);
        assert(epok::fog_environment.color[2]==before.color[2]);
    }
    epok::fog_environment=epok::FogEnvironment{};
}

// The scripted screen-fade facade over the raw `epok::screen_fade` store. Unlike
// the fog range every input has one nearest valid value, so the setter clamps and
// reports nothing; the getter is the whole observation.
static void scene_screen_fade_clamps_and_round_trips() {
    epok::screen_fade=0;
    assert(epok::SceneLibrary::screen_fade()==0);

    // Every representable amount round-trips exactly, including both ends and
    // the 255 the renderer special-cases into an opaque rectangle.
    for(uint32_t amount=0;amount<=255;++amount){
        epok::SceneLibrary::set_screen_fade(amount);
        assert(epok::screen_fade==amount);
        assert(epok::SceneLibrary::screen_fade()==amount);
    }

    // Above the range the setter clamps to opaque black rather than refusing,
    // exactly as a transition's loading color clamps its channels.
    for(uint32_t over:{uint32_t(256),uint32_t(300),uint32_t(4096),uint32_t(0xffffffffu)}){
        epok::SceneLibrary::set_screen_fade(over);
        assert(epok::screen_fade==255&&epok::SceneLibrary::screen_fade()==255);
    }
    // The low end is unsigned, so zero is the only floor there is to reach.
    epok::SceneLibrary::set_screen_fade(0);
    assert(epok::screen_fade==0&&epok::SceneLibrary::screen_fade()==0);

    // The getter reports what was authored, not what is drawn. The drawn amount
    // is the larger of this and the running transition's opacity, and that one
    // is already observable through `transition_snapshot`.
    epok::SceneLibrary::set_screen_fade(64);
    epok::transition.opacity=200;
    assert(epok::SceneLibrary::screen_fade()==64);
    assert(epok::SceneLibrary::transition_snapshot().opacity==200);
    epok::transition.opacity=0;
    assert(epok::SceneLibrary::screen_fade()==64);
    epok::screen_fade=0;
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

// The Vector3 tween is one clock plus two endpoints, so it has to agree with
// three scalar tweens driven by the same plan raw unit for raw unit: on the
// waiting leg, on the forward leg and on the mirrored ping-pong leg alike.
static void vector_tweens_track_scalar_tweens() {
    const epok::GameplayVector3 from{-2.0,0.5,7.0},to{6.0,-3.25,7.0};
    const epok::Fixed starts[3]={from.x,from.y,from.z},ends[3]={to.x,to.y,to.z};
    for(auto easing:{epok::Ease::Linear,epok::Ease::InOutQuint,epok::Ease::OutCirc,epok::Ease::SmoothStep}){
        auto vector=epok::UtilityVectorLibrary::vector_tween_schedule(from,to,2.0,easing,0.5,epok::TweenLoop::PingPong,2);
        epok::GameplayTweenState axes[3];
        for(unsigned axis=0;axis<3;++axis)axes[axis]=epok::UtilityLibrary::tween_schedule(starts[axis],ends[axis],2.0,easing,0.5,epok::TweenLoop::PingPong,2);
        // 0.5 of wait plus two 2.0 legs is exactly twelve steps of 0.375.
        for(int step=0;step<12;++step){
            const auto advanced=epok::UtilityVectorLibrary::vector_tween_advance(vector,0.375);
            vector=advanced.state;
            const epok::Fixed components[3]={advanced.value.x,advanced.value.y,advanced.value.z};
            for(unsigned axis=0;axis<3;++axis){
                const auto scalar=epok::UtilityLibrary::tween_advance(axes[axis],0.375);
                axes[axis]=scalar.state;
                assert(components[axis].raw()==scalar.value.raw());
                assert(advanced.completed==scalar.completed);
                assert(vector.timing.running==scalar.state.running);
                assert(vector.timing.elapsed.raw()==scalar.state.elapsed.raw());
                assert(vector.timing.reversed==scalar.state.reversed);
            }
            // The shared clock is an ordinary 0..1 tween, so its own value is the
            // interpolation parameter the three components were lerped with.
            const auto alpha=epok::UtilityLibrary::tween_value(vector.timing);
            for(unsigned axis=0;axis<3;++axis)assert(epok::lerp(starts[axis],ends[axis],alpha).raw()==components[axis].raw());
            assert(epok::UtilityVectorLibrary::vector_tween_value(vector).y.raw()==components[1].raw());
        }
        // Two mirrored legs end exactly back at the start, on every component.
        assert(!vector.timing.running&&vector.timing.completion_pending);
        const auto ended=epok::UtilityVectorLibrary::vector_tween_value(vector);
        assert(ended.x.raw()==from.x.raw()&&ended.y.raw()==from.y.raw()&&ended.z.raw()==from.z.raw());
    }
    // The short form is the same plan without a wait or a loop, and cancelling
    // keeps the endpoints and the reached value while clearing the playback.
    auto plain=epok::UtilityVectorLibrary::vector_tween_start(from,to,1.0,epok::Ease::OutCubic);
    assert(plain.timing.delay.raw()==0&&plain.timing.loop==epok::TweenLoop::None&&plain.timing.cycles_remaining==1);
    const auto quarter=epok::UtilityVectorLibrary::vector_tween_advance(plain,0.25);
    const auto cancelled=epok::UtilityVectorLibrary::vector_tween_cancel(quarter.state);
    assert(!cancelled.timing.running&&!cancelled.timing.completion_pending);
    const auto held=epok::UtilityVectorLibrary::vector_tween_value(cancelled);
    assert(held.x.raw()==quarter.value.x.raw()&&held.z.raw()==from.z.raw());
    assert(epok::UtilityVectorLibrary::vector_tween_advance(cancelled,9.0).value.x.raw()==held.x.raw());
}

// Three rooms in a row along X with corridors between them: the spawn, a locked room
// on a branch path one level up, and the goal holding the key.
static void dungeon_queries() {
    using epok::DungeonLibrary;
    namespace dungeon=epok::dungeon;
    dungeon::reset({});
    assert(DungeonLibrary::room_count()==0&&DungeonLibrary::spawn_room()==-1&&DungeonLibrary::goal_room()==-1);
    assert(!DungeonLibrary::room(0).valid&&DungeonLibrary::room_at({1.0,0.0,1.0})==-1);
    assert(DungeonLibrary::next_room(0,1)==-1&&DungeonLibrary::distance(0,0)==-1&&!DungeonLibrary::item(0).valid);

    constexpr int32_t u=4096;
    static constexpr dungeon::RoomData rooms[]={
        {{0,0,0},{4*u,6*u,4*u},{2*u,0,2*u},0,0,0,0,dungeon::spawn_room,0,2,1,1},
        {{8*u,0,0},{12*u,6*u,4*u},{10*u,0,2*u},1,1,1,1,dungeon::locked_room,1,1,2,0},
        {{16*u,0,0},{20*u,6*u,4*u},{18*u,0,2*u},0,0,3,1,dungeon::goal_room|dungeon::key_room,2,0,1,0},
    };
    static constexpr uint8_t neighbours[]={1,0,2,1};
    static constexpr dungeon::PointData spots[]={{{u,0,u},0,0}};
    static constexpr dungeon::PointData items[]={{{18*u,0,2*u},2,1}};
    static constexpr dungeon::PointData locks[]={{{8*u,0,2*u},1,1}};
    dungeon::reset({rooms,3,neighbours,spots,items,1,locks,1,0,2,1207u,false});

    assert(DungeonLibrary::room_count()==3&&DungeonLibrary::spawn_room()==0&&DungeonLibrary::goal_room()==2);
    // Rooms are found on the ground plane whatever the height; corridors are no room.
    assert(DungeonLibrary::room_at({2.0,0.0,2.0})==0&&DungeonLibrary::room_at({10.0,3.0,2.0})==1);
    assert(DungeonLibrary::room_at({6.0,0.0,2.0})==-1&&DungeonLibrary::room_at({2.0,0.0,9.0})==-1);
    assert(DungeonLibrary::room_of(epok::ObjectId{})==-1);

    const auto locked=DungeonLibrary::room(1);
    assert(locked.valid&&locked.index==1&&locked.level==1&&locked.path==1&&locked.locked&&!locked.spawn);
    assert(locked.depth==1&&locked.steps_to_goal==1&&locked.neighbours==2&&locked.spots==0);
    assert(locked.center.x.raw()==10*u&&locked.minimum.x.raw()==8*u&&locked.maximum.y.raw()==6*u);
    const auto goal=DungeonLibrary::room(2);
    assert(goal.goal&&goal.key&&!goal.item&&!DungeonLibrary::room(3).valid&&!DungeonLibrary::room(-1).valid);

    assert(DungeonLibrary::neighbour(1,0)==0&&DungeonLibrary::neighbour(1,1)==2);
    assert(DungeonLibrary::neighbour(1,2)==-1&&DungeonLibrary::neighbour(7,0)==-1);
    // Ways through the links, both directions, and to the same room.
    assert(DungeonLibrary::next_room(0,2)==1&&DungeonLibrary::next_room(2,0)==1&&DungeonLibrary::next_room(1,2)==2);
    assert(DungeonLibrary::next_room(1,1)==1&&DungeonLibrary::next_room(0,9)==-1);
    assert(DungeonLibrary::distance(0,2)==2&&DungeonLibrary::distance(2,1)==1&&DungeonLibrary::distance(0,0)==0);

    assert(DungeonLibrary::item_count()==1&&DungeonLibrary::lock_count()==1);
    const auto key=DungeonLibrary::item(0);
    assert(key.valid&&key.room==2&&key.lock==1&&key.position.x.raw()==18*u&&!DungeonLibrary::item(1).valid);
    const auto door=DungeonLibrary::lock(0);
    assert(door.valid&&door.room==1&&door.lock==1&&!DungeonLibrary::lock(1).valid);
    const auto spot=DungeonLibrary::spot(0,0);
    assert(spot.valid&&spot.room==0&&spot.position.x.raw()==u&&!DungeonLibrary::spot(0,1).valid&&!DungeonLibrary::spot(1,0).valid);

    // The gameplay stream is the dungeon's: same inputs, same number, every run.
    assert(DungeonLibrary::seed()==1207u);
    const uint32_t roll=DungeonLibrary::random(1,2,10);
    assert(roll<10&&roll==DungeonLibrary::random(1,2,10));
    assert(roll==dungeon::Random::stream(1207u^(1u*0x85ebca6bu),2).below(10));
    assert(DungeonLibrary::random(1,2,0)==0);

    // The minimap: three rooms of a 1-unit grid from x 0 and z 0, and two corridors
    // between them; walking explores rectangles, entering a room the whole room.
    static constexpr dungeon::MapRect rects[]={
        {0,0,4,2,0,{-1,-1},0},{0,2,4,2,0,{-1,-1},0},
        {8,0,4,4,1,{-1,-1},0},{16,0,4,4,2,{-1,-1},0},
        {4,1,4,1,-1,{0,1},0},{12,1,4,1,-1,{1,2},0},
    };
    static uint32_t explored[1];
    dungeon::Minimap minimap{};
    minimap.rects=rects;minimap.count=6;minimap.columns=20;minimap.rows=4;
    minimap.corner[0]=0;minimap.corner[1]=0;minimap.step[0]=u;minimap.step[1]=u;
    minimap.explored=explored;minimap.flags=dungeon::minimap_visible;
    explored[0]=~0u;
    dungeon::reset_minimap(minimap);
    assert(explored[0]==0&&!DungeonLibrary::room_explored(0)&&DungeonLibrary::minimap_shown());
    int32_t cx=0,cy=0;
    const int32_t corridor[3]={5*u+u/2,0,u+u/2};
    assert(dungeon::map_cell(corridor,cx,cy)&&cx==5&&cy==1&&dungeon::rect_at(cx,cy)==4);
    const int32_t outside[3]={-u/2,0,0};
    assert(!dungeon::map_cell(outside,cx,cy)&&cx==-1);
    dungeon::explore_at(corridor);
    assert(explored[0]==1u<<4&&!DungeonLibrary::room_explored(0)&&!DungeonLibrary::room_explored(1));
    const int32_t first_room[3]={u,0,3*u};
    dungeon::explore_at(first_room);
    assert(DungeonLibrary::room_explored(0)&&explored[0]==(1u<<4|1u<<1|1u<<0));
    DungeonLibrary::explore_room(2);
    assert(DungeonLibrary::room_explored(2)&&(explored[0]>>3&1)&&!(explored[0]>>2&1));
    DungeonLibrary::forget_explored();
    assert(explored[0]==0&&!DungeonLibrary::room_explored(2));
    DungeonLibrary::explore_everything();
    assert(explored[0]==0x3fu&&DungeonLibrary::room_explored(1));
    DungeonLibrary::show_full_map(true);
    assert(DungeonLibrary::full_map_shown()&&DungeonLibrary::minimap_shown());
    DungeonLibrary::show_full_map(false);DungeonLibrary::show_minimap(false);
    assert(!DungeonLibrary::full_map_shown()&&!DungeonLibrary::minimap_shown());
    DungeonLibrary::explore_room(64);DungeonLibrary::explore_room(-1);
    dungeon::reset_minimap({});
    assert(!DungeonLibrary::minimap_shown()&&!DungeonLibrary::room_explored(0));

    // In 2D rooms lie on x and y (pixels) and z is ignored.
    dungeon::reset({rooms,3,neighbours,spots,items,1,locks,1,0,2,1207u,true});
    assert(DungeonLibrary::room_at({10.0,2.0,99.0})==1&&DungeonLibrary::room_at({10.0,9.0,2.0})==-1);
    dungeon::reset({});
    assert(DungeonLibrary::room_count()==0);
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
    visible_packet_ownership();
    integer_square_roots();
    scene_fog_range();
    scene_screen_fade_clamps_and_round_trips();
    persistent_utilities();
    vector_tweens_track_scalar_tweens();
    dungeon_queries();
    std::puts("Gameplay libraries: typed records, input/time, math, bounded payload, fog, screen fade, tweens, focus and dungeon queries pass.");
}
