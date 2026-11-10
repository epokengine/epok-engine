// Compare the complete game's flat-floor motor and idle-query policy before
// replacing its project implementation with the shared character kernel.
#include <cassert>
#include <cstdio>
#include "../../runtime/character3d.hpp"
using namespace epok;
static CollisionWorld<Fixed,8,4> world;
static unsigned sweeps=0,grounds=0;
static Aabb bounds(const ActorData& actor){
    Aabb box;for(unsigned i=0;i<3;++i){const auto center=actor.transform.position[i]+actor.collider.center[i];box.min[i]=center-actor.collider.half_extents[i];box.max[i]=center+actor.collider.half_extents[i];}return box;
}
namespace epok {
const ClassDescriptor object_classes[1]={};const size_t object_class_count=0;
int entity_index(const ActorData*){return -1;}
MoveResult move_and_slide(ActorData& actor,const Fixed* delta,uint32_t mask){
    ++sweeps;const auto result=world.move_and_slide(bounds(actor),delta,mask);
    for(unsigned i=0;i<3;++i)actor.transform.position[i]+=result.displacement[i];return result;
}
SpatialHit query_ground(const ActorData& actor,Fixed distance,uint32_t mask){++grounds;return world.ground(bounds(actor),distance,mask);}
}
#include "character3d_complete_reference.hpp"
using complete_reference::Cache;
using complete_reference::original;
struct SharedQueries {
    bool stable;
    MoveResult move(ActorData& body,const Fixed* delta,uint32_t mask){
        if(stable&&delta[0].raw()==0&&delta[2].raw()==0&&delta[1]==Fixed(-0.05)){
            MoveResult result;result.grounded=true;return result;
        }
        return move_and_slide(body,delta,mask);
    }
    SpatialHit ground(const ActorData& body,Fixed distance,uint32_t mask){return query_ground(body,distance,mask);}
};
static void slab(unsigned slot,Fixed x,Fixed y,Fixed z,Fixed hx,Fixed hy,Fixed hz){
    Collider collider;collider.enabled=true;collider.half_extents[0]=hx;collider.half_extents[1]=hy;collider.half_extents[2]=hz;
    auto transform=Affine<Fixed>::identity();transform.values[0][3]=x;transform.values[1][3]=y;transform.values[2][3]=z;world.set(slot,collider,transform);
}
int main(){
    unsigned comparisons=0,skipped=0,jumps=0;const Fixed dt(1.0/60.0);
    for(unsigned scenario=0;scenario<4;++scenario){
        world.clear();slab(0,0.0,-0.25,0.0,10.0,0.25,10.0);
        if(scenario==1)slab(1,2.0,2.0,0.0,0.1,2.0,4.0);
        if(scenario==2)slab(1,0.0,2.0,0.0,8.0,0.1,4.0);
        ActorData body{};body.alive=body.active=true;body.collider.enabled=true;
        body.collider.center[1]=0.92;body.collider.half_extents[0]=body.collider.half_extents[2]=0.28;body.collider.half_extents[1]=0.92;
        auto old=body;character3d::State state{},legacy;Cache cache{},old_cache{};uint32_t revision=1;
        for(unsigned frame=0;frame<360;++frame){
            GameplayVector2 desired{};
            if(frame>=30&&frame<120)desired={6.0,0.0};
            if(frame>=180&&frame<240)desired={-3.0,2.0};
            if(frame==150){body.transform.position[0]=old.transform.position[0]=0.0;++revision;}
            if(frame==270){body.transform.rotation[2]=old.transform.rotation[2]=10.0;++revision;}
            if(scenario==3&&frame==300){world.clear();++revision;}
            const Fixed facing=frame<180?Fixed(90.0):Fixed(300.0);
            CharacterCommand3D command;command.velocity=desired;command.jump=frame==60||frame==210;
            SharedQueries queries{state.grounded&&cache.same(body,revision)};
            Fixed prior[3];for(unsigned i=0;i<3;++i)prior[i]=body.transform.rotation[i];
            state.facing=facing;character3d::Tuning tuning;tuning.step_height=0.0;
            sweeps=grounds=0;const auto sample=character3d::advance(body,state,tuning,command,dt,queries,false);
            const unsigned actual=sweeps;assert(grounds==0);skipped+=actual==0;
            cache.store(body,revision,prior);
            sweeps=grounds=0;const bool jumped=original(old,legacy,old_cache,desired,facing,command.jump,dt,revision);
            assert(sample.status==CharacterMotionStatus::Applied&&sample.jumped==jumped&&sweeps==actual&&grounds==0);
            for(unsigned i=0;i<3;++i){
                assert(body.transform.position[i]==old.transform.position[i]);
                assert(body.transform.rotation[i]==old.transform.rotation[i]);
                assert(state.velocity[i]==legacy.velocity[i]);
            }
            assert(state.grounded==legacy.grounded&&state.facing==legacy.facing);
            jumps+=jumped;++comparisons;
        }
    }
    assert(comparisons==1440&&skipped&&jumps);
    std::printf("Complete motor: %u exact Q12 steps, %u idle query skips; wall, ceiling, jump, teleport, rotation and collision revisions match\n",comparisons,skipped);
}
