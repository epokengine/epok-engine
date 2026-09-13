#pragma once
#if __has_include("data-config.hh")
#include "data-config.hh"
#endif
#if EPOK_SERIAL_DEBUG
#include "serial_kernel.hpp"
#include "psyqo/kernel.hh"
#include "common/syscalls/syscalls.h"

namespace epok::serial_debug {
struct Memory {
  uint32_t read(uint32_t address) { return *reinterpret_cast<volatile uint32_t*>(address); }
  void write(uint32_t address, uint32_t value) { *reinterpret_cast<volatile uint32_t*>(address) = value; }
  void enqueue(uint32_t priority, uint32_t address) {
    syscall_sysEnqIntRP(int(priority), reinterpret_cast<HandlerInfo*>(address));
  }
};
inline SerialKernelState state;
inline Memory memory;
inline bool captured = false;
inline void capture() {
  // Main runs before PSYQo resets the BIOS queues. Keep interrupts disabled until
  // PSYQo finishes initialization and reenables them in Application::run().
  psyqo::Kernel::fastEnterCriticalSection();
  captured = state.capture(memory);
}
inline void restore() {
  if (!captured) psyqo::Kernel::abort("Cannot preserve the resident serial handler: invalid BIOS queues");
  state.restore(memory);
}
}
#else
namespace epok::serial_debug {
inline void capture() {}
inline void restore() {}
}
#endif
