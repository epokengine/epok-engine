#include <cassert>
#include <cstdint>
#include <cstdio>
#include "psyqo/fixed-point.hh"
using namespace psyqo::FixedPointInternals;
static int32_t reference(int32_t a,int32_t b,unsigned scale){
    const uint64_t n=a<0?uint64_t(-int64_t(a)):uint64_t(a);
    const uint64_t d=b<0?uint64_t(-int64_t(b)):uint64_t(b);
    const uint32_t q=uint32_t(n*scale/d);
    return int32_t((a<0)!=(b<0)?0u-q:q);
}
static uint32_t state=0x67f53ab1;
static uint32_t random_value(){state^=state<<13;state^=state>>17;state^=state<<5;return state;}
static_assert(dDiv(-12000,68,4096)==-722823);
static_assert(dDiv(INT32_MIN,INT32_MIN,4096)==4096);
int main(){
    constexpr int32_t edges[]={INT32_MIN,INT32_MIN+1,-1048576,-1048575,-524288,-1,0,1,524287,1048575,1048576,INT32_MAX};
    for(auto a:edges)for(auto b:edges)if(b)for(unsigned scale:{0u,1u,4096u,65536u,UINT32_MAX})
        assert(dDiv(a,b,scale)==reference(a,b,scale));
    for(unsigned i=0;i<1000000;++i){
        const uint32_t a=random_value(),b=random_value()|1u,scale=(i&1)?4096u:random_value();
        assert(iDiv(a,b,scale)==uint32_t(uint64_t(a)*scale/b));
        const int32_t sa=int32_t(a),sb=int32_t(b);
        assert(dDiv(sa,sb,scale)==reference(sa,sb,scale));
        using Q=psyqo::FixedPoint<12,int32_t>;
        Q x(sa,Q::RAW),y(sb,Q::RAW);
        assert((x/y).raw()==reference(sa,sb,4096));x/=y;assert(x.raw()==reference(sa,sb,4096));
    }
    std::puts("Fixed division: 1,000,000 signed/unsigned ratios, operators and full-range boundaries match wide division.");
}
