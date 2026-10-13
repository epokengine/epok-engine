#pragma once
#include <stddef.h>
#include <stdint.h>

namespace epok {
// Ordered sequence codec shared with the offline cooker. Input pages must stay
// pinned until decoding finishes. Decode into a separate scratch list; publish
// it only on success. A partial result is never an active draw list.
struct DrawOrderSpan { uint32_t offset,bytes; uint16_t count; bool reset; };
static_assert(sizeof(DrawOrderSpan)==12);
inline bool draw_order_varint(const uint8_t*& cursor,const uint8_t* end,uint32_t& value) {
    value=0;
    for(unsigned shift=0;shift<=28;shift+=7){
        if(cursor==end)return false;
        const uint8_t byte=*cursor++;
        if(shift==28&&(byte&127)>1)return false;
        value|=uint32_t(byte&127)<<shift;
        if(!(byte&128))return !shift || (byte&127)!=0;
    }
    return false;
}
inline bool decode_draw_order(const uint8_t* data,size_t bytes,
    const uint32_t* previous,size_t previous_count,uint32_t* output,size_t capacity,size_t expected) {
    if(previous_count>65535||capacity>65535||expected>capacity||(!previous&&previous_count)
        ||(!output&&expected)||(!data&&bytes))return false;
    // In-place copies of moved ranges could corrupt unread previous elements.
    const uintptr_t a=reinterpret_cast<uintptr_t>(previous),b=reinterpret_cast<uintptr_t>(output);
    if(previous_count&&expected&&a<b+expected*4&&b<a+previous_count*4)return false;
    if(!bytes)return expected==0;
    const uint8_t* cursor=data;const uint8_t* end=data+bytes;
    size_t written=0;
    while(cursor!=end){
        uint32_t command;
        if(!draw_order_varint(cursor,end,command))return false;
        const size_t count=(command>>1)+1;
        if(count>expected-written)return false;
        if(command&1){
            for(size_t i=0;i<count;++i){uint32_t value;if(!draw_order_varint(cursor,end,value))return false;output[written++]=value;}
        }else{
            uint32_t first;
            if(!draw_order_varint(cursor,end,first)||first>previous_count||count>previous_count-first)return false;
            for(size_t i=0;i<count;++i)output[written++]=previous[first+i];
        }
    }
    return written==expected;
}
}
