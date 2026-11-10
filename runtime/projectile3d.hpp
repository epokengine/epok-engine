#pragma once
#include "gameplay_api.hpp"

namespace epok {
struct EPOK_VALUE(Id="c29551ed-c81b-5554-84de-2eb49aa78d3c") ProjectileHandle {
    ObjectId pool{};
    uint32_t ticket=0;
};
struct EPOK_VALUE(Id="7e690007-eedb-50fe-8f32-06ff841e25de") ProjectileSpawn3D {
    GameplayVector3 origin{},velocity{};
    Fixed lifetime=0.0;
    uint32_t tag=0;
};
struct EPOK_VALUE(Id="5bb6b14b-ef0a-5a80-ae29-455621e85d7b") ProjectileTraceFilter {
    uint32_t mask=0xffffffffu;
    ObjectId ignore{};
    bool triggers=false;
};
struct EPOK_VALUE(Configuration,Id="e9f15129-c183-539e-aaff-70e10c61b637") ProjectileSpriteGrid {
    bool enabled=false;
    uint32_t frames=4;
    uint32_t ticks_per_frame=2;
    uint32_t cell_width=32;
    uint32_t cell_height=32;
    uint32_t start_x=0;
    uint32_t start_y=0;
};
enum class ProjectileResult:uint8_t {
    Applied,Unchanged,Full,InvalidOwner,InvalidRange,InvalidBinding,InvalidSpawn,InvalidHandle,InvalidGrid,Busy
};
enum class ProjectileStepState:uint8_t {
    Empty,Moved,Hit,Expired,InvalidOwner,InvalidRange,InvalidDelta,InvalidBinding,InvalidContext,InvalidFilter,InvalidGrid,InvalidBatch
};
// Eight wire words per result; a four-shot batch fits the existing 32-word ABI.
struct EPOK_VALUE(Id="6f041d29-dbfb-5c21-8291-404e5db7c6b9") ProjectileStep {
    ProjectileStepState state=ProjectileStepState::Empty;
    ProjectileHandle handle{};
    ObjectId target{};
    GameplayVector3 point{};
    uint32_t tag=0;
};
struct ProjectileStep4 {
    ProjectileStep first{},second{},third{},fourth{};
};
// The script ABI carries a small checked cursor. Read results before the next
// step on this pool. A gameplay reset preserves this immutable impact batch.
struct EPOK_VALUE(Id="ce293ffe-76d7-57b9-955f-37510934146f") ProjectileBatch {
    ObjectId pool{};
    uint32_t sequence=0,count=0;
    ProjectileStepState state=ProjectileStepState::Empty;
};
struct EPOK_VALUE(Id="e35cb5d6-7baa-5c35-b5d5-d8e04ea25661") ProjectileLaunchResult {
    ProjectileResult result=ProjectileResult::InvalidOwner;
    ProjectileHandle handle{};
    ObjectId actor{};
};
struct EPOK_VALUE(Id="03875a8f-40a0-549f-9519-3429e036095a") ProjectileSnapshot {
    bool valid=false,active=false;
    ObjectId actor{};
    GameplayVector3 position{},velocity{};
    Fixed lifetime=0.0;
};

// Bounded motion over ordinary authored actors. Projects own firing, targeting,
// teams, damage and impact effects. There is no global pool or automatic tick.
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Capability=world-gameplay,Id="99c8c724-d8ee-5697-9d7f-927bfb9adc7e") ProjectilePool3DComponent:public ActorComponent {
    struct Slot {
        ObjectId actor{},sprite{};
        DataHandle data{};
        GameplayVector3 velocity{};
        Fixed life=0.0;
        uint32_t generation=0,age=0,tag=0;
        bool active=false,managed=false;
    } slots_[8]{};
    ObjectRegistry* registry_=nullptr;
    ProjectileStep4 batch_{};
    uint32_t batch_sequence_=0,batch_count_=0;
    ObjectId& visual(uint32_t slot){
        switch(slot){case 0:return visual0;case 1:return visual1;case 2:return visual2;case 3:return visual3;
        case 4:return visual4;case 5:return visual5;case 6:return visual6;default:return visual7;}
    }
    const ObjectId& visual(uint32_t slot)const{return const_cast<ProjectilePool3DComponent*>(this)->visual(slot);}
    Actor* available_owner()const{
        auto* self=const_cast<ProjectilePool3DComponent*>(this);auto* owner=self->get_owner();
        return active_object_registry&&active_object_registry->get(id())==this
            &&!m_ended&&is_active(self)&&owner&&!owner->pending_destruction()?owner:nullptr;
    }
    Actor3D* actor(ObjectId value,Actor& owner)const{
        auto* result=active_object_registry->resolve<Actor3D>(value);
        return result&&result->level_id()==owner.level_id()&&!result->pending_destruction()?result:nullptr;
    }
    static void invalidate(Slot& slot){slot.generation=(slot.generation+1)&0x1fffffffu;if(!slot.generation)slot.generation=1;}
    ProjectileHandle token(uint32_t index)const{return {id(),(slots_[index].generation<<3)|index};}
    bool matches(ProjectileHandle value)const{
        return value.pool==id()&&value.ticket>=8&&slots_[value.ticket&7].generation==(value.ticket>>3);
    }
    bool grid_valid()const{
        return !sprite_grid.enabled||(sprite_grid.frames&&sprite_grid.ticks_per_frame&&sprite_grid.cell_width&&sprite_grid.cell_height
            &&uint64_t(sprite_grid.start_x)+uint64_t(sprite_grid.frames)*sprite_grid.cell_width<=65535
            &&uint64_t(sprite_grid.start_y)+sprite_grid.cell_height<=65535);
    }
    ActorData* capture(uint32_t index,Actor& owner){
        auto* bound=actor(visual(index),owner);auto* data=bound?bound->data():nullptr;
        if(!data||!is_active(bound)||data->parent>=0||data->collider.enabled)return nullptr;
        Sprite3DComponent* sprite=nullptr;
        if(manage_sprite_visibility||sprite_grid.enabled){
            auto* level=active_object_registry->resolve<Level>(owner.level_id());
            sprite=level?level->get_component<Sprite3DComponent>(*bound):nullptr;
            if(!sprite||!is_active(sprite))return nullptr;
        }
        auto& slot=slots_[index];slot.actor=bound->id();slot.data=handle(data);
        slot.sprite=sprite?sprite->id():ObjectId{};slot.managed=manage_sprite_visibility;return data;
    }
    ActorData* live(uint32_t index,Actor& owner)const{
        auto& slot=slots_[index];
        if(registry_!=active_object_registry||slot.actor!=visual(index))return nullptr;
        // capture() proved both classes, and an id cannot outlive its generation.
        auto* bound=static_cast<Actor3D*>(active_object_registry->get(slot.actor));auto* data=slot.data.get();
        if(!bound||bound->level_id()!=owner.level_id()||bound->pending_destruction())return nullptr;
        if(!data||bound->data()!=data||!is_active(bound)||data->parent>=0||data->collider.enabled)return nullptr;
        if(slot.managed||sprite_grid.enabled){
            auto* sprite=static_cast<Sprite3DComponent*>(active_object_registry->get(slot.sprite));
            if(!sprite||!is_active(sprite)||sprite->get_owner()!=bound)return nullptr;
        }
        return data;
    }
    void stop_slot(Slot& slot){
        if(slot.managed&&registry_&&registry_==active_object_registry){
            auto* bound=active_object_registry->resolve<Actor3D>(slot.actor);
            auto* sprite=active_object_registry->resolve<Sprite3DComponent>(slot.sprite);
            auto* data=slot.data.get();
            if(bound&&data&&bound->data()==data&&sprite&&sprite->get_owner()==bound)data->sprite.enabled=false;
        }
        slot.active=false;slot.life=0.0;
    }
    void animate(Slot& slot,ActorData& data){
        if(sprite_grid.enabled){
            data.sprite.region[0]=uint16_t(sprite_grid.start_x+((slot.age/sprite_grid.ticks_per_frame)%sprite_grid.frames)*sprite_grid.cell_width);
            data.sprite.region[1]=uint16_t(sprite_grid.start_y);data.sprite.region[2]=uint16_t(sprite_grid.cell_width);data.sprite.region[3]=uint16_t(sprite_grid.cell_height);
        }
        if(slot.age!=UINT32_MAX)++slot.age;
    }
    static ProjectileStep4 error(ProjectileStepState state){ProjectileStep value;value.state=state;return {value,value,value,value};}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("99c8c724-d8ee-5697-9d7f-927bfb9adc7e");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual0{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual1{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual2{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual3{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual4{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual5{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual6{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") ObjectId visual7{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpatialQueryContextComponent") ObjectId query_context{};
    EPOK_PROPERTY(EditAnywhere) bool manage_sprite_visibility=true;
    EPOK_PROPERTY(EditAnywhere) ProjectileSpriteGrid sprite_grid{false,4,2,32,32,0,0};
    EPOK_FUNCTION(BlueprintPure) ObjectId binding(uint32_t slot)const{return slot<8?visual(slot):ObjectId{};}
    EPOK_FUNCTION(BlueprintPure) uint32_t active_count()const{uint32_t count=0;for(const auto& slot:slots_)if(slot.active)++count;return count;}
    // Native callers can skip a range that has nothing to advance.
    bool active_in(uint32_t first,uint32_t count)const{for(uint32_t i=first;i<first+count&&i<8;++i)if(slots_[i].active)return true;return false;}
    EPOK_FUNCTION(BlueprintCallable) ProjectileResult bind(uint32_t index,ObjectId value){
        auto* owner=available_owner();if(!owner)return ProjectileResult::InvalidOwner;
        if(index>=8)return ProjectileResult::InvalidRange;
        if(slots_[index].active)return ProjectileResult::Busy;
        if(value.valid()){
            if(!actor(value,*owner))return ProjectileResult::InvalidBinding;
            for(uint32_t i=0;i<8;++i)if(i!=index&&visual(i)==value)return ProjectileResult::InvalidBinding;
        }
        if(visual(index)==value)return ProjectileResult::Unchanged;
        visual(index)=value;invalidate(slots_[index]);return ProjectileResult::Applied;
    }
    EPOK_FUNCTION(BlueprintCallable) ProjectileLaunchResult launch(uint32_t first,uint32_t count,ProjectileSpawn3D spawn){
        auto* owner=available_owner();if(!owner)return {};
        if(first>=8||!count||count>8-first)return {ProjectileResult::InvalidRange,{},{}};
        if(spawn.lifetime.raw()<=0)return {ProjectileResult::InvalidSpawn,{},{}};
        if(!grid_valid())return {ProjectileResult::InvalidGrid,{},{}};
        if(registry_&&registry_!=active_object_registry)return {ProjectileResult::InvalidBinding,{},{}};
        registry_=active_object_registry;
        bool invalid=false;
        for(uint32_t index=first;index<first+count;++index){
            auto& slot=slots_[index];if(slot.active)continue;
            bool duplicate=false;for(uint32_t i=0;i<8;++i)if(i!=index&&visual(index).valid()&&visual(i)==visual(index))duplicate=true;
            auto* data=duplicate?nullptr:capture(index,*owner);if(!data){invalid=true;continue;}
            invalidate(slot);slot.velocity=spawn.velocity;slot.life=spawn.lifetime;slot.tag=spawn.tag;slot.age=0;slot.active=true;
            gameplay_vector(spawn.origin,data->transform.position);
            if(sprite_grid.enabled){data->sprite.region[0]=uint16_t(sprite_grid.start_x);data->sprite.region[1]=uint16_t(sprite_grid.start_y);data->sprite.region[2]=uint16_t(sprite_grid.cell_width);data->sprite.region[3]=uint16_t(sprite_grid.cell_height);}
            if(slot.managed)data->sprite.enabled=true;
            return {ProjectileResult::Applied,token(index),slot.actor};
        }
        return {invalid?ProjectileResult::InvalidBinding:ProjectileResult::Full,{},{}};
    }
    // Native callers may reuse a synchronized context. Script providers use step().
    ProjectileStep4 step_shared(uint32_t first,uint32_t count,Fixed dt,ProjectileTraceFilter filter,const SpatialQueryBatch& queries){
        auto* owner=available_owner();if(!owner)return error(ProjectileStepState::InvalidOwner);
        if(first>=8||!count||count>4||count>8-first)return error(ProjectileStepState::InvalidRange);
        if(dt.raw()<=0||dt.raw()>4096)return error(ProjectileStepState::InvalidDelta);
        if(!grid_valid())return error(ProjectileStepState::InvalidGrid);
        auto* ignored=filter.ignore.valid()?actor(filter.ignore,*owner):nullptr;
        if(filter.ignore.valid()&&(!ignored||!ignored->data()))return error(ProjectileStepState::InvalidFilter);
        ProjectileStep4 result;ProjectileStep* output[4]={&result.first,&result.second,&result.third,&result.fourth};
        RaycastQuery rays[4]{};SpatialHit hits[4]{};ActorData* data[4]{};uint32_t indices[4]{},ray_count=0;
        for(uint32_t offset=0;offset<count;++offset){
            const auto index=first+offset;auto& slot=slots_[index];if(!slot.active)continue;
            auto& item=*output[offset];item.handle=token(index);item.tag=slot.tag;data[offset]=live(index,*owner);
            if(!data[offset]){item.state=ProjectileStepState::InvalidBinding;stop_slot(slot);continue;}
            auto& ray=rays[ray_count];gameplay_vector(gameplay_vector(data[offset]->transform.position),ray.origin);
            Fixed velocity[3];gameplay_vector(slot.velocity,velocity);bool valid=true;
            for(uint32_t axis=0;axis<3;++axis){
                const int64_t delta=(int64_t(velocity[axis].raw())*dt.raw())>>12;
                const int64_t end=int64_t(ray.origin[axis].raw())+delta;
                if(delta<INT32_MIN||delta>INT32_MAX||end<INT32_MIN||end>INT32_MAX)valid=false;
                else ray.displacement[axis]=Fixed(int32_t(delta),Fixed::RAW);
            }
            if(!valid){item.state=ProjectileStepState::InvalidDelta;continue;}
            indices[ray_count++]=offset;
        }
        if(ray_count)queries.raycast_batch(rays,hits,ray_count,filter.mask,ignored?ignored->data():nullptr,filter.triggers);
        for(uint32_t n=0;n<ray_count;++n){
            const auto offset=indices[n];auto& slot=slots_[first+offset];auto& item=*output[offset];const auto& hit=hits[n];auto& target=*data[offset];
            if(hit&&hit_entity(hit).get()){
                item.state=ProjectileStepState::Hit;item.target=gameplay_actor_id(hit_entity(hit));item.point=gameplay_vector(hit.point);
                animate(slot,target);stop_slot(slot);
            }else{
                for(uint32_t axis=0;axis<3;++axis)target.transform.position[axis]+=rays[n].displacement[axis];
                slot.life-=dt;item.point=gameplay_vector(target.transform.position);animate(slot,target);
                item.state=slot.life.raw()>0?ProjectileStepState::Moved:ProjectileStepState::Expired;
                if(slot.life.raw()<=0)stop_slot(slot);
            }
        }
        return result;
    }
    EPOK_FUNCTION(BlueprintCallable) ProjectileBatch step(uint32_t first,uint32_t count,Fixed dt,ProjectileTraceFilter filter){
        auto* owner=available_owner();if(!owner)return {{},0,0,ProjectileStepState::InvalidOwner};
        auto* context=query_context.valid()?active_object_registry->resolve<SpatialQueryContextComponent>(query_context):nullptr;
        if(!query_context.valid())if(auto* level=active_object_registry->resolve<Level>(owner->level_id()))context=level->get_component<SpatialQueryContextComponent>(*owner);
        auto* source=context?context->get_owner():nullptr;
        auto* queries=source&&source->level_id()==owner->level_id()&&is_active(context)?context->live_batch():nullptr;
        batch_=queries?step_shared(first,count,dt,filter,*queries):error(ProjectileStepState::InvalidContext);
        if(!++batch_sequence_)++batch_sequence_;batch_count_=count<=4?count:4;
        return {id(),batch_sequence_,batch_count_,batch_.first.state};
    }
    EPOK_FUNCTION(BlueprintPure) ProjectileStep result(ProjectileBatch batch,uint32_t index)const{
        if(batch.pool!=id()||batch.sequence!=batch_sequence_||!batch.sequence||index>=batch_count_||index>=batch.count){
            ProjectileStep item;item.state=ProjectileStepState::InvalidBatch;return item;
        }
        switch(index){case 0:return batch_.first;case 1:return batch_.second;case 2:return batch_.third;default:return batch_.fourth;}
    }
    EPOK_FUNCTION(BlueprintCallable) ProjectileResult stop(ProjectileHandle value){
        if(!available_owner())return ProjectileResult::InvalidOwner;
        if(!matches(value))return ProjectileResult::InvalidHandle;
        auto& slot=slots_[value.ticket&7];if(!slot.active)return ProjectileResult::Unchanged;
        stop_slot(slot);return ProjectileResult::Applied;
    }
    EPOK_FUNCTION(BlueprintCallable) void reset(){for(auto& slot:slots_){stop_slot(slot);invalidate(slot);}}
    EPOK_FUNCTION(BlueprintPure) ProjectileSnapshot snapshot(ProjectileHandle value)const{
        auto* owner=available_owner();if(!owner||!matches(value))return {};
        const auto index=value.ticket&7;auto* data=live(index,*owner);if(!data)return {};
        const auto& slot=slots_[index];return {true,slot.active,slot.actor,gameplay_vector(data->transform.position),slot.velocity,slot.life};
    }
    void begin_play()override{reset();registry_=active_object_registry;}
    void on_disable()override{reset();}
    void end_play(EndPlayReason)override{reset();batch_sequence_=batch_count_=0;registry_=nullptr;}
};
struct EPOK_FUNCTION_LIBRARY(Category="Projectiles",Capability=world-gameplay,Id="52135312-ea59-5025-89fe-2e449be6f34b") ProjectileLibrary {
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="f2e2aee3-6cad-5a01-8799-030df58f4dd4") static ProjectileSpawn3D spawn(GameplayVector3 origin,GameplayVector3 velocity,Fixed lifetime,uint32_t tag){return {origin,velocity,lifetime,tag};}
    EPOK_FUNCTION(BlueprintPure,PureValue,Id="15421ae5-7a20-53a0-bf77-6de10a74dbdc") static ProjectileTraceFilter filter(uint32_t mask,ObjectId ignore,bool triggers){return {mask,ignore,triggers};}
};
}
