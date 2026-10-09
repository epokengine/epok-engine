#include <array>
#include <cassert>
#include <cstdio>
#include <climits>
#include <type_traits>
#include "psyqo/fixed-point.hh"
// The test runner copies the actual production header next to register stubs.
// This checks the matrix upload, not emulated GTE instructions/rasterization.
#include "gte-matrix/gte_geometry.hpp"
using Q=psyqo::FixedPoint<12>;
using Matrix=epok::Affine<Q>;
using Camera=epok::CameraSpace<Q>;
static bool equal(const Matrix& a,const Matrix& b){
    for(int r=0;r<3;++r)for(int c=0;c<4;++c)
        if(a.values[r][c].raw()!=b.values[r][c].raw())return false;
    return true;
}
static auto registers(){
    std::array<uint32_t,unsigned(psyqo::GTE::Register::Count)> out;
    for(unsigned i=0;i<out.size();++i)out[i]=psyqo::GTE::registers[i];
    return out;
}
// Literal copy of the general 64-bit transform: the 32-bit fast path must
// agree with it on every input, including the range edges where it declines.
static void reference_point(const Camera& camera,const Q* in,Q* out){
    int64_t delta[3];bool narrow=true;
    for(int c=0;c<3;++c){
        delta[c]=int64_t(in[c].raw())-camera.origin[c].raw();
        narrow&=delta[c]>=INT32_MIN&&delta[c]<=INT32_MAX;
    }
    for(int r=0;r<3;++r){
        int64_t sum=0;
        for(int c=0;c<3;++c){
            const int32_t coefficient=camera.basis.values[r][c].raw();
            const int64_t product=narrow?int64_t(coefficient)*int32_t(delta[c]):int64_t(coefficient)*delta[c];
            sum+=product/Q::scale;
        }
        out[r]=Q(int32_t(sum<INT32_MIN?INT32_MIN:sum>INT32_MAX?INT32_MAX:sum),Q::RAW);
    }
}
static void camera_point_fast_path(){
    uint32_t random=7;
    const auto next=[&](){random=random*1664525u+1013904223u;return random;};
    const int32_t edges[]={0,1,-1,(1<<19)-1,-(1<<19)+1,1<<19,-(1<<19),INT32_MAX,INT32_MIN,4096,-4096,4097,-4097};
    for(unsigned sample=0;sample<400000;++sample){
        Camera camera;
        for(int r=0;r<3;++r)for(int c=0;c<3;++c){
            const uint32_t kind=next()%8;
            const int32_t value=kind<5?int32_t(next()%8193)-4096:kind<7?edges[next()%13]:int32_t(next());
            camera.basis.values[r][c]=Q(value,Q::RAW);
        }
        Q in[3],fast[3],slow[3];
        for(int c=0;c<3;++c){
            const int32_t origin=next()%4?int32_t(next())>>(next()%12):edges[next()%13];
            const uint32_t kind=next()%6;
            const int32_t delta=kind<3?int32_t(next()%(1<<21))-(1<<20):kind<5?edges[next()%13]:int32_t(next());
            camera.origin[c]=Q(origin,Q::RAW);
            in[c]=Q(int32_t(uint32_t(origin)+uint32_t(delta)),Q::RAW);
        }
        camera.point(in,fast);reference_point(camera,in,slow);
        for(int c=0;c<3;++c)assert(fast[c].raw()==slow[c].raw());
    }
    std::puts("Camera point: 400000 fast-path/64-bit agreements including range edges passed.");
}
int main(){
    camera_point_fast_path();
    // Width changes storage only, not the decoded Q12 coordinates or GTE Q8
    // inputs. Exercise offsets on both signs and an exact int16 boundary.
    const int16_t narrow[][3]={{-32768,32767,16},{17,-17,0},{0,0,1}};
    const int32_t same[][3]={{-32768,32767,16},{17,-17,0},{0,0,1}};
    const int32_t large[][3]={{-65536,131072,524272},{123456,-234567,345678},{0,0,-524288}};
    epok::MeshVertices small(narrow),wide(same),big(large);
    assert(small.stride==6&&wide.stride==12&&big.wide());
    for(unsigned i=0;i<3;++i)for(unsigned c=0;c<3;++c){
        assert(small[i][c]==wide[i][c]);
        assert((big+i)[0][c]==large[i][c]);
    }
    const int32_t zero[3]={},center[3]={},extent[3]={524272,524272,524272};
    assert(epok::gte_vertex_bounds_fit(zero,center,extent));
    const int32_t overflow[3]={524288,0,0};
    assert(!epok::gte_vertex_bounds_fit(zero,center,overflow));
    const int32_t shifted[3]={1,0,0};
    assert(!epok::gte_vertex_bounds_fit(shifted,center,extent));
    int32_t camera_out[3],depths[3];uint32_t screen,flags,screens[3];
    epok::project_geometry_vertex(small[0],zero,camera_out,screen,flags);
    const auto packed=registers();
    epok::project_geometry_vertex(wide[0],zero,camera_out,screen,flags);
    assert(packed==registers());
    epok::project_geometry_triple(small,zero,screens,depths);const auto triple=registers();
    epok::project_geometry_triple(wide,zero,screens,depths);assert(triple==registers());
    epok::GteProjectionInput inputs[3];
    for(unsigned i=0;i<3;++i)assert(epok::pack_projection_input(same[i],zero,inputs[i]));
    epok::begin_projection_triple(inputs);assert(epok::finish_projection_triple(screens,depths));
    assert(triple==registers());
    assert(epok::load_projection_input<0>(same[0],zero));
    assert(epok::load_projection_input<1>(same[1],zero));
    assert(epok::load_projection_input<2>(same[2],zero));
    epok::begin_loaded_projection_triple();assert(registers()==triple);
    epok::project_geometry_vertex(wide[0],zero,camera_out,screen,flags);const auto single=registers();
    assert(epok::project_packed_vertex(inputs[0],screen,depths[0]));assert(single==registers());
    psyqo::GTE::write<psyqo::GTE::Register::FLAG>(epok::gte_projection_flags);
    assert(!epok::finish_projection_triple(screens,depths));
    assert(!epok::project_packed_vertex(inputs[0],screen,depths[0]));
    psyqo::GTE::clear<psyqo::GTE::Register::FLAG>();
    {
        using namespace psyqo::GTE;
        // Output contract, not a host emulation of RTPT: all accepted outputs
        // overwrite poison, the unused fourth slot stays untouched, and a
        // rejected batch publishes nothing (the caller must use CPU fallback).
        uint32_t out_screen[4]={0xdeadbeefu,0xdeadbeefu,0xdeadbeefu,0xdeadbeefu};
        int32_t out_depth[4]={INT32_MIN,INT32_MIN,INT32_MIN,INT32_MIN};
        write<Register::SXY0>(11);write<Register::SXY1>(22);write<Register::SXY2>(33);
        write<Register::SZ1>(44);write<Register::SZ2>(55);write<Register::SZ3>(66);
        assert(epok::finish_projection_triple(out_screen,out_depth));
        for(unsigned i=0;i<3;++i){assert(out_screen[i]==11*(i+1));assert(out_depth[i]==int32_t(44+11*i));}
        assert(out_screen[3]==0xdeadbeefu&&out_depth[3]==INT32_MIN);
        write<Register::FLAG>(epok::gte_projection_flags);
        for(unsigned i=0;i<4;++i){out_screen[i]=0xdeadbeefu;out_depth[i]=INT32_MIN;}
        assert(!epok::finish_projection_triple(out_screen,out_depth));
        for(unsigned i=0;i<4;++i)assert(out_screen[i]==0xdeadbeefu&&out_depth[i]==INT32_MIN);
        clear<Register::FLAG>();
    }
    const int32_t translation[]={12345,-67890,51200};
    const auto prior_translation=registers();epok::load_projection_translation(translation);
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::TRX>()==12345);
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::TRY>()==67890);
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::TRZ>()==51200);
    const auto translated=registers();
    for(unsigned r=0;r<translated.size();++r)
        if(r!=unsigned(psyqo::GTE::Register::TRX)&&r!=unsigned(psyqo::GTE::Register::TRY)&&r!=unsigned(psyqo::GTE::Register::TRZ))
            assert(translated[r]==prior_translation[r]);
    epok::project_geometry_vertex(big[0],zero,camera_out,screen,flags);
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::VXY0>()==(uint32_t(uint16_t(-4096))|(uint32_t(8192)<<16)));
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::VZ0>()==32767);
    uint32_t random=73641;
    auto next=[&](){random=random*1664525u+1013904223u;return random;};
    for(unsigned sample=0;sample<10000;++sample){
        int32_t world[3],offset[3];bool fits=true;
        for(unsigned c=0;c<3;++c){
            world[c]=int32_t(next());
            offset[c]=-(world[c]>>4)+int32_t(next()%131072)-65536;
            const int64_t v=(int64_t(world[c])>>4)+offset[c];fits&=v>=-32768&&v<=32767;
        }
        epok::GteProjectionInput packed_input;
        assert(epok::pack_projection_input(world,offset,packed_input)==fits);
        epok::project_geometry_vertex(world,offset,camera_out,screen,flags);
        const auto expected=registers();
        assert(epok::project_packed_vertex(packed_input,screen,depths[0]));assert(expected==registers());
        // Direct loads retain the exact packed bits even on rejection. Only
        // their selected pair may change; anchor retries overwrite all pairs.
        const auto check_load=[&](auto slot){
            constexpr unsigned index=decltype(slot)::value;
            constexpr auto xy=index==0?psyqo::GTE::Register::VXY0:index==1?psyqo::GTE::Register::VXY1:psyqo::GTE::Register::VXY2;
            constexpr auto z=index==0?psyqo::GTE::Register::VZ0:index==1?psyqo::GTE::Register::VZ1:psyqo::GTE::Register::VZ2;
            const auto before=registers();
            assert(epok::load_projection_input<index>(world,offset)==fits);
            const auto after=registers();
            for(unsigned r=0;r<after.size();++r)
                assert(after[r]==(r==unsigned(xy)?packed_input.xy:r==unsigned(z)?packed_input.z:before[r]));
        };
        check_load(std::integral_constant<unsigned,0>{});
        check_load(std::integral_constant<unsigned,1>{});
        check_load(std::integral_constant<unsigned,2>{});
    }
    for(unsigned sample=0;sample<10000;++sample){
        Camera camera;Matrix world=Matrix::identity();
        // Camera inverse bases include rotations, nonuniform scale and shear
        // as produced by hierarchical cameras. Translation must stay separate.
        for(int r=0;r<3;++r){
            camera.origin[r]=Q(int32_t(next()%81920)-40960,Q::RAW);
            for(int c=0;c<3;++c)camera.basis.values[r][c]=Q(int32_t(next()%8193)-4096,Q::RAW);
            world.values[r][3]=camera.origin[r]+Q(int32_t(next()%163840)-81920,Q::RAW);
            for(int c=0;c<3;++c)world.values[r][c]=Q(int32_t(next()%8193)-4096,Q::RAW);
        }
        if(sample%2==0)for(int r=0;r<3;++r)for(int c=0;c<3;++c)
            world.values[r][c]=r==c?Q(1.):Q(0.); // diagonal fast path
        const auto expected=camera.compose(world);
        assert(epok::load_projection_matrix(expected));const auto before=registers();
        auto shifted=camera;auto moved=world;
        for(int r=0;r<3;++r){
            const Q offset(int32_t(next()%65536001)-32768000,Q::RAW);
            shifted.origin[r]+=offset;moved.values[r][3]+=offset;
        }
        const auto actual=shifted.compose(moved);
        assert(equal(expected,actual));
        assert(epok::load_projection_matrix(actual));assert(before==registers());
        Q position[3],a[3],b[3];
        for(int r=0;r<3;++r)position[r]=world.values[r][3];
        camera.point(position,a);
        for(int r=0;r<3;++r)position[r]=moved.values[r][3];
        shifted.point(position,b);
        for(int r=0;r<3;++r)assert(a[r]==b[r]&&a[r]==expected.values[r][3]);
    }
    // Main's actual starting origin and world slot: world Z is well beyond
    // 128, but its camera-relative Z is 8 and the GTE receives Q8 TRZ=2048.
    Camera main;main.origin[0]=120.74;main.origin[1]=24.04;main.origin[2]=906.76;
    auto object=Matrix::identity();object.values[0][3]=120.74;object.values[1][3]=20.04;object.values[2][3]=914.76;
    const auto relative=main.compose(object);
    assert(relative.values[0][3]==0.&&relative.values[1][3]==-4.&&relative.values[2][3]==8.);
    assert(epok::load_projection_matrix(relative));
    assert(psyqo::GTE::readRaw<psyqo::GTE::Register::TRZ>()==2048);
    // Across the entire signed Q12 range subtraction/products never wrap.
    Camera extreme;Q p[3]={Q(INT32_MAX,Q::RAW),0.,0.},out[3];
    extreme.origin[0]=Q(INT32_MIN,Q::RAW);extreme.point(p,out);
    assert(out[0].raw()==INT32_MAX);
    extreme.basis.values[0][0]=-1.;extreme.point(p,out);assert(out[0].raw()==INT32_MIN);
    // Accumulate wide before narrowing: individually overflowing terms may cancel.
    extreme.basis.values[0][0]=2.;extreme.basis.values[0][1]=-2.;
    extreme.origin[0]=0.;p[1]=p[0];extreme.point(p,out);assert(out[0].raw()==0);
    std::puts("Camera-relative Q12: 10000 translated matrix/GTE-register pairs, Main origin and overflow tests passed.");
}
