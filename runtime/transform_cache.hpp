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
        uint32_t generation=0,revision=0,parent_revision=0;
        bool initialized=false,alive=false,world_dirty=false,descendants_dirty=false;
    };
    Entry entries[Capacity];
    size_t previous_count=0;
public:
    uint32_t local_rebuilds=0,world_rebuilds=0;
    void clear(){for(auto& e:entries)e.initialized=false;previous_count=0;}
    uint32_t revision(size_t index) const{return entries[index].revision;}
    // Common physics case: an unparented collider. A query can synchronize
    // just that root; a later full/subset sync still propagates its change to
    // visual or collider children through descendants_dirty.
    template<class Object,class MakeLocal>
    void sync_root(const Object& object,Affine<Number>& world,size_t index,MakeLocal make_local) {
        auto& e=entries[index];const auto& t=object.transform;
        const uint32_t basis_difference=uint32_t(e.pose[3]^t.rotation[0].raw())|uint32_t(e.pose[4]^t.rotation[1].raw())|uint32_t(e.pose[5]^t.rotation[2].raw())|
            uint32_t(e.pose[6]^t.scale[0].raw())|uint32_t(e.pose[7]^t.scale[1].raw())|uint32_t(e.pose[8]^t.scale[2].raw());
        const uint32_t difference=basis_difference|uint32_t(e.pose[0]^t.position[0].raw())|uint32_t(e.pose[1]^t.position[1].raw())|uint32_t(e.pose[2]^t.position[2].raw());
        const bool pose_changed=!e.initialized||difference;
        const bool changed=pose_changed||e.parent!=object.parent||e.generation!=object.generation||e.alive!=object.alive||e.world_dirty;
        if(!changed)return;
        if(pose_changed){for(int k=0;k<3;++k){e.pose[k]=t.position[k].raw();e.pose[3+k]=t.rotation[k].raw();e.pose[6+k]=t.scale[k].raw();}
            if(!e.initialized||basis_difference)e.local=make_local(t);
            else for(int k=0;k<3;++k)e.local.values[k][3]=t.position[k];
            ++local_rebuilds;}
        e.parent=object.parent;e.generation=object.generation;e.alive=object.alive;e.initialized=true;e.parent_revision=0;
        world=e.alive?e.local:Affine<Number>::identity();e.world_dirty=false;e.descendants_dirty=true;
        if(!++e.revision)++e.revision;++world_rebuilds;
    }
    // A single query needs its target and ancestor path, rather than two full
    // Capacity scans. Pending descendant changes still reach the next full sync.
    // Unusual dead/invalid/cyclic ancestor chains retain the original path.
    template<class Objects,class Matrices,class MakeLocal>
    bool sync_path(const Objects& objects,Matrices& world,size_t count,size_t index,MakeLocal make_local){
        local_rebuilds=world_rebuilds=0;if(count>Capacity)count=Capacity;if(index>=count)return false;
        size_t chain[33],depth=0;int current=int(index);
        while(current>=0){
            if(size_t(current)>=count||depth==33)return false;
            for(size_t n=0;n<depth;++n)if(chain[n]==size_t(current))return false;
            if(depth&&!objects[size_t(current)].alive)return false;
            chain[depth++]=size_t(current);current=objects[size_t(current)].parent;
        }
        while(depth){
            const size_t i=chain[--depth];const auto& object=objects[i];const auto& t=object.transform;auto& e=entries[i];
            const uint32_t basis=uint32_t(e.pose[3]^t.rotation[0].raw())|uint32_t(e.pose[4]^t.rotation[1].raw())|uint32_t(e.pose[5]^t.rotation[2].raw())|
                uint32_t(e.pose[6]^t.scale[0].raw())|uint32_t(e.pose[7]^t.scale[1].raw())|uint32_t(e.pose[8]^t.scale[2].raw());
            const uint32_t difference=basis|uint32_t(e.pose[0]^t.position[0].raw())|uint32_t(e.pose[1]^t.position[1].raw())|uint32_t(e.pose[2]^t.position[2].raw());
            const bool pose_changed=!e.initialized||difference;
            const uint32_t parent_revision=object.parent<0?0:entries[size_t(object.parent)].revision;
            const bool dirty=pose_changed||e.parent!=object.parent||e.generation!=object.generation||e.alive!=object.alive||e.world_dirty||e.parent_revision!=parent_revision;
            if(pose_changed){
                for(int k=0;k<3;++k){e.pose[k]=t.position[k].raw();e.pose[k+3]=t.rotation[k].raw();e.pose[k+6]=t.scale[k].raw();}
                if(!e.initialized||basis)e.local=make_local(t);else for(int k=0;k<3;++k)e.local.values[k][3]=t.position[k];
                ++local_rebuilds;
            }
            e.parent=object.parent;e.generation=object.generation;e.alive=object.alive;e.initialized=true;e.parent_revision=parent_revision;
            if(dirty){
                world[i]=!e.alive?Affine<Number>::identity():object.parent<0?e.local:world[size_t(object.parent)].compose(e.local);
                e.world_dirty=false;e.descendants_dirty=true;if(!++e.revision)++e.revision;++world_rebuilds;
            }
        }
        return true;
    }
    template<class Objects,class Matrices,class MakeLocal>
    void sync(const Objects& objects,Matrices& world,size_t count,MakeLocal make_local,const bool* selected=nullptr) {
        local_rebuilds=world_rebuilds=0;
        if(count>Capacity)count=Capacity;
        bool changed[Capacity]={};
        const bool resized=count!=previous_count;
        bool any=resized;
        for(size_t i=0;i<count;++i){
            // A subset must include the complete ancestor chain of each target.
            // Unselected descendants retain a pending world invalidation below.
            if(selected&&!selected[i])continue;
            const auto& object=objects[i];auto& e=entries[i];
            const auto& t=object.transform;
            // One branch-free comparison of the nine scalars; the cached pose is
            // rewritten only when something differs.
            // Compare the original fields directly. Building/transposing a
            // nine-word temporary on every query is expensive on the R3000A.
            const uint32_t basis_difference=
                uint32_t(e.pose[3]^t.rotation[0].raw()) |
                uint32_t(e.pose[4]^t.rotation[1].raw()) |
                uint32_t(e.pose[5]^t.rotation[2].raw()) |
                uint32_t(e.pose[6]^t.scale[0].raw()) |
                uint32_t(e.pose[7]^t.scale[1].raw()) |
                uint32_t(e.pose[8]^t.scale[2].raw());
            const uint32_t difference=basis_difference|
                uint32_t(e.pose[0]^t.position[0].raw()) |
                uint32_t(e.pose[1]^t.position[1].raw()) |
                uint32_t(e.pose[2]^t.position[2].raw());
            const bool pose_changed=!e.initialized||difference!=0;
            if(pose_changed)for(int k=0;k<3;++k){e.pose[k]=t.position[k].raw();e.pose[3+k]=t.rotation[k].raw();e.pose[6+k]=t.scale[k].raw();}
            changed[i]=pose_changed||!e.initialized||e.parent!=object.parent||e.generation!=object.generation||e.alive!=object.alive||e.descendants_dirty;
            e.descendants_dirty=false;
            any|=changed[i]||e.world_dirty;
            if(pose_changed){
                // Standard TRS: translation does not change the local basis.
                // Walking with unchanged heading need not redo trigonometry.
                if(!e.initialized||basis_difference)e.local=make_local(t);
                else for(int k=0;k<3;++k)e.local.values[k][3]=t.position[k];
                ++local_rebuilds;
            }
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
                if(resized||changed[i]||entries[i].world_dirty){
                    if(selected&&!selected[i]){entries[i].world_dirty=true;continue;}
                    world[i]=entries[i].alive?entries[i].local:Affine<Number>::identity();
                    entries[i].world_dirty=false;
                    auto& revision=entries[i].revision;if(!++revision)++revision;
                    ++world_rebuilds;
                }
                continue;
            }
            int ancestors[33];size_t depth=0;int current=int(i);
            bool dirty=resized||entries[i].world_dirty;
            while(current>=0&&size_t(current)<count&&depth<33){
                ancestors[depth++]=current;dirty|=changed[current];current=entries[current].parent;
            }
            if(!dirty)continue;
            if(selected&&!selected[i]){entries[i].world_dirty=true;continue;}
            auto matrix=Affine<Number>::identity();
            // Preserve the runtime's bounded-walk behavior for invalid parents
            // and cycles. Slot order does not have to follow hierarchy order.
            if(entries[i].alive&&current<0)
                while(depth)matrix=matrix.compose(entries[ancestors[--depth]].local);
            world[i]=matrix;
            entries[i].parent_revision=entries[i].parent>=0&&size_t(entries[i].parent)<count?entries[size_t(entries[i].parent)].revision:0;
            entries[i].world_dirty=false;
            auto& revision=entries[i].revision;if(!++revision)++revision;
            ++world_rebuilds;
        }
    }
};
}
