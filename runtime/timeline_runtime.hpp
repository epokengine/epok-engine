#pragma once
#include "timeline.hpp"
#include "blueprint_runtime.hpp"
#include "actor_blueprint.hpp"

// The cooked asset owns immutable data. Instances own only bounded playback
// state; callbacks are generated typed adapters, never offsets into C++ objects.
namespace epok::timeline {
inline constexpr size_t slot_limit=8,track_limit=16,marker_limit=64,signal_limit=128;
// The effect service supplies one typed generation validator. There is no
// reflection lookup, allocation, or scene-entity surrogate in this binding.
inline EffectLayer* (*layer_resolver)(EffectLayerHandle)=nullptr;
struct BoundTarget {
    union {ObjectId object;EffectLayerHandle layer;};
    bool internal=false;
    BoundTarget():object{}{}
    BoundTarget(ObjectId value):object(value){}
    BoundTarget(DataHandle value):object(value.get()&&value.get()->owner?value.get()->owner->id():ObjectId{}){}
    BoundTarget(EffectLayerHandle value):layer(value),internal(true){}
    EffectLayer* effect_layer()const{return internal&&layer_resolver?layer_resolver(layer):nullptr;}
    Object* get()const{return internal?nullptr:object.get();}
    ActorData* data()const{return internal?nullptr:bp::object_data(object);}
    DataHandle data_slot()const{return internal?DataHandle{}:bp::data_handle(object);}
    operator ObjectId()const{return internal?ObjectId{}:object;}
    bool valid()const{return internal?effect_layer()!=nullptr:get()!=nullptr;}
    bool active()const{if(internal){const auto* value=effect_layer();return value&&value->runtime_active;}return is_active(get());}
    bool visible()const{if(internal){const auto* value=effect_layer();return value&&value->runtime_visible;}return is_active(get());}
    bool same(const BoundTarget& other)const{
        if(internal!=other.internal)return false;
        return internal?layer.index==other.layer.index&&layer.generation==other.layer.generation:object==other.object;
    }
    bool same_sink(const BoundTarget& other)const{
        if(internal!=other.internal)return false;
        if(internal)return same(other);
        auto* a=data();auto* b=other.data();
        return a&&b?a==b:object==other.object;
    }
};
struct Value { int32_t lanes[4]={}; };
struct Target { bool required; bool (*accepts)(BoundTarget); };
struct Property {
    uint64_t id;
    uint16_t slot;
    uint8_t channels;
    bool additive,restore;
    const Curve* curves;
    bool (*read)(BoundTarget,Value&);
    bool (*write)(BoundTarget,const Value&);
    int32_t start=0,end=INT32_MAX,offset=0,numerator=1,denominator=1;
    bool active(int32_t tick,int32_t duration)const{return tick>=start&&(tick<end||(tick==duration&&end==duration));}
    int32_t source_tick(int32_t tick)const{return saturate(int64_t(offset)+(int64_t(tick)-start)*numerator/denominator);}
};
struct Event {
    uint16_t slot;
    uint8_t count;
    bool idempotent;
    const Argument* arguments;
    bool (*invoke)(BoundTarget,const BoundTarget*,const Argument*);
};
struct Asset {
    uint64_t id;
    int32_t duration;
    bool repeat;
    uint16_t target_count,property_count,event_count,marker_count,signal_count;
    const Target* targets;
    const Property* properties;
    const Event* events;
    const uint64_t* markers;
    const Signal* signals;
};
enum class State:uint8_t {Invalid,Playing,Completed,Cancelled};
enum class Reason:uint8_t {InvalidTarget,SkippedEvent,Capacity,Conflict,ClampedTime};
struct Diagnostic {uint64_t asset=0;uint16_t item=0;Reason reason=Reason::InvalidTarget;};
struct Stats {
    uint32_t active=0,peak=0,dropped=0,conflicts=0,skipped_targets=0,
             skipped_events=0,properties=0,events=0,markers=0,completed=0,
             cancelled=0,clamped_ticks=0,diagnostics_dropped=0;
};

template<size_t Capacity=8> class Director {
    static_assert(Capacity>0&&Capacity<=8);
    struct Instance {
        const Asset* asset=nullptr;
        BoundTarget owner,targets[slot_limit];
        Value initial[track_limit];
        bool captured[track_limit]={};
        uint32_t markers[marker_limit]={};
        uint32_t generation=0,scene=0;
        int32_t tick=0;
        uint16_t cursor=0;
        State state=State::Invalid;
        bool paused=false,busy=false,backwards=false;
    } instances[Capacity];
    Diagnostic diagnostics[32];
    uint16_t diagnostic_begin=0,diagnostic_count=0;
    bool cancelling=false,advancing=false;
    static void add(uint32_t& value,uint32_t amount=1){value=UINT32_MAX-value<amount?UINT32_MAX:value+amount;}
    Instance* find(Handle h){return h.generation&&h.index<Capacity&&instances[h.index].generation==h.generation?&instances[h.index]:nullptr;}
    const Instance* find(Handle h)const{return h.generation&&h.index<Capacity&&instances[h.index].generation==h.generation?&instances[h.index]:nullptr;}
    void report(const Asset* asset,uint16_t item,Reason reason){
        if(diagnostic_count==32){add(stats.diagnostics_dropped);return;}
        diagnostics[(diagnostic_begin+diagnostic_count++)%32]={asset?asset->id:0,item,reason};
    }
    bool target(const Instance& instance,uint16_t slot)const{
        if(slot>=instance.asset->target_count)return false;
        const auto owner=instance.targets[slot];
        const auto accepts=instance.asset->targets[slot].accepts;
        return owner.valid()&&owner.active()&&accepts&&accepts(owner);
    }
    static bool same_property(const Instance& instance,size_t a,size_t b){
        const auto& p=instance.asset->properties[a];const auto& q=instance.asset->properties[b];
        return p.id==q.id&&instance.targets[p.slot].same_sink(instance.targets[q.slot]);
    }
    void sample_properties(Instance& instance){
        const auto& asset=*instance.asset;
        bool valid[track_limit]={};
        // Capture every initial value before making the first write. Aliased
        // tracks therefore share the same baseline and never accumulate drift.
        for(uint16_t i=0;i<asset.property_count;++i){
            const auto& property=asset.properties[i];
            valid[i]=target(instance,property.slot);
            if(valid[i]&&!instance.captured[i])instance.captured[i]=property.read&&property.read(instance.targets[property.slot],instance.initial[i]);
            valid[i]=valid[i]&&instance.captured[i];
            if(!valid[i]){add(stats.skipped_targets);report(&asset,i,Reason::InvalidTarget);}
        }
        for(uint16_t i=0;i<asset.property_count;++i){
            if(!valid[i])continue;
            bool earlier=false;for(uint16_t j=0;j<i;++j)earlier|=valid[j]&&same_property(instance,i,j);
            if(earlier)continue;
            bool contributes=false;
            for(uint16_t j=i;j<asset.property_count;++j)
                contributes|=valid[j]&&same_property(instance,i,j)&&asset.properties[j].active(instance.tick,asset.duration);
            if(!contributes)continue;
            Value value=instance.initial[i];
            for(uint16_t j=i;j<asset.property_count;++j){
                if(!valid[j]||!same_property(instance,i,j))continue;
                const auto& property=asset.properties[j];
                if(!property.active(instance.tick,asset.duration))continue;
                for(uint8_t lane=0;lane<property.channels;++lane){
                    const auto sampled=sample(property.curves[lane],property.source_tick(instance.tick));
                    if(!property.additive)value.lanes[lane]=sampled;
                    else if(property.curves[lane].unsigned_values)value.lanes[lane]=int32_t(bp::uadd(uint32_t(value.lanes[lane]),uint32_t(sampled)));
                    else value.lanes[lane]=saturate(int64_t(value.lanes[lane])+sampled);
                }
                add(stats.properties);
            }
            const auto& property=asset.properties[i];
            if(!property.write||!property.write(instance.targets[property.slot],value)){
                add(stats.skipped_targets);report(&asset,i,Reason::InvalidTarget);
            }
        }
    }
    void finish(Instance& instance,State state){
        if(instance.state!=State::Playing)return;
        instance.state=state;--stats.active;
        add(state==State::Completed?stats.completed:stats.cancelled);
        const bool was_busy=instance.busy;instance.busy=true;
        if(state==State::Completed)for(uint16_t i=0;i<instance.asset->property_count;++i){
            const auto& p=instance.asset->properties[i];
            if(!p.restore||!instance.captured[i])continue;
            bool earlier=false;for(uint16_t j=0;j<i;++j)earlier|=instance.captured[j]&&same_property(instance,i,j);
            if(earlier)continue;
            if(!target(instance,p.slot)||!p.write||!p.write(instance.targets[p.slot],instance.initial[i])){
                add(stats.skipped_targets);report(instance.asset,i,Reason::InvalidTarget);
            }
        }
        instance.busy=was_busy;
    }
    bool alive(Instance& instance,uint32_t scene){
        if(instance.state!=State::Playing)return false;
        if(!instance.owner.valid()||instance.scene!=scene){finish(instance,State::Cancelled);return false;}
        return instance.owner.active()&&!instance.paused;
    }
    void invoke(Instance& instance,uint16_t index){
        const auto& event=instance.asset->events[index];
        bool valid=target(instance,event.slot)&&event.invoke;
        for(uint8_t i=0;i<event.count;++i){
            const int slot=event.arguments[i].slot;
            if(slot>=0)valid=valid&&target(instance,uint16_t(slot));
        }
        if(valid)valid=event.invoke(instance.targets[event.slot],instance.targets,event.arguments);
        if(valid)add(stats.events);
        else{add(stats.skipped_events);report(instance.asset,index,Reason::SkippedEvent);}
    }
public:
    Stats stats;
    void capacity_drop(const Asset& asset){add(stats.dropped);report(&asset,0,Reason::Capacity);}
    bool poll_diagnostic(Diagnostic& result){
        if(!diagnostic_count)return false;
        result=diagnostics[diagnostic_begin];diagnostic_begin=(diagnostic_begin+1)%32;--diagnostic_count;return true;
    }
    State state(Handle h)const{auto* instance=find(h);return instance?instance->state:State::Invalid;}
    bp::PlaybackSnapshot snapshot(Handle h,uint64_t asset=0,uint64_t marker=0)const{
        const auto* value=find(h);
        if(!value||!value->asset||(asset&&value->asset->id!=asset))return {};
        uint32_t revision=0;
        if(marker){
            bool found=false;
            for(uint16_t i=0;i<value->asset->marker_count;++i)if(value->asset->markers[i]==marker){revision=value->markers[i];found=true;break;}
            if(!found)return {};
        }
        const auto result=value->state==State::Playing?bp::PlaybackResult::Pending:
            value->state==State::Completed?bp::PlaybackResult::Completed:bp::PlaybackResult::Cancelled;
        return {result,revision};
    }
    int32_t tick(Handle h)const{auto* instance=find(h);return instance?instance->tick:0;}
    uint32_t marker(Handle h,uint16_t index)const{
        auto* instance=find(h);return instance&&index<instance->asset->marker_count?instance->markers[index]:0;
    }
    uint32_t marker_revision(Handle h,uint64_t id)const{
        const auto* instance=find(h);if(!instance||!instance->asset->markers)return 0;
        for(uint16_t i=0;i<instance->asset->marker_count;++i)if(instance->asset->markers[i]==id)return instance->markers[i];
        return 0;
    }
    Handle play(const Asset& asset,DataHandle owner,const DataHandle* targets,uint32_t scene){
        if(asset.target_count>slot_limit){capacity_drop(asset);return {};}
        BoundTarget bindings[slot_limit];
        if(targets)for(uint16_t i=0;i<asset.target_count;++i)bindings[i]=targets[i];
        return play(asset,BoundTarget(owner),targets?bindings:nullptr,scene);
    }
    Handle play(const Asset& asset,BoundTarget owner,const BoundTarget* targets,uint32_t scene){
        if(cancelling||!owner.valid()||asset.duration<=0||asset.target_count>slot_limit||asset.property_count>track_limit||asset.marker_count>marker_limit||asset.event_count>64||asset.signal_count>signal_limit||
           (asset.target_count&&(!asset.targets||!targets))||(asset.property_count&&!asset.properties)||(asset.event_count&&!asset.events)||(asset.signal_count&&!asset.signals)){
            add(stats.dropped);return {};
        }
        for(uint16_t i=0;i<asset.target_count;++i)if(asset.targets[i].required&&(!targets[i].valid()||!asset.targets[i].accepts||!asset.targets[i].accepts(targets[i]))){add(stats.dropped);report(&asset,i,Reason::InvalidTarget);return {};}
        for(uint16_t i=0;i<asset.property_count;++i){
            const auto& p=asset.properties[i];if(p.slot>=asset.target_count||!p.channels||p.channels>3||!p.curves||p.start<0||p.end<=p.start||p.offset<0||p.numerator<1||p.denominator<1||p.numerator>1024||p.denominator>1024){add(stats.dropped);return {};}
            for(uint8_t c=0;c<p.channels;++c)if(!p.curves[c].keys||!p.curves[c].count||p.curves[c].count>key_limit){add(stats.dropped);return {};}
        }
        for(uint16_t i=0;i<asset.event_count;++i){const auto& e=asset.events[i];if(e.slot>=asset.target_count||e.count>4||(e.count&&!e.arguments)){add(stats.dropped);return {};}}
        for(uint16_t i=0;i<asset.signal_count;++i){const auto& s=asset.signals[i];if(s.index>=(s.event?asset.event_count:asset.marker_count)||s.tick<0||s.tick>asset.duration||(i&&s.tick<asset.signals[i-1].tick)){add(stats.dropped);return {};}}
        // Cross-instance writes are exclusive. Within an asset, validated
        // priority/blend order combines tracks. A second owner cannot restore
        // a snapshot over another active sequence's output.
        for(const auto& instance:instances)if(instance.state==State::Playing){
            for(uint16_t i=0;i<asset.property_count;++i)for(uint16_t j=0;j<instance.asset->property_count;++j){
                const auto& a=asset.properties[i];const auto& b=instance.asset->properties[j];
                if(a.id==b.id&&targets[a.slot].valid()&&targets[a.slot].same_sink(instance.targets[b.slot])){
                    add(stats.dropped);add(stats.conflicts);report(&asset,i,Reason::Conflict);return {};
                }
            }
        }
        for(uint16_t i=0;i<Capacity;++i){auto& instance=instances[i];if(instance.state==State::Playing||instance.busy)continue;
            if(instance.generation)bp::observe_playback();
            uint32_t generation=instance.generation+1;if(!generation)generation=1;
            instance={};instance.generation=generation;instance.asset=&asset;instance.owner=owner;instance.scene=scene;instance.state=State::Playing;
            for(uint16_t slot=0;slot<asset.target_count;++slot)instance.targets[slot]=targets[slot];
            ++stats.active;if(stats.active>stats.peak)stats.peak=stats.active;return {i,generation};
        }
        add(stats.dropped);report(&asset,0,Reason::Capacity);return {};
    }
    bool stop(Handle h){auto* instance=find(h);if(!instance||instance->state!=State::Playing)return false;finish(*instance,State::Cancelled);return true;}
    bool pause(Handle h,bool paused){auto* instance=find(h);if(!instance||instance->state!=State::Playing)return false;instance->paused=paused;return true;}
    bool reverse(Handle h,bool backwards){
        auto* instance=find(h);if(!instance||instance->state!=State::Playing||instance->busy)return false;
        instance->backwards=backwards;
        if(backwards&&instance->tick==0){instance->tick=instance->asset->duration;instance->cursor=instance->asset->signal_count;sample_properties(*instance);}
        return true;
    }
    void cancel_all(){const bool previous=cancelling;cancelling=true;for(auto& instance:instances)finish(instance,State::Cancelled);cancelling=previous;}
    void cancel_owner(BoundTarget owner){for(auto& instance:instances)if(instance.owner.same(owner))finish(instance,State::Cancelled);}
    bool seek(Handle h,int32_t tick,uint32_t scene){
        auto* instance=find(h);
        if(!instance||instance->busy||instance->state!=State::Playing)return false;
        if(!instance->owner.valid()||instance->scene!=scene){finish(*instance,State::Cancelled);return false;}
        // Scrubbing is an absolute sample, including while paused. An idempotent
        // action is not necessarily reversible: seeking never dispatches signals.
        instance->busy=true;instance->tick=tick<0?0:tick>instance->asset->duration?instance->asset->duration:tick;
        sample_properties(*instance);instance->cursor=0;
        while(instance->cursor<instance->asset->signal_count&&instance->asset->signals[instance->cursor].tick<=instance->tick)++instance->cursor;
        instance->busy=false;return true;
    }
    void advance(Fixed dt,uint32_t scene,bool paused=false){
        if(advancing)return;
        advancing=true;
        uint32_t snapshot[Capacity]={};for(size_t i=0;i<Capacity;++i)if(instances[i].state==State::Playing)snapshot[i]=instances[i].generation;
        for(size_t i=0;i<Capacity;++i){auto& instance=instances[i];
            if(!snapshot[i]||snapshot[i]!=instance.generation||instance.busy||!alive(instance,scene)||paused||dt.raw()<=0)continue;
            instance.busy=true;
            // At most one cycle of time is accepted, split into at most two
            // segments across a wrap. Ordinary fixed-step remainder is retained.
            int32_t pending=dt.raw()>instance.asset->duration?instance.asset->duration:dt.raw();
            if(dt.raw()>pending){add(stats.clamped_ticks,uint32_t(dt.raw()-pending));report(instance.asset,0,Reason::ClampedTime);}
            for(unsigned segment=0;segment<2&&alive(instance,scene);++segment){
                if(instance.backwards){
                    const int32_t remaining=instance.tick;
                    const int32_t delta=pending>remaining?remaining:pending;
                    const int32_t end=instance.tick-delta;pending-=delta;
                    uint16_t cursor=0;while(cursor<instance.asset->signal_count&&instance.asset->signals[cursor].tick<instance.tick)++cursor;
                    while(cursor&&instance.asset->signals[cursor-1].tick>=end){
                        const auto signal=instance.asset->signals[--cursor];instance.tick=signal.tick;
                        if(signal.event){sample_properties(instance);invoke(instance,signal.index);}
                        else{add(instance.markers[signal.index]);add(stats.markers);}
                        if(!alive(instance,scene))break;
                    }
                    if(!alive(instance,scene))break;
                    instance.tick=end;sample_properties(instance);
                    instance.cursor=0;while(instance.cursor<instance.asset->signal_count&&instance.asset->signals[instance.cursor].tick<=end)++instance.cursor;
                    if(instance.tick==0){
                        if(instance.asset->repeat){instance.tick=instance.asset->duration;instance.cursor=instance.asset->signal_count;}
                        else finish(instance,State::Completed);
                    }
                    if(!pending)break;
                    continue;
                }
                const int32_t remaining=instance.asset->duration-instance.tick;
                const int32_t delta=pending>remaining?remaining:pending;
                const int32_t end=instance.tick+delta;pending-=delta;
                while(instance.cursor<instance.asset->signal_count&&instance.asset->signals[instance.cursor].tick<=end){
                    const auto signal=instance.asset->signals[instance.cursor++];
                    instance.tick=signal.tick;
                    if(signal.event){sample_properties(instance);invoke(instance,signal.index);}
                    else{add(instance.markers[signal.index]);add(stats.markers);}
                    if(!alive(instance,scene))break;
                }
                if(!alive(instance,scene))break;
                instance.tick=end;sample_properties(instance);
                if(instance.tick==instance.asset->duration){
                    if(instance.asset->repeat){instance.tick=0;instance.cursor=0;}
                    else finish(instance,State::Completed);
                }
                if(!pending)break;
            }
            instance.busy=false;
        }
        advancing=false;
    }
};
}
