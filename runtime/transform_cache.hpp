#pragma once
#include "affine.hpp"
#include <stddef.h>
#include <stdint.h>

namespace epok {
// Scripts retain direct Transform writes. Compare their nine scalar inputs at
// every synchronization boundary, but only rebuild affected ancestor chains.
template<class Number,size_t Capacity> class TransformCache {
    struct Entry {
        int32_t pose[9]={};
        Affine<Number> local{};
        int parent=-1;
        uint32_t generation=0,revision=0;
        bool initialized=false,alive=false;
    };
    Entry entries[Capacity];
    size_t previous_count=0;
public:
    uint32_t local_rebuilds=0,world_rebuilds=0;
    void clear(){for(auto& e:entries)e.initialized=false;previous_count=0;}
    uint32_t revision(size_t index) const{return entries[index].revision;}
    template<class Objects,class Matrices,class MakeLocal>
    void sync(const Objects& objects,Matrices& world,size_t count,MakeLocal make_local) {
        local_rebuilds=world_rebuilds=0;
        if(count>Capacity)count=Capacity;
        bool changed[Capacity]={};
        const bool resized=count!=previous_count;
        bool any=resized;
        for(size_t i=0;i<count;++i){
            const auto& object=objects[i];auto& e=entries[i];
            const auto& t=object.transform;
            // One branch-free comparison of the nine scalars; the cached pose is
            // rewritten only when something differs.
            // Compare the original fields directly. Building/transposing a
            // nine-word temporary on every query is expensive on the R3000A.
            const uint32_t difference=
                uint32_t(e.pose[0]^t.position[0].raw()) |
                uint32_t(e.pose[1]^t.position[1].raw()) |
                uint32_t(e.pose[2]^t.position[2].raw()) |
                uint32_t(e.pose[3]^t.rotation[0].raw()) |
                uint32_t(e.pose[4]^t.rotation[1].raw()) |
                uint32_t(e.pose[5]^t.rotation[2].raw()) |
                uint32_t(e.pose[6]^t.scale[0].raw()) |
                uint32_t(e.pose[7]^t.scale[1].raw()) |
                uint32_t(e.pose[8]^t.scale[2].raw());
            const bool pose_changed=!e.initialized||difference!=0;
            if(pose_changed)for(int k=0;k<3;++k){e.pose[k]=t.position[k].raw();e.pose[3+k]=t.rotation[k].raw();e.pose[6+k]=t.scale[k].raw();}
            changed[i]=pose_changed||!e.initialized||e.parent!=object.parent||e.generation!=object.generation||e.alive!=object.alive;
            any|=changed[i];
            if(pose_changed){e.local=make_local(t);++local_rebuilds;}
            e.parent=object.parent;e.generation=object.generation;e.alive=object.alive;e.initialized=true;
        }
        for(size_t i=count;i<previous_count;++i)entries[i].initialized=false;
        previous_count=count;
        if(!any)return;
        for(size_t i=0;i<count;++i){
            // Root matrices do not depend on any other slot. In particular,
            // a moving player must not rebuild/walk every static root, nor
            // multiply its own matrix by identity at each collision query.
            if(entries[i].parent<0){
                if(resized||changed[i]){
                    world[i]=entries[i].alive?entries[i].local:Affine<Number>::identity();
                    auto& revision=entries[i].revision;if(!++revision)++revision;
                    ++world_rebuilds;
                }
                continue;
            }
            int ancestors[33];size_t depth=0;int current=int(i);
            bool dirty=resized;
            while(current>=0&&size_t(current)<count&&depth<33){
                ancestors[depth++]=current;dirty|=changed[current];current=entries[current].parent;
            }
            if(!dirty)continue;
            auto matrix=Affine<Number>::identity();
            // Preserve the runtime's bounded-walk behavior for invalid parents
            // and cycles. Slot order does not have to follow hierarchy order.
            if(entries[i].alive&&current<0)
                while(depth)matrix=matrix.compose(entries[ancestors[--depth]].local);
            world[i]=matrix;
            auto& revision=entries[i].revision;if(!++revision)++revision;
            ++world_rebuilds;
        }
    }
};
}
