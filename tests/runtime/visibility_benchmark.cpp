// Synthetic CPU benchmark; host timings are not PlayStation frame-rate data.
#include "visibility.hpp"
#include "frustum.hpp"
#include <array>
#include <vector>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>

using Plane=std::array<int32_t,4>;
struct Frame:std::array<Plane,5> {int32_t rows[3][3],translation[3];};
struct Box {int32_t center[3],extent[3];};
constexpr size_t chunks=168,words=(chunks+31)/32;
struct Fixture {
    std::array<Box,chunks> boxes;
    std::vector<uint16_t> rows;
    std::vector<uint32_t> masks;
    uint32_t combined[words]={},bounds_valid[words]={},bounds_result[words]={};
    uint32_t basis_valid[words]={};
    epok::ChunkBasisBounds basis_bounds[chunks]{};
    epok::ChunkVisibilityCache cache{combined,bounds_valid,bounds_result,basis_bounds,basis_valid};
    epok::ChunkVisibility grid;
    Fixture():rows(54*32),masks(54*32*words),grid{rows.data(),masks.data(),uint16_t(chunks),uint16_t(words),-16*16384,16384,32,&cache} {
        for(size_t i=0;i<chunks;++i)boxes[i]={{int32_t(i%14)*16384-26*4096,int32_t(i%3)*4096,int32_t(i/14)*16384-20*4096},{4096,4096,4096}};
        for(int direction=0;direction<54;++direction) {
            const int dominant=direction/18,sign=direction/9%2?2:-2;
            int digits[3]={1,1,1},code=direction%9;
            for(int a=2;a>=0;--a)if(a!=dominant){digits[a]=code%3;code/=3;}
            for(int slab=0;slab<32;++slab) {
                const int row=direction*32+slab;rows[row]=uint16_t(row);
                for(size_t i=0;i<chunks;++i) {
                    int64_t sum=2*int64_t(grid.offset_min+(slab+1)*grid.offset_step);
                    for(int a=0;a<3;++a) {
                        int low=digits[a]==0?-2:digits[a]==1?-1:1,high=digits[a]==0?-1:digits[a]==1?1:2;
                        if(a==dominant)low=high=sign;
                        const int64_t left=int64_t(boxes[i].center[a])-boxes[i].extent[a]-32,right=int64_t(boxes[i].center[a])+boxes[i].extent[a]+32;
                        sum+=std::max({low*left,low*right,high*left,high*right});
                    }
                    if(sum>=0)masks[size_t(row)*words+i/32]|=uint32_t(1)<<(i%32);
                }
            }
        }
    }
};
Frame frame(double yaw,double camera_x,double camera_z) {
    const double pitch=.55,focal=2.1445,c=std::cos(yaw),s=std::sin(yaw),cp=std::cos(pitch),sp=std::sin(pitch);
    double rows[3][4]={{focal*c,0,-focal*s,0},{focal*sp*s,focal*cp,focal*sp*c,0},{cp*s,-sp,cp*c,0}};
    for(auto& row:rows)row[3]=-row[0]*camera_x-row[1]*9-row[2]*camera_z;
    Frame result;
    for(int r=0;r<3;++r) {
        for(int c=0;c<3;++c)result.rows[r][c]=int32_t(std::llround(rows[r][c]*4096));
        result.translation[r]=int32_t(std::llround(rows[r][3]*4096));
    }
    for(int k=0;k<4;++k) {
        const int32_t x=k<3?result.rows[0][k]:result.translation[0];
        const int32_t y=k<3?result.rows[1][k]:result.translation[1];
        const int32_t z=k<3?result.rows[2][k]:result.translation[2];
        result[0][k]=z;
        result[1][k]=z+x;result[2][k]=z-x;
        result[3][k]=3*z+4*y;result[4][k]=3*z-4*y;
    }
    return result;
}
bool bounds_accept(const int32_t center[3],const int32_t extent[3]) {
    const int32_t far_z=center[2]+extent[2];
    return !(far_z<1024 || center[2]-extent[2]>=128*4096 ||
             center[0]-extent[0]>far_z || center[0]+extent[0]<-far_z ||
             (center[1]-extent[1])*4>far_z*3 || (center[1]+extent[1])*4<-far_z*3);
}
struct Result {double micros;uint64_t tested,accepted,basis_hits;};
Result run(Fixture& f,const std::vector<Frame>& frames,bool enabled) {
    f.cache=epok::ChunkVisibilityCache{f.combined,f.bounds_valid,f.bounds_result,f.basis_bounds,f.basis_valid};
    uint64_t tested=0,accepted=0,basis_hits=0;
    const auto begin=std::chrono::steady_clock::now();
    constexpr unsigned count=60000;
    for(unsigned tick=0;tick<count;++tick) {
        const auto& planes=frames[tick%frames.size()];
        int32_t absolute[3][3],peak=0;
        for(int r=0;r<3;++r)for(int c=0;c<3;++c) {
            absolute[r][c]=std::abs(planes.rows[r][c]);peak=std::max(peak,absolute[r][c]);
        }
        const bool narrow_view=peak<16384;
        const bool isolated_x=epok::chunk_bounds_isolated_x(planes.rows);
        epok::ChunkVisibilityMask mask;
        epok::ChunkVisibilityQuery query(enabled?&f.grid:nullptr);
        if(enabled) {
            if(!query.reuse_matrix(planes.rows,planes.translation,true))
                for(const auto& p:planes)query.add_plane(p[0],p[1],p[2],p[3]);
            mask=query.merged();
        }
        const bool bounds_cache=query.bounds_cache_active();
        const bool basis_cache=query.basis_cache_active();
        for(size_t i=0;i<chunks;++i) {
            if(!mask.candidate(i))continue;
            const int cached=bounds_cache?query.cached_bounds(i):-1;
            if(cached>=0){accepted+=cached;continue;}
            ++tested;
            int32_t center[3],extent[3];
            const auto* basis=basis_cache?query.cached_basis_bounds(i):nullptr;
            if(basis) {
                ++basis_hits;
                for(int r=0;r<3;++r){center[r]=planes.translation[r]+basis->center[r];extent[r]=basis->extent[r];}
            } else {
                const auto& box=f.boxes[i];
                bool narrow=narrow_view;
                for(int r=0;r<3;++r)if(box.center[r]>=131072 || box.center[r]<=-131072 || box.extent[r]>=131072)narrow=false;
                if(narrow) {
                    if(!basis_cache)epok::narrow_chunk_bounds(planes.rows,absolute,planes.translation,box.center,box.extent,center,extent,isolated_x);
                    else {
                        const int32_t zero[3]={};
                        epok::narrow_chunk_bounds(planes.rows,absolute,zero,box.center,box.extent,center,extent,isolated_x);
                        query.remember_basis_bounds(i,center,extent);
                        for(int r=0;r<3;++r)center[r]+=planes.translation[r];
                    }
                } else {
                    for(int r=0;r<3;++r) {
                        int64_t sum=0,span=0;
                        for(int c=0;c<3;++c){sum+=int64_t(planes.rows[r][c])*box.center[c];span+=int64_t(absolute[r][c])*box.extent[c];}
                        center[r]=planes.translation[r]+int32_t(sum>>12);extent[r]=int32_t(span>>12)+1;
                    }
                }
            }
            const bool visible=bounds_accept(center,extent);accepted+=visible;
            if(bounds_cache)query.remember_bounds(i,visible);
        }
    }
    return {std::chrono::duration<double,std::micro>(std::chrono::steady_clock::now()-begin).count()/count,tested,accepted,basis_hits};
}
int main() {
    Fixture fixture;
    std::vector<Frame> stationary{frame(0,0,-14)},translation,moving,instances{frame(0,-7,-14),frame(.2,7,-14)};
    for(unsigned i=0;i<256;++i){const double a=i*6.283185307179586/256;translation.push_back(frame(0,5*std::sin(a),-14+6*std::cos(a)));moving.push_back(frame(.75*std::sin(a),5*std::sin(a),-14+6*std::cos(a)));}
    const std::vector<Frame>* scenarios[4]={&stationary,&translation,&moving,&instances};
    const char* names[4]={"stationary_camera","moving_camera_translation","moving_camera_rotation","shared_geometry_instances"};
    std::puts("{\"environment\":\"optimized host CPU; not PSX FPS\",\"policy\":\"defer masks until the exact matrix repeats, as in the renderer\",\"chunks\":168,\"frames_per_run\":60000,\"scenarios\":[");
    for(int s=0;s<4;++s) {
        std::array<double,3> a,b;Result baseline{},visibility{};
        for(int repeat=0;repeat<3;++repeat){baseline=run(fixture,*scenarios[s],false);visibility=run(fixture,*scenarios[s],true);a[repeat]=baseline.micros;b[repeat]=visibility.micros;if(baseline.accepted!=visibility.accepted)return EXIT_FAILURE;}
        std::sort(a.begin(),a.end());std::sort(b.begin(),b.end());
        std::printf("{\"name\":\"%s\",\"baseline_us\":%.3f,\"visibility_us\":%.3f,\"ratio\":%.4f,\"bounds_skipped_percent\":%.2f,\"basis_reused_percent_of_bounds_tests\":%.2f,\"same_accepted_chunks\":true}%s\n",names[s],a[1],b[1],b[1]/a[1],100.*(1.-double(visibility.tested)/double(baseline.tested)),visibility.tested?100.*double(visibility.basis_hits)/double(visibility.tested):0.,s==3?"":",");
    }
    std::puts("]}");
}
