#include "../../runtime/navigation.hpp"
#include <cassert>
#include <cstdio>
using namespace epok::nav;
static void complete(World& w,Handle h){for(int i=0;i<4096;++i){w.tick();assert(w.stats.work<=w.budget);auto* r=w.get(h);if(r&&r->status!=Status::Queued&&r->status!=Status::Searching)return;}assert(false);}
static void dynamic_and_crowd(){
    World w;Node n[4]={{0,0,0,{1,2,invalid,invalid,invalid,invalid}},{256,0,0,{0,3,invalid,invalid,invalid,invalid}},
        {0,0,256,{0,3,invalid,invalid,invalid,invalid}},{256,0,256,{1,2,invalid,invalid,invalid,invalid}}};
    w.reset({n,4,256});w.budget=3;int32_t a[]={0,0,0},b[]={256,0,256};
    auto h=w.request(a,b);complete(w,h);assert(w.get(h)->length==3);
    auto door=w.add_obstacle();int32_t lo[]={100,0,-80},hi[]={180,200,80};assert(w.set_obstacle(door,lo,hi));
    for(int i=0;i<20;++i){w.tick();assert(w.stats.work<=w.budget);}w.replan(h);complete(w,h);
    assert(!w.edge_open(0,1)&&w.edge_open(0,2));assert(w.get(h)->path[1]==2);
    w.remove_obstacle(door);for(int i=0;i<20;++i)w.tick();assert(w.edge_open(0,1));
    int32_t c[]={256,0,0};auto other=w.request(c,a);complete(w,other);w.occupy(other,1);assert(!w.reserve(h,0,1));
    w.cancel(other);assert(w.reserve(h,0,1));
    // Replacing the entire graph preserves handles/goals but discards paths.
    auto old=h;assert(w.rebuild({n,4,256}));assert(w.get(old)&&w.get(old)->status==Status::Queued);complete(w,h);assert(w.get(h)->status==Status::Ready);
    Handle slots[8];for(auto& s:slots)s={};
    uint16_t obstacles[16];for(auto& o:obstacles){o=w.add_obstacle();assert(o!=invalid);}assert(w.add_obstacle()==invalid);
    for(auto o:obstacles)w.remove_obstacle(o);
    // Continuous movement must complete rebuild snapshots within a bound.
    door=w.add_obstacle();const auto before=w.stats.rebuilt;
    for(int i=0;i<100;++i){lo[0]=1000+i;hi[0]=1100+i;w.set_obstacle(door,lo,hi);w.tick();assert(w.stats.work<=w.budget);}assert(w.stats.rebuilt>before+5);
    w.cancel(h);h=w.request(a,b);w.budget=1;
    for(int i=0;i<300&&w.get(h)->status!=Status::Ready;++i){lo[0]=1000+i;hi[0]=1100+i;w.set_obstacle(door,lo,hi);w.tick();assert(w.stats.work<=1);}assert(w.get(h)->status==Status::Ready);
}
static void surfaces(){
    World w;Surface t[]={{{{0,0,0},{0,0,512},{512,256,512}}},{{{0,0,0},{512,256,512},{512,256,0}}}};
    Graph g;g.surfaces=t;g.surface_count=2;w.reset(g);int32_t y;
    assert(w.floor(256,256,0,512,y)&&y==128);assert(!w.floor(600,0,0,512,y));assert(!w.floor(256,256,0,64,y));
}
int main(){
    dynamic_and_crowd();surfaces();
    World w;Node nodes[100]{};
    for(int i=0;i<100;++i){nodes[i].x=int16_t(i*128);for(auto& l:nodes[i].links)l=invalid;if(i)nodes[i].links[0]=i-1;if(i<99)nodes[i].links[1]=i+1;}
    w.reset({nodes,100,128});w.budget=1;
    int32_t a[]={0,0,0},b[]={10*128,0,0};auto h=w.request(a,b);complete(w,h);assert(w.get(h)->status==Status::Ready);assert(w.get(h)->length==11);
    w.cancel(h);assert(!w.get(h));auto newer=w.request(a,b);assert(!w.get(h));w.cancel(newer);
    nodes[5].links[1]=invalid;h=w.request(a,b);complete(w,h);assert(w.get(h)->status==Status::NoPath);w.cancel(h);nodes[5].links[1]=6;
    b[0]=99*128;h=w.request(a,b);complete(w,h);assert(w.get(h)->status==Status::TooLong);w.cancel(h);
    b[0]=200*128;h=w.request(a,b);complete(w,h);assert(w.get(h)->status==Status::OffGraph);w.cancel(h);
    b[0]=128;Handle jobs[8];for(auto& j:jobs){j=w.request(a,b);assert(w.get(j));}assert(!w.get(w.request(a,b)));
    for(auto j:jobs){complete(w,j);assert(w.get(j)->status==Status::Ready);}
    w.reset({});for(auto j:jobs)assert(!w.get(j));h=w.request(a,b);complete(w,h);assert(w.get(h)->status==Status::OffGraph);
    std::printf("NavLite OK: bounded work, route, no path, off graph, capacity, cancellation, reset. World=%zu bytes, Node=%zu bytes\n",sizeof(World),sizeof(Node));
}
