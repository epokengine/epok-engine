#pragma once
#include "navigation.hpp"
namespace epok {
namespace nav {
// Installed by the World3D runtime. Host HUD preview has no collision world;
// leaving this null avoids linking or pretending to simulate unavailable physics.
inline MoveResult (*move_actor)(ActorData&,const Fixed*,uint32_t)=nullptr;
}
// The owner's unit cube Transform defines bake bounds. No renderer/collider needed.
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a01") NavigationBakeVolumeComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a01");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) Fixed spacing=0.5;
    EPOK_PROPERTY(EditAnywhere) Fixed radius=0.18;
    EPOK_PROPERTY(EditAnywhere) Fixed height=0.5;
    EPOK_PROPERTY(EditAnywhere) uint32_t collision_mask=1;
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a02") NavigationAgentComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a02");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) Fixed speed=0.6;
    EPOK_PROPERTY(EditAnywhere) bool face_movement=true;
    EPOK_PROPERTY(EditAnywhere) int32_t moving_clip=-1;
    EPOK_PROPERTY(EditAnywhere) int32_t idle_clip=-1;
    EPOK_PROPERTY(EditAnywhere) uint32_t collision_mask=0xffffffffu;
    EPOK_PROPERTY(EditAnywhere) bool move_on_start=false;
    EPOK_PROPERTY(EditAnywhere) Fixed target_x=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed target_y=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed target_z=0.0;
    EPOK_FUNCTION(BlueprintCallable) bool move_to(Fixed x,Fixed y,Fixed z){
        stop();auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        // Foot-origin, unparented agent. Keep local and world motion unambiguous.
        if(!data||data->parent>=0||!data->collider.enabled)return false;
        int32_t from[3],to[]={x.raw()/16,y.raw()/16,z.raw()/16};
        for(int k=0;k<3;++k)from[k]=data->transform.position[k].raw()/16;
        request_=nav::world.request(from,to);last_=nav::Status::Queued;next_=0;
        if(!nav::world.get(request_)){last_=nav::Status::Blocked;return false;}return true;
    }
    EPOK_FUNCTION(BlueprintCallable) void stop(){nav::world.cancel(request_);request_={};last_=nav::Status::Idle;}
    EPOK_FUNCTION(BlueprintPure) uint32_t status() const{auto* r=nav::world.get(request_);return uint32_t(r?r->status:last_);}
    EPOK_FUNCTION(BlueprintPure) bool arrived() const{return status()==uint32_t(nav::Status::Arrived);}
    EPOK_FUNCTION(BlueprintPure) bool failed() const{const auto s=status();return s>=uint32_t(nav::Status::NoPath)&&s<=uint32_t(nav::Status::Blocked);}
    void begin_play() override{if(move_on_start)move_to(target_x,target_y,target_z);}
    void end_play(EndPlayReason) override{stop();}
    void on_disable() override{stop();}
    void tick(Fixed dt) override{
        auto* r=nav::world.get(request_);if(!r)return;
        if(r->status!=nav::Status::Ready){
            if(r->status!=nav::Status::Searching&&r->status!=nav::Status::Queued)finish(r->status);
            return;
        }
        auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        if(!data){finish(nav::Status::Blocked);return;}
        if(next_>=r->length){finish(nav::Status::Arrived);return;}
        const auto& node=nav::world.graph.nodes[r->path[r->length-1-next_]];
        const int32_t target[]={int32_t(node.x)*16,int32_t(node.y)*16,int32_t(node.z)*16};
        Fixed delta[3];int32_t length=0;
        for(int k=0;k<3;++k){delta[k]=Fixed(target[k]-data->transform.position[k].raw(),Fixed::RAW);const auto v=delta[k].raw();length+=v<0?-v:v;}
        if(length<=32){++next_;return;}
        if(moving_clip>=0&&data->animator.clip!=moving_clip)data->animator.play(moving_clip);
        if(face_movement){
            const auto x=delta[0].raw(),z=delta[2].raw();
            const auto ax=x<0?-x:x,az=z<0?-z:z;
            if(ax>32||az>32)data->transform.rotation[1]=Fixed((ax>az?(x>0?90:270):(z>0?0:180))*4096,Fixed::RAW);
        }
        const Fixed amount=speed*dt;
        if(amount.raw()<=0)return;
        // Divide first: (short distance * small dt) would round to zero in Q12
        // before division and leave agents stuck just short of a waypoint.
        if(length>amount.raw()){
            const Fixed ratio=amount/Fixed(length,Fixed::RAW);
            for(auto& v:delta)v=v*ratio;
        }
        if(!nav::move_actor){finish(nav::Status::Blocked);return;}
        const auto result=nav::move_actor(*data,delta,collision_mask);
        if(result.unresolved_overlap||result.blocked)finish(nav::Status::Blocked);
    }
private:
    nav::Handle request_{};nav::Status last_=nav::Status::Idle;uint16_t next_=0;
    void finish(nav::Status value){
        nav::world.cancel(request_);request_={};last_=value;
        auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        if(data&&idle_clip>=0&&data->animator.clip!=idle_clip)data->animator.play(idle_clip);
    }
};
}
