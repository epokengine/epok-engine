#include <array>
#include <cassert>
#include <cmath>
#include <cstdio>
#include "psyqo/fixed-point.hh"
#include "motion_interpolation.hpp"
#include "time.hpp"
using Q=psyqo::FixedPoint<12>;
using Matrix=epok::Affine<Q>;
struct Transform{Q position[3]={},rotation[3]={},scale[3]={1.,1.,1.};};
struct Object{Transform transform;int parent=-1;uint32_t generation=1;bool alive=true,active=true;};
using Objects=std::array<Object,4>;
using Matrices=std::array<Matrix,4>;
using Motion=epok::MotionInterpolation<Q,4,true>;
static Matrix local(const Transform& t){auto m=Matrix::identity();for(int i=0;i<3;++i){m.values[i][3]=t.position[i];m.values[i][i]=t.scale[i];}return m;}
static Matrices worlds(const Objects& objects){
    Matrices result;
    for(unsigned i=0;i<4;++i){auto m=Matrix::identity();int chain[33],n=0,j=int(i);
        while(j>=0&&j<4&&n<33){chain[n++]=j;j=objects[j].parent;}
        if(j<0)while(n)m=m.compose(local(objects[chain[--n]].transform));
        result[i]=m;
    }return result;
}
int main(){
    Objects objects;Motion motion;
    objects[0].parent=3;objects[3].transform.scale[0]=2.;
    motion.select(objects,4,[](size_t i){return i==0;}); // Include non-rendering ancestor 3.
    motion.before_tick(objects,4);
    objects[3].transform.position[0]=2.;objects[0].transform.position[0]=4.;
    motion.after_tick(objects,4);const auto current=worlds(objects);
    auto half=motion.prepare(objects,current,4,2048);
    assert(half[3].values[0][3]==1.&&half[0].values[0][3]==5.);
    assert(current[0].values[0][3]==10.&&objects[0].transform.position[0]==4.);
    assert(motion.local(0,objects[0].transform).position[0]==2.);
    assert(motion.prepare(objects,current,4,0)[0].values[0][3]==0.);
    assert(motion.prepare(objects,current,4,4096)[0].values[0][3]==10.);
    motion.select(objects,4,[](size_t){return false;});
    assert(&motion.prepare(objects,current,4,0)==&current);
    motion.select(objects,4,[](size_t){return true;});
    motion.before_tick(objects,4);objects[0].transform.position[0]=6.;motion.after_tick(objects,4);
    objects[0].transform.position[0]=4.; // Out-of-tick writes invalidate that interpolation.
    // Direct writes outside a fixed step, slot reuse and reparenting snap.
    objects[0].transform.position[0]=20.;auto teleported=worlds(objects);
    assert(motion.prepare(objects,teleported,4,2048)[0].values[0][3]==42.);
    objects[0].transform.position[0]=4.;++objects[3].generation;
    assert(motion.prepare(objects,current,4,2048)[0].values[0][3]==10.);
    motion.clear();assert(motion.prepare(objects,current,4,0)[0].values[0][3]==10.);
    motion.before_tick(objects,4);objects[0].parent=-1;motion.after_tick(objects,4);
    auto reparented=worlds(objects);assert(motion.prepare(objects,reparented,4,0)[0].values[0][3]==4.);
    motion.before_tick(objects,4);objects[0].active=false;motion.after_tick(objects,4);
    assert(motion.prepare(objects,reparented,4,0)[0].values[0][3]==4.);
    // Clearing during a tick (teleport/camera cut) must survive after_tick.
    motion.before_tick(objects,4);objects[0].transform.position[0]=8.;motion.clear();motion.after_tick(objects,4);
    assert(motion.prepare(objects,worlds(objects),4,0)[0].values[0][3]==8.);
    objects[0].parent=3;objects[3].parent=0;motion.before_tick(objects,4);motion.after_tick(objects,4);
    assert(motion.prepare(objects,worlds(objects),4,1024)[0].values[0][3]==0.);
    // Constant velocity remains uniformly presented at mismatched render rates.
    for(unsigned fps:{30u,34u,47u,60u,120u}){
        epok::Time clock;clock.reset(0);Objects moving;Motion smooth;
        for(unsigned f=1;f<=fps*2;++f){
            const uint32_t now=uint32_t(uint64_t(f)*1000000/fps);unsigned steps=clock.advance(now);
            for(unsigned s=0;s<steps;++s){smooth.before_tick(moving,4);clock.begin_tick();moving[0].transform.position[0]+=Q(clock.delta_raw,Q::RAW);}
            if(steps)smooth.after_tick(moving,4);
            auto authoritative=worlds(moving);const auto& rendered=smooth.prepare(moving,authoritative,4,clock.interpolation_raw());
            if(clock.ticks){const double expected=double(now)/1000000.-1./60.;
                assert(std::abs(rendered[0].values[0][3].raw()/4096.-expected)<.001);}
            assert(authoritative[0].values[0][3]==moving[0].transform.position[0]);
        }
    }
    epok::MotionInterpolation<Q,4,false> off;static_assert(sizeof(off)==1);
    assert(&off.prepare(objects,current,4,0)==&current);
    std::puts("Motion interpolation: cadence, parent scale/order, snap, lifecycle, cycles and authoritative state passed.");
}
