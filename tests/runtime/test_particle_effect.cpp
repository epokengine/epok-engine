#include "particle_effect_runtime.hpp"
#include "particles.hpp"
#include <array>
#include <cassert>
#include <cstdio>
namespace epok {
static std::array<Entity,2> entities;
Entity* EntityHandle::get()const{return index<entities.size()&&entities[index].alive&&entities[index].generation==generation?&entities[index]:nullptr;}
bool is_active(const Entity* entity){return entity&&entity->alive&&entity->active;}
}
using namespace epok;
static bool accepts(timeline::BoundTarget target){return target.effect_layer()!=nullptr;}
static Fixed raw(int32_t value){return Fixed(value,Fixed::RAW);}
int main(){
    timeline::Director<8> sequences;effects::Pool pool(sequences);
    const timeline::Target targets[]={{true,accepts}};
    const timeline::Asset sequence={1,100,false,1,0,0,0,0,targets,nullptr,nullptr,nullptr,nullptr};
    effects::LayerDefinition layers[1];layers[0].emitter=true;layers[0].initial.emitter.lifetime=raw(200);
    const effects::Asset asset={2,&sequence,layers,1,123};
    const auto world=Affine<Fixed>::identity();
    effects::Handle handles[8];
    for(auto& h:handles){h=pool.spawn(asset,world,1);assert(pool.state(h)==effects::State::Playing);}
    assert(pool.stats.active==8&&pool.stats.active_layers==8&&sequences.stats.active==8);
    assert(pool.state(pool.spawn(asset,world,1))==effects::State::Invalid&&pool.stats.dropped==1);
    const auto old=handles[0];const auto old_layer=pool.layer(old,0);
    assert(timeline::layer_resolver(old_layer)&&pool.stop(old)&&!timeline::layer_resolver(old_layer));
    handles[0]=pool.spawn(asset,world,1);assert(handles[0].index==old.index&&handles[0].generation!=old.generation);
    assert(pool.state(old)==effects::State::Invalid&&!timeline::layer_resolver(old_layer));
    assert(entities[0].alive&&entities[1].alive); // No spare entity allocations.
    pool.reset();assert(pool.stats.active==0&&sequences.stats.active==0);
    auto h=pool.spawn(asset,world,1,nullptr,EntityHandle{0,1});
    entities[0].active=false;pool.prepare(1);sequences.advance(raw(50),1);pool.advance(raw(50));
    assert(sequences.tick(pool.sequence(h))==0);
    entities[0].active=true;assert(pool.pause(h,true));pool.prepare(1);sequences.advance(raw(50),1);pool.advance(raw(50));
    assert(sequences.tick(pool.sequence(h))==0);
    assert(pool.pause(h,false));pool.prepare(1);sequences.advance(raw(50),1);pool.advance(raw(50));
    assert(sequences.tick(pool.sequence(h))==50);
    entities[0].alive=false;++entities[0].generation;pool.prepare(1);
    assert(pool.state(h)==effects::State::Cancelled&&sequences.stats.active==0);
    h=pool.spawn(asset,world,1);const auto layer=pool.layer(h,0);
    auto* state=timeline::layer_resolver(layer);assert(state);state->burst(UINT32_MAX);assert(state->emitter.pending==256&&state->dropped_requests==UINT32_MAX-256);
    pool.prepare(1);sequences.advance(raw(100),1);pool.advance(raw(100));
    assert(pool.state(h)==effects::State::Draining&&pool.stats.dropped_bursts==UINT32_MAX-256);
    assert(!state->emitter.playing&&timeline::layer_resolver(layer));
    pool.advance(raw(100));assert(pool.state(h)==effects::State::Draining);
    pool.advance(raw(100));assert(pool.state(h)==effects::State::Completed&&!timeline::layer_resolver(layer)&&pool.stats.completed==1);
    // Completing a sequence releases its director slot before the effect's
    // particle tail ends. Reuse must preserve that tail, including same-frame
    // gameplay spawning between timeline evaluation and effect advancement.
    h=pool.spawn(asset,world,1);pool.prepare(1);sequences.advance(raw(100),1);pool.observe();
    auto successor=pool.spawn(asset,world,1);
    assert(sequences.state(pool.sequence(h))==timeline::State::Invalid);
    pool.advance(raw(50));assert(pool.state(h)==effects::State::Draining);
    pool.prepare(1);sequences.advance(raw(50),1);pool.observe();pool.advance(raw(50));
    assert(pool.state(h)==effects::State::Draining&&pool.state(successor)==effects::State::Playing);
    pool.reset();
    h=pool.spawn(asset,world,1);pool.prepare(2);assert(pool.state(h)==effects::State::Cancelled);
    auto invalid=asset;invalid.layer_count=0;assert(pool.state(pool.spawn(invalid,world,1))==effects::State::Invalid);
    // Sequence exhaustion must roll back an effect reservation without leaving
    // a live layer behind, even if all effect slots themselves are free.
    const timeline::Asset empty={3,100,false,0,0,0,0,0,nullptr,nullptr,nullptr,nullptr,nullptr};
    for(unsigned i=0;i<8;++i)assert(sequences.state(sequences.play(empty,EntityHandle{1,1},nullptr,1))==timeline::State::Playing);
    assert(pool.state(pool.spawn(asset,world,1))==effects::State::Invalid&&pool.stats.active==0);
    sequences.cancel_all();assert(pool.state(pool.spawn(asset,world,1))==effects::State::Playing);
    pool.reset();
    // Scene and effect emitters share the same particle and emitter budgets.
    ParticlePool particles;particles.clear();
    auto& scene_emitter=entities[1].particle_emitter;
    scene_emitter.enabled=true;scene_emitter.continuous=false;scene_emitter.max_particles=scene_emitter.burst_count=128;
    layers[0].initial.emitter.continuous=false;layers[0].initial.emitter.max_particles=layers[0].initial.emitter.burst_count=128;layers[0].initial.emitter.lifetime=1.0;
    auto first=pool.spawn(asset,world,1),second=pool.spawn(asset,world,1);
    auto tick=[&](int32_t dt){pool.prepare(1);sequences.advance(raw(dt),1);pool.advance(raw(dt));particles.begin(raw(dt));
        particles.emitter(EntityHandle{1,1},scene_emitter,world);
        pool.emitters([&](EffectLayerHandle owner,ParticleEmitter& emitter,const Affine<Fixed>& matrix){particles.emitter(owner,emitter,matrix);});particles.advance();};
    tick(1);assert(particle_stats.alive==256&&particle_stats.dropped==128);
    unsigned native=0,internal=0;
    particles.each_all([&](timeline::BoundTarget owner,const Sprite&,const Affine<Fixed>&){if(owner.internal)++internal;else ++native;});
    assert(native==128&&internal==128);
    const auto age=particles.particles[128].age.raw();pool.pause(first,true);tick(10);
    assert(particles.particles[128].age.raw()==age&&particle_stats.alive==256);
    internal=0;particles.each_all([&](timeline::BoundTarget owner,const Sprite&,const Affine<Fixed>&){if(owner.internal)++internal;});assert(internal==128);
    pool.stop(first);tick(1);assert(particle_stats.alive==128);
    pool.stop(second);pool.reset();particles.clear();
    effects::LayerDefinition many[8];timeline::Target many_targets[8];
    for(unsigned i=0;i<8;++i){many[i]=layers[0];many[i].slot=uint16_t(i);many[i].initial.emitter.burst_count=many[i].initial.emitter.max_particles=1;many_targets[i]={true,accepts};}
    auto many_sequence=sequence;many_sequence.target_count=8;many_sequence.targets=many_targets;
    auto many_asset=asset;many_asset.timeline=&many_sequence;many_asset.layers=many;many_asset.layer_count=8;
    scene_emitter.burst_count=scene_emitter.max_particles=1;scene_emitter.play();
    for(auto& handle:handles){handle=pool.spawn(many_asset,world,1);assert(pool.state(handle)==effects::State::Playing);}
    tick(1);assert(pool.stats.active_layers==64&&particle_stats.alive==64&&particle_stats.dropped==1&&particle_stats.dropped_emitters==1);
    pool.reset();tick(1);assert(particle_stats.alive==1);
    // Per-instance initialization runs before binding and initial world capture.
    // The immutable asset and later unmodified plays retain their defaults.
    auto overridden=pool.spawn(asset,world,1,nullptr,{},0,[](EffectLayer& l,uint16_t index){assert(index==0);l.position[0]=3.0;l.opacity=0.25;l.rate=17.0;});
    auto* overridden_layer=timeline::layer_resolver(pool.layer(overridden,0));
    assert(overridden_layer&&overridden_layer->position[0].raw()==12288&&overridden_layer->opacity.raw()==1024);
    auto unchanged=pool.spawn(asset,world,1);auto* unchanged_layer=timeline::layer_resolver(pool.layer(unchanged,0));
    assert(unchanged_layer&&unchanged_layer->position[0].raw()==0&&unchanged_layer->opacity.raw()==4096);
    assert(layers[0].initial.position[0].raw()==0);pool.reset();
    std::printf("Effect pool: capacity, generations, detached ownership, pause, drain, scene cancellation and shared director exhaustion passed; pool=%zu bytes.\n",sizeof(pool));
}
