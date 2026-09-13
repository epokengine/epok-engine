#pragma once
#include "streaming_pool.hpp"
#include "music.hpp"
#include "psyqo/gpu.hh"
#include <new>
#if __has_include("data-config.hh")
#include "data-config.hh"
#endif
#if EPOK_HOST_DATA
#include "common/kernel/pcdrv.h"
#endif

namespace epok {
// Generated scene.hh provides stream_page_count, stream_pool_pages and
// stream_archive_path. Geometry metadata, collisions and textures stay resident.
// Disabled builds have no page payload allocation (only a four-byte placeholder
// in an otherwise dead-strippable object).
inline constexpr bool stream_archive_fits = stream_page_count > 0 && stream_page_count <= stream_pool_pages;
inline StreamPagePool<(stream_page_count > 0 ? stream_pool_pages : 1),
                      (stream_page_count > 0 ? stream_page_bytes : 4), stream_archive_fits> stream_pool;
// Fixed-slot pools never evict. Publish a stable pointer only after a complete,
// checksum-verified read; zero initialization keeps this small index in BSS.
inline const uint8_t *stream_page_data[stream_archive_fits ? stream_page_count : 1]{};
struct StreamingStats {
  uint32_t reads = 0, bytes = 0, stalls = 0, stall_us = 0,
           errors = 0, timeouts = 0, xa_interruptions = 0;
};
inline StreamingStats streaming_stats;
inline psyqo::ISO9660Parser::DirEntry stream_entry;
inline bool stream_lookup_started = false, stream_lookup_pending = false,
            stream_ready = false, stream_failed = false, stream_read_pending = false;
inline bool stream_warm_scene = false;
inline constexpr uint32_t stream_timeout_us = 10000000;
#if EPOK_HOST_DATA
inline int stream_host_file = -1;
#endif

inline void streaming_scene_changed() { stream_warm_scene = false; }
inline bool streaming_prefetch_needed() {
  return stream_page_count > 0 && !stream_warm_scene;
}

inline void streaming_prepare() {
#if !EPOK_HOST_DATA
  if constexpr (stream_page_count > 0) music_prepare(true);
#endif
}

// Release the controller only after every parser/read/XA callback has drained.
// In particular a timeout must never release or reuse an active DMA buffer.
inline void streaming_tick() {
  if constexpr (stream_page_count == 0) return;
  if (!music_data_owner) return;
  if (!stream_lookup_pending && !stream_read_pending && !music_active &&
      !music_lookup && music_drive.isIdle()) music_data_owner = false;
}

inline void streaming_lookup() {
#if EPOK_HOST_DATA
  if (stream_lookup_started) return;
  stream_lookup_started = true;
  PCinit();
  stream_host_file = PCopen("GEOMETRY.BIN", 0, 0);
  stream_ready = stream_host_file >= 0 &&
      PClseek(stream_host_file, 0, PCDRV_SEEK_END) == int(stream_page_count * stream_page_bytes);
  if (!stream_ready) { stream_failed = true; ++streaming_stats.errors; }
#else
  if (stream_lookup_started || !music_ready || music_active || music_lookup ||
      !music_drive.isIdle()) return;
  stream_lookup_started = stream_lookup_pending = true;
  music_data_owner = true;
  music_files.getDirentry(stream_archive_path, &stream_entry, [](bool ok) {
    stream_lookup_pending = false;
    // Exact length catches a truncated or mismatched-size archive.
    stream_ready = ok && stream_entry.type == psyqo::ISO9660Parser::DirEntry::FILE &&
                   uint64_t(stream_entry.size) == uint64_t(stream_page_count) * stream_page_bytes;
    if (!stream_ready) { stream_failed = true; stream_warm_scene = false; ++streaming_stats.errors; }
  });
#endif
}

inline bool streaming_start_read(uint32_t page) {
#if EPOK_HOST_DATA
  if (page >= stream_page_count || stream_failed || !stream_ready || stream_read_pending) return false;
  if (stream_pool.ready(page)) return true;
  const int slot = stream_pool.reserve(page);
  if (slot < 0) return false;
  stream_read_pending = true;
  ++streaming_stats.reads;
  bool ok = PClseek(stream_host_file, page * stream_page_bytes, PCDRV_SEEK_SET) == int(page * stream_page_bytes);
  if (ok) ok = PCread(stream_host_file, stream_pool.destination(slot), stream_page_bytes) == int(stream_page_bytes);
  if (ok) ok = stream_page_hash(stream_pool.destination(slot)) == stream_page_hashes[page];
  stream_pool.complete(slot, ok);
  if constexpr (stream_archive_fits) if (ok) stream_page_data[page] = stream_pool.destination(slot);
  stream_read_pending = false;
  if (ok) streaming_stats.bytes += stream_page_bytes;
  else { stream_failed = true; stream_warm_scene = false; ++streaming_stats.errors; }
  return true;
#else
  if (page >= stream_page_count || stream_failed || !stream_ready ||
      stream_read_pending || music_active || music_lookup || !music_drive.isIdle()) return false;
  if (stream_pool.ready(page)) return true;
  int slot = stream_pool.reserve(page);
  if (slot < 0) return false;
  music_data_owner = stream_read_pending = true;
  ++streaming_stats.reads;
  music_drive.readSectors(stream_entry.LBA + page * stream_page_sectors,
                         stream_page_sectors, stream_pool.destination(slot),
                         [slot, page](bool ok) {
    if (ok) ok = stream_page_hash(stream_pool.destination(slot)) == stream_page_hashes[page];
    stream_pool.complete(slot, ok);
    if constexpr (stream_archive_fits)
      if (ok) stream_page_data[page] = stream_pool.destination(slot);
    stream_read_pending = false;
    if (ok) streaming_stats.bytes += stream_page_bytes;
    else { stream_failed = true; stream_warm_scene = false; ++streaming_stats.errors; }
  });
  return true;
#endif
}

// Opportunistic only: XA playback and a pending XA request retain priority.
// Call after acquisition while its page is pinned to protect current geometry.
inline bool streaming_prefetch(uint32_t page) {
#if EPOK_HOST_DATA
  // PCDrv is synchronous. Speculative serial reads would stall visible gameplay.
  (void)page;
  return false;
#else
  if (page >= stream_page_count || stream_failed) return false;
  if (stream_pool.ready(page)) return true;
  // Prediction must not evict useful resident pages. Demand acquisition owns
  // the LRU eviction policy; speculative work uses only unused slots.
  if (!stream_pool.has_free_slot()) return false;
  if (music_active || music_requested || music_lookup || music_data_owner ||
      !music_ready || !music_drive.isIdle()) return false;
  if (!stream_ready) { streaming_lookup(); return false; }
  return streaming_start_read(page);
#endif
}

// Keep callback pumping and its stack frame out of the resident-page path.
#if defined(__GNUC__)
__attribute__((noinline))
#endif
inline int streaming_acquire_slot_slow(uint32_t page, psyqo::GPU &gpu) {
  // A demand miss proves that the active working set is no longer fully warm.
  stream_warm_scene = false;
  ++streaming_stats.stalls;
  uint32_t start = gpu.now();
  if (music_active && !music_data_owner) ++streaming_stats.xa_interruptions;
  music_data_owner = true;
  // Service device/timer callbacks without invoking engine script/frame
  // traversal. CD callbacks do not retain entity or geometry metadata pointers.
  bool requested = false;
  for (;;) {
    music_tick();
    gpu.pumpCallbacks();
    if (music_boot_failed) { stream_failed = true; ++streaming_stats.errors; }
    if (stream_failed) break;
    if (int slot = stream_pool.pin_slot(page); slot >= 0) {
      streaming_stats.stall_us += gpu.now() - start;
      streaming_tick();
      return slot;
    }
    streaming_lookup();
    if (stream_ready && !stream_read_pending) {
      if (requested) break; // A completed read must have produced a resident page.
      requested = streaming_start_read(page);
      if (!requested && music_drive.isIdle() && !music_active && !music_lookup)
        break; // Every slot is pinned; waiting cannot make progress here.
    }
    if (uint32_t(gpu.now() - start) >= stream_timeout_us) {
      stream_failed = true;
      ++streaming_stats.errors;
      ++streaming_stats.timeouts;
      break;
    }
  }
  streaming_stats.stall_us += gpu.now() - start;
  streaming_tick();
  return -1;
}

inline int streaming_acquire_slot(uint32_t page, psyqo::GPU &gpu) {
  if (page >= stream_page_count || stream_failed) return -1;
  if (int slot = stream_pool.pin_slot(page); slot >= 0) return slot;
  return streaming_acquire_slot_slow(page, gpu);
}

inline const uint8_t *streaming_acquire(uint32_t page, psyqo::GPU &gpu) {
  return stream_pool.destination(streaming_acquire_slot(page, gpu));
}
inline void streaming_release(uint32_t page) { stream_pool.unpin(page); }

// Only whole-archive pools permit unpinned payload views. FixedSlots enforces
// immutable page identity even for reservations made outside this backend.
// Failures remain global: a cached pointer must not bypass a later read error.
inline const uint8_t *streaming_resolve_stable(uint32_t page, psyqo::GPU &gpu) {
  if constexpr (!stream_archive_fits) return nullptr;
  if (page >= stream_page_count || stream_failed) return nullptr;
  if (const auto *data = stream_page_data[page]) return data;
  const int slot = streaming_acquire_slot(page, gpu);
  if (slot < 0) return nullptr;
  stream_pool.unpin_slot(slot, page);
  return stream_page_data[page];
}

struct StreamingWarmupStats {
  uint32_t attempts = 0, pages = 0, reads = 0, stall_us = 0, rejected = 0, failures = 0;
};
inline StreamingWarmupStats streaming_warmup_stats;

// The caller selects a startup working set from resident metadata.
// Validate the entire set before any I/O. A set that cannot fit is rejected,
// rather than repeatedly evicting pages and calling that work a warmup.
// All required pages remain pinned until the batch completes. Demand counters
// retain the real I/O cost; these separate counters identify startup work.
inline bool streaming_warmup(const uint32_t *pages, size_t count, psyqo::GPU &gpu) {
  if constexpr (stream_page_count == 0) return count == 0;
  ++streaming_warmup_stats.attempts;
  if (stream_failed) { ++streaming_warmup_stats.failures; return false; }
  if (!pages && count) { ++streaming_warmup_stats.rejected; return false; }
  uint32_t required[decltype(stream_pool)::capacity];
  int slots[decltype(stream_pool)::capacity];
  size_t unique = 0;
  for (size_t i = 0; i < count; ++i) {
    bool seen = false;
    for (size_t j = 0; j < unique; ++j) if (required[j] == pages[i]) { seen = true; break; }
    if (seen) continue;
    if (pages[i] >= stream_page_count || unique == decltype(stream_pool)::capacity) {
      ++streaming_warmup_stats.rejected;
      return false;
    }
    required[unique++] = pages[i];
  }
  const auto reads = streaming_stats.reads, stall_us = streaming_stats.stall_us;
  // Protect already resident members first, even when they occur late in the
  // input list. Loading a missing page must not evict another required member.
  for (size_t i = 0; i < unique; ++i) slots[i] = stream_pool.pin_slot(required[i]);
  bool success = true;
  for (size_t i = 0; i < unique; ++i) {
    if (slots[i] >= 0) continue;
    slots[i] = streaming_acquire_slot(required[i], gpu);
    if (slots[i] < 0) { success = false; break; }
  }
  if (success) streaming_warmup_stats.pages += uint32_t(unique);
  else ++streaming_warmup_stats.failures;
  for (size_t i = 0; i < unique; ++i)
    if (slots[i] >= 0) stream_pool.unpin_slot(slots[i], required[i]);
  streaming_warmup_stats.reads += streaming_stats.reads - reads;
  streaming_warmup_stats.stall_us += streaming_stats.stall_us - stall_us;
  return success;
}

// A bounded small-scene startup policy. Include every active object's page so
// camera/transform changes in the first script frame cannot invalidate a purely
// camera-based warmup. Large active sets are rejected before reading anything.
// The caller supplies its normal hierarchy-aware activity predicate.
template <class Entity, class Active>
inline bool streaming_warmup_scene(const Entity *objects, size_t count,
                                  Active &&active, psyqo::GPU &gpu) {
  stream_warm_scene = false;
  if constexpr (stream_page_count == 0) return true;
  uint32_t pages[decltype(stream_pool)::capacity + 1];
  size_t unique = 0;
  for (size_t i = 0; i < count; ++i) {
    if (!active(i)) continue;
    for (auto *mesh = objects[i].geometry; mesh; mesh = mesh->next) {
      if (mesh->stream_page == stream_invalid_page) continue;
      bool seen = false;
      for (size_t j = 0; j < unique; ++j)
        if (pages[j] == mesh->stream_page) { seen = true; break; }
      if (seen) continue;
      pages[unique++] = mesh->stream_page;
      if (unique > decltype(stream_pool)::capacity)
        return streaming_warmup(pages, unique, gpu);
    }
  }
  const bool success = streaming_warmup(pages, unique, gpu);
  stream_warm_scene = success;
  return success;
}

// Exported descriptors are immutable. A renderer may validate their whole
// geometry chain once, then retain that result until the chain changes.
// Arbitrary callers still get these checks from the default lease constructor.
constexpr bool streaming_descriptor_valid(const MeshGeometry &mesh) {
  return mesh.stream_page == stream_invalid_page ||
         (mesh.stream_page < stream_page_count &&
          mesh.vertex_count <= stream_page_bytes / 6 &&
          mesh.quad_count <= stream_page_bytes / sizeof(MeshQuad) &&
          mesh.stream_vertex_offset % 2 == 0 &&
          uint32_t(mesh.stream_vertex_offset) + uint32_t(mesh.vertex_count) * 6 <= stream_page_bytes &&
          mesh.stream_quad_offset % 4 == 0 &&
          uint32_t(mesh.stream_quad_offset) + uint32_t(mesh.quad_count) * sizeof(MeshQuad) <= stream_page_bytes);
}

// Scope-bound pin covers CPU geometry consumption. GPU packets contain copied
// vertices/material data; they must never retain pointers into the page pool.
struct StreamMeshView {
  const int16_t (*vertices)[3];
  const MeshQuad *quads;
  bool valid;
};

// Keep archive lookup and its uncommon validation/read branches outside the
// renderer's large frame function, whose register allocation affects polygons.
#if defined(__GNUC__)
__attribute__((noinline))
#endif
inline StreamMeshView streaming_resolve_archive_mesh(const MeshGeometry &mesh,
                                                     psyqo::GPU &gpu,
                                                     bool metadata_validated) {
  if (mesh.stream_page == stream_invalid_page) return {mesh.vertices, mesh.quads, true};
  if (!metadata_validated && !streaming_descriptor_valid(mesh)) {
    ++streaming_stats.errors;
    return {nullptr, nullptr, false};
  }
  const auto *data = streaming_resolve_stable(mesh.stream_page, gpu);
  if (!data) return {nullptr, nullptr, false};
  // Only payload bindings are mutable. Fixed slots preserve these pointers
  // across scene switches; page IDs, offsets and topology remain immutable.
  mesh.vertices = reinterpret_cast<const int16_t (*)[3]>(data + mesh.stream_vertex_offset);
  mesh.quads = reinterpret_cast<const MeshQuad *>(data + mesh.stream_quad_offset);
  return {mesh.vertices, mesh.quads, true};
}

struct StreamObjectBinding {
  const MeshGeometry *root = nullptr;
  bool uses_stream = false, all_streamed = false;
};

// Publish the root only after every immutable chunk has a valid binding. A
// partially read chain may hold safe pointers, but is never exposed for drawing.
#if defined(__GNUC__)
__attribute__((noinline))
#endif
inline bool streaming_bind_chain(const MeshGeometry *root, StreamObjectBinding &binding,
                                 psyqo::GPU &gpu) {
  if constexpr (!stream_archive_fits) return false;
  bool uses_stream = false, all_streamed = root != nullptr;
  for (auto *mesh = root; mesh; mesh = mesh->next) {
    const bool streamed = mesh->stream_page != stream_invalid_page;
    uses_stream |= streamed;
    all_streamed &= streamed;
    if (!streaming_descriptor_valid(*mesh)) { ++streaming_stats.errors; return false; }
  }
  if (uses_stream && stream_failed) return false;
  for (auto *mesh = root; mesh; mesh = mesh->next)
    if (mesh->stream_page != stream_invalid_page &&
        !streaming_resolve_archive_mesh(*mesh, gpu, true).valid) return false;
  binding = {root, uses_stream, all_streamed};
  return true;
}

inline bool streaming_bind_object(const MeshGeometry *root, StreamObjectBinding &binding,
                                  psyqo::GPU &gpu) {
  if (binding.root == root) return !binding.uses_stream || !stream_failed;
  return streaming_bind_chain(root, binding, gpu);
}

// Consecutive chunks can borrow one renderer-scoped page pin. Advancing the
// cursor is permitted only after its previous leases have finished consuming
// the payload, so an eviction cannot invalidate a live chunk.
class StreamPageCursor {
  uint32_t page_ = stream_invalid_page, borrowers_ = 0;
  int slot_ = -1;
  const uint8_t *data_ = nullptr;
  const uint8_t *seek(uint32_t page, psyqo::GPU &gpu) {
    if (page >= stream_page_count || stream_failed) return nullptr;
    if (page != page_) {
      if (!release()) return nullptr;
      slot_ = streaming_acquire_slot(page, gpu);
      if (slot_ < 0) return nullptr;
      page_ = page;
      data_ = stream_pool.destination(slot_);
    }
    return data_;
  }
public:
  StreamPageCursor() = default;
  StreamPageCursor(const StreamPageCursor &) = delete;
  StreamPageCursor &operator=(const StreamPageCursor &) = delete;
  ~StreamPageCursor() {
    if constexpr (stream_page_count > 0)
      if (slot_ >= 0) stream_pool.unpin_slot(slot_, page_);
  }
  bool release() {
    if (borrowers_) return false;
    if (slot_ >= 0) stream_pool.unpin_slot(slot_, page_);
    page_ = stream_invalid_page; slot_ = -1; data_ = nullptr;
    return true;
  }
  const uint8_t *borrow(uint32_t page, psyqo::GPU &gpu) {
    if (borrowers_ == UINT32_MAX || !seek(page, gpu)) return nullptr;
    ++borrowers_;
    return data_;
  }
  bool release_borrow(uint32_t page) {
    if (page_ != page || !borrowers_) return false;
    --borrowers_;
    return true;
  }
  // Renderer-only sequential view: consume all payload data before the next
  // resolve/advance/release. Its pointers expire when the held page changes.
  // Unlike a public lease this raw view owns no borrow counter or destructor.
  // Outstanding public leases still prevent advancement to another page.
  StreamMeshView resolve(const MeshGeometry &mesh, psyqo::GPU &gpu,
                         bool metadata_validated = false) {
    if constexpr (stream_page_count == 0) return {mesh.vertices, mesh.quads, true};
    if constexpr (stream_archive_fits) {
      if (mesh.stream_page == stream_invalid_page) return {mesh.vertices, mesh.quads, true};
      if (stream_failed) return {nullptr, nullptr, false};
      if (metadata_validated && mesh.vertices && mesh.quads) return {mesh.vertices, mesh.quads, true};
      return streaming_resolve_archive_mesh(mesh, gpu, metadata_validated);
    }
    if (mesh.stream_page == stream_invalid_page) return {mesh.vertices, mesh.quads, true};
    if (!metadata_validated && !streaming_descriptor_valid(mesh)) {
      ++streaming_stats.errors;
      return {nullptr, nullptr, false};
    }
    const auto *data = seek(mesh.stream_page, gpu);
    if (!data) return {nullptr, nullptr, false};
    return {reinterpret_cast<const int16_t (*)[3]>(data + mesh.stream_vertex_offset),
            reinterpret_cast<const MeshQuad *>(data + mesh.stream_quad_offset), true};
  }
};

class StreamMeshLease {
  // No initialization/copy of metadata for resident meshes. Start the union
  // member's lifetime only when a streamed payload actually needs rebinding.
  union { mutable MeshGeometry mesh_; };
  const MeshGeometry *source_;
  const uint8_t *data_ = nullptr;
  StreamPageCursor *cursor_ = nullptr;
  int slot_ = -1;
  mutable bool materialized_ = false;
  bool valid_ = true;
public:
  StreamMeshLease(const MeshGeometry &mesh, psyqo::GPU &gpu,
                  StreamPageCursor *cursor = nullptr,
                  bool metadata_validated = false) : source_(&mesh) {
    static_assert(sizeof(Material) == 24 && sizeof(MeshQuad) == 72,
                  "geometry archive material/quad ABI changed; update exporter");
    static_assert(offsetof(Material, texture) == 4 && offsetof(Material, blend) == 8 &&
                  offsetof(Material, depth_bias) == 12 && offsetof(Material, uv_scroll) == 16 &&
                  offsetof(MeshQuad, face) == 8 && offsetof(MeshQuad, normal) == 10 &&
                  offsetof(MeshQuad, material) == 16 && offsetof(MeshQuad, color_offset) == 40 &&
                  offsetof(MeshQuad, uv) == 44 && offsetof(MeshQuad, packed_uv) == 60 &&
                  offsetof(MeshQuad, uvw) == 62,
                  "geometry archive field offsets changed; update exporter");
    if constexpr (stream_page_count == 0) return;
    if (mesh.stream_page == stream_invalid_page) return;
    valid_ = false;
    if (!metadata_validated && !streaming_descriptor_valid(mesh)) {
      ++streaming_stats.errors;
      return;
    }
    if (cursor) {
      data_ = cursor->borrow(mesh.stream_page, gpu);
      if (!data_) return;
      cursor_ = cursor;
    } else {
      slot_ = streaming_acquire_slot(mesh.stream_page, gpu);
      if (slot_ < 0) return;
      data_ = stream_pool.destination(slot_);
    }
    valid_ = true;
  }
  ~StreamMeshLease() {
    if constexpr (stream_page_count > 0)
      if (cursor_) cursor_->release_borrow(source_->stream_page);
      else if (slot_ >= 0) stream_pool.unpin_slot(slot_, source_->stream_page);
  }
  StreamMeshLease(const StreamMeshLease &) = delete;
  StreamMeshLease &operator=(const StreamMeshLease &) = delete;
  bool valid() const { return valid_; }
  auto vertices() const -> const int16_t (*)[3] {
    return data_ ? reinterpret_cast<const int16_t (*)[3]>(data_ + source_->stream_vertex_offset) : source_->vertices;
  }
  const MeshQuad *quads() const {
    return data_ ? reinterpret_cast<const MeshQuad *>(data_ + source_->stream_quad_offset) : source_->quads;
  }
  // Compatibility accessor for users needing a full rebound descriptor. The
  // renderer can use valid()/vertices()/quads() and retain its original metadata
  // reference, so the normal resident-page path copies no MeshGeometry at all.
  const MeshGeometry *geometry() const {
    if (!valid_) return nullptr;
    if (!data_) return source_;
    if (!materialized_) {
      new (&mesh_) MeshGeometry(*source_);
      mesh_.vertices = vertices();
      mesh_.quads = quads();
      materialized_ = true;
    }
    return &mesh_;
  }
};
} // namespace epok
