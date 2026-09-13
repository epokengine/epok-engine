#include <cassert>
#include <cstdio>
#include <vector>
#include <array>
#include "visibility.hpp"

int main() {
    // Independent reference mask construction for 64 point-sized chunks spread
    // through a cube. Exercise the production lookup at bucket boundaries and
    // arbitrary interior directions/offsets, including parent-scale-like planes.
    std::array<std::array<int32_t,3>,64> points;
    for(int i=0;i<64;++i)points[i]={{((i/16)%4-2)*16384,((i/4)%4-2)*16384,(i%4-2)*16384}};
    std::vector<uint16_t> rows(54*32);
    std::vector<uint32_t> masks(54*32*2);
    constexpr int32_t offset_min=-16*8192,step=8192;
    for(int direction=0;direction<54;++direction) {
        const int dominant=direction/18,sign=(direction/9%2)?2:-2;
        int digits[3]={1,1,1},code=direction%9;
        for(int axis=2;axis>=0;--axis)if(axis!=dominant){digits[axis]=code%3;code/=3;}
        for(int slab=0;slab<32;++slab) {
            const int row=direction*32+slab;rows[row]=uint16_t(row);
            for(int i=0;i<64;++i) {
                int64_t support=2*int64_t(offset_min+(slab+1)*step);
                for(int axis=0;axis<3;++axis) {
                    int low=digits[axis]==0?-2:digits[axis]==1?-1:1;
                    int high=digits[axis]==0?-1:digits[axis]==1?1:2;
                    if(axis==dominant)low=high=sign;
                    const auto p=points[i][axis];
                    support+=int64_t(p>=0?high:low)*p;
                }
                if(support>=0)masks[row*2+i/32]|=uint32_t(1)<<(i%32);
            }
        }
    }
    epok::ChunkVisibility grid{rows.data(),masks.data(),64,2,offset_min,step,32};
    uint32_t combined[2]={},bounds_valid[2]={},bounds_result[2]={};
    epok::ChunkVisibilityCache cache{combined,bounds_valid,bounds_result};
    auto cached_grid=grid;cached_grid.cache=&cache;
    uint32_t seed=0x1234abcdu;size_t rejected=0;
    auto random=[&](){seed=1664525u*seed+1013904223u;return seed;};
    for(int trial=0;trial<30000;++trial) {
        int32_t plane[4];
        for(int axis=0;axis<3;++axis)plane[axis]=int32_t(random()%131073)-65536;
        plane[3]=int32_t(random()%262145)-131072;
        if(trial<1000) {plane[0]=4096;plane[1]=(trial%5-2)*2048;plane[2]=0;plane[3]=(trial/5%32-16)*step-256;}
        epok::ChunkVisibilityQuery query(&grid);
        query.add_plane(plane[0],plane[1],plane[2],plane[3]);
        epok::ChunkVisibilityQuery cached(&cached_grid);
        cached.add_plane(plane[0],plane[1],plane[2],plane[3]);
        for(size_t i=0;i<points.size();++i)assert(query.candidate(i)==cached.candidate(i));
        // Exact repeated inputs reuse masks. A second instance sharing the
        // descriptor safely replaces this cache on its next changed transform.
        epok::ChunkVisibilityQuery repeated(&cached_grid);
        repeated.add_plane(plane[0],plane[1],plane[2],plane[3]);
        for(size_t i=0;i<points.size();++i)assert(query.candidate(i)==repeated.candidate(i));
        for(size_t i=0;i<points.size();++i)if(!query.candidate(i)) {
            ++rejected;
            int64_t dot=int64_t(plane[3])*4096;
            for(int axis=0;axis<3;++axis)dot+=int64_t(plane[axis])*points[i][axis];
            assert(dot<0);
        }
        assert(query.candidate(64)); // changed or unsupported topology
    }
    assert(rejected>1000);
    for(int trial=0;trial<200;++trial) {
        epok::ChunkVisibilityQuery reference(&grid),cached(&cached_grid);
        for(int plane=0;plane<trial%7;++plane) {
            const int64_t x=int32_t(random()%65537)-32768,y=int32_t(random()%65537)-32768,z=int32_t(random()%65537)-32768,d=int32_t(random()%262145)-131072;
            reference.add_plane(x,y,z,d);cached.add_plane(x,y,z,d);
        }
        for(size_t i=0;i<64;++i)assert(reference.candidate(i)==cached.candidate(i));
    }
    const int32_t matrix[3][3]={{4096,0,0},{0,4096,0},{0,0,4096}};
    int32_t translation[3]={-8192,0,0};
    {
        epok::ChunkVisibilityQuery first(&cached_grid);
        assert(!first.reuse_matrix(matrix,translation));
        assert(!first.bounds_cache_active());
        first.add_plane(4096,0,0,translation[0]);
        auto mask=first.merged();assert(mask.candidate(64));
        epok::ChunkVisibilityQuery second(&cached_grid);
        assert(second.reuse_matrix(matrix,translation));
        assert(second.bounds_cache_active());
        assert(second.cached_bounds(0)==-1);
        second.remember_bounds(0,true);second.remember_bounds(63,false);
        assert(second.cached_bounds(0)==1 && second.cached_bounds(63)==0);
        assert(second.cached_bounds(1)==-1 && second.cached_bounds(64)==-1);
        second.remember_bounds(64,true);
        const auto reused=second.merged();
        for(size_t i=0;i<64;++i)assert(mask.candidate(i)==reused.candidate(i));
        epok::ChunkVisibilityQuery third(&cached_grid);
        assert(third.reuse_matrix(matrix,translation));
        assert(third.cached_bounds(0)==1 && third.cached_bounds(63)==0);
    }
    translation[0]=8192;
    {
        // An abandoned changed-matrix query must not validate stale masks.
        epok::ChunkVisibilityQuery abandoned(&cached_grid);
        assert(!abandoned.reuse_matrix(matrix,translation));
        assert(!abandoned.bounds_cache_active());
        assert(abandoned.cached_bounds(0)==-1);
        abandoned.add_plane(4096,0,0,translation[0]);
    }
    {
        epok::ChunkVisibilityQuery changed(&cached_grid),reference(&grid);
        assert(!changed.reuse_matrix(matrix,translation));
        changed.add_plane(4096,0,0,translation[0]);reference.add_plane(4096,0,0,translation[0]);
        auto mask=changed.merged();
        for(size_t i=0;i<64;++i)assert(mask.candidate(i)==reference.candidate(i));
        epok::ChunkVisibilityQuery steady(&cached_grid);
        assert(steady.reuse_matrix(matrix,translation));
        assert(steady.cached_bounds(0)==-1 && steady.cached_bounds(63)==-1);
    }
    {
        // Direct plane updates invalidate a previous matrix cache key.
        epok::ChunkVisibilityQuery direct(&cached_grid);
        direct.add_plane(-4096,0,0,0);direct.merged();
        epok::ChunkVisibilityQuery next(&cached_grid);
        assert(!next.reuse_matrix(matrix,translation));
    }
    {
        // Renderer policy: moving matrices only record an observation. Never
        // expose the last stable mask or mark that observation as mask-valid.
        cache=epok::ChunkVisibilityCache{combined,bounds_valid,bounds_result};
        for(int frame=0;frame<100;++frame) {
            translation[0]=frame*128-8192;
            epok::ChunkVisibilityQuery moving(&cached_grid);
            assert(moving.reuse_matrix(matrix,translation,true));
            assert(!moving.merged().words && !cache.matrix_valid);
            assert(cache.matrix_observed && !moving.bounds_cache_active());
            assert(moving.cached_bounds(0)==-1);
            for(size_t i=0;i<=64;++i)assert(moving.candidate(i));
        }
        auto build_stable=[&]() {
            epok::ChunkVisibilityQuery stable(&cached_grid),reference(&grid);
            assert(!stable.reuse_matrix(matrix,translation,true));
            assert(!stable.bounds_cache_active());
            stable.add_plane(4096,0,0,translation[0]);
            reference.add_plane(4096,0,0,translation[0]);
            const auto mask=stable.merged();
            assert(cache.matrix_valid);
            for(size_t i=0;i<=64;++i)assert(mask.candidate(i)==reference.candidate(i));
        };
        build_stable();
        {
            epok::ChunkVisibilityQuery repeat(&cached_grid);
            assert(repeat.reuse_matrix(matrix,translation,true));
            assert(repeat.bounds_cache_active());
            repeat.remember_bounds(0,false);
        }
        {
            epok::ChunkVisibilityQuery repeat(&cached_grid);
            assert(repeat.reuse_matrix(matrix,translation,true));
            assert(repeat.cached_bounds(0)==0);
        }
        // Rotation/scale alone must invalidate even when translation matches.
        int32_t rotated[3][3]={{0,0,4096},{0,4096,0},{-4096,0,0}};
        {
            epok::ChunkVisibilityQuery rotation(&cached_grid);
            assert(rotation.reuse_matrix(rotated,translation,true));
            assert(!rotation.merged().words && !rotation.bounds_cache_active());
            assert(!bounds_valid[0] && !bounds_valid[1]);
        }
        // Interleaved shared instances safely bypass every changing key.
        for(int instance=0;instance<40;++instance) {
            translation[0]=(instance%2)?8192:-8192;
            epok::ChunkVisibilityQuery shared(&cached_grid);
            assert(shared.reuse_matrix(matrix,translation,true));
            assert(!shared.merged().words && !cache.matrix_valid);
        }
        // Abandoning the observed key or its first stable build cannot publish
        // either the previous mask or a partially rebuilt plane collection.
        translation[0]=0;
        {
            epok::ChunkVisibilityQuery abandoned_observation(&cached_grid);
            assert(abandoned_observation.reuse_matrix(matrix,translation,true));
        }
        {
            epok::ChunkVisibilityQuery abandoned_build(&cached_grid);
            assert(!abandoned_build.reuse_matrix(matrix,translation,true));
            abandoned_build.add_plane(-4096,0,0,translation[0]);
        }
        assert(!cache.matrix_valid);
        build_stable();
        epok::ChunkVisibilityQuery no_cache(&grid),no_grid(nullptr);
        assert(no_cache.reuse_matrix(matrix,translation,true));
        assert(no_grid.reuse_matrix(matrix,translation,true));
        assert(!no_cache.merged().words && !no_grid.merged().words);
    }
    {
        epok::ChunkBasisBounds basis[64]{};
        uint32_t valid[2]={};
        cache=epok::ChunkVisibilityCache{combined,bounds_valid,bounds_result,basis,valid};
        const int32_t center[3]={-1,8192,-4096},extent[3]={3,4099,7};
        translation[0]=0;
        epok::ChunkVisibilityQuery first(&cached_grid);
        first.reuse_matrix(matrix,translation,true);
        first.remember_basis_bounds(0,center,extent);
        assert(!first.basis_cache_active() && !valid[0] && !valid[1]);
        epok::ChunkVisibilityQuery repeated(&cached_grid);
        repeated.reuse_matrix(matrix,translation,true);
        assert(repeated.basis_cache_active());
        repeated.remember_basis_bounds(0,center,extent);
        repeated.remember_basis_bounds(63,center,extent);
        assert(repeated.cached_basis_bounds(0) && repeated.cached_basis_bounds(63));
        assert(!repeated.cached_basis_bounds(1) && !repeated.cached_basis_bounds(64));
        // A shared instance can retain products only for the same exact basis.
        for(int instance=0;instance<40;++instance) {
            translation[0]=(instance%2)?8192:-8192;
            epok::ChunkVisibilityQuery shared(&cached_grid);
            shared.reuse_matrix(matrix,translation,true);
            assert(shared.cached_basis_bounds(0) && shared.cached_basis_bounds(63));
        }
        int32_t changed[3][3]={{4096,0,0},{0,4096,0},{0,0,4096}};
        for(int r=0;r<3;++r)for(int c=0;c<3;++c) {
            ++changed[r][c];
            epok::ChunkVisibilityQuery rotation_scale_fov(&cached_grid);
            rotation_scale_fov.reuse_matrix(changed,translation,true);
            assert(!rotation_scale_fov.cached_basis_bounds(0));
            assert(!rotation_scale_fov.cached_basis_bounds(63));
            rotation_scale_fov.remember_basis_bounds(0,center,extent);
            assert(!rotation_scale_fov.basis_cache_active() && !valid[0] && !valid[1]);
        }
        {
            epok::ChunkVisibilityQuery repeated_basis(&cached_grid);
            translation[0]+=128; // Repeating the basis alone enables recording.
            repeated_basis.reuse_matrix(changed,translation,true);
            assert(repeated_basis.basis_cache_active());
            repeated_basis.remember_basis_bounds(0,center,extent);
            assert(repeated_basis.cached_basis_bounds(0));
        }
        for(int instance=0;instance<40;++instance) {
            epok::ChunkVisibilityQuery alternating(&cached_grid);
            alternating.reuse_matrix(instance%2?changed:matrix,translation,true);
            assert(!alternating.basis_cache_active());
            alternating.remember_basis_bounds(0,center,extent);
            assert(!alternating.cached_basis_bounds(0) && !valid[0] && !valid[1]);
        }
    }
    epok::ChunkVisibilityQuery fallback(&grid);
    fallback.add_plane(0,0,0,0);
    fallback.add_plane(INT64_MAX,0,0,0);
    fallback.add_plane(4096,0,0,INT32_MAX);
    fallback.add_plane(4096,0,0,INT32_MIN);
    fallback.add_plane(4096,0,0,INT64_MAX);
    fallback.add_plane(4096,0,0,INT64_MIN);
    for(int i=0;i<64;++i)assert(fallback.candidate(i));
    epok::ChunkVisibilityQuery absent(nullptr);assert(absent.candidate(0));
    std::puts("Conservative visibility: 30,000 planes, bucket edges, affine scales and fallback passed.");
}
