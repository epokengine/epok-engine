#include "timeline_runtime.hpp"
#include <array>
#include <cassert>
#include <cstdio>
#include "actor_scene_fixture.hpp"
using namespace epok;
using namespace epok::timeline;
static bool compatible=true;
static int32_t observed[16];static size_t calls;
static Director<2>* current=nullptr;static Handle current_handle;
static int action=0;
static bool accepts(BoundTarget target){return compatible&&target.get();}
static bool read(BoundTarget target,Value& value){if(!accepts(target))return false;value.lanes[0]=target.data()->transform.position[0].raw();return true;}
static bool write(BoundTarget target,const Value& value){if(!accepts(target))return false;target.data()->transform.position[0]=Fixed(value.lanes[0],Fixed::RAW);return true;}
static bool event(BoundTarget target,const BoundTarget*,const Argument*){
    assert(target.get());assert(calls<16);observed[calls++]=target.data()->transform.position[0].raw();
    if(action==1){test_invalidate(0);}
    if(action==2)current->stop(current_handle);
    if(action==3)current->advance(Fixed(100,Fixed::RAW),1); // Reentrant advance is ignored.
    return true;
}
static const Key keys[]={{0,0},{100,100}};
static const Curve curves[]={{keys,2}};
static const Target targets[]={{true,accepts}};
static const Property properties[]={{1,0,1,false,true,curves,read,write}};
static const Event events[]={{0,0,false,nullptr,event}};
static const uint64_t marker_ids[]={11,12};
static const Signal signals[]={{0,0,false},{50,0,true},{50,1,false},{100,0,true}};
static const Asset asset={1,100,false,1,1,1,2,4,targets,properties,events,marker_ids,signals};
static const DataHandle owner{0,1},binding[]={DataHandle{1,1}};
static Fixed dt(int32_t value){return Fixed(value,Fixed::RAW);}
static int32_t value(){return entities[1].transform.position[0].raw();}
static void reset(){test_reset_scene();for(auto& entity:entities)entity.parent=-1;entities[1].transform.position[0]=dt(42);compatible=true;calls=0;action=0;}
static EffectLayer layer;static uint32_t layer_generation=1;
static EffectLayer* resolve_layer(EffectLayerHandle handle){return handle.index==1&&handle.generation==layer_generation?&layer:nullptr;}
static bool accepts_layer(BoundTarget target){return target.effect_layer()!=nullptr;}
static bool read_layer(BoundTarget target,Value& value){auto* p=target.effect_layer();if(!p)return false;value.lanes[0]=p->size.raw();return true;}
static bool write_layer(BoundTarget target,const Value& value){auto* p=target.effect_layer();if(!p)return false;p->size=dt(value.lanes[0]);return true;}
static bool event_layer(BoundTarget target,const BoundTarget*,const Argument*){auto* p=target.effect_layer();if(!p)return false;p->burst(8);return true;}
int main(){
    reset();Director<2> director;current=&director;
    assert(director.marker({0,0},0)==0);
    auto h=director.play(asset,owner,binding,1);current_handle=h;
    assert(director.state(h)==State::Playing);
    director.advance(dt(75),1);assert(value()==75&&calls==1&&observed[0]==50);
    assert(director.marker(h,0)==1&&director.marker(h,1)==1);
    assert(director.marker_revision(h,11)==1&&director.marker_revision(h,999)==0);
    director.advance(dt(0),1);assert(calls==1);
    director.advance(dt(25),1);assert(calls==2&&observed[1]==100&&value()==42);
    assert(director.state(h)==State::Completed&&director.stats.completed==1&&director.stats.active==0);

    // A section writes only inside its half-open range and final root sample.
    // Its root capture remains the restoration source across inactive gaps.
    reset();const Property section_property[]={{1,0,1,false,true,curves,read,write,25,75,0,1,1}};
    auto section_asset=asset;section_asset.properties=section_property;section_asset.signal_count=0;
    h=director.play(section_asset,owner,binding,1);
    assert(director.seek(h,10,1)&&value()==42);
    assert(director.seek(h,50,1)&&value()==25);
    assert(director.seek(h,80,1)&&value()==25);
    director.stop(h);assert(value()==25);
    auto next=director.play(asset,owner,binding,1);assert(next.generation!=h.generation&&director.state(h)==State::Invalid);
    director.advance(dt(20),1);test_set_active(0,false);
    director.advance(dt(30),1);assert(director.tick(next)==20);
    test_set_active(0,true);director.pause(next,true);director.advance(dt(30),1);assert(director.tick(next)==20);
    director.pause(next,false);director.advance(dt(30),2);assert(director.state(next)==State::Cancelled&&value()==20);

    // Optional target destruction/reuse never writes into the replacement slot.
    reset();auto optional=asset;const Target optional_targets[]={{false,accepts}};optional.targets=optional_targets;
    h=director.play(optional,owner,binding,2);director.advance(dt(10),2);test_invalidate(1);entities[1].transform.position[0]=dt(777);
    director.advance(dt(90),2);assert(value()==777&&director.stats.skipped_targets>0&&director.stats.skipped_events>0);
    assert(director.state(director.play(asset,owner,binding,2))==State::Invalid);
    compatible=false;const DataHandle replacement[]={DataHandle{1,2}};
    assert(director.state(director.play(asset,owner,replacement,2))==State::Invalid);

    // A callback can destroy its owner or cancel playback; later signals stop.
    reset();h=director.play(asset,owner,binding,1);current_handle=h;action=1;director.advance(dt(100),1);
    assert(calls==1&&value()==50&&director.state(h)==State::Cancelled);
    reset();h=director.play(asset,owner,binding,1);current_handle=h;action=2;director.advance(dt(100),1);
    assert(calls==1&&director.state(h)==State::Cancelled&&value()==50);
    reset();h=director.play(asset,owner,binding,1);current_handle=h;action=3;director.advance(dt(100),1);assert(calls==2);

    // Loop remainder survives an ordinary wrap. Large dt is deliberately bounded.
    reset();auto looping=asset;looping.repeat=true;h=director.play(looping,owner,binding,1);
    director.advance(dt(75),1);director.advance(dt(75),1);
    assert(director.tick(h)==50&&director.marker(h,0)==2&&director.marker(h,1)==2&&calls==3);
    director.advance(dt(1000),1);assert(director.tick(h)==50&&director.stats.clamped_ticks==900);
    assert(director.marker(h,0)==3&&director.marker(h,1)==3);
    director.cancel_all();assert(value()==50);

    // Absolute seek is silent in either direction and while paused. Idempotent
    // events are not treated as reversible. Advance crosses only new keys.
    reset();h=director.play(asset,owner,binding,1);assert(director.seek(h,75,1));assert(value()==75&&calls==0&&director.marker(h,0)==0);
    assert(director.seek(h,25,1)&&director.tick(h)==25&&value()==25&&calls==0);
    director.pause(h,true);assert(director.seek(h,75,1)&&value()==75&&calls==0);director.pause(h,false);
    director.advance(dt(25),1);assert(calls==1&&observed[0]==100);director.cancel_all();
    reset();auto seek_asset=asset;const Event actions[]={{0,0,true,nullptr,event}};seek_asset.events=actions;
    h=director.play(seek_asset,owner,binding,1);assert(director.seek(h,75,1));assert(calls==0);director.cancel_all();

    // Reverse traverses [new,current), including zero but excluding the old
    // endpoint. Signals at one tick have a stable reverse order.
    reset();h=director.play(asset,owner,binding,1);assert(director.seek(h,75,1));assert(director.reverse(h,true));
    director.advance(dt(50),1);assert(director.tick(h)==25&&value()==25&&calls==1&&observed[0]==50);
    assert(director.marker(h,0)==0&&director.marker(h,1)==1);
    director.advance(dt(25),1);assert(director.state(h)==State::Completed&&value()==42&&director.marker(h,0)==1);

    // Absolute plus additive tracks use one captured baseline and do not drift.
    reset();const Key delta_keys[]={{0,0},{100,10}};const Curve delta_curves[]={{delta_keys,2}};
    const Property blended[]={{1,0,1,false,true,curves,read,write},{1,0,1,true,true,delta_curves,read,write}};
    auto blended_asset=asset;blended_asset.properties=blended;blended_asset.property_count=2;blended_asset.signal_count=0;
    h=director.play(blended_asset,owner,binding,1);director.advance(dt(50),1);assert(value()==55);director.advance(dt(25),1);assert(value()==82);director.stop(h);assert(value()==82);

    // Cross-instance property claims and the instance pool both fail predictably.
    reset();h=director.play(asset,owner,binding,1);assert(director.state(director.play(asset,owner,binding,1))==State::Invalid&&director.stats.conflicts>0);
    auto* aliased_actor=test_registry.resolve<Actor3D>(test_actors[1]);assert(aliased_actor);
    const BoundTarget aliased_binding[]={aliased_actor->root.id()};
    assert(director.state(director.play(asset,test_actors[0],aliased_binding,1))==State::Invalid&&director.stats.conflicts>1);
    auto empty=asset;empty.property_count=0;empty.signal_count=0;auto second=director.play(empty,owner,binding,1);
    assert(director.state(second)==State::Playing);assert(director.state(director.play(empty,owner,binding,1))==State::Invalid);
    director.cancel_all();assert(director.stats.active==0);
    reset();optional.repeat=true;const DataHandle missing[]={{}};
    h=director.play(optional,owner,missing,1);
    for(int i=0;i<40;++i)director.advance(dt(1),1);
    assert(director.stats.diagnostics_dropped>0);
    Diagnostic diagnostic;size_t count=0;while(director.poll_diagnostic(diagnostic))++count;
    assert(count==32);director.cancel_all();
    // One director handles both target kinds. Equal numeric indices/generations
    // cannot alias an entity and an effect layer, and a layer can own playback.
    reset();layer_resolver=resolve_layer;layer={};layer_generation=1;
    const BoundTarget layer_binding[]={EffectLayerHandle{1,1}};
    const Target layer_targets[]={{true,accepts_layer}};
    const Property layer_properties[]={{1,0,1,false,true,curves,read_layer,write_layer}};
    const Event layer_events[]={{0,0,false,nullptr,event_layer}};
    auto layer_asset=asset;layer_asset.targets=layer_targets;layer_asset.properties=layer_properties;layer_asset.events=layer_events;
    h=director.play(layer_asset,layer_binding[0],layer_binding,1);
    second=director.play(asset,owner,binding,1);assert(director.state(h)==State::Playing&&director.state(second)==State::Playing);
    director.advance(dt(25),1);assert(layer.size.raw()==25&&value()==25);
    layer.runtime_active=false;director.advance(dt(25),1);assert(layer.size.raw()==25&&value()==50&&layer.emitter.pending==0);
    layer.runtime_active=true;director.advance(dt(25),1);assert(layer.size.raw()==50&&layer.emitter.pending==8);
    ++layer_generation;layer.size=77.0;director.advance(dt(25),1);
    assert(director.state(h)==State::Cancelled&&layer.size.raw()==77*4096&&layer.emitter.pending==8);
    director.cancel_all();layer_resolver=nullptr;
    std::puts("Timeline director: curves/events, restoration, seek, looping, pause, generations, cancellation and bounded pools passed.");
}
