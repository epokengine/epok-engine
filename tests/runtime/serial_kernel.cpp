#include "serial_kernel.hpp"
#include <array>
#include <cassert>
#include <cstdio>
#include <functional>
#include <list>

// Preserve PsyQo's timer dispatch semantics, including adding a timer inside
// the callback and removing a one-shot only after its callback returns.
struct GPU {
  struct Timer {std::function<void(uint32_t)> callback;uint32_t deadline,period;};
  std::list<Timer> timers;
  uint32_t current=0;
  unsigned peak=0;
  uint32_t now()const{return current;}
  void armTimer(uint32_t deadline,std::function<void(uint32_t)> callback){
    timers.push_back({callback,deadline,0});if(timers.size()>peak)peak=unsigned(timers.size());
  }
  void armPeriodicTimer(uint32_t period,std::function<void(uint32_t)> callback){
    timers.push_back({callback,current+period,period});
  }
  void pump(uint32_t time){
    current=time;bool done=false;
    while(!done){
      done=true;
      for(auto it=timers.begin();it!=timers.end();++it){
        if(int32_t(it->deadline-current)>0)continue;
        if(it->period)it->deadline+=it->period;
        it->callback(current);
        if(it->period)continue;
        timers.erase(it);done=false;break;
      }
    }
  }
};

struct Memory {
  std::array<uint32_t, 0x200000 / 4> words{};
  uint32_t mask = 0;
  uint32_t irq = 0, dicr = 0;
  unsigned enqueues = 0;
  uint32_t read(uint32_t address) {
    if (address == 0x1f801074) return mask;
    if (address == 0x1f801070) return irq;
    if (address == 0x1f8010f4) return dicr;
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
  // Exact stranded-DMA state captured on a physical PSX: DMA2 is complete,
  // DICR asserts an enabled GPU flag, but I_STAT has no DMA notification.
  memory.mask=0x149;memory.irq=0x341;memory.dicr=0x84c40000;
  unsigned delivered=0;
  const auto service=[&](){++delivered;memory.dicr=0x00c40000;};
  assert(epok::serial_recover_dma(memory,service)&&delivered==1);
  assert(!epok::serial_recover_dma(memory,service)&&delivered==1);
  memory.dicr=0x84c40000;memory.irq|=8;
  assert(!epok::serial_recover_dma(memory,service)); // Normal IRQ owns it.
  memory.irq&=~8;memory.mask&=~8u;
  assert(!epok::serial_recover_dma(memory,service)); // Intentionally masked.
  memory.mask|=8;memory.dicr=0x84400000;
  assert(!epok::serial_recover_dma(memory,service)); // GPU channel disabled.
  memory.dicr=0x04440000;
  assert(!epok::serial_recover_dma(memory,service)); // Master disabled.

  // Replay a two-hour session at 60 Hz, crossing the 32-bit microsecond wrap.
  // The previous periodic check stops after its overdue deadline exceeds the
  // signed comparison range; the replacement must keep polling every frame.
  GPU periodic,watchdog;unsigned old_polls=0,polls=0,frames=0;
  periodic.armPeriodicTimer(2000,[&](uint32_t){++old_polls;});
  epok::serial_schedule_dma_poll(watchdog,[&](){++polls;epok::serial_recover_dma(memory,service);});
  for(uint64_t time=16667;time<=7200000000ull;time+=16667){
    periodic.pump(uint32_t(time));watchdog.pump(uint32_t(time));++frames;
    assert(polls==frames&&watchdog.timers.size()==1);
  }
  assert(old_polls<frames&&watchdog.peak==2);
  // A GPU wait pumps more frequently than a frame. Even after the long run,
  // a newly stranded DMA event must be delivered on the next 2 ms deadline.
  memory.mask=0x149;memory.irq=0x141;memory.dicr=0x84c40000;
  watchdog.pump(watchdog.now()+1999);assert(delivered==1);
  watchdog.pump(watchdog.now()+1);assert(delivered==2&&polls==frames+1);
  watchdog.pump(watchdog.now()+1000000);assert(polls==frames+2&&watchdog.timers.size()==1);

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
  std::puts("Serial kernel preservation, stranded DMA and two-hour timer replay passed");
}
