#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t camera_projection_probe[2]={};}
// The same public screen-query scenarios as the Native Play adapter test.
class EPOK_CLASS(Blueprintable,Id="82367884-c392-48b6-9662-3a69252e33d8") ProjectionProbe:public epok::Actor3D {
 bool checked=false;
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("82367884-c392-48b6-9662-3a69252e33d8");
 uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 void on_frame(uint32_t)override{
  if(checked)return;checked=true;unsigned passed=0,total=0;
  auto check=[&](bool good){++total;if(good)++passed;};
  auto project=[](epok::Fixed x,epok::Fixed y,epok::Fixed z){return epok::SceneLibrary::project({x,y,z});};
  auto point=project(0.0,0.0,1.0);check(point.success&&point.x.raw()==epok::display_width/2*4096&&point.y.raw()==epok::display_height/2*4096);
  check(project(0.0,0.0,0.25).success);check(!project(0.0,0.0,epok::Fixed(1023,epok::Fixed::RAW)).success);
  check(!project(0.0,0.0,-1.0).success);check(project(0.0,0.0,127.0).success);check(!project(0.0,0.0,128.0).success);
  point=project(-1.0,0.0,1.0);check(point.success&&point.x.raw()==0);check(!project(1.0,0.0,1.0).success);
  point=project(0.0,0.75,1.0);check(point.success&&point.y.raw()==0);check(!project(0.0,-0.75,1.0).success);
  auto* camera=epok::active_camera().get();check(camera&&camera->owner);
  if(camera&&camera->owner){
   camera->camera_settings.mirror_x=true;point=project(0.5,0.0,1.0);check(point.success&&point.x.raw()==epok::display_width/4*4096);camera->camera_settings.mirror_x=false;
   camera->camera_settings.field_of_view=25.0;check(!project(0.5,0.0,1.0).success);
   camera->camera_settings.field_of_view=120.0;check(project(0.5,0.0,1.0).success);
   camera->camera_settings.field_of_view=90.0;camera->owner->set_active(false);check(!project(0.0,0.0,1.0).success);camera->owner->set_active(true);check(project(0.0,0.0,1.0).success);
  }
  camera_projection_probe[0]=passed;camera_projection_probe[1]=total;
 }
};
