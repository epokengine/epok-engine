// Compile the same runtime in a private namespace: editor asset previews have
// deliberately empty scene services, and must never share a session or registry
// with the isolated conversation preview. No project class is linked here.
#define epok epok_dialogue_preview_runtime
#define EpokHudSink EpokDialogueHudSink
#define epok_hud_rect epok_dialogue_hud_rect
#define EPOK_GAMEPLAY_CONTROL 1
#define EPOK_INTERACTION 1
#define EPOK_DIALOGUE 1
#define EPOK_TEXT_REVEAL 1
#include "dialogue_helpers.hpp"
#include "hud_nodes.hpp"
#undef epok_hud_rect
#undef EpokHudSink
#undef epok
#include "dialogue_preview.h"
#include <array>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <cstdio>

namespace r=epok_dialogue_preview_runtime;
namespace d=r::dialogue;
namespace epok_dialogue_preview_runtime {
static ActorData* preview_data=nullptr;
static size_t preview_count=0;
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing()const{return false;}
bool scene_loading(){return false;}
WorldAffineSample gameplay_world_affine(const ActorData*){return {};}
int entity_index(const ActorData* value){for(size_t i=0;i<preview_count;++i)if(value==preview_data+i)return int(i);return -1;}
DataHandle handle(const ActorData* value){const int index=entity_index(value);return index>=0&&value->alive?DataHandle{uint16_t(index),value->generation}:DataHandle{};}
ActorData* DataHandle::get()const{return index<preview_count&&preview_data[index].alive&&preview_data[index].generation==generation?preview_data+index:nullptr;}
bool is_active(const ActorData* value){return value&&value->alive&&value->owner&&is_active(value->owner);}
template<class T,size_t N=64>ClassDescriptor actor(uint64_t parent,ObjectDomain domain){return {T::static_class_id,parent,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,N>::acquire,&ObjectPool<T,N>::release};}
template<class T>ClassDescriptor component(uint64_t parent,ObjectDomain domain,uint8_t owners,uint8_t flags=0){return {T::static_class_id,parent,ObjectFamily::Component,domain,owners,flags,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,64>::acquire,&ObjectPool<T,64>::release};}
static ClassDescriptor widget(){
 auto result=actor<DialoguePresenter,1>(UIActor::static_class_id,ObjectDomain::UI);
 result.default_component_count=3;
 result.default_component=[](Object& object,size_t index)->NativeComponentDefault{
  auto& view=static_cast<DialoguePresenter&>(object);
  switch(index){case 0:return {&view.root,"Root",true,-1,RectTransformComponent::static_class_id};case 1:return {&view.canvas,"Canvas",false,-1,CanvasComponent::static_class_id};case 2:return {&view.audio,"Audio",false,-1,AudioComponent::static_class_id};default:return {};}
 };
 return result;
}
// Only native SDK bases participate in preview. This is not the project's class
// catalog; derived provider code is exercised by explicit Play instead.
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
 {InteractableComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::None,3,ObjectClassAbstract},
 component<DialogueInteractionComponent>(InteractableComponent::static_class_id,ObjectDomain::None,3),
 actor<Actor3D,1>(Actor::static_class_id,ObjectDomain::World3D),actor<UIActor>(Actor::static_class_id,ObjectDomain::UI),widget(),
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
namespace dialogue {
static Conversation preview_table;
static Style preview_style;
const Resource resources[]={{1,0,&preview_table,&preview_style,0}};
const size_t resource_count=1;
}
}
namespace {
std::mutex preview_mutex;
const EpokDialogueGraph* callbacks=nullptr;
std::vector<EpokDialogueTrace>* trace=nullptr;
void entered(r::ObjectId,uint32_t node){trace->push_back({0,node,node});}
template<size_t I>bool condition(r::ObjectId,r::ObjectId){return callbacks->bindings[I].value!=0;}
template<size_t I>void action(r::ObjectId,r::ObjectId,uint32_t node){trace->push_back({1,callbacks->bindings[I].id,node});}
template<size_t... I>constexpr auto conditions(std::index_sequence<I...>){return std::array<d::Condition,sizeof...(I)>{&condition<I>...};}
template<size_t... I>constexpr auto actions(std::index_sequence<I...>){return std::array<d::Action,sizeof...(I)>{&action<I>...};}
constexpr auto condition_table=conditions(std::make_index_sequence<64>{});
constexpr auto action_table=actions(std::make_index_sequence<64>{});
void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
uint16_t index(int32_t value){return value<0?d::missing:uint16_t(value);}
struct World {
 r::ObjectRegistryStorage<512> registry;
 r::Level level;
 std::vector<r::ActorData> data;
 explicit World(const EpokDialogueRequest& input):data(epok_hud_entities(input.nodes,input.count)){
  data.emplace_back();r::preview_data=data.data();r::preview_count=data.size();
  require(level.bind(registry),"Preview level could not bind");
 }
 ~World(){d::session().reset();level.end_play_all(r::EndPlayReason::LevelUnloaded);registry.release(level.id());r::active_object_registry=nullptr;r::preview_data=nullptr;r::preview_count=0;callbacks=nullptr;trace=nullptr;}
 r::Actor* spawn(uint64_t type,size_t slot,bool active=true){
  const auto id=level.spawn_actor(*r::find_object_class(type),"Preview");auto* actor=registry.resolve<r::Actor>(id);
  require(actor,"Widget and preview source exceed the runtime actor budget");
  actor->bind_data(data[slot]);require(level.set_active(id,active),"Preview actor activation failed");return actor;
 }
};
}
extern "C" uint32_t epok_dialogue_preview(const EpokDialogueRequest* input,EpokHudCommand* output,uint32_t capacity,uint32_t* stats,EpokDialogueSnapshot* snapshot,EpokDialogueTrace* output_trace,uint32_t trace_capacity,char* error,uint32_t error_capacity){
 std::lock_guard<std::mutex> lock(preview_mutex);
 if(error_capacity)error[0]=0;*snapshot={};std::fill_n(stats,6,0);
 try {
  require(input&&input->nodes&&input->components&&input->logical_parents&&input->count&&input->count<64&&input->root<input->count,"Invalid widget preview actor count or root (the conversation also needs one source actor)");
  const auto& graph=input->graph;
  require(graph.nodes&&graph.node_count&&graph.node_count<=1024&&graph.entry<graph.node_count&&graph.role_count<=8&&graph.binding_count<=64,"Invalid preview conversation table");
  std::vector<d::Page> pages;pages.reserve(graph.page_count);
  for(unsigned i=0;i<graph.page_count;++i){require(graph.pages[i].text&&graph.pages[i].glyphs<=65535,"Invalid preview page");pages.push_back({graph.pages[i].text,uint16_t(graph.pages[i].glyphs)});}
  std::vector<std::array<uint16_t,2>> offsets(graph.text_count);
  std::vector<d::Text> texts;std::vector<d::Role> roles;std::vector<d::Binding> bindings;std::vector<d::Option> options;std::vector<d::Node> nodes;
  texts.reserve(graph.text_count);roles.reserve(graph.role_count);bindings.reserve(graph.binding_count);options.reserve(graph.option_count);nodes.reserve(graph.node_count);
  for(unsigned i=0;i<graph.text_count;++i){const auto& t=graph.texts[i];require(t.count&&t.count<=65535&&t.first<=pages.size()&&t.count<=pages.size()-t.first,"Invalid preview page range");offsets[i]={0,uint16_t(t.count)};texts.push_back({pages.data()+t.first,offsets[i].data(),uint16_t(t.count),1});}
  for(unsigned i=0;i<graph.role_count;++i){const auto& role=graph.roles[i];roles.push_back({role.id,index(role.name),int16_t(role.portrait)});}
  for(unsigned i=0;i<graph.binding_count;++i){const auto& binding=graph.bindings[i];require(binding.role<graph.role_count&&binding.kind<=1&&(binding.kind||binding.value<=1),"Set an explicit preview value for every condition");bindings.push_back({uint8_t(binding.role),r::Actor3D::static_class_id,binding.kind?nullptr:condition_table[i],binding.kind?action_table[i]:nullptr});}
  for(unsigned i=0;i<graph.option_count;++i){const auto& option=graph.options[i];options.push_back({option.id,index(option.text),index(option.next),index(option.binding)});}
  for(unsigned i=0;i<graph.node_count;++i){const auto& node=graph.nodes[i];require(node.kind<=4&&node.first_option<=options.size()&&node.option_count<=options.size()-node.first_option,"Invalid preview node or option range");d::Node result;result.id=node.id;result.kind=d::Kind(node.kind);result.speaker=node.speaker<0?d::no_role:uint8_t(node.speaker);result.text=index(node.text);result.next=index(node.next);result.otherwise=index(node.otherwise);result.binding=index(node.binding);result.options=node.option_count?options.data()+node.first_option:nullptr;result.option_count=uint16_t(node.option_count);result.reveal_ms=uint16_t(node.reveal_ms);result.auto_ms=node.auto_ms;nodes.push_back(result);}
  d::preview_table={1,roles.data(),uint16_t(roles.size()),bindings.data(),uint16_t(bindings.size()),texts.data(),uint16_t(texts.size()),nodes.data(),uint16_t(nodes.size()),uint16_t(graph.entry)};
  d::preview_style=d::Style{};d::preview_style.timing={uint16_t(input->reveal_ms),input->auto_ms};
  d::session().reset();r::input.reset();r::gameplay_controls().reset();r::localization.language=0;
  std::vector<EpokDialogueTrace> events;World world(*input);callbacks=&graph;trace=&events;
  std::vector<r::ObjectId> actors(input->count),labels_text(input->count),labels_image(input->count);
  r::DialoguePresenter* view=nullptr;
  for(unsigned i=0;i<input->count;++i){const bool active=world.data[i].active;auto* actor=world.spawn(i==input->root?r::DialoguePresenter::static_class_id:r::UIActor::static_class_id,i,active);actors[i]=actor->id();
   if(i==input->root)view=static_cast<r::DialoguePresenter*>(actor);
   if(input->components[i]&1){auto* component=world.level.add_component<r::TextComponent>(*actor,"Text");require(component,"Preview Text allocation failed");labels_text[i]=component->id();}
   if(input->components[i]&2){auto* component=world.level.add_component<r::ImageComponent>(*actor,"Image");require(component,"Preview Image allocation failed");labels_image[i]=component->id();}
   if(input->components[i]&4)require(world.level.add_component<r::FocusableComponent>(*actor,"Focus"),"Preview Focusable allocation failed");
  }
  for(unsigned i=0;i<input->count;++i)if(input->logical_parents[i]>=0){require(unsigned(input->logical_parents[i])<input->count&&world.level.set_logical_parent(actors[i],actors[input->logical_parents[i]]),"Invalid preview logical hierarchy");}
  r::ObjectId* fields[]={&view->panel,&view->body,&view->speaker_name,&view->portrait,&view->choice0,&view->choice1,&view->choice2,&view->choice3,&view->prompt,&view->continuation,&view->choice_panel};
  for(unsigned i=0;i<11;++i)if(input->controls[i]>=0){const auto slot=unsigned(input->controls[i]);require(slot<input->count,"Invalid preview control binding");*fields[i]=(i==0||i==3||i==10)?labels_image[slot]:labels_text[slot];}
  view->default_timing={input->reveal_ms,input->auto_ms};
  auto* source=world.spawn(r::Actor3D::static_class_id,input->count);
  auto* endpoint=world.level.add_component<r::DialogueInteractionComponent>(*source,"Conversation");require(endpoint,"Preview source allocation failed");endpoint->conversation=1;endpoint->presenter=view->id();
  require(view->prepare(endpoint->id(),d::resources[0]),"Dialogue base cannot present: check active controls and required bindings");
  std::array<d::Assignment,8> assignments;for(unsigned i=0;i<roles.size();++i)assignments[i]={roles[i].id,source->id()};
  r::input.sample(0,true,0);r::gameplay_controls().begin_frame();
  d::Start start;start.owner=endpoint->id();start.initiator=source->id();start.presenter=view->id();start.roles=assignments.data();start.role_count=uint16_t(roles.size());start.timing=d::preview_style.timing;start.entered=&entered;
  const auto started=d::session().start(d::preview_table,start);view->begin(endpoint->id());view->on_frame(0);
  for(unsigned i=0;i<input->frame_count;++i){const auto& frame=input->frames[i];r::input.sample(0,true,uint16_t(frame.buttons));r::gameplay_controls().begin_frame();d::session().update_microseconds(frame.microseconds);view->on_frame(frame.microseconds);}
  const auto state=d::session().snapshot();*snapshot={uint32_t(state.phase),uint32_t(state.result),uint32_t(started),state.node,state.revealed,state.glyphs,state.page,state.pages,state.choices,state.choice_page,state.selected,uint32_t(events.size())};
  require(events.size()<=trace_capacity,"Preview trace output is too small");std::copy(events.begin(),events.end(),output_trace);
  EpokDialogueHudSink sink;sink.dimensions.assign(input->dimensions,input->dimensions+input->texture_count*2);sink.load_fonts(input->fonts,input->font_words);
  std::vector<int> first(input->count),next(input->count);std::vector<r::Fixed> measured(size_t(input->count)*2);std::vector<r::hud_core::Rect> rects(input->count);std::vector<r::hud_core::Affine2> transforms(input->count);
  r::hud_core::Compiler compiler(sink,input->width,input->height,{input->budget[0],input->budget[1],input->budget[2],input->budget[3],input->budget[4]});
  compiler.draw(world.data.data(),input->count,first.data(),next.data(),reinterpret_cast<r::Fixed(*)[2]>(measured.data()),rects.data(),transforms.data());
  const auto s=compiler.stats;stats[0]=s.rectangles;stats[1]=s.glyphs;stats[2]=s.texts;stats[3]=s.images;stats[4]=s.dropped;stats[5]=s.rotated;
  require(sink.commands.size()<=capacity,"Preview command output is too small");std::copy(sink.commands.begin(),sink.commands.end(),output);return uint32_t(sink.commands.size());
 }catch(const std::exception& failure){if(error_capacity)std::snprintf(error,error_capacity,"%s",failure.what());return 0;}
}
