#define EPOK_WORLD_GAMEPLAY 1
#define EPOK_TEST_ACTOR_SLOTS 12
#define EPOK_TEST_ACTOR_POOL 16
#define EPOK_TEST_REGISTRY_SLOTS 64
#define EPOK_TEST_EXTRA_CLASSES \
 {ProjectilePool3DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<ProjectilePool3DComponent>,&object_destruct,sizeof(ProjectilePool3DComponent),alignof(ProjectilePool3DComponent),&ObjectPool<ProjectilePool3DComponent,2>::acquire,&ObjectPool<ProjectilePool3DComponent,2>::release}, \
 {Sprite3DComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<Sprite3DComponent>,&object_destruct,sizeof(Sprite3DComponent),alignof(Sprite3DComponent),&ObjectPool<Sprite3DComponent,8>::acquire,&ObjectPool<Sprite3DComponent,8>::release},
#include "actor_scene_fixture.hpp"
#include <cstdio>
using namespace epok;
namespace epok {
static uint32_t batches=0,query_count=0,hit_bits=0,observed_mask=0;
static const ActorData* observed_ignore=nullptr;
static bool observed_triggers=false;
void SpatialQueryBatch::raycast_batch(const RaycastQuery* rays,SpatialHit* hits,size_t count,uint32_t mask,const ActorData* ignore,bool triggers)const{
    ++batches;query_count=uint32_t(count);observed_mask=mask;observed_ignore=ignore;observed_triggers=triggers;
    for(size_t i=0;i<count;++i){hits[i]={};if(hit_bits&(1u<<i)){
        hits[i].entity=10;hits[i].generation=entities[10].generation;
        for(unsigned axis=0;axis<3;++axis)hits[i].point[axis]=rays[i].origin[axis]+rays[i].displacement[axis]*Fixed(.5);
    }}
}
}
static ProjectileStep4 script_step(ProjectilePool3DComponent& pool,uint32_t first,uint32_t count,Fixed dt,ProjectileTraceFilter filter){
    const auto batch=pool.step(first,count,dt,filter);
    ProjectileStep4 result;
    if(!batch.sequence){result.first.state=batch.state;return result;}
    result.first=pool.result(batch,0);
    if(batch.count>1)result.second=pool.result(batch,1);
    if(batch.count>2)result.third=pool.result(batch,2);
    if(batch.count>3)result.fourth=pool.result(batch,3);
    return result;
}
int main(){
    test_reset_scene();for(auto& data:entities)data.parent=-1;
    auto* owner=test_registry.resolve<Actor3D>(test_owner(0));
    auto* pool=test_level.add_component<ProjectilePool3DComponent>(*owner,"Projectiles");assert(pool);
    auto* context=test_level.add_component<SpatialQueryContextComponent>(*owner,"Queries");assert(context);
    for(uint32_t i=0;i<8;++i){
        auto* actor=test_registry.resolve<Actor3D>(test_owner(i+1));
        assert(test_level.add_component<Sprite3DComponent>(*actor,"Sprite"));
        assert(pool->bind(i,actor->id())==ProjectileResult::Applied);
    }
    pool->sprite_grid.enabled=true;
    const auto spawn=ProjectileLibrary::spawn({1.0,2.0,3.0},{22.0,0.0,-7.0},1.7,42);
    ProjectileHandle tokens[8];
    for(uint32_t i=0;i<8;++i){auto result=pool->launch(0,8,spawn);assert(result.result==ProjectileResult::Applied);tokens[i]=result.handle;assert((result.handle.ticket&7)==i);}
    assert(pool->active_count()==8&&pool->launch(0,8,spawn).result==ProjectileResult::Full);
    assert(pool->bind(0,{} )==ProjectileResult::Busy);
    const auto filter=ProjectileLibrary::filter(3,test_owner(11),false);
    const auto dt=Fixed(68,Fixed::RAW);
    auto result=script_step(*pool,0,4,dt,filter);
    assert(batches==1&&query_count==4&&observed_mask==3&&observed_ignore==&entities[11]&&!observed_triggers);
    assert(result.first.state==ProjectileStepState::Moved&&result.fourth.tag==42&&result.third.handle.ticket==tokens[2].ticket);
    assert(result.first.point.x.raw()==4096+22*68&&result.first.point.z.raw()==3*4096-7*68);
    // Whole-group collision results survive reset in an impact/death callback.
    hit_bits=15;const auto impacts=script_step(*pool,0,4,dt,filter);
    assert(impacts.first.state==ProjectileStepState::Hit&&impacts.fourth.target==test_owner(10));
    assert(!entities[1].sprite.enabled&&pool->active_count()==4);
    pool->reset();assert(pool->active_count()==0&&impacts.fourth.state==ProjectileStepState::Hit);
    assert(pool->snapshot(tokens[0]).valid==false&&pool->stop(tokens[7])==ProjectileResult::InvalidHandle);
    for(unsigned i=1;i<=8;++i)assert(!entities[i].sprite.enabled);
    // Short TTL still performs the complete positive simulation step.
    hit_bits=0;auto short_spawn=spawn;short_spawn.lifetime=Fixed(1,Fixed::RAW);
    const auto short_shot=pool->launch(0,1,short_spawn);result=script_step(*pool,0,1,dt,filter);
    assert(result.first.state==ProjectileStepState::Expired&&result.second.state==ProjectileStepState::Empty);
    assert(result.first.point.x.raw()==4096+22*68&&!entities[1].sprite.enabled);
    assert(pool->stop(short_shot.handle)==ProjectileResult::Unchanged);
    // Independent pools cannot accept each other's ticket, even with equal slot/generation.
    auto* other_owner=test_registry.resolve<Actor3D>(test_owner(11));
    auto* other=test_level.add_component<ProjectilePool3DComponent>(*other_owner,"Other pool");assert(other);
    assert(other->stop(short_shot.handle)==ProjectileResult::InvalidHandle);
    // Rejected input does not consume a live shot or query the world.
    const auto current=pool->launch(0,1,spawn);const auto before=batches;
    assert(script_step(*pool,0,5,dt,filter).first.state==ProjectileStepState::InvalidRange);
    assert(script_step(*pool,0,1,Fixed(-1,Fixed::RAW),filter).first.state==ProjectileStepState::InvalidDelta);
    assert(script_step(*pool,0,1,dt,{1,{0xfffe,1},false}).first.state==ProjectileStepState::InvalidFilter);
    pool->sprite_grid.frames=0;assert(script_step(*pool,0,1,dt,filter).first.state==ProjectileStepState::InvalidGrid);pool->sprite_grid.frames=4;
    pool->query_context={0xfffe,1};assert(script_step(*pool,0,1,dt,filter).first.state==ProjectileStepState::InvalidContext);pool->query_context={};
    assert(batches==before&&pool->snapshot(current.handle).active);
    // Authored binding edits and data generation reuse cannot move a replacement actor.
    pool->visual0=test_owner(2);result=script_step(*pool,0,1,dt,filter);
    assert(result.first.state==ProjectileStepState::InvalidBinding&&!entities[1].sprite.enabled);
    pool->visual0=test_owner(1);pool->reset();
    auto changed=pool->launch(0,1,spawn);assert(changed.result==ProjectileResult::Applied);
    ++entities[1].generation;result=script_step(*pool,0,1,dt,filter);
    assert(result.first.state==ProjectileStepState::InvalidBinding);
    // Attached/colliding visuals are explicitly incompatible with world linear motion.
    entities[1].parent=0;assert(pool->launch(0,1,spawn).result==ProjectileResult::InvalidBinding);entities[1].parent=-1;
    entities[1].collider.enabled=true;assert(pool->launch(0,1,spawn).result==ProjectileResult::InvalidBinding);entities[1].collider.enabled=false;
    pool->visual1=pool->visual0;assert(pool->launch(0,1,spawn).result==ProjectileResult::InvalidBinding);pool->visual1=test_owner(2);
    changed=pool->launch(0,1,spawn);assert(changed.result==ProjectileResult::Applied);
    test_set_active(1,false);assert(script_step(*pool,0,1,dt,filter).first.state==ProjectileStepState::InvalidBinding);test_set_active(1,true);
    // Exact Q12 displacement/TTL and tick-grid animation over repeated reuse.
    uint32_t comparisons=0;
    for(uint32_t round=0;round<60;++round){
        pool->reset();const auto shot=pool->launch(0,1,spawn);assert(shot.result==ProjectileResult::Applied);
        int32_t x=4096,z=3*4096,life=Fixed(1.7).raw();
        for(unsigned tick=0;tick<120&&life>0;++tick){
            x+=22*68;z-=7*68;life-=68;const auto stepped=script_step(*pool,0,1,dt,filter);
            assert(stepped.first.point.x.raw()==x&&stepped.first.point.z.raw()==z);
            assert(entities[1].sprite.region[0]==((tick/2)%4)*32);
            assert(stepped.first.state==(life>0?ProjectileStepState::Moved:ProjectileStepState::Expired));++comparisons;
        }
    }
    puts("projectile3d lifecycle, batches and exact legacy motion passed");printf("%u exact motion checkpoints\n",comparisons);
}
