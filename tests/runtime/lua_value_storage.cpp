#include "../../runtime/lua_value_storage.hpp"
#include <cassert>
#include <cstdio>

struct Value {uint32_t count=32;bool composite=true;int32_t words[32]{};};
using epok::lua::detail::ValueStorage;

int main(){
    ValueStorage<Value,16,64> full;
    const Value* handles[16][64]{};
    // Every supported depth can keep all64 maximum-width values concurrently.
    for(size_t frame=0;frame<16;++frame){
        full.begin(frame);
        for(size_t slot=0;slot<64;++slot){
            Value value;value.words[0]=int32_t(frame*64+slot);value.words[31]=~value.words[0];
            handles[frame][slot]=full.retain(frame,value);
            assert(handles[frame][slot]&&full.contains(frame,handles[frame][slot]));
        }
        assert(!full.retain(frame,Value{}));
    }
    for(size_t frame=0;frame<16;++frame)for(size_t slot=0;slot<64;++slot){
        assert(handles[frame][slot]->words[0]==int32_t(frame*64+slot));
        assert(handles[frame][slot]->words[31]==~int32_t(frame*64+slot));
        assert(!full.contains((frame+1)%16,handles[frame][slot]));
    }
    // Reuse a small proven plan without moving live aliases. Foreign/stale
    // frame pointers cannot become roots or displace another live snapshot.
    ValueStorage<Value,16,3> compact;
    Value root;root.words[0]=123;root.words[31]=456;
    compact.begin(0);const auto* saved=compact.retain(0,root);const auto* alias=saved;
    for(unsigned iteration=0;iteration<4096;++iteration){
        compact.sweep(0,compact.root(0,saved)|compact.root(0,alias));
        Value transient;transient.words[0]=int32_t(iteration);
        const auto* first=compact.retain(0,transient);
        const auto* second=compact.retain(0,transient);
        assert(first&&second&&!compact.retain(0,transient));
        assert(saved==alias&&saved->words[0]==123&&saved->words[31]==456);
        for(size_t frame=1;frame<16;++frame){
            compact.begin(frame);
            const auto* nested=compact.retain(frame,*saved);
            assert(nested&&nested!=saved&&!compact.root(frame,saved));
            compact.sweep(frame,compact.root(frame,nested));
            assert(nested->words[0]==123&&saved->words[31]==456);
        }
    }
    compact.sweep(0,0);assert(!compact.contains(0,saved));
    assert(!compact.root(0,&root));assert(!compact.retain(16,root));
    compact.begin(0);assert(compact.retain(0,root));
    std::puts("Lua wire storage:16depth x64values x32words;4096 alias/loop/re-entry rounds passed");
}
