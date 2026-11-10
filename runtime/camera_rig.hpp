#pragma once
// World-unit camera mechanisms over the existing Actor3D, camera and ray services.
#define EPOK_INCLUDE_FROM_CAMERA_RIG 1
#include "gameplay_api.hpp"
#include "gameplay_control.hpp"
#undef EPOK_INCLUDE_FROM_CAMERA_RIG

namespace epok {
enum class CameraRigStatus:uint8_t {Applied,NoStep,InvalidOwner,InvalidTarget,InvalidConfiguration,InvalidArm,Busy};
struct EPOK_VALUE(Id="a5c65cff-9953-5f02-94d8-127d0a469ed5") SpringArmSample {
    CameraRigStatus status=CameraRigStatus::NoStep;
    GameplayVector3 position{};
    Fixed fraction=1.0;
    bool blocked=false;
};
struct EPOK_VALUE(Id="92a4560c-840a-5a47-b50f-a76663581391") CameraRigSample {
    CameraRigStatus status=CameraRigStatus::NoStep;
    GameplayVector3 position{};
    Fixed yaw=0.0,pitch=0.0,roll=0.0,distance=0.0;
    bool blocked=false;
};
namespace camera_rig {
inline bool bounded(Fixed value,int32_t maximum){return int64_t(value.raw())>=-int64_t(maximum)*4096&&int64_t(value.raw())<=int64_t(maximum)*4096;}
inline GameplayVector3 boom(Fixed yaw,Fixed pitch,Fixed distance){
    const Fixed reach=distance*MathLibrary::cosine_degrees(pitch);
    return {-MathLibrary::sine_degrees(yaw)*reach,distance*MathLibrary::sine_degrees(pitch),-MathLibrary::cosine_degrees(yaw)*reach};
}
struct ArmTuning {
    bool collision_enabled=true;
    uint32_t mask=0xffffffffu;
    Fixed pull_in=0.04,minimum_fraction=0.12,extension_response=0.0;
    bool adaptive_pull_in=false;
};
struct ArmState {Fixed fraction=1.0;bool initialized=false;};
struct WorldQueries {
    SpatialHit cast(const Fixed* origin,const Fixed* displacement,uint32_t mask,const ActorData* ignore){return raycast(origin,displacement,mask,ignore);}
};
template<class Queries>
inline SpringArmSample solve(ArmState& state,const ArmTuning& tuning,GameplayVector3 origin,GameplayVector3 offset,const ActorData* ignore,Fixed dt,Queries& queries){
    SpringArmSample sample;sample.position=origin;sample.fraction=state.fraction;
    if(dt.raw()==0)return sample;
    if(dt<0.0||dt>1.0||tuning.pull_in<0.0||tuning.pull_in>1.0||tuning.minimum_fraction<0.0||tuning.minimum_fraction>1.0
        ||tuning.extension_response<0.0||!bounded(tuning.extension_response,4096)
        ||!bounded(offset.x,4096)||!bounded(offset.y,4096)||!bounded(offset.z,4096)) {sample.status=CameraRigStatus::InvalidConfiguration;return sample;}
    Fixed fraction=1.0;bool blocked=false;
    if(tuning.collision_enabled&&(offset.x.raw()||offset.y.raw()||offset.z.raw())){
        const Fixed from[3]={origin.x,origin.y,origin.z},delta[3]={offset.x,offset.y,offset.z};
        const auto hit=queries.cast(from,delta,tuning.mask,ignore);
        if(hit){
            blocked=true;
            const Fixed clearance=tuning.adaptive_pull_in&&hit.fraction<tuning.pull_in*Fixed(2.0)?hit.fraction/Fixed(2.0):tuning.pull_in;
            fraction=hit.fraction-clearance;if(fraction<tuning.minimum_fraction)fraction=tuning.minimum_fraction;
        }
    }
    // Retract immediately; smooth only the safe extension after an obstruction.
    if(state.initialized&&fraction>state.fraction&&tuning.extension_response>0.0){
        const Fixed alpha=MathLibrary::clamp(tuning.extension_response*dt,Fixed(0.0),Fixed(1.0));
        fraction=state.fraction+(fraction-state.fraction)*alpha;
    }
    state.fraction=fraction;state.initialized=true;
    sample.status=CameraRigStatus::Applied;sample.fraction=fraction;sample.blocked=blocked;
    sample.position={origin.x+offset.x*fraction,origin.y+offset.y*fraction,origin.z+offset.z*fraction};return sample;
}
inline SpringArmSample solve(ArmState& state,const ArmTuning& tuning,GameplayVector3 origin,GameplayVector3 offset,const ActorData* ignore,Fixed dt){
    WorldQueries queries;return solve(state,tuning,origin,offset,ignore,dt,queries);
}
}
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="37538bc0-b257-52af-be93-b344301667b1") SpringArmComponent:public ActorComponent {
    camera_rig::ArmState state_{};
    SpringArmSample sample_{};
    camera_rig::ArmTuning tuning()const{return {collision_enabled,collision_mask,pull_in,minimum_fraction,extension_response,adaptive_pull_in};}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("37538bc0-b257-52af-be93-b344301667b1");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) bool collision_enabled=true;
    EPOK_PROPERTY(EditAnywhere) uint32_t collision_mask=0xffffffffu;
    EPOK_PROPERTY(EditAnywhere) Fixed pull_in=0.04;
    EPOK_PROPERTY(EditAnywhere) Fixed minimum_fraction=0.12;
    EPOK_PROPERTY(EditAnywhere) Fixed extension_response=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="162dcebb-6fc7-5170-b069-936e3931a832") bool adaptive_pull_in=false;
    EPOK_FUNCTION(BlueprintCallable) SpringArmSample solve(GameplayVector3 origin,GameplayVector3 offset,ObjectId ignore,Fixed delta_seconds){
        auto* owner=get_owner();if(!owner||!is_active(owner)){sample_={};sample_.status=CameraRigStatus::InvalidOwner;return sample_;}
        auto* ignored=gameplay_actor_data(ignore);
        if(ignore.valid()&&!ignored){sample_={};sample_.status=CameraRigStatus::InvalidTarget;return sample_;}
        sample_=camera_rig::solve(state_,tuning(),origin,offset,ignored,delta_seconds);return sample_;
    }
    EPOK_FUNCTION(BlueprintCallable,Id="ad455611-12f5-5e62-ab11-2a192a23b9c2") SpringArmSample solve_shared(GameplayVector3 origin,GameplayVector3 offset,ObjectId ignore,Fixed delta_seconds,ObjectId query_context,uint32_t collision_revision){
        auto* owner=get_owner();
        if(m_ended||!owner||owner->pending_destruction()||!is_active(owner)){sample_={};sample_.status=CameraRigStatus::InvalidOwner;return sample_;}
        auto* context=active_object_registry?active_object_registry->resolve<SpatialQueryContextComponent>(query_context):nullptr;
        auto* context_owner=context?context->get_owner():nullptr;
        auto* batch=context_owner&&context_owner->level_id()==owner->level_id()?context->live_batch():nullptr;
        auto* ignored=gameplay_actor_data(ignore);
        if(!batch||(ignore.valid()&&(!ignored||!ignored->owner||ignored->owner->level_id()!=owner->level_id()))){sample_={};sample_.status=CameraRigStatus::InvalidTarget;return sample_;}
        struct SharedQueries {
            SpatialQueryContextComponent& context;ObjectId consumer,ignore;uint32_t revision;bool allow_cache;
            SpatialHit cast(const Fixed* from,const Fixed* delta,uint32_t mask,const ActorData* ignored){
                return context.cached_ray(consumer,from,delta,mask,ignored,ignore,revision,allow_cache);
            }
        } queries{*context,id(),ignore,collision_revision,state_.initialized};
        sample_=camera_rig::solve(state_,tuning(),origin,offset,ignored,delta_seconds,queries);
        return sample_;
    }
    EPOK_FUNCTION(BlueprintCallable) void reset(){state_={};sample_={};}
    EPOK_FUNCTION(BlueprintPure) SpringArmSample sample()const{return sample_;}
    void begin_play()override{reset();}void on_enable()override{reset();}void on_disable()override{reset();}void end_play(EndPlayReason)override{reset();}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="c95b4c92-f2c9-5c5d-985a-ff4cc0f6c8c0") CameraRig3D:public Actor3D {
    GameplayVector3 focus_{};
    ObjectId last_target_{};
    bool initialized_=false,refreshing_=false;
    CameraRigSample sample_{};
    SpringArmComponent* arm(){
        auto* registry=active_object_registry;if(!registry)return nullptr;
        auto* value=spring_arm.valid()?registry->resolve<SpringArmComponent>(spring_arm):nullptr;
        if(!spring_arm.valid())if(auto* level=registry->resolve<Level>(level_id()))value=level->get_component<SpringArmComponent>(*this);
        return value&&value->owner_id()==id()?value:nullptr;
    }
    bool valid_tuning()const{return distance>=0.0&&camera_rig::bounded(distance,4096)&&pitch_min>=-90.0&&pitch_max<=90.0&&pitch_min<=pitch_max
        &&pitch>=pitch_min&&pitch<=pitch_max&&follow_response>=0.0&&camera_rig::bounded(follow_response,4096)
        &&camera_rig::bounded(offset_x,4096)&&camera_rig::bounded(offset_y,4096)&&camera_rig::bounded(offset_z,4096);}
public:
    static constexpr TickGroup tick_group=TickGroup::PostPhysics;
    static constexpr uint64_t static_class_id=detail::compact_class_id("c95b4c92-f2c9-5c5d-985a-ff4cc0f6c8c0");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Camera") Camera3DComponent camera;
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpringArmComponent") ObjectId spring_arm{};
    EPOK_PROPERTY(EditAnywhere) Fixed offset_x=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed offset_y=1.1;
    EPOK_PROPERTY(EditAnywhere) Fixed offset_z=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed distance=5.4;
    EPOK_PROPERTY(EditAnywhere) Fixed yaw=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed pitch=18.0;
    EPOK_PROPERTY(EditAnywhere) Fixed pitch_min=-10.0;
    EPOK_PROPERTY(EditAnywhere) Fixed pitch_max=55.0;
    EPOK_PROPERTY(EditAnywhere) Fixed follow_response=0.0;
    EPOK_PROPERTY(EditAnywhere) bool follow_target=true;
    EPOK_PROPERTY(EditAnywhere) bool automatic_tick=true;
    EPOK_FUNCTION(BlueprintCallable) bool set_angles(Fixed yaw_degrees,Fixed pitch_degrees){
        if(pitch_min<-90.0||pitch_max>90.0||pitch_min>pitch_max)return false;
        yaw=MathLibrary::wrap_degrees(yaw_degrees);pitch=MathLibrary::clamp(pitch_degrees,pitch_min,pitch_max);return true;
    }
    EPOK_FUNCTION(BlueprintCallable) bool add_look(GameplayVector2 delta_degrees){
        const int64_t x=int64_t(yaw.raw())+delta_degrees.x.raw(),y=int64_t(pitch.raw())+delta_degrees.y.raw();
        const int32_t raw_y=int32_t(y<INT32_MIN?INT32_MIN:y>INT32_MAX?INT32_MAX:y);
        return set_angles(Fixed(int32_t(x%(360*4096)),Fixed::RAW),Fixed(raw_y,Fixed::RAW));
    }
    EPOK_FUNCTION(BlueprintEvent) virtual CameraRigSample calculate_view(Fixed delta_seconds){
        CameraRigSample result;result.yaw=yaw;result.pitch=pitch;result.distance=distance;
        auto* data=this->data();
        if(!data||!is_active(this)||root.attach_parent.valid()){result.status=CameraRigStatus::InvalidOwner;return result;}
        if(delta_seconds.raw()==0)return result;
        if(delta_seconds<0.0||delta_seconds>1.0||!valid_tuning()){result.status=CameraRigStatus::InvalidConfiguration;return result;}
        if(!follow_target){
            const auto world=gameplay_world_affine(data);if(!world.success){result.status=CameraRigStatus::InvalidOwner;return result;}
            result.position={world.position[0],world.position[1],world.position[2]};result.yaw=data->transform.rotation[1];result.pitch=data->transform.rotation[0];result.roll=data->transform.rotation[2];
            result.status=CameraRigStatus::Applied;return result;
        }
        auto* actor=active_object_registry?active_object_registry->resolve<Actor3D>(target):nullptr;
        if(!actor||actor==this||actor->level_id()!=level_id()||!is_active(actor)){result.status=CameraRigStatus::InvalidTarget;return result;}
        const auto world=gameplay_world_affine(actor->data());if(!world.success){result.status=CameraRigStatus::InvalidTarget;return result;}
        GameplayVector3 focus{world.position[0]+offset_x,world.position[1]+offset_y,world.position[2]+offset_z};
        if(initialized_&&last_target_==target&&follow_response>0.0){
            const Fixed alpha=MathLibrary::clamp(follow_response*delta_seconds,Fixed(0.0),Fixed(1.0));
            focus={focus_.x+(focus.x-focus_.x)*alpha,focus_.y+(focus.y-focus_.y)*alpha,focus_.z+(focus.z-focus_.z)*alpha};
        }
        const auto offset=camera_rig::boom(yaw,pitch,distance);auto* boom=arm();
        if(spring_arm.valid()&&!boom){result.status=CameraRigStatus::InvalidArm;return result;}
        if(boom){const auto solved=boom->solve(focus,offset,target,delta_seconds);if(solved.status!=CameraRigStatus::Applied){result.status=solved.status;return result;}result.position=solved.position;result.blocked=solved.blocked;}
        else result.position={focus.x+offset.x,focus.y+offset.y,focus.z+offset.z};
        focus_=focus;last_target_=target;initialized_=true;result.status=CameraRigStatus::Applied;return result;
    }
    EPOK_FUNCTION(BlueprintEvent) virtual void view_updated(CameraRigSample view){(void)view;}
    EPOK_FUNCTION(BlueprintCallable) CameraRigSample refresh(Fixed delta_seconds){
        auto* registry=active_object_registry;auto* data=this->data();
        if(!registry||!data||!is_active(this)||root.attach_parent.valid()){sample_={};sample_.status=CameraRigStatus::InvalidOwner;return sample_;}
        if(refreshing_){CameraRigSample busy;busy.status=CameraRigStatus::Busy;return busy;}
        ObjectDispatchScope scope(*registry);const ObjectId self=id();
        struct RefreshScope {bool& value;explicit RefreshScope(bool& flag):value(flag){value=true;}~RefreshScope(){value=false;}} refresh_scope(refreshing_);
        auto view=calculate_view(delta_seconds);
        if(registry->resolve<CameraRig3D>(self)!=this||!is_active(this)||!this->data()){view.status=CameraRigStatus::InvalidOwner;return view;}
        if(view.status==CameraRigStatus::Applied){
            if(!camera_rig::bounded(view.pitch,90)||view.distance<0.0||!camera_rig::bounded(view.distance,4096)){view.status=CameraRigStatus::InvalidConfiguration;sample_=view;return view;}
            view.yaw=MathLibrary::wrap_degrees(view.yaw);view.roll=MathLibrary::wrap_degrees(view.roll);
            data->transform.position[0]=view.position.x;data->transform.position[1]=view.position.y;data->transform.position[2]=view.position.z;
            data->transform.rotation[0]=view.pitch;data->transform.rotation[1]=view.yaw;data->transform.rotation[2]=view.roll;
            sample_=view;view_updated(view);return view;
        }
        sample_=view;return view;
    }
    EPOK_FUNCTION(BlueprintCallable) CameraRigSample snap(){initialized_=false;last_target_={};if(auto* boom=arm())boom->reset();return refresh(Fixed(1.0/60.0));}
    EPOK_FUNCTION(BlueprintPure) CameraRigSample sample()const{return sample_;}
    EPOK_FUNCTION(BlueprintCallable) bool make_active(){
        auto* registry=active_object_registry;if(!registry)return false;
        ObjectDispatchScope scope(*registry);const ObjectId self=id();const auto view=snap();
        auto* camera=registry->resolve<CameraRig3D>(self);
        return view.status==CameraRigStatus::Applied&&camera==this&&is_active(camera)&&set_active_camera(camera->data());
    }
    void begin_play()override{initialized_=false;last_target_={};sample_={};}
    void tick(Fixed dt)override{if(automatic_tick)refresh(dt);}
    void on_disable()override{initialized_=false;last_target_={};}
    void end_play(EndPlayReason)override{initialized_=false;last_target_={};sample_={};}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="121b95af-7e66-50df-bcc1-7b61da65ae7b") FirstPersonCamera:public CameraRig3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("121b95af-7e66-50df-bcc1-7b61da65ae7b");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    FirstPersonCamera(){distance=0.0;offset_y=1.6;pitch=0.0;pitch_min=-85.0;pitch_max=85.0;}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="d43dbec0-82ca-5088-8b34-b7380fd4e1d4") ThirdPersonCamera:public CameraRig3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("d43dbec0-82ca-5088-8b34-b7380fd4e1d4");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Spring Arm") SpringArmComponent boom;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="1c40f100-3758-5838-b19a-f08d5744e2be") TopDownCamera:public CameraRig3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("1c40f100-3758-5838-b19a-f08d5744e2be");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    TopDownCamera(){distance=10.0;offset_y=0.0;pitch=90.0;pitch_min=90.0;pitch_max=90.0;}
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="1c06764a-ac44-5a70-a9cf-ed827a4cf9ba") IsometricCamera:public CameraRig3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("1c06764a-ac44-5a70-a9cf-ed827a4cf9ba");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    IsometricCamera(){distance=8.0;offset_y=0.0;yaw=45.0;pitch=35.264;pitch_min=35.264;pitch_max=35.264;}
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Capability=gameplay-control,Id="ad1162b7-8041-5495-8558-991f319cc5c8") CameraOrbitInputComponent:public ActorComponent {
public:
    static constexpr TickGroup tick_group=TickGroup::Input;
    static constexpr uint64_t static_class_id=detail::compact_class_id("ad1162b7-8041-5495-8558-991f319cc5c8");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") ObjectId rig{};
    EPOK_PROPERTY(EditAnywhere) uint32_t port=0;
    EPOK_PROPERTY(EditAnywhere) Fixed yaw_rate=140.0;
    EPOK_PROPERTY(EditAnywhere) Fixed pitch_rate=70.0;
    EPOK_PROPERTY(EditAnywhere) bool enabled=true;
    EPOK_PROPERTY(EditAnywhere) bool only_when_current=true;
    EPOK_FUNCTION(BlueprintCallable) CameraRigStatus update(Fixed delta_seconds){
        if(!enabled||delta_seconds.raw()==0)return CameraRigStatus::NoStep;
        if(port>=4||delta_seconds<0.0||delta_seconds>1.0||yaw_rate<0.0||pitch_rate<0.0||!camera_rig::bounded(yaw_rate,4096)||!camera_rig::bounded(pitch_rate,4096))return CameraRigStatus::InvalidConfiguration;
        auto* owner=get_owner();if(!owner||!is_active(owner))return CameraRigStatus::InvalidOwner;
        auto* registry=active_object_registry;auto* camera=registry?registry->resolve<CameraRig3D>(rig.valid()?rig:owner_id()):nullptr;
        if(!camera||camera->level_id()!=owner->level_id()||!is_active(camera))return CameraRigStatus::InvalidTarget;
        if(only_when_current&&active_camera().get()!=camera->data())return CameraRigStatus::NoStep;
        const auto look=MathLibrary::stick_intent(Fixed(GameplayInputLibrary::axis_raw(Axis::RightX,port),Fixed::RAW),Fixed(GameplayInputLibrary::axis_raw(Axis::RightY,port),Fixed::RAW));
        return camera->add_look({look.x*yaw_rate*delta_seconds,-look.y*pitch_rate*delta_seconds})?CameraRigStatus::Applied:CameraRigStatus::InvalidConfiguration;
    }
    void tick(Fixed dt)override{update(dt);}
};
}
