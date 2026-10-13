#pragma once
#include <stdint.h>

/* EPN2: the header ends with the active visibility track sample (or ~0u). */
#define EPOK_NATIVE_PLAY_MAGIC 0x324e5045u /* "EPN2" */
#define EPOK_NATIVE_PLAY_ACTOR_WORDS 16u
#define EPOK_NATIVE_PLAY_AUDIO_WORDS 6u
#define EPOK_NATIVE_PLAY_AUDIO_LIMIT 256u
