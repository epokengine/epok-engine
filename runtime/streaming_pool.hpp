#pragma once
#include <stddef.h>
#include <stdint.h>

namespace epok {
inline constexpr uint32_t stream_page_bytes = 64u * 1024u;
inline constexpr uint32_t stream_page_sectors = stream_page_bytes / 2048u;
inline constexpr uint32_t stream_invalid_page = UINT32_MAX;
inline uint32_t stream_page_hash(const uint8_t *data, size_t bytes = stream_page_bytes) {
  uint32_t hash = 2166136261u;
  for (size_t i = 0; i < bytes; ++i) hash = (hash ^ data[i]) * 16777619u;
  return hash;
}

// Platform independent page ownership. A DMA destination cannot be evicted,
// and callers must pin ready pages for as long as they retain payload pointers.
template <size_t Capacity, size_t PageBytes = stream_page_bytes, bool FixedSlots = false> class StreamPagePool {
  static_assert(Capacity > 0, "streaming needs at least one page slot");
  struct Slot {
    uint32_t page = 0, touched = 0, pins = 0;
    bool ready = false, occupied = false;
  };
  alignas(4) uint8_t data_[Capacity][PageBytes]{};
  Slot slots_[Capacity]{};
  uint32_t clock_ = 0;
  // Every initial byte is zero, including bookkeeping. Otherwise the linker
  // puts the entire inline object (and its large zero page payload) in .data.
  mutable int last_slot_ = 0;

public:
  static constexpr size_t capacity = Capacity;
  uint32_t hits = 0, misses = 0, evictions = 0, failures = 0;
  int find(uint32_t page) const {
    if (page == stream_invalid_page) return -1;
    // Consecutive chunks normally share a page. Validate the slot identity on
    // every hit: eviction or a failed DMA can never revive a stale page hint.
    if (slots_[last_slot_].occupied && slots_[last_slot_].page == page) return last_slot_;
    for (size_t i = 0; i < Capacity; ++i)
      if (slots_[i].occupied && slots_[i].page == page) { last_slot_ = int(i); return int(i); }
    return -1;
  }
  bool ready(uint32_t page) const {
    int i = find(page);
    return i >= 0 && slots_[i].ready;
  }
  bool has_free_slot() const {
    for (const auto &s : slots_) if (!s.occupied) return true;
    return false;
  }
  // Return the resident slot or reserve an unpinned slot for a read. Existing
  // in-flight reads remain owned by their original requester.
  int reserve(uint32_t page) {
    // Whole-archive pools permanently assign slot N to page N. No public
    // reservation can evict another page, including an unpinned ready page.
    if constexpr (FixedSlots) {
      if (page >= Capacity) return -1;
      auto &slot = slots_[page];
      if (!slot.occupied) {
        ++misses;
        slot = {page, ++clock_, 0, false, true};
      }
      return int(page);
    }
    if (page == stream_invalid_page) return -1;
    int found = find(page);
    if (found >= 0) {
      slots_[found].touched = ++clock_;
      return found;
    }
    ++misses;
    int victim = -1;
    uint32_t oldest_age = 0;
    for (size_t i = 0; i < Capacity; ++i) {
      const auto &s = slots_[i];
      if (!s.occupied) { victim = int(i); break; }
      if (!s.ready || s.pins) continue;
      uint32_t age = clock_ - s.touched;
      if (victim < 0 || age > oldest_age) { victim = int(i); oldest_age = age; }
    }
    if (victim < 0) return -1;
    if (slots_[victim].occupied) ++evictions;
    slots_[victim] = {page, ++clock_, 0, false, true};
    return victim;
  }
  uint8_t *destination(int slot) {
    return slot >= 0 && size_t(slot) < Capacity ? data_[slot] : nullptr;
  }
  void complete(int slot, bool success) {
    if (slot < 0 || size_t(slot) >= Capacity ||
        !slots_[slot].occupied || slots_[slot].ready) return;
    if (success) slots_[slot].ready = true;
    else { slots_[slot] = {}; ++failures; }
  }
  int pin_slot(uint32_t page) {
    int i = find(page);
    if (i < 0 || !slots_[i].ready || slots_[i].pins == UINT32_MAX) return -1;
    ++hits;
    ++slots_[i].pins;
    slots_[i].touched = ++clock_;
    return i;
  }
  const uint8_t *pin(uint32_t page) {
    return destination(pin_slot(page));
  }
  bool unpin_slot(int slot, uint32_t page) {
    if (slot < 0 || size_t(slot) >= Capacity ||
        !slots_[slot].occupied || slots_[slot].page != page || !slots_[slot].pins) return false;
    --slots_[slot].pins;
    return true;
  }
  bool unpin(uint32_t page) {
    return unpin_slot(find(page), page);
  }
  size_t resident_count() const {
    size_t n = 0;
    for (const auto &s : slots_) if (s.ready) ++n;
    return n;
  }
  static bool valid_range(uint32_t offset, uint32_t bytes, uint32_t alignment = 4) {
    return alignment && offset % alignment == 0 && offset <= stream_page_bytes &&
           bytes <= stream_page_bytes - offset;
  }
};
} // namespace epok
