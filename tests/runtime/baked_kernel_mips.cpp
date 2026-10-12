// Freestanding cross-compile probe: the same headers used by the host tests.
// This does not stand in for a game/emulator integration or cycle benchmark.
#include "../../runtime/draw_order.hpp"
#include "../../runtime/baked_packet.hpp"
#include "../../runtime/ordered_draw.hpp"
extern "C" bool epok_decode_order(const uint8_t* bytes,unsigned count,const uint32_t* previous,
    unsigned previous_count,uint32_t* output,unsigned capacity,unsigned expected) {
    return epok::decode_draw_order(bytes,count,previous,previous_count,output,capacity,expected);
}
extern "C" void epok_write_gt4(uint32_t* output,const uint32_t* colors,const uint32_t* screen,
    epok::BakedTextureWords uv,uint32_t command) {
    epok::BakedPacket<true,true>::write(output,colors,screen,uv,command);
}
extern "C" bool epok_prepare_ordered(epok::OrderedDrawCursor<938>* cursor,
    const epok::OrderedDrawData* data,uint16_t sample,uint32_t resident,const uint8_t* page) {
    return cursor->prepare(data,sample,[&](uint32_t id){return id==resident?page:nullptr;});
}
extern "C" uint32_t epok_ordered_storage_bytes(){return sizeof(epok::OrderedDrawCursor<938>);}
