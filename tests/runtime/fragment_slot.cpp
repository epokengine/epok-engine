#include "fragment_slot.hpp"
#include "baked_packet.hpp"
#include <cassert>
#include <cstdint>

struct Plain {uint32_t tag=0,command=0x30000000,words[5]{};};
struct Textured {uint32_t tag=0,command=0x34000000,words[8]{};};
using Slot=epok::FragmentSlot<Plain,Textured>;
static_assert(sizeof(Slot)==sizeof(Textured));
static_assert(alignof(Slot)>=alignof(Plain)&&alignof(Slot)>=alignof(Textured));
int main(){
    Slot frames[2][4];
    for(unsigned frame=0;frame<6;++frame){
        auto& slots=frames[frame&1];
        auto& plain=slots[0].make_plain();plain.words[4]=frame+1;
        auto& textured=slots[1].make_textured();textured.words[7]=frame+2;
        assert(plain.command==0x30000000&&textured.command==0x34000000);
        // A recycled slot may change packet type; adjacent retained slots and
        // the other in-flight frame must keep their bytes.
        auto& recycled=slots[0].make_textured();recycled.words[7]=99;
        assert(slots[1].textured.words[7]==frame+2);
        auto& next=slots[0].make_plain();assert(next.command==0x30000000);
        assert(slots[1].textured.words[7]==frame+2);
        if(frame)assert(frames[(frame&1)^1][1].textured.words[7]==frame+1);
    }
    // Prepared GT4/G4 packets borrow two adjacent slots. Their wider packets
    // must fit the shared stride and preserve the following retained slot.
    Slot pair[3];pair[2].make_textured().words[7]=0x12345678;
    const uint32_t rgb[4]={1,2,3,4},xy[4]={5,6,7,8};
    auto* raw=reinterpret_cast<uint32_t*>(pair);
    epok::BakedPacket<true,true>::write(raw+1,rgb,xy,{},0x34000000);
    assert(pair[2].textured.words[7]==0x12345678);
    epok::BakedPacket<false,true>::write(raw+1,rgb,xy,{},0x30000000);
    assert(pair[2].textured.words[7]==0x12345678);
    for(unsigned textured=0;textured<2;++textured){
        uint32_t packet[10];for(unsigned i=0;i<10;++i)packet[i]=100+i;
        uint32_t bucket=0x123456;
        if(textured)epok::update_retained_triangle<true>(packet,9,8,7,bucket);
        else epok::update_retained_triangle<false>(packet,9,8,7,bucket);
        assert(packet[0]==((textured?9u:6u)<<24|0x123456u));
        assert(packet[2]==9&&packet[textured?5:4]==8&&packet[textured?8:6]==7);
        assert(packet[1]==101&&packet[3]==103&&packet[9]==109);
        assert(bucket==(reinterpret_cast<uintptr_t>(packet)&0xffffffu));
        const uint32_t command=0x36000000,color=0x7f3512;
        if(textured)epok::update_retained_uniform_color<true>(packet,command,color);
        else epok::update_retained_uniform_color<false>(packet,command,color);
        assert(packet[1]==((textured?command:0x30000000u)|color));
        assert(packet[textured?4:3]==color&&packet[textured?7:5]==color);
        assert(packet[2]==9&&packet[textured?5:4]==8&&packet[textured?8:6]==7);
        assert(packet[0]==((textured?9u:6u)<<24|0x123456u)&&packet[9]==109);
    }
}
