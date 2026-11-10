#pragma once
#include "epok.hpp"
#include "DungeonRulesBinding.hpp"

struct EPOK_VALUE(Configuration,Id="222eb084-c41e-586c-be67-abf45686b68e") DungeonVitalsTuning {
    uint32_t maximum=100;
    epok::Fixed hit_immunity=.7;
};

// Data and virtual dispatch only. Each project flavor owns eligibility, cooldown
// progression and the decisions that invoke the shared health arithmetic.
class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="60930832-058d-5a4d-a0ef-9fde884c7396") DungeonPlayerVitalsBase:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("60930832-058d-5a4d-a0ef-9fde884c7396");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,Id="b0a2404e-e3e7-52a5-8919-c19cac5d7f7f") DungeonVitalsTuning tuning{100,.7};
    EPOK_PROPERTY(Id="07e618de-b6f6-5e47-be5f-4fca7dc2ccd2") epok::HealthState health{100,100};
    EPOK_PROPERTY(Id="11f1a48e-8255-5744-8be7-76e25740544e") epok::Fixed immunity=0.0;
    EPOK_PROPERTY(Id="3a8ccaf6-40c8-5c6f-8425-7c0d1a2c13f4") bool valid=true;
    EPOK_FUNCTION(BlueprintCallable,Id="a600b3a7-1caa-52a9-ad7b-be1e3117e9b6") void assign(epok::HealthState value,epok::Fixed cooldown,bool available){health=value;immunity=cooldown;valid=available;}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="e6d01f77-d4dc-5d3e-99cb-4222aaa4918e") virtual void start()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="b1ce094f-f401-5d71-a766-51c677ba264f") virtual void advance(epok::Fixed dt)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="79e4e6eb-28c2-5117-8999-78a721c76e7d") virtual epok::HealthChange damage(int32_t amount,bool enabled)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="0d4253ba-23f6-502a-b103-2a879b7a4fca") virtual epok::HealthChange heal(int32_t amount)=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="31cfbe2e-070b-55dc-b6c7-6d723232f1ba") virtual bool can_heal()const=0;
};
using DungeonPlayerVitalsBinding=DungeonRulesBinding<DungeonPlayerVitalsBase>;
