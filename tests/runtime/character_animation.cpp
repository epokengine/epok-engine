#define EPOK_TEST_CHARACTER_ANIMATION 1
#include "character_command_fixture.hpp"
#include "../../templates/CharacterMotion.hpp"

static AnimationClip skeletal_clips[]={
 {nullptr,nullptr,nullptr,12,nullptr},{nullptr,nullptr,nullptr,24,nullptr},
 {nullptr,nullptr,nullptr,18,nullptr},{nullptr,nullptr,nullptr,8,nullptr},
 {nullptr,nullptr,nullptr,6,nullptr},{nullptr,nullptr,nullptr,5,nullptr},
};
static SkeletalMesh model=[](){SkeletalMesh result{};result.clips=skeletal_clips;result.clip_count=6;return result;}();
static const SpriteFrame sprite_frames[]={{{0,0,8,8},Fixed(1.0/60.0),7},{{8,0,8,8},Fixed(1.0/60.0),8}};
static const SpriteClip sprite_clips[]={
 {sprite_frames,2,true,"Idle"},{sprite_frames,2,true,"Walk"},{sprite_frames,2,true,"Run"},
 {sprite_frames,2,false,"Jump"},{sprite_frames,2,false,"Fall"},{sprite_frames,2,false,"Land"},
};
static void visual(Character3D& actor,Mesh3DComponent& mesh){
 actor.visual=mesh.id();auto& animator=actor.data()->animator;animator.enabled=true;animator.model=&model;
}
int main(){
 // Frozen state choice and speed-driven phase compared with the former starter.
 character_motion::AnimationState legacy;LocomotionState selected=LocomotionState::Idle;
 const LocomotionTuning tuning;const LocomotionClips clips{0,1,2,3,4,5};const Fixed dt(1.0/60.0);
 unsigned states[6]={};Fixed old_phase=0.0,new_phase=0.0;
 for(unsigned frame=0;frame<2400;++frame){
  const bool grounded=frame%240<160,jumped=frame%240==160,finished=frame%11==0;
  const Fixed vertical=frame%240<190?Fixed(2.0):Fixed(-2.0);
  const Fixed speed(int32_t((frame*137)%27000),Fixed::RAW);
  const auto before=selected;
  const auto old=legacy.choose(grounded,vertical.raw(),speed.raw(),finished,jumped);
  selected=locomotion::choose(selected,grounded,vertical,speed,finished,jumped,tuning);
  assert(unsigned(selected)==unsigned(old));++states[unsigned(selected)];
  const auto value=locomotion::selection(selected,speed,clips,tuning,true);
  assert(value.clip==int(selected)&&value.looping==character_motion::AnimationState::looping(old));
  if(before!=selected)old_phase=new_phase=0.0;
  if(locomotion::moving(selected)){
   const Fixed rate=MathLibrary::clamp(speed/(selected==LocomotionState::Walk?Fixed(2.6):Fixed(6.0)),Fixed(0.2),Fixed(1.6));
   assert(value.rate==rate);old_phase+=rate*dt*Fixed(60.0);
   const Fixed period(int32_t((skeletal_clips[unsigned(selected)].frames-1)*2),0);
   for(int pass=0;pass<4;++pass)if(old_phase>=period)old_phase-=period;
   new_phase=Fixed(int32_t((int64_t(new_phase.raw())+int64_t((value.rate*dt).raw())*60)%period.raw()),Fixed::RAW);
   assert(old_phase==new_phase);
  }
 }
 for(auto count:states)assert(count>0);
 ObjectRegistryStorage<256> registry;active_object_registry=&registry;Level level;assert(level.bind(registry));world2d::service.reset();world2d::install();
 Collider floor;floor.enabled=true;floor.half_extents[0]=floor.half_extents[2]=1000.0;floor.half_extents[1]=0.25;
 auto pose=Affine<Fixed>::identity();pose.values[1][3]=-0.25;world.set(31,floor,pose);
 auto* actor=spawn<Character3D>(level);actor->motor.automatic_tick=false;
 auto* mesh=level.add_component<Mesh3DComponent>(*actor,"Mesh");assert(mesh);visual(*actor,*mesh);
 auto* driver=level.add_component<SkeletalLocomotionComponent>(*actor,"Locomotion");assert(driver);driver->automatic_tick=false;driver->clips=clips;
 assert(driver->update(dt).status==LocomotionStatus::NoStep);actor->motor.simulate(dt);
 assert(driver->update(dt).status==LocomotionStatus::Applied&&actor->data()->animator.clip==0);
 for(unsigned frame=0;frame<360;++frame){
  actor->motor.submit({{frame<180?Fixed(2.0):Fixed(6.0),0.0},90.0,true,frame==100});actor->motor.simulate(dt);
  const auto result=driver->update(dt);assert(result.status==LocomotionStatus::Applied);
  assert(actor->data()->animator.clip==result.clip);
  if(locomotion::moving(result.state))assert(!actor->data()->animator.playing);
  actor->data()->animator.advance();
 }
 const auto cursor=actor->data()->animator.ticks;
 assert(driver->update(0.0).status==LocomotionStatus::NoStep&&actor->data()->animator.ticks==cursor);
 driver->tuning.walk_cycle_speed=0.0;assert(driver->update(dt).status==LocomotionStatus::InvalidConfiguration&&actor->data()->animator.ticks==cursor);driver->tuning=tuning;
 driver->clips={};assert(driver->update(dt).status==LocomotionStatus::MissingClip&&actor->data()->animator.ticks==cursor);driver->clips=clips;
 driver->set_enabled(false);assert(driver->update(dt).status==LocomotionStatus::NoStep&&actor->data()->animator.ticks==cursor);driver->set_enabled(true);
 assert(driver->update(dt).changed);driver->distance_driven=false;assert(driver->update(dt).status==LocomotionStatus::Applied&&actor->data()->animator.playing);
 driver->visual=actor->root.id();assert(driver->update(dt).status==LocomotionStatus::InvalidVisual);driver->visual={};
 auto invalid_model=model;invalid_model.clips=nullptr;actor->data()->animator.model=&invalid_model;
 assert(driver->update(dt).status==LocomotionStatus::InvalidVisual);actor->data()->animator.model=&model;
 // Sprite selection reuses the player's actual frame/event service.
 auto* two=spawn<Character2D>(level);two->motor.automatic_tick=false;two->body.floating=true;
 auto* sprite=level.add_component<Sprite2DComponent>(*two,"Sprite");assert(sprite);sprite->clips=sprite_clips;sprite->clip_count=6;two->visual=sprite->id();
 auto* sprite_driver=level.add_component<SpriteLocomotion2DComponent>(*two,"Locomotion");assert(sprite_driver);sprite_driver->automatic_tick=false;
 two->motor.submit({{32.0,0.0},false});two->motor.simulate(dt);auto result=sprite_driver->update(dt);assert(result.status==LocomotionStatus::Applied&&result.clip==1&&result.changed&&!sprite->flip_h);
 assert(sprite->pending_event==7);sprite->pending_event=0;sprite->advance(dt);assert(sprite->frame==1&&sprite->pending_event==8);
 sprite->pending_event=0;assert(!sprite_driver->update(dt).changed&&sprite->pending_event==0); // No repeated first-frame events.
 two->motor.submit({{-32.0,0.0},false});two->motor.simulate(dt);sprite_driver->update(dt);assert(sprite->flip_h);
 two->motor.stop();two->motor.simulate(dt);sprite_driver->update(dt);assert(sprite->animation==0&&sprite->flip_h);
 sprite_driver->distance_driven=true;two->motor.submit({{48.0,0.0},false});two->motor.simulate(dt);result=sprite_driver->update(dt);assert(result.rate==0.5&&sprite->speed_scale==0.5);
 sprite_driver->clips.walk=99;const auto old_frame=sprite->frame;assert(sprite_driver->update(dt).status==LocomotionStatus::MissingClip&&sprite->frame==old_frame);sprite_driver->clips.walk=1;
 sprite_driver->visual=two->root.id();assert(sprite_driver->update(dt).status==LocomotionStatus::InvalidVisual);sprite_driver->visual={};
 sprite_driver->animate_air=true;sprite_driver->clips=clips;
 two->motor.submit({{0.0,-32.0},false});two->motor.simulate(dt);result=sprite_driver->update(dt);assert(result.state==LocomotionState::JumpUp&&result.clip==3);
 two->motor.submit({{0.0,32.0},false});two->motor.simulate(dt);result=sprite_driver->update(dt);assert(result.state==LocomotionState::JumpDown&&result.clip==4);
 auto* floor2=spawn<Actor2D>(level);floor2->root.transform.position[1]=16.0;auto* floor_shape=level.add_component<Collider2DComponent>(*floor2,"Floor");assert(floor_shape);floor_shape->set_box_size(2000.0,16.0);
 two->collider.set_box_size(16.0,16.0);two->root.transform.position[1]=0.0;two->body.floating=false;two->motor.reset_motion();world2d::service.sync(true);two->motor.simulate(dt);
 result=sprite_driver->update(dt);assert(result.status==LocomotionStatus::Applied&&result.state==LocomotionState::Land);
 assert(sprite_driver->update(dt).state==LocomotionState::Land);sprite->advance(dt);sprite->advance(dt);sprite->advance(dt);
 assert(sprite_driver->update(dt).state==LocomotionState::Idle);
 Level foreign;assert(foreign.bind(registry));auto* remote=spawn<Character2D>(foreign);remote->motor.automatic_tick=false;sprite_driver->motor=remote->motor.id();assert(sprite_driver->update(dt).status==LocomotionStatus::InvalidMotor);sprite_driver->motor={};
 // Global PostPhysics ordering works with the driver after the motor in storage.
 driver->automatic_tick=true;actor->motor.automatic_tick=true;actor->motor.submit({{1.0,0.0}});level.tick(dt);assert(driver->sample().status==LocomotionStatus::Applied);
 driver->automatic_tick=actor->motor.automatic_tick=false;
 for(unsigned action=0;action<=6;++action){
  auto* owner=spawn<Character3D>(level);owner->motor.automatic_tick=false;
  auto* part=level.add_component<Mesh3DComponent>(*owner,"Mesh");assert(part);visual(*owner,*part);owner->motor.simulate(dt);
  auto* callback=level.add_component<CallbackLocomotion>(*owner,"Callback");assert(callback);callback->automatic_tick=false;callback->clips=clips;callback->action=action;const auto id=owner->id();
  const auto applied=callback->update(dt);
  if(action==1){assert(applied.status==LocomotionStatus::InvalidOwner);level.tick(0.0);assert(!registry.get(id));}
  else if(action==2)assert(applied.status==LocomotionStatus::NoStep);
  else if(action==3)assert(applied.status==LocomotionStatus::InvalidConfiguration);
  else if(action==4)assert(applied.status==LocomotionStatus::MissingClip);
  else if(action==5){assert(applied.status==LocomotionStatus::Applied);level.tick(0.0);assert(!registry.get(id));}
  else if(action==6)assert(applied.status==LocomotionStatus::InvalidVisual);
  else assert(applied.status==LocomotionStatus::Applied&&callback->nested==LocomotionStatus::Busy&&callback->calls==1);
 }
 for(unsigned action=0;action<=3;++action){
  auto* owner=spawn<Character2D>(level);owner->motor.automatic_tick=false;owner->body.floating=true;
  owner->root.transform.position[0]=Fixed(int32_t(100+action*50),0);
  auto* part=level.add_component<Sprite2DComponent>(*owner,"Sprite");assert(part);part->clips=sprite_clips;part->clip_count=6;
  auto* callback=level.add_component<CallbackSpriteLocomotion>(*owner,"Callback");assert(callback);callback->automatic_tick=false;callback->visual=part->id();callback->action=action;
  owner->motor.simulate(dt);const auto id=owner->id();const auto applied=callback->update(dt);
  if(action==1){assert(applied.status==LocomotionStatus::InvalidOwner);level.tick(0.0);assert(!registry.get(id));}
  else if(action==2)assert(applied.status==LocomotionStatus::InvalidVisual);
  else if(action==3){assert(applied.status==LocomotionStatus::Applied);level.tick(0.0);assert(!registry.get(id));}
  else assert(applied.status==LocomotionStatus::Applied&&callback->nested==LocomotionStatus::Busy&&callback->calls==1);
 }
 foreign.end_play_all(EndPlayReason::LevelUnloaded);
 level.end_play_all(EndPlayReason::LevelUnloaded);
 std::puts("Character animation: 2400 Q12 legacy choices/phases, real motor playback, sprite events/facing/rate, tick order and callback lifetimes passed");
}
