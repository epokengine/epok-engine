#pragma once
#include <stdint.h>

// The saved economy is independent of a dungeon run. Cumulative receipts make
// an interrupted PocketStation acknowledgement safe to retry on this save.
namespace dungeon_inventory {
constexpr uint32_t magic=0x41535431,limit=1000000;
inline constexpr const char file_name[]="EPOK-DUNGEON";
inline constexpr const char file_title[]="Dungeon - Asteroid Supply";
struct Receipt {uint32_t serial=0,total=0;};
struct Save {
    uint32_t signature=magic,version=1,ore=0,credits=0,medkits=0,tanks=0,buster=0;
    Receipt receipts[8]{};
};
static_assert(sizeof(Save)==92,"Keep the original seven words and eight receipt pairs");
}
