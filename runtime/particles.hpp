#pragma once
#include "epok.hpp"
#include "affine.hpp"
#include "timeline_runtime.hpp"
#include <array>
namespace epok {
// Scene emitters and effect layers share these exact 256 particle records.
class ParticlePool {
    struct Source {timeline::BoundTarget owner;ParticleEmitter* emitter=nullptr;const Affine<Fixed>* world=nullptr;};
    Source sources[64];
    uint16_t source_count=0;
    Fixed step=0.0;
    static void add(uint32_t& value,uint32_t amount=1){value=UINT32_MAX-value<amount?UINT32_MAX:value+amount;}
    static uint32_t requests(ParticleEmitter& e,Fixed dt){
        uint32_t due=e.pending;e.pending=0;
        if(!e.seeded){e.seeded=true;e.random_state=e.seed?e.seed:1;}
        if(e.playing){if(!e.started){e.started=true;if(!e.continuous)due+=e.burst_count;}
            if(e.continuous&&e.rate.raw()>0){e.accumulator+=e.rate*dt;const uint32_t amount=uint32_t(e.accumulator.raw()/4096);e.accumulator-=Fixed(int32_t(amount*4096),Fixed::RAW);due+=amount;}}
        return due;
    }
public:
    static constexpr size_t capacity=256,emitter_capacity=64;
    struct Particle {
        bool alive=false,local=false;uint16_t source=0;
        timeline::BoundTarget owner;
        Fixed age=0.0,lifetime=1.0;
        Fixed position[3]={},velocity[3]={},gravity[3]={};Fixed start_size=1.0,end_size=0.0;
        uint8_t start_color[3]={},end_color[3]={};Sprite sprite;uint16_t frames=1,columns=1;Fixed frame_duration=0.1;
    };
    std::array<Particle,capacity> particles;
    void clear(){for(auto& p:particles)p.alive=false;source_count=0;particle_stats={};}
    void remove_owner(size_t owner){for(auto& p:particles)if(p.alive&&!p.owner.internal&&p.owner.data_slot().index==owner){p.alive=false;if(particle_stats.alive)--particle_stats.alive;}}
    void remove_layer(EffectLayerHandle owner){for(auto& p:particles)if(p.alive&&p.owner.same(owner)){p.alive=false;if(particle_stats.alive)--particle_stats.alive;}}
    void begin(Fixed dt){source_count=0;step=dt.raw()>410?Fixed(410,Fixed::RAW):dt;}
    bool emitter(timeline::BoundTarget owner,ParticleEmitter& emitter,const Affine<Fixed>& world){
        if(!emitter.enabled||!owner.valid())return false;
        if(source_count==emitter_capacity){add(particle_stats.dropped_emitters);if(step.raw()>0&&owner.active())add(particle_stats.dropped,requests(emitter,step));return false;}
        sources[source_count++]={owner,&emitter,&world};return true;
    }
    template<size_t N>void scene_emitters(std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count){
        if(count>N)count=N;
        for(size_t i=0;i<count;++i)emitter(DataHandle{uint16_t(i),objects[i].generation},objects[i].particle_emitter,world[i]);
    }
    void advance(){
        if(step.raw()<=0)return;
        uint16_t per_owner[emitter_capacity]={};uint32_t alive=0;
        for(auto& p:particles){if(!p.alive)continue;
            if(!p.owner.valid()){p.alive=false;continue;}
            if(p.source>=source_count||!p.owner.same(sources[p.source].owner)){
                p.source=source_count;for(uint16_t i=0;i<source_count;++i)if(p.owner.same(sources[i].owner)){p.source=i;break;}
            }
            if(p.source>=source_count||!sources[p.source].emitter->enabled){p.alive=false;continue;}
            // Existing scene particles age while their inactive owner hides
            // them. Effect pause freezes the whole effect, including particles.
            if(p.owner.internal&&!p.owner.active()){++per_owner[p.source];++alive;continue;}
            p.age+=step;if(p.age>=p.lifetime){p.alive=false;continue;}
            for(int c=0;c<3;++c){p.velocity[c]+=p.gravity[c]*step;p.position[c]+=p.velocity[c]*step;}++per_owner[p.source];++alive;
        }
        for(uint16_t i=0;i<source_count;++i){auto& source=sources[i];auto& e=*source.emitter;if(!source.owner.active())continue;
            const uint32_t due=requests(e,step);auto limit=e.max_particles>128?128:e.max_particles;
            uint32_t room=limit>per_owner[i]?limit-per_owner[i]:0;if(room>capacity-alive)room=capacity-alive;
            uint32_t accepted=due<room?due:room;add(particle_stats.dropped,due-accepted);
            for(auto& p:particles){if(!accepted)break;if(p.alive)continue;--accepted;++alive;add(particle_stats.spawned);
                p=Particle{};p.alive=true;p.source=i;p.owner=source.owner;p.local=e.local_space;p.lifetime=e.lifetime.raw()>0?e.lifetime:Fixed(1,Fixed::RAW);p.start_size=e.start_size;p.end_size=e.end_size;p.sprite=e.sprite;p.frames=e.frames?e.frames:1;p.columns=e.frame_columns?e.frame_columns:1;p.frame_duration=e.frame_duration.raw()>0?e.frame_duration:Fixed(1,Fixed::RAW);
                Fixed local_velocity[3];for(int c=0;c<3;++c){e.random_state=e.random_state*1664525u+1013904223u;int32_t random=int32_t((e.random_state>>16)&65535)-32768;local_velocity[c]=e.velocity[c]+Fixed(random/8,Fixed::RAW)*e.spread[c];p.gravity[c]=e.gravity[c];p.start_color[c]=e.start_color[c];p.end_color[c]=e.end_color[c];}
                for(int r=0;r<3;++r){p.position[r]=p.local?Fixed(0,Fixed::RAW):source.world->values[r][3];if(p.local)p.velocity[r]=local_velocity[r];else {p.velocity[r]=0.0;for(int c=0;c<3;++c)p.velocity[r]+=source.world->values[r][c]*local_velocity[c];}}
            }
        }
        particle_stats.alive=alive;if(alive>particle_stats.peak)particle_stats.peak=alive;
    }
    template<size_t N>void advance(std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count,Fixed dt){begin(dt);scene_emitters(objects,world,count);advance();}
    template<class Emit>void each_all(Emit emit)const{
        for(const auto& p:particles){if(!p.alive||!p.owner.valid()||!p.owner.visible()||p.source>=source_count||!p.owner.same(sources[p.source].owner))continue;
            auto sprite=p.sprite;sprite.enabled=true;Fixed t=p.age/p.lifetime;auto size=p.start_size+(p.end_size-p.start_size)*t;
            for(int c=0;c<2;++c)sprite.size[c]=sprite.size[c]*size;if(size.raw()<=0)continue;
            for(int c=0;c<3;++c){int level=int(p.start_color[c])+int64_t(int(p.end_color[c])-int(p.start_color[c]))*t.raw()/4096;sprite.color[c]=uint8_t(int(sprite.color[c])*level/255);}
            if(p.frames>1){uint32_t f=uint32_t(p.age.raw()/p.frame_duration.raw());if(f>=p.frames)f=p.frames-1;sprite.region[0]+=uint16_t(f%p.columns)*sprite.region[2];sprite.region[1]+=uint16_t(f/p.columns)*sprite.region[3];}
            auto matrix=Affine<Fixed>::identity();
            if(p.local)matrix=sources[p.source].world->translated(p.position);
            else for(int c=0;c<3;++c)matrix.values[c][3]=p.position[c];
            emit(p.owner,sprite,matrix);
        }
    }
    template<size_t N,class Emit>void each(const std::array<Affine<Fixed>,N>&,size_t count,Emit emit)const{
        each_all([&](timeline::BoundTarget owner,const Sprite& sprite,const Affine<Fixed>& matrix){if(!owner.internal&&owner.data_slot().index<count)emit(owner.data_slot().index,sprite,matrix);});
    }
};
}
