#define EPOK_TEST_STREAMING_DISABLED
#include "streaming-backend/streaming.hpp"
#include <cstdio>

int main() {
  static_assert(sizeof(epok::stream_pool) < 128, "Disabled streaming must not reserve page payload memory");
  psyqo::GPU gpu;
  epok::MeshGeometry mesh;
  {
    epok::StreamPageCursor cursor;
    epok::StreamMeshLease lease(mesh, gpu, &cursor);
    assert(lease.geometry() == &mesh);
  }
  epok::streaming_prepare();
  epok::streaming_tick();
  assert(!epok::streaming_prefetch(0));
  assert(!epok::streaming_acquire(0, gpu));
  assert(epok::streaming_warmup(nullptr, 0, gpu));
  assert(fake_cd::idle_checks == 0 && fake_cd::reads == 0 && fake_cd::now == 0);
  std::puts("disabled streaming: no page allocation, metadata copy, CD polling or callback pumping passed");
}
