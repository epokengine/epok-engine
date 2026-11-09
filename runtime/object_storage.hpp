#pragma once
#include <cstddef>
#include <cstdint>

namespace epok::object_storage {
inline constexpr size_t unit_bytes=16;
// An authored spawn reserve sizes the arena for its live peak. This margin holds
// what the cook cannot count: the scene script and small runtime-added components.
inline constexpr size_t spawn_margin_bytes=4096;
struct Stats {
    uint32_t used_units=0,peak_units=0,live_allocations=0,peak_allocations=0;
    uint32_t rejected=0,invalid_releases=0;
};
constexpr size_t rounded(size_t bytes){return (bytes+unit_bytes-1)/unit_bytes*unit_bytes;}
constexpr size_t arena_bytes(size_t units){return rounded(units*unit_bytes+2*((units+31)/32)*sizeof(uint32_t)+sizeof(Stats)+sizeof(size_t));}
constexpr size_t capacity_units(size_t fixed,size_t demand,size_t limit=65536){
    size_t units=demand/unit_bytes;
    if(units>limit/unit_bytes)units=limit/unit_bytes;
    while(units&&arena_bytes(units)+fixed>limit)--units;
    return units;
}
template<size_t Units> struct Arena {
    static_assert(Units>0,"Object storage needs a bounded nonempty payload");
    alignas(unit_bytes) unsigned char values[Units*unit_bytes]={};
    uint32_t occupied[(Units+31)/32]={};
    uint32_t starts[(Units+31)/32]={};
    Stats stats{};
    size_t first_free=0;
    bool used(size_t i)const{return (occupied[i/32]&(uint32_t(1)<<(i%32)))!=0;}
    void mark(size_t i,bool value){const auto bit=uint32_t(1)<<(i%32);if(value)occupied[i/32]|=bit;else occupied[i/32]&=~bit;}
    void* acquire(size_t bytes,size_t alignment){
        if(!bytes||bytes>sizeof(values)||!alignment||(alignment&(alignment-1))){++stats.rejected;return nullptr;}
        const size_t count=(bytes+unit_bytes-1)/unit_bytes;
        if(count>Units){++stats.rejected;return nullptr;}
        for(size_t i=first_free;i+count<=Units;++i){
            if(reinterpret_cast<uintptr_t>(values+i*unit_bytes)&(alignment-1))continue;
            size_t j=0;while(j<count&&!used(i+j))++j;
            if(j<count){i+=j;continue;}
            for(j=0;j<count;++j)mark(i+j,true);
            starts[i/32]|=uint32_t(1)<<(i%32);
            stats.used_units+=uint32_t(count);++stats.live_allocations;
            if(stats.used_units>stats.peak_units)stats.peak_units=stats.used_units;
            if(stats.live_allocations>stats.peak_allocations)stats.peak_allocations=stats.live_allocations;
            if(i==first_free)while(first_free<Units&&used(first_free))++first_free;
            return values+i*unit_bytes;
        }
        ++stats.rejected;return nullptr;
    }
    bool release(void* block,size_t bytes){
        const uintptr_t address=reinterpret_cast<uintptr_t>(block),base=reinterpret_cast<uintptr_t>(values);
        if(!bytes||bytes>sizeof(values)||address<base||address>=base+sizeof(values)||(address-base)%unit_bytes){++stats.invalid_releases;return false;}
        const size_t count=(bytes+unit_bytes-1)/unit_bytes;
        const size_t i=(address-base)/unit_bytes;
        auto start=[&](size_t index){return (starts[index/32]&(uint32_t(1)<<(index%32)))!=0;};
        if(count>Units-i||!start(i)){++stats.invalid_releases;return false;}
        for(size_t j=0;j<count;++j)if(!used(i+j)){++stats.invalid_releases;return false;}
        for(size_t j=1;j<count;++j)if(start(i+j)){++stats.invalid_releases;return false;}
        if(i+count<Units&&used(i+count)&&!start(i+count)){++stats.invalid_releases;return false;}
        for(size_t j=0;j<count;++j)mark(i+j,false);
        starts[i/32]&=~(uint32_t(1)<<(i%32));
        stats.used_units-=uint32_t(count);--stats.live_allocations;
        if(i<first_free)first_free=i;
        return true;
    }
};
struct Hooks {
    void* context=nullptr;
    void* (*acquire)(void*,size_t,size_t)=nullptr;
    void (*release)(void*,void*,size_t)=nullptr;
    Stats* statistics=nullptr;
};
inline Hooks hooks{};
template<size_t Units> struct Binding {
    explicit Binding(Arena<Units>& arena){
        hooks={&arena,
            +[](void* context,size_t bytes,size_t alignment)->void*{return static_cast<Arena<Units>*>(context)->acquire(bytes,alignment);},
            +[](void* context,void* block,size_t bytes){static_cast<Arena<Units>*>(context)->release(block,bytes);},&arena.stats};
    }
};
template<size_t Units> Binding(Arena<Units>&)->Binding<Units>;
}
