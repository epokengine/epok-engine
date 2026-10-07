#include "streaming_pool.hpp"
#include <cassert>
#include <cstdio>

static uint32_t scalar(uint32_t hash,const uint8_t* bytes,size_t count){
    for(size_t i=0;i<count;++i)hash=(hash^bytes[i])*16777619u;
    return hash;
}
int main(){
    assert(epok::stream_page_hash(nullptr,0)==2166136261u);
    assert(epok::stream_page_hash(reinterpret_cast<const uint8_t*>("a"),1)==0xe40c292cu);
    assert(epok::stream_page_hash(reinterpret_cast<const uint8_t*>("foobar"),6)==0xbf9cf968u);
    alignas(16) uint8_t data[65536+32];uint32_t random=76381;
    const auto next=[&](){random=random*1664525u+1013904223u;return random;};
    for (unsigned bit=0;bit<32;++bit) {
        const uint32_t value=uint32_t(1)<<bit;
        assert(epok::stream_hash_multiply(value)==value*16777619u);
        assert(epok::stream_hash_multiply(~value)==(~value)*16777619u);
    }
    for(unsigned i=0;i<100000;++i) {
        const auto value=next();
        assert(epok::stream_hash_multiply(value)==value*16777619u);
    }
    for(auto& byte:data)byte=uint8_t(next()>>19);
    // Every alignment, every short tail, arbitrary incoming hash and split.
    unsigned cases=0;
    for(unsigned offset=0;offset<16;++offset)for(unsigned count=0;count<=257;++count){
        const auto* input=data+offset;const auto seed=next();
        const auto expected=scalar(seed,input,count);
        assert(epok::stream_page_hash_update(seed,input,count)==expected);
        const auto split=count?next()%(count+1):0;
        const auto prefix=epok::stream_page_hash_update(seed,input,split);
        assert(epok::stream_page_hash_update(prefix,input+split,count-split)==expected);
        ++cases;
    }
    for(unsigned sample=0;sample<20000;++sample){
        const size_t offset=next()%16,count=next()%4097,split=next()%(count+1);
        const auto seed=next();const auto expected=scalar(seed,data+offset,count);
        const auto prefix=epok::stream_page_hash_update(seed,data+offset,split);
        assert(epok::stream_page_hash_update(prefix,data+offset+split,count-split)==expected);
        assert(epok::stream_page_hash_update(seed,data+offset,count)==expected);++cases;
    }
    for(unsigned offset=0;offset<16;++offset){
        const auto* input=data+offset;const auto expected=scalar(2166136261u,input,65536);
        auto actual=2166136261u;
        for(unsigned position=0;position<65536;position+=4096)
            actual=epok::stream_page_hash_update(actual,input+position,4096);
        assert(actual==expected&&epok::stream_page_hash(input)==expected);++cases;
    }
    std::printf("FNV1a: %u scalar/alignment/tail/split/page cases preserve every byte and hash.\n",cases);
}
