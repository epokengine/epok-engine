#pragma once
#include <stdint.h>
#include <stddef.h>
#include <limits.h>
namespace epok {
struct ChunkBasisBounds {
    int32_t center[3], extent[3];
};
struct ChunkVisibilityCache {
    uint32_t* combined;
    uint32_t* bounds_valid = nullptr;
    uint32_t* bounds_result = nullptr;
    ChunkBasisBounds* basis_bounds = nullptr;
    uint32_t* basis_valid = nullptr;
    int64_t planes[6][4] = {};
    const uint32_t* masks[6] = {};
    bool valid[6] = {};
    unsigned combined_count = 0;
    bool any_rejection = false;
    int32_t matrix[12] = {};
    bool matrix_valid = false;
    bool dirty = false;
    bool matrix_observed = false;
};
// Offline conservative interval masks for planes in object-local coordinates.
// The dominant component is exactly +/-1; the other two components are in
// [-1,-.5], [-.5,.5], or [.5,1]. Offset slabs include their entire upper edge.
struct ChunkVisibility {
    const uint16_t* rows;
    const uint32_t* masks;
    uint16_t chunk_count, words;
    int32_t offset_min, offset_step;
    uint8_t slab_count;
    ChunkVisibilityCache* cache = nullptr;
};
struct ChunkVisibilityMask {
    const uint32_t* words=nullptr;
    size_t chunk_count=0;
    bool candidate(size_t chunk) const {
        return !words || chunk>=chunk_count || (words[chunk>>5]&(uint32_t(1)<<(chunk&31)));
    }
};
class ChunkVisibilityQuery {
    const ChunkVisibility* grid;
    const uint32_t* masks[6] = {};
    unsigned count = 0;
    bool changed = false;
    mutable bool prepared = false;
    bool matrix_pending = false;
    bool bounds_active = false;
    bool basis_active = false;
    bool bypass = false;
    const uint32_t* lookup(int64_t x,int64_t y,int64_t z,int64_t d) const {
        if(!grid || grid->offset_step<=0 || !grid->slab_count) return nullptr;
        // Reject unsupported offsets before adding the rounding guard.
        if(d<int64_t(INT32_MIN)-256 || d>int64_t(INT32_MAX)-256)return nullptr;
        d+=256;
        const int64_t n[3]={x,y,z};int64_t peak=0;unsigned dominant=0;
        for(unsigned i=0;i<3;++i){auto v=n[i];if(v<INT32_MIN || v>INT32_MAX)return nullptr;int64_t a=v<0?-v:v;if(a>peak){peak=a;dominant=i;}}
        if(!peak || d<INT32_MIN || d>INT32_MAX)return nullptr;
        unsigned direction=0;
        for(unsigned i=0;i<3;++i)if(i!=dominant){auto v=n[i];unsigned digit=1;if(2*(v<0?-v:v)>=peak)digit=v<0?0:2;direction=direction*3+digit;}
        direction+=(dominant*2+(n[dominant]>0?1:0))*9;
        const int64_t numerator=d*4096-int64_t(grid->offset_min)*peak;
        if(numerator<0)return nullptr;
        const int64_t denominator=peak*grid->offset_step;
        // Ordinary scene coefficients fit unsigned32 here even when the
        // positive shifted numerator exceeds INT32_MAX. Avoid software div64.
        const int64_t slab=(uint64_t(numerator)<=UINT32_MAX && uint64_t(denominator)<=UINT32_MAX)
            ? uint32_t(numerator)/uint32_t(denominator) : numerator/denominator;
        if(slab>=grid->slab_count)return nullptr;
        return grid->masks+size_t(grid->rows[direction*grid->slab_count+size_t(slab)])*grid->words;
    }
public:
    explicit ChunkVisibilityQuery(const ChunkVisibility* value):grid(value){}
    // Compare the source matrix before constructing its five derived planes.
    // Call merged() after rebuilding so a partial query cannot validate a key.
    bool reuse_matrix(const int32_t rows[3][3],const int32_t translation[3],
                      bool defer_until_stable=false) {
        if(!grid || !grid->cache)return true;
        auto* cache=grid->cache;
        bool same=cache->matrix_valid || (defer_until_stable && cache->matrix_observed);
        // Translation changes on ordinary movement; stop at its first mismatch.
        for(unsigned r=0;r<3 && same;++r)if(cache->matrix[r*4+3]!=translation[r])same=false;
        for(unsigned r=0;r<3 && same;++r)
            for(unsigned c=0;c<3 && same;++c)if(cache->matrix[r*4+c]!=rows[r][c])same=false;
        if(same)basis_active=cache->basis_bounds && cache->basis_valid;
        if(same && cache->matrix_valid){count=cache->combined_count;prepared=true;bounds_active=cache->bounds_valid && cache->bounds_result;return true;}
        if(same) {matrix_pending=true;return false;} // First repeated key builds its masks.
        // Translation leaves these exact basis products unchanged. Preserve
        // them across camera/object movement, but never rotation, scale or FOV.
        if(cache->basis_valid) {
            bool same_basis=cache->matrix_observed;
            for(unsigned r=0;r<3 && same_basis;++r)
                for(unsigned c=0;c<3 && same_basis;++c)
                    if(cache->matrix[r*4+c]!=rows[r][c])same_basis=false;
            basis_active=same_basis && cache->basis_bounds;
            if(!same_basis)for(unsigned word=0;word<grid->words;++word)cache->basis_valid[word]=0;
        }
        for(unsigned r=0;r<3;++r) {
            for(unsigned c=0;c<3;++c)cache->matrix[r*4+c]=rows[r][c];
            cache->matrix[r*4+3]=translation[r];
        }
        cache->matrix_observed=true;
        cache->matrix_valid=false;matrix_pending=!defer_until_stable;
        if(cache->bounds_valid)for(unsigned word=0;word<grid->words;++word)cache->bounds_valid[word]=0;
        // The renderer can bypass lookup during movement. Observing a key is
        // not publishing a mask: merged must never expose the previous view.
        if(defer_until_stable){bypass=true;return true;}
        return false;
    }
    bool bounds_cache_active() const {return bounds_active;}
    bool basis_cache_active() const {return basis_active;}
    const ChunkBasisBounds* cached_basis_bounds(size_t chunk) const {
        if(!basis_active || chunk>=grid->chunk_count)return nullptr;
        const auto* cache=grid->cache;
        if(!(cache->basis_valid[chunk>>5]&(uint32_t(1)<<(chunk&31))))return nullptr;
        return cache->basis_bounds+chunk;
    }
    void remember_basis_bounds(size_t chunk,const int32_t center[3],const int32_t extent[3]) const {
        if(!basis_active || chunk>=grid->chunk_count)return;
        auto* cache=grid->cache;
        auto& value=cache->basis_bounds[chunk];
        for(unsigned r=0;r<3;++r){value.center[r]=center[r];value.extent[r]=extent[r];}
        cache->basis_valid[chunk>>5]|=uint32_t(1)<<(chunk&31);
    }
    int cached_bounds(size_t chunk) const {
        if(!bounds_active || chunk>=grid->chunk_count)return -1;
        const auto* cache=grid->cache;
        const uint32_t bit=uint32_t(1)<<(chunk&31);
        if(!(cache->bounds_valid[chunk>>5]&bit))return -1;
        return (cache->bounds_result[chunk>>5]&bit)?1:0;
    }
    void remember_bounds(size_t chunk,bool accepted) const {
        if(!bounds_active || chunk>=grid->chunk_count)return;
        auto* cache=grid->cache;
        const uint32_t bit=uint32_t(1)<<(chunk&31);
        if(accepted)cache->bounds_result[chunk>>5]|=bit;else cache->bounds_result[chunk>>5]&=~bit;
        cache->bounds_valid[chunk>>5]|=bit;
    }
    // x*local.x + y*local.y + z*local.z + d*4096 >= 0, all in Q12.
    // Unsupported ranges retain the normal renderer's full bounds test.
    void add_plane(int64_t x,int64_t y,int64_t z,int64_t d) {
        if(!grid || count==6)return;
        // Guard Q12->Q8 translation/MAC rounding for the renderer's z, z +/- x
        // and 3z +/- 4y planes. Export bounds also include local truncation.
        auto* cache=grid->cache;
        if(cache) {
            auto* old=cache->planes[count];
            if(!cache->valid[count] || old[0]!=x || old[1]!=y || old[2]!=z || old[3]!=d) {
                old[0]=x;old[1]=y;old[2]=z;old[3]=d;
                cache->masks[count]=lookup(x,y,z,d);cache->valid[count]=true;changed=true;
                if(cache->matrix_valid && cache->bounds_valid)
                    for(unsigned word=0;word<grid->words;++word)cache->bounds_valid[word]=0;
                cache->matrix_valid=false;
                cache->dirty=true;
            }
            masks[count]=cache->masks[count];
        } else masks[count]=lookup(x,y,z,d);
        ++count;prepared=false;
    }
    // Generated metadata always supplies a cache. A null result means the
    // current view has no useful rejection; the renderer can bypass bit tests.
    const uint32_t* merged_mask() const {
        if(bypass)return nullptr;
        if(grid) if(auto* cache=grid->cache) {
            if(!prepared) {
                if(changed || cache->dirty || cache->combined_count!=count) {
                    cache->matrix_valid=false;
                    cache->any_rejection=false;
                    for(unsigned word=0;word<grid->words;++word) {
                        uint32_t value=UINT32_MAX;
                        for(unsigned i=0;i<count;++i)if(masks[i])value&=masks[i][word];
                        cache->combined[word]=value;
                        const unsigned remaining=grid->chunk_count-word*32;
                        const uint32_t active=remaining>=32?UINT32_MAX:(uint32_t(1)<<remaining)-1;
                        if((value&active)!=active)cache->any_rejection=true;
                    }
                    cache->combined_count=count;
                    cache->dirty=false;
                }
                prepared=true;
            }
            if(matrix_pending)cache->matrix_valid=true;
            return cache->any_rejection ? cache->combined : nullptr;
        }
        return nullptr;
    }
    ChunkVisibilityMask merged() const {return {merged_mask(),grid?size_t(grid->chunk_count):size_t(0)};}
    bool candidate(size_t chunk) const {
        if(!grid || chunk>=grid->chunk_count)return true;
        if(grid->cache) {
            const auto* combined=merged_mask();
            return !combined || (combined[chunk>>5]&(uint32_t(1)<<(chunk&31)));
        }
        const uint32_t bit=uint32_t(1)<<(chunk&31);
        for(unsigned i=0;i<count;++i)if(masks[i] && !(masks[i][chunk>>5]&bit))return false;
        return true;
    }
};
}
