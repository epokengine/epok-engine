#include <cassert>
#include <cstdio>
#include "../../runtime/particles.hpp"
namespace epok {
bool is_active(const Entity* e){return e&&e->alive&&e->active;}
// Host stand-in for the runtime slot table: each section registers its array.
inline const Entity* slot_objects=nullptr;
inline size_t slot_count=0;
Entity* EntityHandle::get()const{return slot_objects&&index<slot_count&&slot_objects[index].alive&&slot_objects[index].generation==generation?const_cast<Entity*>(&slot_objects[index]):nullptr;}
bool is_active_slot(size_t i){return slot_objects&&is_active(&slot_objects[i]);}
}
using namespace epok;
static void animator_events(){
    const SpriteFrame frames[]={{{0,0,16,16},0.1,1},{{16,0,16,16},0.2,2},{{32,0,16,16},0.1,3}};
    const SpriteClip clips[]={{frames,3,false,"Once"},{frames,3,true,"Loop"}};
    SpriteAnimator a;a.enabled=true;a.clips=clips;a.clip_count=2;Sprite sprite;
    assert(a.play(0));uint16_t event=0;assert(a.poll_event(event)&&event==1);
    a.advance(0.35,sprite);assert(a.frame==2&&sprite.region[0]==32);assert(a.poll_event(event)&&event==2);assert(a.poll_event(event)&&event==3);assert(!a.poll_event(event));
    a.advance(0.1,sprite);assert(!a.playing&&a.take_completion());assert(!a.take_completion());assert(a.frame==2);
    assert(a.play(1));a.advance(0.45,sprite);assert(a.frame==0&&a.playing);assert(!a.take_completion());
    a.pause();a.advance(0.2,sprite);assert(a.frame==0);a.resume();a.advance(0.2,sprite);assert(a.frame==1);
    assert(!a.play(20));assert(a.clip==1);
    // Slow frames preserve FIFO events up to an explicit bounded limit.
    a.play(1);for(int i=0;i<30;++i)a.advance(0.1,sprite);assert(a.event_count==16&&a.dropped_events>0);
}
static void particle_limits_and_reuse(){
    std::array<Entity,4> objects{};slot_objects=objects.data();slot_count=objects.size();std::array<Affine<Fixed>,4> world{};
    for(size_t i=0;i<objects.size();++i){world[i]=Affine<Fixed>::identity();auto& e=objects[i].particle_emitter;e.enabled=true;e.continuous=false;e.max_particles=128;e.burst_count=128;e.seed=17;e.lifetime=0.2;e.spread[0]=e.spread[1]=e.spread[2]=0.0;}
    ParticlePool pool;pool.clear();pool.advance(objects,world,4,0.05);assert(particle_stats.alive==256);assert(particle_stats.dropped==256);
    auto velocity=pool.particles[0].velocity[1].raw();assert(velocity==4096);
    for(int i=0;i<5;++i)pool.advance(objects,world,4,0.05);assert(particle_stats.alive==0);
    objects[0].particle_emitter.burst(7);pool.advance(objects,world,4,0.05);assert(particle_stats.alive==7);
    ++objects[0].generation;pool.advance(objects,world,4,0.05);assert(particle_stats.alive==0); // reused owner cannot inherit old particles
    objects[0].active=false;objects[0].particle_emitter.burst(7);pool.advance(objects,world,4,0.05);assert(particle_stats.alive==0);
}
static void world_local_and_interpolation(){
    std::array<Entity,2> objects{};slot_objects=objects.data();slot_count=objects.size();std::array<Affine<Fixed>,2> world{};
    for(int i=0;i<2;++i){world[i]=Affine<Fixed>::identity();world[i].values[0][3]=5.0;auto& e=objects[i].particle_emitter;e.enabled=true;e.continuous=false;e.burst_count=1;e.lifetime=1.0;e.local_space=i==1;e.start_size=1.0;e.end_size=0.0;e.sprite.size[0]=e.sprite.size[1]=1.0;e.spread[0]=e.spread[1]=e.spread[2]=0.0;}
    ParticlePool pool;pool.clear();pool.advance(objects,world,2,0.05);assert(pool.particles[0].position[0].raw()==5*4096);assert(pool.particles[1].position[0].raw()==0);
    world[0].values[0][3]=world[1].values[0][3]=10.0;
    for(int i=0;i<5;++i)pool.advance(objects,world,2,0.1);
    int draws=0;pool.each(world,2,[&](size_t owner,const Sprite& sprite,const Affine<Fixed>& matrix){assert(sprite.size[0].raw()>2000&&sprite.size[0].raw()<2100);assert(matrix.values[0][3].raw()==int(owner==0?5:10)*4096);++draws;});assert(draws==2);
    pool.remove_owner(1);assert(particle_stats.alive==1);pool.clear();assert(particle_stats.alive==0);
}
static void translated_basis_preserves_q12_composition(){
    for(int sample=0;sample<32;++sample){
        auto parent=Affine<Fixed>::identity(),local=Affine<Fixed>::identity();
        Fixed offset[3];
        for(int r=0;r<3;++r){
            offset[r]=Fixed((sample-16)*(r+1)*173,Fixed::RAW);local.values[r][3]=offset[r];
            for(int c=0;c<4;++c)parent.values[r][c]=Fixed((sample-11)*(r+2)*(c+1)*71,Fixed::RAW);
        }
        const auto expected=parent.compose(local),actual=parent.translated(offset);
        for(int r=0;r<3;++r)for(int c=0;c<4;++c)assert(actual.values[r][c].raw()==expected.values[r][c].raw());
    }
}
static void manual_bursts_honor_seed(){
    std::array<Entity,2> objects{};slot_objects=objects.data();slot_count=objects.size();std::array<Affine<Fixed>,2> world{};
    for(int i=0;i<2;++i){world[i]=Affine<Fixed>::identity();auto& e=objects[i].particle_emitter;e.enabled=true;e.playing=false;e.seed=uint32_t(17+i);e.spread[0]=1.0;e.burst(1);}
    ParticlePool pool;pool.clear();pool.advance(objects,world,2,0.05);assert(pool.particles[0].velocity[0].raw()!=pool.particles[1].velocity[0].raw());
    assert(objects[0].particle_emitter.seeded&&objects[1].particle_emitter.seeded);
}
static void wrapped_uv_and_fog(){
    const UvVertex input[]={{{0,0,4096},{255,0,0},{0,0}},{{4096,0,4096},{0,255,0},{4096,0}},{{0,4096,4096},{0,0,255},{0,4096}}};
    for(int sign=-1;sign<=1;sign+=2){int32_t speed[]={sign*1024,sign*1638};int64_t area=0;int count=0;
        scroll_triangle(input,speed,60,[&](const UvVertex& a,const UvVertex& b,const UvVertex& c){for(const auto* p:{&a,&b,&c})for(int axis=0;axis<2;++axis)assert(p->uv[axis]>=0&&p->uv[axis]<=4096);int64_t triangle=int64_t(b.camera[0]-a.camera[0])*(c.camera[1]-a.camera[1])-int64_t(b.camera[1]-a.camera[1])*(c.camera[0]-a.camera[0]);area+=triangle<0?-triangle:triangle;++count;});
        assert(area>4096ll*4096-16384&&area<4096ll*4096+16384);assert(count<=10);
    }
    struct Color{uint8_t r,g,b;};fog_environment={true,2*4096,10*4096,{255,0,0}};auto color=fog_color(Color{0,100,200},6*4096);assert(color.r==128&&color.g==50&&color.b==100);fog_environment={};
}
int main(){animator_events();particle_limits_and_reuse();world_local_and_interpolation();translated_basis_preserves_q12_composition();manual_bursts_honor_seed();wrapped_uv_and_fog();std::puts("Runtime sprite events, particle pools, UV seams and fog tests passed.");}
