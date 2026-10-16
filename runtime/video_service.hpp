#pragma once
// Full-screen video player. A clip is a .STR stream on the CD: BS video frames
// interleaved with XA-ADPCM audio sectors. The CD controller routes the audio
// sectors to the SPU; data sectors arrive here, are assembled into frames,
// expanded to MDEC codes on the CPU and decoded by the MDEC one 16-pixel column
// at a time straight into the frame buffer. Gameplay is suspended meanwhile.
//
// Linked once by main.cpp after video-bank.hh, music.hpp and streaming.hpp.
#include "display.hh"
#include "input.hpp"
#include "music.hpp"
#include "streaming.hpp"
// Outside a game build (reflection, API reference) there are no clips.
#if __has_include("video-bank.hh")
#include "video-bank.hh"
#endif

#ifdef EPOK_HAS_VIDEO
#include "common/hardware/dma.h"
#include "common/hardware/hwregs.h"
#include "common/hardware/spu.h"
#include "fmv_bitstream.hpp"
#include "psyqo/gpu.hh"
#include "psyqo/hardware/gpu.hh"
#include "psyqo/primitives/misc.hh"

namespace epok::fmv {
inline constexpr uint32_t chunk_bytes = 2016;
inline volatile uint32_t &mdec_command = HW_U32(0x1f801820);
inline volatile uint32_t &mdec_control = HW_U32(0x1f801824);

enum class Slot : uint8_t { Free, Filling, Ready, Decoding };
// One frame being assembled from CD sectors. The CD interrupt fills it; the
// main loop decodes it. Each field has a single writer at a time.
struct Frame {
  uint8_t *data = nullptr;
  volatile uint32_t number = 0, bytes = 0;
  volatile uint16_t chunks = 0, received = 0;
  volatile Slot state = Slot::Free;
};
struct Player {
  const VideoClip *clip = nullptr;
  bool skippable = false, display_taken = false, owner = false, cancelled = false;
  volatile bool cd_done = false, cd_failed = false;
  Frame frames[2];
  uint32_t *codes = nullptr;
  uint16_t *slices[2] = {};
  uint32_t saved_mode = 0;
  int16_t x = 0, y = 0;
  uint16_t buffer_y[2] = {};
  uint32_t serial = 0, buffer_serial[2] = {};
  int8_t shown = 0, pending = -1;
  volatile uint32_t sectors = 0;
  uint32_t seen_sectors = 0, quiet_frames = 0, last_shown = 0, after_last = 0;
};
alignas(4) inline uint8_t memory[video_memory_bytes];
inline Player player;
inline psyqo::ISO9660Parser::DirEntry entry;

// Frame chosen for an incoming chunk, or null to skip the sector.
inline Frame *destination(uint32_t number, uint32_t chunk, uint32_t chunks, uint32_t bytes) {
  const auto &clip = *player.clip;
  if (!chunks || chunk >= chunks || chunks * chunk_bytes > clip.frame_bytes || bytes > chunks * chunk_bytes)
    return nullptr;
  Frame *frame = nullptr;
  for (auto &f : player.frames)
    if (f.state == Slot::Filling && f.number == number)
      frame = &f;
  if (!frame) {
    if (chunk)
      return nullptr; // Joined mid-frame.
    // A newer frame abandons any partial one.
    for (auto &f : player.frames)
      if (f.state == Slot::Filling) {
        f.state = Slot::Free;
        ++video_stats.frames_dropped;
      }
    for (auto &f : player.frames)
      if (f.state == Slot::Free) {
        frame = &f;
        break;
      }
    if (!frame) {
      ++video_stats.frames_dropped; // One frame decoding, one queued.
      return nullptr;
    }
    frame->number = number;
    frame->chunks = uint16_t(chunks);
    frame->received = 0;
    frame->bytes = bytes;
    frame->state = Slot::Filling;
  }
  if (chunk != frame->received) {
    frame->state = Slot::Free; // A sector was lost.
    ++video_stats.frames_dropped;
    return nullptr;
  }
  return frame;
}

enum class CdState : uint8_t { IDLE, UNMUTE, MODE, FILTER, LOCATION, READ, STREAMING, PAUSE, PAUSE_ACK };
class StreamAction final : public psyqo::CDRomDevice::Action<CdState> {
  uint32_t lba = 0;
  uint8_t file = 0, channel = 0;
  bool stop_requested = false;
  alignas(4) uint8_t header[32];
  using CDL = psyqo::Hardware::CDRom::CDL;
  void pause() {
    setState(CdState::PAUSE);
    psyqo::Hardware::CDRom::Command.send(CDL::PAUSE);
  }
  static bool read(void *to, uint32_t words) {
    DMA_CTRL[DMA_CDROM].MADR = reinterpret_cast<uintptr_t>(to);
    DMA_CTRL[DMA_CDROM].BCR = 0x10000 | words;
    DMA_CTRL[DMA_CDROM].CHCR = 0x11000000;
    uint32_t timeout = 100000;
    while ((DMA_CTRL[DMA_CDROM].CHCR & 0x01000000) && --timeout) {
    }
    if (!timeout)
      DMA_CTRL[DMA_CDROM].CHCR = 0;
    return timeout != 0;
  }

public:
  StreamAction() : Action("Epok video") {}
  void start(uint32_t sector, uint8_t xa_file, uint8_t xa_channel, eastl::function<void(bool)> &&callback) {
    registerMe(&music_drive);
    setCallback(eastl::move(callback));
    lba = sector;
    file = xa_file;
    channel = xa_channel;
    stop_requested = false;
    setState(CdState::UNMUTE);
    SPU_CTRL = SPU_CTRL | 1;
    psyqo::Hardware::CDRom::Command.send(CDL::UNMUTE);
  }
  void stop() {
    stop_requested = true;
    if (getState() == CdState::STREAMING)
      pause();
  }
  bool acknowledge(const psyqo::CDRomDevice::Response &) override {
    using namespace psyqo::Hardware::CDRom;
    switch (getState()) {
    case CdState::UNMUTE:
      setState(CdState::MODE);
      // Double speed, XA-ADPCM to the SPU, only this clip's audio channel,
      // 2048-byte data sectors to the CPU.
      Command.send(CDL::SETMODE, 0xc8);
      break;
    case CdState::MODE:
      setState(CdState::FILTER);
      Command.send(CDL::SETFILTER, file, channel);
      break;
    case CdState::FILTER: {
      setState(CdState::LOCATION);
      psyqo::MSF msf(lba + 150);
      uint8_t bcd[3];
      msf.toBCD(bcd);
      Command.send(CDL::SETLOC, bcd[0], bcd[1], bcd[2]);
      break;
    }
    case CdState::LOCATION:
      setState(CdState::READ);
      Command.send(CDL::READS);
      break;
    case CdState::READ:
      setState(CdState::STREAMING);
      if (stop_requested)
        pause();
      break;
    case CdState::PAUSE:
      setState(CdState::PAUSE_ACK);
      break;
    default:
      break;
    }
    return false;
  }
  bool dataReady(const psyqo::CDRomDevice::Response &) override {
    if (getState() != CdState::STREAMING)
      return false;
    using namespace psyqo::Hardware::CDRom;
    DataRequest = 0;
    DataRequest = 0x80;
    if (!read(header, 8)) {
      ++video_stats.errors;
      return false;
    }
    player.sectors = player.sectors + 1;
    const char *marker = "EPOKEND1";
    bool end = true;
    for (int i = 0; i < 8; ++i)
      end = end && header[i] == uint8_t(marker[i]);
    if (end) {
      pause();
      return false;
    }
    auto u16 = [this](int at) { return uint32_t(header[at] | header[at + 1] << 8); };
    auto u32 = [&](int at) { return u16(at) | u16(at + 2) << 16; };
    if (u16(0) != 0x0160 || u16(2) != 0x8001 || u16(16) != player.clip->width ||
        u16(18) != player.clip->height) {
      ++video_stats.errors;
      video_stats.error_code = 4;
      return false;
    }
    const uint32_t chunk = u16(4);
    Frame *frame = destination(u32(8), chunk, u16(6), u32(12));
    if (!frame)
      return false;
    if (!read(frame->data + chunk * chunk_bytes, chunk_bytes / 4)) {
      frame->state = Slot::Free;
      ++video_stats.frames_dropped;
      return false;
    }
    frame->received = uint16_t(frame->received + 1);
    if (frame->received == frame->chunks)
      frame->state = Slot::Ready;
    return false;
  }
  bool complete(const psyqo::CDRomDevice::Response &) override {
    setSuccess(getState() == CdState::PAUSE_ACK);
    return true;
  }
  bool end(const psyqo::CDRomDevice::Response &) override {
    if (getState() == CdState::STREAMING)
      pause();
    return false;
  }
};
inline StreamAction action;

inline void error(uint32_t code) {
  ++video_stats.errors;
  video_stats.error_code = code;
}
inline bool wait_dma(uint32_t channel) {
  uint32_t timeout = 2000000;
  while ((DMA_CTRL[channel].CHCR & 0x01000000) && --timeout) {
  }
  if (!timeout)
    DMA_CTRL[channel].CHCR = 0;
  return timeout != 0;
}
inline void mdec_reset() {
  mdec_control = 0x80000000;
  mdec_control = 0x60000000; // Data-in and data-out DMA requests.
}
inline bool mdec_send(uint32_t command, const uint32_t *words, uint32_t count) {
  mdec_command = command;
  DMA_CTRL[DMA_MDECIN].MADR = reinterpret_cast<uintptr_t>(words);
  DMA_CTRL[DMA_MDECIN].BCR = (count / 32) << 16 | 32;
  DMA_CTRL[DMA_MDECIN].CHCR = 0x01000201;
  return wait_dma(DMA_MDECIN);
}
// Quant and IDCT tables, staged through the (still unused) pixel columns.
inline bool mdec_prepare(bool tv_levels) {
  DPCR = DPCR | 0x88; // MDEC in/out DMA channels.
  mdec_reset();
  const auto &quant = tv_levels ? tv_quant : standard_quant;
  auto *staging = reinterpret_cast<uint8_t *>(player.slices[0]);
  for (int i = 0; i < 64; ++i) {
    staging[i] = quant.luma[i];
    staging[64 + i] = quant.chroma[i];
  }
  if (!mdec_send(0x40000001, reinterpret_cast<uint32_t *>(staging), 32))
    return false;
  auto *scale = reinterpret_cast<int16_t *>(player.slices[1]);
  for (int i = 0; i < 64; ++i)
    scale[i] = mdec_idct[i];
  return mdec_send(0x60000000, reinterpret_cast<uint32_t *>(scale), 32);
}

struct VramCopy {
  uint32_t command = 0x80000000;
  psyqo::Vertex source, target, size;
};
static_assert(sizeof(VramCopy) == 16);
inline psyqo::Rect rect(int16_t x, int16_t y, int16_t w, int16_t h) {
  psyqo::Rect r;
  r.pos.x = x;
  r.pos.y = y;
  r.size.w = w;
  r.size.h = h;
  return r;
}
// A 320- (or 256-) pixel 240-line mode: square-ish pixels for 4:3 video. The
// interlace bit is kept so PsyQo's field-paced flips keep working on 480i.
inline uint32_t video_mode(uint32_t game_mode) {
  const uint32_t width = display_width == 256 ? 0 : 1;
  return width | (game_mode & 0x28); // NTSC/PAL and interlace flags.
}
inline uint32_t current_mode() {
  const uint32_t stat = psyqo::Hardware::GPU::Ctrl;
  return ((stat >> 17) & 3) | ((stat >> 19) & 1) << 2 | ((stat >> 20) & 1) << 3 | ((stat >> 21) & 1) << 4 |
         ((stat >> 22) & 1) << 5 | ((stat >> 16) & 1) << 6;
}
inline void take_display(psyqo::GPU &gpu) {
  if (player.display_taken)
    return;
  player.display_taken = true;
  gpu.waitChainIdle();
  const int16_t width = display_width == 256 ? 256 : 320;
  player.buffer_y[0] = 0;
  player.buffer_y[1] = display_interlaced ? 240 : 256;
  for (auto y : player.buffer_y) {
    psyqo::Prim::FastFill fill(psyqo::Color{{.r = 0, .g = 0, .b = 0}});
    fill.rect = rect(0, int16_t(y), width, 240);
    gpu.sendPrimitive(fill);
  }
  player.saved_mode = current_mode();
  psyqo::Hardware::GPU::Ctrl = 0x08000000 | video_mode(player.saved_mode);
  if (display_interlaced)
    psyqo::Hardware::GPU::Ctrl = 0x05000000;
  player.shown = 0;
  player.pending = -1;
  player.serial = player.buffer_serial[0] = player.buffer_serial[1] = 0;
}
inline void release_display(psyqo::GPU &gpu) {
  if (!player.display_taken)
    return;
  player.display_taken = false;
  gpu.waitChainIdle();
  // The game redraws its own buffers; start them black, not on the video.
  for (auto y : player.buffer_y) {
    psyqo::Prim::FastFill fill(psyqo::Color{{.r = 0, .g = 0, .b = 0}});
    fill.rect = rect(0, int16_t(y), display_width == 256 ? 256 : 320, 240);
    gpu.sendPrimitive(fill);
  }
  psyqo::Hardware::GPU::Ctrl = 0x08000000 | player.saved_mode;
  if (display_interlaced)
    psyqo::Hardware::GPU::Ctrl = 0x05000000;
}

// Decodes a complete frame into the buffer at `top`.
inline bool show(Frame &frame, psyqo::GPU &gpu, uint16_t top) {
  const auto &clip = *player.clip;
  const uint32_t number = frame.number;
  frame.state = Slot::Decoding;
  const uint16_t begin = COUNTERS[1].value; // Free-running scanline count.
  const auto decoded = decode_frame(frame.data, frame.bytes, clip.width, clip.height, player.codes,
                                    clip.mdec_words, clip.flags & video_tv_levels);
  frame.state = Slot::Free; // The CD may refill it while the MDEC works.
  video_stats.bitstream_lines = uint16_t(COUNTERS[1].value - begin);
  if (decoded.status != BsStatus::OK) {
    video_stats.failed_frame = frame.number;
    video_stats.failed_status = uint32_t(decoded.status);
    error(5);
    ++video_stats.frames_dropped;
    return false;
  }
  mdec_command = player.codes[0];
  DMA_CTRL[DMA_MDECIN].MADR = reinterpret_cast<uintptr_t>(player.codes + 1);
  DMA_CTRL[DMA_MDECIN].BCR = ((decoded.words - 1) / 32) << 16 | 32;
  DMA_CTRL[DMA_MDECIN].CHCR = 0x01000201;
  // One column of macroblocks: 16 x height pixels, 8 * height words.
  const uint32_t blocks = clip.height / 4;
  for (uint32_t column = 0; column < clip.width / 16u; ++column) {
    uint16_t *slice = player.slices[column & 1];
    DMA_CTRL[DMA_MDECOUT].MADR = reinterpret_cast<uintptr_t>(slice);
    DMA_CTRL[DMA_MDECOUT].BCR = blocks << 16 | 32;
    DMA_CTRL[DMA_MDECOUT].CHCR = 0x01000200;
    if (!wait_dma(DMA_MDECOUT)) {
      DMA_CTRL[DMA_MDECIN].CHCR = 0;
      mdec_reset();
      error(6);
      return false;
    }
    gpu.uploadToVRAM(slice, rect(int16_t(player.x + 16 * column), int16_t(top + player.y), 16,
                                 int16_t(clip.height)));
  }
  wait_dma(DMA_MDECIN);
  video_stats.decode_lines = uint16_t(COUNTERS[1].value - begin);
  player.last_shown = number;
  ++video_stats.frames_shown;
  return true;
}
inline Frame *ready() {
  Frame *best = nullptr;
  for (auto &f : player.frames)
    if (f.state == Slot::Ready && (!best || f.number < best->number))
      best = &f;
  return best;
}

inline void finish(psyqo::GPU &gpu) {
  release_display(gpu);
  if (player.owner) {
    player.owner = false;
    SPU_VOL_CD_LEFT = 0;
    SPU_VOL_CD_RIGHT = 0;
  }
  // BGM interrupted by the video resumes; geometry reads may start again.
  music_data_owner = false;
  for (auto &f : player.frames)
    f.state = Slot::Free;
  input.discard_edges();
  video_stats.state = VideoState::Idle;
}

inline bool play(size_t index, bool skippable) {
  if (index >= video_clip_count || !video_clips[index].file || video_stats.state != VideoState::Idle)
    return false;
  const auto &clip = video_clips[index];
  player.clip = &clip;
  player.skippable = skippable;
  player.frames[0].data = memory;
  player.frames[1].data = memory + clip.frame_bytes;
  player.codes = reinterpret_cast<uint32_t *>(memory + 2 * clip.frame_bytes);
  auto *columns = memory + 2 * clip.frame_bytes + 4 * clip.mdec_words;
  player.slices[0] = reinterpret_cast<uint16_t *>(columns);
  player.slices[1] = reinterpret_cast<uint16_t *>(columns + 16 * 240 * 2);
  const int16_t width = display_width == 256 ? 256 : 320;
  player.x = int16_t((width - int16_t(clip.width)) / 2);
  player.y = int16_t((240 - int16_t(clip.height)) / 2);
  player.cd_done = false;
  player.cd_failed = false;
  player.cancelled = false;
  player.sectors = player.seen_sectors = player.quiet_frames = 0;
  player.last_shown = player.after_last = 0;
  for (auto &f : player.frames)
    f.state = Slot::Free;
  video_stats.state = VideoState::Waiting;
  video_stats.clip = int32_t(index);
  ++video_stats.plays;
  // Interrupts BGM now; the XA interruption path resumes it afterwards.
  music_data_owner = true;
  return true;
}
inline void stop() {
  switch (video_stats.state) {
  case VideoState::Waiting:
    video_stats.state = VideoState::Finished;
    break;
  case VideoState::Lookup:
    player.cancelled = true; // The lookup callback finishes instead of streaming.
    break;
  case VideoState::Starting:
  case VideoState::Playing:
    action.stop();
    video_stats.state = VideoState::Stopping;
    break;
  default:
    break;
  }
}
inline bool playing() { return video_stats.state != VideoState::Idle; }

inline void start_stream() {
  video_stats.state = VideoState::Lookup;
  music_files.getDirentry(player.clip->file, &entry, [](bool ok) {
    if (video_stats.state != VideoState::Lookup)
      return;
    if (player.cancelled) {
      video_stats.state = VideoState::Finished;
      return;
    }
    if (!ok || entry.type != psyqo::ISO9660Parser::DirEntry::FILE) {
      error(2);
      video_stats.state = VideoState::Finished;
      return;
    }
    if (!mdec_prepare(player.clip->flags & video_tv_levels)) {
      error(6);
      video_stats.state = VideoState::Finished;
      return;
    }
    video_stats.state = VideoState::Starting;
    // Full volume, as BGM at volume 1.0.
    SPU_VOL_CD_LEFT = 0x7ff8;
    SPU_VOL_CD_RIGHT = 0x7ff8;
    action.start(entry.LBA, player.clip->xa_file, player.clip->xa_channel, [](bool ok) {
      player.cd_failed = !ok;
      player.cd_done = true;
    });
  });
}

// Runs the player for one display frame. True while it owns the screen; the
// game's simulation and rendering are skipped for that frame.
inline bool frame(psyqo::GPU &gpu, const psyqo::AdvancedPad &pad) {
  if (video_stats.state == VideoState::Idle)
    return false;
  take_display(gpu);
  switch (video_stats.state) {
  case VideoState::Waiting:
    // Finishes the drive boot and pauses BGM; geometry reads drain.
    music_tick();
    if (music_boot_failed) {
      error(1);
      video_stats.state = VideoState::Finished;
    } else if (music_ready && !music_active && !music_lookup && music_drive.isIdle() && !stream_read_pending &&
               !stream_lookup_pending) {
      player.owner = true;
      start_stream();
    }
    break;
  case VideoState::Starting:
  case VideoState::Playing:
  case VideoState::Stopping: {
    if (display_interlaced && player.pending >= 0) {
      // Show last frame's decode now, just after the vertical blank.
      player.shown = player.pending;
      player.pending = -1;
      psyqo::Hardware::GPU::Ctrl = 0x05000000 | uint32_t(player.buffer_y[player.shown]) << 10;
    }
    const int target = display_interlaced ? player.shown ^ 1 : gpu.getParity();
    if (Frame *next = ready()) {
      if (show(*next, gpu, player.buffer_y[target])) {
        video_stats.state = video_stats.state == VideoState::Starting ? VideoState::Playing : video_stats.state;
        player.buffer_serial[target] = ++player.serial;
        if (display_interlaced)
          player.pending = int8_t(target);
      }
    } else if (!display_interlaced && player.serial && player.buffer_serial[target] != player.serial) {
      // PsyQo flips every frame: give the other buffer the newest picture.
      const int other = target ^ 1;
      VramCopy copy;
      copy.source.x = 0;
      copy.source.y = int16_t(player.buffer_y[other]);
      copy.target.x = 0;
      copy.target.y = int16_t(player.buffer_y[target]);
      copy.size.w = display_width == 256 ? 256 : 320;
      copy.size.h = 240;
      gpu.sendPrimitive(copy);
      player.buffer_serial[target] = player.serial;
    }
    input.poll_multitap(pad);
    if (player.skippable && video_stats.state != VideoState::Stopping &&
        (input.frame_pressed(Button::Start) || input.frame_pressed(Button::Cross))) {
      action.stop();
      video_stats.state = VideoState::Stopping;
    }
    const uint32_t sectors = player.sectors;
    player.quiet_frames = sectors == player.seen_sectors ? player.quiet_frames + 1 : 0;
    player.seen_sectors = sectors;
    video_stats.sectors = sectors;
    if (player.quiet_frames > 180 && video_stats.state != VideoState::Stopping) {
      error(7);
      action.stop();
      video_stats.state = VideoState::Stopping;
    }
    // The end marker normally stops the drive right after the last frame; if a
    // read error lost it, stop half a second after that frame instead.
    if (player.last_shown == player.clip->frames && ++player.after_last > 30 &&
        video_stats.state != VideoState::Stopping) {
      action.stop();
      video_stats.state = VideoState::Stopping;
    }
    if (player.cd_done && !ready()) {
      if (player.cd_failed)
        error(3);
      video_stats.state = VideoState::Finished;
    }
    break;
  }
  case VideoState::Lookup:
    break;
  case VideoState::Finished:
    finish(gpu);
    break;
  default:
    break;
  }
  return true;
}
} // namespace epok::fmv
#endif

// Definitions of the blueprint_api.hpp declarations.
namespace epok::bp::api {
bool play_video(uint64_t video, bool skippable) {
#ifdef EPOK_HAS_VIDEO
  const int index = video ? bp::video_index(video) : -1;
  return index >= 0 && fmv::play(size_t(index), skippable);
#else
  (void)video;
  (void)skippable;
  return false;
#endif
}
void stop_video() {
#ifdef EPOK_HAS_VIDEO
  fmv::stop();
#endif
}
bool video_playing() {
#ifdef EPOK_HAS_VIDEO
  return fmv::playing();
#else
  return false;
#endif
}
} // namespace epok::bp::api
