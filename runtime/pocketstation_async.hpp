#pragma once
#include "pocketstation.hpp"
#if defined(__mips__) || defined(EPOK_POCKET_SIO_HOST_TEST)
#include "common/hardware/counters.h"
#endif

namespace epok {
// One owned, bounded transaction. In Play the request is posted and polled on
// later ticks, leaving the renderer free while the host bridge answers or a
// physical card recovers with /CS high. Only the actual SIO command is polled.
class PocketRequest {
    uint8_t outgoing[140]{},incoming[140]{};
    uint16_t length_=0,received_=0;
    uint32_t token=0,elapsed=0;
    bool pending_=false,ready_=false;
    PocketError error_=PocketError::OK;
#if defined(__mips__) || defined(EPOK_POCKET_SIO_HOST_TEST)
    uint16_t recovery_start_=0;
    int16_t length_at_=-1;
    uint8_t port_=0,attempt_=0;
    bool owns_bus_=false;
    static uint16_t hsync(){
        uint16_t before=COUNTERS[1].value;
        for(unsigned i=0;i<8;++i){
            const uint16_t after=COUNTERS[1].value;
            if(uint16_t(after-before)<=1)return after;
            before=after;
        }
        return before;
    }
    void release_bus(){if(owns_bus_){psyqo::SIO0Bus::release();owns_bus_=false;}}
#else
    void release_bus(){}
#endif
public:
    PocketRequest()=default;
    ~PocketRequest(){release_bus();}
    PocketRequest(const PocketRequest&)=delete;
    PocketRequest& operator=(const PocketRequest&)=delete;
    PocketRequest(PocketRequest&& other){*this=static_cast<PocketRequest&&>(other);}
    PocketRequest& operator=(PocketRequest&& other){
        if(this==&other)return *this;
        release_bus();
        for(unsigned i=0;i<sizeof(outgoing);++i){outgoing[i]=other.outgoing[i];incoming[i]=other.incoming[i];}
        length_=other.length_;received_=other.received_;token=other.token;elapsed=other.elapsed;
        pending_=other.pending_;ready_=other.ready_;error_=other.error_;
#if defined(__mips__) || defined(EPOK_POCKET_SIO_HOST_TEST)
        recovery_start_=other.recovery_start_;length_at_=other.length_at_;
        port_=other.port_;attempt_=other.attempt_;owns_bus_=other.owns_bus_;other.owns_bus_=false;
#endif
        other.pending_=false;other.ready_=false;return *this;
    }
    bool busy()const{return pending_;}
    bool ready()const{return ready_;}
    PocketError error()const{return error_;}
    uint16_t received()const{return received_;}
    const uint8_t* reply()const{return incoming;}
    bool begin(unsigned port,const uint8_t* data,uint16_t length,int16_t length_at=-1){
        if(pending_)return false;
        ready_=false;received_=0;elapsed=0;error_=PocketError::OK;
        if(port>1||!data||!length||length>sizeof(outgoing)||length_at>=int(length)){
            error_=port>1?PocketError::BadPort:PocketError::BadArgument;ready_=true;return false;
        }
        length_=length;for(unsigned i=0;i<length;++i){outgoing[i]=data[i];incoming[i]=0;}
        if(pocket_detail::mailbox_attached()){
            auto& box=epok_pocket_mailbox;
            if(box.request!=box.answered){error_=PocketError::Refused;ready_=true;return false;}
            box.port=port;box.frames=1;box.lengths[0]=length;
            for(unsigned i=0;i<length;++i)box.send[i]=outgoing[i];
            token=box.request+1;pending_=true;box.request=token;return true;
        }
#if defined(__mips__) || defined(EPOK_POCKET_SIO_HOST_TEST)
        if(psyqo::SIO0Bus::owned()){error_=PocketError::Busy;ready_=true;return true;}
        psyqo::SIO0Bus::acquire();owns_bus_=true;
        psyqo::Hardware::SIO::Ctrl=0;
        recovery_start_=hsync();port_=uint8_t(port);length_at_=length_at;attempt_=0;
        pending_=true;return true;
#else
        PocketFrame frame{outgoing,incoming,length,length_at};
        error_=pocketstation.transfer(port,&frame,1);received_=frame.received;
        ready_=true;if(error_==PocketError::OK&&received_<2)error_=PocketError::NoCard;return true;
#endif
    }
    bool poll(uint32_t microseconds){
        if(!pending_)return ready_;
#if defined(__mips__) || defined(EPOK_POCKET_SIO_HOST_TEST)
        if(owns_bus_){
            // 270 HSyncs exceed 17 ms in NTSC/PAL. Simulation catch-up ticks
            // can run back-to-back: only the free-running GPU counter counts.
            if(uint16_t(hsync()-recovery_start_)<270)return false;
            received_=pocket_detail::sio_exchange(port_,outgoing,incoming,length_,length_at_,true);
            // Retry only an unanswered address: no command byte was sent, so
            // even launch/link requests cannot execute twice through a retry.
            if(received_<2&&++attempt_<3){recovery_start_=hsync();return false;}
            if(received_<2)error_=PocketError::NoCard;
            pending_=false;ready_=true;release_bus();return true;
        }
#endif
        auto& box=epok_pocket_mailbox;
        if(box.answered==token){
            received_=box.received[0]<length_?box.received[0]:length_;
            for(unsigned i=0;i<received_;++i)incoming[i]=box.reply[i];
            pending_=false;ready_=true;if(received_<2)error_=PocketError::NoCard;
        }else{
            elapsed+=microseconds;
            if(elapsed>=5000000u){pending_=false;ready_=true;error_=PocketError::Timeout;}
        }
        return ready_;
    }
};
}
