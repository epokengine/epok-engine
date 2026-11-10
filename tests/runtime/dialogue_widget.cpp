#define EPOK_GAMEPLAY_CONTROL 1
#define EPOK_INTERACTION 1
#define EPOK_DIALOGUE 1
#define EPOK_UI_ANCHOR 1
#include <cassert>
#include <cstdio>
#include <cstring>
#include "../../runtime/dialogue_helpers.hpp"
#include "../../runtime/ui_anchor_service.hpp"
using namespace epok;
namespace d=epok::dialogue;
static ActorData data[16];static unsigned used=0;
static unsigned opened=0,closed=0,source_started=0,source_finished=0;
static DialogueSnapshot opening{},ending{};
static unsigned open_action=0;
static bool closing_animation=false;

class Widget final:public DialoguePresenter {
public:
 void dialogue_opened(DialogueSnapshot state) override {
  ++opened;opening=state;
  d::Start options;options.owner=state.source;options.initiator=state.initiator;options.presenter=state.presenter;
  assert(d::session().start(*d::resources[0].table,options)==DialogueResult::Busy);
  if(open_action==1)interaction::actor(state.source)->set_active(false);
  if(open_action==2)set_active(false);
  if(open_action==3)interaction::actor(state.source)->destroy();
  if(open_action==4)destroy();
 }
 void dialogue_closed(DialogueSnapshot state) override {
  ++closed;ending=state;
  assert(state.source==opening.source&&state.initiator==opening.initiator&&state.presenter==opening.presenter&&state.generation==opening.generation);
  assert(!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(state.source,GameplayInputLibrary::all_actions(),state.port)));
  for(auto reference:{body,prompt,continuation})if(auto* actor=interaction::actor(reference))assert(!actor->data()->text.enabled);
  d::Start options;options.owner=state.source;options.initiator=state.initiator;options.presenter=state.presenter;
  assert(d::session().start(*d::resources[0].table,options)==DialogueResult::Busy);
  if(closing_animation)interaction::actor(body)->data()->text.enabled=true;
 }
};
class Endpoint final:public DialogueInteractionComponent {
public:
 void dialogue_started(ObjectId,uint32_t,DialogueHandle) override{++source_started;}
 void dialogue_finished(ObjectId,uint32_t,DialogueResult) override{++source_finished;}
};
namespace epok {
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing() const{return false;}
bool scene_loading(){return false;}
bool camera_project(const Fixed*,Fixed*){return false;}
WorldAffineSample gameplay_world_affine(const ActorData*){WorldAffineSample result;result.success=true;return result;}
int entity_index(const ActorData* value){for(unsigned i=0;i<used;++i)if(value==data+i)return int(i);return -1;}
template<class T>ClassDescriptor actor(uint64_t parent,ObjectDomain domain){return {T::static_class_id,parent,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,16>::acquire,&ObjectPool<T,16>::release};}
template<class T>ClassDescriptor component(uint64_t parent,ObjectDomain domain,uint8_t owners,uint8_t flags=0){return {T::static_class_id,parent,ObjectFamily::Component,domain,owners,flags,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,16>::acquire,&ObjectPool<T,16>::release};}
static ClassDescriptor widget(){auto result=actor<Widget>(UIActor::static_class_id,ObjectDomain::UI);result.default_component_count=3;result.default_component=[](Object& object,size_t index)->NativeComponentDefault{auto& view=static_cast<DialoguePresenter&>(object);switch(index){case 0:return {&view.root,"Root",true,-1,RectTransformComponent::static_class_id};case 1:return {&view.canvas,"Canvas",false,-1,CanvasComponent::static_class_id};case 2:return {&view.audio,"Audio",false,-1,AudioComponent::static_class_id};default:return {};}};return result;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {UIComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassAbstract},
 component<SceneComponent3D>(ActorComponent::static_class_id,ObjectDomain::World3D,1,ObjectClassRoot),
 component<RectTransformComponent>(UIComponent::static_class_id,ObjectDomain::UI,4,ObjectClassRoot),
 component<CanvasComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<AudioComponent>(ActorComponent::static_class_id,ObjectDomain::None,7),
 component<TextComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<ImageComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<FocusableComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<ScreenAnchorComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 {InteractableComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::None,3,ObjectClassAbstract},
 component<Endpoint>(InteractableComponent::static_class_id,ObjectDomain::None,3),
 actor<Actor3D>(Actor::static_class_id,ObjectDomain::World3D),actor<UIActor>(Actor::static_class_id,ObjectDomain::UI),widget(),
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
namespace dialogue {
static const Page line[]={{"Hello",5}},option[]={{"Option",6}},prompt[]={{"Question A",10},{"Question B",10}};
static const uint16_t offsets[]={0,1},prompt_offsets[]={0,2};
static const Text texts[]={{line,offsets,1,1},{option,offsets,1,1},{prompt,prompt_offsets,2,1}};
static const Option options[]={{1,1,2},{2,1,2},{3,1,2},{4,1,2},{5,1,2},{6,1,2}};
static const Node nodes[]={{10,Kind::Line,no_role,0,1},{20,Kind::Choice,no_role,2,missing,missing,missing,options,6},{30,Kind::End}};
static const Conversation table={1,nullptr,0,nullptr,0,texts,3,nodes,3,0};
static const Style style=[](){Style result;result.timing.reveal_ms=0;return result;}();
const Resource resources[]={{1,0,&table,&style,77},{1,2,&table,&style,77}};const size_t resource_count=2;
}
}
struct Fixture {
 ObjectRegistryStorage<128> registry;Level level;
 ObjectId source{},player{},view_id{},endpoint_id{};
 DialoguePresenter* view=nullptr;DialogueInteractionComponent* endpoint=nullptr;
 ActorData *panel=nullptr,*body=nullptr,*prompt=nullptr,*continuation=nullptr;
 ObjectId spawn(uint64_t type){auto id=level.spawn_actor(*find_object_class(type),"Widget test");assert(id.valid());auto* actor=registry.resolve<Actor>(id);assert(used<16);data[used]=ActorData{};data[used].alive=data[used].active=true;data[used].parent=-1;actor->bind_data(data[used++]);return id;}
 ObjectId add(bool image){const auto id=spawn(UIActor::static_class_id);auto* owner=registry.resolve<Actor>(id);ActorComponent* component=image?static_cast<ActorComponent*>(level.add_component<ImageComponent>(*owner,"Control")):static_cast<ActorComponent*>(level.add_component<TextComponent>(*owner,"Control"));assert(component);auto* slot=owner->data();slot->rect.enabled=true;slot->rect.position[0]=17.0;slot->rect.position[1]=23.0;slot->rect.size[0]=128.0;slot->rect.size[1]=32.0;slot->rect.rotation=12.0;slot->text.enabled=false;slot->text.font=4;slot->text.color[0]=11;slot->text.color[1]=22;slot->text.color[2]=33;slot->text.align=TextAlign::Right;slot->text.wrap=false;slot->image.texture=8;slot->image.borders[0]=3;return component->id();}
 Fixture(){
  used=opened=closed=source_started=source_finished=0;open_action=0;closing_animation=false;assert(level.bind(registry));
  source=spawn(Actor3D::static_class_id);player=spawn(Actor3D::static_class_id);view_id=spawn(DialoguePresenter::static_class_id);view=registry.resolve<DialoguePresenter>(view_id);assert(view);view->default_timing.reveal_ms=0;
  view->panel=add(true);view->portrait=add(true);view->choice_panel=add(true);view->speaker_name=add(false);view->body=add(false);view->choice0=add(false);view->choice1=add(false);view->choice2=add(false);view->choice3=add(false);view->prompt=add(false);view->continuation=add(false);
  panel=interaction::actor(view->panel)->data();body=interaction::actor(view->body)->data();prompt=interaction::actor(view->prompt)->data();continuation=interaction::actor(view->continuation)->data();continuation->text.set_text("NEXT");
  auto* focus=level.add_component<FocusableComponent>(*interaction::actor(view->choice0),"Choice focus");assert(focus&&!focus->outline_enabled());focus->set_outline(true);assert(focus->outline_enabled());focus->set_outline(false);assert(!focus->outline_enabled());
  d::bind_widget_layout(*view,77);
  endpoint=level.add_component<DialogueInteractionComponent>(*registry.resolve<Actor>(source),"Dialogue");assert(endpoint);endpoint_id=endpoint->id();endpoint->conversation=1;endpoint->presenter=view_id;endpoint->automatic_input=false;
 }
 ~Fixture(){d::session().reset();level.end_play_all(EndPlayReason::LevelUnloaded);registry.release(level.id());active_object_registry=nullptr;}
 DialogueStartResult start(){return DialogueSessionLibrary::start(endpoint_id,player,0);}
 void unchanged(){for(auto id:{view->body,view->prompt,view->continuation,view->speaker_name,view->choice0,view->choice1,view->choice2,view->choice3}){auto* slot=interaction::actor(id)->data();assert(slot->rect.position[0].raw()==17*4096&&slot->rect.position[1].raw()==23*4096&&slot->rect.size[0].raw()==128*4096&&slot->rect.size[1].raw()==32*4096&&slot->rect.rotation.raw()==12*4096);assert(slot->text.font==4&&slot->text.color[0]==11&&slot->text.color[1]==22&&slot->text.color[2]==33&&slot->text.align==TextAlign::Right&&!slot->text.wrap);}assert(panel->image.texture==8&&panel->image.borders[0]==3);assert(!strcmp(continuation->text.value,"NEXT"));}
};
int main(){
 {
  Fixture f;f.view->default_timing.reveal_ms=25;const auto started=f.start();assert(started.result==DialogueResult::Accepted);
  assert(d::session().snapshot().phase==d::Phase::Revealing);
  d::session().update(25);assert(d::session().snapshot().revealed==1);
  DialogueSessionLibrary::cancel(started.session);
  f.endpoint->style=2;const auto overridden=f.start();assert(overridden.result==DialogueResult::Accepted&&d::session().snapshot().phase==d::Phase::Waiting);
  DialogueSessionLibrary::cancel(overridden.session);
  f.endpoint->style=0;f.view->default_timing.reveal_ms=1001;assert(f.start().result==DialogueResult::MissingPresenter);
  f.view->default_timing.reveal_ms=0;f.view->default_timing.auto_advance_ms=3600001;assert(f.start().result==DialogueResult::MissingPresenter);
  assert(!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(f.endpoint_id,GameplayInputLibrary::all_actions(),0)));
 }
 {
  Fixture f;auto* owner=f.registry.resolve<Actor>(f.view_id);owner->data()->canvas.enabled=true;
  auto* child=f.registry.resolve<Actor>(f.spawn(UIActor::static_class_id));child->data()->rect.enabled=true;
  child->data()->rect.size[0]=100.0;child->data()->rect.size[1]=40.0;child->data()->parent=entity_index(owner->data());
  auto* anchor=f.level.add_component<ScreenAnchorComponent>(*owner);assert(anchor);anchor->content=child->root_id();f.view->anchor=anchor->id();
  const auto started=f.start();assert(started.result==DialogueResult::Accepted&&anchor->target==f.source&&anchor->status()==ScreenAnchorStatus::Fallback);
  DialogueSessionLibrary::cancel(started.session);assert(!anchor->target.valid());const auto x=child->data()->rect.position[0];anchor->on_frame(16);assert(child->data()->rect.position[0]==x);
  const auto again=f.start();assert(again.result==DialogueResult::Accepted);child->data()->rect.enabled=false;f.view->on_frame(16);assert(!DialogueSessionLibrary::valid(again.session)&&!anchor->target.valid());
  assert(!GameplayInputLibrary::valid(GameplayInputLibrary::owner_capture(f.endpoint_id,GameplayInputLibrary::all_actions(),0)));
 }
 {
  Fixture f;const auto started=f.start();assert(started.result==DialogueResult::Accepted&&opened==1&&closed==0&&source_started==1);
  assert(opening.phase==DialoguePhase::Executing&&opening.generation==started.session.generation);
  f.view->on_frame(1000);assert(f.body->text.enabled&&!f.prompt->text.enabled&&f.continuation->text.enabled);assert(!interaction::actor(f.view->choice_panel)->data()->image.enabled);f.unchanged();
  assert(DialogueSessionLibrary::advance(started.session)==DialogueResult::Accepted);f.view->on_frame(1000);
  assert(!f.body->text.enabled&&f.prompt->text.enabled&&f.continuation->text.enabled&&!strcmp(f.prompt->text.value,"Question A"));f.unchanged();
  DialogueSessionLibrary::advance(started.session);f.view->on_frame(1000);assert(!strcmp(f.prompt->text.value,"Question B"));
  DialogueSessionLibrary::advance(started.session);f.view->on_frame(1000);assert(f.prompt->text.enabled&&!f.continuation->text.enabled&&interaction::actor(f.view->choice0)->data()->text.enabled);assert(interaction::actor(f.view->choice_panel)->data()->image.enabled);
  DialogueSessionLibrary::move_selection(started.session,4);f.view->on_frame(1000);f.unchanged();assert(f.view->canvas.focused()==entity_index(interaction::actor(f.view->choice0)->data()));
  assert(DialogueSessionLibrary::cancel(started.session)==DialogueResult::Accepted&&closed==1&&source_finished==1&&ending.result==DialogueResult::UserCancelled&&ending.phase==DialoguePhase::Cancelled);
  assert(!f.body->text.enabled&&!f.prompt->text.enabled&&!f.continuation->text.enabled&&!f.panel->image.enabled);assert(!interaction::actor(f.view->choice_panel)->data()->image.enabled);f.view->on_frame(1000);assert(closed==1);f.unchanged();
  const auto completed=f.start();for(unsigned i=0;i<4;++i)DialogueSessionLibrary::advance(completed.session);
  assert(closed==2&&source_finished==2&&ending.result==DialogueResult::Completed&&ending.phase==DialoguePhase::Completed);
  closing_animation=true;const auto animated=f.start();DialogueSessionLibrary::cancel(animated.session);assert(f.body->text.enabled);f.view->on_frame(1000);assert(f.body->text.enabled&&closed==3);closing_animation=false;
 }
 for(unsigned action=1;action<=4;++action){
  Fixture f;open_action=action;const auto started=f.start();
  assert(started.result==((action==1||action==3)?DialogueResult::SourceUnavailable:DialogueResult::PresenterUnavailable));
  assert(opened==1&&closed==1&&source_started==0&&source_finished==((action==1||action==3)?0u:1u));
  assert(!d::session().active()&&!f.body->text.enabled&&!f.prompt->text.enabled&&!f.continuation->text.enabled);
  if(action==1){f.registry.resolve<Actor>(f.source)->set_active(true);f.view->on_frame(1000);assert(!f.body->text.enabled&&closed==1);}
  if(action==2){f.registry.resolve<Actor>(f.view_id)->set_active(true);f.view->on_frame(1000);assert(!f.body->text.enabled&&closed==1);}
 }
 {
  Fixture f;const auto started=f.start();f.view->on_frame(1000);interaction::actor(f.view->body)->set_active(false);d::session().prune();f.view->on_frame(1000);
  assert(!DialogueSessionLibrary::valid(started.session)&&closed==1&&!f.body->text.enabled);
  interaction::actor(f.view->body)->set_active(true);f.view->on_frame(1000);assert(!f.body->text.enabled);
 }
 {
  Fixture f;const auto started=f.start();DialogueSessionLibrary::advance(started.session);f.view->on_frame(1000);
  auto* question=interaction::actor(f.view->prompt);question->set_active(false);f.view->on_frame(1000);
  assert(DialogueSessionLibrary::valid(started.session)&&!f.body->text.enabled);
  DialogueSessionLibrary::advance(started.session);question->set_active(true);f.view->on_frame(1000);
  assert(!f.body->text.enabled&&f.prompt->text.enabled&&!strcmp(f.prompt->text.value,"Question B"));
  DialogueSessionLibrary::cancel(started.session);assert(closed==1&&!f.prompt->text.enabled);
 }
 {
  Fixture f;f.start();d::session().reset();assert(closed==1&&ending.result==DialogueResult::LevelUnloaded);f.view->on_frame(1000);assert(closed==1);
 }
 {
  Fixture f;f.start();
  ObjectRegistryStorage<128> other_registry;Level other_level;assert(other_level.bind(other_registry));
  d::session().prune();assert(!d::session().active());
  ActorData other_data[3];unsigned count=0;
  auto spawn=[&](uint64_t type){const auto id=other_level.spawn_actor(*find_object_class(type),"Other registry");assert(id.valid());other_data[count].alive=other_data[count].active=true;other_registry.resolve<Actor>(id)->bind_data(other_data[count++]);return id;};
  d::Start options;options.owner=spawn(Actor3D::static_class_id);options.initiator=spawn(Actor3D::static_class_id);options.presenter=spawn(UIActor::static_class_id);options.automatic_input=false;
  assert(options.presenter==f.view_id);assert(d::session().start(*d::resources[0].table,options)==DialogueResult::Accepted);const auto generation=d::session().snapshot().generation;
  f.view->on_frame(1000);assert(d::session().active()&&d::session().snapshot().generation==generation&&closed==0);
  d::session().reset();other_level.end_play_all(EndPlayReason::LevelUnloaded);other_registry.release(other_level.id());active_object_registry=&f.registry;
 }
 std::puts("dialogue widget: authored controls, paged prompts, continuation, lifecycle/reentry, destruction and close animation passed");
}
