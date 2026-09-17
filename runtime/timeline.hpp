#pragma once
#include <stdint.h>
#include <stddef.h>

// Shared cooked timeline evaluation kernel. Descriptive IDs, bindings and
// reflection stay on the host. The director (not this stateless kernel) owns
// lifetime, activation, restoration and the existing DataHandle checks.
namespace epok::timeline {
inline constexpr size_t key_limit = 256;
struct Key { int32_t tick, value; };
enum class Interpolation:uint8_t {Linear,Step,Smoothstep,EaseIn,EaseOut};
struct Curve { const Key* keys; uint16_t count; Interpolation interpolation=Interpolation::Linear; bool unsigned_values=false; };
struct Marker { int32_t tick; uint16_t id; };
struct Signal { int32_t tick; uint16_t index; bool event; };
struct Argument { int32_t lanes[4]; uint64_t resource; int16_t slot; };

constexpr int32_t saturate(int64_t value) {
    return value > INT32_MAX ? INT32_MAX : value < INT32_MIN ? INT32_MIN : int32_t(value);
}
// Same signed division toward zero and wide intermediate as bp::Timeline.
// Ticks and values are raw Q12. Do not quantize alpha before interpolation.
inline int32_t sample(Curve curve, int32_t tick) {
    if (!curve.keys || !curve.count || curve.count > key_limit) return 0;
    if (tick <= curve.keys[0].tick) return curve.keys[0].value;
    uint16_t low=1,high=curve.count;
    while(low<high){const auto mid=uint16_t(low+(high-low)/2);if(curve.keys[mid].tick<=tick)low=mid+1;else high=mid;}
    if (low < curve.count) {
        const auto i=low;
        const auto& a = curve.keys[i - 1];
        const auto& b = curve.keys[i];
        if (tick < b.tick) {
            const int64_t span = int64_t(b.tick) - a.tick;
            if (span <= 0) return a.value; // Invalid cooked data never divides by zero.
            if(curve.interpolation==Interpolation::Step)return a.value;
            const int64_t x=curve.unsigned_values?int64_t(uint32_t(a.value)):int64_t(a.value);
            const int64_t y=curve.unsigned_values?int64_t(uint32_t(b.value)):int64_t(b.value);
            const int64_t part=int64_t(tick)-a.tick;
            int64_t value;
            if(curve.interpolation==Interpolation::Linear)value=x+(y-x)*part/span;
            else{
                const int64_t t=part*4096/span;
                int64_t eased=t;
                switch(curve.interpolation){
                    case Interpolation::Smoothstep:eased=t*t*(12288-2*t)/(4096*4096);break;
                    case Interpolation::EaseIn:eased=t*t/4096;break;
                    case Interpolation::EaseOut:eased=4096-(4096-t)*(4096-t)/4096;break;
                    default:break;
                }
                value=x+(y-x)*eased/4096;
            }
            return curve.unsigned_values?int32_t(uint32_t(value<0?0:value>UINT32_MAX?UINT32_MAX:value)):saturate(value);
        }
    }
    return curve.keys[curve.count - 1].value;
}
// Sorted tables use (tick, persistent marker ID) at cook time. The caller stores
// one cursor per playback, resets it only on restart, and supplies clamped time.
// Zero-time markers are dispatched on the first positive advance. Polling at
// the same time cannot repeat a marker. No callback or side effect is retained.
inline bool poll_marker(const Marker* markers, uint16_t count, uint16_t& cursor,
                        int32_t tick, uint16_t& id) {
    if (!markers || cursor >= count || markers[cursor].tick > tick) return false;
    id = markers[cursor++].id;
    return true;
}
}
