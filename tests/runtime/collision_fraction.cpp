#include <cassert>
#include <cstdint>
#include "../../runtime/collision.hpp"
#include "psyqo/fixed-point.hh"
using Q=psyqo::FixedPoint<12>;
static uint32_t random_state=0xced01245;
static uint32_t random_word(){random_state^=random_state<<13;random_state^=random_state>>17;return random_state^=random_state<<5;}
int main(){
    for(unsigned i=0;i<200000;++i){
        const uint64_t denominator=1+(random_word()%(i%2?65535u:0xffffffffu));
        const uint64_t numerator=random_word()%(denominator+1);
        assert(epok::collision_fraction24(numerator,denominator)==int64_t((numerator<<24)/denominator));
    }
    // Compare complete public ray results against the previous wide slab math,
    // including zero axes, degenerate boxes, reverse rays and large deltas.
    epok::CollisionWorld<Q,1,1> world;
    epok::CollisionWorld<Q,1,1,true,true,false> wide_world;
    for(unsigned i=0;i<100000;++i){
        epok::ColliderT<Q> c;c.enabled=true;
        for(int k=0;k<3;++k)c.half_extents[k]=Q(i%19?int32_t(random_word()%8000):0,Q::RAW);
        world.set(0,c,epok::Affine<Q>::identity());
        wide_world.set(0,c,epok::Affine<Q>::identity());
        const auto& box=*world.bounds(0);Q origin[3],delta[3];
        for(int k=0;k<3;++k){origin[k]=Q(int32_t(random_word()%32000)-16000,Q::RAW);delta[k]=Q(i%13?int32_t(random_word()%64000)-32000:int32_t(random_word()),Q::RAW);}
        if(i%7==0)delta[i%3]=0.0;
        constexpr int64_t one=int64_t(1)<<24;
        int64_t enter=0,leave=one;int axis=-1,sign=0;bool inside=true,hit=true;
        for(int k=0;k<3;++k){
            int64_t p=origin[k].raw(),d=delta[k].raw(),lo=box.min[k].raw(),hi=box.max[k].raw();
            if(p<=lo||p>=hi)inside=false;
            if(!d){if(p<lo||p>hi){hit=false;break;}continue;}
            int64_t near=((lo-p)*one)/d,far=((hi-p)*one)/d;int normal=-1;
            if(near>far){auto temp=near;near=far;far=temp;normal=1;}
            if(near>enter||(near==enter&&axis<0)){enter=near;axis=k;sign=normal;}
            if(far<leave)leave=far;
            if(enter>leave){hit=false;break;}
        }
        epok::AabbT<Q> point;for(int k=0;k<3;++k)point.min[k]=point.max[k]=origin[k];
        hit=hit&&leave>=0&&enter<=one&&epok::aabb_may_touch(epok::swept_bounds(point,delta),box);
        const auto actual=world.raycast(origin,delta);
        const auto wide=wide_world.raycast(origin,delta);
        assert(actual.entity==wide.entity&&actual.generation==wide.generation&&actual.fraction==wide.fraction&&actual.started_inside==wide.started_inside);
        for(int k=0;k<3;++k){assert(actual.point[k]==wide.point[k]);assert(actual.normal[k]==wide.normal[k]);}
        if(i%4==0){
            Q movement[3];epok::AabbT<Q> moving;
            for(int k=0;k<3;++k){const auto center=Q(int32_t(random_word()%32000)-16000,Q::RAW);
                moving.min[k]=center-Q(512,Q::RAW);moving.max[k]=center+Q(512,Q::RAW);
                movement[k]=Q(int32_t(random_word()%16000)-8000,Q::RAW);}
            const auto a=world.move_and_slide(moving,movement),b=wide_world.move_and_slide(moving,movement);
            assert(a.entity==b.entity&&a.generation==b.generation&&a.grounded==b.grounded&&a.blocked==b.blocked&&a.unresolved_overlap==b.unresolved_overlap);
            for(int k=0;k<3;++k){assert(a.displacement[k]==b.displacement[k]);assert(a.normal[k]==b.normal[k]);}
        }
        assert(bool(actual)==hit);
        if(hit){assert(actual.started_inside==inside);assert(actual.fraction.raw()==int32_t(enter*4096/one));for(int k=0;k<3;++k)assert(actual.normal[k].raw()==(k==axis?sign*4096:0));}
    }
}
