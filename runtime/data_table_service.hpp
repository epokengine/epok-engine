#pragma once
#include "data_table.hpp"
// Included once by the runtime entry point, after the generated scene tables.
// Native scripts may call these APIs from separately compiled translation units.
namespace epok::assets {
uint64_t find(uint64_t table,const char* key,Kind kind) {
    return bp::data_table_find(table,key,uint32_t(kind));
}
uint64_t cast(uint64_t asset,Kind kind) {
    return bp::data_table_cast(asset,uint32_t(kind));
}
int audio_clip(uint64_t asset) { return asset ? bp::asset_index(asset,2) : -1; }
}
