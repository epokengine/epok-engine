// Included inside the EXISTING DungeonEconomyPolicyProbe. Fixture input and
// assertions only: every menu/policy choice runs through selected virtual APIs.
epok::ObjectId ui_previous_view_{};
DungeonEconomyUIOutput ui_output_{};
uint32_t ui_fail_=0,ui_stage_=0;
bool ui_controls_=true;
DungeonHUDSnapshot ui_snapshot_{61,100,70,280,0,0,false,true,true,false};
DungeonEconomyPresenterBase* ui_hud_value(){return epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyPresenterBase>(ui_hud):nullptr;}
epok::ActorData* ui_data(epok::ObjectId value){if(auto* data=epok::gameplay_actor_data(value))return data;auto* component=epok::active_object_registry?epok::active_object_registry->resolve<epok::ActorComponent>(value):nullptr;auto* owner=component?component->get_owner():nullptr;return owner?owner->data():nullptr;}
epok::Level* ui_level(){return epok::active_object_registry?epok::active_object_registry->resolve<epok::Level>(level_id()):nullptr;}
void ui_check(bool value,uint32_t flag){if(!value)ui_fail_|=flag;}
void ui_move(epok::Fixed x,epok::Fixed y=0.0,epok::Fixed z=0.0){if(auto* d=data()){d->transform.position[0]=x;d->transform.position[1]=y;d->transform.position[2]=z;}}
uint32_t ui_text_hash(epok::ObjectId actor){auto* d=ui_data(actor);uint32_t h=2166136261u;if(d)for(uint32_t n=0;n<sizeof(d->text.value)&&d->text.value[n];++n)h=(h^uint8_t(d->text.value[n]))*16777619u;return h;}
bool ui_contains(epok::ObjectId actor,const char* needle){auto* d=ui_data(actor);if(!d||!needle)return false;if(!*needle)return true;for(uint32_t at=0;at<sizeof(d->text.value)&&d->text.value[at];++at){uint32_t n=0;while(needle[n]&&at+n<sizeof(d->text.value)&&d->text.value[at+n]==needle[n])++n;if(!needle[n])return true;}return false;}
DungeonEconomyWidgetBase* ui_view(){auto* h=ui_hud_value();return h&&epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyWidgetBase>(h->view):nullptr;}
void ui_frame(uint16_t held=0){
 using namespace epok;input.sample(0,true,held);gameplay_controls().begin_frame();
 auto* h=ui_hud_value();auto* i=active_object_registry?active_object_registry->resolve<InteractorComponent>(ui_interactor):nullptr;
 if(i)i->frame_update(16667);
 if(h&&is_active(h))ui_output_=h->frame({ui_controls_,true,0,Fixed(137,Fixed::RAW)},16667);
 else ui_output_={};
}
uint16_t ui_button(epok::Button value){return uint16_t(1u<<unsigned(value));}
void ui_run(uint32_t stage,DungeonEconomyPolicyBase& p){
 using namespace epok;using namespace economy_probe;ui_stage_=stage;
 auto* h=ui_hud_value();auto* level=ui_level();
 auto* a=active_object_registry?active_object_registry->resolve<DungeonEconomyStationBase>(ui_pocket):nullptr;
 auto* b=active_object_registry?active_object_registry->resolve<DungeonEconomyStationBase>(ui_shop):nullptr;
 auto* i=active_object_registry?active_object_registry->resolve<InteractorComponent>(ui_interactor):nullptr;
 if(!h||!level||!a||!b||!i){ui_fail_|=1;return;}
 switch(stage){
 case 0:{
  card.drain();card.failure=0;p.assign_saved(seed(0));p.assign_pending(p.saved);p.assign_tx({});p.assign_loaded(true);p.pocket={};
  p.tuning={1000000,9,9,3,10,30,80,120,35,100,8};
  static const PocketApp no_app{};p.wire_assign_image(no_app);
  h->policy=p.id();h->shutdown();a->hud=b->hud=h->id();a->menu=1;b->menu=2;i->range_3d=3.61;
  if(auto* d=a->get_owner()->data()){d->transform.position[0]=0.0;d->transform.position[1]=d->transform.position[2]=0.0;}
  if(auto* d=b->get_owner()->data()){d->transform.position[0]=8.0;d->transform.position[1]=d->transform.position[2]=0.0;}
  ui_move(50.0);ui_controls_=true;h->render_hud(ui_snapshot_);ui_frame();break;
 }
 case 1:h->render_hud(ui_snapshot_);ui_check(ui_data(h->energy_bar)&&ui_data(h->energy_bar)->image.region[2]==49,16);ui_check(ui_contains(h->energy_label,"61"),512);ui_frame();break;
 case 2:ui_frame(ui_button(Button::Triangle));break;
 case 3:ui_frame();ui_check(ui_view()&&ui_view()->menu_kind()==3&&h->screen.menu==3,2);ui_check(GameplayInputLibrary::valid(h->capture_token())&&GameplayInputLibrary::captured(GameplayAction::Attack,0),4);break;
 case 4:ui_frame(ui_button(Button::Up));ui_check(h->screen.selected==3,2);break;
 case 5:ui_frame();break;
 case 6:ui_frame(ui_button(Button::Down));ui_check(h->screen.selected==0,2);break;
 case 7:ui_frame();break;
 case 8:ui_frame(ui_button(Button::Cross));ui_check(p.tx.phase==2&&p.saved.medkits==2&&!ui_output_.heal,32);break;
 case 9:ui_frame(ui_button(Button::Circle));ui_check(h->screen.menu==3&&h->screen.busy,32);break;
 case 10:card.failure=1;card.drain();ui_frame();card.failure=0;ui_check(p.saved.medkits==2&&!h->screen.busy,32);break;
 case 11:ui_frame(ui_button(Button::Cross));ui_check(p.tx.phase==2,32);break;
 case 12:ui_previous_view_=h->view;level->set_active(h->id(),false);card.drain();ui_frame();ui_check(!GameplayInputLibrary::valid(h->capture_token()),256);break;
 case 13:ui_check(!active_object_registry->get(ui_previous_view_),256);level->set_active(h->id(),true);ui_frame();ui_check(p.saved.medkits==1&&ui_output_.heal==35,1024);break;
 case 14:ui_frame();ui_check(!ui_output_.heal&&!h->view.valid(),1024);break;
 case 15:ui_frame(ui_button(Button::Triangle));break;
 case 16:ui_frame();ui_check(ui_view()&&ui_view()->menu_kind()==3,2);break;
 case 17:ui_frame(ui_button(Button::Triangle));ui_check(!h->view.valid()&&!GameplayInputLibrary::valid(h->capture_token()),4);break;
 case 18:ui_frame();break;
 case 19:ui_move(8.0);ui_frame();ui_check(i->focused_target()==b->id(),64);break;
 case 20:ui_frame(ui_button(Button::Cross));break;
 case 21:ui_frame();ui_check(ui_view()&&ui_view()->menu_kind()==2,2);break;
 case 22:p.tuning.medkit_price=17;p.tuning.medkit_heal=44;ui_frame();if(auto* w=ui_view())ui_check(ui_contains(w->body,"+44")&&ui_contains(w->body,"17 C"),512);else ui_fail_|=512;break;
 case 23:ui_frame(ui_button(Button::Down));ui_check(h->screen.selected==1,2);break;
 case 24:ui_frame();break;
 case 25:ui_frame(ui_button(Button::Cross));ui_check(p.tx.phase==2&&p.saved.credits==900,32);break;
 case 26:card.drain();ui_frame();ui_check(p.saved.credits==883&&p.saved.medkits==2,8);break;
 case 27:ui_frame(ui_button(Button::Circle));break;
 case 28:ui_frame();break;
 case 29:ui_move(0.0);ui_frame();ui_check(i->focused_target()==a->id(),64);break;
 case 30:ui_frame(ui_button(Button::Cross));break;
 case 31:ui_frame();ui_check(ui_view()&&ui_view()->menu_kind()==1,2);break;
 case 32:ui_frame(ui_button(Button::Right));ui_check(h->screen.port==1,2);break;
 case 33:ui_frame();if(auto* w=ui_view())ui_check(ui_contains(w->summary,"SLOT 2"),512);break;
 case 34:ui_frame(ui_button(Button::Cross));ui_check(h->screen.notice==31&&!h->screen.busy,32);break;
 case 35:ui_frame(ui_button(Button::Circle));break;
 case 36:ui_frame();break;
 case 37:ui_move(2.5);ui_frame();ui_check(!i->focused_target().valid(),64);break;
 case 38:ui_move(0.0,2.6);ui_frame();ui_check(!i->focused_target().valid(),64);break;
 case 39:ui_move(2.4,2.5);ui_frame();ui_check(i->focused_target()==a->id(),64);break;
 case 40:ui_move(50.0);ui_frame(ui_button(Button::Triangle));break;
 case 41:ui_frame();ui_previous_view_=h->view;ui_check(ui_view()&&level->actor_count()==64,128);break;
 case 42:level->destroy_actor(ui_previous_view_);ui_frame();break;
 case 43:ui_frame();ui_check(!active_object_registry->get(ui_previous_view_),256);break;
 case 44:ui_check(ui_view()&&h->view!=ui_previous_view_,256);ui_frame(ui_button(Button::Circle));ui_check(!h->view.valid(),256);break;
 case 45:ui_frame();ui_check(level->actor_count()==58,128);break;
 case 46:level->set_active(b->owner_id(),false);ui_move(8.0);ui_frame();ui_check(!i->focused_target().valid(),64);break;
 case 47:level->set_active(b->owner_id(),true);ui_frame();ui_check(i->focused_target()==b->id(),64);break;
 case 48:ui_move(50.0);ui_frame(ui_button(Button::Triangle));break;
 case 49:ui_frame();ui_check(ui_view()!=nullptr,2);break;
 case 50:ui_controls_=false;ui_snapshot_.state=2;ui_snapshot_.aiming=false;h->render_hud(ui_snapshot_);ui_frame();break;
 case 51:ui_frame();ui_check(!h->view.valid()&&!GameplayInputLibrary::valid(h->capture_token()),4);ui_check(ui_data(h->game_message)&&ui_data(h->game_message)->text.enabled&&ui_contains(h->game_message,"PLAY AGAIN"),512);break;
 case 52:ui_snapshot_.state=3;ui_snapshot_.alive=false;h->render_hud(ui_snapshot_);ui_frame();ui_check(ui_contains(h->game_message,"OUT OF ENERGY")&&ui_contains(h->objective,"FAILED"),512);break;
 case 53:ui_snapshot_={25,100,180,720,0,0,true,false,true,false};h->style.critical_green=71;h->render_hud(ui_snapshot_);ui_frame();ui_check(ui_data(h->energy_bar)&&ui_data(h->energy_bar)->image.color[1]==71,16);ui_check(ui_data(h->boss_bar)&&ui_data(h->boss_bar)->progress.value==Fixed(.25),16);break;
 case 54:h->damage_hud({true,9,210,20,30});ui_frame();ui_check(h->data()&&h->data()->image.enabled&&h->data()->image.borders[0]==9,16);break;
 case 55:h->damage_hud({false,0,0,0,0});ui_frame();ui_check(level->actor_count()==58&&!ui_output_.heal&&!h->view.valid(),128);break;
 default:ui_fail_|=1;break;
 }
}
void ui_row(uint32_t* row){
 using namespace epok;auto* h=ui_hud_value();auto* w=ui_view();auto* level=ui_level();auto* bar=h?ui_data(h->energy_bar):nullptr;auto* boss=h?ui_data(h->boss_bar):nullptr;
 const uint32_t values[]={ui_stage_,h?h->screen.menu:0,h?h->screen.selected:0,h?h->screen.notice:0,ui_output_.blocked,ui_output_.heal,h&&GameplayInputLibrary::valid(h->capture_token()),w?w->menu_kind():0,bar?uint32_t(bar->image.region[2]):0,boss?uint32_t(boss->progress.value.raw()):0,h&&ui_data(h->game_message)&&ui_data(h->game_message)->text.enabled,
  w?ui_text_hash(w->body):h?ui_text_hash(h->game_message):0,level?uint32_t(level->actor_count()):0,ui_fail_};
 for(uint32_t k=0;k<14;++k)row[72+k]=values[k];
}
