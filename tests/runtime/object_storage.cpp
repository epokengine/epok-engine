#include <cassert>
#include <cstdint>
#include <cstdio>
#include "../../runtime/object_storage.hpp"
using namespace epok::object_storage;
int main(){
    constexpr auto units=capacity_units(4096,200000);
    static_assert(sizeof(Arena<units>)+4096<=65536);
    static_assert(sizeof(Arena<units+1>)+4096>65536);
    static_assert(capacity_units(0,128)==8);
    Arena<16> arena;
    assert(!arena.acquire(0,16)&&!arena.acquire(17,3)&&!arena.acquire(SIZE_MAX,16));
    void* a=arena.acquire(17,16);void* b=arena.acquire(16,16);void* c=arena.acquire(64,64);
    assert(a&&b&&c&&reinterpret_cast<uintptr_t>(c)%64==0);
    assert(!arena.release(static_cast<unsigned char*>(a)+16,16));
    assert(!arena.release(a,16)&&!arena.release(a,48));
    assert(arena.release(a,17)&&!arena.release(a,17));
    assert(arena.release(b,16)&&arena.release(c,64));
    assert(arena.stats.used_units==0&&arena.stats.live_allocations==0);
    assert(arena.stats.peak_units==7&&arena.stats.peak_allocations==3);
    // Freed neighboring blocks support a later larger allocation.
    void* blocks[16];for(auto& block:blocks){block=arena.acquire(16,16);assert(block);}
    assert(!arena.acquire(1,1));
    for(unsigned i=4;i<12;++i)assert(arena.release(blocks[i],16));
    void* large=arena.acquire(128,16);assert(large==blocks[4]);
    assert(arena.release(large,128));
    for(unsigned i=0;i<16;++i)if(i<4||i>=12)assert(arena.release(blocks[i],16));
    // Repeat mixed-size lifetimes to catch bitmap/cursor leaks.
    for(unsigned pass=0;pass<10000;++pass){
        void* x=arena.acquire(48,16);void* y=arena.acquire(80,16);
        assert(x&&y&&arena.release(x,48)&&arena.release(y,80));
    }
    assert(arena.stats.used_units==0&&arena.stats.live_allocations==0&&arena.first_free==0);
    std::puts("object storage: alignment, bounds, invalid frees, exhaustion, coalescing and reuse passed");
}
