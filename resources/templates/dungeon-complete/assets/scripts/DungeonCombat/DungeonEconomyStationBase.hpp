#pragma once
#include "DungeonEconomyPresenterBase.hpp"
#include "interaction.hpp"
struct EPOK_VALUE(Id="e19ca84d-0df0-5522-9855-366f8d9ed90a") DungeonStationMeasure {bool valid=false;uint32_t high=0;uint32_t low=0;epok::Fixed vertical=0.0;};
struct EPOK_VALUE(Id="95b91d71-fea0-5c33-8918-ee364e8d2a74") DungeonStationRadius {uint32_t high=0;uint32_t low=0;};
class EPOK_CLASS(Abstract,Blueprintable,Domain=None,Owners=World3D,Capability=interaction,Id="8e028058-64d0-5d66-9de7-a8d4f2111cb6") DungeonEconomyStationBase:public epok::InteractableComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("8e028058-64d0-5d66-9de7-a8d4f2111cb6");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    DungeonEconomyStationBase(){range_3d=3.61;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="DungeonEconomyPresenterBase",Id="ae8eeba5-d25d-56c7-a9b1-a697bcb0f820") epok::ObjectId hud{};
    EPOK_PROPERTY(EditAnywhere,Id="2170fe4d-2238-5ca9-9f28-718170b49ace") uint32_t menu=1;
    EPOK_PROPERTY(EditAnywhere,Id="5e559a68-3649-5201-81c9-f1bf8b94050a") uint32_t input_port=0;
    EPOK_PROPERTY(EditAnywhere,Id="92a3a831-a897-5b24-98f8-fa1ee2643033") epok::Fixed horizontal_range=2.5;
    EPOK_PROPERTY(EditAnywhere,Id="31c36199-a44e-562f-8bcd-5682a9105193") epok::Fixed height_range=2.6;
    static uint64_t geometry_squared(int64_t x,int64_t z){const uint64_t a=x<0?uint64_t(-x):uint64_t(x),b=z<0?uint64_t(-z):uint64_t(z);const auto aa=a*a,bb=b*b;return UINT64_MAX-aa<bb?UINT64_MAX:aa+bb;}
    EPOK_FUNCTION(BlueprintPure,Id="1be607ea-d3aa-5a96-8594-2ef12bf26549") DungeonStationMeasure measure(epok::ObjectId initiator)const{const auto a=epok::gameplay_world_affine(epok::gameplay_actor_data(initiator));const auto b=epok::gameplay_world_affine(epok::gameplay_actor_data(owner_id()));if(!a.success||!b.success)return {};const auto d=geometry_squared(int64_t(a.position[0].raw())-b.position[0].raw(),int64_t(a.position[2].raw())-b.position[2].raw());return {true,uint32_t(d>>32),uint32_t(d),a.position[1]-b.position[1]};}
    EPOK_FUNCTION(BlueprintPure,Id="d28e04be-c63d-52d7-9733-81245b23a371") DungeonStationRadius squared_range(epok::Fixed radius)const{const auto d=geometry_squared(radius.raw(),0);return {uint32_t(d>>32),uint32_t(d)};}
    DungeonEconomyPresenterBase* hud_value()const{auto* p=epok::active_object_registry?epok::active_object_registry->resolve<DungeonEconomyPresenterBase>(hud):nullptr;auto* owner=const_cast<DungeonEconomyStationBase*>(this)->get_owner();return p&&owner&&p->level_id()==owner->level_id()?p:nullptr;}
};
