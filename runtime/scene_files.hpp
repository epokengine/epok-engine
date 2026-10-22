#pragma once
#include "streaming.hpp"

namespace epok {
// External scenes keep their data out of the executable. The cooker links the
// data of every external bank into the same RAM region (a linker overlay) and
// saves each image as SCENEnn.BIN. Before a bank's load function runs, its file
// is read into that region from the CD or through PCDrv and must match the size
// and FNV-1a hash the editor wrote into scene_file_records after linking. Only
// then is the bank published; a missing, short or corrupt file never runs.
// Builds without external scenes link no region and compile these paths out.
extern "C" uint8_t __epok_scene_region_start[] __attribute__((weak));
extern "C" uint8_t __epok_scene_region_end[] __attribute__((weak));

enum class SceneFileState : uint8_t { Idle, Waiting, Reading, Ready, Failed };
enum class SceneFileError : uint8_t { None, Record, Drive, Missing, Size, Read, Checksum };
// Loading time and bytes are measured here, apart from gameplay frame timing.
struct SceneFileStats {
  uint32_t loads = 0, failures = 0, bytes = 0, last_us = 0, max_us = 0;
  uint32_t loaded = 0, total = 0; // Verified bytes of the file being loaded.
  int8_t resident = -1, loading = -1; // File indices; -1 means none.
  SceneFileState state = SceneFileState::Idle;
  SceneFileError error = SceneFileError::None;
};
inline SceneFileStats scene_file_stats;
// Bank whose file is being loaded before its load function may run, or -1.
inline int scene_file_bank = -1;

struct SceneFileLoad {
  uint32_t hash = 2166136261u, started = 0;
  int host = -1;
  bool lookup = false, lookup_done = false, found = false;
  bool read_started = false, read_complete = false, read_ok = false;
  // A CD callback still owns the region (and its directory entry).
  bool pending() const { return (lookup && !lookup_done) || (read_started && !read_complete); }
};
inline SceneFileLoad scene_file_load;
#if !EPOK_HOST_DATA
inline psyqo::ISO9660Parser::DirEntry scene_file_entry;
inline StreamReadAction scene_read_action;
#endif
inline constexpr uint32_t scene_file_timeout_us = 30000000;
// PCDrv reads block the frame; smaller reads keep the loading screen updating.
inline constexpr uint32_t scene_file_host_chunk = 16384;

inline uint8_t* scene_region() { return __epok_scene_region_start; }
inline uint32_t scene_region_bytes() {
  return uint32_t(__epok_scene_region_end - __epok_scene_region_start);
}
inline bool scene_bank_external(size_t bank) {
  return bank < scene_bank_count && scene_banks[bank].file >= 0;
}
inline bool scene_bank_ready(size_t bank) {
  if (!scene_bank_external(bank)) return true;
  return scene_file_stats.state == SceneFileState::Ready &&
         scene_file_stats.resident == scene_banks[bank].file;
}
inline bool scene_file_waiting() { return scene_file_bank >= 0; }
// Fraction of the current file verified so far, 0..4096 (Q12). Real bytes,
// not a timer; a bank already in the region reports 4096.
inline uint32_t scene_file_progress() {
  if (scene_file_stats.state == SceneFileState::Ready) return 4096;
  if (!scene_file_stats.total) return 0;
  return uint32_t(uint64_t(scene_file_stats.loaded) * 4096 / scene_file_stats.total);
}

inline void scene_files_prepare() {
#if !EPOK_HOST_DATA
  if constexpr (scene_file_count > 0) music_prepare(true);
#endif
}

inline void scene_file_release_drive() {
#if !EPOK_HOST_DATA
  music_scene_reading = false;
  if (!stream_read_pending && !stream_lookup_pending) music_data_owner = false;
#endif
}

inline void scene_file_fail(SceneFileError error) {
  auto& s = scene_file_stats;
  s.state = SceneFileState::Failed;
  s.error = error;
  ++s.failures;
#if EPOK_HOST_DATA
  if (scene_file_load.host >= 0) { PCclose(scene_file_load.host); scene_file_load.host = -1; }
#endif
  // A pending CD callback still owns the region and the drive.
  if (!scene_file_load.pending()) scene_file_release_drive();
}

// Start loading the file of an external bank. The region is no longer valid for
// any other bank from this point on, even if the load later fails.
inline void scene_file_begin(size_t bank, uint32_t now) {
  if constexpr (scene_file_count == 0) { (void)bank; (void)now; return; }
  if (scene_bank_ready(bank)) return;
  auto& s = scene_file_stats;
  const int file = scene_banks[bank].file;
  s.loading = int8_t(file);
  s.resident = -1;
  s.loaded = 0;
  s.total = scene_file_records[file].bytes;
  s.error = SceneFileError::None;
  s.state = SceneFileState::Waiting;
  scene_file_load = SceneFileLoad{};
  scene_file_load.started = now;
  // An unpatched or oversized record means this executable was not finished
  // by the editor's link step. Never read past the region.
  if (s.total == 0 || s.total % 2048 || s.total > scene_region_bytes()) scene_file_fail(SceneFileError::Record);
}

inline void scene_file_publish(uint32_t now) {
  auto& s = scene_file_stats;
  if (scene_file_load.hash != scene_file_records[s.loading].hash) {
    scene_file_fail(SceneFileError::Checksum);
    return;
  }
  s.state = SceneFileState::Ready;
  s.resident = s.loading;
  s.loading = -1;
  ++s.loads;
  s.bytes += s.total;
  s.last_us = now - scene_file_load.started;
  if (s.last_us > s.max_us) s.max_us = s.last_us;
  scene_file_release_drive();
}

// Advance the current load. Call once per frame while scene_file_waiting().
inline void scene_file_service(uint32_t now) {
  if constexpr (scene_file_count == 0) { (void)now; return; }
  auto& s = scene_file_stats;
  auto& load = scene_file_load;
  if (s.state == SceneFileState::Failed) {
#if !EPOK_HOST_DATA
    if (music_scene_reading && !load.pending() && StreamReadAction::dma_idle()) scene_file_release_drive();
#endif
    return;
  }
  if (s.state != SceneFileState::Waiting && s.state != SceneFileState::Reading) return;
  uint8_t* region = scene_region();
#if EPOK_HOST_DATA
  if (s.state == SceneFileState::Waiting) {
    static bool initialized = false;
    if (!initialized) { PCinit(); initialized = true; }
    load.host = PCopen(scene_file_names[s.loading], 0, 0);
    if (load.host < 0) { scene_file_fail(SceneFileError::Missing); return; }
    if (PClseek(load.host, 0, PCDRV_SEEK_END) != int(s.total) ||
        PClseek(load.host, 0, PCDRV_SEEK_SET) != 0) {
      scene_file_fail(SceneFileError::Size);
      return;
    }
    s.state = SceneFileState::Reading;
  }
  const uint32_t remaining = s.total - s.loaded;
  const uint32_t chunk = remaining < scene_file_host_chunk ? remaining : scene_file_host_chunk;
  if (PCread(load.host, region + s.loaded, int(chunk)) != int(chunk)) {
    scene_file_fail(SceneFileError::Read);
    return;
  }
  load.hash = streaming_hash_slice(load.hash, region + s.loaded, chunk);
  s.loaded += chunk;
  if (s.loaded == s.total) {
    PCclose(load.host);
    load.host = -1;
    scene_file_publish(now);
  }
#else
  if (now - load.started > scene_file_timeout_us) {
    // Report the stall, but keep the region owned until any callback returns.
    scene_file_fail(s.state == SceneFileState::Waiting && !load.lookup ? SceneFileError::Drive
                                                                       : SceneFileError::Read);
    return;
  }
  if (s.state == SceneFileState::Waiting) {
    if (music_boot_failed) { scene_file_fail(SceneFileError::Drive); return; }
    if (!load.lookup) {
      // XA and geometry share the drive: wait until neither has work in flight.
      if (!music_ready || music_active || music_lookup || !music_drive.isIdle() ||
          stream_read_pending || stream_lookup_pending || !StreamReadAction::dma_idle()) return;
      music_data_owner = music_scene_reading = true;
      load.lookup = true;
      music_files.getDirentry(scene_file_paths[s.loading], &scene_file_entry, [](bool ok) {
        scene_file_load.found = ok && scene_file_entry.type == psyqo::ISO9660Parser::DirEntry::FILE;
        scene_file_load.lookup_done = true;
      });
      return;
    }
    if (!load.lookup_done) return;
    if (!load.found) { scene_file_fail(SceneFileError::Missing); return; }
    // Exact length rejects a truncated file or one from another build.
    if (scene_file_entry.size != s.total) { scene_file_fail(SceneFileError::Size); return; }
    if (!music_drive.isIdle()) return;
    load.read_started = true;
    s.state = SceneFileState::Reading;
    scene_read_action.start(&music_drive, scene_file_entry.LBA, s.total / 2048, region, [](bool ok) {
      // Hash and publish on the main loop only, after the final DMA drained.
      scene_file_load.read_ok = ok;
      scene_file_load.read_complete = true;
    });
    return;
  }
  const uint32_t available = scene_read_action.available_bytes();
  const uint32_t limit = available < s.total ? available : s.total;
  if (limit > s.loaded) {
    load.hash = streaming_hash_slice(load.hash, region + s.loaded, limit - s.loaded);
    s.loaded = limit;
  }
  if (load.read_complete && StreamReadAction::dma_idle()) {
    if (!load.read_ok) { scene_file_fail(SceneFileError::Read); return; }
    if (s.loaded != s.total) {
      // The final sectors became visible only after the callback.
      load.hash = streaming_hash_slice(load.hash, region + s.loaded, s.total - s.loaded);
      s.loaded = s.total;
    }
    scene_file_publish(now);
  }
#endif
}

// After a failure, start the same file again once no read owns the region.
inline bool scene_file_retry(uint32_t now) {
  if (scene_file_bank < 0 || scene_file_stats.state != SceneFileState::Failed) return false;
#if !EPOK_HOST_DATA
  if (scene_file_load.pending() || !StreamReadAction::dma_idle()) return false;
  scene_file_release_drive();
#endif
  scene_file_stats.state = SceneFileState::Idle;
  scene_file_begin(size_t(scene_file_bank), now);
  return scene_file_stats.state != SceneFileState::Failed;
}

// At boot, an external startup bank waits for its file like a transition.
inline void scene_file_boot(uint32_t now) {
  if constexpr (scene_file_count == 0) { (void)now; return; }
  if (!scene_bank_external(0)) return;
  scene_file_bank = 0;
  scene_file_begin(0, now);
}
} // namespace epok
