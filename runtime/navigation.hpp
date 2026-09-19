#pragma once
#include <stdint.h>
#include <stddef.h>

// NavLite: immutable Q8 graph, one shared incremental search, fixed request pool.
// A work unit is ONE projection candidate, clear, BFS expansion (<=4 edges), or
// path reconstruction node. No whole-graph scans or allocations hidden in tick.
namespace epok::nav {
constexpr uint16_t invalid=0xffff, max_nodes=512, max_requests=8, max_path=64;
struct Node {int16_t x,y,z; uint16_t links[4];};
static_assert(sizeof(Node)==14);
struct Graph {const Node* nodes=nullptr;uint16_t count=0;uint16_t snap=192;};
enum class Status:uint8_t {Idle,Queued,Searching,Ready,NoPath,TooLong,OffGraph,Blocked,Arrived};
struct Handle {uint16_t slot=invalid,generation=0;};
struct Request {
    uint16_t generation=0;bool used=false;Status status=Status::Idle;
    int32_t from[3]{},to[3]{};
    uint16_t path[max_path]{},length=0;
};
struct Stats {uint32_t work=0,expanded=0,completed=0,rejected=0;};
class World {
public:
    Graph graph{};Stats stats{};
    Request requests[max_requests]{};
    // Intentionally public budget, shared by ALL agents; set once per frame.
    uint16_t budget=64;
    void reset(Graph value) {
        graph=value.count<=max_nodes?value:Graph{};active=invalid;cursor=0;
        for(auto& r:requests){r.used=false;++r.generation;}
        stats={};
    }
    Request* get(Handle h) {
        if(h.slot>=max_requests)return nullptr;
        auto& r=requests[h.slot];return r.used&&r.generation==h.generation?&r:nullptr;
    }
    void cancel(Handle h) {if(auto* r=get(h)){r->used=false;if(active==h.slot)active=invalid;}}
    Handle request(const int32_t* from,const int32_t* to) {
        for(uint16_t i=0;i<max_requests;++i)if(!requests[i].used){
            auto& r=requests[i];r.used=true;++r.generation;r.status=Status::Queued;r.length=0;
            for(int k=0;k<3;++k){r.from[k]=from[k];r.to[k]=to[k];}
            return {i,r.generation};
        }
        ++stats.rejected;return {};
    }
    void tick() {
        stats.work=stats.expanded=0;
        for(uint16_t work=0;work<budget;++work){
            if(active==invalid){
                for(uint16_t i=0;i<max_requests;++i){const auto n=uint16_t((cursor+i)%max_requests);
                    if(requests[n].used&&requests[n].status==Status::Queued){active=n;cursor=(n+1)%max_requests;break;}}
                if(active==invalid)break;
                auto& r=requests[active];r.status=Status::Searching;
                scan=0;start=goal=invalid;best_start=best_goal=0x7fffffff;phase=0;head=tail=0;
                if(!graph.nodes||!graph.count){finish(Status::OffGraph);continue;}
            }
            ++stats.work;
            auto& r=requests[active];
            if(phase==0){
                const auto& n=graph.nodes[scan];parent[scan]=invalid;
                const int32_t a=best_start?distance(n,r.from):0,b=best_goal?distance(n,r.to):0;
                if(a<best_start){best_start=a;start=scan;}if(b<best_goal){best_goal=b;goal=scan;}
                if(++scan==graph.count){
                    if(best_start>graph.snap||best_goal>graph.snap){finish(Status::OffGraph);continue;}
                    queue[tail++]=start;parent[start]=start;phase=1;
                }
            }else if(phase==1){
                if(head==tail){finish(Status::NoPath);continue;}
                const auto n=queue[head++];++stats.expanded;
                if(n==goal){trace=goal;phase=2;continue;}
                for(const auto next:graph.nodes[n].links)if(next<graph.count&&parent[next]==invalid){parent[next]=n;queue[tail++]=next;}
            }else{
                if(r.length==max_path){finish(Status::TooLong);continue;}
                r.path[r.length++]=trace;
                if(trace==start){finish(Status::Ready);continue;}
                trace=parent[trace];
            }
        }
    }
private:
    uint16_t parent[max_nodes]{},queue[max_nodes]{};
    uint16_t active=invalid,cursor=0,scan=0,start=invalid,goal=invalid,head=0,tail=0,trace=0;
    uint8_t phase=0;int32_t best_start=0,best_goal=0;
    static int32_t distance(const Node& n,const int32_t* p){
        // L1 bound, no 64-bit products/divides on R3000A. Callers use Q8.
        const int32_t x=int32_t(n.x)-p[0],y=int32_t(n.y)-p[1],z=int32_t(n.z)-p[2];
        return (x<0?-x:x)+(y<0?-y:y)+(z<0?-z:z);
    }
    void finish(Status value){requests[active].status=value;++stats.completed;active=invalid;}
};
// One fixed workspace shared by all agents and reused on scene transitions.
inline World world;
}
