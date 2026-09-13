#include "serial_kernel.hpp"
#include <array>
#include <cassert>
#include <cstdio>

struct Memory {
  std::array<uint32_t, 0x200000 / 4> words{};
  uint32_t mask = 0;
  unsigned enqueues = 0;
  uint32_t read(uint32_t address) {
    if (address == 0x1f801074) return mask;
    return words.at((address & 0x1fffffff) / 4);
  }
  void write(uint32_t address, uint32_t value) {
    if (address == 0x1f801074) mask = value;
    else words.at((address & 0x1fffffff) / 4) = value;
  }
  void enqueue(uint32_t priority, uint32_t address) {
    ++enqueues;
    const auto slot = read(0x100) + priority * 8;
    write(address, read(slot)); write(slot, address);
  }
  void node(uint32_t address, uint32_t next, uint32_t handler, uint32_t verifier) {
    write(address, next); write(address + 4, handler); write(address + 8, verifier);
  }
  Memory() { write(0x100, 0x80000800); write(0x104, 32); }
};
static Memory memory;
int main() {
  using epok::SerialKernelState;
  SerialKernelState state;
  // Two resident nodes surround an obsolete loader callback. Keep only the
  // resident nodes, in order, and chain them ahead of fresh PSYQo handlers.
  memory.mask = SerialKernelState::sio_irq | 0x40;
  memory.write(0x808, 0x80001000);
  memory.node(0x1000, 0x80011000, 0x8000c000, 0xbfc00100);
  memory.node(0x11000, 0x80001020, 0x80020000, 0x80020040);
  memory.node(0x1020, 0, 0x8000c100, 0x8000c200);
  assert(state.capture(memory) && state.count == 2);
  memory.mask = 8;
  memory.write(0x808, 0x80002000);
  memory.node(0x2000, 0, 0xbfc10000, 0xbfc10100);
  memory.write(0x1004, 0); // Simulate initialization touching the saved node.
  state.restore(memory);
  assert(memory.read(0x808) == 0x80001000);
  assert(memory.read(0x1000) == 0x80001020);
  assert(memory.read(0x1020) == 0x80002000);
  assert(memory.read(0x1004) == 0x8000c000);
  assert(memory.mask == (8 | SerialKernelState::sio_irq));
  assert(memory.enqueues == 2 && state.count == 0);

  // A BIOS that keeps its default handlers in low RAM may have reinstalled
  // those exact nodes. Re-enqueuing them would create a cycle.
  assert(state.capture(memory));
  state.restore(memory);
  assert(memory.enqueues == 2);
  assert(memory.read(0x1020) == 0x80002000);

  // Malformed lists must stop rather than read arbitrary memory or hang boot.
  memory.write(0x1028, 0x80030000); // Mixed resident/obsolete loader callbacks.
  assert(!state.capture(memory));
  memory.write(0x1028, 0x8000c200);
  memory.write(0x1000, 0x80001000);
  assert(!state.capture(memory));
  memory.write(0x100, 0x1f801000);
  assert(!state.capture(memory));
  assert(!SerialKernelState::ram(0x80000101, 4));
  assert(!SerialKernelState::resident(0x80010000));
  assert(SerialKernelState::resident(0xa000c000));
  std::puts("Serial kernel preservation checks passed");
}
