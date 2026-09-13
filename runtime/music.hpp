#pragma once
#include "audio.hpp"
#include "common/hardware/cdrom.h"
#include "psyqo/cdrom-device.hh"
#include "psyqo/hardware/cdrom.hh"
#include "psyqo/iso9660-parser.hh"
#include "psyqo/kernel.hh"

namespace epok {
// One CD controller owns one XA stream. These counters also expose failures to
// game scripts/debuggers.
inline MusicStats music_stats;
inline psyqo::CDRomDevice music_drive;
inline psyqo::ISO9660Parser music_files(&music_drive);
inline psyqo::ISO9660Parser::DirEntry music_entry;
inline AudioSource *music_requested = nullptr;
inline AudioSource *music_active = nullptr;
inline bool music_prepared = false, music_booting = false, music_ready = false,
            music_lookup = false;
// Geometry reads and XA share the physical controller and ISO parser. The
// owner flag prevents XA lookup/start until the outstanding data read finishes.
inline bool music_data_owner = false, music_boot_failed = false;
inline uint32_t music_request_serial = 0, music_active_serial = 0;
enum class XAState : uint8_t {
  IDLE,
  UNMUTE,
  MODE,
  FILTER,
  LOCATION,
  READ,
  PLAYING,
  PAUSE,
  PAUSE_ACK
};
class XAAction final : public psyqo::CDRomDevice::Action<XAState> {
  uint32_t lba = 0;
  bool stop_requested = false;
  alignas(4) uint8_t marker[2048];
  using CDL = psyqo::Hardware::CDRom::CDL;
  void pause() {
    setState(XAState::PAUSE);
    music_stats.state = 5;
    psyqo::Hardware::CDRom::Command.send(CDL::PAUSE);
  }

public:
  bool natural_end = false;
  XAAction() : Action("Epok XA") {}
  void start(uint32_t sector, eastl::function<void(bool)> &&callback) {
    registerMe(&music_drive);
    setCallback(eastl::move(callback));
    lba = sector;
    stop_requested = false;
    natural_end = false;
    setState(XAState::UNMUTE);
    music_stats.state = 3;
    SPU_CTRL = SPU_CTRL | 1;
    psyqo::Hardware::CDRom::Command.send(CDL::UNMUTE);
  }
  void stop() {
    stop_requested = true;
    if (getState() == XAState::PLAYING)
      pause();
  }
  bool acknowledge(const psyqo::CDRomDevice::Response &) override {
    using namespace psyqo::Hardware::CDRom;
    switch (getState()) {
    case XAState::UNMUTE:
      setState(XAState::MODE);
      Command.send(CDL::SETMODE, 0xc8);
      break;
    case XAState::MODE:
      setState(XAState::FILTER);
      Command.send(CDL::SETFILTER, 0, 0);
      break;
    case XAState::FILTER: {
      setState(XAState::LOCATION);
      psyqo::MSF msf(lba + 150);
      uint8_t bcd[3];
      msf.toBCD(bcd);
      Command.send(CDL::SETLOC, bcd[0], bcd[1], bcd[2]);
      break;
    }
    case XAState::LOCATION:
      setState(XAState::READ);
      Command.send(CDL::READS);
      break;
    case XAState::READ:
      setState(XAState::PLAYING);
      music_stats.state = 4;
      ++music_stats.starts;
      if (stop_requested)
        pause();
      break;
    case XAState::PAUSE:
      setState(XAState::PAUSE_ACK);
      break;
    default:
      break;
    }
    return false;
  }
  bool dataReady(const psyqo::CDRomDevice::Response &) override {
    if (getState() != XAState::PLAYING)
      return false;
    // Audio and dummy sectors are consumed/filtered by the CD hardware. Only
    // the final ordinary data sector reaches the CPU; no PCM passes through
    // main RAM.
    using namespace psyqo::Hardware::CDRom;
    DataRequest = 0;
    DataRequest = 0x80;
    DMA_CTRL[DMA_CDROM].MADR = (uint32_t)marker;
    DMA_CTRL[DMA_CDROM].BCR = 0x10200;
    DMA_CTRL[DMA_CDROM].CHCR = 0x11000000;
    uint32_t timeout = 100000;
    while ((DMA_CTRL[DMA_CDROM].CHCR & 0x01000000) && --timeout) {
    }
    const char *expected = "EPOKEND1";
    natural_end = timeout != 0;
    for (int i = 0; i < 8; ++i)
      if (marker[i] != expected[i])
        natural_end = false;
    if (!natural_end) {
      ++music_stats.errors;
      music_stats.error_code = 5;
      if (!timeout)
        DMA_CTRL[DMA_CDROM].CHCR = 0;
    }
    pause();
    return false;
  }
  bool complete(const psyqo::CDRomDevice::Response &) override {
    setSuccess(getState() == XAState::PAUSE_ACK);
    return true;
  }
  bool end(const psyqo::CDRomDevice::Response &) override {
    if (getState() == XAState::PLAYING)
      pause();
    return false;
  }
};
inline XAAction music_action;
inline void music_prepare(bool require_data = false) {
  if (music_prepared)
    return;
  if (require_data) {
    music_drive.prepare();
    music_prepared = true;
    music_stats.state = 1;
    return;
  }
  for (size_t i = 0; i < audio_count; ++i)
    if (audio_bank[i].music) {
      music_drive.prepare();
      music_prepared = true;
      music_stats.state = 1;
      return;
    }
}
void music_play(AudioSource *source) {
  if (!music_prepared)
    return;
  if (music_requested && music_requested != source &&
      music_requested->priority > source->priority)
    return;
  music_requested = source;
  ++music_request_serial;
  if (music_active && !music_lookup)
    music_action.stop();
}
void music_stop(AudioSource *source) {
  if (music_requested == source)
    music_requested = nullptr;
  if (music_active == source && !music_lookup)
    music_action.stop();
}
bool music_is_playing(const AudioSource *source) {
  return music_active == source && music_stats.state == 4;
}
inline void music_error(uint32_t code) {
  ++music_stats.errors;
  music_stats.error_code = code;
  music_stats.state = 6;
  music_requested = nullptr;
  music_active = nullptr;
  SPU_VOL_CD_LEFT = 0;
  SPU_VOL_CD_RIGHT = 0;
}
inline void music_tick() {
  if (!music_prepared)
    return;
  if (!music_booting) {
    music_booting = true;
    music_drive.reset([](bool ok) {
      if (!ok) {
        music_boot_failed = true;
        music_error(1);
        return;
      }
      music_files.initialize([](bool valid) {
        music_ready = valid;
        music_boot_failed = !valid;
        if (!valid)
          music_error(2);
        else
          music_stats.state = 2;
      });
    });
    return;
  }
  if (!music_ready)
    return;
  if (music_data_owner) {
    SPU_VOL_CD_LEFT = 0;
    SPU_VOL_CD_RIGHT = 0;
    if (music_active && !music_lookup)
      music_action.stop();
    return;
  }
  if (music_active) {
    if (music_requested != music_active || !music_active->enabled ||
        music_active->clip != music_stats.clip) {
      if (music_requested == music_active)
        music_requested = nullptr;
      if (!music_lookup)
        music_action.stop();
    }
    int volume = music_active->volume.raw();
    if (volume < 0)
      volume = 0;
    if (volume > 4095)
      volume = 4095;
    volume = int(uint32_t(volume) * transition_audio_gain() / 4096);
    SPU_VOL_CD_LEFT = volume * 8;
    SPU_VOL_CD_RIGHT = volume * 8;
    return;
  }
  SPU_VOL_CD_LEFT = 0;
  SPU_VOL_CD_RIGHT = 0;
  if (!music_requested || !music_drive.isIdle())
    return;
  auto *source = music_requested;
  if (!source->enabled || source->clip < 0 ||
      size_t(source->clip) >= audio_count || !audio_bank[source->clip].music) {
    music_requested = nullptr;
    return;
  }
  music_active = source;
  music_active_serial = music_request_serial;
  music_stats.clip = source->clip;
  music_lookup = true;
  music_stats.state = 3;
  music_files.getDirentry(
      audio_bank[source->clip].music, &music_entry, [](bool ok) {
        music_lookup = false;
        if (!ok || music_entry.type != psyqo::ISO9660Parser::DirEntry::FILE) {
          music_error(3);
          return;
        }
        if (music_requested != music_active || music_data_owner) {
          music_active = nullptr;
          music_stats.state = 2;
          return;
        }
        music_action.start(music_entry.LBA, [](bool success) {
          if (!success) {
            music_error(4);
            return;
          }
          if (music_action.natural_end)
            ++music_stats.ends;
          // A data read interrupts XA deliberately. Preserve the latest play
          // request, but respect stop/replacement requests made meanwhile.
          // XA resumes from the track beginning: its exact sector is not
          // available from this custom READS action.
          if (!(music_data_owner && !music_action.natural_end) &&
              music_requested == music_active &&
              music_request_serial == music_active_serial) {
            if (music_action.natural_end &&
                audio_bank[music_stats.clip].looping)
              ++music_stats.loops;
            else
              music_requested = nullptr;
          }
          music_active = nullptr;
          music_stats.state = 2;
        });
      });
}
} // namespace epok
