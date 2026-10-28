#include "SpatialProbe.hpp"
#include <cassert>

using namespace epok;
static void same_hit(const SpatialHit& a,const SpatialHit& b){
    assert(a.entity==b.entity&&a.generation==b.generation&&a.fraction==b.fraction&&a.started_inside==b.started_inside);
    for(unsigned k=0;k<3;++k)assert(a.point[k]==b.point[k]&&a.normal[k]==b.normal[k]);
}
static void same_move(const MoveResult& a,const MoveResult& b){
    assert(a.entity==b.entity&&a.generation==b.generation&&a.grounded==b.grounded&&a.blocked==b.blocked&&a.unresolved_overlap==b.unresolved_overlap);
    for(unsigned k=0;k<3;++k)assert(a.displacement[k]==b.displacement[k]&&a.normal[k]==b.normal[k]);
}
void SpatialProbe::begin_play(){
    ActorData* actors[]={find_actor_data("Actor0"),find_actor_data("Actor1"),find_actor_data("Actor2"),find_actor_data("Actor3")};
    for(size_t i=0;i<4;++i){
        assert(actors[i]);auto& actor=*actors[i];actor.parent=-1;actor.active=true;
        for(unsigned k=0;k<3;++k){actor.transform.position[k]=0.0;actor.transform.rotation[k]=0.0;actor.transform.scale[k]=1.0;actor.collider.center[k]=0.0;actor.collider.half_extents[k]=0.5;}
        actor.collider.enabled=i<2;actor.collider.trigger=false;actor.collider.layer=1;actor.collider.mask=0xffffffffu;
    }
    auto& mover=*actors[0];auto& wall=*actors[1];
    Fixed origin[3]={0.0,0.0,0.0},delta[3]={20.0,0.0,0.0};
    SpatialQueryBatch batch;
    wall.transform.position[0]=5.0;batch.changed(wall);
    const auto initial=batch.raycast(origin,delta,1,&mover);
    assert(initial.entity==1);same_hit(initial,epok::raycast(origin,delta,1,&mover));

    // A batch observes direct edits when notified; no unrelated full query may
    // repair stale bounds before this assertion.
    wall.transform.position[0]=8.0;batch.changed(wall);
    const auto moved=batch.raycast(origin,delta,1,&mover);
    assert(moved.fraction>initial.fraction);same_hit(moved,epok::raycast(origin,delta,1,&mover));
    RaycastQuery queries[2]{};queries[0].displacement[0]=20.0;queries[1].origin[1]=10.0;queries[1].displacement[0]=20.0;
    SpatialHit results[2];batch.raycast_batch(queries,results,2,1,&mover);
    same_hit(results[0],moved);assert(results[1].entity<0);
    batch.raycast_batch(nullptr,results,2);batch.raycast_batch(queries,nullptr,2);batch.raycast_batch(queries,results,0);

    assert(set_active(&wall,false));batch.sync(wall);assert(batch.raycast(origin,delta,1,&mover).entity<0);
    assert(set_active(&wall,true));wall.collider.layer=2;batch.changed(wall);
    assert(batch.raycast(origin,delta,1,&mover).entity<0);
    assert(batch.raycast(origin,delta,2,&mover).entity==1);
    wall.collider.trigger=true;batch.changed(wall);
    assert(batch.raycast(origin,delta,2,&mover).entity<0);
    assert(batch.raycast(origin,delta,2,&mover,true).entity==1);
    assert(batch.raycast(origin,delta,2,&wall,true).entity<0);
    wall.collider.trigger=false;wall.collider.layer=1;batch.sync(wall);

    const auto movement=batch.move_and_slide(mover,delta,1);
    assert(movement.blocked&&mover.transform.position[0]>Fixed(6.0)&&mover.transform.position[0]<Fixed(8.0));
    Fixed behind[3]={-5.0,0.0,0.0};
    const auto published=batch.raycast(behind,delta,1,&wall);
    assert(published.entity==0&&published.fraction>Fixed(0.5));
    same_hit(published,epok::raycast(behind,delta,1,&wall));
    mover.transform.position[0]=0.0;same_move(movement,epok::move_and_slide(mover,delta,1));

    // Parented motion must convert world displacement back to local space and
    // publish its bounds, including changes to the parent's transform.
    auto& parent=*actors[2];auto& child=*actors[3];assert(set_active(&mover,false));
    parent.transform.scale[0]=2.0;child.parent=2;child.collider.enabled=true;
    SpatialQueryBatch parented;
    const auto parent_move=parented.move_and_slide(child,delta,1);
    assert(parent_move.blocked&&child.transform.position[0]>Fixed(2.0)&&child.transform.position[0]<Fixed(4.0));
    const auto child_hit=parented.raycast(behind,delta,1,&wall);
    assert(child_hit.entity==3);same_hit(child_hit,epok::raycast(behind,delta,1,&wall));
    child.transform.position[0]=0.0;same_move(parent_move,epok::move_and_slide(child,delta,1));
    parent.transform.position[0]=1.0;parented.sync(child);
    const auto shifted=parented.raycast(behind,delta,1,&wall);
    assert(shifted.fraction>child_hit.fraction);same_hit(shifted,epok::raycast(behind,delta,1,&wall));
    child.collider.enabled=false;parented.changed(child);assert(parented.raycast(behind,delta,1,&wall).entity<0);

    ActorData outside;parented.sync(outside);
    assert(parented.move_and_slide(outside,delta).unresolved_overlap);
    assert(!prewarm_geometry(nullptr)&&!prewarm_geometry(&wall));
    mover.transform.position[1]=42.0; // The parent test receives this only after every assertion passes.
}
