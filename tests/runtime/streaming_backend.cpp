#include "streaming-backend/streaming.hpp"
#include <cstdio>
using namespace epok;

void reset() {
  stream_pool = {};
  streaming_stats = {};
  streaming_warmup_stats = {};
  stream_lookup_started = stream_lookup_pending = stream_ready = stream_failed = stream_read_pending = false;
  stream_warm_scene = false;
  stream_gameplay_request_count = 0;
  music_active = music_requested = music_lookup = music_ready = music_boot_failed = music_data_owner = false;
  fake_cd::pending = {};
  fake_cd::idle = fake_cd::read_ok = true;
  fake_cd::hung = fake_cd::corrupt = false;
  fake_cd::now = fake_cd::reads = fake_cd::pauses = 0;
}
int main() {
  static_assert(streaming_descriptor_valid(MeshGeometry{}));
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
  gpu.pumpCallbacks(); streaming_tick();
  auto ready_sample=sample_mesh_vertex(&requested_actor,1,CoordinateSpace::World);
  assert(ready_sample.success&&ready_sample.data_state==MeshDataState::Ready);
  assert(ready_sample.position[0].raw()==0x0101+4096);
  assert(!stream_pool.unpin(1));
  assert(!sample_mesh_vertex(&requested_actor,2,CoordinateSpace::Model).success);
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
      assert(reinterpret_cast<const uint8_t *>(third.vertices())[0] == 2);
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
      assert(!blocked.valid && reinterpret_cast<const uint8_t *>(first.vertices)[0] == 0);
    }
    const auto second = cursor.resolve(second_mesh, gpu);
    assert(second.valid && reinterpret_cast<const uint8_t *>(second.vertices)[0] == 1);
    const auto third = cursor.resolve(third_mesh, gpu);
    assert(third.valid && !stream_pool.ready(0));
    assert(cursor.release() && !stream_pool.unpin(2));
  }
  std::puts("streaming backend: CD failures/checksums, XA arbitration, descriptors, leases/cursors, warmup and predictor invalidation passed");
}
