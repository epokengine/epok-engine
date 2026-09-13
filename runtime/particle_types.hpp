#pragma once
#include "sprite_types.hpp"
namespace epok {
struct ParticleEmitter {
    bool enabled=false,playing=true,continuous=true;Fixed rate=8.0;uint16_t burst_count=8,max_particles=32;
    Fixed lifetime=1.0,velocity[3]={0.0,1.0,0.0},spread[3]={0.3,0.2,0.3},gravity[3]={0.0,-0.2,0.0};
    Fixed start_size=0.25,end_size=0.05;uint8_t start_color[3]={255,153,38},end_color[3]={38,13,0};
    bool local_space=false;uint32_t seed=1;Sprite sprite;uint16_t frames=1,frame_columns=1;Fixed frame_duration=0.1;
    uint16_t pending=0;bool started=false,seeded=false;Fixed accumulator=0.0;uint32_t random_state=0;
    // Requests are consumed at the next simulation tick and saturate at the global pool size.
    void burst(uint16_t count=0){uint32_t sum=pending+(count?count:burst_count);pending=uint16_t(sum>256?256:sum);}
    void play(){playing=true;started=false;seeded=false;}void stop(){playing=false;accumulator=0.0;}
};
struct ParticleStats {uint32_t alive=0,spawned=0,dropped=0,peak=0,dropped_emitters=0;};
inline ParticleStats particle_stats;
}
