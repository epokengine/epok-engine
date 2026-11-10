#include "character_command_fixture.hpp"
int main(){
 ObjectRegistryStorage<256> registry;active_object_registry=&registry;Level level;assert(level.bind(registry));world2d::service.reset();world2d::install();
 input.reset();gameplay_controls().reset();const Fixed dt(1.0/60.0);
 Collider floor;floor.enabled=true;floor.half_extents[0]=floor.half_extents[2]=1000.0;floor.half_extents[1]=0.25;auto pose=Affine<Fixed>::identity();pose.values[1][3]=-0.25;world.set(31,floor,pose);
 auto* p3=spawn<Player3D>(level);auto* p2=spawn<Player2D>(level);auto* n3=spawn<NPC3D>(level);auto* n2=spawn<NPC2D>(level);
 assert(p3->component_count()==4&&p2->component_count()==5&&n3->component_count()==4&&n2->component_count()==5);
 p3->motor.automatic_tick=p2->motor.automatic_tick=n3->motor.automatic_tick=n2->motor.automatic_tick=false;
 p3->input_source.automatic_tick=p2->input_source.automatic_tick=n3->command_source.automatic_tick=n2->command_source.automatic_tick=false;
 p3->motor.simulate(dt);assert(p3->motor.motion().grounded);
 constexpr uint16_t right=1u<<unsigned(Button::Right),up=1u<<unsigned(Button::Up),cross=1u<<unsigned(Button::Cross);
 frame(right);auto command=p3->input_source.update(dt);assert(command.status==CharacterCommandStatus::Applied&&command.command.velocity.x==6.0&&command.command.velocity.y==0.0);
 auto* camera=spawn<CameraRig3D>(level);camera->automatic_tick=false;camera->yaw=90.0;p3->camera=camera->id();
 frame(up);command=p3->input_source.update(dt);assert(command.command.velocity.x==6.0&&command.command.velocity.y==0.0&&command.command.heading==90.0);
 p3->camera={};current=camera->data();current->transform.rotation[1]=90.0;
 command=p3->input_source.update(dt);assert(command.command.velocity.x==6.0&&command.command.velocity.y==0.0);p3->camera=camera->id();current=nullptr;
 frame(0,0,true,192,128);command=p3->input_source.update(dt);const auto intent=MathLibrary::stick_intent(GameplayInputLibrary::sample(GameplayAction::MoveX,0).value,0.0);assert(command.command.velocity.y==-intent.x*Fixed(6.0));
 // Logical capture independently blocks movement, jump and look; physics continues.
 auto capture=GameplayInputLibrary::capture(p3->id(),GameplayInputLibrary::movement_actions(),100,0);frame(right|cross);
 command=p3->input_source.update(dt);assert(command.command.velocity.x==0.0&&command.command.velocity.y==0.0&&command.command.jump);
 assert(!GameplayInputLibrary::sample(GameplayAction::Jump,0).pressed);assert(p3->motor.simulate(dt).jumped);
 const auto vertical=p3->motor.motion().velocity.y;p3->input_source.update(dt);assert(p3->motor.simulate(dt).velocity.y<vertical);
 assert(GameplayInputLibrary::release(capture));
 // A pending source jump is cancelled when dialogue takes the physical alias.
 p3->motor.reset_motion();p3->data()->transform.position[1]=1.0;p3->motor.simulate(dt);frame(0);frame(cross);p3->input_source.update(dt);
 capture=GameplayInputLibrary::capture(p3->id(),GameplayInputLibrary::mask(GameplayAction::Confirm),100,0);
 assert(GameplayInputLibrary::captured(GameplayAction::Jump,0));p3->input_source.update(dt);assert(!p3->motor.simulate(dt).jumped);GameplayInputLibrary::release(capture);
 for(unsigned port=0;port<4;++port){p3->input_source.port=port;p3->input_source.camera_relative=false;frame(right,port);assert(p3->input_source.update(dt).command.velocity.x==6.0);}
 p3->input_source.port=4;assert(p3->input_source.update(dt).status==CharacterCommandStatus::InvalidConfiguration);p3->input_source.port=0;
 input.sample(0,false,0);gameplay_controls().begin_frame();assert(p3->input_source.update(dt).status==CharacterCommandStatus::NoStep);
 frame(right|up);p2->body.floating=true;auto two=p2->input_source.update(dt);assert(two.command.velocity.x>0.0&&two.command.velocity.y<0.0&&!two.command.jump);
 p2->body.floating=false;two=p2->input_source.update(dt);assert(two.command.velocity.x==96.0&&two.command.velocity.y==0.0);
 p2->body.up_direction[0]=1.0;p2->body.up_direction[1]=0.0;two=p2->input_source.update(dt);assert(two.command.velocity.x==0.0&&two.command.velocity.y==96.0);
 p2->body.up_direction[0]=0.0;p2->body.up_direction[1]=-1.0;
 // NPC policies resolve world destinations, stop radii and per-step speed caps.
 n3->command_source.use_point=true;n3->command_source.goal={10.0,0.0,0.0};auto ai=n3->command_source.update(dt);assert(ai.command.velocity.x==3.0&&ai.command.heading==90.0);
 n3->command_source.stop_distance=0.25;n3->command_source.goal.x=0.5;ai=n3->command_source.update(1.0);assert(ai.command.velocity.x==0.25);
 n3->command_source.goal.x=0.1;assert(n3->command_source.update(dt).command.velocity.x==0.0);
 n3->command_source.target=p3->id();p3->data()->transform.position[0]=20.0;assert(n3->command_source.update(dt).command.velocity.x==3.0);
 p3->set_active(false);assert(n3->command_source.update(dt).status==CharacterCommandStatus::InvalidTarget);p3->set_active(true);
 n2->body.floating=true;n2->command_source.use_point=true;n2->command_source.goal={10.0,-10.0};two=n2->command_source.update(dt);assert(two.command.velocity.x>0.0&&two.command.velocity.y<0.0);
 n2->body.floating=false;two=n2->command_source.update(dt);assert(two.command.velocity.x==64.0&&two.command.velocity.y==0.0);
 n2->command_source.target=p2->id();p2->root.transform.position[0]=-100.0;assert(n2->command_source.update(dt).command.velocity.x==-64.0);
 // Scripted producers own only their most recently submitted motor revision.
 auto* script=level.add_component<ScriptedCommand3DComponent>(*p3,"Script");assert(script);script->automatic_tick=false;
 script->desired={{4.0,0.0},0.0,false,true};assert(script->update(0.0).status==CharacterCommandStatus::NoStep&&script->desired.jump);
 assert(script->update(dt).status==CharacterCommandStatus::Applied&&!script->desired.jump);script->update(dt);
 p3->motor.reset_motion();p3->data()->transform.position[1]=1.0;p3->motor.simulate(dt);
 script->desired.jump=true;script->update(dt);script->update(dt);assert(p3->motor.simulate(dt).jumped);
 p3->input_source.camera_relative=false;frame(right);p3->input_source.update(dt);assert(!script->release_command());
 assert(p3->motor.simulate(dt).velocity.x>0.0); // Old producer cannot stop the new one.
 script->desired={{-4.0,0.0},0.0,false,false};script->update(dt);assert(!p3->input_source.release_command());assert(script->release_command());
 script->desired.velocity.x=5000.0;assert(script->update(dt).status==CharacterCommandStatus::InvalidConfiguration);
 script->desired.velocity.x=1.0;script->motor=n3->motor.id();assert(script->update(dt).status==CharacterCommandStatus::Applied);script->set_enabled(false);assert(!script->release_command());script->set_enabled(true);
 auto* script2=level.add_component<ScriptedCommand2DComponent>(*p2,"Script");assert(script2);script2->automatic_tick=false;script2->desired={{12.0,0.0},true};assert(script2->update(dt).status==CharacterCommandStatus::Applied&&!script2->desired.jump);assert(script2->release_command());
 // Same-step ordering: producer comes after the embedded motor in composition.
 p2->input_source.automatic_tick=true;p2->motor.automatic_tick=true;p2->body.floating=true;frame(right);const auto x=p2->root.transform.position[0];level.tick(dt);assert(p2->root.transform.position[0]>x);p2->motor.automatic_tick=p2->input_source.automatic_tick=false;
 Level foreign;assert(foreign.bind(registry));auto* other=spawn<Character3D>(foreign);script->motor=other->motor.id();assert(script->update(dt).status==CharacterCommandStatus::InvalidMotor);
 n3->command_source.target=other->id();assert(n3->command_source.update(dt).status==CharacterCommandStatus::InvalidTarget);
 capture=GameplayInputLibrary::capture(p3->id(),GameplayInputLibrary::mask(GameplayAction::Jump),100,0);
 assert(GameplayInputLibrary::captured(GameplayAction::Jump,0));active_object_registry=nullptr;
 assert(!GameplayInputLibrary::captured(GameplayAction::Jump,0));active_object_registry=&registry;assert(!GameplayInputLibrary::valid(capture));
 // Reentry and destruction in either extension hook are safe registry transactions.
 for(unsigned action=0;action<=5;++action){auto* owner=spawn<Character3D>(level);owner->motor.automatic_tick=false;auto* source=level.add_component<CallbackSource>(*owner,"Callback");assert(source);source->automatic_tick=false;source->motor=owner->motor.id();source->action=action;const auto id=source->id();const auto result=source->update(dt);
  if(action==1||action==2){assert(result.status==CharacterCommandStatus::InvalidOwner);level.tick(0.0);assert(!registry.get(id));}
  else if(action==3)assert(result.status==CharacterCommandStatus::InvalidMotor);
  else if(action==4)assert(result.status==CharacterCommandStatus::NoStep);
  else if(action==5)assert(result.status==CharacterCommandStatus::Applied&&!registry.get(id));
  else assert(result.status==CharacterCommandStatus::Applied&&source->nested==CharacterCommandStatus::Busy&&source->submitted==1);
 }
 foreign.end_play_all(EndPlayReason::LevelUnloaded);level.end_play_all(EndPlayReason::LevelUnloaded);
 std::puts("Character producers: Player/NPC compositions, input/capture, movement, destination policies, lease handoff, callbacks and destruction passed");
}
