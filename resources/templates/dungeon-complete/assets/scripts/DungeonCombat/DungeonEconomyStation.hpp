#pragma once
#include "DungeonEconomyStationBase.hpp"
class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D,Capability=interaction,Id="81e70d57-1589-555f-9a42-fc6ba0683807") DungeonEconomyStation:public DungeonEconomyStationBase {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("81e70d57-1589-555f-9a42-fc6ba0683807");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    bool can_interact(epok::ObjectId initiator)const override{auto* p=hud_value();if(!p||horizontal_range<=epok::Fixed(0.0)||height_range<=epok::Fixed(0.0))return false;const auto sample=measure(initiator);if(!sample.valid||sample.vertical<=-height_range||sample.vertical>=height_range)return false;const auto limit=squared_range(horizontal_range);if(sample.high>limit.high||(sample.high==limit.high&&sample.low>=limit.low))return false;return p->station_available(input_port);}
    void interaction_started(epok::ObjectId,uint32_t port)override{if(auto* p=hud_value())p->open_station(menu,port);}
    void focus_gained(epok::ObjectId,uint32_t)override{if(auto* p=hud_value())p->station_focus(menu,true);}
    void focus_lost(epok::ObjectId,uint32_t)override{if(auto* p=hud_value())p->station_focus(menu,false);}
};
