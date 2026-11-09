#define EPOK_GAMEPLAY_CONTROL 1
#define EPOK_INTERACTION 1
#define EPOK_DIALOGUE 1
#include <cassert>
#include <cstdio>
#include "../../runtime/dialogue.hpp"
using namespace epok;
namespace d=epok::dialogue;
namespace epok {
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing() const{return false;}
bool scene_loading(){return false;}
// This session fixture owns no ActorData slots; identity/lifetime use Objects.
int entity_index(const ActorData*){return -1;}
WorldAffineSample gameplay_world_affine(const ActorData*){return {};}
template<class T> ClassDescriptor actor(ObjectDomain domain){return {T::static_class_id,Actor::static_class_id,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,16>::acquire,&ObjectPool<T,16>::release};}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot,&object_construct<SceneComponent3D>,&object_destruct,sizeof(SceneComponent3D),alignof(SceneComponent3D)},
 {RectTransformComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassRoot,&object_construct<RectTransformComponent>,&object_destruct,sizeof(RectTransformComponent),alignof(RectTransformComponent)},
 actor<Actor3D>(ObjectDomain::World3D),actor<UIActor>(ObjectDomain::UI),
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
static d::Session* session=nullptr;
static bool allowed=true,destroy_on_action=false;
static int entered=0,actions=0,completed=0;
static unsigned callback_order=0,start_order=0,entry_order=0,finish_order=0;
static bool disable_on_start=false;
static d::Result nested=d::Result::Accepted,last=d::Result::NoSession;
static bool condition(ObjectId,ObjectId){return allowed;}
static void action(ObjectId receiver,ObjectId,uint32_t){++actions;nested=session->advance();if(destroy_on_action)active_object_registry->resolve<Actor>(receiver)->destroy();}
static void on_start(ObjectId source,ObjectId,uint32_t,uint32_t generation){start_order=++callback_order;assert(session->snapshot().generation==generation);assert(session->cancel()==d::Result::Busy);if(disable_on_start)active_object_registry->resolve<Actor>(source)->set_active(false);}
static void on_enter(ObjectId,uint32_t){++entered;if(!entry_order)entry_order=++callback_order;}
static void on_finish(ObjectId,ObjectId,ObjectId,uint32_t,d::Result result){++completed;finish_order=++callback_order;last=result;nested=session->cancel();}
static void input_frame(uint16_t buttons){input.sample(0,true,buttons);gameplay_controls().begin_frame();}

int main(){
 ObjectRegistryStorage<64> registry;Level level;assert(level.bind(registry));
 auto spawn=[&](uint64_t type=Actor3D::static_class_id){const auto id=level.spawn_actor(*find_object_class(type),"Dialogue role");assert(id.valid());return id;};
 const auto owner=spawn(),player=spawn(),rules=spawn(),presenter=spawn(UIActor::static_class_id);
 auto& value=d::session();session=&value;value.reset();
 const d::Page pages[]={{"[color=#ff0000]Hola[/color]",4},{"Again",5},{"Si",2},{"Yes",3}};
 const uint16_t offsets[]={0,1,2};
 const uint16_t option_offsets[]={0,1};
 const d::Text texts[]={{pages,offsets,2,2},{pages+2,option_offsets,1,1}};
 const d::Role roles[]={{101}};
 const d::Assignment assignments[]={{101,rules}};
 const d::Binding bindings[]={{0,Actor3D::static_class_id,&condition,nullptr},{0,Actor3D::static_class_id,nullptr,&action}};
 d::Node nodes[5]{};
 nodes[0].id=10;nodes[0].kind=d::Kind::Line;nodes[0].text=0;nodes[0].next=1;
 nodes[1].id=11;nodes[1].kind=d::Kind::Branch;nodes[1].binding=0;nodes[1].next=2;nodes[1].otherwise=4;
 nodes[2].id=12;nodes[2].kind=d::Kind::Event;nodes[2].binding=1;nodes[2].next=4;
 nodes[3].id=13;nodes[3].kind=d::Kind::Choice;
 nodes[4].id=14;nodes[4].kind=d::Kind::End;
 d::Conversation table{42,roles,1,bindings,2,texts,2,nodes,5,0};
 d::Start start;start.owner=owner;start.initiator=player;start.presenter=presenter;start.roles=assignments;start.role_count=1;start.entered=&on_enter;start.finished=&on_finish;
 input.reset();gameplay_controls().reset();input_frame(0);
 input_frame(1u<<unsigned(Button::Cross));
 const auto opened=value.start(table,start);
 assert(opened==d::Result::Accepted&&value.active());
 assert(value.start(table,start)==d::Result::Busy&&entered==1);
 value.update(0);assert(value.snapshot().phase==d::Phase::Revealing&&value.snapshot().revealed==0);
 assert(!GameplayInputLibrary::frame_sample(GameplayAction::Jump,0).pressed);
 value.update(60);assert(value.snapshot().revealed==2);
 assert(value.advance()==d::Result::Accepted&&value.snapshot().phase==d::Phase::Waiting);
 assert(value.advance()==d::Result::Completed&&!value.active());
 assert(actions==1&&completed==1&&last==d::Result::Completed&&nested==d::Result::Busy&&entered==4);
 assert(value.text()[0]==0&&!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0)));

 // Each press is separate. The opening/closing Cross cannot escape into Jump.
 input_frame(0);assert(value.start(table,start)==d::Result::Accepted);
 input_frame(1u<<unsigned(Button::Cross));value.update(0);assert(value.snapshot().phase==d::Phase::Waiting);
 input_frame(0);value.update(0);input_frame(1u<<unsigned(Button::Cross));value.update(0);
 assert(!value.active());input.begin_tick();gameplay_controls().begin_tick();assert(!GameplayInputLibrary::take_pressed(GameplayAction::Jump,0));
 input_frame(0);assert(value.start(table,start)==d::Result::Accepted);
 localization.language=1;value.update(0);assert(value.snapshot().glyphs==5&&value.text()[0]=='A');
 assert(value.cancel()==d::Result::Accepted&&last==d::Result::UserCancelled);
 localization.language=0;

 // More than four choices use the same fixed candidate array and page selection.
 d::Option options[6];for(unsigned i=0;i<6;++i)options[i]={200+i,1,4,d::missing};
 nodes[3].options=options;nodes[3].option_count=6;table.entry=3;
 input_frame(0);assert(value.start(table,start)==d::Result::Accepted&&value.snapshot().phase==d::Phase::Selecting);
 assert(value.choice_id(3)==203&&value.choice_id(4)==0);
 assert(value.move_selection(4)==d::Result::Accepted&&value.snapshot().choice_page==1&&value.choice_id(0)==204&&value.choice_id(2)==0);
 assert(value.advance()==d::Result::Completed);
 options[0].binding=0;nodes[3].option_count=1;
 allowed=false;assert(value.start(table,start)==d::Result::NoChoices&&!value.active());allowed=true;
 assert(!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0)));

 // A non-yielding graph fails at the fixed transition budget, not a stack loop.
 nodes[1].next=1;table.entry=1;const int before=entered;
 assert(value.start(table,start)==d::Result::TransitionBudget&&entered-before==32);
 nodes[1].next=2;table.entry=2;
 const auto dead_rules=rules;destroy_on_action=true;
 const auto destroyed=value.start(table,start);
 assert(destroyed==d::Result::SourceUnavailable&&!interaction::live(dead_rules)&&!value.active());
 assert(!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0))&&value.text()[0]==0);destroy_on_action=false;

 // Independent roles reject mismatched bindings before entering a node.
 const d::Assignment wrong[]={{102,owner}};start.roles=wrong;
 assert(value.start(table,start)==d::Result::InvalidBinding);start.role_count=0;start.roles=nullptr;
 table.role_count=0;table.binding_count=0;table.entry=0;
 assert(value.start(table,start)==d::Result::Accepted);
 registry.resolve<Actor>(presenter)->set_active(false);assert(last==d::Result::PresenterUnavailable&&!value.active());
 registry.resolve<Actor>(presenter)->set_active(true);
 // Automatic pages yield, and page advances do not repeat node-entry events.
 const uint16_t multiple_offsets[]={0,2};const d::Text multiple[]={{pages,multiple_offsets,2,1}};
 table.texts=multiple;table.text_count=1;nodes[0].next=4;
 start.timing.reveal_ms=0;start.timing.auto_ms=20;
 input_frame(0);const auto before_pages=entered;
 assert(value.start(table,start)==d::Result::Accepted&&value.snapshot().pages==2&&value.snapshot().phase==d::Phase::Waiting);
 value.update(20);assert(value.snapshot().page==1&&value.active()&&entered==before_pages+1);
 value.update(20);assert(!value.active()&&last==d::Result::Completed&&entered==before_pages+2);
 start.timing={};table.texts=texts;table.text_count=2;
 // A forced source cancellation also consumes the current frame's button edge.
 input_frame(0);assert(value.start(table,start)==d::Result::Accepted);
 input_frame(1u<<unsigned(Button::Cross));registry.resolve<Actor>(owner)->set_active(false);
 assert(!value.active()&&!GameplayInputLibrary::frame_sample(GameplayAction::Jump,0).pressed);
 registry.resolve<Actor>(owner)->set_active(true);input_frame(0);
 // Pool exhaustion is explicit and does not replace the previous session snapshot.
 InputCapture captures[16];for(auto& capture:captures){capture=GameplayInputLibrary::capture(owner,GameplayInputLibrary::all_actions(),1,0);assert(GameplayInputLibrary::valid(capture));}
 assert(value.start(table,start)==d::Result::CaptureExhausted&&!value.active());
 for(auto capture:captures)assert(GameplayInputLibrary::release(capture));
 assert(value.start(table,start)==d::Result::Accepted);
 assert(GameplayInputLibrary::release(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0)));
 value.update(0);assert(!value.active()&&last==d::Result::CaptureLost&&value.snapshot().phase==d::Phase::Cancelled);
 assert(value.start(table,start)==d::Result::Accepted);value.reset();assert(value.snapshot().phase==d::Phase::Idle&&!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0)));
 // Public handles retain completed state, but never control a later session.
 assert(value.start(table,start)==d::Result::Accepted);
 const auto old=DialogueSessionLibrary::owner_session(owner);
 assert(old.generation&&DialogueSessionLibrary::valid(old));
 const auto snapshot=DialogueSessionLibrary::snapshot(old);
 assert(snapshot.source==owner&&snapshot.initiator==player&&snapshot.presenter==presenter&&snapshot.node==10);
 assert(!DialogueSessionLibrary::owner_session(player).generation);
 assert(DialogueSessionLibrary::cancel(old)==DialogueResult::Accepted);
 assert(!DialogueSessionLibrary::valid(old)&&DialogueSessionLibrary::snapshot(old).phase==DialoguePhase::Cancelled);
 assert(value.start(table,start)==d::Result::Accepted);
 const auto latest=DialogueSessionLibrary::owner_session(owner);
 assert(latest.generation>old.generation);
 const auto before_stale=value.snapshot();
 assert(DialogueSessionLibrary::advance(old)==DialogueResult::NoSession);
 assert(DialogueSessionLibrary::cancel(old)==DialogueResult::NoSession);
 assert(DialogueSessionLibrary::move_selection(old,1)==DialogueResult::NoSession);
 assert(!DialogueSessionLibrary::choice_id(old,0));
 assert(value.active()&&value.snapshot().generation==latest.generation&&value.snapshot().revealed==before_stale.revealed);
 assert(DialogueSessionLibrary::snapshot(old).result==DialogueResult::NoSession);
 value.reset();assert(!DialogueSessionLibrary::valid(latest));
 // The public opening event precedes even an immediate End. Reentry is busy;
 // disabling the source from that event skips entry and releases capture once.
 start.started=&on_start;table.entry=4;callback_order=start_order=entry_order=finish_order=0;
 const auto before_immediate=completed;
 assert(value.start(table,start)==d::Result::Completed);
 assert(start_order==1&&entry_order==2&&finish_order==3&&completed==before_immediate+1);
 callback_order=start_order=entry_order=finish_order=0;disable_on_start=true;table.entry=0;
 assert(value.start(table,start)==d::Result::SourceUnavailable);
 assert(start_order==1&&entry_order==0&&finish_order==2&&completed==before_immediate+2);
 assert(!value.active()&&!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(owner,GameplayInputLibrary::all_actions(),0)));
 registry.resolve<Actor>(owner)->set_active(true);disable_on_start=false;start.started=nullptr;
 DialogueRoleBindingsComponent mapping;
 for(unsigned i=0;i<8;++i){assert(mapping.set_binding(i,100+i,player));assert(mapping.role_id(i)==100+i&&mapping.receiver(i)==player);}
 assert(!mapping.set_binding(8,999,owner)&&mapping.role_id(7)==107&&mapping.receiver(7)==player);
 assert(mapping.role_id(8)==0&&!mapping.receiver(8).valid());
 level.end_play_all(EndPlayReason::LevelUnloaded);registry.release(level.id());active_object_registry=nullptr;
 std::printf("Dialogue sessions: capture edges, reveal, language, choices, callbacks, lifecycle and transition bounds pass; %zu bytes host state.\n",sizeof(value));
}
