#include "streaming_pool.hpp"
#include <cassert>
#include <cstdio>

using epok::StreamPagePool;
int main() {
  assert(epok::stream_page_hash(reinterpret_cast<const uint8_t *>("hello"), 5) == 0x4f9f2cabu);
  static StreamPagePool<2> pool;
  assert(pool.find(0) == -1 && !pool.ready(0));
  assert(pool.reserve(epok::stream_invalid_page) == -1);
  int first = pool.reserve(10);
  assert(first >= 0 && !pool.ready(10) && !pool.pin(10));
  pool.destination(first)[123] = 42;
  int second = pool.reserve(11);
  assert(second >= 0 && second != first);
  // Neither unfinished DMA destination may be evicted.
  assert(pool.reserve(12) == -1);
  pool.complete(first, true);
  const auto *pinned = pool.pin(10);
  assert(pinned && pinned[123] == 42);
  assert(pool.reserve(12) == -1);
  pool.complete(second, true);
  assert(pool.reserve(12) == second);
  assert(pool.ready(10) && !pool.ready(11) && pinned[123] == 42);
  pool.complete(second, false);
  assert(pool.find(12) == -1 && pool.failures == 1);
  assert(pool.reserve(13) == second);
  pool.complete(second, true);
  assert(pool.pin(13));
  assert(pool.reserve(14) == -1);
  assert(pool.unpin(10) && !pool.unpin(10));
  assert(pool.reserve(14) == first);
  pool.complete(first, true);
  assert(pool.resident_count() == 2);
  assert(pool.unpin(13));
  assert(pool.pin(14) && pool.unpin(14));
  // The older unpinned page is evicted, keeping the recently used page.
  assert(pool.reserve(15) == second);
  assert(pool.find(14) == first && pool.find(13) == -1);
  // Reuse a recently cached slot, then ask for its old page identity. Neither
  // lookup nor a stale slot release may affect the new page in that slot.
  assert(!pool.pin(13));
  pool.complete(second, true);
  int pinned_slot = pool.pin_slot(15);
  assert(pinned_slot == second);
  assert(!pool.unpin_slot(pinned_slot, 13));
  assert(pool.unpin_slot(pinned_slot, 15));
  assert(pool.evictions == 3);
  assert(pool.valid_range(65532, 4));
  assert(!pool.valid_range(65532, 8));
  assert(!pool.valid_range(UINT32_MAX - 3, 8));
  assert(!pool.valid_range(1, 4));
  assert(pool.valid_range(2, 6, 2));
  assert(!pool.valid_range(0, 4, 0));
  std::puts("streaming pool: DMA ownership, pins, bounded eviction, LRU, read failures and ranges passed");
}
