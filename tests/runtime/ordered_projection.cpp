#include <cassert>
#include <cstdio>
#include <cmath>
#include "psyqo/fixed-point.hh"
#include "../../runtime/ordered_projection.hpp"
#include "gte-matrix/gte_geometry.hpp"
using Q=psyqo::FixedPoint<12>;
using Units=epok::CameraUnits<64,1<<27,6,epok::ScreenFrustum<512,240>>;
// Compact cooked vertices must reproduce the per-vertex GTE inputs bit for bit
// whenever the block bounds admit the offset, and decode to the exact Q12.
static void compact_vertices(){
    uint32_t random=91;
    const auto next=[&](){random=random*1664525u+1013904223u;return random;};
    for(unsigned sample=0;sample<200000;++sample){
        epok::OrderedProjectionBlock block{0,1,epok::ordered_projection_compact,
            {int32_t(next())>>6,int32_t(next())>>6,int32_t(next())>>6},{-32768,-32768,-32768},{32767,32767,32767}};
        int16_t relative[3];int32_t q12[3];uint32_t fraction=0;
        for(unsigned c=0;c<3;++c){
            relative[c]=int16_t(next());const uint32_t bits=next()&15u;fraction|=bits<<(16+4*c);
            q12[c]=int32_t(uint32_t(relative[c]+block.center[c])*16u+bits);
        }
        const uint32_t words[2]={uint32_t(uint16_t(relative[0]))|(uint32_t(uint16_t(relative[1]))<<16),
                                 uint32_t(uint16_t(relative[2]))|fraction};
        const auto* entry=reinterpret_cast<const uint8_t*>(words);
        int32_t decoded[3];epok::ordered_vertex_world(block,entry,decoded);
        for(unsigned c=0;c<3;++c)assert(decoded[c]==q12[c]);
        // Any camera offset that keeps this vertex in range: the bounds check
        // in the renderer is per block, the arithmetic here is per vertex.
        int32_t offset[3];
        for(unsigned c=0;c<3;++c){
            const int32_t target=int32_t(int16_t(next()));
            offset[c]=target-(q12[c]>>4);
        }
        epok::GteProjectionInput expected;
        assert(epok::pack_projection_input(q12,offset,expected));
        int32_t sum[3];for(unsigned c=0;c<3;++c)sum[c]=block.center[c]+offset[c];
        const epok::CompactProjectionOffset compact{uint32_t(uint16_t(sum[0]))|(uint32_t(sum[1])<<16),
            uint32_t(uint16_t(sum[0])),uint32_t(sum[2])};
        const auto actual=epok::compact_projection_input(entry,compact);
        assert(actual.xy==expected.xy);
        assert(int16_t(actual.z)==int16_t(expected.z)&&actual.z==expected.z);
        epok::GteProjectionInput checked;
        assert(epok::checked_compact_projection_input(entry,sum,checked));
        assert(checked.xy==expected.xy&&checked.z==expected.z);
        // Out of range on any axis: the same rejection and words as per vertex.
        int32_t far[3]={offset[0],offset[1],offset[2]};far[sample%3]+=(sample&8)?40000:-40000;
        int32_t far_sum[3];for(unsigned c=0;c<3;++c)far_sum[c]=block.center[c]+far[c];
        epok::GteProjectionInput reference;
        const bool reference_fits=epok::pack_projection_input(q12,far,reference);
        assert(epok::checked_compact_projection_input(entry,far_sum,checked)==reference_fits);
        assert(checked.xy==reference.xy&&checked.z==reference.z);
    }
    // Wide blocks keep absolute Q12 words.
    const int32_t absolute[3]={-123456789,5,987654321};int32_t decoded[3];
    epok::OrderedProjectionBlock wide{0,1,0,{},{},{}};
    epok::ordered_vertex_world(wide,reinterpret_cast<const uint8_t*>(absolute),decoded);
    for(unsigned c=0;c<3;++c)assert(decoded[c]==absolute[c]);
    assert(epok::ordered_face_block(uint16_t((513u<<2)|1u))==513);
    std::puts("Compact ordered vertices: 200000 exact GTE inputs, checked out-of-range faces, Q12 decode and wide blocks passed.");
}
// GTE perspective division model: documented seed table, overflow boundary,
// and the reciprocal within two units of the ideal rounded quotient.
static void gte_division(){
    assert(epok::gte_unr_table.value[0]==0xff&&epok::gte_unr_table.value[256]==0);
    for(uint32_t h:{1u,120u,160u,213u,240u,1000u}){
        uint32_t n;
        assert(!epok::gte_divide(h,h/2,n)&&n==0x1ffff);
        uint32_t previous=0xffffffffu;
        for(uint32_t sz=h/2+1;sz<=0xffff;++sz){
            assert(epok::gte_divide(h,sz,n));
            const uint64_t ideal=((uint64_t(h)*0x20000/sz)+1)/2;
            assert(n<=0x1ffff&&(n>ideal?n-ideal:ideal-n)<=2);
            assert(n<=previous);previous=n;
        }
    }
    // Screen saturation and divide overflow are rejected like GTE flags.
    uint32_t screen;
    const int32_t centre[3]={0,0,320},left[3]={-4000,0,320},near_camera[3]={0,0,79};
    assert(epok::gte_project_point(centre,160,256,120,screen)&&screen==((120u<<16)|256u));
    assert(!epok::gte_project_point(left,160,256,120,screen));
    assert(!epok::gte_project_point(near_camera,160,256,120,screen));
    std::puts("GTE division model: seed table, overflow, monotonic reciprocal within two units passed.");
}
static void anchors(){
    epok::OrderedProjectionBlock blocks[]={{40,3,0,{0,0,0},{},{}},{100,2,0,{0,0,0},{},{}}};
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
    gte_division();
    anchors();
    compact_vertices();
    const auto check_depth=[](int32_t value){
        assert(epok::ordered_depth_average(value,false)==value/192);
        assert(epok::ordered_depth_average(value,true)==value/256);
    };
    // Include every rounding boundary near zero and both signed extremes.
    for(int32_t i=-65536;i<=65536;++i)check_depth(i);
    for(int32_t i=0;i<65536;++i){check_depth(INT32_MIN+i);check_depth(INT32_MAX-i);}
    // GTE depth (AVSZ3/AVSZ4 contract): quads equal the CPU division exactly
    // for every non-negative four-depth sum; triangles are never farther and
    // at most 1.6% (plus rounding) nearer; out-of-range sums saturate.
    for(int32_t sum=0;sum<=4*65535;++sum){
        assert(epok::ordered_gte_depth_average(sum,true)==sum/256);
        if(sum<=3*65535){
            const int32_t cpu=sum/192,gte=epok::ordered_gte_depth_average(sum,false);
            assert(gte<=cpu&&cpu-gte<=cpu/60+1);
        }
    }
    assert(epok::ordered_gte_depth_average(-1,false)==0&&epok::ordered_gte_depth_average(-4096,true)==0);
    assert(epok::ordered_gte_depth_average(INT32_MAX/16,true)==0xffff);
    std::puts("GTE depth average: exact quads, bounded triangles and saturation passed.");
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
    epok::CameraSpace<Q> camera;
    const Q scale[]={1.,1.};
    epok::ProjectedVertex projected[4];uint32_t screen[4];int32_t depth[4];
    auto project=[&](unsigned count=4){
        // Renderer output arrays need no initialization: even a rejected CPU
        // projection must write every used depth/screen (zero if invisible).
        // A triangle must not touch or depend on its unused fourth corner.
        for(unsigned i=0;i<4;++i){screen[i]=0xdeadbeefu;depth[i]=INT32_MIN;}
        const bool direct=epok::project_ordered_cpu<Units,512,240>(
            points,count,camera,scale,projected,screen,depth);
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
