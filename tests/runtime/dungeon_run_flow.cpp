#include "actor_scene_fixture.hpp"
#include "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonRunFlow.hpp"
#include "../fixtures/dungeon-legacy/DungeonFlow.hpp"
#include <cstdio>
using namespace epok;

static void compare(const DungeonRunFlow& flow,const DungeonFlow& old,bool reset){
 const auto& s=flow.status;
 assert(s.valid&&s.phase==uint32_t(old.phase)&&s.elapsed.raw()==int32_t(old.elapsed));
 assert(s.fade==old.fade()&&s.music_gain.raw()==old.music_gain());
 assert(s.playing==old.playing()&&s.restart==reset&&s.stop_music==(old.phase==DungeonFlow::RestartHold));
 assert(s.alive==(s.state<2));
}
static void binding_lifetime(){
 auto* previous=active_object_registry;
 ObjectRegistryStorage<4> first,second;
 DungeonRunFlow flow,other;
 const ClassDescriptor type{DungeonRunFlowBase::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::None,1};
 const auto* wrong_type=find_object_class(ActorComponent::static_class_id);assert(wrong_type);
 DungeonRunFlowBinding binding;
 active_object_registry=&first;
 assert(!binding.resolve({}));
 auto id=first.adopt(flow,type);assert(id.valid());
 assert(binding.resolve(id)==&flow&&binding.resolve(id)==&flow);
 flow.status.phase=5;assert(binding.resolve(id)->status.phase==5);
 const auto wrong=first.adopt(other,*wrong_type);assert(wrong.valid());
 assert(!binding.resolve(wrong));assert(binding.resolve(id)==&flow);
 assert(first.release(id));assert(!binding.resolve(id));
 auto reused=first.adopt(flow,type);
 assert(reused.index==id.index&&reused.generation!=id.generation);
 assert(!binding.resolve(id)&&binding.resolve(reused)==&flow);
 {ObjectDispatchScope dispatch(first);assert(first.release(reused));assert(!binding.resolve(reused));}
 auto current=first.adopt(flow,type);assert(binding.resolve(current)==&flow);
 // Different registries can recycle the identical address and handle bits.
 // The cached cast must not survive an authoritative type change in that case.
 assert(first.release(current));
 for(unsigned generation=1;generation<current.generation;++generation){auto temporary=second.adopt(flow,type);assert(second.release(temporary));}
 auto changed=second.adopt(flow,*wrong_type);assert(changed==current);
 active_object_registry=&second;assert(!binding.resolve(changed));
 assert(second.release(changed));
 auto restored=second.adopt(flow,type);assert(binding.resolve(restored)==&flow);
 active_object_registry=nullptr;assert(!binding.resolve(restored));
 active_object_registry=&second;assert(binding.resolve(restored)==&flow);
 assert(second.release(restored)&&first.release(wrong));
 active_object_registry=previous;
}
int main(){
 binding_lifetime();
 assert(MathLibrary::truncate_unsigned(-1.5)==0&&MathLibrary::truncate_unsigned(0.999)==0);
 assert(MathLibrary::truncate_unsigned(1.0)==1&&MathLibrary::truncate_unsigned(255.999)==255);
 assert(MathLibrary::truncate_unsigned(Fixed(INT32_MAX,Fixed::RAW))==524287);
 unsigned checks=0,restarts=0;
 for(int raw:{68,69,137,4096}){
  DungeonRunFlow flow;DungeonFlow old;flow.start();old.begin();compare(flow,old,false);
  for(unsigned i=0;i<2400;++i){
   if(old.playing()){flow.die();old.die();assert(flow.status.state==3);}
   const bool reset=old.step(uint32_t(raw),i>=19);
   flow.advance(Fixed(raw,Fixed::RAW),i>=19);compare(flow,old,reset);++checks;
   assert(!flow.restart_requested(true));
   if(reset){flow.start();old.begin();++restarts;}
  }
 }
 assert(restarts>20);
 DungeonRunFlow flow;flow.start();const auto first=flow.status;
 flow.advance(0.0,true);assert(flow.status.phase==first.phase&&flow.status.elapsed==first.elapsed);
 flow.advance(-1.0,true);assert(flow.status.phase==first.phase);
 flow.advance(2.0,true);assert(flow.status.phase==first.phase);
 assert(!flow.try_enter_boss(false,100.0));assert(!flow.try_enter_boss(true,53.0));
 assert(flow.try_enter_boss(true,Fixed(53.0)+Fixed(1,Fixed::RAW))&&flow.status.state==1);
 assert(!flow.try_enter_boss(true,100.0));flow.win();assert(flow.status.state==2&&!flow.status.alive);
 flow.die();assert(flow.status.state==2);
 flow.start();flow.advance(Fixed(1.0),true);
 for(unsigned i=0;i<4;++i)flow.advance(Fixed(1.0),true);
 assert(flow.status.playing);
 assert(flow.try_enter_boss(true,54.0));flow.win();assert(flow.restart_requested(true)&&!flow.restart_requested(false));
 flow.start();flow.timing.music_lead=0.0;flow.timing.fade_in=0.0;flow.timing.control_delay=0.0;
 for(unsigned i=0;i<4;++i)flow.advance(Fixed(1,Fixed::RAW),true);
 assert(flow.status.playing&&flow.status.fade==0);
 flow.timing.death_hold=-1.0;flow.start();assert(!flow.status.valid);
 flow.timing.death_hold=2.0;flow.start();assert(flow.status.valid);
 flow.die();flow.advance(Fixed(.25),true);const auto held=flow.status.elapsed;
 flow.die();assert(flow.status.elapsed==held);
 std::printf("Run flow: %u exact legacy Q12 samples, %u auto restarts; gate, victory, invalid/zero delta, editable timing and idempotent death passed\n",checks,restarts);
}
