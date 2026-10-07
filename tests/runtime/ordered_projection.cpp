#include <cassert>
#include <cstdio>
#include <cmath>
#include "psyqo/fixed-point.hh"
#include "../../runtime/ordered_projection.hpp"
using Q=psyqo::FixedPoint<12>;
using Units=epok::CameraUnits<64,1<<27,6,epok::ScreenFrustum<512,240>>;
static void anchors(){
    epok::OrderedProjectionBlock blocks[]={{40,3,{0,0,0}},{100,2,{0,0,0}}};
    epok::OrderedDrawData data{};data.projections=blocks;data.projection_count=2;
    for(uint32_t token=0;token<112;++token){
        const auto* expected=token>=40&&token<52?blocks:token>=100&&token<108?blocks+1:nullptr;
        assert(epok::ordered_projection_block(&data,token)==expected);
    }
    assert(!epok::ordered_projection_block(nullptr,40));
    data.projections=nullptr;assert(!epok::ordered_projection_block(&data,40));
    uint32_t random=823147;
    const auto next=[&](){random=random*1664525u+1013904223u;return random;};
    unsigned accepted=0;
    for(unsigned sample=0;sample<100000;++sample){
        auto matrix=epok::Affine<Q>::identity();int32_t base[3],center[3],world[3];
        for(unsigned c=0;c<3;++c){
            base[c]=-(int32_t(next())>>4);center[c]=int32_t(next())>>4;
            const int64_t p=int64_t(center[c])*16+int32_t(next()%65536)-32768;
            world[c]=int32_t(p<INT32_MIN?INT32_MIN:p>INT32_MAX?INT32_MAX:p);
        }
        for(unsigned r=0;r<3;++r){
            for(unsigned c=0;c<3;++c)matrix.values[r][c]=Q(int32_t(next()%32769)-16384,Q::RAW);
            matrix.values[r][3]=Q(int32_t(next()%131072)-65536,Q::RAW);
        }
        int32_t offset[3]={17,18,19},tr[3]={20,21,22};
        if(!epok::ordered_projection_anchor(matrix,base,center,offset,tr)){
            for(unsigned c=0;c<3;++c){assert(offset[c]==int32_t(17+c));assert(tr[c]==int32_t(20+c));}
            continue;
        }
        ++accepted;int32_t reference[3];epok::ordered_gte_camera_reference(matrix,world,base,reference);
        for(unsigned c=0;c<3;++c){
            assert((offset[c]-base[c])%4096==0);
            assert(epok::ordered_gte_input_fits(world[c],-offset[c]));
        }
        for(unsigned r=0;r<3;++r){
            const int sign=r==1?-1:1;
            int64_t a=int64_t(sign)*(matrix.values[r][3].raw()>>4)*4096;
            int64_t b=int64_t(sign)*tr[r]*4096;
            for(unsigned c=0;c<3;++c){
                a+=int64_t(sign)*matrix.values[r][c].raw()*((world[c]>>4)+base[c]);
                b+=int64_t(sign)*matrix.values[r][c].raw()*((world[c]>>4)+offset[c]);
            }
            assert(a==b);assert(sign*int32_t(b>>12)==reference[r]);
        }
    }
    assert(accepted>90000);
    auto extreme=epok::Affine<Q>::identity();
    for(unsigned r=0;r<3;++r)for(unsigned c=0;c<3;++c)extreme.values[r][c]=Q(32767,Q::RAW);
    const int32_t base[]={134217728,134217728,134217728},center[]={134217727,134217727,134217727};
    int32_t offset[]={17,18,19},tr[]={20,21,22};
    assert(!epok::ordered_projection_anchor(extreme,base,center,offset,tr));
    assert(offset[0]==17&&tr[2]==22);
    // Native far quad from the regression below now fits GTE inputs without
    // losing any fractional accumulator bits or changing the near/far plane.
    const int32_t origin[]={0,0,0},far_center[]={0,0,200*256};
    assert(epok::ordered_projection_anchor(epok::Affine<Q>::identity(),origin,far_center,offset,tr));
    const int32_t corner[]={10*4096,10*4096,200*4096};
    for(unsigned c=0;c<3;++c)assert(epok::ordered_gte_input_fits(corner[c],-offset[c]));
    assert(tr[2]+(corner[2]>>4)+offset[2]==200*256);
    std::printf("Projection anchors: %u exact wide MAC sums, signed boundaries, rejection and block lookup passed.\n",accepted);
}
int main(){
    anchors();
    const auto check_depth=[](int32_t value){
        assert(epok::ordered_depth_average(value,false)==value/192);
        assert(epok::ordered_depth_average(value,true)==value/256);
    };
    // Include every rounding boundary near zero and both signed extremes.
    for(int32_t i=-65536;i<=65536;++i)check_depth(i);
    for(int32_t i=0;i<65536;++i){check_depth(INT32_MIN+i);check_depth(INT32_MAX-i);}
    uint32_t depth_random=61723;
    for(unsigned i=0;i<1000000;++i){
        depth_random=depth_random*1664525u+1013904223u;check_depth(int32_t(depth_random));
    }
    epok::Affine<Q> matrix=epok::Affine<Q>::identity();
    const int32_t input[]={31,-17,1600},zero[]={0,0,0};int32_t result[3];
    epok::ordered_gte_camera_reference(matrix,input,zero,result);
    assert(result[0]==1&&result[1]==-2&&result[2]==100);
    matrix.values[1][1]=0.5;matrix.values[1][3]=Q(-1,Q::RAW);
    const int32_t half[]={0,16,1600};
    epok::ordered_gte_camera_reference(matrix,half,zero,result);
    assert(result[1]==0); // Negated Y sum floors BEFORE converting back to Y-up.
    uint32_t oracle_random=29173;
    for(unsigned test=0;test<10000;++test){
        int32_t world[3],offset[3];
        for(unsigned c=0;c<3;++c){
            oracle_random=oracle_random*1664525u+1013904223u;world[c]=int32_t(oracle_random);
            offset[c]=-(world[c]>>4)+int32_t(oracle_random%65536)-32768;
        }
        for(unsigned r=0;r<3;++r)for(unsigned c=0;c<4;++c){
            oracle_random=oracle_random*1664525u+1013904223u;
            matrix.values[r][c]=Q(int32_t(oracle_random%65535)-32767,Q::RAW);
        }
        epok::ordered_gte_camera_reference(matrix,world,offset,result);
        for(unsigned r=0;r<3;++r){
            const int sign=r==1?-1:1;
            double sum=sign*std::floor(double(matrix.values[r][3].raw())/16)*4096;
            for(unsigned c=0;c<3;++c)
                sum+=sign*double(matrix.values[r][c].raw())*(std::floor(double(world[c])/16)+offset[c]);
            assert(result[r]==sign*int32_t(std::floor(sum/4096)));
        }
    }
    assert(epok::ordered_gte_input_fits(-524288,0));
    assert(epok::ordered_gte_input_fits(524287,0));
    assert(!epok::ordered_gte_input_fits(-524289,0));
    assert(!epok::ordered_gte_input_fits(524288,0));
    assert(!epok::ordered_gte_input_fits(INT32_MAX,INT32_MIN>>4));
    assert(!epok::ordered_gte_input_fits(INT32_MIN,INT32_MAX>>4));
    assert(epok::ordered_gte_input_fits(INT32_MIN,INT32_MIN>>4));
    assert(epok::ordered_gte_input_fits(INT32_MAX,INT32_MAX>>4));
    uint32_t random=17456;
    for(unsigned i=0;i<10000;++i){
        random=random*1664525u+1013904223u;const int32_t point=random;
        random=random*1664525u+1013904223u;const int32_t camera=int32_t(random)>>4;
        const int64_t expected=(int64_t(point)-int64_t(camera)*16)>>4;
        assert(epok::ordered_gte_input_fits(point,camera)==(expected>=-32768&&expected<=32767));
        assert((point>>4)-camera==expected);
    }
    // Vertices well beyond signed Q8 GTE inputs still project to a native quad.
    int32_t points[4][3]={{-10*4096,10*4096,200*4096},{10*4096,10*4096,200*4096},
                        {-10*4096,-10*4096,200*4096},{10*4096,-10*4096,200*4096}};
    epok::BakedDrawFace face{};for(unsigned i=0;i<4;++i)face.indices[i]=i*12;
    epok::CameraSpace<Q> camera;
    const Q scale[]={1.,1.};const unsigned corners[]={0,1,2,3};
    epok::ProjectedVertex projected[4];uint32_t screen[4];int32_t depth[4];
    auto project=[&](unsigned count=4){
        // Renderer output arrays need no initialization: even a rejected CPU
        // projection must write every used depth/screen (zero if invisible).
        // A triangle must not touch or depend on its unused fourth corner.
        for(unsigned i=0;i<4;++i){screen[i]=0xdeadbeefu;depth[i]=INT32_MIN;}
        const bool direct=epok::project_ordered_cpu<Units,512,240>(
            face,reinterpret_cast<const uint8_t*>(points),corners,count,camera,scale,projected,screen,depth);
        for(unsigned i=0;i<count;++i){
            assert(depth[i]==projected[i].camera[2]);
            assert(screen[i]==(projected[i].visible?projected[i].screen.packed:0u));
            assert(projected[i].fog==0);
        }
        for(unsigned i=count;i<4;++i)assert(screen[i]==0xdeadbeefu&&depth[i]==INT32_MIN);
        return direct;
    };
    assert(project());
    for(unsigned i=0;i<4;++i){assert(depth[i]==51200&&projected[i].visible);assert(screen[i]==projected[i].screen.packed);}
    assert(projected[0].screen.x==248&&projected[0].screen.y==112);
    assert(projected[3].screen.x==264&&projected[3].screen.y==128);
    // Translating both camera and world never changes the projected packet.
    const auto original=screen[0];
    for(auto& point:points)point[2]+=1000*4096;
    camera.origin[2]=1000.;assert(project()&&screen[0]==original);
    // A fourth corner crossing the near plane must fall back; a half remains valid.
    points[3][2]=1000*4096;assert(!project());assert(project(3));
    // Guard-band or GPU span overflow must never be reclassified as direct.
    camera.origin[2]=0.;
    for(unsigned i=0;i<4;++i){points[i][0]=(i&1?90:-90)*4096;points[i][1]=0;points[i][2]=20*4096;}
    assert(!project()); // 1440 pixels wide despite every corner fitting the guard band.
    for(unsigned i=0;i<4;++i){points[i][0]=0;points[i][1]=(i&1?40:-40)*4096;}
    assert(!project()); // 640 pixels high, also inside the signed guard band.
    for(unsigned i=0;i<4;++i)points[i][0]=1000*4096;
    assert(!project());
    std::puts("Ordered CPU projection: native far quads, translated world, half-quads, near and span guards pass.");
}
