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
    uint32_t random=19231;
    for(int sample=0;sample<10000;++sample){
        Matrix a,b,expected;
        for(int r=0;r<3;++r)for(int c=0;c<4;++c){
            random=random*1664525u+1013904223u;a.values[r][c]=Q(int32_t(random%65536)-32768,Q::RAW);
            random=random*1664525u+1013904223u;b.values[r][c]=Q(int32_t(random%65536)-32768,Q::RAW);
        }
        if(sample%2==0)for(int r=0;r<3;++r)for(int c=0;c<3;++c)if(r!=c)b.values[r][c]=0.;
        if(sample%4==0)for(int k=0;k<3;++k)b.values[k][k]=1.;
        if(sample%3==0)for(int k=0;k<3;++k)b.values[k][3]=0.;
        for(int r=0;r<3;++r)for(int c=0;c<4;++c){
            if(c==3)expected.values[r][c]=a.values[r][3];
            for(int k=0;k<3;++k)expected.values[r][c]+=a.values[r][k]*b.values[k][c];
        }
        assert(equal(a.compose(b),expected));
        // Shared trig across columns must not reassociate fixed-point math.
        for(int columns=3;columns<=4;++columns)for(int reverse=0;reverse<2;++reverse){
            auto rotated=a,reference=a;
            Q sine[3]={b.values[0][0],b.values[1][1],b.values[2][2]};
            Q cosine[3]={b.values[0][1],b.values[1][2],b.values[2][0]};
            for(int n=0;n<3;++n){int axis=reverse?2-n:n;rotated.rotate_rows(axis,sine[axis],cosine[axis],columns);}
            for(int col=0;col<columns;++col)for(int n=0;n<3;++n){
                int axis=reverse?2-n:n,x=(axis+1)%3,y=(axis+2)%3;
                auto first=reference.values[x][col]*cosine[axis]-reference.values[y][col]*sine[axis];
                reference.values[y][col]=reference.values[x][col]*sine[axis]+reference.values[y][col]*cosine[axis];
                reference.values[x][col]=first;
            }
            assert(equal(rotated,reference));
        }
    }
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
    // Physics-only synchronization must not consume parent changes before an
    // unselected visual child sees them on the following full synchronization.
    objects={};objects[1].parent=0;objects[2].parent=1;
    cache.clear();cache.sync(objects,worlds,8,local);
    bool selected[8]={true,false,false,true,false,false,false,false};
    objects[0].transform.position[0]=7.;objects[3].transform.position[1]=2.;
    objects[6].transform.position[2]=9.;
    cache.sync(objects,worlds,8,local,selected);
    assert(cache.local_rebuilds==2&&cache.world_rebuilds==2);
    assert(worlds[1].values[0][3]==0.);
    cache.sync(objects,worlds,8,local,selected);assert(cache.world_rebuilds==0);
    cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    assert(cache.local_rebuilds==1&&cache.world_rebuilds==3);
    selected[1]=selected[2]=true;
    objects[0].transform.position[2]=3.;
    cache.sync_root(objects[0],worlds[0],0,local);
    objects[0].transform.position[2]=4.;
    cache.sync_root(objects[0],worlds[0],0,local);
    cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    for(int step=0;step<100;++step){
        objects[step%8].transform.position[step%3]=Q(step*39,Q::RAW);
        cache.sync(objects,worlds,8,local,selected);
        cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    }
    objects[0].parent=2; // Selected closure also covers cycles.
    cache.sync(objects,worlds,8,local,selected);cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    cache.sync(objects,worlds,4,local,selected);cache.sync(objects,worlds,8,local);check(objects,worlds,8);
    {
        std::array<Object,1> moving;std::array<Matrix,1> matrices;
        epok::TransformCache<Q,1> moving_cache;unsigned basis_builds=0;
        auto counted=[&](const Transform& t){++basis_builds;return local(t);};
        moving[0].transform.rotation[1]=.25;moving[0].transform.scale[2]=2.;
        moving_cache.sync(moving,matrices,1,counted);assert(basis_builds==1);
        moving[0].transform.position[2]=4.;moving_cache.sync(moving,matrices,1,counted);
        assert(basis_builds==1&&equal(matrices[0],local(moving[0].transform)));
        moving[0].transform.position[0]=-3.;moving_cache.sync_root(moving[0],matrices[0],0,counted);
        assert(basis_builds==1&&equal(matrices[0],local(moving[0].transform)));
        moving[0].transform.rotation[0]=.5;moving_cache.sync_root(moving[0],matrices[0],0,counted);
        assert(basis_builds==2&&equal(matrices[0],local(moving[0].transform)));
        moving[0].transform.scale[0]=3.;moving_cache.sync(moving,matrices,1,counted);
        assert(basis_builds==3&&equal(matrices[0],local(moving[0].transform)));
    }
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
