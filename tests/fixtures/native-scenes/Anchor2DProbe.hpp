#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t screen_anchor_2d_probe[2]={};}
class EPOK_CLASS(Blueprintable,Id="bdb514d9-bc71-5c05-b199-63270681f240") Anchor2DProbe:public epok::Actor2D {
 bool checked=false;
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("bdb514d9-bc71-5c05-b199-63270681f240");
 uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ScreenAnchorComponent") epok::ObjectId anchor{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Camera2DComponent") epok::ObjectId camera{};
 void on_frame(uint32_t)override {
  if(checked||epok::scene_loading())return;checked=true;unsigned passed=0,total=0;
  auto check=[&](bool good){++total;if(good)++passed;};
  auto& registry=*epok::active_object_registry;
  auto* a=registry.resolve<epok::ScreenAnchorComponent>(anchor);
  auto* c=registry.resolve<epok::Camera2DComponent>(camera);
  auto* root=registry.resolve<epok::SceneComponent2D>(root_id());
  auto* content_root=a?registry.resolve<epok::RectTransformComponent>(a->content):nullptr;
  auto* content=content_root?content_root->get_owner()->data():nullptr;
  check(a&&c&&root&&content);check(a&&a->valid_content());
  if(a&&c&&root&&content) {
   c->make_current();c->position[0]=c->position[1]=0.0;c->zoom=1.0;c->rotating=false;c->rotation=0.0;
   a->pixel_y=0.0;root->set_position(20.0,10.0);a->set_target(id());
   const epok::Fixed x(epok::display_width/2,0),y(epok::display_height/2,0);
   check(a->status()==epok::ScreenAnchorStatus::Following&&content->rect.position[0]==x+20.0&&content->rect.position[1]==y-10.0);
   c->zoom=2.0;a->refresh();check(content->rect.position[0]==x+40.0&&content->rect.position[1]==y-20.0);
   c->rotating=true;c->rotation=90.0;a->refresh();check(content->rect.position[0]==x+20.0&&content->rect.position[1]==y+40.0);
   c->position[0]=20.0;c->position[1]=10.0;a->refresh();check(content->rect.position[0]==x&&content->rect.position[1]==y);
   root->set_position(10000.0,10000.0);check(a->refresh()==epok::ScreenAnchorStatus::Fallback&&content->rect.position[1]==8.0);
   set_active(false);check(a->refresh()==epok::ScreenAnchorStatus::Fallback);set_active(true);
   a->set_target({});check(a->status()==epok::ScreenAnchorStatus::Idle&&content->rect.position[1]==8.0);
   check(content->rect.size[0]==100.0&&content->rect.size[1]==40.0&&content->rect.pivot[0]==0.5);
  }
  screen_anchor_2d_probe[0]=passed;screen_anchor_2d_probe[1]=total;
 }
};
