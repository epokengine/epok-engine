#pragma once
#include "epok.hpp"
class EPOK_CLASS(Abstract,Blueprintable,Domain=UI,Id="2d29ee04-9738-5493-beff-dc81ada513e3") DungeonEconomyPresenterEndpoint:public epok::UIActor {public:static constexpr uint64_t static_class_id=epok::detail::compact_class_id("2d29ee04-9738-5493-beff-dc81ada513e3");uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="f5a64097-edf0-50d1-bfa0-b7e784644969") virtual bool widget_ready(epok::ObjectId candidate)=0;};
