#pragma once
#include "camera_rig.hpp"

// Project-owned view proposal. The player rules choose orbit, aim and shoulder
// offsets; this class extends the common rig's normal calculation/application API.
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="a86b95c7-5021-5bd2-a87c-fc666a9ee174") DungeonCamera:public epok::ThirdPersonCamera {
    epok::GameplayVector3 origin_{},offset_{};
    uint32_t collision_revision_=0;
    bool configured_=false;
    static bool same(const epok::GameplayVector3& a,const epok::GameplayVector3& b){return a.x==b.x&&a.y==b.y&&a.z==b.z;}
public:
    static constexpr epok::TickGroup tick_group=epok::TickGroup::Gameplay;
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("a86b95c7-5021-5bd2-a87c-fc666a9ee174");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpatialQueryContextComponent",Id="927b9eb2-7dd7-56d4-9af5-58ebe1117cc2") epok::ObjectId query_context{};
    EPOK_FUNCTION(BlueprintCallable,Id="74c00a47-8d41-5df2-8423-a85ade3cf3ad") epok::CameraRigSample place(epok::GameplayVector3 origin,epok::GameplayVector3 offset,epok::Fixed view_yaw,epok::Fixed view_pitch,epok::Fixed requested_distance,uint32_t collision_revision){
        // Nothing moved, so the solved view is the last one unless the boom is
        // still easing back out.
        if(configured_&&collision_revision==collision_revision_&&boom.extension_response.raw()==0&&view_yaw==yaw&&view_pitch==pitch&&requested_distance==distance
            &&same(origin,origin_)&&same(offset,offset_)&&sample().status==epok::CameraRigStatus::Applied)return sample();
        origin_=origin;offset_=offset;yaw=view_yaw;pitch=view_pitch;distance=requested_distance;collision_revision_=collision_revision;configured_=true;
        return refresh(epok::Fixed(1.0/60.0));
    }
    EPOK_FUNCTION(BlueprintEvent) epok::CameraRigSample calculate_view(epok::Fixed delta_seconds)override{
        if(!configured_)return epok::ThirdPersonCamera::calculate_view(delta_seconds);
        epok::CameraRigSample result;result.yaw=yaw;result.pitch=pitch;result.distance=distance;
        const auto solved=boom.solve_shared(origin_,offset_,target,delta_seconds,query_context,collision_revision_);
        result.status=solved.status;result.position=solved.position;result.blocked=solved.blocked;return result;
    }
};
