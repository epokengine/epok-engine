#pragma once
#include <stdint.h>
#include <stddef.h>

// NavLite: Q8 surfaces/graph, incremental obstacle rebuild and shared search.
namespace epok::nav {
constexpr uint16_t invalid=0xffff, max_nodes=512, max_requests=8, max_path=64;
struct Node {int16_t x,y,z; uint16_t links[6];};
static_assert(sizeof(Node)==18);
struct Surface {int32_t p[3][3];};
struct Traversal {uint16_t from,to;uint8_t kind;int32_t arc,duration;};
struct Graph {
    const Node* nodes=nullptr;uint16_t count=0,snap=192;
    const Surface* surfaces=nullptr;uint16_t surface_count=0;
    const Traversal* traversals=nullptr;uint16_t traversal_count=0;
    uint16_t radius=46,height=128,step=102;
};
enum class Status:uint8_t {Idle,Queued,Searching,Ready,NoPath,TooLong,OffGraph,Blocked,Arrived};
struct Handle {uint16_t slot=invalid,generation=0;};
struct Request {
    uint16_t generation=0;bool used=false;Status status=Status::Idle;
    int32_t from[3]{},to[3]{};
    uint16_t path[max_path]{},length=0;
    uint16_t occupied=invalid,reserved=invalid,avoid=invalid;
    uint32_t revision=0;
};
struct Stats {uint32_t work=0,expanded=0,completed=0,rejected=0,rebuilt=0,yields=0;};
class World {
public:
    Graph graph{};Stats stats{};
    Request requests[max_requests]{};
    // Intentionally public budget, shared by ALL agents; set once per frame.
    uint16_t budget=64;
    uint32_t revision=0;
    void reset(Graph value) {
        graph=value.count<=max_nodes?value:Graph{};active=invalid;cursor=0;
        for(auto& r:requests){r.used=false;++r.generation;}
        for(auto& o:obstacles)o={};for(auto& o:rebuild_obstacles)o={};for(auto& set:blocked)for(auto& b:set)b=0;
        front=0;rebuilding=dirty_again=false;rebuild_cursor=0;++revision;stats={};
    }
    // Replace the current topology (procedural levels/streaming). No old paths
    // survive; goals and generation-checked request handles remain valid.
    bool rebuild(Graph value) {
        if(value.count>max_nodes||(!value.nodes&&value.count)||value.surface_count>4096||value.traversal_count>32)return false;
        graph=value;active=invalid;++revision;rebuilding=dirty_again=false;
        for(auto& set:blocked)for(auto& b:set)b=0;
        for(auto& r:requests)if(r.used){r.status=Status::Queued;r.length=0;r.occupied=r.reserved=invalid;r.revision=revision;}
        dirty();return true;
    }
    struct Obstacle {bool used=false;int32_t min[3]{},max[3]{};};
    static constexpr uint16_t max_obstacles=16;
    uint16_t add_obstacle(){for(uint16_t i=0;i<max_obstacles;++i)if(!obstacles[i].used){obstacles[i].used=true;return i;}++stats.rejected;return invalid;}
    bool set_obstacle(uint16_t id,const int32_t* lo,const int32_t* hi){
        if(id>=max_obstacles||!obstacles[id].used)return false;
        bool changed=false;for(int k=0;k<3;++k){changed|=obstacles[id].min[k]!=lo[k]||obstacles[id].max[k]!=hi[k];obstacles[id].min[k]=lo[k];obstacles[id].max[k]=hi[k];}
        if(changed)dirty();return true;
    }
    void remove_obstacle(uint16_t id){if(id<max_obstacles&&obstacles[id].used){obstacles[id]={};dirty();}}
    bool updating()const{return rebuilding;}
    bool edge_open(uint16_t from,uint16_t to)const{
        if(from>=graph.count||to>=graph.count)return false;
        for(int k=0;k<6;++k)if(graph.nodes[from].links[k]==to)return !(blocked[front][from]&(1u<<k))&&!obstructed(from,to,obstacles);return false;
    }
    const Traversal* traversal(uint16_t from,uint16_t to)const{
        for(uint16_t i=0;i<graph.traversal_count;++i)if(graph.traversals[i].from==from&&graph.traversals[i].to==to)return &graph.traversals[i];return nullptr;
    }
    void locate(Handle h,const int32_t* p){if(auto* r=get(h))for(int k=0;k<3;++k)r->from[k]=p[k];}
    void replan(Handle h,uint16_t avoid=invalid){if(auto* r=get(h)){r->status=Status::Queued;r->length=0;r->reserved=invalid;r->avoid=avoid;if(active==h.slot)active=invalid;}}
    bool reserve(Handle h,uint16_t from,uint16_t to){
        auto* r=get(h);if(!r)return false;r->occupied=from;
        if(to>=graph.count)return false;const auto& dest=graph.nodes[to];
        for(uint16_t i=0;i<max_requests;++i){auto& other=requests[i];if(i==h.slot||!other.used)continue;
            bool collision=other.occupied==to||other.reserved==to;
            const uint16_t occupied[]={other.occupied,other.reserved};for(auto index:occupied)if(index<graph.count){const auto& n=graph.nodes[index];
                collision|=abs(int32_t(n.x)-dest.x)<int32_t(graph.radius)*2&&abs(int32_t(n.z)-dest.z)<int32_t(graph.radius)*2&&abs(int32_t(n.y)-dest.y)<graph.height;
            }
            if(collision){++stats.yields;return false;}
        }
        r->reserved=to;return true;
    }
    void occupy(Handle h,uint16_t node){if(auto* r=get(h)){r->occupied=node;r->reserved=invalid;}}
    // Barycentric floor query in Q8; the closest reachable layer wins. Geometry
    // is bounded at bake time. Caller can restrict to a local surface below.
    bool floor(int32_t x,int32_t z,int32_t near,int32_t reach,int32_t& y)const{
        int32_t best=INT32_MAX;
        for(uint16_t i=0;i<graph.surface_count;++i){const auto& s=graph.surfaces[i];const auto* a=s.p[0];const auto* b=s.p[1];const auto* c=s.p[2];
            const int32_t minx=a[0]<b[0]?(a[0]<c[0]?a[0]:c[0]):(b[0]<c[0]?b[0]:c[0]);
            const int32_t maxx=a[0]>b[0]?(a[0]>c[0]?a[0]:c[0]):(b[0]>c[0]?b[0]:c[0]);
            const int32_t minz=a[2]<b[2]?(a[2]<c[2]?a[2]:c[2]):(b[2]<c[2]?b[2]:c[2]);
            const int32_t maxz=a[2]>b[2]?(a[2]>c[2]?a[2]:c[2]):(b[2]>c[2]?b[2]:c[2]);
            if(x<minx||x>maxx||z<minz||z>maxz)continue;
            int64_t d=int64_t(b[2]-c[2])*(a[0]-c[0])+int64_t(c[0]-b[0])*(a[2]-c[2]);if(!d)continue;
            int64_t u=int64_t(b[2]-c[2])*(x-c[0])+int64_t(c[0]-b[0])*(z-c[2]);
            int64_t v=int64_t(c[2]-a[2])*(x-c[0])+int64_t(a[0]-c[0])*(z-c[2]);
            if(d<0){d=-d;u=-u;v=-v;}if(u<0||v<0||u+v>d)continue;
            const int32_t height=int32_t((u*a[1]+v*b[1]+(d-u-v)*c[1])/d),diff=height>near?height-near:near-height;
            if(diff<=reach&&(best==INT32_MAX||height>y)){best=diff;y=height;}
        }return best!=INT32_MAX;
    }
    Request* get(Handle h) {
        if(h.slot>=max_requests)return nullptr;
        auto& r=requests[h.slot];return r.used&&r.generation==h.generation?&r:nullptr;
    }
    void cancel(Handle h) {if(auto* r=get(h)){r->used=false;if(active==h.slot)active=invalid;}}
    Handle request(const int32_t* from,const int32_t* to) {
        for(uint16_t i=0;i<max_requests;++i)if(!requests[i].used){
            auto& r=requests[i];r.used=true;++r.generation;r.status=Status::Queued;r.length=0;r.occupied=r.reserved=r.avoid=invalid;r.revision=revision;
            for(int k=0;k<3;++k){r.from[k]=from[k];r.to[k]=to[k];}
            return {i,r.generation};
        }
        ++stats.rejected;return {};
    }
    void tick() {
        stats.work=stats.expanded=0;
        for(uint16_t work=0;work<budget;++work){
            if(rebuilding&&((work_serial++&1u)==0)){
                ++stats.work;
                if(rebuild_cursor<graph.count){const auto i=rebuild_cursor++;blocked[front^1][i]=0;
                    for(int k=0;k<6;++k){const auto j=graph.nodes[i].links[k];if(j<graph.count&&obstructed(i,j,rebuild_obstacles))blocked[front^1][i]|=uint8_t(1u<<k);}
                    continue;
                }
                rebuilding=false;front^=1;++revision;++stats.rebuilt;
                for(auto& r:requests)if(r.used){
                    // Following agents validate their current edge; do not scan
                    // eight entire paths in this single rebuild work unit.
                    if(r.status==Status::NoPath){r.status=Status::Queued;r.length=0;r.reserved=invalid;r.revision=revision;}
                }
                // Finish each snapshot even if an obstacle moves every frame.
                // Queries continue between rebuilds and movement also checks
                // the live obstacle set. A moving door cannot starve searches.
                if(dirty_again){dirty_again=false;dirty();}continue;
            }
            if(active==invalid){
                for(uint16_t i=0;i<max_requests;++i){const auto n=uint16_t((cursor+i)%max_requests);
                    if(requests[n].used&&requests[n].status==Status::Queued){active=n;cursor=(n+1)%max_requests;break;}}
                if(active==invalid){if(rebuilding)continue;break;}
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
                for(int k=0;k<6;++k){const auto next=graph.nodes[n].links[k];if(next<graph.count&&next!=r.avoid&&!(blocked[front][n]&(1u<<k))&&parent[next]==invalid){parent[next]=n;queue[tail++]=next;}}
            }else{
                if(r.length==max_path){finish(Status::TooLong);continue;}
                r.path[r.length++]=trace;
                if(trace==start){finish(Status::Ready);continue;}
                if(!edge_open(parent[trace],trace)){r.length=0;r.status=Status::Queued;active=invalid;continue;}
                trace=parent[trace];
            }
        }
    }
private:
    Obstacle obstacles[max_obstacles]{},rebuild_obstacles[max_obstacles]{};uint8_t blocked[2][max_nodes]{};uint8_t front=0;
    bool rebuilding=false,dirty_again=false;uint16_t rebuild_cursor=0;uint32_t work_serial=0;
    static int32_t abs(int32_t v){return v<0?-v:v;}
    void dirty(){if(rebuilding){dirty_again=true;return;}rebuilding=true;rebuild_cursor=0;for(uint16_t i=0;i<max_obstacles;++i)rebuild_obstacles[i]=obstacles[i];}
    bool obstructed(uint16_t from,uint16_t to,const Obstacle* values)const{
        const auto& a=graph.nodes[from];const auto& b=graph.nodes[to];
        int32_t lo[]={a.x<b.x?a.x:b.x,a.y<b.y?a.y:b.y,a.z<b.z?a.z:b.z};
        int32_t hi[]={a.x>b.x?a.x:b.x,a.y>b.y?a.y:b.y,a.z>b.z?a.z:b.z};
        lo[0]-=graph.radius;lo[2]-=graph.radius;lo[1]+=1;hi[0]+=graph.radius;hi[2]+=graph.radius;hi[1]+=graph.height;
        if(const auto* t=traversal(from,to))hi[1]+=t->kind==1?t->arc:0;
        for(uint16_t i=0;i<max_obstacles;++i){const auto& o=values[i];if(o.used){bool hit=true;for(int k=0;k<3;++k)hit&=hi[k]>o.min[k]&&lo[k]<o.max[k];if(hit)return true;}}return false;
    }
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
