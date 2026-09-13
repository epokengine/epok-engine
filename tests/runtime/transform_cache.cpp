#include <array>
#include <cassert>
#include <cstdio>
#include "psyqo/fixed-point.hh"
#include "transform_cache.hpp"
#include "collision.hpp"
using Q=psyqo::FixedPoint<12>;
using Matrix=epok::Affine<Q>;
struct Transform{Q position[3]={},rotation[3]={},scale[3]={1.,1.,1.};};
struct Object{Transform transform;int parent=-1;uint32_t generation=1;bool alive=true;};
// Non-diagonal bases exercise inherited shear and non-uniform scale without
// depending on the target-only trig initialization.
static Matrix local(const Transform& t){
    Matrix m;
    for(int k=0;k<3;++k){m.values[k][k]=t.scale[k];m.values[k][3]=t.position[k];m.values[k][(k+1)%3]=t.rotation[k];}
    return m;
}
static bool equal(const Matrix& a,const Matrix& b){for(int r=0;r<3;++r)for(int c=0;c<4;++c)if(a.values[r][c].raw()!=b.values[r][c].raw())return false;return true;}
static void check(const std::array<Object,8>& objects,const std::array<Matrix,8>& worlds,size_t count){
    for(size_t i=0;i<count;++i){
        auto expected=Matrix::identity();int chain[33],depth=0,current=int(i);
        if(objects[i].alive){
            while(current>=0&&size_t(current)<count&&depth<33){chain[depth++]=current;current=objects[current].parent;}
            if(current<0)while(depth)expected=expected.compose(local(objects[chain[--depth]].transform));
        }
        assert(equal(expected,worlds[i]));
    }
}
int main(){
    std::array<Object,8> objects;std::array<Matrix,8> worlds;
    epok::TransformCache<Q,8> cache;
    objects[0].parent=4;objects[1].parent=0;
    objects[4].transform.position[0]=3.;objects[4].transform.scale[1]=2.;objects[0].transform.rotation[0]=.5;
    cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    assert(cache.local_rebuilds==8&&cache.world_rebuilds==8);
    cache.sync(objects,worlds,8,local);assert(!cache.local_rebuilds&&!cache.world_rebuilds);
    objects[0].transform.position[1]=2.;cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    assert(cache.local_rebuilds==1&&cache.world_rebuilds==2);
    objects[0].parent=3;cache.sync(objects,worlds,8,local);check(objects,worlds,8);assert(!cache.local_rebuilds);
    objects[3].parent=1;cache.sync(objects,worlds,8,local);check(objects,worlds,8); // cycle
    objects[3].parent=-1;objects[3].alive=false;cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    objects[3].alive=true;++objects[3].generation;cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    objects[0].parent=7;cache.sync(objects,worlds,4,local);check(objects,worlds,4); // invalid parent after shrinking
    cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    cache.clear();cache.sync(objects,worlds,8,local);check(objects,worlds,8);assert(cache.local_rebuilds==8);
    for(int step=0;step<80;++step){
        int i=(step*7)%8;
        objects[i].transform.position[step%3]=Q(step*71-2000,Q::RAW);
        objects[i].transform.scale[(step+1)%3]=Q(4096+step*17,Q::RAW);
        objects[i].transform.rotation[(step+2)%3]=Q(step*31-500,Q::RAW);
        if(step%5==0)objects[i].parent=(step%11)-2;
        cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    }
    epok::CollisionWorld<Q,8> collisions;epok::ColliderT<Q> box;box.enabled=true;
    auto m=Matrix::identity();
    assert(collisions.set_cached(0,box,m,1));
    collisions.begin_sync();assert(!collisions.set_cached(0,box,m,1));assert(collisions.bounds(0));
    box.center[0]=2.;assert(collisions.set_cached(0,box,m,1));assert(collisions.bounds(0)->min[0]==1.5);
    m.values[0][3]=4.;assert(collisions.set_cached(0,box,m,2));assert(collisions.bounds(0)->min[0]==5.5);
    box.half_extents[0]=1.;assert(collisions.set_cached(0,box,m,2));assert(collisions.bounds(0)->min[0]==5.);
    box.layer=2;assert(!collisions.set_cached(0,box,m,2,true,7));
    Q origin[3]={},delta[3]={10.,0.,0.};assert(!collisions.raycast(origin,delta,1));assert(collisions.raycast(origin,delta,2).generation==7);
    assert(!collisions.set_cached(0,box,m,2,false));assert(!collisions.bounds(0));
    assert(!collisions.set_cached(0,box,m,2,true,8));assert(collisions.raycast(origin,delta,2).generation==8);
    collisions.clear();assert(collisions.set_cached(0,box,m,2));
    collisions.set(0,box,m);assert(collisions.set_cached(0,box,m,2));
    std::puts("Transform cache hierarchy, direct writes, lifecycle and cached collision tests passed.");
}
