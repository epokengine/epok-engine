#pragma once
#include "blueprint_runtime.hpp"
#if defined(_MSC_VER)
#include <intrin.h>
#endif

#if defined(EPOK_BLUEPRINT_TRACE) && EPOK_BLUEPRINT_TRACE
namespace epok::bp {
// Stable debugger ABI: 119 little-endian 32-bit words (476 bytes). Values are
// copied by generated typed accessors, never by host-computed member offsets.
struct DebugEntry { uint32_t member=0,type=0,length=0,value[4]={}; };
struct DebugSnapshot {
    uint32_t magic=0x55425144u,version=1,class_id=0,node_id=0;
    uint32_t owner_index=0,owner_generation=0,count=0;
    DebugEntry entries[16] = {};
};
static_assert(sizeof(DebugEntry)==28 && sizeof(DebugSnapshot)==476);
}
extern "C" {
inline epok::bp::DebugSnapshot epok_blueprint_debug_snapshot;
// A real execution breakpoint on this symbol pauses before returning to the
// still-live native graph stack. The barrier makes it observable to optimizers.
#if defined(_MSC_VER)
__declspec(noinline) inline void epok_blueprint_debug_hook() { _ReadWriteBarrier(); }
#else
__attribute__((noinline,used)) inline void epok_blueprint_debug_hook() { asm volatile("" ::: "memory"); }
#endif
}
namespace epok::bp {
inline void debug_begin(uint32_t class_id,uint32_t node_id,ObjectId owner) {
    auto& snapshot=epok_blueprint_debug_snapshot;
    snapshot.class_id=class_id;snapshot.node_id=node_id;snapshot.owner_index=owner.index;
    snapshot.owner_generation=owner.generation;snapshot.count=0;
}
inline void debug_value(uint32_t member,uint32_t type,uint32_t length,uint32_t a=0,uint32_t b=0,uint32_t c=0,uint32_t d=0) {
    auto& snapshot=epok_blueprint_debug_snapshot;if(snapshot.count>=16)return;
    snapshot.entries[snapshot.count++]={member,type,length,{a,b,c,d}};
}
}
#endif
