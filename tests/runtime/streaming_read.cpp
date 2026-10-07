// Actual production action, with hardware registers and PsyQo dispatch stubbed.
#include <cassert>
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <functional>
#include <utility>
#include <vector>
#undef assert
inline void assert(bool value){if(!value){std::fputs("Sector transport assertion failed\n",stderr);std::abort();}}
namespace eastl { using std::function;using std::move;using std::atomic_signal_fence;
using std::memory_order_acquire;using std::memory_order_release; }
struct Dma {uintptr_t MADR=0;uint32_t BCR=0;volatile uint32_t CHCR=0;};
inline Dma DMA_CTRL[1];inline constexpr unsigned DMA_CDROM=0;
namespace psyqo {
namespace Kernel {inline void assert(bool value,const char*){::assert(value);}
[[noreturn]] inline void abort(const char*){::assert(false);std::abort();}}
struct MSF {uint32_t sector;explicit MSF(uint32_t n):sector(n){}
void toBCD(uint8_t* out)const{const uint32_t values[]={sector/4500,(sector/75)%60,sector%75};
for(unsigned i=0;i<3;++i)out[i]=uint8_t((values[i]/10)*16+values[i]%10);}};
struct CDRomDevice {
  struct Response {};
  uint8_t state=0;bool success=false;std::function<void(bool)> callback;
  bool isIdle()const{return state==0;}
  template<class State> class Action {
  protected:
    CDRomDevice* device_=nullptr;
    explicit Action(const char*){}
    void registerMe(CDRomDevice* d){device_=d;}
    void setCallback(std::function<void(bool)>&& cb){::assert(!device_->callback);device_->callback=std::move(cb);}
    void setState(State s){device_->state=uint8_t(s);}
    State getState()const{return State(device_->state);}
    void setSuccess(bool s){device_->success=s;}
    virtual bool dataReady(const Response&)=0;
    virtual bool acknowledge(const Response&)=0;
    virtual bool complete(const Response&)=0;
  };
  void finish(){state=0;auto cb=std::move(callback);callback={};cb(success);success=false;}
};
namespace Hardware {
namespace CDRom {
enum class CDL {SETLOC,SETMODE,READN,PAUSE};
struct Register {void throwAway(){}};
inline Register Ctrl,InterruptControl;inline uint32_t DataRequest=0;
inline std::vector<std::vector<uint32_t>> commands;
struct CommandRegister {template<class... Args>void send(CDL command,Args... args){
commands.push_back({uint32_t(command),uint32_t(args)...});}};
inline CommandRegister Command;
}
namespace SBus {inline uint32_t Dev5Ctrl=0,ComCtrl=0;}
}
}
#include "streaming-read/streaming_read.hpp"

int main(){
  using namespace epok;using namespace psyqo::Hardware::CDRom;
  for(uint32_t before=0;before<=32;++before)for(uint32_t after=before;after<=32;++after){
    assert(stream_read_committed_sectors(before,after,true)==(before?before-1:0));
    assert(stream_read_committed_sectors(before,after,false)==(before==after?before:(before?before-1:0)));
  }
  psyqo::CDRomDevice drive;alignas(4) uint8_t buffer[65536];unsigned callbacks=0;
  for(unsigned sectors:{1u,5u,32u}){
    commands.clear();bool success=false;
    stream_read_action.start(&drive,600,sectors,buffer,[&](bool ok){++callbacks;success=ok;});
    assert(stream_read_action.available_bytes()==0);
    assert(commands[0]==std::vector<uint32_t>({uint32_t(CDL::SETLOC),0,0x10,0}));
    assert(!stream_read_action.acknowledge({}));
    assert(commands.back()==std::vector<uint32_t>({uint32_t(CDL::SETMODE),0x80}));
    assert(!stream_read_action.acknowledge({}));
    assert(commands.back()==std::vector<uint32_t>({uint32_t(CDL::READN)}));
    assert(!stream_read_action.acknowledge({}));
    for(unsigned i=0;i<sectors;++i){
      assert(stream_read_action.available_bytes()==i*2048);
      assert(!stream_read_action.dataReady({}));
      assert(DMA_CTRL[0].MADR==reinterpret_cast<uintptr_t>(buffer+i*2048));
      assert(DMA_CTRL[0].BCR==0x10200&&DMA_CTRL[0].CHCR==0x11000000);
      assert(!stream_read_action.dma_idle()&&stream_read_action.available_bytes()==i*2048);
      if(i+1<sectors){DMA_CTRL[0].CHCR=0;assert(stream_read_action.available_bytes()==(i+1)*2048);}
    }
    assert(commands.back()==std::vector<uint32_t>({uint32_t(CDL::PAUSE)}));
    assert(!stream_read_action.acknowledge({}));assert(stream_read_action.complete({}));
    // Transport success may precede the last DMA drain; progress excludes it.
    drive.finish();assert(success&&!stream_read_action.dma_idle());
    assert(stream_read_action.available_bytes()==(sectors-1)*2048);
    DMA_CTRL[0].CHCR=0;assert(stream_read_action.available_bytes()==sectors*2048);
  }
  assert(callbacks==3);
  bool success=true;
  stream_read_action.start(&drive,600,2,buffer,[&](bool ok){++callbacks;success=ok;});
  for(unsigned i=0;i<3;++i)stream_read_action.acknowledge({});
  stream_read_action.dataReady({});const auto address=DMA_CTRL[0].MADR;
  // A second interrupt with DMA still busy cannot overwrite its destination.
  stream_read_action.dataReady({});assert(DMA_CTRL[0].MADR==address);
  assert(commands.back()==std::vector<uint32_t>({uint32_t(CDL::PAUSE)}));
  stream_read_action.acknowledge({});assert(stream_read_action.complete({}));
  drive.finish();assert(!success&&callbacks==4&&!stream_read_action.dma_idle());
  DMA_CTRL[0].CHCR=0;
  std::puts("Sector read action: command sequence, 1/5/32 sectors, DMA barriers, IRQ snapshot interleavings and overlap failure passed.");
}
