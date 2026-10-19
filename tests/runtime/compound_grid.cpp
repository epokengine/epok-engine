// Differential tests: indexed and original linear queries must return exactly
// the same Q12 values, contact ties and trigger event sequence.
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/collision.hpp"
#include "psyqo/fixed-point.hh"

using Q=psyqo::FixedPoint<12>;
using Box=epok::AabbT<Q>;
using Collider=epok::ColliderT<Q>;
using Indexed=epok::CollisionWorld<Q,8,16>;
using Linear=epok::CollisionWorld<Q,8,16,false>;
static Q raw(int32_t v){return Q(v,Q::RAW);}
static uint32_t random_state=0x71c853a9;
static uint32_t random_value(){random_state^=random_state<<13;random_state^=random_state>>17;random_state^=random_state<<5;return random_state;}
static int between(int lo,int hi){return lo+int(random_value()%unsigned(hi-lo+1));}
static epok::Affine<Q> matrix(int x=0,int y=0,int z=0){
    auto m=epok::Affine<Q>::identity();m.values[0][3]=raw(x);m.values[1][3]=raw(y);m.values[2][3]=raw(z);return m;
}

struct Fixture {
    std::vector<int16_t> boxes;
    std::vector<uint16_t> offsets,indices;
    epok::CompoundGrid grid{};
    Collider collider;
    Fixture(){
        auto add=[&](int x,int y,int z,int X,int Y,int Z){for(int v:{x,y,z,X,Y,Z})boxes.push_back(int16_t(v));};
        // Adjacent floors at different heights, thin walls, stair treads,
        // duplicate/tied faces and parts spanning many cells.
        for(int z=-4;z<4;++z)for(int x=-4;x<4;++x){
            const int top=((x+z+8)%4)*102;
            add(x*2048,top-128,z*2048,(x+1)*2048,top,(z+1)*2048);
            if((x+z+8)%3==0)add(x*2048,top,z*2048,x*2048+2,top+768,(z+1)*2048);
        }
        add(-8192,128,-8192,8192,130,8192);
        add(-256,-128,-256,0,768,0);
        add(0,-128,-256,256,768,0);
        for(int i=0;i<80;++i){
            int x=between(-8192,7000),z=between(-8192,7000),y=between(-128,1200);
            add(x,y,z,x+between(2,1192),y+between(2,500),z+between(2,1192));
        }
        grid.boxes=boxes.data();grid.box_count=uint16_t(boxes.size()/6);
        grid.columns=grid.rows=9;grid.origin_x=grid.origin_z=-8192;grid.cell_shift=15;
        std::vector<std::vector<uint16_t>> cells(81);
        for(unsigned n=0;n<grid.box_count;++n){const auto* b=boxes.data()+n*6;
            for(int z=(int(b[2])+8192)/2048;z<=(int(b[5])+8192)/2048;++z)
                for(int x=(int(b[0])+8192)/2048;x<=(int(b[3])+8192)/2048;++x)cells[size_t(z)*9+x].push_back(uint16_t(n));
        }
        offsets.push_back(0);
        for(const auto& cell:cells){indices.insert(indices.end(),cell.begin(),cell.end());offsets.push_back(uint16_t(indices.size()));}
        grid.offsets=offsets.data();grid.indices=indices.data();
        collider.enabled=true;collider.boxes=boxes.data();collider.box_count=grid.box_count;collider.box_grid=&grid;
        collider.half_extents[0]=collider.half_extents[2]=Q(32.);
        collider.center[1]=Q(3.);collider.half_extents[1]=Q(4.);
    }
};

static void same(const epok::SpatialHitT<Q>& a,const epok::SpatialHitT<Q>& b){
    assert(a.entity==b.entity&&a.generation==b.generation&&a.started_inside==b.started_inside&&a.fraction.raw()==b.fraction.raw());
    for(int k=0;k<3;++k)assert(a.point[k].raw()==b.point[k].raw()&&a.normal[k].raw()==b.normal[k].raw());
}
static void same(const epok::MoveResultT<Q>& a,const epok::MoveResultT<Q>& b){
    assert(a.entity==b.entity&&a.generation==b.generation&&a.grounded==b.grounded&&a.blocked==b.blocked&&a.unresolved_overlap==b.unresolved_overlap);
    for(int k=0;k<3;++k)assert(a.displacement[k].raw()==b.displacement[k].raw()&&a.normal[k].raw()==b.normal[k].raw());
}
static void same(const std::vector<epok::TriggerEvent>& a,const std::vector<epok::TriggerEvent>& b){
    assert(a.size()==b.size());for(size_t n=0;n<a.size();++n)
        assert(a[n].first==b[n].first&&a[n].second==b[n].second&&a[n].first_generation==b[n].first_generation&&a[n].second_generation==b[n].second_generation&&a[n].phase==b[n].phase);
}

static void differential_queries(){
    Fixture fixture;Indexed indexed;Linear linear;
    for(int translation:{0,37*4096,-41*4096}){
        const auto transform=matrix(translation,4096+translation/16,-translation);
        indexed.clear();linear.clear();
        indexed.set_cached(0,fixture.collider,transform,1,true,7);linear.set_cached(0,fixture.collider,transform,1,true,7);
        Collider trigger;trigger.enabled=trigger.trigger=true;trigger.layer=2;
        indexed.set(1,trigger,matrix(translation,4096,-translation),true,19);linear.set(1,trigger,matrix(translation,4096,-translation),true,19);
        for(unsigned n=0;n<3000;++n){
            Q origin[3]={raw(between(-40*4096,40*4096)+translation),raw(between(-2*4096,12*4096)+transform.values[1][3].raw()),raw(between(-40*4096,40*4096)-translation)};
            // Exact boundaries and one Q12 unit to either side, including the
            // maximum/outside cells, negative positions and zero-length rays.
            if(n%5==0){origin[0]=raw((int(n%11)-5)*8*4096+translation+int(n%3)-1);origin[2]=raw((int(n%9)-4)*8*4096-translation);}
            Q delta[3]={raw(between(-80*4096,80*4096)),raw(between(-8*4096,8*4096)),raw(between(-80*4096,80*4096))};
            if(n%7==0)for(auto& d:delta)d=Q(0.);
            Box box;for(int k=0;k<3;++k){const auto extent=raw(k==1?4096:between(1,6000));box.min[k]=origin[k]-extent;box.max[k]=origin[k]+extent;}
            const uint32_t mask=n%3==0?0:n%3==1?1:0xffffffffu;const int ignore=n%4==0?0:-1;
            same(indexed.raycast(origin,delta,mask,ignore,n%2),linear.raycast(origin,delta,mask,ignore,n%2));
            same(indexed.move_and_slide(box,delta,mask,ignore),linear.move_and_slide(box,delta,mask,ignore));
            const Q distance=raw(n%11==0?-1:between(0,12*4096));
            same(indexed.ground(box,distance,mask,ignore),linear.ground(box,distance,mask,ignore));
            uint16_t a[8]={},b[8]={};const unsigned capacity=n%9;
            assert(indexed.overlap(box,a,capacity,mask,ignore)==linear.overlap(box,b,capacity,mask,ignore));
            for(unsigned i=0;i<8;++i)assert(a[i]==b[i]);
        }
    }
}

static void cached_changes_and_trigger_lifecycle(){
    Fixture fixture;Indexed indexed;Linear linear;
    Collider trigger;trigger.enabled=trigger.trigger=true;
    std::vector<int16_t> replacement=fixture.boxes;
    for(unsigned tick=0;tick<80;++tick){
        const auto owner=matrix(int(tick%3)*4096,0,-int(tick%5)*4096);
        auto collider=fixture.collider;
        if(tick%4==1)collider.box_grid=nullptr;
        if(tick%4==2)collider.boxes=replacement.data(); // stale grid must fall back
        if(tick%4==3)--collider.box_count; // changed count must also fall back
        indexed.begin_sync();linear.begin_sync();
        indexed.set_cached(0,collider,owner,tick+1,tick%13!=0,tick/17);
        linear.set_cached(0,collider,owner,tick+1,tick%13!=0,tick/17);
        // Move into/out of floors and across cell boundaries, replace slots,
        // toggle layers/masks and disable triggers with a previous live pair.
        trigger.layer=tick%3==0?2:1;trigger.mask=tick%7==0?0:0xffffffffu;
        const auto at=matrix(int(tick%9)*4096-4*4096,int(tick%4)*1024,-int(tick%9)*4096);
        indexed.set_cached(1,trigger,at,tick+1,tick%11!=0,tick/19);
        linear.set_cached(1,trigger,at,tick+1,tick%11!=0,tick/19);
        std::vector<epok::TriggerEvent> a,b;
        indexed.update_triggers([&](const auto& e){a.push_back(e);});linear.update_triggers([&](const auto& e){b.push_back(e);});same(a,b);
        Q origin[3]={0.,6.,0.},delta[3]={0.,-12.,0.};same(indexed.raycast(origin,delta),linear.raycast(origin,delta));
    }
    // Grid metadata updates even when transform revision and bounds repeat.
    indexed.begin_sync();linear.begin_sync();
    indexed.set_cached(0,fixture.collider,matrix(),100);linear.set_cached(0,fixture.collider,matrix(),100);
    auto smaller=fixture.collider;--smaller.box_count;
    indexed.begin_sync();linear.begin_sync();
    assert(!indexed.set_cached(0,smaller,matrix(),100));assert(!linear.set_cached(0,smaller,matrix(),100));
    Q p[3]={0.,4.,0.},d[3]={32.,-8.,-32.};same(indexed.raycast(p,d),linear.raycast(p,d));
}

int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    differential_queries();cached_changes_and_trigger_lifecycle();
    std::puts("Compound grid: 9,000 indexed/linear query sets and trigger lifecycle match exactly.");
}
