#pragma once
#include "ordered_draw_types.hpp"
#include "draw_order.hpp"

namespace epok {
// A prepared list is not published until the camera/geometry transaction commits.
// Three buffers are intentional: arbitrary seeks need TWO decode buffers while
// retaining the currently displayed list, including on a malformed later delta.
template<size_t Capacity> class OrderedDrawCursor {
    static_assert(Capacity>0&&Capacity<=65535);
    uint32_t lists_[3][Capacity]{};
    const OrderedDrawData* owner_=nullptr;
    const OrderedDrawData* pending_owner_=nullptr;
    uint16_t active_sample_=0xffff,pending_sample_=0xffff;
    uint16_t active_count_=0,pending_count_=0;
    uint8_t active_=0,pending_=1;
    static uint16_t half(const uint8_t* p){return uint16_t(p[0])|(uint16_t(p[1])<<8);}
    static uint32_t word(const uint8_t* p){return uint32_t(half(p))|(uint32_t(half(p+2))<<16);}
public:
    void reset(){owner_=pending_owner_=nullptr;active_sample_=pending_sample_=0xffff;active_count_=pending_count_=0;active_=0;pending_=1;}
    const uint32_t* tokens() const {return active_sample_==0xffff?nullptr:lists_[active_];}
    size_t count() const {return active_count_;}
    uint16_t sample() const {return active_sample_;}
    const OrderedDrawData* owner() const {return owner_;}
    bool publish(const OrderedDrawData* data,uint16_t sample){
        if(!data||sample>=data->sample_count)return false;
        if(data==owner_&&sample==active_sample_)return true;
        if(data!=pending_owner_||sample!=pending_sample_)return false;
        owner_=data;active_=pending_;active_sample_=sample;active_count_=pending_count_;pending_sample_=0xffff;pending_owner_=nullptr;
        return true;
    }
    // page() only returns verified, already resident data. The caller separately
    // checks/pins the sample's ENTIRE page set before publishing its camera.
    template<class Page> bool prepare(const OrderedDrawData* data,uint16_t sample,Page&& page){
        if(!data||!data->blocks||!data->sample_blocks||sample>=data->sample_count||data->capacity>Capacity)return false;
        if((data==owner_&&sample==active_sample_)||(data==pending_owner_&&sample==pending_sample_))return true;
        pending_sample_=0xffff;pending_owner_=nullptr;
        const auto block_index=data->sample_blocks[sample];if(block_index>=data->block_count)return false;
        const auto& block=data->blocks[block_index];
        if(block.first>sample||sample-block.first>=32||block.offset>65528||(block.offset&31))return false;
        const auto* bytes=page(block.page);if(!bytes)return false;
        const auto* base=bytes+block.offset;const size_t available=65536-size_t(block.offset);
        if(word(base)!=0x3152444fu)return false;
        const size_t rows=half(base+4),header=half(base+6),target=sample-block.first;
        if(!rows||rows>32||target>=rows||header!=8+rows*8||header>available)return false;
        const uint8_t scratch[2]={uint8_t((active_+1)%3),uint8_t((active_+2)%3)};
        size_t first=0,previous_count=0;const uint32_t* previous=nullptr;unsigned destination=0;
        // The hot forward case is one delta. Jumps/cuts/reverse start at the
        // block's checkpoint and alternate only the non-active scratch buffers.
        if(owner_==data&&active_sample_!=0xffff&&sample==active_sample_+1&&target
            &&data->sample_blocks[active_sample_]==block_index){
            first=target;previous=lists_[active_];previous_count=active_count_;
        }
        size_t count=0;
        for(size_t row=first;row<=target;++row){
            const auto* entry=base+8+row*8;
            const size_t offset=half(entry),length=half(entry+2);count=half(entry+4);
            const auto flags=half(entry+6);
            if(flags!=(row==0?1:0)||offset<header||offset>available||length>available-offset||count>data->capacity)return false;
            if(!decode_draw_order(base+offset,length,previous,previous_count,lists_[scratch[destination]],Capacity,count))return false;
            previous=lists_[scratch[destination]];previous_count=count;destination^=1;
        }
        pending_owner_=data;pending_=scratch[destination^1];pending_sample_=sample;pending_count_=uint16_t(count);
        return true;
    }
};
}
