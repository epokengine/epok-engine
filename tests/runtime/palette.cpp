#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/palette_types.hpp"
#include "../../runtime/time.hpp"
using namespace epok;
void rotation_and_transparency(){
    PaletteAnimator animation;animation.enabled=true;animation.first=1;animation.last=4;animation.speed=8.0;
    uint16_t original[256],rotated[256];for(unsigned i=0;i<256;++i)original[i]=uint16_t(i);original[0]=0;
    animation.advance(0.25);assert(animation.offset==2);animation.apply(original,rotated);
    assert(rotated[0]==0&&rotated[1]==3&&rotated[2]==4&&rotated[3]==1&&rotated[4]==2&&rotated[5]==5);
    animation.reset();animation.advance(0.125);animation.reverse=true;animation.apply(original,rotated);assert(rotated[1]==4&&rotated[4]==3);
    animation.enabled=false;animation.apply(original,rotated);for(unsigned i=0;i<256;++i)assert(original[i]==rotated[i]);
    animation.enabled=true;animation.first=0;animation.advance(1.0);assert(animation.offset==1);animation.apply(original,rotated);assert(rotated[0]==0&&rotated[1]==1);
}
void timing_and_bounds(){
    PaletteAnimator animation;animation.enabled=true;animation.first=1;animation.last=255;animation.speed=7.0;Time clock;
    for(unsigned tick=1;tick<=6000;++tick){clock.begin_tick();animation.advance(Fixed(clock.delta_raw,Fixed::RAW));if(tick%60==0)assert(animation.offset==uint16_t((tick/60*7)%255));}
    animation.reset();assert(animation.offset==0&&animation.remainder==0);
    animation.speed=-1.0;animation.advance(10.0);assert(animation.offset==0);animation.speed=60.0;animation.advance(-10.0);assert(animation.offset==0);
    animation.advance(Fixed(INT32_MAX,Fixed::RAW));assert(animation.offset<255&&animation.remainder<16777216u);
}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    rotation_and_transparency();timing_and_bounds();std::puts("Q12 palette timing, reverse cycling and transparent index preservation passed.");
}
