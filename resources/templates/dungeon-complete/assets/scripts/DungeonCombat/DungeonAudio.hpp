#pragma once
#include <stdint.h>

enum class DungeonSound : uint8_t {
    PlayerShot, WorldHit, RobotHit, PlayerDamage, RobotDestroyed, Pickup,
    EnemyShot, BossCharge, BossDestroyed, LockOn, UiOpen, UiMove, UiClose,
    UiConfirm, UiSuccess, UiError, Heal, Count
};

namespace dungeon_audio {
struct Cue {const char* key;uint8_t channel;uint16_t volume;};
// Seven resident sources retain bounded voice use. Combat feedback has its own
// channels; interface cues share the terminal source while combat is paused.
inline constexpr Cue cues[]={
    {"PlayerShot",0,1966}, {"WorldHit",1,1311}, {"RobotHit",2,2376},
    {"PlayerDamage",3,2949}, {"RobotDestroyed",5,2785}, {"Pickup",6,2662},
    {"EnemyShot",4,1434}, {"BossCharge",4,2458}, {"BossDestroyed",5,3195},
    {"LockOn",6,2130}, {"UiOpen",6,1843}, {"UiMove",6,1229},
    {"UiClose",6,1638}, {"UiConfirm",6,1638}, {"UiSuccess",6,2376},
    {"UiError",6,2048}, {"Heal",6,2662}
};
static_assert(sizeof(cues)/sizeof(*cues)==unsigned(DungeonSound::Count));
}

// Exposed to the emulator debugger to verify actual event routing and missing
// assets without adding per-frame audio work.
struct DungeonAudioTelemetry {
    uint32_t events[unsigned(DungeonSound::Count)]{};
    uint32_t missing=0;
};
extern DungeonAudioTelemetry dungeon_audio_telemetry;
