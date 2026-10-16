#pragma once
#include <stddef.h>
#include <stdint.h>

// Types of the generated video bank (video-bank.hh). The player itself is
// runtime/video_service.hpp, linked once by main.cpp.
namespace epok {
// One cooked .STR stream on the disc, as the editor validated it.
struct VideoClip {
  const char *file;    // ISO9660 name, e.g. "V0000000.STR;1"
  uint16_t width, height;
  uint32_t frames;
  uint32_t frame_bytes; // largest frame's sectors * 2016
  uint32_t mdec_words;  // decoder buffer: largest frame plus capacity_margin
  uint8_t flags, xa_file, xa_channel;
};
inline constexpr uint8_t video_tv_levels = 1, video_has_audio = 2;

enum class VideoState : uint8_t { Idle, Waiting, Lookup, Starting, Playing, Stopping, Finished };
// Errors: 1 no CD drive, 2 lookup, 3 CD command, 4 bad stream, 5 decode,
// 6 MDEC timeout, 7 no data for three seconds.
struct VideoStats {
  VideoState state = VideoState::Idle;
  int32_t clip = -1;
  uint32_t plays = 0, frames_shown = 0, frames_dropped = 0, sectors = 0, errors = 0, error_code = 0;
  // Scanlines (63.6 us each) the last frame took to decode: in total
  // (bitstream, MDEC and upload) and in the CPU bitstream pass alone.
  uint32_t decode_lines = 0, bitstream_lines = 0;
  // The last frame that failed to decode and its BsStatus.
  uint32_t failed_frame = 0, failed_status = 0;
};
inline VideoStats video_stats;
} // namespace epok
