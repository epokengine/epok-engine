#define EPOK_TEST_STREAMING_ARCHIVE_FITS
#include "streaming-backend/streaming.hpp"
#include <cstdio>

using namespace epok;
static void reset() {
  stream_pool = {};
  for (auto &data : stream_page_data) data = nullptr;
  streaming_stats = {};
  stream_entry = {};
  stream_lookup_started = stream_lookup_pending = stream_ready = stream_failed = stream_read_pending = false;
  stream_warm_scene = false;
  stream_gameplay_request_count = 0;
  music_active = music_requested = music_lookup = music_ready = music_boot_failed = music_data_owner = false;
  fake_cd::pending = {};
  fake_cd::idle = fake_cd::read_ok = true;
  fake_cd::hung = fake_cd::corrupt = false;
  fake_cd::now = fake_cd::reads = fake_cd::pauses = fake_cd::idle_checks = 0;
}
int main() {
  static_assert(stream_archive_fits);
  psyqo::GPU gpu;
  reset();
  // Out-of-order reads still assign slots by immutable archive page ID.
  music_ready = true;
  streaming_lookup(); gpu.pumpCallbacks(); streaming_tick();
  assert(streaming_start_read(1));
  assert(stream_pool.find(1) == 1 && !stream_page_data[1]);
  assert(!stream_pool.pin(1));
  gpu.pumpCallbacks(); streaming_tick();
  assert(stream_page_data[1] == stream_pool.destination(1));
  const auto *one = stream_page_data[1];
  assert(one[1234] == 1);
  MeshGeometry mesh;
  mesh.stream_page = 1;
  StreamPageCursor cursor;
  const auto before_hits = stream_pool.hits;
  const auto before_time = fake_cd::now;
  for (unsigned i = 0; i < 1000; ++i) {
    const auto view = cursor.resolve(mesh, gpu, true);
    assert(view.valid && reinterpret_cast<const uint8_t *>(view.vertices) == one);
  }
  assert(stream_pool.hits == before_hits && fake_cd::now == before_time);
  assert(reinterpret_cast<const uint8_t *>(mesh.vertices) == one && mesh.quads);
  assert(!stream_pool.unpin(1)); // Raw views add no pin or lookup-counter writes.
  // Changing topology creates a fresh descriptor, without a previous binding.
  mesh = {};
  mesh.stream_page = 0;
  const auto zero = cursor.resolve(mesh, gpu);
  assert(zero.valid && stream_page_data[0] == stream_pool.destination(0));
  assert(streaming_stats.reads == 2 && streaming_stats.stalls == 1);
  assert(!stream_pool.unpin(0) && stream_pool.evictions == 0);
  assert(one[1234] == 1 && reinterpret_cast<const uint8_t *>(zero.vertices)[1234] == 0);
  // The pool policy itself forbids introducing a third identity, even with no pins.
  assert(stream_pool.reserve(2) == -1 && stream_pool.reserve(UINT32_MAX) == -1);
  assert(stream_pool.reserve(0) == 0 && stream_pool.reserve(1) == 1);
  assert(stream_pool.evictions == 0 && stream_page_data[1] == one);
  {
    StreamMeshLease lease(mesh, gpu, &cursor);
    assert(lease.valid());
    MeshGeometry other;
    other.stream_page = 1;
    assert(cursor.resolve(other, gpu).valid); // Stable views cannot invalidate the lease.
    assert(reinterpret_cast<const uint8_t *>(lease.vertices())[1234] == 0);
  }
  assert(cursor.release());
  mesh = {};
  mesh.stream_page = 0;
  mesh.stream_quad_offset = 3;
  assert(!cursor.resolve(mesh, gpu).valid && streaming_stats.errors == 1);
  mesh.stream_quad_offset = 8;
  mesh.stream_page = 2;
  assert(!cursor.resolve(mesh, gpu).valid);
  // Caller-supplied non-null pointers do not prove a streamed descriptor is
  // valid. Only the renderer's previously validated immutable chain may reuse
  // bindings without validation.
  MeshGeometry forged;
  forged.vertices = reinterpret_cast<const int16_t (*)[3]>(one);
  forged.quads = reinterpret_cast<const MeshQuad *>(one + 8);
  forged.stream_page = 2;
  assert(!cursor.resolve(forged, gpu).valid);
  forged.stream_page = 1;
  forged.stream_quad_offset = 3;
  assert(!cursor.resolve(forged, gpu).valid);

  reset();
  // A same-size bad checksum is never published. Existing good pointers also
  // cannot bypass the backend's global failure state.
  const auto *good = streaming_resolve_stable(0, gpu);
  assert(good && stream_page_data[0] == good);
  const MeshGeometry cached = [] { MeshGeometry value; value.stream_page = 0; return value; }();
  assert(cursor.resolve(cached, gpu).valid && cached.vertices && cached.quads);
  streaming_scene_changed();
  assert(cursor.resolve(cached, gpu).valid); // Scene changes preserve bindings.
  fake_cd::corrupt = true;
  const MeshGeometry bad = [] { MeshGeometry value; value.stream_page = 1; return value; }();
  assert(!cursor.resolve(bad, gpu).valid && !bad.vertices && !bad.quads);
  assert(stream_failed && streaming_stats.errors == 1 && !stream_page_data[1]);
  assert(!streaming_resolve_stable(0, gpu) && good[1234] == 0);
  assert(!cursor.resolve(cached, gpu).valid); // A cached descriptor cannot bypass failure.
  assert(stream_pool.failures == 1 && stream_pool.evictions == 0);

  reset();
  fake_cd::hung = true;
  assert(!streaming_resolve_stable(0, gpu));
  assert(streaming_stats.timeouts == 1 && !stream_page_data[0]);
  assert(music_data_owner && stream_lookup_pending); // Pending callback ownership survives timeout.
  fake_cd::hung = false;
  gpu.pumpCallbacks(); streaming_tick();
  assert(!streaming_resolve_stable(0, gpu));

  reset();
  MeshGeometry last;
  last.stream_page = 1;
  MeshGeometry first;
  first.stream_page = 0;
  first.next = &last;
  StreamObjectBinding binding;
  assert(streaming_bind_object(&first, binding, gpu));
  assert(binding.root == &first && binding.uses_stream && binding.all_streamed);
  assert(first.vertices && first.quads && last.vertices && last.quads);
  assert(streaming_stats.reads == 2 && streaming_stats.errors == 0);
  const auto bound_time = fake_cd::now;
  const auto bound_hits = stream_pool.hits;
  for (unsigned i = 0; i < 1000; ++i) assert(streaming_bind_object(&first, binding, gpu));
  assert(fake_cd::now == bound_time && stream_pool.hits == bound_hits);
  binding = {}; // Scene reset invalidates object state, never global payloads.
  assert(streaming_bind_object(&first, binding, gpu) && streaming_stats.reads == 2);
  MeshGeometry resident;
  MeshGeometry mixed;
  mixed.next = &first;
  assert(streaming_bind_object(&mixed, binding, gpu));
  assert(binding.uses_stream && !binding.all_streamed);
  stream_failed = true;
  assert(!streaming_bind_object(&mixed, binding, gpu));
  assert(streaming_bind_object(&resident, binding, gpu));
  assert(binding.root == &resident && !binding.uses_stream && !binding.all_streamed);

  reset();
  first = {}; last = {}; binding = {};
  first.stream_page = 0; first.next = &last;
  last.stream_page = 2;
  assert(!streaming_bind_object(&first, binding, gpu));
  assert(!binding.root && !first.vertices && streaming_stats.reads == 0);
  // Validation covers the entire chain before even a valid first page loads.
  assert(streaming_stats.errors == 1);
  last.stream_page = 1;
  assert(streaming_resolve_stable(0, gpu));
  assert(streaming_bind_object(&resident, binding, gpu));
  fake_cd::corrupt = true;
  assert(!streaming_bind_object(&first, binding, gpu));
  assert(first.vertices && !last.vertices && binding.root == &resident);
  assert(stream_failed && streaming_stats.reads == 2);
  assert(!streaming_bind_object(&first, binding, gpu));
  assert(streaming_bind_object(&resident, binding, gpu));
  std::puts("archive-fits streaming: fixed slots, checksum publication, zero-pin hot path, lease safety and failures passed");
}
