#pragma once
#include <stdint.h>

/* EPN4: the EPN3 header is followed by four fixed motor records
 * (identity, revision, large, small, remaining_us), then actors/HUD/audio/text.
 * Requests start with this magic and elapsed_us, then four pad records
 * (buttons, analog, packed_axes, flags, identity), then audio completions.
 * flags: bit 0 connected, bit 1 rumble capable, bit 2 host output enabled. */
#define EPOK_NATIVE_PLAY_MAGIC 0x344e5045u /* "EPN4" */
#define EPOK_NATIVE_PLAY_RUMBLE_LIMIT_US 60000000u
#define EPOK_NATIVE_PLAY_ACTOR_WORDS 16u
#define EPOK_NATIVE_PLAY_AUDIO_WORDS 6u
#define EPOK_NATIVE_PLAY_AUDIO_LIMIT 256u
