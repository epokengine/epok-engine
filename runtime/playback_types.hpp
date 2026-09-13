#pragma once
#include <stdint.h>

// The existing playback identities, shared with Blueprint signatures without
// including the directors. Runtime slots/generations never enter source assets.
namespace epok::timeline {
struct Handle {uint16_t index=0xffff;uint32_t generation=0;};
struct Asset;
}
namespace epok::effects {
struct Handle {uint16_t index=0xffff;uint32_t generation=0;};
struct Asset;
}

namespace epok::bp {
// One runtime-owned observation pass over existing behaviour frames. This is
// not a callback registry: observation may only capture results and signal the
// existing continuation slots, never execute a graph or start playback.
inline void (*playback_observer)()=nullptr;
inline void observe_playback(){
    static bool observing=false;
    if(!playback_observer||observing)return;
    observing=true;playback_observer();observing=false;
}
enum class PlaybackResult:uint8_t {Pending,Reached,Completed,Cancelled};
struct PlaybackSnapshot {
    PlaybackResult result=PlaybackResult::Cancelled;
    uint32_t revision=0;
};
// Captured in the graph's existing latent frame. The discriminator keeps effect
// completion (including particle drain) distinct from sequence completion.
struct PlaybackWait {
    union {timeline::Handle sequence;effects::Handle effect;};
    uint64_t asset=0,marker=0;
    uint32_t revision=0;
    PlaybackResult result=PlaybackResult::Pending;
    bool effect_completion=false;
    PlaybackWait():sequence{}{}
    void begin(timeline::Handle target,uint64_t expected_asset=0,uint64_t marker_id=0,uint32_t after=0){
        sequence=target;asset=expected_asset;marker=marker_id;revision=after;
        effect_completion=false;result=PlaybackResult::Pending;
    }
    void begin(effects::Handle target){
        effect=target;asset=marker=0;revision=0;
        effect_completion=true;result=PlaybackResult::Pending;
    }
    bool capture(PlaybackSnapshot snapshot){
        if(result!=PlaybackResult::Pending)return true;
        if(marker&&snapshot.revision>revision){result=PlaybackResult::Reached;++revision;}
        else if(snapshot.result!=PlaybackResult::Pending)result=snapshot.result;
        return result!=PlaybackResult::Pending;
    }
};
// A repeating marker listener lives in its graph frame, using the same external
// continuation slot as Delay/Wait. Revisions compact the backlog without a queue.
// Observation continues while a reached branch is suspended or already ready,
// so terminal slot reuse cannot erase crossings that have not been delivered.
struct PlaybackSubscription:PlaybackWait {
    uint32_t observed=0;
    PlaybackResult terminal=PlaybackResult::Pending;
    bool active=false;
    void start(PlaybackSnapshot snapshot){
        observed=revision=snapshot.revision;terminal=snapshot.result;
        active=true;result=PlaybackResult::Pending;resolve();
    }
    bool resolve(){
        if(result==PlaybackResult::Pending){
            if(observed>revision){++revision;result=PlaybackResult::Reached;}
            else if(terminal!=PlaybackResult::Pending)result=terminal;
        }
        return result!=PlaybackResult::Pending;
    }
    bool capture(PlaybackSnapshot snapshot){
        if(terminal==PlaybackResult::Pending){
            if(snapshot.revision>observed)observed=snapshot.revision;
            terminal=snapshot.result;
        }
        return resolve();
    }
    void rearm(){result=PlaybackResult::Pending;resolve();}
};
}
