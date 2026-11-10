#include "actor_scene_fixture.hpp"
#include <cstdio>
using namespace epok;

// The adapter owns lifecycle/identity and value marshalling. Collision service
// calls are observed here; integration exercises the production world backend.
namespace epok {
static unsigned rays=0,pairs=0,moves=0,published=0,synchronizations=0;
static uint32_t seen_mask=0;
static bool seen_triggers=false;
static const ActorData* seen_ignore=nullptr;
static GameplayVector3 seen_origin{},seen_delta{};
void SpatialQueryBatch::ensure()const{if(!ready){ready=true;++synchronizations;}}
void SpatialQueryBatch::sync(ActorData& data){++published;assert(&data==&entities[1]);}
SpatialHit SpatialQueryBatch::raycast(const Fixed* from,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers)const{
    ensure();
    ++rays;seen_mask=mask;seen_ignore=ignore;seen_triggers=triggers;
    seen_origin=gameplay_vector(from);seen_delta=gameplay_vector(delta);
    SpatialHit hit;hit.entity=1;hit.generation=entities[1].generation;hit.fraction=.5;
    hit.point[0]=3.0;hit.normal[2]=-1.0;return hit;
}
void SpatialQueryBatch::raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask,const ActorData* ignore,bool triggers)const{
    ++pairs;assert(count==2);
    for(size_t i=0;i<count;++i)results[i]=raycast(queries[i].origin,queries[i].displacement,mask,ignore,triggers);
    results[1].entity=2;results[1].generation=entities[2].generation;results[1].fraction=.25;
}
MoveResult SpatialQueryBatch::move_and_slide(ActorData& actor,const Fixed* delta,uint32_t mask){
    ++moves;seen_delta=gameplay_vector(delta);seen_mask=mask;assert(&actor==&entities[1]);
    MoveResult result;result.grounded=true;result.blocked=true;result.entity=2;result.generation=entities[2].generation;
    result.displacement[0]=delta[0];result.normal[1]=1.0;return result;
}
}

int main(){
    test_reset_scene();
    auto* owner=test_registry.resolve<Actor3D>(test_owner(0));
    auto* context=test_level.add_component<SpatialQueryContextComponent>(*owner,"Queries");assert(context);
    assert(context->automatic_reset);
    const auto hit=context->raycast_segment({1.0,2.0,3.0},{4.0,5.0,6.0},7,test_owner(3),true);
    assert(hit.hit&&hit.actor==test_owner(1)&&hit.fraction==Fixed(.5)&&hit.point.x==3.0&&hit.normal.z==-1.0);
    assert(rays==1&&seen_mask==7&&seen_ignore==&entities[3]&&seen_triggers);
    assert(seen_origin.y==2.0&&seen_delta.z==6.0);
    const auto pair=context->raycast_pair({0.0,1.0,2.0},{3.0,4.0,5.0},{6.0,7.0,8.0},{9.0,10.0,11.0},3,{},false);
    assert(pairs==1&&rays==3&&pair.first.actor==test_owner(1)&&pair.second.actor==test_owner(2));
    assert(pair.first.fraction==Fixed(.5)&&pair.second.fraction==Fixed(.25));
    assert(seen_origin.x==6.0&&seen_delta.z==11.0&&!seen_ignore&&!seen_triggers);
    const auto moved=context->move(test_owner(1),{2.0,0.0,0.0},9);
    assert(moved.valid&&moved.grounded&&moved.blocked&&moved.actor==test_owner(2));
    assert(moved.displacement.x==2.0&&moved.normal.y==1.0&&moves==1&&seen_mask==9);
    // Invalid references and other levels never reach the world service.
    const ObjectId invalid{0xfffe,1};
    assert(!context->move(invalid,{},1).valid&&!context->publish_actor(invalid));
    assert(!context->raycast_segment({},{},1,invalid,false).hit&&rays==3);
    Level other;assert(other.bind(test_registry));
    const auto foreign=other.spawn_actor(*find_object_class(Actor3D::static_class_id),"Other level");
    ActorData foreign_data{};other.registry()->resolve<Actor3D>(foreign)->bind_data(foreign_data);
    assert(!context->move(foreign,{},1).valid&&!context->publish_actor(foreign));
    assert(!context->raycast_pair({},{},{},{},1,foreign,false).first.hit&&pairs==1);
    test_set_active(1,false);
    assert(!context->move(test_owner(1),{},1).valid&&moves==1);
    // Publishing inactive actors is allowed so disabled colliders can be removed.
    assert(context->publish_actor(test_owner(1))&&published==1);
    test_set_active(0,false);
    assert(!context->raycast_segment({},{},1,{},false).hit&&rays==3);
    assert(!context->publish_actor(test_owner(1)));
    test_set_active(0,true);
    context->raycast_segment({},{},1,{},false);assert(synchronizations==2);
    context->automatic_reset=false;context->tick(Fixed(1./60.));
    context->raycast_segment({},{},1,{},false);assert(synchronizations==2);
    context->reset_queries();context->raycast_segment({},{},1,{},false);assert(synchronizations==3);
    context->automatic_reset=true;context->tick(Fixed(1./60.));
    context->raycast_segment({},{},1,{},false);assert(synchronizations==4);
    const auto old=context->id();assert(test_level.remove_component(*owner,old));
    assert(!test_registry.resolve<SpatialQueryContextComponent>(old));
    context=test_level.add_component<SpatialQueryContextComponent>(*owner,"Replacement");
    assert(context&&context->id()!=old&&!test_registry.resolve<SpatialQueryContextComponent>(old));
    other.end_play_all(EndPlayReason::LevelUnloaded);test_registry.release(other.id());
    std::puts("Spatial query adapter: independent results, typed references, levels and activation passed.");
}
