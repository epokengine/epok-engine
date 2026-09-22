#pragma once
#include "navigation.hpp"
namespace epok {
namespace nav {
// Installed by the World3D runtime. Host HUD preview has no collision world;
// leaving this null avoids linking or pretending to simulate unavailable physics.
inline MoveResult (*move_actor)(ActorData&,const Fixed*,uint32_t)=nullptr;
inline bool (*bounds_actor)(const ActorData&,Aabb&)=nullptr;
inline bool move_walk(ActorData& data,const Fixed* delta,uint32_t mask){
    if(!move_actor)return false;
    Fixed saved[3];for(int k=0;k<3;++k)saved[k]=data.transform.position[k];
    const int32_t x=(saved[0]+delta[0]).raw()/16,z=(saved[2]+delta[2]).raw()/16,near=saved[1].raw()/16;
    int32_t y=near;
    if(world.graph.surface_count){
        const int32_t reach=world.graph.step+world.graph.radius*2+4;
        if(!world.floor(x,z,near,reach,y))return false;
        // A box must clear the highest supporting point under its footprint.
        for(int dx=-1;dx<=1;dx+=2)for(int dz=-1;dz<=1;dz+=2){int32_t corner;
            if(!world.floor(x+dx*world.graph.radius,z+dz*world.graph.radius,near,reach,corner))return false;
            if(corner>y)y=corner;
        }
    }else y=(saved[1]+delta[1]).raw()/16;
    const Fixed lift(y*16+2-saved[1].raw(),Fixed::RAW);
    Fixed up[3]{};if(lift.raw()>0)up[1]=lift;
    if(up[1].raw()){
        const auto hit=move_actor(data,up,mask);
        if(hit.unresolved_overlap||hit.displacement[1].raw()+4<up[1].raw()){for(int k=0;k<3;++k)data.transform.position[k]=saved[k];return false;}
    }
    Fixed along[3]={delta[0],0.0,delta[2]};const auto hit=move_actor(data,along,mask);
    if(hit.unresolved_overlap||(hit.blocked&&(hit.normal[0].raw()||hit.normal[2].raw()))){for(int k=0;k<3;++k)data.transform.position[k]=saved[k];return false;}
    if(lift<0.0){Fixed down[3]={0.0,lift,0.0};const auto hit=move_actor(data,down,mask);if(hit.unresolved_overlap){for(int k=0;k<3;++k)data.transform.position[k]=saved[k];return false;}}
    return true;
}
}
// The owner's unit cube Transform defines bake bounds. No renderer/collider needed.
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a01") NavigationBakeVolumeComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a01");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) Fixed spacing=0.5;
    EPOK_PROPERTY(EditAnywhere) Fixed radius=0.18;
    EPOK_PROPERTY(EditAnywhere) Fixed height=0.5;
    EPOK_PROPERTY(EditAnywhere) Fixed step_height=0.4;
    EPOK_PROPERTY(EditAnywhere) Fixed max_slope=45.0;
    EPOK_PROPERTY(EditAnywhere) uint32_t collision_mask=1;
};
// Opt-in: bake the owner's EditableMesh triangles, including their transform.
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a03") NavigationSurfaceComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a03");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a04") NavigationLinkComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a04");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) Fixed end_x=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed end_y=0.0;
    EPOK_PROPERTY(EditAnywhere) Fixed end_z=2.0;
    EPOK_PROPERTY(EditAnywhere) uint32_t kind=1;
    EPOK_PROPERTY(EditAnywhere) Fixed arc_height=1.0;
    EPOK_PROPERTY(EditAnywhere) Fixed duration=1.0;
    EPOK_PROPERTY(EditAnywhere) bool bidirectional=true;
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a05") NavigationObstacleComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a05");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    void begin_play() override{sync();}
    void on_enable() override{sync();}
    void tick(Fixed) override{sync();}
    void on_disable() override{release();}
    void end_play(EndPlayReason) override{release();}
    EPOK_FUNCTION(BlueprintPure) bool registered() const{return slot_!=nav::invalid;}
private:
    uint16_t slot_=nav::invalid;
    void release(){nav::world.remove_obstacle(slot_);slot_=nav::invalid;}
    void sync(){auto* owner=get_owner();auto* data=owner?owner->data():nullptr;Aabb b;
        if(!data||!nav::bounds_actor||!nav::bounds_actor(*data,b)||data->collider.trigger){release();return;}
        if(slot_==nav::invalid)slot_=nav::world.add_obstacle();if(slot_==nav::invalid)return;
        int32_t lo[3],hi[3];for(int k=0;k<3;++k){lo[k]=b.min[k].raw()/16;hi[k]=b.max[k].raw()/16;}nav::world.set_obstacle(slot_,lo,hi);
    }
};
class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Id="e56c5741-4b0f-4861-a732-e91430c72a02") NavigationAgentComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("e56c5741-4b0f-4861-a732-e91430c72a02");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) Fixed speed=0.6;
    EPOK_PROPERTY(EditAnywhere) bool face_movement=true;
    EPOK_PROPERTY(EditAnywhere) bool avoid_agents=true;
    EPOK_PROPERTY(EditAnywhere) bool auto_repath=true;
    EPOK_PROPERTY(EditAnywhere) Fixed repath_delay=0.5;
    EPOK_PROPERTY(EditAnywhere) Fixed blocked_timeout=5.0;
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
        request_=nav::world.request(from,to);last_=nav::Status::Queued;next_=0;elapsed_=wait_=blocked_time_=0.0;revision_=nav::world.revision;
        if(!nav::world.get(request_)){last_=nav::Status::Blocked;return false;}return true;
    }
    EPOK_FUNCTION(BlueprintCallable) void stop(){nav::world.cancel(request_);request_={};last_=nav::Status::Idle;}
    EPOK_FUNCTION(BlueprintPure) uint32_t status() const{auto* r=nav::world.get(request_);return uint32_t(r?r->status:last_);}
    EPOK_FUNCTION(BlueprintPure) bool arrived() const{return status()==uint32_t(nav::Status::Arrived);}
    EPOK_FUNCTION(BlueprintPure) bool failed() const{const auto s=status();if(auto_repath&&s==uint32_t(nav::Status::NoPath)&&nav::world.get(request_))return false;return s>=uint32_t(nav::Status::NoPath)&&s<=uint32_t(nav::Status::Blocked);}
    void begin_play() override{if(move_on_start)move_to(target_x,target_y,target_z);}
    void end_play(EndPlayReason) override{stop();}
    void on_disable() override{stop();}
    void tick(Fixed dt) override{
        auto* r=nav::world.get(request_);if(!r)return;
        auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        if(!data){finish(nav::Status::Blocked);return;}
        int32_t position[3];for(int k=0;k<3;++k)position[k]=data->transform.position[k].raw()/16;
        nav::world.locate(request_,position);
        if(revision_!=r->revision){revision_=r->revision;next_=0;elapsed_=0.0;}
        if(r->status!=nav::Status::Ready){
            if(r->status==nav::Status::NoPath&&auto_repath){retry(dt,nav::invalid);return;}
            if(r->status!=nav::Status::Searching&&r->status!=nav::Status::Queued)finish(r->status);
            return;
        }
        if(next_>=r->length){finish(nav::Status::Arrived);return;}
        const auto target_node=r->path[r->length-1-next_];
        const auto previous=next_?r->path[r->length-next_]:target_node;
        if(next_&&!nav::world.edge_open(previous,target_node)){retry(dt,target_node);return;}
        if(avoid_agents&&!nav::world.reserve(request_,previous,target_node)){retry(dt,target_node);return;}
        const auto& node=nav::world.graph.nodes[target_node];
        if(next_)if(const auto* link=nav::world.traversal(previous,target_node)){
            if(!nav::move_actor){finish(nav::Status::Blocked);return;}
            const auto& start=nav::world.graph.nodes[previous];
            elapsed_+=dt;const int32_t t=elapsed_.raw()>=link->duration?4096:int32_t(int64_t(elapsed_.raw())*4096/link->duration);
            int32_t a[]={start.x,start.y,start.z},b[]={node.x,node.y,node.z};Fixed delta[3];
            for(int k=0;k<3;++k){int32_t fraction=t;if(link->kind==2)fraction=k==1?(t<2048?t*2:4096):(t<2048?0:t*2-4096);
                int32_t value=a[k]*16+int32_t(int64_t(b[k]-a[k])*16*fraction/4096);
                if(k==1&&link->kind==1)value+=int32_t(int64_t(4)*link->arc*16*t*(4096-t)/(4096*4096));
                delta[k]=Fixed(value-data->transform.position[k].raw(),Fixed::RAW);
            }
            const auto hit=nav::move_actor(*data,delta,collision_mask);
            if(hit.unresolved_overlap||(hit.blocked&&!hit.grounded)){finish(nav::Status::Blocked);return;}
            if(t==4096){elapsed_=0.0;++next_;nav::world.occupy(request_,target_node);}return;
        }
        const int32_t target[]={int32_t(node.x)*16,int32_t(node.y)*16,int32_t(node.z)*16};
        Fixed delta[3];int32_t length=0;
        for(int k=0;k<3;++k){delta[k]=Fixed(target[k]-data->transform.position[k].raw(),Fixed::RAW);const auto v=delta[k].raw();length+=v<0?-v:v;}
        const auto horizontal=(delta[0].raw()<0?-delta[0].raw():delta[0].raw())+(delta[2].raw()<0?-delta[2].raw():delta[2].raw());
        if(horizontal<=32&&(delta[1].raw()<0?-delta[1].raw():delta[1].raw())<=(nav::world.graph.step+nav::world.graph.radius*2)*16+32){++next_;nav::world.occupy(request_,target_node);wait_=0.0;return;}
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
        if(!nav::move_walk(*data,delta,collision_mask))retry(dt,target_node);else wait_=blocked_time_=0.0;
    }
private:
    nav::Handle request_{};nav::Status last_=nav::Status::Idle;uint16_t next_=0;
    Fixed elapsed_=0.0,wait_=0.0,blocked_time_=0.0;uint32_t revision_=0;
    void retry(Fixed dt,uint16_t avoid){
        if(!auto_repath){finish(nav::Status::Blocked);return;}
        blocked_time_+=dt;if(blocked_timeout>0.0&&blocked_time_>=blocked_timeout){finish(nav::Status::Blocked);return;}
        wait_+=dt;if(wait_<repath_delay)return;
        nav::world.replan(request_,avoid);next_=0;elapsed_=wait_=0.0;
    }
    void finish(nav::Status value){
        nav::world.cancel(request_);request_={};last_=value;
        auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        if(data&&idle_clip>=0&&data->animator.clip!=idle_clip)data->animator.play(idle_clip);
    }
};
}
