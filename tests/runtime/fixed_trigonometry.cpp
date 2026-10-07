#include <cassert>
#include <cmath>
#include <cstdio>
#include "fixed_trigonometry.hpp"

int main(){
    constexpr double pi=3.14159265358979323846;
    uint32_t hash=2166136261u;
    for(unsigned phase=0;phase<4096;++phase){
        const int sine=epok::sine_turn12(phase),cosine=epok::cosine_turn12(phase);
        assert(sine==int(std::round(std::sin(phase*pi/2048.0)*4096)));
        assert(cosine==int(std::round(std::cos(phase*pi/2048.0)*4096)));
        assert(sine==-epok::sine_turn12(0u-phase));
        assert(cosine==epok::cosine_turn12(0u-phase));
        assert(sine==epok::sine_turn12(phase+4096u));
        assert(sine==epok::sine_turn12(phase+0xfffff000u));
    }
    assert(epok::sine_turn12(0)==0&&epok::sine_turn12(1024)==4096);
    assert(epok::sine_turn12(2048)==0&&epok::sine_turn12(3072)==-4096);
    for(auto value:epok::fixed_trig_detail::quarter_sine.values){
        hash=(hash^uint8_t(value))*16777619u;
        hash=(hash^uint8_t(uint16_t(value)>>8))*16777619u;
    }
    std::printf("4096 sine/cosine phases, exact cardinal angles and wrapping passed; quarter bytes=2050 FNV=%08x\n",hash);
}
