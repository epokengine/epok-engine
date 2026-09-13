#pragma once
#include <stdint.h>

namespace epok {
// PSYQo clears I_MASK and BIOS interrupt queues during Application::run().
// Preserve only handlers whose code is resident below the executable's 64 KiB
// load boundary. Loader/application callbacks above that boundary are obsolete.
// Memory is an adapter so queue rebuilding can also be tested without a console.
struct SerialKernelState {
  struct Saved { uint32_t priority, address, handler, verifier; } saved[16]{};
  unsigned count = 0;
  uint32_t sio_mask = 0;
  static constexpr uint32_t sio_irq = 1u << 8;
  static bool ram(uint32_t address, uint32_t bytes) {
    const uint32_t segment = address >> 29;
    const uint32_t offset = address & 0x1fffffff;
    return (segment == 0 || segment == 4 || segment == 5) && !(address & 3)
        && offset >= 0x100 && bytes <= 0x200000 && offset <= 0x200000 - bytes;
  }
  static bool resident(uint32_t address) {
    return ram(address, 4) && (address & 0x1fffffff) < 0x10000;
  }
  static bool callback(uint32_t address) {
    const uint32_t offset = address & 0x1fffffff;
    return address == 0 || resident(address)
        || (!(address & 3) && (address >> 29 == 4 || address >> 29 == 5)
            && offset >= 0x1fc00000 && offset < 0x1fc80000);
  }
  template<class Memory> bool capture(Memory& memory) {
    count = 0;
    sio_mask = memory.read(0x1f801074) & sio_irq;
    const uint32_t table = memory.read(0x100), bytes = memory.read(0x104);
    if (bytes != 32 || !ram(table, bytes)) return false;
    for (uint32_t priority = 0; priority < bytes / 8; ++priority) {
      uint32_t address = memory.read(table + priority * 8);
      unsigned visited = 0;
      while (address) {
        if (++visited > 32 || !ram(address, 16)) return false;
        const uint32_t handler = memory.read(address + 4), verifier = memory.read(address + 8);
        if (resident(handler) || resident(verifier)) {
          if (!resident(address) || !callback(handler) || !callback(verifier) || count == 16) return false;
          // Duplicate nodes indicate a cycle or an invalid shared chain.
          for (unsigned i = 0; i < count; ++i)
            if ((saved[i].address & 0x1fffffff) == (address & 0x1fffffff)) return false;
          saved[count++] = {priority, address, handler, verifier};
        }
        address = memory.read(address);
      }
    }
    return true;
  }
  template<class Memory> void restore(Memory& memory) {
    // BIOS enqueue prepends. Reverse iteration preserves resident handler order
    // while retaining the fresh PSYQo handlers at the tail of each priority.
    for (unsigned i = count; i > 0; --i) {
      const auto& entry = saved[i - 1];
      // Some BIOS implementations keep their own handlers in RAM too. PSYQo
      // may already have reinstalled the same node; do not create a list cycle.
      bool present = false;
      for (unsigned priority = 0; priority < 4 && !present; ++priority) {
        uint32_t current = memory.read(memory.read(0x100) + priority * 8);
        for (unsigned n = 0; current && n < 32 && ram(current, 16); ++n) {
          if ((current & 0x1fffffff) == (entry.address & 0x1fffffff)) { present = true; break; }
          current = memory.read(current);
        }
      }
      if (!present) {
        memory.write(entry.address + 4, entry.handler);
        memory.write(entry.address + 8, entry.verifier);
        memory.enqueue(entry.priority, entry.address);
      }
    }
    memory.write(0x1f801074, memory.read(0x1f801074) | sio_mask);
    count = 0;
  }
};
} // namespace epok
