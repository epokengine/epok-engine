#include "../../runtime/transition.hpp"
#include "../../runtime/time.hpp"
#include <cassert>
#include <cstring>
#include <cstdio>
int main(){
    epok::TransitionState state;
    epok::TransitionOptions options;
    options.fade_out_ms=300;options.fade_in_ms=600;
    char message[]="Entering forest";options.loading.text=message;
    state.begin(options,0xffffff00u);
    message[0]='X';assert(std::strcmp(state.options.loading.text,"Entering forest")==0);
    state.advance(uint32_t(0xffffff00u+150000));
    assert(state.audio_gain==2048&&state.opacity==128);
    state.advance(uint32_t(0xffffff00u+300000));
    assert(state.loading()&&state.audio_gain==0&&state.opacity==255&&!state.presented);
    state.advance(800000);assert(state.loading());
    state.loaded(800000);state.advance(1100000);
    assert(state.audio_gain==2048&&state.opacity==128);
    state.advance(1400000);assert(!state.busy()&&state.audio_gain==4096&&state.opacity==0);
    options.fade_out_ms=options.fade_in_ms=0;
    state.begin(options,42);state.advance(42);assert(state.loading()&&!state.presented);
    state.fail();state.advance(10000000);assert(state.phase==epok::TransitionPhase::Failed&&state.audio_gain==0);
    epok::Time time;time.reset(0);time.advance(20000);time.begin_tick();time.dropped_steps=5;
    time.set_paused(true);time.synchronize(10000000);assert(time.ticks==1&&time.dropped_steps==5&&time.paused());
    time.set_paused(false);assert(time.advance(10000001)==0);
    std::puts("Transition audio/visual timing, wraparound, copied text, failure and simulation pause passed.");
}
