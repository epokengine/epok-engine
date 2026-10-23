#pragma once
#include "epok.hpp"
#if defined(__mips__)
#include "common/hardware/counters.h"
// One timer pair per gameplay subsystem, independent of per-polygon profiling.
struct DungeonProfile {uint32_t frame=0,scanlines[12]{};};
extern DungeonProfile dungeon_profile;
struct DungeonMeasure {
    unsigned field;uint16_t start;
    explicit DungeonMeasure(unsigned f):field(f),start(COUNTERS[1].value){
        if(dungeon_profile.frame!=epok::performance_stats.frame+1){dungeon_profile={};dungeon_profile.frame=epok::performance_stats.frame+1;}
    }
    ~DungeonMeasure(){dungeon_profile.scanlines[field]+=uint16_t(COUNTERS[1].value-start);}
};
#else
struct DungeonMeasure {explicit DungeonMeasure(unsigned){}};
#endif
