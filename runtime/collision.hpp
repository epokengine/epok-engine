#pragma once
#include <bit>
#include <stddef.h>
#include <stdint.h>
#include "affine.hpp"

namespace epok {
// Immutable, cooked XZ cell lists for a compound collider. Lists contain part
// indices in source order; Q8 origins and a Q12 cell shift match the solid data.
// Binding the source pointer/count lets a replaced collider fall back safely.
struct CompoundGrid {
    const int16_t* boxes;
    const uint16_t* offsets;
    const uint16_t* indices;
    uint16_t box_count,columns,rows;
    int16_t origin_x,origin_z;
    uint8_t cell_shift;
};
inline constexpr size_t compound_grid_max_parts=2048;
// Exact Q24 fraction for 0 <= numerator <= denominator. Splitting at 16/8
// bits keeps ordinary local queries on the R3000's native 32-bit DIV instead
// of calling the software 64-bit divider for every slab intersection.
inline int64_t collision_fraction24(uint64_t numerator,uint64_t denominator) {
    if(!numerator)return 0;
    if(numerator==denominator)return int64_t(1)<<24;
    if(denominator<=65535){
        const uint32_t n=uint32_t(numerator),d=uint32_t(denominator);
        if(n<=255)return (n<<24)/d;
        const uint32_t scaled=n<<16,high=scaled/d,remainder=scaled%d;
        return (high<<8)+((remainder<<8)/d);
    }
    return int64_t((numerator<<24)/denominator);
}
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
    // A height grid, which is the same idea as the ramp taken to two axes and
    // arbitrary steps: a surface to stand on, never an obstacle. `heights` has
    // (height_cells[0]+1) * (height_cells[1]+1) Q8 entries, row-major with X
    // varying fastest, measured upwards from the bottom of the box.
    //
    // The grid is indexed from the box's world minimum, so the owning object
    // must be translated only: the cooker rejects a rotated or scaled terrain
    // rather than sampling a grid that no longer lines up with its box.
    const int16_t* heights=nullptr;
    uint16_t height_cells[2]={0,0};
    Number height_step=1.0;
    // A compound static collider: `box_count` solid boxes of six Q8 values each
    // (minimum x, y, z, then maximum x, y, z), relative to the owner's position.
    // The collider's own box encloses them and is the broad phase; queries then
    // test the parts. Like a height grid, the owner must be translated only: the
    // cooker rejects a rotated or scaled compound collider.
    const int16_t* boxes=nullptr;
    uint16_t box_count=0;
    const CompoundGrid* box_grid=nullptr;
};
template<class Number> struct AabbT { Number min[3],max[3]; };
template<class Number> struct RaycastQueryT {Number origin[3],displacement[3];};
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
template<class Number,size_t Capacity,size_t PairCapacity=256,bool UseCompoundGrid=true,bool Dense=true,bool NarrowSegments=true> class CollisionWorld {
    struct Entry {
        AabbT<Number> box{};uint32_t layer=1,mask=0xffffffffu,generation=0;bool enabled=false,trigger=false;
        Number center[3]={},half_extents[3]={};uint32_t world_revision=0;bool cached=false;
        Number slope_rise{};uint8_t slope_axis=0;
        const int16_t* heights=nullptr;uint16_t height_cells[2]={0,0};Number height_step{};
        const int16_t* boxes=nullptr;uint16_t box_count=0;Number origin[3]={};
        const CompoundGrid* box_grid=nullptr;
    };
    Entry entries[Capacity];
    // Highest enabled slot + 1 and the number of enabled triggers, refreshed by
    // every synchronization so queries and pair searches skip empty slots.
    size_t limit=0,trigger_count=0;
    uint32_t enabled_slots[Dense?Capacity:1]={},positions[Dense?Capacity:1]={};size_t enabled_count=0;
    size_t query_count()const{return Dense?enabled_count:limit;}
    size_t query_slot(size_t ordinal)const{return Dense?enabled_slots[ordinal]:ordinal;}
    void update_enabled(size_t index,bool was_enabled,bool was_trigger){
        const auto& e=entries[index];
        if(was_enabled&&was_trigger)--trigger_count;
        if(e.enabled&&e.trigger)++trigger_count;
        if(e.enabled&&index>=limit)limit=index+1;
        if constexpr(Dense){
            if(was_enabled==e.enabled)return;
            if(e.enabled){
                size_t n=enabled_count++;
                // Keep source-slot order: equal-distance collision ties and
                // trigger callbacks must stay identical to a linear scan.
                while(n&&enabled_slots[n-1]>index){enabled_slots[n]=enabled_slots[n-1];positions[enabled_slots[n]]=uint32_t(n);--n;}
                enabled_slots[n]=uint32_t(index);positions[index]=uint32_t(n);
            }else{
                const size_t begin=positions[index];--enabled_count;
                for(size_t n=begin;n<enabled_count;++n){enabled_slots[n]=enabled_slots[n+1];positions[enabled_slots[n]]=uint32_t(n);}
            }
        }
    }
    struct Pair { uint16_t a,b;uint32_t ga,gb; };
    Pair previous[PairCapacity];size_t previous_count=0;
    static constexpr int64_t one=int64_t(1)<<24;
    static Number raw(int32_t value) { return Number(value,Number::RAW); }
    static int64_t abs64(int64_t value) { return value<0?-value:value; }
    // Top of an entry under a horizontal position: the ramp's surface where one
    // is authored, the box top otherwise. Clamped to the box so a character
    // stepping off the end rests on the edge rather than extrapolating.
    static bool field(const Entry& e) {
        return e.heights&&e.height_cells[0]&&e.height_cells[1]&&e.height_step.raw()>0;
    }
    // Bilinear height at a horizontal position, clamped to the grid so a
    // character stepping off the edge rests on the border row rather than
    // extrapolating off a cliff that is not there.
    static Number sample(const Entry& e,const Number* point) {
        const int32_t step=e.height_step.raw();
        const int32_t last_x=int32_t(e.height_cells[0])-1,last_z=int32_t(e.height_cells[1])-1;
        const int64_t along_x=int64_t(point[0].raw())-e.box.min[0].raw();
        const int64_t along_z=int64_t(point[2].raw())-e.box.min[2].raw();
        int32_t cx=0,fx=0,cz=0,fz=0;
        if(along_x>0) {
            cx=int32_t(along_x/step);fx=int32_t(along_x-int64_t(cx)*step);
            if(cx>last_x){cx=last_x;fx=step;}
        }
        if(along_z>0) {
            cz=int32_t(along_z/step);fz=int32_t(along_z-int64_t(cz)*step);
            if(cz>last_z){cz=last_z;fz=step;}
        }
        const int32_t stride=int32_t(e.height_cells[0])+1;
        // Q8 grid entries widen to the Q12 the solver works in.
        const int64_t h00=int64_t(e.heights[cz*stride+cx])<<4;
        const int64_t h10=int64_t(e.heights[cz*stride+cx+1])<<4;
        const int64_t h01=int64_t(e.heights[(cz+1)*stride+cx])<<4;
        const int64_t h11=int64_t(e.heights[(cz+1)*stride+cx+1])<<4;
        const int64_t a=h00+(h10-h00)*fx/step;
        const int64_t b=h01+(h11-h01)*fx/step;
        const int64_t height=a+(b-a)*fz/step;
        return raw(int32_t(int64_t(e.box.min[1].raw())+height));
    }
    static Number surface(const Entry& e,const Number* point) {
        if(field(e))return sample(e,point);
        if(!e.slope_rise.raw())return e.box.max[1];
        const int axis=e.slope_axis;
        const int32_t span=e.box.max[axis].raw()-e.box.min[axis].raw();
        if(span<=0)return e.box.max[1];
        int32_t along=point[axis].raw()-e.box.min[axis].raw();
        if(along<0)along=0;else if(along>span)along=span;
        const int32_t rise=int32_t(int64_t(e.slope_rise.raw())*along/span);
        return raw(e.box.max[1].raw()-e.slope_rise.raw()+rise);
    }
    // A standable entry is resolved by lifting the character onto its surface
    // instead of by pushing it sideways. Ramps and height grids both qualify.
    static bool ramp(const Entry& e) { return e.slope_rise.raw()!=0||field(e); }
    static bool compound(const Entry& e) { return e.boxes&&e.box_count; }
    template<class Visit> static void part(const Entry& e,uint16_t n,Visit&& visit) {
        const int16_t* p=e.boxes+size_t(n)*6;AabbT<Number> b;
        for(int k=0;k<3;++k) {
            b.min[k]=raw(e.origin[k].raw()+int32_t(p[k])*16);
            b.max[k]=raw(e.origin[k].raw()+int32_t(p[k+3])*16);
        }
        visit(b);
    }
    // Select by the complete query footprint, including swept movement/rays.
    // Inclusive cell boundaries keep endpoint contacts. Candidate deduplication
    // and ascending part indices preserve every narrow-phase tie and callback.
    // No heap or per-frame index rebuild: translating owners move the query into
    // the immutable local grid. Unindexed/replaced data keeps the linear path.
    template<class Visit> static void parts(const Entry& e,const AabbT<Number>& query,Visit&& visit) {
        if(!compound(e)) { visit(e.box); return; }
        if constexpr(UseCompoundGrid) {
            const auto* grid=e.box_grid;
            if(grid&&grid->boxes==e.boxes&&grid->box_count==e.box_count&&e.box_count<=compound_grid_max_parts
               &&grid->columns&&grid->rows&&grid->cell_shift>=4&&grid->cell_shift<32) {
                const int64_t origin_x=int64_t(e.origin[0].raw())+int32_t(grid->origin_x)*16;
                const int64_t origin_z=int64_t(e.origin[2].raw())+int32_t(grid->origin_z)*16;
                int64_t x0=(int64_t(query.min[0].raw())-origin_x)>>grid->cell_shift;
                int64_t x1=(int64_t(query.max[0].raw())-origin_x)>>grid->cell_shift;
                int64_t z0=(int64_t(query.min[2].raw())-origin_z)>>grid->cell_shift;
                int64_t z1=(int64_t(query.max[2].raw())-origin_z)>>grid->cell_shift;
                if(x1<0||z1<0||x0>=grid->columns||z0>=grid->rows)return;
                if(x0<0)x0=0;if(z0<0)z0=0;
                if(x1>=grid->columns)x1=grid->columns-1;
                if(z1>=grid->rows)z1=grid->rows-1;
                if(x0==x1&&z0==z1) {
                    const size_t cell=size_t(z0)*grid->columns+size_t(x0);
                    for(unsigned i=grid->offsets[cell];i<grid->offsets[cell+1];++i)part(e,grid->indices[i],visit);
                    return;
                }
                uint32_t selected[(compound_grid_max_parts+31)/32]={};
                unsigned first=unsigned((compound_grid_max_parts+31)/32),last=0;
                for(int z=int(z0);z<=int(z1);++z)for(int x=int(x0);x<=int(x1);++x) {
                    const size_t cell=size_t(z)*grid->columns+size_t(x);
                    for(unsigned i=grid->offsets[cell];i<grid->offsets[cell+1];++i) {
                        const unsigned n=grid->indices[i],word=n/32;
                        selected[word]|=uint32_t(1)<<(n%32);
                        if(word<first)first=word;if(word>last)last=word;
                    }
                }
                for(unsigned word=first;word<=last;++word) {
                    uint32_t bits=selected[word];
                    while(bits) {
                        const unsigned bit=std::countr_zero(bits);bits&=bits-1;
                        part(e,uint16_t(word*32+bit),visit);
                    }
                }
                return;
            }
        }
        for(uint16_t n=0;n<e.box_count;++n)part(e,n,visit);
    }
    static bool parts_overlap(const Entry& e,const AabbT<Number>& box) {
        bool hit=false;parts(e,box,[&](const AabbT<Number>& b){ if(!hit&&aabb_overlap(box,b))hit=true; });
        return hit;
    }
    // Trigger pairs compare a compound entry's parts, so a trigger inside the
    // enclosing box of a compound collider is not mistaken for a contact.
    static bool touching(const Entry& x,const Entry& y) {
        if(compound(x))return parts_overlap(x,y.box);
        if(compound(y))return parts_overlap(y,x.box);
        return true;
    }
    static bool same(const Pair& a,const Pair& b) { return a.a==b.a&&a.b==b.b&&a.ga==b.ga&&a.gb==b.gb; }
    bool match(size_t i,uint32_t mask,int ignore,bool triggers) const {
        return entries[i].enabled&&int(i)!=ignore&&(entries[i].layer&mask)&&(triggers||!entries[i].trigger);
    }
    // Q24 fraction avoids thin-obstacle tunnelling through Q12 time rounding.
    template<class Coordinate>
#if defined(__GNUC__) && !defined(__clang__)
    __attribute__((noinline,optimize("O3")))
#endif
    static bool segment_width(const Number* origin,const Number* delta,const AabbT<Number>& box,int64_t& t,int& axis,int& sign,bool sweep,bool& inside) {
        Coordinate enter=0,leave=one;axis=-1;sign=0;inside=true;
        for(int k=0;k<3;++k) {
            Coordinate p=origin[k].raw(),d=delta[k].raw(),lo=box.min[k].raw(),hi=box.max[k].raw();
            if(p<=lo||p>=hi)inside=false;
            if(!d) {
                if(sweep?(p<=lo||p>=hi):(p<lo||p>hi))return false;
                continue;
            }
            const Coordinate magnitude=d>0?d:-d;
            const Coordinate near_distance=d>0?lo-p:p-hi,far_distance=d>0?hi-p:p-lo;
            if(far_distance<0||near_distance>magnitude)return false;
            // enter starts at zero and leave at one. Fractions outside that
            // interval cannot tighten it; preserve a rounded zero for contact
            // normal selection when the original negative quotient was zero.
            const Coordinate near=near_distance<0?
                ((uint64_t(-near_distance)<<24)<uint64_t(magnitude)?0:-1):
                collision_fraction24(uint64_t(near_distance),uint64_t(magnitude));
            const Coordinate far=far_distance>=magnitude?one:
                collision_fraction24(uint64_t(far_distance),uint64_t(magnitude));
            const int normal=d>0||near_distance==far_distance||
                (far_distance<=magnitude&&near==far)?-1:1;
            if(near>enter||(near==enter&&axis<0)) { enter=near;axis=k;sign=normal; }
            if(far<leave)leave=far;
            if(enter>leave)return false;
        }
        if(leave<0||enter>one)return false;
        if(sweep&&!inside&&axis<0)return false; // touching and moving away
        t=enter;return true;
    }
    static bool segment(const Number* origin,const Number* delta,const AabbT<Number>& box,int64_t& t,int& axis,int& sign,bool sweep,bool& inside) {
        if constexpr(NarrowSegments){
            // Differences of bounded coordinates and every Q24 time fit one
            // native word. Unusual world sizes/deltas retain the wide solver.
            constexpr uint32_t bound=1u<<28;bool narrow=true;
            for(int k=0;k<3;++k)if(uint32_t(origin[k].raw())+bound>bound*2||uint32_t(delta[k].raw())+bound>bound*2||
                uint32_t(box.min[k].raw())+bound>bound*2||uint32_t(box.max[k].raw())+bound>bound*2){narrow=false;break;}
            if(narrow)return segment_width<int32_t>(origin,delta,box,t,axis,sign,sweep,inside);
        }
        return segment_width<int64_t>(origin,delta,box,t,axis,sign,sweep,inside);
    }
public:
    uint32_t dropped_trigger_pairs=0;
    bool has_trigger_pairs() const{return previous_count!=0;}
    void clear() { for(auto& e:entries){e.enabled=false;e.cached=false;}previous_count=0;dropped_trigger_pairs=0;limit=trigger_count=enabled_count=0; }
    void begin_sync() { for(size_t n=0;n<query_count();++n)entries[query_slot(n)].enabled=false;limit=trigger_count=enabled_count=0; }
    void set(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,bool active=true,uint32_t generation=0) {
        if(index>=Capacity)return;
        const bool was_enabled=entries[index].enabled,was_trigger=entries[index].trigger;
        entries[index]={collider_bounds(collider,matrix),collider.layer,collider.mask,generation,active&&collider.enabled,collider.trigger};
        entries[index].slope_rise=collider.slope_rise;
        entries[index].slope_axis=collider.slope_axis<3?collider.slope_axis:0;
        entries[index].heights=collider.heights;
        entries[index].height_cells[0]=collider.height_cells[0];
        entries[index].height_cells[1]=collider.height_cells[1];
        entries[index].height_step=collider.height_step;
        entries[index].boxes=collider.boxes;entries[index].box_count=collider.box_count;
        entries[index].box_grid=collider.box_grid;
        for(int k=0;k<3;++k)entries[index].origin[k]=matrix.values[k][3];
        update_enabled(index,was_enabled,was_trigger);
    }
    // Returns whether bounds were rebuilt. Metadata remains live even when a
    // static box is reused; direct collider edits and slot generations are safe.
    bool set_cached(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,uint32_t revision,bool active=true,uint32_t generation=0) {
        if(index>=Capacity)return false;
        auto& e=entries[index];
        const bool was_enabled=e.enabled,was_trigger=e.trigger;
        e.enabled=active&&collider.enabled;e.trigger=collider.trigger;e.layer=collider.layer;e.mask=collider.mask;e.generation=generation;
        e.slope_rise=collider.slope_rise;e.slope_axis=collider.slope_axis<3?collider.slope_axis:0;
        e.heights=collider.heights;e.height_cells[0]=collider.height_cells[0];
        e.height_cells[1]=collider.height_cells[1];e.height_step=collider.height_step;
        e.boxes=collider.boxes;e.box_count=collider.box_count;
        e.box_grid=collider.box_grid;
        update_enabled(index,was_enabled,was_trigger);
        if(!e.enabled)return false;
        bool dirty=!e.cached||e.world_revision!=revision;
        for(int k=0;k<3;++k){dirty|=e.center[k].raw()!=collider.center[k].raw()||e.half_extents[k].raw()!=collider.half_extents[k].raw();}
        if(dirty){
            e.box=collider_bounds(collider,matrix);e.world_revision=revision;e.cached=true;
            for(int k=0;k<3;++k)e.origin[k]=matrix.values[k][3];
            for(int k=0;k<3;++k){e.center[k]=collider.center[k];e.half_extents[k]=collider.half_extents[k];}
        }
        return dirty;
    }
    const AabbT<Number>* bounds(size_t index) const { return index<Capacity&&entries[index].enabled?&entries[index].box:nullptr; }
    // A root actor's movement changes only translation. Publish that exact
    // world displacement without recomputing its unchanged rotated extents.
    // A later ordinary synchronization still observes every direct source edit.
    void translate(size_t index,const Number* delta){
        if(index>=Capacity)return;auto& e=entries[index];if(!e.enabled)return;
        for(int k=0;k<3;++k){e.box.min[k]+=delta[k];e.box.max[k]+=delta[k];e.origin[k]+=delta[k];}
        e.cached=false;
    }
    // Returns total matches; only the first output_capacity indices are written.
    size_t overlap(const AabbT<Number>& box,uint16_t* output,size_t output_capacity,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=true) const {
        size_t count=0;
        for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,triggers)&&aabb_overlap(box,entries[i].box)) {
            if(compound(entries[i])&&!parts_overlap(entries[i],box))continue;
            if(output&&count<output_capacity)output[count]=uint16_t(i);++count;
        }}
        return count;
    }
    // Segment query: delta is the complete displacement, not a unit direction.
    SpatialHitT<Number> raycast(const Number* origin,const Number* delta,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=false) const {
        SpatialHitT<Number> hit;int64_t best=one+1;
        AabbT<Number> point;for(int k=0;k<3;++k)point.min[k]=point.max[k]=origin[k];
        const auto sweep=swept_bounds(point,delta);
        for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,triggers)&&aabb_may_touch(sweep,entries[i].box)) {
            parts(entries[i],sweep,[&](const AabbT<Number>& box) {
                if(!aabb_may_touch(sweep,box))return;
                int64_t t;int axis,sign;bool inside;
                if(segment(origin,delta,box,t,axis,sign,false,inside)&&t<best) {
                    best=t;hit.entity=int(i);hit.generation=entries[i].generation;hit.started_inside=inside;hit.fraction=raw(int32_t(t*4096/one));
                    for(int k=0;k<3;++k) { hit.point[k]=origin[k]+raw(int32_t(int64_t(delta[k].raw())*t/one));hit.normal[k]=raw(k==axis?sign*4096:0); }
                }
            });
        }}
        return hit;
    }
    void raycast_batch(const RaycastQueryT<Number>* queries,SpatialHitT<Number>* results,size_t count,
                       uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=false) const {
        if(!queries||!results)return;
        for(size_t i=0;i<count;++i)results[i]=raycast(queries[i].origin,queries[i].displacement,mask,ignore,triggers);
    }
    // Downward box sweep uses the complete footprint, including ledges missed by
    // a center ray. distance must be nonnegative; normal points upwards.
    SpatialHitT<Number> ground(const AabbT<Number>& box,Number distance,uint32_t mask=0xffffffffu,int ignore=-1) const {
        SpatialHitT<Number> hit;if(distance<0.0)return hit;
        Number best=distance;
        for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,false)) {
            const auto& outer=entries[i].box;
            if(box.max[0]<=outer.min[0]||box.min[0]>=outer.max[0]||box.max[2]<=outer.min[2]||box.min[2]>=outer.max[2])continue;
            parts(entries[i],box,[&](const AabbT<Number>& b) {
                if(box.max[0]<=b.min[0]||box.min[0]>=b.max[0]||box.max[2]<=b.min[2]||box.min[2]>=b.max[2])return;
                Number probe[3];for(int k=0;k<3;++k)probe[k]=(box.min[k]+box.max[k])/2;
                const auto top=compound(entries[i])?b.max[1]:surface(entries[i],probe);
                auto gap=box.min[1]-top;
                if(gap<0.0||gap>best)return;
                if(hit.entity>=0&&gap==best)return;
                best=gap;hit.entity=int(i);hit.generation=entries[i].generation;hit.normal[1]=1.0;
                hit.fraction=distance>0.0?gap/distance:raw(0);
                hit.point[0]=(box.min[0]+box.max[0])/2;hit.point[1]=top;hit.point[2]=(box.min[2]+box.max[2])/2;
            });
        }}
        return hit;
    }
    MoveResultT<Number> move_and_slide(AabbT<Number> box,const Number* displacement,uint32_t mask=0xffffffffu,int ignore=-1) const {
        MoveResultT<Number> result;Number remaining[3]={displacement[0],displacement[1],displacement[2]};
        // Resolve existing overlap by minimum translation, bounded to 8 passes.
        for(unsigned pass=0;pass<8;++pass) {
            bool found=false;int best_axis=0,best_entity=-1;int64_t best=INT64_MAX;
            for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,false)&&aabb_overlap(box,entries[i].box)) {
                if(ramp(entries[i])) {
                    // Standing on a ramp is the only way to resolve against it:
                    // pushing sideways would let a character be shoved off a
                    // slope it is simply walking up.
                    Number probe[3];for(int k=0;k<3;++k)probe[k]=(box.min[k]+box.max[k])/2;
                    const int64_t up=int64_t(surface(entries[i],probe).raw())-box.min[1].raw()+1;
                    if(up>0&&(!found||abs64(up)<abs64(best))) { found=true;best=up;best_axis=1;best_entity=int(i); }
                    continue;
                }
                parts(entries[i],box,[&](const AabbT<Number>& part) {
                    if(!aabb_overlap(box,part))return;
                    for(int k=0;k<3;++k) {
                        int64_t neg=int64_t(part.min[k].raw())-box.max[k].raw()-1,pos=int64_t(part.max[k].raw())-box.min[k].raw()+1;
                        int64_t d=abs64(neg)<abs64(pos)?neg:pos;
                        if(!found||abs64(d)<abs64(best)) { found=true;best=d;best_axis=k;best_entity=int(i); }
                    }
                });
            }}
            if(!found)break;
            auto shift=raw(int32_t(best));box.min[best_axis]+=shift;box.max[best_axis]+=shift;result.displacement[best_axis]+=shift;
            result.blocked=true;result.entity=best_entity;result.generation=entries[size_t(best_entity)].generation;result.normal[best_axis]=raw(best<0?-4096:4096);
            if(best_axis==1&&best>0)result.grounded=true;
        }
        // A ramp always overlaps the character standing on it, by design; only a
        // solid box still intersecting here is genuinely unresolved.
        for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,false)&&!ramp(entries[i])&&aabb_overlap(box,entries[i].box)&&parts_overlap(entries[i],box)) {
            result.unresolved_overlap=true;return result;
        }}
        // Axis-aligned contact normals need at most three sliding planes.
        for(unsigned pass=0;pass<3;++pass) {
            if(!remaining[0].raw()&&!remaining[1].raw()&&!remaining[2].raw())break;
            const auto sweep=swept_bounds(box,remaining);
            Number center[3],extent[3];
            for(int k=0;k<3;++k) { center[k]=(box.min[k]+box.max[k])/2;extent[k]=(box.max[k]-box.min[k])/2; }
            int64_t best=one+1;int best_axis=-1,best_sign=0,best_entity=-1;
            for(size_t ordinal=0;ordinal<query_count();++ordinal){const size_t i=query_slot(ordinal);if(match(i,mask,ignore,false)&&!ramp(entries[i])&&aabb_may_touch(sweep,entries[i].box)) {
                parts(entries[i],sweep,[&](const AabbT<Number>& part) {
                    if(!aabb_may_touch(sweep,part))return;
                    auto expanded=part;
                    for(int k=0;k<3;++k) { expanded.min[k]-=extent[k];expanded.max[k]+=extent[k]; }
                    int64_t t;int axis,sign;bool inside;
                    if(segment(center,remaining,expanded,t,axis,sign,true,inside)&&axis>=0&&t<best) { best=t;best_axis=axis;best_sign=sign;best_entity=int(i); }
                });
            }}
            // The common unobstructed step needs neither a Q24 multiply nor
            // another pass. This is exactly fraction==one in the path below.
            if(best_entity<0){
                for(int k=0;k<3;++k)result.displacement[k]+=remaining[k];
                break;
            }
            int64_t fraction=best;
            for(int k=0;k<3;++k) {
                auto moved=raw(int32_t(int64_t(remaining[k].raw())*fraction/one));
                box.min[k]+=moved;box.max[k]+=moved;result.displacement[k]+=moved;remaining[k]-=moved;
            }
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
        if(trigger_count)for(size_t na=0;na<query_count();++na)for(size_t nb=na+1;nb<query_count();++nb) {
            const size_t a=query_slot(na),b=query_slot(nb);if(!entries[a].enabled)continue;
            const auto& x=entries[a];const auto& y=entries[b];
            if(!y.enabled||(!x.trigger&&!y.trigger)||!(x.layer&y.mask)||!(y.layer&x.mask)||!aabb_overlap(x.box,y.box)||!touching(x,y))continue;
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
