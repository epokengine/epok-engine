#include "streaming_pool.hpp"
#include <cassert>
#include <cstdio>

using epok::StreamPagePool;
int main() {
  uint8_t hash_input[257];
  for (unsigned i = 0; i < sizeof(hash_input); ++i) hash_input[i] = uint8_t(i * 43);
  for (unsigned split = 0; split <= sizeof(hash_input); ++split) {
    const auto prefix = epok::stream_page_hash_update(2166136261u, hash_input, split);
    assert(epok::stream_page_hash_update(prefix, hash_input + split, sizeof(hash_input) - split) ==
           epok::stream_page_hash(hash_input, sizeof(hash_input)));
  }
  assert(epok::stream_page_hash(reinterpret_cast<const uint8_t *>("hello"), 5) == 0x4f9f2cabu);
  epok::StreamPageWorkingSet<3> view;
  view.consider(9, 90);
  view.consider(3, 30);
  view.consider(7, 70);
  view.consider(4, 40);
  assert(view.count() == 3 && view.page(0) == 3 && view.page(1) == 4 && view.page(2) == 7);
  view.consider(7, 10);
  view.consider(2, 10);
  assert(view.page(0) == 2 && view.page(1) == 7 && view.page(2) == 3);
  assert(view.contains(7) && !view.contains(9));
  view.clear();
  assert(view.count() == 0 && view.page(0) == epok::stream_invalid_page);
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
