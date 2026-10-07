#include "streaming-backend/streaming.hpp"
#include <cstdio>
using namespace epok;

void reset() {
  stream_pool = {};
  streaming_stats = {};
  streaming_verify_stats = {};
  stream_verify_overlap_bytes=0;
  stream_verification = {};
  streaming_warmup_stats = {};
  stream_lookup_started = stream_lookup_pending = stream_ready = stream_failed = stream_read_pending = false;
  stream_warm_scene = false;
  stream_gameplay_request_count = 0;
  music_active = music_requested = music_lookup = music_ready = music_boot_failed = music_data_owner = false;
  fake_cd::pending = {};
  fake_cd::idle = fake_cd::read_ok = true;
  fake_cd::hung = fake_cd::corrupt = false;
  fake_cd::corrupt_offset = 1234;
  fake_cd::available = 0;fake_cd::dma_busy = false;
  fake_cd::now = fake_cd::reads = fake_cd::pauses = 0;
}
int main() {
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.vertices={nullptr,7}; return m; }()));
  static_assert(streaming_descriptor_valid([] { MeshGeometry m; m.vertices={nullptr,12}; m.stream_page=0; m.stream_vertex_offset=65524; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.vertices={nullptr,12}; m.stream_page=0; m.stream_vertex_offset=65528; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.vertices={nullptr,12}; m.stream_page=0; m.stream_vertex_offset=2; return m; }()));
  static_assert(streaming_descriptor_valid(MeshGeometry{}));
  // Same terminal offset accepts one compact face but not a 72-byte face.
  static_assert(streaming_descriptor_valid([] { MeshGeometry m; m.stream_page=0; m.editable=3; m.stream_quad_offset=65504; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page=0; m.editable=1; m.stream_quad_offset=65504; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page=0; m.editable=3; m.stream_quad_offset=65508; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page = 0; m.vertex_count = SIZE_MAX; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page = 0; m.stream_quad_offset = 65532; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page = stream_page_count; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page = 0; m.stream_vertex_offset = 1; return m; }()));
  static_assert(!streaming_descriptor_valid([] { MeshGeometry m; m.stream_page = 0; m.stream_quad_offset = 2; return m; }()));
  psyqo::GPU gpu;
  // Gameplay acquisition queues media work and copies one vertex from a
  // briefly pinned resident page. The script-facing call itself never pumps.
  reset();
  MeshGeometry requested_mesh; requested_mesh.stream_page = 1;
  requested_mesh.vertex_count = 2;
  ActorData requested_actor{&requested_mesh};
  assert(mesh_geometry_state(&requested_mesh) == MeshDataState::Pending);
  assert(request_mesh_geometry(&requested_mesh));
  assert(stream_gameplay_request_count == 1 && fake_cd::reads == 0);
  streaming_service_gameplay_requests();
  assert(!stream_lookup_pending && fake_cd::reads == 0);
  music_tick();
  streaming_service_gameplay_requests();
  assert(stream_lookup_pending && fake_cd::reads == 0);
  gpu.pumpCallbacks(); streaming_tick();
  streaming_service_gameplay_requests();
  assert(stream_read_pending && fake_cd::reads == 1);
  auto pending_sample=sample_mesh_vertex(&requested_actor,0,CoordinateSpace::Model);
  assert(!pending_sample.success&&pending_sample.error==MeshVertexError::Pending);
  gpu.pumpCallbacks();
  assert(stream_verification.active && stream_read_pending && streaming_verify_stats.bytes == 0);
  assert(!stream_pool.ready(1) && !stream_pool.pin(1) && !streaming_retry());
  streaming_tick();
  assert(stream_verification.offset == stream_verify_slice_bytes && !stream_pool.ready(1));
  while (stream_verification.active) streaming_tick();
  assert(streaming_verify_stats.bytes == stream_page_bytes && streaming_verify_stats.batches == stream_page_bytes/stream_verify_slice_bytes);
  auto ready_sample=sample_mesh_vertex(&requested_actor,1,CoordinateSpace::World);
  assert(ready_sample.success&&ready_sample.data_state==MeshDataState::Ready);
  assert(ready_sample.position[0].raw()==0x0101+4096);
  assert(!stream_pool.unpin(1));
  assert(!sample_mesh_vertex(&requested_actor,2,CoordinateSpace::Model).success);
  // The same page is read at its declared width, including gameplay copies and
  // lease rebinding. A wide pointer must never silently become a narrow view.
  requested_mesh.vertices={nullptr,12};
  const auto wide_sample=sample_mesh_vertex(&requested_actor,1,CoordinateSpace::Model);
  assert(wide_sample.success&&wide_sample.position[0].raw()==0x01010101);
  {
    StreamMeshLease wide(requested_mesh,gpu);
    assert(wide.valid()&&wide.vertices().wide()&&wide.geometry()->vertices.wide());
    assert(wide.vertices()[1][2]==0x01010101);
  }
  assert(!stream_pool.unpin(1));
  reset();
  streaming_prepare();
  const uint8_t *first = streaming_acquire(1, gpu);
  assert(first && first[0] == 1 && fake_cd::reads == 1);
  assert(streaming_stats.stalls == 1 && streaming_stats.stall_us > 0);
  assert(!music_data_owner);
  // Prefetch is asynchronous and cannot evict a pinned page.
  assert(streaming_prefetch(2) && stream_read_pending);
  assert(!stream_pool.ready(2) && first[0] == 1);
  gpu.pumpCallbacks(); streaming_tick();
  while (stream_verification.active) streaming_tick();
  const auto *second = streaming_acquire(2, gpu);
  assert(second && second[0] == 2 && streaming_stats.stalls == 1);
  assert(!streaming_prefetch(3));
  assert(!streaming_acquire(3, gpu)); // All pool slots pinned: immediate safe failure.
  assert(streaming_stats.timeouts == 0 && !stream_failed);
  streaming_release(1); streaming_release(2);
  // Demand loads pause XA; they do not consume its pending play request.
  music_active = music_requested = true;
  assert(!streaming_prefetch(3));
  assert(streaming_acquire(3, gpu));
  assert(fake_cd::pauses == 1 && streaming_stats.xa_interruptions == 1 && music_requested);
  music_tick(); assert(music_active);
  streaming_release(3);
  // A lease validates offsets and releases its page when geometry is consumed.
  music_active = music_requested = false;
  MeshGeometry mesh; mesh.stream_page = 0;
  {
    StreamMeshLease lease(mesh, gpu);
    assert(lease.valid() && lease.vertices() && lease.quads());
    assert(!mesh.vertices && !mesh.quads); // Resident descriptor remains immutable.
    assert(lease.geometry() && lease.geometry()->vertices == lease.vertices() && lease.geometry()->quads == lease.quads());
  }
  assert(!stream_pool.unpin(0));
  mesh.stream_quad_offset = 65532;
  { StreamMeshLease invalid(mesh, gpu); assert(!invalid.geometry()); }
  reset(); fake_cd::read_ok = false;
  assert(!streaming_acquire(0, gpu));
  assert(stream_failed && stream_pool.resident_count() == 0 && streaming_stats.errors == 1);
  reset(); fake_cd::corrupt = true;
  assert(!streaming_acquire(0, gpu));
  assert(stream_failed && stream_pool.resident_count() == 0 && streaming_stats.errors == 1);
  assert(stream_pool.failures == 1 && streaming_stats.bytes == 0);
  reset(); music_ready = true;
  streaming_lookup(); gpu.pumpCallbacks(); streaming_tick();
  assert(stream_ready && streaming_prefetch(0));
  fake_cd::hung = true;
  assert(!streaming_acquire(0, gpu));
  assert(streaming_stats.timeouts == 1 && music_data_owner && stream_read_pending);
  // A timed-out DMA slot stays owned until its eventual callback arrives.
  assert(stream_pool.find(0) >= 0 && !stream_pool.ready(0));
  fake_cd::hung = false; gpu.pumpCallbacks(); streaming_tick();
  assert(!music_data_owner && !stream_read_pending && stream_failed);
  assert(!stream_verification.active && !stream_pool.ready(0) && stream_pool.find(0) < 0);
  assert(streaming_stats.errors == 1 && streaming_stats.bytes == 0 && streaming_verify_stats.batches == 0);
  // Warmup validates its working set before I/O and deduplicates shared pages.
  reset();
  const uint32_t oversized[] = {0, 1, 2};
  assert(!streaming_warmup(oversized, 3, gpu) && fake_cd::reads == 0);
  assert(streaming_warmup_stats.rejected == 1 && !stream_failed);
  const uint32_t initial[] = {0, 1, 0};
  assert(streaming_warmup(initial, 3, gpu));
  assert(stream_pool.ready(0) && stream_pool.ready(1));
  assert(streaming_warmup_stats.pages == 2 && streaming_warmup_stats.reads == 2);
  assert(streaming_warmup_stats.stall_us == streaming_stats.stall_us && streaming_stats.stall_us > 0);
  const auto reads_before = fake_cd::reads;
  const auto stalls_before = streaming_stats.stalls;
  assert(streaming_acquire(0, gpu) && streaming_acquire(1, gpu));
  assert(fake_cd::reads == reads_before && streaming_stats.stalls == stalls_before);
  streaming_release(0); streaming_release(1);
  // Protect existing required pages before loading any missing member, even
  // when the requested resident page is older and appears last in the list.
  const uint32_t changed[] = {2, 0};
  assert(streaming_warmup(changed, 2, gpu));
  assert(fake_cd::reads == reads_before + 1 && stream_pool.ready(0) && stream_pool.ready(2));
  assert(!stream_pool.unpin(0) && !stream_pool.unpin(2));
  reset(); fake_cd::corrupt = true;
  assert(!streaming_warmup(initial, 3, gpu));
  assert(streaming_warmup_stats.failures == 1 && stream_failed);
  reset();
  MeshGeometry first_mesh, second_mesh, third_mesh;
  first_mesh.stream_page = 0; second_mesh.stream_page = 1; third_mesh.stream_page = 2;
  first_mesh.next = &second_mesh;
  struct SceneObject { const MeshGeometry *geometry; bool active; };
  SceneObject scene[] = {{&first_mesh, true}, {&third_mesh, false}, {&second_mesh, true}};
  auto active = [&](size_t i) { return scene[i].active; };
  assert(streaming_warmup_scene(scene, 3, active, gpu));
  assert(fake_cd::reads == 2 && stream_pool.ready(0) && stream_pool.ready(1));
  assert(stream_warm_scene && !streaming_prefetch_needed());
  streaming_scene_changed();
  assert(!stream_warm_scene && streaming_prefetch_needed() && stream_pool.ready(0));
  assert(streaming_warmup_scene(scene, 3, active, gpu));
  assert(stream_warm_scene && fake_cd::reads == 2);
  assert(streaming_acquire(2, gpu));
  assert(!stream_warm_scene && streaming_prefetch_needed());
  streaming_release(2);
  reset(); scene[1].active = true;
  stream_warm_scene = true;
  assert(!streaming_warmup_scene(scene, 3, active, gpu));
  assert(fake_cd::reads == 0 && streaming_warmup_stats.rejected == 1 && !stream_warm_scene);
  reset();
  first_mesh.next = nullptr;
  {
    StreamPageCursor cursor;
    {
      StreamMeshLease a(first_mesh, gpu, &cursor);
      StreamMeshLease same(first_mesh, gpu, &cursor);
      assert(a.valid() && same.valid() && a.vertices() == same.vertices());
      assert(fake_cd::reads == 1 && stream_pool.hits == 1);
      assert(!cursor.release());
      // Advancing cannot invalidate a payload still consumed by either lease.
      StreamMeshLease premature(second_mesh, gpu, &cursor);
      assert(!premature.valid() && fake_cd::reads == 1 && a.vertices()[0][0] == 0);
    }
    {
      StreamMeshLease same_again(first_mesh, gpu, &cursor);
      assert(same_again.valid() && fake_cd::reads == 1 && stream_pool.hits == 1);
    }
    {
      StreamMeshLease second(second_mesh, gpu, &cursor);
      assert(second.valid() && fake_cd::reads == 2);
    }
    {
      StreamMeshLease third(third_mesh, gpu, &cursor);
      assert(third.valid() && fake_cd::reads == 3 && stream_pool.evictions == 1);
      assert(!stream_pool.ready(0));
      assert(static_cast<const uint8_t *>(third.vertices().data)[0] == 2);
    }
  }
  assert(!stream_pool.unpin(2)); // Renderer scope released its final held pin.
  reset();
  {
    StreamPageCursor cursor;
    { StreamMeshLease first(first_mesh, gpu, &cursor); assert(first.valid()); }
    fake_cd::read_ok = false;
    { StreamMeshLease failed(second_mesh, gpu, &cursor); assert(!failed.valid()); }
    assert(stream_failed && !stream_pool.unpin(0));
  }
  reset();
  {
    StreamPageCursor cursor;
    const auto first = cursor.resolve(first_mesh, gpu);
    const auto same = cursor.resolve(first_mesh, gpu, streaming_descriptor_valid(first_mesh));
    assert(first.valid && same.valid && first.vertices == same.vertices && stream_pool.hits == 1);
    {
      StreamMeshLease protected_view(first_mesh, gpu, &cursor, streaming_descriptor_valid(first_mesh));
      assert(protected_view.valid());
      const auto blocked = cursor.resolve(second_mesh, gpu);
      assert(!blocked.valid && static_cast<const uint8_t *>(first.vertices.data)[0] == 0);
    }
    const auto second = cursor.resolve(second_mesh, gpu);
    assert(second.valid && static_cast<const uint8_t *>(second.vertices.data)[0] == 1);
    const auto third = cursor.resolve(third_mesh, gpu);
    assert(third.valid && !stream_pool.ready(0));
    assert(cursor.release() && !stream_pool.unpin(2));
  }
  // A corrupt LAST byte cannot be published during any earlier slice. The
  // reservation stays non-evictable even when all other slots are pinned.
  reset(); music_ready = true;
  streaming_lookup(); gpu.pumpCallbacks(); streaming_tick();
  fake_cd::corrupt = true; fake_cd::corrupt_offset = stream_page_bytes - 1;
  assert(streaming_start_read(0)); gpu.pumpCallbacks();
  const int held = stream_pool.reserve(1); stream_pool.complete(held, true);
  assert(stream_pool.pin(1));
  for (unsigned slice = 0; slice < stream_page_bytes/stream_verify_slice_bytes-1; ++slice) {
    assert(!stream_pool.ready(0) && !stream_pool.pin(0));
    assert(stream_pool.reserve(2) == -1 && !streaming_retry());
    streaming_tick();
    assert(streaming_stats.bytes == 0 && streaming_stats.errors == 0);
  }
  assert(stream_verification.offset == stream_page_bytes-stream_verify_slice_bytes);
  streaming_scene_changed();
  assert(stream_read_pending && stream_verification.active && music_data_owner);
  streaming_tick();
  assert(stream_failed && !stream_read_pending && !stream_verification.active && !music_data_owner);
  assert(!stream_pool.ready(0) && stream_pool.find(0) < 0 && streaming_stats.bytes == 0);
  assert(streaming_stats.errors == 1 && streaming_verify_stats.bytes == stream_page_bytes);
  assert(stream_pool.unpin(1) && streaming_retry());
  // Failure during CPU verification can discard a drained DMA slot safely;
  // repeated ticks must not publish it or double-count the existing error.
  reset(); music_ready = true;
  streaming_lookup(); gpu.pumpCallbacks(); streaming_tick();
  assert(streaming_start_read(0)); gpu.pumpCallbacks(); streaming_tick();
  stream_failed = true; ++streaming_stats.errors;
  streaming_tick(); streaming_tick();
  assert(!stream_read_pending && !stream_verification.active && !music_data_owner);
  assert(!stream_pool.ready(0) && streaming_stats.bytes == 0 && streaming_stats.errors == 1);
  assert(streaming_retry());
  // Interleave 32 sector arrivals with main-thread hashing. Unarrived bytes are
  // poisoned, and multiple ticks without a new sector must not consume them.
  for(unsigned outcome:{0u,1u,2u}){
    const bool late_failure=outcome==1,corrupt_prefix=outcome==2;
    reset();music_ready=true;
    streaming_lookup();gpu.pumpCallbacks();streaming_tick();
    assert(streaming_start_read(0));
    const int slot=stream_pool.find(0);auto* data=stream_pool.destination(slot);
    fake_cd::corrupt=corrupt_prefix;fake_cd::corrupt_offset=0;
    std::memset(data,0xa5,stream_page_bytes);
    for(unsigned i=0;i<stream_page_sectors;++i){
      const auto offset=stream_verification.offset;
      streaming_tick();streaming_tick();assert(stream_verification.offset==offset);
      std::memset(data+i*2048,0,2048);fake_cd::available=(i+1)*2048;
      if(corrupt_prefix&&i==0)data[0]=1;
      streaming_tick();assert(stream_verification.offset==(i+1)*2048);
      assert(!stream_pool.ready(0)&&!stream_pool.pin(0)&&!streaming_retry());
      assert(stream_read_pending&&music_data_owner&&streaming_stats.bytes==0);
    }
    assert((stream_verification.hash==stream_page_hashes[0])==!corrupt_prefix);
    assert(!stream_verification.read_complete&&streaming_verify_stats.batches==32);
    assert(stream_verify_overlap_bytes==stream_page_bytes);
    // Hash can finish first. It is not transport success and cannot publish.
    streaming_tick();assert(stream_read_pending&&!stream_pool.ready(0));
    fake_cd::read_ok=!late_failure;gpu.pumpCallbacks();
    // Neither success nor failure may release while DMA still owns memory.
    fake_cd::dma_busy=true;streaming_tick();
    assert(stream_read_pending&&stream_verification.active&&!stream_pool.ready(0));
    fake_cd::dma_busy=false;streaming_tick();
    assert(!stream_read_pending&&!stream_verification.active&&!music_data_owner);
    const bool valid=!late_failure&&!corrupt_prefix;
    assert(stream_pool.ready(0)==valid);
    assert(streaming_stats.bytes==(valid?stream_page_bytes:0));
    assert(streaming_stats.errors==unsigned(!valid));
  }
  // Timeout after a verified prefix must keep the DMA reservation and stop
  // hashing; a late callback cannot turn that prefix into a successful page.
  reset();music_ready=true;
  streaming_lookup();gpu.pumpCallbacks();streaming_tick();assert(streaming_start_read(0));
  std::memset(stream_pool.destination(stream_pool.find(0)),0,2048);
  fake_cd::available=2048;streaming_tick();assert(stream_verification.offset==2048);
  fake_cd::hung=true;fake_cd::dma_busy=true;assert(!streaming_acquire(0,gpu));
  assert(streaming_stats.timeouts==1&&stream_read_pending&&music_data_owner);
  fake_cd::hung=false;gpu.pumpCallbacks();streaming_tick();
  assert(stream_read_pending&&stream_pool.find(0)>=0&&!stream_pool.ready(0));
  fake_cd::dma_busy=false;streaming_tick();
  assert(!stream_read_pending&&!stream_verification.active&&!music_data_owner);
  assert(streaming_stats.errors==1&&streaming_stats.bytes==0&&streaming_verify_stats.bytes==2048);
  assert(stream_verify_overlap_bytes==2048);
  assert(stream_pool.find(0)<0&&streaming_retry());
  // Background verification gets one sector; actual camera demand can borrow
  // one more, shared across repeated current/next/page-set readiness checks.
  reset();music_ready=true;
  streaming_lookup();gpu.pumpCallbacks();streaming_tick();assert(streaming_start_read(0));
  gpu.pumpCallbacks();streaming_tick();assert(stream_verification.offset==2048);
  bool serviced=false;
  assert(!streaming_required_page_ready(1,serviced)&&!serviced&&stream_verification.offset==2048);
  assert(!streaming_required_page_ready(0,serviced)&&serviced&&stream_verification.offset==4096);
  for(unsigned i=0;i<16;++i)assert(!streaming_required_page_ready(0,serviced));
  assert(stream_verification.offset==4096); // Never one boost per lookup/page.
  while(stream_verification.offset<stream_page_bytes-2048)streaming_tick();
  serviced=false;assert(streaming_required_page_ready(0,serviced)&&serviced);
  assert(!stream_verification.active&&!stream_read_pending&&streaming_stats.bytes==stream_page_bytes);
  serviced=false;assert(streaming_required_page_ready(0,serviced)&&!serviced);
  std::puts("streaming backend: interleaved sector checksums, final callback/DMA barriers, late failures, XA, queries, leases and warmup passed");
}
