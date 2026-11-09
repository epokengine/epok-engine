#define EPOK_UI_ANCHOR 1
#define EPOK_WORLD2D 1
#include <cassert>
#include <cstdio>
#include "../../runtime/ui_anchor_service.hpp"
#include "../../runtime/camera_projection.hpp"
using namespace epok;
static ActorData slots[12];static unsigned used=0;static bool camera_available=true;
namespace epok {
void AudioSource::stop(){}
int entity_index(const ActorData* value){for(unsigned i=0;i<used;++i)if(value==slots+i)return int(i);return -1;}
WorldAffineSample gameplay_world_affine(const ActorData* data){WorldAffineSample out;if(data){out.success=true;for(unsigned i=0;i<3;++i)out.position[i]=data->transform.position[i];}return out;}
bool camera_project(const Fixed* point,Fixed* screen){int32_t p[3]={point[0].raw(),point[1].raw(),point[2].raw()},out[2];if(!camera_available||!camera_projection::point(p,4096,4096,display_width,display_height,out))return false;screen[0]=Fixed(out[0],Fixed::RAW);screen[1]=Fixed(out[1],Fixed::RAW);return true;}
template<class T>ClassDescriptor actor(ObjectDomain domain){return {T::static_class_id,Actor::static_class_id,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,12>::acquire,&ObjectPool<T,12>::release};}
template<class T>ClassDescriptor component(uint64_t parent,ObjectDomain domain,uint8_t owners,uint8_t flags=0){return {T::static_class_id,parent,ObjectFamily::Component,domain,owners,flags,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,12>::acquire,&ObjectPool<T,12>::release};}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {UIComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassAbstract},
 component<SceneComponent3D>(ActorComponent::static_class_id,ObjectDomain::World3D,1,ObjectClassRoot),
 component<SceneComponent2D>(ActorComponent::static_class_id,ObjectDomain::World2D,2,ObjectClassRoot),
 component<RectTransformComponent>(UIComponent::static_class_id,ObjectDomain::UI,4,ObjectClassRoot),
 component<CanvasComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<ScreenAnchorComponent>(UIComponent::static_class_id,ObjectDomain::UI,4),
 component<Camera2DComponent>(ActorComponent::static_class_id,ObjectDomain::World2D,2),
 actor<Actor3D>(ObjectDomain::World3D),actor<Actor2D>(ObjectDomain::World2D),actor<UIActor>(ObjectDomain::UI),
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
struct Fixture {
 ObjectRegistryStorage<96> registry;Level level;
 Actor *widget=nullptr,*content=nullptr,*target=nullptr;ScreenAnchorComponent* anchor=nullptr;
 Actor* spawn(uint64_t type){auto id=level.spawn_actor(*find_object_class(type),"Anchor test");assert(id.valid());auto* actor=registry.resolve<Actor>(id);assert(used<12);slots[used]=ActorData{};slots[used].alive=slots[used].active=true;slots[used].parent=-1;actor->bind_data(slots[used++]);return actor;}
 Fixture(){
  used=0;camera_available=true;world2d::service.reset();world2d::install();assert(level.bind(registry));
  widget=spawn(UIActor::static_class_id);content=spawn(UIActor::static_class_id);target=spawn(Actor3D::static_class_id);
  assert(level.add_component<CanvasComponent>(*widget));widget->data()->canvas.enabled=true;
  auto& rect=content->data()->rect;rect.enabled=true;rect.anchor_min[0]=rect.anchor_max[0]=rect.anchor_min[1]=rect.anchor_max[1]=0.0;
  rect.pivot[0]=0.5;rect.pivot[1]=0.0;rect.size[0]=100.0;rect.size[1]=40.0;
  rect.position[0]=17.0;rect.position[1]=23.0;content->data()->parent=entity_index(widget->data());
  target->data()->transform.position[2]=2.0;
  anchor=level.add_component<ScreenAnchorComponent>(*widget);assert(anchor);anchor->content=content->root_id();anchor->set_target(target->id());
 }
 ~Fixture(){level.end_play_all(EndPlayReason::LevelUnloaded);registry.release(level.id());active_object_registry=nullptr;world2d::service.reset();}
 RectTransform& rect(){return content->data()->rect;}
 void inside(){const auto box=ui_anchor::detail::content(*anchor);for(unsigned i=0;i<2;++i){const auto p=rect().position[i]+box.origin[i];assert(p+box.low[i]>=anchor->margin);assert(p+box.high[i]<=Fixed(i?display_height:display_width,0)-anchor->margin);}}
};
int main(){
 {
  Fixture f;assert(f.anchor->valid_content()&&f.anchor->status()==ScreenAnchorStatus::Following);
  assert(f.rect().position[0]==160.0&&f.rect().position[1]==132.0);
  const auto authored=f.rect();
  f.target->data()->transform.position[0]=1.0;f.anchor->on_frame(16);assert(f.rect().position[0]==240.0);
  assert(f.rect().size[0]==authored.size[0]&&f.rect().size[1]==authored.size[1]&&f.rect().pivot[0]==authored.pivot[0]&&f.rect().rotation==authored.rotation);
  f.anchor->pixel_x=Fixed(INT32_MAX,Fixed::RAW);f.anchor->pixel_y=Fixed(INT32_MIN,Fixed::RAW);f.anchor->refresh();f.inside();
  f.anchor->pixel_x=Fixed(INT32_MIN,Fixed::RAW);f.anchor->pixel_y=Fixed(INT32_MAX,Fixed::RAW);f.anchor->refresh();f.inside();
  f.anchor->pixel_x=f.anchor->pixel_y=0.0;f.rect().rotation=45.0;f.anchor->refresh();f.inside();
  // Authored nonzero anchors and a rotated pivot remain ordinary HUD geometry.
  f.rect().anchor_min[0]=f.rect().anchor_max[0]=0.5;f.rect().anchor_min[1]=f.rect().anchor_max[1]=1.0;
  f.anchor->refresh();f.inside();
  f.target->data()->transform.position[2]=-1.0;assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);f.inside();
  f.target->data()->transform.position[2]=2.0;camera_available=false;assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);
  camera_available=true;f.target->set_active(false);assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);f.target->set_active(true);
  f.target->data()->transform.position[0]=Fixed(INT32_MAX,Fixed::RAW);f.anchor->offset_x=1.0;assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);
  const auto x=f.rect().position[0],y=f.rect().position[1];f.anchor->set_target({});f.anchor->on_frame(16);assert(f.anchor->status()==ScreenAnchorStatus::Idle&&f.rect().position[0]==x&&f.rect().position[1]==y);
  f.anchor->enabled=false;f.anchor->set_target(f.target->id());assert(f.anchor->status()==ScreenAnchorStatus::Idle&&f.rect().position[0]==x);
 }
 {
  Fixture f;auto* parent=f.spawn(UIActor::static_class_id);f.widget->data()->parent=entity_index(parent->data());
  assert(f.anchor->refresh()==ScreenAnchorStatus::InvalidContent);
  CanvasComponent* canvas=nullptr;for(size_t i=0;i<f.widget->component_count();++i)if(auto* value=f.registry.resolve<CanvasComponent>(f.widget->component_id(i)))canvas=value;
  assert(canvas&&!canvas->screen_overlay());canvas->set_screen_overlay(true);assert(canvas->screen_overlay());
  assert(f.anchor->refresh()==ScreenAnchorStatus::Following&&f.rect().position[0]==160.0&&f.rect().position[1]==132.0);
 }
 {
  Fixture f;f.content->data()->parent=-1;assert(!f.anchor->valid_content()&&f.anchor->refresh()==ScreenAnchorStatus::InvalidContent);
  f.content->data()->parent=entity_index(f.widget->data());f.rect().size[0]=400.0;assert(f.anchor->refresh()==ScreenAnchorStatus::ContentTooLarge);
  f.rect().size[0]=100.0;f.anchor->margin=Fixed(INT32_MAX,Fixed::RAW);assert(f.anchor->refresh()==ScreenAnchorStatus::ContentTooLarge);
  f.anchor->margin=8.0;f.rect().pivot[0]=Fixed(INT32_MAX,Fixed::RAW);assert(f.anchor->refresh()==ScreenAnchorStatus::InvalidContent);f.rect().pivot[0]=0.5;
  f.anchor->margin=8.0;f.widget->data()->layout_container.enabled=true;f.widget->data()->layout_container.kind=LayoutKind::Horizontal;assert(f.anchor->refresh()==ScreenAnchorStatus::InvalidContent);
 }
 {
  Fixture f;auto* target=f.spawn(Actor2D::static_class_id);auto* root=f.registry.resolve<SceneComponent2D>(target->root_id());assert(root);
  f.anchor->set_target(target->id());assert(f.anchor->status()==ScreenAnchorStatus::Fallback);
  auto* camera_owner=f.spawn(Actor2D::static_class_id);auto* camera=f.level.add_component<Camera2DComponent>(*camera_owner);assert(camera);camera->position[0]=camera->position[1]=0.0;camera->zoom=1.0;
  f.anchor->pixel_y=0.0;root->set_position(20.0,10.0);assert(f.anchor->refresh()==ScreenAnchorStatus::Following);
  assert(f.rect().position[0]==180.0&&f.rect().position[1]==110.0);
  camera->zoom=2.0;f.anchor->refresh();assert(f.rect().position[0]==200.0&&f.rect().position[1]==100.0);
  camera->rotating=true;camera->rotation=90.0;f.anchor->refresh();assert(f.rect().position[0]==180.0&&f.rect().position[1]==160.0);
  camera->position[0]=20.0;camera->position[1]=10.0;f.anchor->refresh();assert(f.rect().position[0]==160.0&&f.rect().position[1]==120.0);
  root->set_position(10000.0,10000.0);assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);f.inside();
  target->destroy();assert(f.anchor->refresh()==ScreenAnchorStatus::Fallback);
 }
 std::puts("UI anchors: 3D and 2D cameras, authored containers, rotated bounds, fallback and lifecycle passed");
}
