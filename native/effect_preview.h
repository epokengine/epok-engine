#pragma once
#include <stdint.h>

// Host-only C ABI for already compiled data. This header is never staged to PSX.
// Every scalar is explicitly sized; no serialized/runtime entity handles cross it.
struct PreviewSprite {
    int32_t texture,region[4],size[2],pivot[2],color[3];
    uint32_t enabled,flip_x,flip_y,orientation,unlit,blend;
    int32_t depth_bias;
};
struct PreviewLayer {
    PreviewSprite sprite;
    uint32_t slot,emitter,enabled,playing,continuous,local_space,seed,burst,max_particles;
    int32_t position[3],velocity[3],spread[3],gravity[3];
    int32_t rate,lifetime,start_size,end_size,start_color[3],end_color[3];
    uint32_t frames,columns;
    int32_t frame_ticks;
};
struct PreviewKey {int32_t tick,value;};
struct PreviewTrack {
    uint64_t property;
    uint32_t slot,field,channels,additive,restore,interpolation;
    uint32_t lengths[4];
    PreviewKey keys[4][4];
};
struct PreviewEvent {uint32_t slot,function,count,idempotent;};
struct PreviewSignal {int32_t tick;uint32_t index,event;};
struct PreviewQuad {uint32_t layer;PreviewSprite sprite;int32_t world[12];};
struct PreviewParticle {uint32_t index,layer;int32_t age,lifetime,position[3],velocity[3];};
struct PreviewStats {uint32_t state,tick,alive,spawned,dropped,peak,events,markers,skipped_targets,skipped_events,dropped_emitters,diagnostics_dropped;};
extern "C" {
void* epok_preview_create();
void epok_preview_destroy(void*);
uint32_t epok_preview_layer(void*,const PreviewLayer*);
uint32_t epok_preview_track(void*,const PreviewTrack*);
uint32_t epok_preview_event(void*,const PreviewEvent*);
uint32_t epok_preview_start(void*,uint64_t,uint64_t,int32_t,uint32_t,uint32_t,uint32_t,uint32_t,const uint64_t*,uint32_t,const PreviewSignal*,uint32_t);
void epok_preview_step(void*,int32_t,uint32_t);
void epok_preview_stats(void*,PreviewStats*);
uint32_t epok_preview_quads(void*,PreviewQuad*,uint32_t);
uint32_t epok_preview_particles(void*,PreviewParticle*,uint32_t);
uint32_t epok_preview_abi(uint32_t);
}
