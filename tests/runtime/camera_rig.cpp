#define EPOK_GAMEPLAY_CONTROL 1
#include <cassert>
#include <cstdio>
#include "../../runtime/camera_rig.hpp"
using namespace epok;
static CollisionWorld<Fixed,16,4> world;
static ActorData slots[10];static unsigned used=0,queries=0;static ActorData* current=nullptr;
class CallbackCamera:public CameraRig3D {
public:
 static constexpr uint64_t static_class_id=0x11223344;
 uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 bool destroy_calculated=false,destroy_updated=false,custom_fixed=false;CameraRigStatus nested=CameraRigStatus::NoStep;
 CameraRigSample calculate_view(Fixed dt)override{auto view=CameraRig3D::calculate_view(dt);if(custom_fixed){view.position.x+=1.0;view.roll=12.0;}if(destroy_calculated)destroy();return view;}
 void view_updated(CameraRigSample)override{nested=refresh(Fixed(1.0/60.0)).status;if(destroy_updated)destroy();}
};
template<class T> static NativeComponentDefault camera_defaults(Object& object,size_t index){
 auto& camera=static_cast<T&>(object);
 if(index==0)return {&camera.root,"Root",true,-1,SceneComponent3D::static_class_id};
 if(index==1)return {&camera.camera,"Camera",false,-1,Camera3DComponent::static_class_id};
 if constexpr(std::is_same_v<T,ThirdPersonCamera>)if(index==2)return {&camera.boom,"Spring Arm",false,-1,SpringArmComponent::static_class_id};
 return {};
}
template<class T,class Base> static ClassDescriptor descriptor(ObjectFamily family,uint8_t flags=0,size_t count=0,NativeComponentDefault(*defaults)(Object&,size_t)=nullptr){
 auto value=ClassDescriptor{T::static_class_id,Base::static_class_id,family,ObjectDomain::World3D,1,flags,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,8>::acquire,&ObjectPool<T,8>::release,count,defaults};
 value.tick_group=TickGroupOf<T>::value;
 if constexpr(std::is_base_of_v<ActorComponent,T>){value.component_tick=ComponentCallbacks<T>::tick;value.component_frame=ComponentCallbacks<T>::frame;}
 else{value.component_tick=ActorCallbacks<T>::tick;value.component_frame=ActorCallbacks<T>::frame;}
 return value;
}
namespace epok {
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
 descriptor<Actor3D,Actor>(ObjectFamily::Actor,6),
 descriptor<SceneComponent3D,ActorComponent>(ObjectFamily::Component,ObjectClassRoot),
 descriptor<Camera3DComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<SpringArmComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<SpatialQueryContextComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<CameraOrbitInputComponent,ActorComponent>(ObjectFamily::Component),
 descriptor<CameraRig3D,Actor3D>(ObjectFamily::Actor,6,2,&camera_defaults<CameraRig3D>),
 descriptor<FirstPersonCamera,CameraRig3D>(ObjectFamily::Actor,6,2,&camera_defaults<FirstPersonCamera>),
 descriptor<ThirdPersonCamera,CameraRig3D>(ObjectFamily::Actor,6,3,&camera_defaults<ThirdPersonCamera>),
 descriptor<TopDownCamera,CameraRig3D>(ObjectFamily::Actor,6,2,&camera_defaults<TopDownCamera>),
 descriptor<IsometricCamera,CameraRig3D>(ObjectFamily::Actor,6,2,&camera_defaults<IsometricCamera>),
 descriptor<CallbackCamera,CameraRig3D>(ObjectFamily::Actor,6,2,&camera_defaults<CallbackCamera>),
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
int entity_index(const ActorData* data){for(unsigned i=0;i<used;++i)if(data==slots+i)return int(i);return -1;}
ActorData* DataHandle::get()const{return index<used&&slots[index].alive&&slots[index].generation==generation?slots+index:nullptr;}
DataHandle handle(const ActorData* data){const auto index=entity_index(data);return index>=0&&data->alive?DataHandle{uint16_t(index),data->generation}:DataHandle{};}
bool set_active_camera(ActorData* data){if(!data||!data->owner||!is_active(data->owner)||!data->camera_settings.enabled)return false;current=data;return true;}
DataHandle active_camera(){return handle(current);}
WorldAffineSample gameplay_world_affine(const ActorData* data){WorldAffineSample result;if(data&&data->alive){result.success=true;for(unsigned i=0;i<3;++i)result.position[i]=data->transform.position[i];}return result;}
SpatialHit raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers){++queries;return world.raycast(origin,delta,mask,entity_index(ignore),triggers);}
SpatialHit SpatialQueryBatch::raycast(const Fixed* origin,const Fixed* delta,uint32_t mask,const ActorData* ignore,bool triggers)const{return epok::raycast(origin,delta,mask,ignore,triggers);}
void SpatialQueryBatch::sync(ActorData&){ready=true;}
MoveResult SpatialQueryBatch::move_and_slide(ActorData& actor,const Fixed* delta,uint32_t mask){
 Aabb bounds;for(unsigned i=0;i<3;++i){const auto center=actor.transform.position[i]+actor.collider.center[i];bounds.min[i]=center-actor.collider.half_extents[i];bounds.max[i]=center+actor.collider.half_extents[i];}
 const auto result=world.move_and_slide(bounds,delta,mask,entity_index(&actor));
 for(unsigned i=0;i<3;++i)actor.transform.position[i]+=result.displacement[i];return result;
}
}
template<class T> static T* spawn(Level& level,const char* name){
 const auto id=level.spawn_actor(*find_object_class(T::static_class_id),name);auto* actor=active_object_registry->resolve<T>(id);assert(actor&&used<10);
 auto& data=slots[used++];data.alive=data.active=true;data.transform.scale[0]=data.transform.scale[1]=data.transform.scale[2]=1.0;
 actor->bind_data(data);actor->root.bind_slot(data);if constexpr(std::is_base_of_v<CameraRig3D,T>){data.camera=data.camera_settings.enabled=true;actor->automatic_tick=false;}
 return actor;
}
static void wall(Fixed z,uint32_t layer=1){Collider collider;collider.enabled=true;collider.layer=layer;collider.half_extents[0]=4.0;collider.half_extents[1]=3.0;collider.half_extents[2]=0.1;auto pose=Affine<Fixed>::identity();pose.values[2][3]=z;world.set(12,collider,pose);}
int main(){
 ObjectRegistryStorage<96> registry;active_object_registry=&registry;Level level;assert(level.bind(registry));
 auto* target=spawn<Actor3D>(level,"Target");auto* first=spawn<FirstPersonCamera>(level,"Same name");
 auto* third=spawn<ThirdPersonCamera>(level,"Same name");auto* top=spawn<TopDownCamera>(level,"Top");auto* iso=spawn<IsometricCamera>(level,"Iso");
 for(auto* camera:{static_cast<CameraRig3D*>(first),static_cast<CameraRig3D*>(third),static_cast<CameraRig3D*>(top),static_cast<CameraRig3D*>(iso)})camera->target=target->id();
 assert(first->component_count()==2&&third->component_count()==3&&top->component_count()==2&&iso->component_count()==2);
 assert(first->distance==0.0&&first->pitch==0.0&&first->offset_y==1.6&&top->pitch==90.0&&iso->yaw==45.0);
 assert(first->snap().status==CameraRigStatus::Applied&&first->data()->transform.position[1]==Fixed(1.6));
 assert(top->snap().position.y==Fixed(10.0));assert(iso->snap().position.x<0.0&&iso->sample().position.z<0.0);
 unsigned blocked=0;wall(-3.0);const Fixed dt(1.0/60.0);
 // Frozen original third-person boom order, using the real ray/obstruction query.
 for(unsigned frame=0;frame<720;++frame){
  third->yaw=Fixed(int32_t(frame%360),0);third->pitch=Fixed(int32_t(frame%66)-10,0);
  target->data()->transform.position[0]=Fixed(double(frame%80)*0.02);
  const auto& transform=target->data()->transform;const Fixed reach=Fixed(5.4)*MathLibrary::cosine_degrees(third->pitch);
  const Fixed sine=MathLibrary::sine_degrees(third->yaw),cosine=MathLibrary::cosine_degrees(third->yaw);
  const Fixed origin[3]={transform.position[0],transform.position[1]+Fixed(1.1),transform.position[2]};
  Fixed offset[3]={-sine*reach,Fixed(5.4)*MathLibrary::sine_degrees(third->pitch),-cosine*reach};
  const auto hit=raycast(origin,offset,0xffffffffu,target->data());
  if(hit){Fixed fraction=hit.fraction-Fixed(0.04);if(fraction<Fixed(0.12))fraction=0.12;for(auto& value:offset)value*=fraction;++blocked;}
  const auto view=third->refresh(dt);assert(view.status==CameraRigStatus::Applied&&view.blocked==bool(hit));
  for(unsigned i=0;i<3;++i)assert(third->data()->transform.position[i]==origin[i]+offset[i]);
  assert(third->data()->transform.rotation[0]==third->pitch&&third->data()->transform.rotation[1]==third->yaw);
 }
 assert(blocked>50);
 // Independent identity and rename, smoothing of focus, invalid/no-step transactions.
 third->set_name("Renamed");first->set_name("Renamed");world.clear();target->data()->transform.position[0]=0.0;third->snap();third->follow_response=6.0;
 const auto before=third->sample().position.x;target->data()->transform.position[0]=10.0;
 const auto smoothed=third->refresh(dt);assert(smoothed.position.x>before&&smoothed.position.x<before+Fixed(10.0));
 first->snap();assert(first->data()->transform.position[0]==10.0);
 const Fixed saved=third->data()->transform.position[0];const auto query_count=queries;
 assert(third->refresh(0.0).status==CameraRigStatus::NoStep&&queries==query_count&&third->data()->transform.position[0]==saved);
 third->distance=-1.0;assert(third->refresh(dt).status==CameraRigStatus::InvalidConfiguration&&third->data()->transform.position[0]==saved);third->distance=5.4;
 third->spring_arm=third->root.id();assert(third->refresh(dt).status==CameraRigStatus::InvalidArm);third->spring_arm={};
 // Retraction is immediate; extension is independently smoothed and filtered.
 camera_rig::ArmState arm;camera_rig::ArmTuning tuning;GameplayVector3 origin{},offset{0.0,0.0,-5.0};wall(-3.0);
 const auto close=camera_rig::solve(arm,tuning,origin,offset,nullptr,dt);assert(close.blocked&&close.fraction<1.0);
 world.clear();tuning.extension_response=6.0;const auto extending=camera_rig::solve(arm,tuning,origin,offset,nullptr,dt);assert(extending.fraction>close.fraction&&extending.fraction<1.0);
 wall(-1.0);const auto nearer=camera_rig::solve(arm,tuning,origin,offset,nullptr,dt);assert(nearer.fraction<close.fraction);
 tuning.mask=0;tuning.extension_response=0.0;assert(camera_rig::solve(arm,tuning,origin,offset,nullptr,dt).fraction==1.0);
 tuning.pull_in=-0.1;const auto previous=arm.fraction;assert(camera_rig::solve(arm,tuning,origin,offset,nullptr,dt).status==CameraRigStatus::InvalidConfiguration&&arm.fraction==previous);world.clear();
 // Shared queries retain the complete project's close-obstacle clearance,
 // invalidate on identity/input/publication changes, and reuse stable rays.
 auto* context=level.add_component<SpatialQueryContextComponent>(*target,"Queries");assert(context);context->automatic_reset=false;
 auto& shared=third->boom;shared.adaptive_pull_in=true;shared.minimum_fraction=0.0;shared.collision_mask=1;
 Fixed ray_origin[3],ray_offset[3];gameplay_vector(origin,ray_origin);gameplay_vector(offset,ray_offset);
 wall(-0.2);auto expected=raycast(ray_origin,ray_offset,1,target->data());
 assert(expected&&expected.fraction<Fixed(.08));
 const auto initial_queries=queries;
 auto view=shared.solve_shared(origin,offset,target->id(),dt,context->id(),1);
 assert(view.status==CameraRigStatus::Applied&&view.blocked&&view.fraction==expected.fraction-expected.fraction/Fixed(2.0));
 assert(queries==initial_queries+1);
 assert(shared.solve_shared(origin,offset,target->id(),dt,context->id(),1).fraction==view.fraction&&queries==initial_queries+1);
 context->reset_queries();shared.solve_shared(origin,offset,target->id(),dt,context->id(),1);assert(queries==initial_queries+2);
 shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==initial_queries+3);
 shared.solve_shared(origin,offset,target->id(),dt,context->id(),0);shared.solve_shared(origin,offset,target->id(),dt,context->id(),0);assert(queries==initial_queries+5);
 auto shifted=origin;shifted.y=0.2;shared.solve_shared(shifted,offset,target->id(),dt,context->id(),2);assert(queries==initial_queries+6);
 shared.collision_mask=0;assert(!shared.solve_shared(shifted,offset,target->id(),dt,context->id(),2).blocked&&queries==initial_queries+7);shared.collision_mask=1;
 const auto invalid_queries=queries;
 assert(shared.solve_shared(origin,offset,target->root.id(),dt,context->id(),2).status==CameraRigStatus::InvalidTarget&&queries==invalid_queries);
 assert(shared.solve_shared(origin,offset,target->id(),dt,target->root.id(),2).status==CameraRigStatus::InvalidTarget&&queries==invalid_queries);
 assert(level.set_active(target->id(),false));assert(!context->live_batch());
 assert(shared.solve_shared(origin,offset,{},dt,context->id(),2).status==CameraRigStatus::InvalidTarget&&queries==invalid_queries);
 assert(level.set_active(target->id(),true));shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==invalid_queries+1);
 const auto no_step_queries=queries;
 assert(shared.solve_shared(origin,offset,target->id(),0.0,context->id(),2).status==CameraRigStatus::NoStep&&queries==no_step_queries);
 Level foreign;assert(foreign.bind(registry));auto* stranger=spawn<Actor3D>(foreign,"Foreign");
 auto* foreign_context=foreign.add_component<SpatialQueryContextComponent>(*stranger,"Foreign queries");assert(foreign_context);
 assert(shared.solve_shared(origin,offset,stranger->id(),dt,context->id(),2).status==CameraRigStatus::InvalidTarget&&queries==no_step_queries);
 assert(shared.solve_shared(origin,offset,{},dt,foreign_context->id(),2).status==CameraRigStatus::InvalidTarget&&queries==no_step_queries);
 const auto stale_context=context->id();assert(level.remove_component(*target,stale_context));
 assert(shared.solve_shared(origin,offset,{},dt,stale_context,2).status==CameraRigStatus::InvalidTarget&&queries==no_step_queries);
 context=level.add_component<SpatialQueryContextComponent>(*target,"Replacement queries");assert(context&&context->id()!=stale_context);
 shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==no_step_queries+1);
 shared.reset();shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==no_step_queries+2);
 const auto publications=queries;
 assert(context->publish_actor(target->id()));shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==publications+1);
 assert(context->move(target->id(),{},1).valid);shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==publications+2);
 auto* second_arm=level.add_component<SpringArmComponent>(*first,"Independent arm");assert(second_arm);
 second_arm->solve_shared(origin,offset,target->id(),dt,context->id(),2);shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==publications+4);
 context->automatic_reset=true;context->tick(dt);shared.solve_shared(origin,offset,target->id(),dt,context->id(),2);assert(queries==publications+5);
 shared.solve_shared(origin,offset,{},dt,context->id(),2);assert(queries==publications+6);
 assert(shared.solve_shared(origin,offset,{},Fixed(-1.0),context->id(),2).status==CameraRigStatus::InvalidConfiguration&&queries==publications+6);
 foreign.end_play_all(EndPlayReason::LevelUnloaded);world.clear();shared.reset();shared.adaptive_pull_in=false;shared.minimum_fraction=0.12;shared.collision_mask=0xffffffffu;
 // Fixed camera uses its authored pose; switching changes only the active camera.
 auto* fixed=spawn<CameraRig3D>(level,"Fixed");fixed->follow_target=false;fixed->data()->transform.position[0]=30.0;fixed->data()->transform.rotation[2]=-12.0;
 assert(fixed->make_active()&&active_camera().get()==fixed->data()&&fixed->sample().position.x==30.0);
 assert(fixed->sample().roll==348.0&&fixed->data()->transform.rotation[2]==348.0);
 assert(first->make_active()&&active_camera().get()==first->data());
 auto* look=level.add_component<CameraOrbitInputComponent>(*first,"Input");assert(look);
 input.reset();gameplay_controls().reset();input.sample(0,true,0,true,128,128,255,128);gameplay_controls().begin_frame();input.begin_tick();gameplay_controls().begin_tick();
 first->set_angles(0.0,0.0);assert(look->update(dt)==CameraRigStatus::Applied&&first->yaw>0.0);
 auto capture=gameplay_controls().capture(target->id(),GameplayInputLibrary::look_actions(),100,0);assert(gameplay_controls().valid(capture));
 const auto locked=first->yaw;assert(look->update(dt)==CameraRigStatus::Applied&&first->yaw==locked);assert(gameplay_controls().release(capture));
 capture=gameplay_controls().capture(target->id(),GameplayInputLibrary::movement_actions(),100,0);look->update(dt);assert(first->yaw>locked);gameplay_controls().release(capture);
 assert(third->make_active());const auto inactive=first->yaw;assert(look->update(dt)==CameraRigStatus::NoStep&&first->yaw==inactive);
 assert(level.set_active(first->id(),false));assert(look->update(dt)==CameraRigStatus::InvalidOwner);assert(level.set_active(first->id(),true));
 auto* callback=spawn<CallbackCamera>(level,"Callback");callback->target=target->id();
 assert(callback->snap().status==CameraRigStatus::Applied&&callback->nested==CameraRigStatus::Busy);
 callback->follow_target=false;callback->custom_fixed=true;const auto fixed_x=callback->data()->transform.position[0];
 assert(callback->snap().status==CameraRigStatus::Applied&&callback->data()->transform.position[0]==fixed_x+Fixed(1.0)&&callback->data()->transform.rotation[2]==12.0);
 const auto callback_id=callback->id();callback->destroy_updated=true;assert(!callback->make_active()&&!is_active(callback));level.tick(0.0);assert(!registry.get(callback_id));
 callback=spawn<CallbackCamera>(level,"Destroyed in calculate");callback->target=target->id();callback->destroy_calculated=true;
 const auto calculated_id=callback->id();assert(callback->snap().status==CameraRigStatus::InvalidOwner&&!is_active(callback));level.tick(0.0);assert(!registry.get(calculated_id));
 const auto target_id=target->id();assert(level.destroy_actor(target_id));assert(first->snap().status==CameraRigStatus::InvalidTarget);
 level.end_play_all(EndPlayReason::LevelUnloaded);
 std::printf("Camera rigs: 720 Q12 original boom frames (%u obstructed), four presets, independent targets, follow/arm smoothing, fixed camera, switching and routed look capture passed\n",blocked);
}
