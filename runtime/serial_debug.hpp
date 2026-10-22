#pragma once
#if __has_include("data-config.hh")
#include "data-config.hh"
#endif
#if EPOK_SERIAL_DEBUG
#include "serial_kernel.hpp"
#include "psyqo/kernel.hh"
#include "psyqo/gpu.hh"
#include "common/kernel/events.h"
#include "common/syscalls/syscalls.h"
#include "common/hardware/pcsxhw.h"

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
inline uint32_t dma_recoveries = 0;
inline void poll() {
  const uint32_t status=psyqo::Kernel::Internal::getCop0Status();
  psyqo::Kernel::Internal::setCop0Status(status&~1u);
  asm volatile("":::"memory");
  if(serial_recover_dma(memory,[]{syscall_deliverEvent(EVENT_DMA,0x1000);}))++dma_recoveries;
  asm volatile("":::"memory");
  psyqo::Kernel::Internal::setCop0Status(status);
}
inline void install(psyqo::GPU& gpu) {
  // pumpCallbacks also runs inside flip/upload waits, where a missing DMA IRQ
  // would otherwise prevent the next GameScene::frame from ever being reached.
  serial_schedule_dma_poll(gpu,[]{poll();});
}
inline void capture() {
  // Main runs before PSYQo resets the BIOS queues. Keep interrupts disabled until
  // PSYQo finishes initialization and reenables them in Application::run().
  psyqo::Kernel::fastEnterCriticalSection();
  captured = state.capture(memory);
}
inline void restore() {
  if (!captured) psyqo::Kernel::abort("Cannot preserve the resident serial handler: invalid BIOS queues");
  // PSYQo enables Redux's kernel checker before prepare(). Rebuilding the
  // captured BIOS queues intentionally accesses kernel RAM outside an IRQ.
  pcsx_checkKernel(0);
  state.restore(memory);
  pcsx_checkKernel(1);
}
}
#else
namespace epok::serial_debug {
inline void capture() {}
inline void restore() {}
template<class GPU> inline void install(GPU&) {}
}
#endif
