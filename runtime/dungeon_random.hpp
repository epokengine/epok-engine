#pragma once
#include <stdint.h>

namespace epok::dungeon {
// The dungeon generator's random stream, bit-identical to src/dungeon_random.rs:
// xoshiro128** seeded through a SplitMix32 sequence. Only 32-bit operations and
// one 32x32->64 multiply, which the R3000 performs natively.
class Random {
public:
    explicit Random(uint32_t seed) {
        uint32_t weyl=seed;
        for(auto& word:state_){
            weyl+=0x9e3779b9u;uint32_t z=weyl;
            z=(z^(z>>16))*0x85ebca6bu;z=(z^(z>>13))*0xc2b2ae35u;word=z^(z>>16);
        }
        if(!(state_[0]|state_[1]|state_[2]|state_[3]))state_[0]=1;
    }
    // An independent stream for one stage of generation.
    static Random stream(uint32_t seed,uint32_t stage) { return Random(seed^rotl(stage*0x9e3779b9u,16)); }
    uint32_t next() {
        uint32_t* s=state_;
        const uint32_t result=rotl(s[1]*5u,7)*9u,t=s[1]<<9;
        s[2]^=s[0];s[3]^=s[1];s[1]^=s[2];s[0]^=s[3];s[2]^=t;s[3]=rotl(s[3],11);
        return result;
    }
    // Uniform in [0, n) without modulo bias; 0 when n is 0.
    uint32_t below(uint32_t n) {
        if(!n)return 0;
        uint64_t product=uint64_t(next())*n;
        if(uint32_t(product)<n){
            const uint32_t threshold=(0u-n)%n;
            while(uint32_t(product)<threshold)product=uint64_t(next())*n;
        }
        return uint32_t(product>>32);
    }
    // Uniform in [low, high]; low when the range is empty.
    int32_t range(int32_t low,int32_t high) {
        if(high<=low)return low;
        return int32_t(uint32_t(low)+below(uint32_t(int64_t(high)-low+1)));
    }
    bool chance(uint32_t percent) { return below(100)<(percent<100?percent:100); }
private:
    static uint32_t rotl(uint32_t x,int k) { return (x<<k)|(x>>(32-k)); }
    uint32_t state_[4];
};
}
