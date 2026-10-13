#pragma once
#include <stdint.h>

namespace epok {
namespace fixed_trig_detail {
// Compile-time generation from the mathematical definition, not game assets.
// The argument is restricted to [0, pi/2]; twelve Taylor terms are comfortably
// more accurate than a half Q12 tick throughout this interval.
constexpr double sine_series(double x) {
    double term=x,sum=x;
    for(int n=1;n<12;++n){term*=-x*x/double((2*n)*(2*n+1));sum+=term;}
    return sum;
}
struct QuarterSine {int16_t values[1025]{};};
consteval QuarterSine make_quarter_sine() {
    QuarterSine result;
    for(unsigned i=0;i<=1024;++i)
        result.values[i]=int16_t(sine_series(3.14159265358979323846*double(i)/2048.0)*4096.0+0.5);
    return result;
}
inline constexpr QuarterSine quarter_sine=make_quarter_sine();
static_assert(sizeof(quarter_sine)==2050);
}

// One turn = 4096; result is signed Q12. Exact quadrant boundaries and modular
// arithmetic for any uint32 input. No startup generation or runtime floats.
inline int32_t sine_turn12(uint32_t phase) {
    phase&=4095u;
    unsigned index=phase&1023u;
    if(phase&1024u)index=1024u-index;
    const int32_t magnitude=fixed_trig_detail::quarter_sine.values[index];
    return phase&2048u?-magnitude:magnitude;
}
inline int32_t cosine_turn12(uint32_t phase) {return sine_turn12(phase+1024u);}
}
