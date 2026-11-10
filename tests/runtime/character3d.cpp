// Preserve the pre-extraction third-person operation order against real collision queries.
#include <cassert>
#include <cstdio>
#include "../../runtime/character3d.hpp"
using namespace epok;
static CollisionWorld<Fixed,16,4> world;
struct Calls {unsigned lift=0,ground=0;};
static Calls calls;
static Aabb bounds(const ActorData& actor){Aabb box;for(unsigned i=0;i<3;++i){box.min[i]=actor.transform.position[i]-actor.collider.half_extents[i];box.max[i]=actor.transform.position[i]+actor.collider.half_extents[i];}return box;}
namespace epok {
const ClassDescriptor object_classes[1]={};const size_t object_class_count=0;
int entity_index(const ActorData*){return -1;}
MoveResult move_and_slide(ActorData& actor,const Fixed* movement,uint32_t mask){
 if(movement[0]==0.0&&movement[2]==0.0&&movement[1]>0.0)++calls.lift;
 auto result=world.move_and_slide(bounds(actor),movement,mask);
 for(unsigned i=0;i<3;++i)actor.transform.position[i]+=result.displacement[i];
 return result;
}
SpatialHit query_ground(const ActorData& actor,Fixed distance,uint32_t mask){++calls.ground;return world.ground(bounds(actor),distance,mask);}
}
// Frozen from the existing template before the shared motor was introduced.
// This reference deliberately does not call the new motor or its tuning helper.
static bool reference(ActorData& body,character3d::State& state,const CharacterCommand3D& command,Fixed dt){
 if(command.turn)state.facing=MathLibrary::move_toward_degrees(state.facing,command.heading,Fixed(540.0)*dt);
 Fixed response=(state.grounded?Fixed(12.0):Fixed(3.0))*dt;if(response>Fixed(1.0))response=1.0;
 state.velocity[0]+=(command.velocity.x-state.velocity[0])*response;
 state.velocity[2]+=(command.velocity.y-state.velocity[2])*response;
 const bool jumped=state.grounded&&command.jump;
 if(state.grounded)state.velocity[1]=jumped?Fixed(9.0):Fixed(0.0);else state.velocity[1]-=Fixed(24.0)*dt;
 const Fixed fall=state.grounded&&!jumped?Fixed(-0.05):state.velocity[1]*dt;
 const Fixed movement[3]={state.velocity[0]*dt,fall,state.velocity[2]*dt};const bool was_grounded=state.grounded;
 const auto result=move_and_slide(body,movement);
 state.grounded=result.grounded;
 if(state.velocity[1]>Fixed(0.0)&&result.displacement[1]+Fixed(0.001)<fall)state.velocity[1]=0.0;
 if(result.blocked&&state.grounded&&!jumped){
  const Fixed remaining[3]={movement[0]-result.displacement[0],0.0,movement[2]-result.displacement[2]};
  if(remaining[0].raw()||remaining[2].raw()){
   const Fixed lift[3]={0.0,0.6,0.0};const auto lifted=move_and_slide(body,lift);move_and_slide(body,remaining);
   const Fixed settle[3]={0.0,-lifted.displacement[1]+Fixed(-0.05),0.0};state.grounded=move_and_slide(body,settle).grounded;
  }
 }else if(was_grounded&&!state.grounded&&!jumped){
  const auto below=query_ground(body,0.6);
  if(below){const Fixed drop[3]={0.0,-(Fixed(0.6)*below.fraction)+Fixed(-0.05),0.0};state.grounded=move_and_slide(body,drop).grounded;}
 }
 body.transform.rotation[1]=state.facing;return jumped;
}
static void slab(unsigned slot,Fixed x,Fixed y,Fixed z,Fixed half_x,Fixed half_y,Fixed half_z){
 Collider collider;collider.enabled=true;collider.half_extents[0]=half_x;collider.half_extents[1]=half_y;collider.half_extents[2]=half_z;
 auto matrix=Affine<Fixed>::identity();matrix.values[0][3]=x;matrix.values[1][3]=y;matrix.values[2][3]=z;world.set(slot,collider,matrix);
}
int main(){
 const Fixed dt(1.0/60.0);unsigned frames=0,total_jumps=0,total_lifts=0,total_ground_queries=0;
 for(unsigned scenario=0;scenario<6;++scenario){
  world.clear();slab(0,0.0,-0.25,0.0,40.0,0.25,10.0);
  if(scenario==1)slab(1,2.0,2.0,0.0,0.1,2.0,4.0); // Wall.
  if(scenario==2)slab(1,0.0,3.0,0.0,20.0,0.25,4.0); // Low ceiling.
  if(scenario==3)slab(1,3.0,0.2,0.0,3.0,0.2,4.0); // Up a 0.4 step.
  if(scenario==4)slab(1,-3.0,0.2,0.0,3.0,0.2,4.0); // Down a 0.4 step.
  ActorData body{},old;body.alive=body.active=true;body.collider.enabled=true;
  body.collider.half_extents[0]=0.4;body.collider.half_extents[1]=1.0;body.collider.half_extents[2]=0.4;
  body.transform.position[0]=-3.0;body.transform.position[1]=scenario==4?Fixed(1.4):Fixed(1.0);old=body;
  character3d::State state{},legacy;Fixed peak=body.transform.position[1];bool hit_wall=false,step_up=false;
  for(unsigned frame=0;frame<240;++frame){
   CharacterCommand3D command;command.velocity.x=frame<150?Fixed(6.0):Fixed(0.0);command.heading=90.0;command.turn=frame<150;
   command.jump=(scenario==0||scenario==2||scenario==5)&&frame==30;
   if(scenario==5&&frame>100){command.velocity.x=-3.0;command.velocity.y=2.0;command.heading=300.0;}
   calls={};const auto sample=character3d::advance(body,state,{},command,dt);const auto motor_calls=calls;
   calls={};const bool jumped=reference(old,legacy,command,dt);
   assert(sample.status==CharacterMotionStatus::Applied&&sample.jumped==jumped);
   assert(calls.lift==motor_calls.lift&&calls.ground==motor_calls.ground);
   for(unsigned i=0;i<3;++i){assert(body.transform.position[i]==old.transform.position[i]);assert(state.velocity[i]==legacy.velocity[i]);}
   assert(state.facing==legacy.facing&&state.grounded==legacy.grounded);
   if(body.transform.position[1]>peak)peak=body.transform.position[1];
   hit_wall|=sample.blocked&&body.transform.position[0]<2.0;
   step_up|=body.transform.position[0]>0.0&&body.transform.position[1]>1.3;
   total_jumps+=jumped;total_lifts+=motor_calls.lift;total_ground_queries+=motor_calls.ground;++frames;
  }
  if(scenario==0)assert(peak>2.5&&state.grounded);
  if(scenario==1)assert(hit_wall&&body.transform.position[0]<1.51);
  if(scenario==2)assert(peak<1.8&&state.grounded);
  if(scenario==3)assert(step_up&&body.transform.position[0]>6.0);
  if(scenario==4)assert(body.transform.position[0]>6.0&&body.transform.position[1]<1.01);
 }
 assert(frames==1440&&total_jumps==3&&total_lifts&&total_ground_queries);
 ActorData body{};character3d::State state{};CharacterCommand3D command;command.jump=true;
 auto zero=character3d::advance(body,state,{},command,0.0);assert(zero.status==CharacterMotionStatus::NoStep&&body.transform.position[1]==0.0);
 character3d::Tuning invalid;invalid.gravity=-1.0;assert(character3d::advance(body,state,invalid,command,dt).status==CharacterMotionStatus::InvalidConfiguration&&body.transform.position[1]==0.0);
 assert(character3d::advance(body,state,{},command,Fixed(-1.0)).status==CharacterMotionStatus::InvalidConfiguration);
 state.facing=Fixed(INT32_MIN,Fixed::RAW);command.heading=Fixed(INT32_MAX,Fixed::RAW);
 assert(character3d::advance(body,state,{},command,dt).status==CharacterMotionStatus::InvalidConfiguration&&state.facing.raw()==INT32_MIN);
 world.clear();slab(0,0.0,-0.25,0.0,10.0,0.25,10.0);state={};command={};body={};
 body.collider.enabled=true;body.collider.half_extents[0]=body.collider.half_extents[1]=body.collider.half_extents[2]=0.5;
 body.transform.position[1]=0.5;character3d::Tuning filtered;filtered.collision_mask=0;
 for(unsigned i=0;i<60;++i)assert(!character3d::advance(body,state,filtered,command,dt).grounded);
 assert(body.transform.position[1]<-5.0); // Every motor query respects the configured collision mask.
 // An injected synchronized query service uses the same integration. Zero
 // stair height disables both lift and snap-down probes, including at a ledge.
 struct RoutedQueries {
  unsigned moves=0,grounds=0;
  MoveResult move(ActorData& actor,const Fixed* delta,uint32_t mask){++moves;return move_and_slide(actor,delta,mask);}
  SpatialHit ground(const ActorData& actor,Fixed distance,uint32_t mask){++grounds;return query_ground(actor,distance,mask);}
 } routed;
 world.clear();state={};state.grounded=true;body={};command={};command.velocity.x=6.0;
 character3d::Tuning flat;flat.step_height=0.0;
 const auto falling=character3d::advance(body,state,flat,command,dt,routed,false);
 assert(falling.status==CharacterMotionStatus::Applied&&!falling.grounded);
 assert(routed.moves==1&&routed.grounds==0&&falling.displacement.x>0.0&&falling.speed==0.0);
 const auto measured=character3d::advance(body,state,flat,command,dt,routed);
 assert(measured.speed>0.0&&routed.moves==2&&routed.grounds==0);
 std::printf("Character motor: %u Q12 frames match the existing movement reference; jumps, wall, ceiling, step-up/down and zero/invalid updates passed\n",frames);
}
