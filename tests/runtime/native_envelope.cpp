#include "spu_envelope.hpp"
#include <cassert>
#include <cstdio>
int main(){
    using namespace epok::native_music;
    // Known SPU layout: instant linear attack, full-level infinite sustain.
    Envelope e;e.key_on(0x00ff,0x1fc0);e.advance(100);assert(e.phase==2 && e.level>24000);
    auto hold=e.level;e.advance(44100);assert(e.level==hold);
    e.key_off();e.advance(10);assert(e.done());
    auto full=fit_envelope(0,0,0,32768,100000);e.key_on(full.lo,full.hi);e.advance(44100);assert(e.level>32000);
    // Fitted rates remain monotonic, bounded, and independent of host chunking.
    for(uint32_t release:{0u,10000u,100000u,1000000u,8000000u}){
        auto fit=fit_envelope(20000,0,200000,16384,release);
        assert(!(fit.hi&0x2000));Envelope a,b;a.key_on(fit.lo,fit.hi);b=a;
        a.advance(44100);for(int n=0;n<44100;++n)b.advance(1);
        assert(a.level==b.level && a.phase==b.phase && a.wait==b.wait);
        a.key_off();b.key_off();a.advance(441000);for(int n=0;n<441000;++n)b.advance(1);
        assert(a.done() && b.done());
    }
    std::puts("native_envelope passed: register layout, freeze, release, fitting and chunk invariance");
}
