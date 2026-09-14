#include "particle_effect_runtime.hpp"
#include <array>
#include <cassert>
#include <cstdio>
#include "actor_scene_fixture.hpp"
using namespace epok;
static timeline::Director<8> director;
static effects::Pool effects_pool(director);
static bp::Continuations<8> tasks;
static bp::PlaybackWait playback_wait;
static bp::PlaybackSubscription subscription;
static unsigned observations=0;
static Fixed raw(int32_t n){return Fixed(n,Fixed::RAW);}
static void observe(){
    ++observations;
    // Nested observation cannot recurse into another traversal.
    bp::observe_playback();
    if(tasks.waiting(7)&&playback_wait.capture(playback_wait.effect_completion?effects_pool.snapshot(playback_wait.effect):director.snapshot(playback_wait.sequence,playback_wait.asset,playback_wait.marker)))tasks.signal(7,1);
    if(subscription.active&&subscription.capture(director.snapshot(subscription.sequence,subscription.asset,subscription.marker))&&tasks.waiting(8))tasks.signal(8,1);
}
static void pending(){assert(tasks.wait_external(7,test_owner(0),1));}
static bool poll(){bp::Continuation result;return tasks.poll(result);}
static bool layer(timeline::BoundTarget target){return target.effect_layer()!=nullptr;}
int main(){
    test_reset_scene();
    const uint64_t markers[]={44};const timeline::Signal signals[]={{50,0,false}};
    const timeline::Asset asset={33,100,false,0,0,0,1,1,nullptr,nullptr,nullptr,markers,signals};
    const DataHandle owner{1,1};
    bp::playback_observer=&observe;
    auto h=director.play(asset,owner,nullptr,1);
    playback_wait.begin(h,33,44);pending();tasks.advance(raw(1),1);
    director.advance(raw(49),1);observe();assert(!poll());
    director.advance(raw(1),1);observe();assert(poll());assert(playback_wait.result==bp::PlaybackResult::Reached&&playback_wait.revision==1);
    observe();assert(!poll()); // A crossing schedules the frame once.
    // Completion is captured before a different play reuses the slot.
    playback_wait.begin(h);pending();tasks.advance(raw(1),1);director.advance(raw(50),1);
    const auto replacement=director.play(asset,owner,nullptr,1);
    assert(replacement.index==h.index&&replacement.generation!=h.generation);
    assert(poll()&&playback_wait.result==bp::PlaybackResult::Completed);
    playback_wait.begin(h);pending();observe();assert(!poll());tasks.advance(raw(1),1);
    assert(poll()&&playback_wait.result==bp::PlaybackResult::Cancelled);
    // A wrong asset or marker cannot accept a coincidentally matching revision.
    playback_wait.begin(replacement,999,44);pending();observe();tasks.advance(raw(1),1);assert(poll()&&playback_wait.result==bp::PlaybackResult::Cancelled);
    playback_wait.begin(replacement,33,999);pending();observe();tasks.advance(raw(1),1);assert(poll()&&playback_wait.result==bp::PlaybackResult::Cancelled);
    // The result survives inactive/paused consumer dispatch and terminal reuse.
    playback_wait.begin(replacement,33,44);pending();test_set_active(0,false);
    director.advance(raw(100),1);director.play(asset,owner,nullptr,1);
    tasks.advance(raw(1),1,true);assert(!poll());test_set_active(0,true);assert(!poll());
    tasks.advance(raw(1),1);assert(poll()&&playback_wait.result==bp::PlaybackResult::Reached);
    director.cancel_all();
    // Cancellation signals the waiting consumer; invalid consumers run nothing.
    h=director.play(asset,owner,nullptr,1);playback_wait.begin(h);pending();director.stop(h);observe();tasks.advance(raw(1),1);
    assert(poll()&&playback_wait.result==bp::PlaybackResult::Cancelled);
    h=director.play(asset,owner,nullptr,1);playback_wait.begin(h);pending();test_invalidate(0);
    director.advance(raw(100),1);observe();tasks.advance(raw(1),1);assert(!poll()&&tasks.size()==0);
    test_restore(0);playback_wait.begin(h);pending();tasks.advance(raw(1),2);assert(!poll()&&tasks.size()==0);
    director.cancel_all();
    // Effect completion waits for bounded particle drain, not just its timeline.
    const timeline::Target targets[]={{true,layer}};
    const timeline::Asset sequence={66,100,false,1,0,0,0,0,targets,nullptr,nullptr,nullptr,nullptr};
    effects::LayerDefinition layers[1];layers[0].emitter=true;layers[0].initial.emitter.lifetime=raw(200);
    const effects::Asset effect={77,&sequence,layers,1,9};
    const auto e=effects_pool.spawn(effect,Affine<Fixed>::identity(),1);
    assert(effects_pool.burst(e,300));
    assert(timeline::layer_resolver(effects_pool.layer(e,0))->emitter.pending==256&&effects_pool.stats.dropped_bursts==44);
    playback_wait.begin(e);pending();tasks.advance(raw(1),1);
    effects_pool.prepare(1);director.advance(raw(100),1);effects_pool.observe();effects_pool.advance(raw(100));observe();
    assert(!poll()&&effects_pool.state(e)==effects::State::Draining);
    assert(!effects_pool.burst(e,1));
    effects_pool.advance(raw(100));observe();assert(!poll());effects_pool.advance(raw(100));
    const auto next=effects_pool.spawn(effect,Affine<Fixed>::identity(),1);
    assert(next.index==e.index&&next.generation!=e.generation);
    assert(!effects_pool.burst(e,1));
    assert(poll()&&playback_wait.result==bp::PlaybackResult::Completed);
    effects_pool.reset();
    // Subscriptions start after the currently observed revision, not at play's
    // beginning. They retain later crossings even while the reached branch is
    // suspended in Delay, and preserve cancellation before terminal slot reuse.
    auto repeating=asset;repeating.repeat=true;
    h=director.play(repeating,owner,nullptr,1);director.advance(raw(50),1);
    subscription.begin(h,33,44);subscription.start(director.snapshot(h,33,44));
    assert(subscription.revision==1&&subscription.result==bp::PlaybackResult::Pending);
    assert(tasks.wait_external(8,test_owner(0),1));
    test_set_active(0,false);
    for(unsigned i=0;i<3;++i){director.advance(raw(100),1);observe();}
    assert(subscription.observed==4&&subscription.revision==2);
    tasks.advance(raw(1),1);assert(!poll());
    test_set_active(0,true);tasks.advance(raw(1),1);assert(poll());
    unsigned delivered=1;
    assert(tasks.delay(8,raw(5),test_owner(0),1));
    director.advance(raw(100),1);observe();assert(subscription.observed==5);
    director.stop(h);director.play(asset,owner,nullptr,1);
    assert(subscription.terminal==bp::PlaybackResult::Cancelled&&subscription.observed==5);
    tasks.advance(raw(5),1);assert(poll()); // The reached branch finishes its Delay.
    for(unsigned i=0;i<3;++i){
        subscription.rearm();assert(tasks.wait_external(8,test_owner(0),1));observe();
        assert(!poll()); // A backlog cannot dispatch repeatedly in the same tick.
        tasks.advance(raw(1),1);assert(poll()&&subscription.result==bp::PlaybackResult::Reached);++delivered;
    }
    subscription.rearm();assert(tasks.wait_external(8,test_owner(0),1));observe();tasks.advance(raw(1),1);
    assert(poll()&&subscription.result==bp::PlaybackResult::Cancelled&&delivered==4);
    subscription.active=false;
    // Completion also survives a later invalid-handle snapshot. An already
    // completed play has no future crossings for a newly registered subscriber.
    bp::PlaybackSubscription complete;
    complete.begin(h,33,44);complete.start({bp::PlaybackResult::Pending,0});
    assert(complete.capture({bp::PlaybackResult::Completed,2}));
    complete.capture({});assert(complete.terminal==bp::PlaybackResult::Completed);
    complete.rearm();assert(complete.result==bp::PlaybackResult::Reached&&complete.revision==2);
    complete.rearm();assert(complete.result==bp::PlaybackResult::Completed);
    complete.start({bp::PlaybackResult::Completed,7});assert(complete.result==bp::PlaybackResult::Completed);
    director.cancel_all();bp::playback_observer=nullptr;
    assert(observations>0);
    std::printf("Blueprint playback: exactly-once markers, subscription backlog, terminal slot reuse, typed effect drain, invalid references and lifecycle cancellation passed; frame=%zu, subscription=%zu, continuations=%zu bytes.\n",sizeof(playback_wait),sizeof(subscription),sizeof(tasks));
}
