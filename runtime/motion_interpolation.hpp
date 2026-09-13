#pragma once
#include <array>
#include <stdint.h>
#include "affine.hpp"

namespace epok {
// Render-only interpolation of entity translations. Rotation/scale and all
// gameplay/collision queries remain at the latest fixed step. No heap.
template<class Number,size_t Capacity,bool Enabled> class MotionInterpolation;
template<class Number,size_t Capacity> class MotionInterpolation<Number,Capacity,false> {
public:
    void clear() {}
    template<class Objects,class Predicate> void select(const Objects&,size_t,Predicate) {}
    template<class Objects> void before_tick(const Objects&,size_t) {}
    template<class Objects> void after_tick(const Objects&,size_t) {}
    template<class Objects> const std::array<Affine<Number>,Capacity>& prepare(
        const Objects&,const std::array<Affine<Number>,Capacity>& world,size_t,unsigned){return world;}
    template<class Transform> Transform local(size_t,const Transform& value) const {return value;}
};
template<class Number,size_t Capacity> class MotionInterpolation<Number,Capacity,true> {
    struct History {
        int32_t previous[3]={},current[3]={};
        int parent=-1;uint32_t generation=0;
        bool alive=false,active=false,valid=false;
    };
    std::array<History,Capacity> history{};
    std::array<Affine<Number>,Capacity> rendered{};
    std::array<uint8_t,Capacity> ready{};
    std::array<bool,Capacity> selected{};
    unsigned alpha=4096;
    size_t captured=0;
    int32_t delta(size_t i,int axis) const {
        const auto& h=history[i];
        // Subtraction is wide: scripts may teleport across the Q12 range.
        return int32_t((int64_t(h.previous[axis])-h.current[axis])*(4096-alpha)/4096);
    }
    template<class Objects,class Matrices> bool resolve(size_t i,const Objects& objects,const Matrices& world,size_t count,unsigned depth){
        if(ready[i])return ready[i]==2;
        ready[i]=1;
        const auto& object=objects[i];const auto& h=history[i];
        bool valid=depth<33&&h.valid&&h.alive&&object.alive&&h.active==object.active&&
                   h.generation==object.generation&&h.parent==object.parent;
        for(int a=0;a<3;++a){
            valid&=h.current[a]==object.transform.position[a].raw();
            const int64_t distance=int64_t(h.previous[a])-h.current[a];
            valid&=distance>=INT32_MIN&&distance<=INT32_MAX;
        }
        const int parent=object.parent;
        if(valid&&parent>=0)valid=size_t(parent)<count&&resolve(size_t(parent),objects,world,count,depth+1);
        if(!valid){ready[i]=3;return false;}
        bool local_moved=false;
        for(int a=0;a<3;++a)local_moved|=h.previous[a]!=h.current[a];
        if(!local_moved&&(parent<0||
            (rendered[parent].values[0][3]==world[parent].values[0][3]&&
             rendered[parent].values[1][3]==world[parent].values[1][3]&&
             rendered[parent].values[2][3]==world[parent].values[2][3]))){ready[i]=2;return true;}
        Number offset[3];bool moved=false;
        for(int a=0;a<3;++a){const int32_t d=delta(i,a);offset[a]=Number(d,Number::RAW);moved|=d!=0;}
        for(int r=0;r<3;++r){
            Number shift=0.;
            if(parent<0)shift=offset[r];
            else {
                shift=rendered[parent].values[r][3]-world[parent].values[r][3];
                if(moved)for(int c=0;c<3;++c)shift+=world[parent].values[r][c]*offset[c];
            }
            rendered[i].values[r][3]+=shift;
        }
        ready[i]=2;return true;
    }
public:
    MotionInterpolation(){selected.fill(true);}
    void clear(){for(auto& h:history)h.valid=false;ready.fill(0);captured=0;alpha=4096;}
    // Only 3D presentation participants and their ancestors need histories.
    // Collider-only entities and screen-space menu trees remain authoritative.
    template<class Objects,class Predicate> void select(const Objects& objects,size_t count,Predicate participant){
        count=count<Capacity?count:Capacity;selected.fill(false);
        for(size_t i=0;i<count;++i)if(participant(i)){
            int j=int(i);unsigned depth=0;
            while(j>=0&&size_t(j)<count&&depth++<33&&!selected[j]){selected[j]=true;j=objects[j].parent;}
        }
        for(size_t i=0;i<Capacity;++i)if(!selected[i])history[i].valid=false;
    }
    template<class Objects> void before_tick(const Objects& objects,size_t count){
        captured=count<Capacity?count:Capacity;
        for(size_t i=0;i<captured;++i){
            if(!selected[i])continue;
            auto& h=history[i];const auto& o=objects[i];
            for(int a=0;a<3;++a)h.previous[a]=o.transform.position[a].raw();
            h.parent=o.parent;h.generation=o.generation;h.alive=o.alive;h.active=o.active;h.valid=true;
        }
        for(size_t i=captured;i<Capacity;++i)history[i].valid=false;
    }
    template<class Objects> void after_tick(const Objects& objects,size_t count){
        for(size_t i=0;i<captured&&i<count;++i){
            if(!selected[i])continue;
            auto& h=history[i];const auto& o=objects[i];
            h.valid&=h.parent==o.parent&&h.generation==o.generation&&h.alive==o.alive&&h.active==o.active;
            for(int a=0;a<3;++a)h.current[a]=o.transform.position[a].raw();
        }
        for(size_t i=count;i<captured;++i)history[i].valid=false;
    }
    template<class Objects> const std::array<Affine<Number>,Capacity>& prepare(
        const Objects& objects,const std::array<Affine<Number>,Capacity>& world,size_t count,unsigned fraction){
        alpha=fraction<4096?fraction:4096;ready.fill(0);
        count=count<Capacity?count:Capacity;
        bool moving=false;
        for(size_t i=0;i<count&&!moving;++i)if(history[i].valid)
            for(int a=0;a<3;++a)moving|=history[i].previous[a]!=history[i].current[a];
        if(!moving||alpha==4096)return world;
        for(size_t i=0;i<count;++i)rendered[i]=world[i];
        for(size_t i=0;i<count;++i)if(selected[i])resolve(i,objects,world,count,0);
        return rendered;
    }
    // Camera inverse composition uses the same local translations as meshes.
    template<class Transform> Transform local(size_t i,const Transform& value) const {
        auto result=value;
        if(i<Capacity&&ready[i]==2)for(int a=0;a<3;++a)result.position[a]+=Number(delta(i,a),Number::RAW);
        return result;
    }
};
}
