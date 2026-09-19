#include "../../runtime/navigation.hpp"
#include <cassert>
#include <cstdio>
using namespace epok::nav;
static void complete(World& w,Handle h){for(int i=0;i<4096;++i){w.tick();assert(w.stats.work<=w.budget);auto* r=w.get(h);if(r&&r->status!=Status::Queued&&r->status!=Status::Searching)return;}assert(false);}
int main(){
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
