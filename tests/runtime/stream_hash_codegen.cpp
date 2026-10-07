#include "streaming_pool.hpp"
extern "C" __attribute__((noinline)) uint32_t hash_update(uint32_t seed,const uint8_t* data,size_t bytes){
    return epok::stream_page_hash_update(seed,data,bytes);
}
