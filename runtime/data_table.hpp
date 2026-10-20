#pragma once
#include <stdint.h>

namespace epok::bp {
int asset_index(uint64_t asset,uint32_t kind);
uint64_t data_table_find(uint64_t table,const char* key,uint32_t kind=0);
uint64_t data_table_cast(uint64_t asset,uint32_t kind);
}
namespace epok::assets {
enum class Kind : uint32_t { Any=0,Texture=1,AudioClip=2,Video=3,PocketStationApp=4,DataTable=5,MusicSequence=6 };
// Persistent authoring UUIDs are cooked to 64-bit handles. Missing/wrong-kind
// references return zero; they never alias asset index zero.
uint64_t find(uint64_t table,const char* key,Kind kind=Kind::Any);
uint64_t cast(uint64_t asset,Kind kind);
int audio_clip(uint64_t asset);
}
