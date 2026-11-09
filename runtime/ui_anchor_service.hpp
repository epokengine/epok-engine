#pragma once
#include "ui_anchor.hpp"
#include "hud_core.hpp"
#ifdef EPOK_WORLD2D
#include "world2d_service.hpp"
#endif

namespace epok::ui_anchor {
namespace detail {
inline bool live(Actor* actor){
    if(!actor||!active_object_registry||actor->id().get()!=actor)return false;
    auto* level=active_object_registry->resolve<Level>(actor->level_id());
    return level&&level->state()!=ObjectState::EndingPlay&&level->state()!=ObjectState::Destroyed&&level->actor_active(*actor);
}
struct Content {ActorData* data=nullptr;Fixed low[2]={},high[2]={},origin[2]={};};
inline bool add(Fixed a,Fixed b,Fixed& out){
    const int64_t value=int64_t(a.raw())+b.raw();
    if(value<INT32_MIN||value>INT32_MAX)return false;
    out=Fixed(int32_t(value),Fixed::RAW);return true;
}
inline Content content(const ScreenAnchorComponent& anchor){
    Content out;
    auto* owner=const_cast<ScreenAnchorComponent&>(anchor).get_owner();
    auto* component=active_object_registry?active_object_registry->resolve<RectTransformComponent>(anchor.content):nullptr;
    auto* child=component?component->get_owner():nullptr;
    if(!live(owner)||!live(child)||owner->level_id()!=child->level_id())return out;
    auto* root=owner->data();auto* data=child->data();
    if(!root||!root->canvas.enabled||(root->parent>=0&&!root->canvas.screen_overlay)||!data||!data->rect.enabled||data->canvas.enabled||data->parent!=entity_index(root)
        ||(root->layout_container.enabled&&root->layout_container.kind!=LayoutKind::None))return out;
    const auto& r=data->rect;
    // Keep the same authored RectTransform bounds before doing fixed arithmetic.
    for(unsigned i=0;i<2;++i)if(r.anchor_min[i]<0.0||r.anchor_max[i]>1.0||r.anchor_min[i]>r.anchor_max[i]
        ||r.pivot[i]<0.0||r.pivot[i]>1.0||r.size[i]<-1024.0||r.size[i]>1024.0)return out;
    if(r.rotation<-3600.0||r.rotation>3600.0)return out;
    auto geometry=r;geometry.position[0]=geometry.position[1]=0.0;
    const auto box=hud_core::resolve({0.0,0.0,Fixed(display_width,0),Fixed(display_height,0)},geometry);
    if(box.w<=0.0||box.h<=0.0||box.w>Fixed(display_width,0)||box.h>Fixed(display_height,0)){out.data=data;return out;}
    const Fixed left=-box.w*r.pivot[0],bottom=-box.h*r.pivot[1];
    const auto rotation=hud_core::rotation_about(0.0,0.0,r.rotation);
    for(unsigned i=0;i<4;++i){
        Fixed x,y;hud_core::apply(rotation,left+(i&1?box.w:Fixed(0.0)),bottom+(i&2?box.h:Fixed(0.0)),x,y);
        if(!i){out.low[0]=out.high[0]=x;out.low[1]=out.high[1]=y;}
        else {if(x<out.low[0])out.low[0]=x;if(x>out.high[0])out.high[0]=x;if(y<out.low[1])out.low[1]=y;if(y>out.high[1])out.high[1]=y;}
    }
    out.origin[0]=Fixed(display_width,0)*(r.anchor_min[0]+(r.anchor_max[0]-r.anchor_min[0])*r.pivot[0]);
    out.origin[1]=Fixed(display_height,0)*(r.anchor_min[1]+(r.anchor_max[1]-r.anchor_min[1])*r.pivot[1]);
    out.data=data;return out;
}
inline bool fits(const Content& box,Fixed margin){
    if(margin<0.0)margin=0.0;
    return box.data&&box.high[0]>box.low[0]&&box.high[1]>box.low[1]
        &&int64_t(box.high[0].raw())-box.low[0].raw()+int64_t(margin.raw())*2<=int64_t(display_width)*4096
        &&int64_t(box.high[1].raw())-box.low[1].raw()+int64_t(margin.raw())*2<=int64_t(display_height)*4096;
}
inline bool project(const ScreenAnchorComponent& anchor,Fixed* screen){
    auto* actor=active_object_registry?active_object_registry->resolve<Actor>(anchor.target):nullptr;
    auto* owner=const_cast<ScreenAnchorComponent&>(anchor).get_owner();
    if(!live(actor)||!owner||actor->level_id()!=owner->level_id())return false;
    if(actor->is_a(Actor3D::static_class_id)){
        const auto world=gameplay_world_affine(actor->data());if(!world.success)return false;
        Fixed point[3];if(!add(world.position[0],anchor.offset_x,point[0])||!add(world.position[1],anchor.offset_y,point[1])||!add(world.position[2],anchor.offset_z,point[2]))return false;
        return camera_project(point,screen);
    }
#ifdef EPOK_WORLD2D
    if(actor->is_a(Actor2D::static_class_id)){
        auto* root=active_object_registry->resolve<SceneComponent2D>(actor->root_id());Affine2D world;Camera2D camera;
        if(!root||!world_matrix_2d(*root,[](ObjectId id){return world2d::resolve_scene2d(id);},world)||!world2d::view_camera(camera))return false;
        Fixed point[2];if(!add(world.t[0],anchor.offset_x,point[0])||!add(world.t[1],anchor.offset_y,point[1]))return false;
        world_to_screen(camera,point,screen);
        return screen[0]>=0.0&&screen[1]>=0.0&&screen[0]<Fixed(display_width,0)&&screen[1]<Fixed(display_height,0);
    }
#endif
    return false;
}
}
bool valid_content(const ScreenAnchorComponent& anchor){return detail::fits(detail::content(anchor),anchor.margin);}
bool valid_binding(ObjectId reference,const Actor& owner){
    auto* component=active_object_registry?active_object_registry->resolve<ScreenAnchorComponent>(reference):nullptr;
    return component&&component->get_owner()==&owner&&(!component->enabled||component->valid_content());
}
void set_target(ObjectId reference,Actor& owner,ObjectId target){
    auto* component=active_object_registry?active_object_registry->resolve<ScreenAnchorComponent>(reference):nullptr;
    if(component&&component->get_owner()==&owner)component->set_target(target);
}
ScreenAnchorStatus update(ScreenAnchorComponent& anchor){
    const auto box=detail::content(anchor);
    if(!box.data)return ScreenAnchorStatus::InvalidContent;
    if(!detail::fits(box,anchor.margin))return ScreenAnchorStatus::ContentTooLarge;
    const Fixed margin=anchor.margin<0.0?Fixed(0.0):anchor.margin;
    Fixed screen[2];const bool projected=detail::project(anchor,screen);
    int64_t point[2]={int64_t(display_width)*2048,int64_t(margin.raw())-box.low[1].raw()};
    if(projected){point[0]=int64_t(screen[0].raw())+anchor.pixel_x.raw();point[1]=int64_t(display_height)*4096-screen[1].raw()+anchor.pixel_y.raw();}
    for(unsigned i=0;i<2;++i){
        const int64_t limit=int64_t(i?display_height:display_width)*4096;
        if(point[i]+box.low[i].raw()<margin.raw())point[i]=int64_t(margin.raw())-box.low[i].raw();
        if(point[i]+box.high[i].raw()>limit-margin.raw())point[i]=limit-margin.raw()-box.high[i].raw();
        box.data->rect.position[i]=Fixed(int32_t(point[i]-box.origin[i].raw()),Fixed::RAW);
    }
    return projected?ScreenAnchorStatus::Following:ScreenAnchorStatus::Fallback;
}
}
