#pragma once
#include <stdint.h>
// Shared Q12 integer primitives. Dependency-free on purpose: the stateless
// timeline kernel, the lighting normaliser and the gameplay utilities all reach
// the same bit-exact results without one of them including the others.
namespace epok::fixed_math {
inline constexpr int32_t one=4096;
// Truncating integer square root. Restoring, two bits per step, no floats.
inline uint32_t sqrt64(uint64_t v){uint64_t result=0,bit=uint64_t(1)<<62;while(bit>v)bit>>=2;while(bit){if(v>=result+bit){v-=result+bit;result=(result>>1)+bit;}else result>>=1;bit>>=2;}return uint32_t(result);}
// `t` raised to `n` in Q12. Truncating once per multiply keeps the error inside
// two raw units up to the fifth power; scaling after the truncation does not.
inline int32_t powq(int32_t t,int n){int64_t acc=t;for(int i=1;i<n;++i)acc=acc*t/one;return int32_t(acc);}
// Square root of a Q12 value, in Q12. Negatives clamp instead of wrapping.
inline int32_t q_sqrt(int32_t raw){return raw<=0?0:int32_t(sqrt64(uint64_t(raw)*one));}
}
