#pragma once
#include "timeline_runtime.hpp"
#include "affine.hpp"

namespace epok::effects {
inline constexpr uint16_t capacity=8,layer_capacity=8;
using LayerInitializer=void(*)(EffectLayer&,uint16_t);
enum class State:uint8_t {Invalid,Playing,Draining,Completed,Cancelled};
struct LayerDefinition {
    EffectLayer initial;
    uint16_t slot=0;
    bool emitter=false;
    uint16_t frames=1,columns=1;
    int32_t frame_ticks=410;
};
struct Asset {
    uint64_t id=0;
    const timeline::Asset* timeline=nullptr;
    const LayerDefinition* layers=nullptr;
    uint16_t layer_count=0;
    uint32_t seed=1;
};
struct Stats {
    uint32_t active=0,peak=0,spawned=0,dropped=0,completed=0,cancelled=0,
             active_layers=0,peak_layers=0,dropped_bursts=0,clamped_ticks=0;
};
// One fixed effect pool uses the shared sequence director. Layer handles map
// only into this pool; no entity is created for persistent or transient effects.
class Pool {
    struct Instance {
        const Asset* asset=nullptr;
        DataHandle owner;
        Affine<Fixed> root=Affine<Fixed>::identity(),world[layer_capacity];
        EffectLayer layers[layer_capacity];
        timeline::Handle playback;
        uint32_t generation=0,scene=0;
        int32_t elapsed=0,tail=0;
        State state=State::Invalid;
        bool owned=false,paused=false,activated=false,sequence_completed=false;
    } instances[capacity];
    timeline::Director<8>& director;
    void (*release_particles)(EffectLayerHandle)=nullptr;
    static inline Pool* resolver_owner=nullptr;
    static bool live(const Instance& value){return value.state==State::Playing||value.state==State::Draining;}
    static void add(uint32_t& value,uint32_t amount=1){value=UINT32_MAX-value<amount?UINT32_MAX:value+amount;}
    Instance* find(Handle h){return h.generation&&h.index<capacity&&instances[h.index].generation==h.generation?&instances[h.index]:nullptr;}
    const Instance* find(Handle h)const{return h.generation&&h.index<capacity&&instances[h.index].generation==h.generation?&instances[h.index]:nullptr;}
    static EffectLayer* resolve(EffectLayerHandle h){
        if(!resolver_owner||h.index>=capacity*layer_capacity||!h.generation)return nullptr;
        auto& value=resolver_owner->instances[h.index/layer_capacity];
        const auto layer=h.index%layer_capacity;
        if(!live(value)||value.generation!=h.generation||layer>=value.asset->layer_count||(value.owned&&!value.owner.get()))return nullptr;
        auto& output=value.layers[layer];output.runtime_visible=!value.owned||is_active(value.owner.get());output.runtime_active=!value.paused&&output.runtime_visible;return &output;
    }
    void finish(Instance& value,State state){
        if(!live(value))return;
        // Restore while generation-checked layers still exist, then invalidate.
        director.stop(value.playback);
        if(release_particles)for(uint16_t i=0;i<value.asset->layer_count;++i)release_particles({uint16_t((&value-instances)*layer_capacity+i),value.generation});
        value.state=state;
        --stats.active;stats.active_layers-=value.asset->layer_count;
        add(state==State::Completed?stats.completed:stats.cancelled);
    }
    static Fixed bounded(Fixed value,int32_t low,int32_t high){return Fixed(value.raw()<low?low:value.raw()>high?high:value.raw(),Fixed::RAW);}
public:
    Stats stats;
    explicit Pool(timeline::Director<8>& shared,void(*release)(EffectLayerHandle)=nullptr):director(shared),release_particles(release){}
    Pool(const Pool&)=delete;Pool& operator=(const Pool&)=delete;
    ~Pool(){if(resolver_owner==this){resolver_owner=nullptr;timeline::layer_resolver=nullptr;}}
    State state(Handle h)const{const auto* value=find(h);return value?value->state:State::Invalid;}
    bp::PlaybackSnapshot snapshot(Handle h)const{
        const auto* value=find(h);if(!value)return {};
        return {live(*value)?bp::PlaybackResult::Pending:value->state==State::Completed?bp::PlaybackResult::Completed:bp::PlaybackResult::Cancelled,0};
    }
    timeline::Handle sequence(Handle h)const{const auto* value=find(h);return value?value->playback:timeline::Handle{};}
    DataHandle lighting_owner(EffectLayerHandle h)const{
        if(h.index>=capacity*layer_capacity)return {};
        const auto& value=instances[h.index/layer_capacity];return value.generation==h.generation&&live(value)&&value.owned?value.owner:DataHandle{};
    }
    EffectLayerHandle layer(Handle h,uint16_t index)const{
        const auto* value=find(h);return value&&live(*value)&&index<value->asset->layer_count?EffectLayerHandle{uint16_t(h.index*layer_capacity+index),h.generation}:EffectLayerHandle{};
    }
    Handle spawn(const Asset& asset,const Affine<Fixed>& world,uint32_t scene,
                 const timeline::BoundTarget* external=nullptr,DataHandle owner={},uint32_t seed=0,LayerInitializer initialize=nullptr){
        if(!asset.id||!asset.timeline||!asset.layers||!asset.layer_count||asset.layer_count>layer_capacity||asset.timeline->target_count>timeline::slot_limit||
           (owner.index!=0xffff&&!owner.get())||(resolver_owner&&resolver_owner!=this&&resolver_owner->stats.active)){add(stats.dropped);return {};}
        bool slots[timeline::slot_limit]={};
        for(uint16_t i=0;i<asset.layer_count;++i){const auto& layer=asset.layers[i];
            if(layer.slot>=asset.timeline->target_count||slots[layer.slot]||!layer.frames||layer.frames>256||!layer.columns||layer.columns>layer.frames||layer.frame_ticks<=0){add(stats.dropped);return {};}
            slots[layer.slot]=true;
        }
        resolver_owner=this;timeline::layer_resolver=&resolve;
        for(uint16_t i=0;i<capacity;++i){auto& value=instances[i];if(live(value))continue;
            if(value.generation)bp::observe_playback();
            uint32_t generation=value.generation+1;if(!generation)generation=1;
            value={};value.generation=generation;value.scene=scene;value.asset=&asset;value.root=world;value.owner=owner;value.owned=owner.index!=0xffff;value.state=State::Playing;
            timeline::BoundTarget bindings[timeline::slot_limit];
            if(external)for(uint16_t n=0;n<asset.timeline->target_count;++n)bindings[n]=external[n];
            for(uint16_t n=0;n<asset.layer_count;++n){value.layers[n]=asset.layers[n].initial;
                if(initialize)initialize(value.layers[n],n);
                auto local=Affine<Fixed>::identity();for(unsigned axis=0;axis<3;++axis)local.values[axis][3]=value.layers[n].position[axis];value.world[n]=world.compose(local);
                auto& emitter=value.layers[n].emitter;
                emitter.pending=0;emitter.started=emitter.seeded=false;emitter.accumulator=0.0;
                emitter.seed=(seed?seed:asset.seed)^((uint32_t(n)+1)*UINT32_C(0x9e3779b9));
                bindings[asset.layers[n].slot]=EffectLayerHandle{uint16_t(i*layer_capacity+n),generation};
            }
            value.playback=director.play(*asset.timeline,EffectLayerHandle{uint16_t(i*layer_capacity),generation},bindings,scene);
            if(director.state(value.playback)!=timeline::State::Playing){value.state=State::Cancelled;add(stats.dropped);return {};}
            ++stats.active;stats.active_layers+=asset.layer_count;
            if(stats.active>stats.peak)stats.peak=stats.active;
            if(stats.active_layers>stats.peak_layers)stats.peak_layers=stats.active_layers;
            add(stats.spawned);return {i,generation};
        }
        add(stats.dropped);return {};
    }
    bool stop(Handle h){auto* value=find(h);if(!value||!live(*value))return false;finish(*value,State::Cancelled);return true;}
    bool burst(Handle h,uint32_t count){
        auto* value=find(h);if(!value||value->state!=State::Playing||(value->owned&&!value->owner.get()))return false;
        bool emitted=false;
        for(uint16_t n=0;n<value->asset->layer_count;++n){auto& layer=value->layers[n];
            if(!value->asset->layers[n].emitter||!layer.enabled||!layer.playing)continue;
            layer.burst(count);add(stats.dropped_bursts,layer.dropped_requests);layer.dropped_requests=0;emitted=true;
        }
        return emitted;
    }
    bool pause(Handle h,bool paused){auto* value=find(h);if(!value||!live(*value))return false;value->paused=paused;return true;}
    bool move(Handle h,const Affine<Fixed>& world){auto* value=find(h);if(!value||!live(*value))return false;value->root=world;return true;}
    void cancel_owner(DataHandle owner){for(auto& value:instances)if(value.owned&&bp::same_owner(value.owner,owner))finish(value,State::Cancelled);}
    void reset(){for(auto& value:instances)finish(value,State::Cancelled);}
    // Call before the shared director advances. This is lifecycle preparation,
    // not another continuation scheduler or another timeline evaluator.
    void prepare(uint32_t scene){for(auto& value:instances)if(live(value)){
        if(value.scene!=scene||(value.owned&&!value.owner.get())){finish(value,State::Cancelled);continue;}
        value.activated=true;
        director.pause(value.playback,value.paused);
    }}
    // Capture completion before gameplay can reuse a completed director slot.
    // A draining effect no longer depends on that sequence handle's lifetime.
    void observe(){for(auto& value:instances)if(value.state==State::Playing&&!value.sequence_completed){
        const auto state=director.state(value.playback);
        if(state==timeline::State::Completed)value.sequence_completed=true;
        else if(state==timeline::State::Cancelled||state==timeline::State::Invalid)finish(value,State::Cancelled);
    }}
    // Call after the director, before the single particle simulation step.
    void advance(Fixed dt,bool paused=false){
        if(dt.raw()<=0||paused)return;
        if(dt.raw()>410){add(stats.clamped_ticks,uint32_t(dt.raw()-410));dt=Fixed(410,Fixed::RAW);}
        for(auto& value:instances){if(!live(value)||!value.activated||value.paused||(value.owned&&!is_active(value.owner.get())))continue;
            const auto state=value.sequence_completed?timeline::State::Completed:director.state(value.playback);
            if(value.state==State::Playing&&(state==timeline::State::Cancelled||state==timeline::State::Invalid)){finish(value,State::Cancelled);continue;}
            if(value.state==State::Playing&&state==timeline::State::Completed){
                value.state=State::Draining;value.tail=0;
                for(uint16_t n=0;n<value.asset->layer_count;++n)if(value.asset->layers[n].emitter){
                    const int32_t lifetime=bounded(value.layers[n].emitter.lifetime,1,60*4096).raw();
                    if(lifetime>value.tail)value.tail=lifetime;
                }
            } else if(value.state==State::Draining){value.tail-=dt.raw();}
            if(value.state==State::Draining&&value.tail<=0){finish(value,State::Completed);continue;}
            value.elapsed=bp::iadd(value.elapsed,dt.raw());
            for(uint16_t n=0;n<value.asset->layer_count;++n){auto& layer=value.layers[n];auto& emitter=layer.emitter;
                value.world[n]=value.root.translated(layer.position);
                emitter.enabled=value.asset->layers[n].emitter&&layer.enabled;
                const bool playing=value.state==State::Playing&&layer.playing;
                if(playing&&!emitter.playing)emitter.play();
                else if(!playing&&emitter.playing)emitter.stop();
                emitter.rate=bounded(layer.rate,0,512*4096);
                for(unsigned axis=0;axis<3;++axis)emitter.velocity[axis]=layer.velocity[axis];
                add(stats.dropped_bursts,layer.dropped_requests);layer.dropped_requests=0;
            }
        }
    }
    template<class Visit>void emitters(Visit visit){
        for(uint16_t i=0;i<capacity;++i){auto& value=instances[i];if(!live(value)||!value.activated)continue;
            for(uint16_t n=0;n<value.asset->layer_count;++n)if(value.asset->layers[n].emitter){
                const EffectLayerHandle owner{uint16_t(i*layer_capacity+n),value.generation};
                if(resolve(owner))visit(owner,value.layers[n].emitter,value.world[n]);
            }
        }
    }
    static void tint(const EffectLayer& layer,Sprite& sprite){
        const int32_t opacity=bounded(layer.opacity,0,4096).raw();
        const auto size=bounded(layer.size,0,128*4096);
        for(unsigned c=0;c<2;++c)sprite.size[c]=bp::mul(sprite.size[c],size);
        for(unsigned c=0;c<3;++c)sprite.color[c]=uint8_t(int64_t(sprite.color[c])*bounded(layer.color[c],0,4096).raw()*opacity/(4096*4096));
    }
    template<class Emit>void sprites(Emit emit)const{
        for(uint16_t i=0;i<capacity;++i){const auto& value=instances[i];if(value.state!=State::Playing||!value.activated)continue;
            for(uint16_t n=0;n<value.asset->layer_count;++n){const auto& definition=value.asset->layers[n];const auto& layer=value.layers[n];
                const EffectLayerHandle owner{uint16_t(i*layer_capacity+n),value.generation};
                if(definition.emitter||!layer.enabled||!layer.playing||!resolve(owner)||!layer.runtime_visible)continue;
                auto sprite=layer.sprite;sprite.enabled=true;tint(layer,sprite);
                const uint32_t frame=uint32_t(value.elapsed/definition.frame_ticks)%definition.frames;
                sprite.region[0]+=uint16_t(frame%definition.columns)*sprite.region[2];sprite.region[1]+=uint16_t(frame/definition.columns)*sprite.region[3];
                emit(owner,sprite,value.world[n]);
            }
        }
    }
};
}
