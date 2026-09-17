#include <cassert>
#include <climits>
#include <cstdio>
#include "frustum.hpp"
#include "visibility.hpp"
struct Point{struct{int x,y;}screen;bool visible=true;};
static void verify_narrow_chunk_bounds() {
    {
        // Preserve sum-before-translation association as well as shifts: adding
        // translation to the first term early would overflow at these edges.
        const int32_t rows[3][3]={{0,0,0},{0,4096,-4096},{0,-4096,4096}};
        const int32_t absolute[3][3]={{0,0,0},{0,4096,4096},{0,4096,4096}};
        const int32_t translation[3]={0,INT32_MAX,INT32_MIN},vector[3]={0,1,2},extent[3]={};
        int32_t center[3],span[3];
        epok::narrow_chunk_bounds(rows,absolute,translation,vector,extent,center,span,true);
        assert(center[1]==INT32_MAX-1 && center[2]==INT32_MIN+1);
    }
    uint32_t seed=0x981723abu;
    uint32_t combined=0,basis_valid=0;
    epok::ChunkBasisBounds basis[2]{};
    epok::ChunkVisibilityCache cache{&combined,nullptr,nullptr,basis,&basis_valid};
    epok::ChunkVisibility grid{nullptr,nullptr,2,1,0,1,1,&cache};
    auto random=[&](){seed=1664525u*seed+1013904223u;return seed;};
    for(unsigned trial=0;trial<100000;++trial) {
        int32_t rows[3][3],absolute[3][3],translation[3],vector[3],extent[3];
        for(int r=0;r<3;++r)for(int c=0;c<3;++c)rows[r][c]=int32_t(random()%32767)-16383;
        const bool isolated=(trial&1)==0;
        if(isolated)rows[0][1]=rows[0][2]=rows[1][0]=rows[2][0]=0;
        // Include exact upper/lower narrow limits, zero scales, and tiny signed
        // coefficients where summing before shifting would round differently.
        if(trial%11==0)rows[0][0]=trial&2?-16383:16383;
        if(trial%13==0)rows[2][2]=0;
        if(trial%17==0)rows[1][1]=-1;
        for(int r=0;r<3;++r) {
            translation[r]=int32_t(random()%1048577)-524288;
            vector[r]=int32_t(random()%262143)-131071;
            extent[r]=int32_t(random()%131072);
            for(int c=0;c<3;++c)absolute[r][c]=rows[r][c]<0?-rows[r][c]:rows[r][c];
        }
        if(trial%7==0){vector[0]=-131071;vector[1]=131071;extent[2]=131071;}
        int32_t expected_center[3],expected_span[3],center[3],span[3];
        for(int r=0;r<3;++r) {
            int32_t sum=0,radius=0;
            for(int c=0;c<3;++c) {
                sum+=(rows[r][c]*vector[c])>>12;
                radius+=(absolute[r][c]*extent[c])>>12;
            }
            expected_center[r]=translation[r]+sum;expected_span[r]=radius+3;
        }
        const bool detected=epok::chunk_bounds_isolated_x(rows);
        assert(detected==isolated);
        epok::narrow_chunk_bounds(rows,absolute,translation,vector,extent,center,span,detected);
        for(int r=0;r<3;++r){assert(center[r]==expected_center[r]);assert(span[r]==expected_span[r]);}
        // The test changes chunk geometry on each trial, unlike immutable
        // exported metadata, so start a fresh cache before translating it.
        cache.matrix_observed=false;
        epok::ChunkVisibilityQuery first(&grid);
        first.reuse_matrix(rows,translation,true);
        assert(!first.cached_basis_bounds(0) && !first.cached_basis_bounds(1));
        const int32_t zero[3]={};
        epok::narrow_chunk_bounds(rows,absolute,zero,vector,extent,center,span,detected);
        first.remember_basis_bounds(0,center,span);
        assert(!first.basis_cache_active() && !basis_valid);
        epok::ChunkVisibilityQuery repeated(&grid);
        repeated.reuse_matrix(rows,translation,true);
        assert(repeated.basis_cache_active());
        repeated.remember_basis_bounds(0,center,span);
        assert(!repeated.cached_basis_bounds(1) && !repeated.cached_basis_bounds(2));
        repeated.remember_basis_bounds(2,center,span); // checked topology bound
        for(int movement=0;movement<4;++movement) {
            for(int r=0;r<3;++r)translation[r]+=int32_t(random()%8193)-4096;
            epok::ChunkVisibilityQuery moving(&grid);
            moving.reuse_matrix(rows,translation,true);
            const auto* stored=moving.cached_basis_bounds(0);assert(stored);
            epok::narrow_chunk_bounds(rows,absolute,translation,vector,extent,center,span,detected);
            for(int r=0;r<3;++r) {
                assert(translation[r]+stored->center[r]==center[r]);
                assert(stored->extent[r]==span[r]);
            }
        }
    }
}
int main(){
    verify_narrow_chunk_bounds();
    const int32_t inside[3]={0,0,8*4096},size[3]={4096,4096,4096};
    assert(epok::chunk_fully_inside(inside,size,160));
    const int32_t near[3]={0,0,1200},empty[3]={};
    assert(!epok::chunk_fully_inside(near,empty,160));
    const int32_t edge[3]={8*4096,0,8*4096},far[3]={0,0,128*4096};
    assert(!epok::chunk_fully_inside(edge,size,160));
    assert(!epok::chunk_fully_inside(far,empty,160));
    const int32_t extreme[3]={INT32_MIN,INT32_MAX,INT32_MAX},huge[3]={INT32_MAX,INT32_MAX,INT32_MAX};
    assert(!epok::chunk_fully_inside(extreme,huge,160));
    for(int x=-12;x<=12;++x)for(int y=-12;y<=12;++y)for(int z=1;z<24;++z){
        const int32_t center[3]={x*4096,y*4096,z*4096};
        if(!epok::chunk_fully_inside(center,size,160))continue;
        // Quantization perturbations up to the guard must never escape a plane.
        for(int corner=0;corner<8;++corner){
            int32_t p[3];for(int a=0;a<3;++a)p[a]=center[a]+((corner&(1<<a))?1:-1)*(size[a]+128);
            assert(epok::frustum_outcode(p[0],p[1],p[2])==0);
        }
    }
    using epok::frustum_outcode;
    assert(frustum_outcode(0,0,1024)==0);
    assert(frustum_outcode(0,0,1023)&1);
    assert(frustum_outcode(0,0,128*4096)&2);
    assert(frustum_outcode(-4096,0,4096)==0);
    assert(frustum_outcode(-4097,0,4096)==4);
    assert(frustum_outcode(4097,0,4096)==8);
    assert(frustum_outcode(0,-3072,4096)==0);
    assert(frustum_outcode(0,-3073,4096)==16);
    assert(frustum_outcode(0,3073,4096)==32);
    assert(frustum_outcode(INT32_MAX,INT32_MIN,4096)==(8|16));
    // A shared outside plane is sufficient for rejection, even when each
    // vertex violates other planes too. Different sides must reach clipping.
    assert((frustum_outcode(-5000,0,4096)&frustum_outcode(-7000,4000,4096)&frustum_outcode(-6000,-4000,4096))==4);
    assert(!(frustum_outcode(-5000,0,4096)&frustum_outcode(5000,0,4096)&frustum_outcode(0,4000,4096)));
    Point a{{-300,-100}},b{{723,411}},c{{20,20}};
    assert(epok::gpu_clip_safe(a,b,c)); // exact hardware span limits
    b.screen.x=724;assert(!epok::gpu_clip_safe(a,b,c));
    b.screen.x=723;b.screen.y=412;assert(!epok::gpu_clip_safe(a,b,c));
    b.screen.y=411;c.visible=false;assert(!epok::gpu_clip_safe(a,b,c));
    std::puts("Frustum outcodes, 100k exact narrow bounds, boundary rejection and GPU guard-band span tests passed.");
}
