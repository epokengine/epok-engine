#pragma once
#include "WidgetClasses.hpp"
extern "C" {inline uint32_t widget_focus_probe[5]={};}
class EPOK_CLASS(Blueprintable,Id="91c0b342-0754-4ca0-a9ef-9c45d62a5b39") WidgetFocusProbe:public epok::Actor3D {
 unsigned stage=0;bool valid=true;epok::ObjectId widgets[3],seconds[3];epok::DataHandle targets[3];
public:
 void on_frame(uint32_t) override {
  auto* registry=epok::active_object_registry;auto* level=registry->resolve<epok::Level>(level_id());const uint64_t classes[]={CLASSES};
  if(stage==0){for(auto id:classes)if(const auto* type=epok::find_object_class(id))level->spawn_actor(*type,"Dynamic widget");stage=1;return;}
  if(stage==1){
   unsigned count=0,mask=0;
   for(size_t i=0;i<level->actor_count();++i){auto* actor=registry->resolve<epok::Actor>(level->actor_at(i));if(!actor)continue;
    for(unsigned n=0;n<3;++n)if(actor->class_id()==classes[n]){
     ++count;mask|=1u<<n;widgets[n]=actor->id();auto* text=registry->resolve<epok::TextComponent>(static_cast<CppWidgetBase*>(actor)->label);auto* first=text?text->get_owner():nullptr;
     targets[n]=first?epok::handle(first->data()):epok::DataHandle{};seconds[n]={};
     valid&=first&&actor->data()->canvas.focused==int(targets[n].index);
     if(first)for(size_t j=0;j<level->actor_count();++j){auto* other=registry->resolve<epok::Actor>(level->actor_at(j));if(other&&epok::entity_index(other->data())==first->data()->focusable.neighbors[1])seconds[n]=other->id();}
     valid&=seconds[n].valid();
    }
   }
   valid&=count==9&&mask==7;stage=2;return;
  }
  if(stage==2){for(auto target:targets){auto* data=target.get();valid&=data&&data->owner;if(data&&data->owner)valid&=level->destroy_actor(data->owner->id());}stage=3;return;}
  if(stage==3){
   for(auto target:targets)valid&=!target.get();
   if(const auto* type=epok::find_object_class(epok::UIActor::static_class_id))for(unsigned n=0;n<3;++n)level->spawn_actor(*type,"Replacement control");else valid=false;
   stage=4;return;
  }
  unsigned mask=0,reused=0;
  for(unsigned n=0;n<3;++n){auto* widget=registry->resolve<epok::Actor>(widgets[n]);auto* second=registry->resolve<epok::Actor>(seconds[n]);
   valid&=widget&&second&&!targets[n].get();
   valid&=widget&&widget->data()->canvas.focused==-1&&second&&second->data()->focusable.neighbors[0]==-1;
   for(size_t i=0;i<level->actor_count();++i){auto* actor=registry->resolve<epok::Actor>(level->actor_at(i));if(!actor||actor->class_id()!=epok::UIActor::static_class_id)continue;
    auto* data=actor->data();if(epok::entity_index(data)==int(targets[n].index)){data->focusable.enabled=true;valid&=data->generation!=targets[n].generation;mask|=1u<<n;++reused;}
   }
  }
  data()->transform.rotation[0]=epok::Fixed(int(mask),0);data()->transform.rotation[1]=epok::Fixed(int(reused),0);data()->transform.rotation[2]=epok::Fixed(valid?1:0,0);
  widget_focus_probe[0]=mask;widget_focus_probe[1]=reused;widget_focus_probe[2]=valid;widget_focus_probe[3]=epok::hud_stats.glyphs;widget_focus_probe[4]=epok::hud_stats.dropped;
 }
};
