// Sector transport adapted from PsyQo's cdrom-device-readsectors.cpp.
// MIT License
// Copyright (c) 2022 PCSX-Redux authors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
#pragma once
#include <EASTL/atomic.h>
#include "common/hardware/dma.h"
#include "psyqo/cdrom-device.hh"
#include "psyqo/hardware/cdrom.hh"
#include "psyqo/hardware/sbus.hh"
#include "psyqo/kernel.hh"
#include "psyqo/msf.hh"

namespace epok {
enum class StreamReadState : uint8_t { IDLE, SETLOC, SETMODE, READ, READ_ACK, PAUSE, PAUSE_ACK };

// A scheduled sector is not necessarily in RAM yet. Recheck the ISR's sequence
// around CHCR: an interrupt between the two reads makes the older prefix the
// conservative answer. The previous sector was drained before the next launch.
constexpr uint32_t stream_read_committed_sectors(uint32_t before,uint32_t after,bool busy) {
  return !busy&&before==after?before:(before?before-1:0);
}

class StreamReadAction final : public psyqo::CDRomDevice::Action<StreamReadState> {
  uint32_t remaining_ = 0;
  uint8_t* next_ = nullptr;
  volatile uint32_t scheduled_ = 0;
  bool failed_ = false;
  using CDL = psyqo::Hardware::CDRom::CDL;
  void pause() {
    setState(StreamReadState::PAUSE);
    psyqo::Hardware::CDRom::Command.send(CDL::PAUSE);
  }
public:
  StreamReadAction() : Action("Epok sector stream") {}
  static bool dma_idle() { return !(DMA_CTRL[DMA_CDROM].CHCR&0x01000000); }
  uint32_t available_bytes() const {
    const uint32_t before=scheduled_;
    eastl::atomic_signal_fence(eastl::memory_order_acquire);
    const bool busy=!dma_idle();
    eastl::atomic_signal_fence(eastl::memory_order_acquire);
    const uint32_t after=scheduled_;
    return stream_read_committed_sectors(before,after,busy)*2048;
  }
  void start(psyqo::CDRomDevice* device,uint32_t sector,uint32_t count,void* buffer,
             eastl::function<void(bool)>&& callback) {
    psyqo::Kernel::assert(device->isIdle()&&dma_idle()&&count,"Stream read started while busy or empty");
    registerMe(device);setCallback(eastl::move(callback));
    remaining_=count;next_=reinterpret_cast<uint8_t*>(buffer);scheduled_=0;failed_=false;
    setState(StreamReadState::SETLOC);
    eastl::atomic_signal_fence(eastl::memory_order_release);
    psyqo::MSF msf(sector+150);uint8_t bcd[3];msf.toBCD(bcd);
    psyqo::Hardware::CDRom::Command.send(CDL::SETLOC,bcd[0],bcd[1],bcd[2]);
  }
  bool dataReady(const psyqo::CDRomDevice::Response&) override {
    psyqo::Kernel::assert(getState()==StreamReadState::READ_ACK,"Stream dataReady in wrong state");
    // Never overwrite an unfinished DMA's registers or report its bytes ready.
    // Pause/drain and fail the page if the device cannot keep up.
    if(!dma_idle()){failed_=true;pause();return false;}
    psyqo::Hardware::CDRom::Ctrl.throwAway();
    psyqo::Hardware::CDRom::DataRequest=0;
    psyqo::Hardware::CDRom::InterruptControl.throwAway();
    psyqo::Hardware::CDRom::DataRequest=0x80;
    psyqo::Hardware::SBus::Dev5Ctrl=0x20943;
    psyqo::Hardware::SBus::ComCtrl=0x132c;
    DMA_CTRL[DMA_CDROM].MADR=reinterpret_cast<uintptr_t>(next_);
    DMA_CTRL[DMA_CDROM].BCR=512|0x10000;
    DMA_CTRL[DMA_CDROM].CHCR=0x11000000;
    next_+=2048;
    eastl::atomic_signal_fence(eastl::memory_order_release);
    scheduled_=scheduled_+1;
    if(--remaining_==0)pause();
    return false;
  }
  bool acknowledge(const psyqo::CDRomDevice::Response&) override {
    using namespace psyqo::Hardware::CDRom;
    switch(getState()){
      case StreamReadState::SETLOC:setState(StreamReadState::SETMODE);Command.send(CDL::SETMODE,0x80);break;
      case StreamReadState::SETMODE:setState(StreamReadState::READ);Command.send(CDL::READN);break;
      case StreamReadState::READ:setState(StreamReadState::READ_ACK);break;
      case StreamReadState::PAUSE:setState(StreamReadState::PAUSE_ACK);break;
      default:psyqo::Kernel::abort("Stream acknowledge in wrong state");
    }
    return false;
  }
  bool complete(const psyqo::CDRomDevice::Response&) override {
    psyqo::Kernel::assert(getState()==StreamReadState::PAUSE_ACK,"Stream complete in wrong state");
    setSuccess(!failed_&&remaining_==0);
    // Main-thread publication still waits for the final DMA to drain.
    return true;
  }
};
inline StreamReadAction stream_read_action;
}
