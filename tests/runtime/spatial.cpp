#include <cassert>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <vector>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/input.hpp"
#include "../../runtime/time.hpp"
#include "../../runtime/collision.hpp"
#include "psyqo/fixed-point.hh"

using Q12=psyqo::FixedPoint<12>;
static Q12 q(double value){return Q12(int32_t(std::round(value*4096)),Q12::RAW);}
using World=epok::CollisionWorld<Q12,16,4>;
using Box=epok::AabbT<Q12>;
using Collider=epok::ColliderT<Q12>;
static epok::Affine<Q12> matrix(double x=0,double y=0,double z=0) {
    auto m=epok::Affine<Q12>::identity();m.values[0][3]=q(x);m.values[1][3]=q(y);m.values[2][3]=q(z);return m;
}
static Collider collider(double x=.5,double y=.5,double z=.5,bool trigger=false) {
    Collider c;c.enabled=true;c.trigger=trigger;c.half_extents[0]=q(x);c.half_extents[1]=q(y);c.half_extents[2]=q(z);return c;
}
static double real(Q12 value){return value.raw()/4096.0;}
static Box box(double x,double y,double z) {return epok::collider_bounds(collider(),matrix(x,y,z));}
static void input_and_clock() {
    epok::Input input;
    input.sample(0,true,1<<14);input.sample(0,true,0);
    input.begin_tick();assert(input.pressed(epok::Button::Cross));assert(input.released(epok::Button::Cross));assert(!input.held(epok::Button::Cross));
    input.end_tick();input.begin_tick();assert(!input.pressed(epok::Button::Cross));assert(!input.released(epok::Button::Cross));
    input.sample(1,true,1<<3);input.begin_tick();assert(input.held(epok::Button::Start,1));
    input.sample(1,false,0xffff);input.begin_tick();assert(input.released(epok::Button::Start,1));assert(!input.connected(1));
    assert(!input.held(epok::Button::Cross,2));
    epok::Time time;time.reset(0);assert(time.advance(16000)==0);assert(time.advance(16667)==1);assert(time.advance(33334)==1);
    assert(time.advance(1000000)==8);assert(time.dropped_steps==50);
    time.set_paused(true);assert(time.advance(2000000)==0);time.set_paused(false);assert(time.advance(2016667)==1);
    time.reset(0xfffffff0);assert(time.advance(0x0000410a)==0);assert(time.advance(0x0000410b)==1);
    int total=0;for(int i=0;i<60;++i){time.begin_tick();total+=time.delta_raw;}assert(total==4096);
    // Rendering cadence must not change elapsed simulation time or distance.
    // Exercise the exact production Q12 clock and the controller's speed*dt.
    for(unsigned fps: {15u,30u,34u,47u,60u,120u}){
        epok::Time clock;clock.reset(0);Q12 position=0.;
        for(unsigned frame=1;frame<=fps*10;++frame){
            unsigned steps=clock.advance(uint32_t(uint64_t(frame)*1000000/fps));
            for(unsigned step=0;step<steps;++step){clock.begin_tick();position+=Q12(4.2)*Q12(clock.delta_raw,Q12::RAW);}
        }
        assert(clock.ticks==600&&clock.dropped_steps==0);
        assert(std::abs(real(position)-42.)<.15); // Q12 multiplication rounds each tick.
        static int32_t reference=position.raw();assert(position.raw()==reference);
    }
}
static void rays_and_ground() {
    World world;world.set(0,collider(),matrix());
    Q12 origin[3]={-2.,0.,0.},delta[3]={4.,0.,0.};
    auto hit=world.raycast(origin,delta);assert(hit.entity==0);assert(real(hit.point[0])==-.5);assert(real(hit.normal[0])==-1.);assert(!hit.started_inside);
    origin[0]=2.;delta[0]=-4.;hit=world.raycast(origin,delta);assert(real(hit.normal[0])==1.);
    origin[0]=0.;delta[0]=0.;hit=world.raycast(origin,delta);assert(hit&&hit.started_inside&&real(hit.fraction)==0.);
    origin[1]=2.;assert(!world.raycast(origin,delta));
    world.set(1,collider(2.,.25,2.),matrix(0.,-2.,0.));
    hit=world.ground(box(0.,0.,0.),3.,0xffffffffu,0);assert(hit.entity==1);assert(real(hit.point[1])==-1.75);assert(real(hit.normal[1])==1.);
    assert(!world.ground(box(0.,0.,0.),1.,0xffffffffu,0));
    hit=world.ground(box(2.25,0.,0.),3.,0xffffffffu,0);assert(hit.entity==1); // footprint catches edge
    uint16_t matches[1]={99};assert(world.overlap(box(0.,0.,0.),matches,1)==1&&matches[0]==0);
    assert(world.overlap(box(0.,0.,0.),nullptr,0)==1);
}
static void sweeps() {
    { // Conservative broad phase retains contacts exactly at the endpoint.
        World w;w.set(0,collider(),matrix(2.,0.,0.));Q12 delta[3]={1.,0.,0.};
        auto result=w.move_and_slide(box(0.,0.,0.),delta);assert(result.blocked&&result.entity==0);
        Q12 origin[3]={0.,0.,0.};delta[0]=1.5;assert(w.raycast(origin,delta).entity==0);
        auto bounds=epok::swept_bounds(box(0.,0.,0.),delta);assert(real(bounds.max[0])==2.);
        delta[0]=Q12(INT32_MAX,Q12::RAW);bounds=epok::swept_bounds(box(1.,0.,0.),delta);assert(bounds.max[0].raw()==INT32_MAX);
    }
    World world;world.set(0,collider(.01,10.,10.),matrix(2.,0.,0.));
    Q12 delta[3]={50.,0.,0.};auto result=world.move_and_slide(box(0.,0.,0.),delta);
    assert(result.blocked);assert(real(result.displacement[0])<1.491);assert(real(result.displacement[0])>1.48);
    delta[1]=3.;result=world.move_and_slide(box(0.,0.,0.),delta);
    assert(real(result.displacement[1])==3.);assert(real(result.normal[0])==-1.);
    delta[0]=-50.;result=world.move_and_slide(box(4.,0.,0.),delta);assert(result.blocked);assert(real(result.displacement[0])>-1.491);
    // A standing box slides horizontally on a touching floor.
    world.clear();world.set(0,collider(10.,.5,10.),matrix(0.,-.5,0.));
    delta[0]=3.;delta[1]=0.;result=world.move_and_slide(box(0.,.5,0.),delta);assert(real(result.displacement[0])==3.);
    delta[0]=0.;delta[1]=-20.;result=world.move_and_slide(box(0.,2.,0.),delta);assert(result.grounded);assert(real(result.displacement[1])>=-1.5);
    // Touching a wall while travelling away must not produce a collision.
    world.clear();world.set(0,collider(),matrix(1.,0.,0.));
    delta[0]=-3.;delta[1]=0.;result=world.move_and_slide(box(0.,0.,0.),delta);assert(!result.blocked&&real(result.displacement[0])==-3.);
    delta[0]=0.;result=world.move_and_slide(box(.2,0.,0.),delta);assert(result.blocked&&!result.unresolved_overlap);assert(real(result.displacement[0])<-.19);
    // Triggers and excluded layers cannot block a mover.
    world.clear();auto c=collider();c.trigger=true;world.set(0,c,matrix(2.,0.,0.));c.trigger=false;c.layer=2;world.set(1,c,matrix(3.,0.,0.));
    delta[0]=10.;result=world.move_and_slide(box(0.,0.,0.),delta,1);assert(!result.blocked&&real(result.displacement[0])==10.);
    // Two perpendicular planes remove the blocked components while preserving Z.
    world.clear();world.set(0,collider(.1,20.,20.),matrix(2.,0.,0.));world.set(1,collider(20.,.1,20.),matrix(0.,2.,0.));
    delta[0]=10.;delta[1]=10.;delta[2]=3.;result=world.move_and_slide(box(0.,0.,0.),delta);assert(real(result.displacement[0])<1.401&&real(result.displacement[1])<1.401&&real(result.displacement[2])==3.);
}
static void ramps() {
    // A ramp four units long rising two, low edge at x = -2, sitting on y = 0.
    World world;auto slope=collider(2.,1.,2.);slope.slope_rise=2.;slope.slope_axis=0;
    world.set(0,slope,matrix(0.,0.,0.));
    // Surface height follows the character across the run, not the box top.
    auto hit=world.ground(box(-2.,4.,0.),8.,0xffffffffu,-1);
    assert(hit.entity==0&&real(hit.point[1])==-1.);
    hit=world.ground(box(0.,4.,0.),8.,0xffffffffu,-1);
    assert(hit.entity==0&&real(hit.point[1])==0.);
    hit=world.ground(box(2.,4.,0.),8.,0xffffffffu,-1);
    assert(hit.entity==0&&real(hit.point[1])==1.);
    // Walking into the low end climbs instead of stopping: a ramp is a floor.
    Q12 delta[3]={3.,0.,0.};
    auto result=world.move_and_slide(box(-2.,-1.,0.),delta);
    assert(!result.blocked||result.grounded);
    assert(real(result.displacement[0])==3.);
    assert(result.displacement[1]>Q12(0.));
    // Past the top edge the surface clamps to the box rather than extrapolating.
    hit=world.ground(box(1.9,4.,0.),8.,0xffffffffu,-1);
    assert(hit.entity==0&&real(hit.point[1])<=1.);
    // Zero rise leaves an ordinary box that still blocks sideways.
    world.clear();world.set(0,collider(2.,1.,2.),matrix(0.,0.,0.));
    result=world.move_and_slide(box(-4.,0.,0.),delta);assert(result.blocked);
}
static void triggers_and_transform() {
    World world;auto c=collider();c.trigger=true;world.set(0,c,matrix(),true,7);world.set(1,collider(),matrix(),true,8);
    std::vector<epok::TriggerEvent> events;auto callback=[&](auto event){events.push_back(event);};
    world.update_triggers(callback);assert(events.size()==1&&events[0].phase==epok::TriggerPhase::Enter);
    events.clear();world.update_triggers(callback);assert(events.size()==1&&events[0].phase==epok::TriggerPhase::Stay);
    events.clear();world.set(1,collider(),matrix(),true,9);world.update_triggers(callback);assert(events.size()==2&&events[0].phase==epok::TriggerPhase::Enter&&events[1].phase==epok::TriggerPhase::Exit);
    events.clear();world.begin_sync();world.set(0,c,matrix(),true,7);world.update_triggers(callback);assert(events.size()==1&&events[0].phase==epok::TriggerPhase::Exit);
    events.clear();world.clear();for(int i=0;i<5;++i)world.set(i,c,matrix());world.update_triggers(callback);assert(events.size()==4&&world.dropped_trigger_pairs==6);
    auto m=matrix(2.,3.,4.);m.values[0][1]=2.;m.values[1][1]=3.;auto bounds=epok::collider_bounds(collider(),m);assert(real(bounds.min[0])==.5&&real(bounds.max[1])==4.5);
}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    input_and_clock();rays_and_ground();sweeps();ramps();triggers_and_transform();std::puts("Runtime input, clock, collision and trigger tests passed.");}
