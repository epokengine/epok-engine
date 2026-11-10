#pragma once
#include "epok.hpp"
extern "C" { inline uint32_t screen_anchor_probe[2]={}; }
class EPOK_CLASS(Blueprintable,Id="3c892c5a-4eb4-50c2-9466-8d6257f2ae74") AnchorProbe:public epok::Actor3D {
 bool checked=false;
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ScreenAnchorComponent") epok::ObjectId anchor{};
 void on_frame(uint32_t)override{
  if(checked)return;checked=true;unsigned passed=0,total=0;auto check=[&](bool good){++total;if(good)++passed;};
  auto* a=epok::active_object_registry->resolve<epok::ScreenAnchorComponent>(anchor);
  check(a&&a->valid_content());
  auto* rect=a?epok::active_object_registry->resolve<epok::RectTransformComponent>(a->content):nullptr;
  auto* content=rect?rect->get_owner()->data():nullptr;check(content!=nullptr);
  if(a&&content){
   check(a->status()==epok::ScreenAnchorStatus::Following&&content->rect.position[0]==epok::Fixed(epok::display_width*5/8,0)&&content->rect.position[1]==epok::Fixed(epok::display_height/2+12,0));
   data()->transform.position[0]=-0.5;a->refresh();check(content->rect.position[0]==epok::Fixed(epok::display_width*3/8,0));
   data()->transform.position[2]=-1.0;check(a->refresh()==epok::ScreenAnchorStatus::Fallback&&content->rect.position[1]==8.0);
   a->set_target({});check(a->status()==epok::ScreenAnchorStatus::Idle&&content->rect.position[1]==8.0);
   a->enabled=false;a->set_target(id());check(a->status()==epok::ScreenAnchorStatus::Idle&&content->rect.position[1]==8.0);
   check(content->rect.size[0]==100.0&&content->rect.size[1]==40.0&&content->rect.pivot[0]==0.5);
  }
  screen_anchor_probe[0]=passed;screen_anchor_probe[1]=total;
  data()->transform.rotation[0]=epok::Fixed(int(passed),0);data()->transform.rotation[1]=epok::Fixed(int(total),0);
 }
};
