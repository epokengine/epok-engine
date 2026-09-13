#include "effect_preview.h"
#include "particle_effect_runtime.hpp"
#include "particles.hpp"
#include <new>

namespace epok {
// Asset preview has no native game entities. Required external scene bindings
// fail playback; optional ones retain the director's bounded skip diagnostics.
Entity* EntityHandle::get()const{return nullptr;}
bool is_active(const Entity*){return false;}
}
namespace {
using namespace epok;
Fixed raw(int32_t value){return Fixed(value,Fixed::RAW);}
Sprite sprite(const PreviewSprite& input){
    Sprite out;out.enabled=input.enabled!=0;out.texture=input.texture;
    for(unsigned i=0;i<4;++i)out.region[i]=uint16_t(input.region[i]);
    for(unsigned i=0;i<2;++i){out.size[i]=raw(input.size[i]);out.pivot[i]=raw(input.pivot[i]);}
    for(unsigned i=0;i<3;++i)out.color[i]=uint8_t(input.color[i]);
    out.flip_x=input.flip_x!=0;out.flip_y=input.flip_y!=0;out.orientation=SpriteOrientation(input.orientation);
    out.unlit=input.unlit!=0;out.blend=BlendMode(input.blend);out.depth_bias=int16_t(input.depth_bias);return out;
}
PreviewSprite sprite(const Sprite& input){
    PreviewSprite out{};out.enabled=input.enabled;out.texture=input.texture;
    for(unsigned i=0;i<4;++i)out.region[i]=input.region[i];
    for(unsigned i=0;i<2;++i){out.size[i]=input.size[i].raw();out.pivot[i]=input.pivot[i].raw();}
    for(unsigned i=0;i<3;++i)out.color[i]=input.color[i];
    out.flip_x=input.flip_x;out.flip_y=input.flip_y;out.orientation=uint32_t(input.orientation);
    out.unlit=input.unlit;out.blend=uint32_t(input.blend);out.depth_bias=input.depth_bias;return out;
}
bool accepts(timeline::BoundTarget target){return target.effect_layer()!=nullptr;}
// Fixed, explicit SDK property adapters. The host supplies a validated adapter
// tag, never an offset, arbitrary pointer, editable-field name or C++ memory span.
template<unsigned Field>bool read(timeline::BoundTarget target,timeline::Value& value){
    const auto* layer=target.effect_layer();if(!layer||!target.active())return false;
    if constexpr(Field==0)value.lanes[0]=layer->enabled;
    else if constexpr(Field==1)value.lanes[0]=layer->playing;
    else if constexpr(Field==2)value.lanes[0]=layer->opacity.raw();
    else if constexpr(Field==3)value.lanes[0]=layer->size.raw();
    else if constexpr(Field==4)for(unsigned i=0;i<3;++i)value.lanes[i]=layer->position[i].raw();
    else if constexpr(Field==5)for(unsigned i=0;i<3;++i)value.lanes[i]=layer->color[i].raw();
    else if constexpr(Field==6)value.lanes[0]=layer->rate.raw();
    else if constexpr(Field==7)for(unsigned i=0;i<3;++i)value.lanes[i]=layer->velocity[i].raw();
    return true;
}
template<unsigned Field>bool write(timeline::BoundTarget target,const timeline::Value& value){
    auto* layer=target.effect_layer();if(!layer||!target.active())return false;
    if constexpr(Field==0)layer->enabled=value.lanes[0]!=0;
    else if constexpr(Field==1)layer->playing=value.lanes[0]!=0;
    else if constexpr(Field==2)layer->opacity=raw(value.lanes[0]);
    else if constexpr(Field==3)layer->size=raw(value.lanes[0]);
    else if constexpr(Field==4)for(unsigned i=0;i<3;++i)layer->position[i]=raw(value.lanes[i]);
    else if constexpr(Field==5)for(unsigned i=0;i<3;++i)layer->color[i]=raw(value.lanes[i]);
    else if constexpr(Field==6)layer->rate=raw(value.lanes[0]);
    else if constexpr(Field==7)for(unsigned i=0;i<3;++i)layer->velocity[i]=raw(value.lanes[i]);
    return true;
}
template<unsigned Function>bool invoke(timeline::BoundTarget target,const timeline::BoundTarget*,const timeline::Argument* args){
    auto* layer=target.effect_layer();if(!layer||!target.active())return false;
    if constexpr(Function==0)layer->play();
    else if constexpr(Function==1)layer->stop();
    else layer->burst(uint32_t(args[0].lanes[0]));
    return true;
}
struct Context;
Context* current=nullptr;
void release(EffectLayerHandle);
struct Context {
    timeline::Director<8> director;
    ParticlePool particles;
    effects::Pool pool{director,release};
    effects::LayerDefinition layers[8];
    timeline::Target targets[8];
    timeline::Property properties[16];
    timeline::Curve curves[16][4];
    timeline::Key keys[16][4][4];
    timeline::Event events[64];
    timeline::Argument arguments[64];
    timeline::Signal signals[128];
    uint64_t markers[64];
    timeline::Asset sequence{};
    effects::Asset effect{};
    effects::Handle handle;
    uint32_t layer_count=0,track_count=0,event_count=0;
    bool started=false;
};
void release(EffectLayerHandle handle){if(current)current->particles.remove_layer(handle);}
Context* context(void* pointer){return pointer&&pointer==current?current:nullptr;}
}
extern "C" void* epok_preview_create(){
    if(current)return nullptr;
    current=new(std::nothrow) Context;
    if(current)current->particles.clear();
    return current;
}
extern "C" void epok_preview_destroy(void* pointer){
    if(auto* value=context(pointer)){value->pool.reset();value->particles.clear();delete value;current=nullptr;}
}
extern "C" uint32_t epok_preview_layer(void* pointer,const PreviewLayer* input){
    auto* value=context(pointer);if(!value||value->started||!input||value->layer_count>=8||input->slot>=8)return 0;
    auto& output=value->layers[value->layer_count++];auto& layer=output.initial;
    output.slot=uint16_t(input->slot);output.emitter=input->emitter!=0;
    if(!output.emitter){output.frames=uint16_t(input->frames);output.columns=uint16_t(input->columns);output.frame_ticks=input->frame_ticks;}
    layer.sprite=sprite(input->sprite);layer.enabled=input->enabled!=0;layer.playing=input->playing!=0;layer.rate=raw(input->rate);
    auto& emitter=layer.emitter;emitter.sprite=layer.sprite;emitter.enabled=layer.enabled;emitter.playing=layer.playing;
    emitter.continuous=input->continuous!=0;emitter.local_space=input->local_space!=0;emitter.seed=input->seed;
    emitter.burst_count=uint16_t(input->burst);emitter.max_particles=uint16_t(input->max_particles);
    emitter.rate=layer.rate;emitter.lifetime=raw(input->lifetime);emitter.start_size=raw(input->start_size);emitter.end_size=raw(input->end_size);
    emitter.frames=uint16_t(input->frames);emitter.frame_columns=uint16_t(input->columns);emitter.frame_duration=raw(input->frame_ticks);
    for(unsigned i=0;i<3;++i){layer.position[i]=raw(input->position[i]);layer.velocity[i]=emitter.velocity[i]=raw(input->velocity[i]);
        emitter.spread[i]=raw(input->spread[i]);emitter.gravity[i]=raw(input->gravity[i]);emitter.start_color[i]=uint8_t(input->start_color[i]);emitter.end_color[i]=uint8_t(input->end_color[i]);}
    return 1;
}
extern "C" uint32_t epok_preview_track(void* pointer,const PreviewTrack* input){
    auto* value=context(pointer);if(!value||value->started||!input||value->track_count>=16||input->slot>=8||input->field>8||input->interpolation>4)return 0;
    const bool vector=input->field==4||input->field==5||input->field==7;
    if(!input->channels||input->channels>4||(input->field!=8&&input->channels!=(vector?3u:1u)))return 0;
    for(unsigned c=0;c<input->channels;++c)if(!input->lengths[c]||input->lengths[c]>4)return 0;
    const auto index=value->track_count++;auto& property=value->properties[index];
    property={input->property,uint16_t(input->slot),uint8_t(input->channels),input->additive!=0,input->restore!=0,value->curves[index],nullptr,nullptr};
    for(unsigned c=0;c<input->channels;++c){for(unsigned k=0;k<input->lengths[c];++k)value->keys[index][c][k]={input->keys[c][k].tick,input->keys[c][k].value};
        value->curves[index][c]={value->keys[index][c],uint16_t(input->lengths[c]),timeline::Interpolation(input->interpolation),false};}
    switch(input->field){
#define EPOK_PREVIEW_FIELD(N) case N:property.read=read<N>;property.write=write<N>;break;
        EPOK_PREVIEW_FIELD(0) EPOK_PREVIEW_FIELD(1) EPOK_PREVIEW_FIELD(2) EPOK_PREVIEW_FIELD(3)
        EPOK_PREVIEW_FIELD(4) EPOK_PREVIEW_FIELD(5) EPOK_PREVIEW_FIELD(6) EPOK_PREVIEW_FIELD(7)
#undef EPOK_PREVIEW_FIELD
    }
    return 1;
}
extern "C" uint32_t epok_preview_event(void* pointer,const PreviewEvent* input){
    auto* value=context(pointer);if(!value||value->started||!input||value->event_count>=64||input->slot>=8||input->function>3)return 0;
    const auto index=value->event_count++;value->arguments[index].lanes[0]=int32_t(input->count);
    auto& event=value->events[index];event={uint16_t(input->slot),uint8_t(input->function==2?1:0),input->idempotent!=0,&value->arguments[index],nullptr};
    switch(input->function){case 0:event.invoke=invoke<0>;break;case 1:event.invoke=invoke<1>;break;case 2:event.invoke=invoke<2>;break;}
    return 1;
}
extern "C" uint32_t epok_preview_start(void* pointer,uint64_t id,uint64_t timeline_id,int32_t duration,uint32_t repeat,uint32_t seed,uint32_t slots,uint32_t required,const uint64_t* markers,uint32_t marker_count,const PreviewSignal* signals,uint32_t signal_count){
    auto* value=context(pointer);if(!value||value->started||!id||!timeline_id||duration<=0||slots>8||marker_count>64||signal_count>128||(!markers&&marker_count)||(!signals&&signal_count))return 0;
    for(unsigned i=0;i<slots;++i)value->targets[i]={(required&(1u<<i))!=0,accepts};
    for(unsigned i=0;i<marker_count;++i)value->markers[i]=markers[i];
    for(unsigned i=0;i<signal_count;++i){const auto& signal=signals[i];if(signal.tick<0||signal.tick>duration||(i&&signal.tick<signals[i-1].tick)||signal.index>=(signal.event?value->event_count:marker_count))return 0;
        value->signals[i]={signal.tick,uint16_t(signal.index),signal.event!=0};}
    value->sequence={timeline_id,duration,repeat!=0,uint16_t(slots),uint16_t(value->track_count),uint16_t(value->event_count),uint16_t(marker_count),uint16_t(signal_count),value->targets,value->properties,value->events,value->markers,value->signals};
    value->effect={id,&value->sequence,value->layers,uint16_t(value->layer_count),seed};
    value->handle=value->pool.spawn(value->effect,Affine<Fixed>::identity(),1);
    value->started=true;return value->pool.state(value->handle)==effects::State::Playing;
}
extern "C" void epok_preview_step(void* pointer,int32_t tick,uint32_t paused){
    auto* value=context(pointer);if(!value||!value->started||tick<=0)return;
    value->pool.pause(value->handle,paused!=0);value->pool.prepare(1);
    value->director.advance(raw(tick),1,paused!=0);value->pool.observe();value->pool.advance(raw(tick),paused!=0);
    if(paused)return;
    value->particles.begin(raw(tick));
    value->pool.emitters([&](EffectLayerHandle owner,ParticleEmitter& emitter,const Affine<Fixed>& world){value->particles.emitter(owner,emitter,world);});
    value->particles.advance();
}
extern "C" void epok_preview_stats(void* pointer,PreviewStats* output){
    auto* value=context(pointer);if(!value||!output)return;const auto& stats=value->director.stats;
    *output={uint32_t(value->pool.state(value->handle)),uint32_t(value->director.tick(value->pool.sequence(value->handle))),particle_stats.alive,particle_stats.spawned,particle_stats.dropped,particle_stats.peak,stats.events,stats.markers,stats.skipped_targets,stats.skipped_events,particle_stats.dropped_emitters,stats.diagnostics_dropped};
}
extern "C" uint32_t epok_preview_quads(void* pointer,PreviewQuad* output,uint32_t capacity){
    auto* value=context(pointer);if(!value||!output)return 0;uint32_t count=0;
    auto emit=[&](EffectLayerHandle layer,const Sprite& sprite,const Affine<Fixed>& world){if(count>=capacity)return;auto& quad=output[count++];quad.layer=layer.index%8;quad.sprite=::sprite(sprite);
        for(unsigned r=0;r<3;++r)for(unsigned c=0;c<4;++c)quad.world[r*4+c]=world.values[r][c].raw();};
    value->pool.sprites(emit);
    value->particles.each_all([&](timeline::BoundTarget owner,const Sprite& source,const Affine<Fixed>& world){if(auto* layer=owner.effect_layer()){auto sprite=source;effects::Pool::tint(*layer,sprite);emit(owner.layer,sprite,world);}});
    return count;
}
extern "C" uint32_t epok_preview_particles(void* pointer,PreviewParticle* output,uint32_t capacity){
    auto* value=context(pointer);if(!value||!output)return 0;uint32_t count=0;
    for(unsigned index=0;index<256&&count<capacity;++index){const auto& p=value->particles.particles[index];if(!p.alive)continue;auto& out=output[count++];
        out.index=index;out.layer=p.owner.layer.index%8;out.age=p.age.raw();out.lifetime=p.lifetime.raw();
        for(unsigned i=0;i<3;++i){out.position[i]=p.position[i].raw();out.velocity[i]=p.velocity[i].raw();}}
    return count;
}
extern "C" uint32_t epok_preview_abi(uint32_t type){
    switch(type){case 0:return sizeof(PreviewSprite);case 1:return sizeof(PreviewLayer);case 2:return sizeof(PreviewTrack);case 3:return sizeof(PreviewEvent);case 4:return sizeof(PreviewSignal);case 5:return sizeof(PreviewQuad);case 6:return sizeof(PreviewParticle);case 7:return sizeof(PreviewStats);default:return 0;}
}
