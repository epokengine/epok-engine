#include <cassert>
#include <cstdio>
#include "../../runtime/epok.hpp"
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
namespace epok {
void AudioSource::stop(){}
ActorData* DataHandle::get()const{return nullptr;}
DataHandle handle(const ActorData*){return {};}
bool is_active(const ActorData*){return true;}
#define ABSTRACT(T,P,F,D) {T::static_class_id,P,ObjectFamily::F,ObjectDomain::D,0,ObjectClassAbstract,nullptr,nullptr,sizeof(T),alignof(T),nullptr,nullptr}
#define ROW(T,P,F,D,OWN,FLAGS) {T::static_class_id,P,ObjectFamily::F,ObjectDomain::D,OWN,FLAGS,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,8>::acquire,&ObjectPool<T,8>::release}
const ClassDescriptor object_classes[]={
    ABSTRACT(Object,0,Object,None),ABSTRACT(Actor,Object::static_class_id,Actor,None),
    ROW(Actor3D,Actor::static_class_id,Actor,World3D,0,6),
    ABSTRACT(ActorComponent,Object::static_class_id,Component,None),
    ROW(SceneComponent3D,ActorComponent::static_class_id,Component,World3D,1,16),
    ROW(NavigationAgentComponent,ActorComponent::static_class_id,Component,World3D,1,32),
    ABSTRACT(Level,Object::static_class_id,Level,None)
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
using namespace epok;
static CollisionWorld<Fixed,8> physics;
static MoveResult move(ActorData& a,const Fixed* d,uint32_t mask){
    auto matrix=Affine<Fixed>::identity();for(int k=0;k<3;++k)matrix.values[k][3]=a.transform.position[k];
    auto box=collider_bounds(a.collider,matrix);auto hit=physics.move_and_slide(box,d,mask);
    for(int k=0;k<3;++k)a.transform.position[k]+=hit.displacement[k];return hit;
}
static void solid(int slot,double x,double y,double z,double hx,double hy,double hz,double rise=0.){
    Collider c;c.enabled=true;c.half_extents[0]=Fixed(int32_t(hx*4096),Fixed::RAW);c.half_extents[1]=Fixed(int32_t(hy*4096),Fixed::RAW);c.half_extents[2]=Fixed(int32_t(hz*4096),Fixed::RAW);c.slope_rise=Fixed(int32_t(rise*4096),Fixed::RAW);
    auto m=Affine<Fixed>::identity();m.values[0][3]=Fixed(int32_t(x*4096),Fixed::RAW);m.values[1][3]=Fixed(int32_t(y*4096),Fixed::RAW);m.values[2][3]=Fixed(int32_t(z*4096),Fixed::RAW);physics.set(slot,c,m);
}
static void run(NavigationAgentComponent& agent,ActorData& data,Fixed x,Fixed y,Fixed z){
    assert(agent.move_to(x,y,z));
    for(int frame=0;frame<3000&&!agent.arrived()&&!agent.failed();++frame){nav::world.tick();agent.tick(Fixed(1./60.));}
    if(!agent.arrived())std::fprintf(stderr,"status %u at %.3f %.3f %.3f\n",agent.status(),data.transform.position[0].raw()/4096.,data.transform.position[1].raw()/4096.,data.transform.position[2].raw()/4096.);
    assert(agent.arrived());assert((data.transform.position[0]-x).raw()<64&&(data.transform.position[0]-x).raw()>-64);
}
int main(){
#ifdef _MSC_VER
    _CrtSetReportMode(_CRT_ASSERT,_CRTDBG_MODE_FILE);_CrtSetReportFile(_CRT_ASSERT,_CRTDBG_FILE_STDERR);
#endif
    ObjectRegistryStorage<32> storage;active_object_registry=&storage;Level level;assert(level.bind(storage));
    ActorSpawnRequest request;request.type=find_object_class(Actor3D::static_class_id);request.name="walker";ObjectId id;assert(level.spawn_batch(&request,1,&id));
    auto* actor=storage.resolve<Actor3D>(id);ActorData data{};data.parent=-1;data.alive=data.active=true;data.collider.enabled=true;
    data.collider.center[1]=0.25;data.collider.half_extents[0]=data.collider.half_extents[2]=0.16;data.collider.half_extents[1]=0.25;actor->bind_data(data);
    auto* agent=level.add_component<NavigationAgentComponent>(*actor,"nav");assert(agent);agent->speed=2.0;nav::move_actor=move;
    nav::Node n[5];for(int i=0;i<5;++i){n[i].x=int16_t(i*128);n[i].y=int16_t(i*64);n[i].z=0;for(auto& j:n[i].links)j=nav::invalid;if(i)n[i].links[0]=i-1;if(i<4)n[i].links[1]=i+1;}
    nav::Surface ramp[]={{{{-128,-64,-512},{-128,-64,512},{640,320,512}}},{{{-128,-64,-512},{640,320,512},{640,320,-512}}}};
    nav::Graph g{n,5,128,ramp,2};nav::world.reset(g);physics.clear();solid(0,1.,0.5,0.,1.5,0.75,2.,1.5);
    run(*agent,data,2.0,1.0,0.0);assert(data.transform.position[1]>1.0);run(*agent,data,0.0,0.0,0.0);assert(data.transform.position[1]<0.12);
    // Separate box steps with a continuous lower slab. Walking must lift before
    // sweeping forward and descend after leaving a riser, without tunnelling.
    physics.clear();solid(0,1.,-0.25,0.,2.,0.25,2.);solid(1,1.25,0.15,0.,0.25,0.15,2.);solid(2,2.,0.3,0.,0.5,0.3,2.);
    nav::Surface stairs[]={{{{-256,0,-512},{-256,0,512},{256,0,512}}},{{{-256,0,-512},{256,0,512},{256,0,-512}}},
        {{{256,77,-512},{256,77,512},{384,77,512}}},{{{256,77,-512},{384,77,512},{384,77,-512}}},
        {{{384,154,-512},{384,154,512},{640,154,512}}},{{{384,154,-512},{640,154,512},{640,154,-512}}}};
    for(int i=0;i<5;++i)n[i].y=i<2?0:i==2?77:154;g.surfaces=stairs;g.surface_count=6;nav::world.reset(g);data.transform.position[0]=data.transform.position[1]=0.0;
    run(*agent,data,2.0,Fixed(154*16,Fixed::RAW),0.0);assert(data.transform.position[1]>0.59);run(*agent,data,0.0,0.0,0.0);assert(data.transform.position[1]<0.01);
    // Authored traversal moves over time along an arc, then a climb trajectory.
    physics.clear();nav::Node jump[2]={{0,0,0,{nav::invalid,nav::invalid,nav::invalid,nav::invalid,1,nav::invalid}},{512,0,0,{nav::invalid,nav::invalid,nav::invalid,nav::invalid,nav::invalid,nav::invalid}}};
    nav::Traversal link{0,1,1,256,4096};nav::Graph jg{jump,2,256,nullptr,0,&link,1};nav::world.reset(jg);run(*agent,data,2.0,0.0,0.0);
    link.kind=2;jump[1].y=256;nav::world.reset(jg);data.transform.position[0]=data.transform.position[1]=0.0;run(*agent,data,2.0,1.0,0.0);assert(data.transform.position[1]==1.0);
    // An intermediate NoPath must not make gameplay cancel an automatic retry.
    jump[0].links[4]=nav::invalid;jg.traversal_count=0;nav::world.reset(jg);data.transform.position[0]=data.transform.position[1]=0.0;
    agent->blocked_timeout=0.2;agent->repath_delay=0.05;assert(agent->move_to(2.0,1.0,0.0));
    nav::world.tick();assert(!agent->failed());
    for(int i=0;i<120&&!agent->failed();++i){nav::world.tick();agent->tick(Fixed(1./60.));}
    assert(agent->failed()&&agent->status()==uint32_t(nav::Status::Blocked));
    std::puts("Navigation movement: slopes, descending, separate stair colliders, jump, climb and arrival passed.");
}
