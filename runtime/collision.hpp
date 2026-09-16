#pragma once
#include <stddef.h>
#include <stdint.h>
#include "affine.hpp"

namespace epok {
// Number is Q12 (PsyQo Fixed on target). The templates also allow host testing.
template<class Number> struct ColliderT {
    bool enabled=false,trigger=false;
    Number center[3]={0.0,0.0,0.0},half_extents[3]={0.5,0.5,0.5};
    uint32_t layer=1,mask=0xffffffffu;
    // A ramp. The top face rises by `slope_rise` across the box along
    // `slope_axis` (0 = X, 2 = Z), low edge at that axis's minimum. Zero rise
    // is an ordinary box and costs nothing.
    //
    // A ramp is a surface to stand on, not an obstacle: the sweep ignores it and
    // the character is lifted onto it instead. That is what lets it be walked up
    // from any side, and it is the classic height-field floor of the era rather
    // than a general inclined plane, which this solver has no normals for.
    Number slope_rise=0.0;
    uint8_t slope_axis=0;
};
template<class Number> struct AabbT { Number min[3],max[3]; };
template<class Number> struct SpatialHitT {
    int entity=-1;uint32_t generation=0;Number fraction=1.0,point[3]={},normal[3]={};bool started_inside=false;
    explicit operator bool() const { return entity>=0; }
};
template<class Number> struct MoveResultT {
    Number displacement[3]={},normal[3]={};int entity=-1;uint32_t generation=0;
    bool grounded=false,blocked=false,unresolved_overlap=false;
};
enum class TriggerPhase { Enter, Stay, Exit };
struct TriggerEvent { uint16_t first,second;uint32_t first_generation,second_generation;TriggerPhase phase; };

template<class Number> inline bool aabb_overlap(const AabbT<Number>& a,const AabbT<Number>& b) {
    for(int k=0;k<3;++k)if(a.max[k]<=b.min[k]||a.min[k]>=b.max[k])return false;
    return true;
}
template<class Number> inline bool aabb_may_touch(const AabbT<Number>& a,const AabbT<Number>& b) {
    for(int k=0;k<3;++k)if(a.max[k]<b.min[k]||a.min[k]>b.max[k])return false;
    return true;
}
template<class Number> inline AabbT<Number> swept_bounds(AabbT<Number> box,const Number* delta) {
    for(int k=0;k<3;++k){
        auto& edge=delta[k].raw()<0?box.min[k]:box.max[k];
        int64_t end=int64_t(edge.raw())+delta[k].raw();
        edge=Number(int32_t(end<INT32_MIN?INT32_MIN:end>INT32_MAX?INT32_MAX:end),Number::RAW);
    }
    return box;
}
template<class Number> inline AabbT<Number> collider_bounds(const ColliderT<Number>& c,const Affine<Number>& world) {
    AabbT<Number> out;Number center[3];world.point(c.center,center);
    for(int r=0;r<3;++r) {
        Number extent=0.0;
        for(int k=0;k<3;++k) { auto n=world.values[r][k];extent+=(n<0.0?-n:n)*c.half_extents[k]; }
        out.min[r]=center[r]-extent;out.max[r]=center[r]+extent;
    }
    return out;
}

// Bounded, allocation-free broad phase. Transformed boxes become conservative
// world AABBs: rotations and inherited nonuniform scale/shear are supported;
// narrow-phase oriented boxes and rigid-body dynamics are intentionally absent.
template<class Number,size_t Capacity,size_t PairCapacity=256> class CollisionWorld {
    struct Entry {
        AabbT<Number> box{};uint32_t layer=1,mask=0xffffffffu,generation=0;bool enabled=false,trigger=false;
        Number center[3]={},half_extents[3]={};uint32_t world_revision=0;bool cached=false;
        Number slope_rise{};uint8_t slope_axis=0;
    };
    Entry entries[Capacity];
    // Highest enabled slot + 1 and the number of enabled triggers, refreshed by
    // every synchronization so queries and pair searches skip empty slots.
    size_t limit=0,trigger_count=0;
    struct Pair { uint16_t a,b;uint32_t ga,gb; };
    Pair previous[PairCapacity];size_t previous_count=0;
    static constexpr int64_t one=int64_t(1)<<24;
    static Number raw(int32_t value) { return Number(value,Number::RAW); }
    static int64_t abs64(int64_t value) { return value<0?-value:value; }
    // Top of an entry under a horizontal position: the ramp's surface where one
    // is authored, the box top otherwise. Clamped to the box so a character
    // stepping off the end rests on the edge rather than extrapolating.
    static Number surface(const Entry& e,const Number* point) {
        if(!e.slope_rise.raw())return e.box.max[1];
        const int axis=e.slope_axis;
        const int32_t span=e.box.max[axis].raw()-e.box.min[axis].raw();
        if(span<=0)return e.box.max[1];
        int32_t along=point[axis].raw()-e.box.min[axis].raw();
        if(along<0)along=0;else if(along>span)along=span;
        const int32_t rise=int32_t(int64_t(e.slope_rise.raw())*along/span);
        return raw(e.box.max[1].raw()-e.slope_rise.raw()+rise);
    }
    static bool ramp(const Entry& e) { return e.slope_rise.raw()!=0; }
    static bool same(const Pair& a,const Pair& b) { return a.a==b.a&&a.b==b.b&&a.ga==b.ga&&a.gb==b.gb; }
    bool match(size_t i,uint32_t mask,int ignore,bool triggers) const {
        return entries[i].enabled&&int(i)!=ignore&&(entries[i].layer&mask)&&(triggers||!entries[i].trigger);
    }
    // Q24 fraction avoids thin-obstacle tunnelling through Q12 time rounding.
    static bool segment(const Number* origin,const Number* delta,const AabbT<Number>& box,int64_t& t,int& axis,int& sign,bool sweep,bool& inside) {
        int64_t enter=0,leave=one;axis=-1;sign=0;inside=true;
        for(int k=0;k<3;++k) {
            int64_t p=origin[k].raw(),d=delta[k].raw(),lo=box.min[k].raw(),hi=box.max[k].raw();
            if(p<=lo||p>=hi)inside=false;
            if(!d) {
                if(sweep?(p<=lo||p>=hi):(p<lo||p>hi))return false;
                continue;
            }
            int64_t near=((lo-p)*one)/d,far=((hi-p)*one)/d;int normal=-1;
            if(near>far) { auto tmp=near;near=far;far=tmp;normal=1; }
            if(near>enter||(near==enter&&axis<0)) { enter=near;axis=k;sign=normal; }
            if(far<leave)leave=far;
            if(enter>leave)return false;
        }
        if(leave<0||enter>one)return false;
        if(sweep&&!inside&&axis<0)return false; // touching and moving away
        t=enter;return true;
    }
public:
    uint32_t dropped_trigger_pairs=0;
    void clear() { for(auto& e:entries){e.enabled=false;e.cached=false;}previous_count=0;dropped_trigger_pairs=0;limit=trigger_count=0; }
    void begin_sync() { for(size_t i=0;i<limit;++i)entries[i].enabled=false;limit=trigger_count=0; }
    void note_enabled(size_t index) { if(index>=limit)limit=index+1;if(entries[index].trigger)++trigger_count; }
    void set(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,bool active=true,uint32_t generation=0) {
        if(index>=Capacity)return;
        entries[index]={collider_bounds(collider,matrix),collider.layer,collider.mask,generation,active&&collider.enabled,collider.trigger};
        entries[index].slope_rise=collider.slope_rise;
        entries[index].slope_axis=collider.slope_axis<3?collider.slope_axis:0;
        if(entries[index].enabled)note_enabled(index);
    }
    // Returns whether bounds were rebuilt. Metadata remains live even when a
    // static box is reused; direct collider edits and slot generations are safe.
    bool set_cached(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,uint32_t revision,bool active=true,uint32_t generation=0) {
        if(index>=Capacity)return false;
        auto& e=entries[index];
        e.enabled=active&&collider.enabled;e.trigger=collider.trigger;e.layer=collider.layer;e.mask=collider.mask;e.generation=generation;
        e.slope_rise=collider.slope_rise;e.slope_axis=collider.slope_axis<3?collider.slope_axis:0;
        if(!e.enabled)return false;
        note_enabled(index);
        bool dirty=!e.cached||e.world_revision!=revision;
        for(int k=0;k<3;++k){dirty|=e.center[k].raw()!=collider.center[k].raw()||e.half_extents[k].raw()!=collider.half_extents[k].raw();}
        if(dirty){
            e.box=collider_bounds(collider,matrix);e.world_revision=revision;e.cached=true;
            for(int k=0;k<3;++k){e.center[k]=collider.center[k];e.half_extents[k]=collider.half_extents[k];}
        }
        return dirty;
    }
    const AabbT<Number>* bounds(size_t index) const { return index<Capacity&&entries[index].enabled?&entries[index].box:nullptr; }
    // Returns total matches; only the first output_capacity indices are written.
    size_t overlap(const AabbT<Number>& box,uint16_t* output,size_t output_capacity,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=true) const {
        size_t count=0;
        for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,triggers)&&aabb_overlap(box,entries[i].box)) {
            if(output&&count<output_capacity)output[count]=uint16_t(i);++count;
        }
        return count;
    }
    // Segment query: delta is the complete displacement, not a unit direction.
    SpatialHitT<Number> raycast(const Number* origin,const Number* delta,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=false) const {
        SpatialHitT<Number> hit;int64_t best=one+1;
        AabbT<Number> point;for(int k=0;k<3;++k)point.min[k]=point.max[k]=origin[k];
        const auto sweep=swept_bounds(point,delta);
        for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,triggers)&&aabb_may_touch(sweep,entries[i].box)) {
            int64_t t;int axis,sign;bool inside;
            if(segment(origin,delta,entries[i].box,t,axis,sign,false,inside)&&t<best) {
                best=t;hit.entity=int(i);hit.generation=entries[i].generation;hit.started_inside=inside;hit.fraction=raw(int32_t(t*4096/one));
                for(int k=0;k<3;++k) { hit.point[k]=origin[k]+raw(int32_t(int64_t(delta[k].raw())*t/one));hit.normal[k]=raw(k==axis?sign*4096:0); }
            }
        }
        return hit;
    }
    // Downward box sweep uses the complete footprint, including ledges missed by
    // a center ray. distance must be nonnegative; normal points upwards.
    SpatialHitT<Number> ground(const AabbT<Number>& box,Number distance,uint32_t mask=0xffffffffu,int ignore=-1) const {
        SpatialHitT<Number> hit;if(distance<0.0)return hit;
        Number best=distance;
        for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,false)) {
            const auto& b=entries[i].box;
            if(box.max[0]<=b.min[0]||box.min[0]>=b.max[0]||box.max[2]<=b.min[2]||box.min[2]>=b.max[2])continue;
            Number probe[3];for(int k=0;k<3;++k)probe[k]=(box.min[k]+box.max[k])/2;
            const auto top=surface(entries[i],probe);
            auto gap=box.min[1]-top;
            if(gap<0.0||gap>best)continue;
            if(hit.entity>=0&&gap==best)continue;
            best=gap;hit.entity=int(i);hit.generation=entries[i].generation;hit.normal[1]=1.0;
            hit.fraction=distance>0.0?gap/distance:raw(0);
            hit.point[0]=(box.min[0]+box.max[0])/2;hit.point[1]=top;hit.point[2]=(box.min[2]+box.max[2])/2;
        }
        return hit;
    }
    MoveResultT<Number> move_and_slide(AabbT<Number> box,const Number* displacement,uint32_t mask=0xffffffffu,int ignore=-1) const {
        MoveResultT<Number> result;Number remaining[3]={displacement[0],displacement[1],displacement[2]};
        // Resolve existing overlap by minimum translation, bounded to 8 passes.
        for(unsigned pass=0;pass<8;++pass) {
            bool found=false;int best_axis=0,best_entity=-1;int64_t best=INT64_MAX;
            for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,false)&&aabb_overlap(box,entries[i].box)) {
                const auto& b=entries[i].box;
                if(ramp(entries[i])) {
                    // Standing on a ramp is the only way to resolve against it:
                    // pushing sideways would let a character be shoved off a
                    // slope it is simply walking up.
                    Number probe[3];for(int k=0;k<3;++k)probe[k]=(box.min[k]+box.max[k])/2;
                    const int64_t up=int64_t(surface(entries[i],probe).raw())-box.min[1].raw()+1;
                    if(up>0&&(!found||abs64(up)<abs64(best))) { found=true;best=up;best_axis=1;best_entity=int(i); }
                    continue;
                }
                for(int k=0;k<3;++k) {
                    int64_t neg=int64_t(b.min[k].raw())-box.max[k].raw()-1,pos=int64_t(b.max[k].raw())-box.min[k].raw()+1;
                    int64_t d=abs64(neg)<abs64(pos)?neg:pos;
                    if(!found||abs64(d)<abs64(best)) { found=true;best=d;best_axis=k;best_entity=int(i); }
                }
            }
            if(!found)break;
            auto shift=raw(int32_t(best));box.min[best_axis]+=shift;box.max[best_axis]+=shift;result.displacement[best_axis]+=shift;
            result.blocked=true;result.entity=best_entity;result.generation=entries[size_t(best_entity)].generation;result.normal[best_axis]=raw(best<0?-4096:4096);
            if(best_axis==1&&best>0)result.grounded=true;
        }
        // A ramp always overlaps the character standing on it, by design; only a
        // solid box still intersecting here is genuinely unresolved.
        for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,false)&&!ramp(entries[i])&&aabb_overlap(box,entries[i].box)) {
            result.unresolved_overlap=true;return result;
        }
        // Axis-aligned contact normals need at most three sliding planes.
        for(unsigned pass=0;pass<3;++pass) {
            if(!remaining[0].raw()&&!remaining[1].raw()&&!remaining[2].raw())break;
            const auto sweep=swept_bounds(box,remaining);
            Number center[3],extent[3];
            for(int k=0;k<3;++k) { center[k]=(box.min[k]+box.max[k])/2;extent[k]=(box.max[k]-box.min[k])/2; }
            int64_t best=one+1;int best_axis=-1,best_sign=0,best_entity=-1;
            for(size_t i=0;i<limit;++i)if(match(i,mask,ignore,false)&&!ramp(entries[i])&&aabb_may_touch(sweep,entries[i].box)) {
                auto expanded=entries[i].box;
                for(int k=0;k<3;++k) { expanded.min[k]-=extent[k];expanded.max[k]+=extent[k]; }
                int64_t t;int axis,sign;bool inside;
                if(segment(center,remaining,expanded,t,axis,sign,true,inside)&&axis>=0&&t<best) { best=t;best_axis=axis;best_sign=sign;best_entity=int(i); }
            }
            int64_t fraction=best_entity<0?one:best;
            for(int k=0;k<3;++k) {
                auto moved=raw(int32_t(int64_t(remaining[k].raw())*fraction/one));
                box.min[k]+=moved;box.max[k]+=moved;result.displacement[k]+=moved;remaining[k]-=moved;
            }
            if(best_entity<0)break;
            auto skin=raw(best_sign);box.min[best_axis]+=skin;box.max[best_axis]+=skin;result.displacement[best_axis]+=skin;remaining[best_axis]=raw(0);
            result.blocked=true;result.entity=best_entity;result.generation=entries[size_t(best_entity)].generation;result.normal[best_axis]=raw(best_sign*4096);
            if(best_axis==1&&best_sign>0)result.grounded=true;
        }
        return result;
    }
    template<class Callback> void update_triggers(Callback&& callback) {
        Pair next[PairCapacity];size_t count=0;dropped_trigger_pairs=0;
        // Without an enabled trigger there are no pairs to enter or stay in;
        // previously overlapping pairs still receive their exit events.
        if(trigger_count)for(size_t a=0;a<limit;++a)if(entries[a].enabled)for(size_t b=a+1;b<limit;++b) {
            const auto& x=entries[a];const auto& y=entries[b];
            if(!y.enabled||(!x.trigger&&!y.trigger)||!(x.layer&y.mask)||!(y.layer&x.mask)||!aabb_overlap(x.box,y.box))continue;
            if(count>=PairCapacity) { ++dropped_trigger_pairs;continue; }
            next[count++]={uint16_t(a),uint16_t(b),x.generation,y.generation};
        }
        // Determine pairs before invoking user code, which may move entities.
        for(size_t n=0;n<count;++n) {
            auto pair=next[n];bool was=false;
            for(size_t i=0;i<previous_count;++i)if(same(pair,previous[i])) { was=true;break; }
            callback(TriggerEvent{pair.a,pair.b,pair.ga,pair.gb,was?TriggerPhase::Stay:TriggerPhase::Enter});
        }
        for(size_t i=0;i<previous_count;++i) {
            auto pair=previous[i];bool stays=false;
            for(size_t j=0;j<count;++j)if(same(pair,next[j])) { stays=true;break; }
            if(!stays)callback(TriggerEvent{pair.a,pair.b,pair.ga,pair.gb,TriggerPhase::Exit});
        }
        previous_count=count;for(size_t i=0;i<count;++i)previous[i]=next[i];
    }
};
}
