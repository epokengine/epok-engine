#pragma once
#include "pocketstation.hpp"

namespace epok {
// One owned, bounded transaction. In Play the request is posted and polled on
// later ticks, leaving the renderer free while the host bridge answers. Physical
// SIO uses one short sector/command at a time; callers pace whole operations.
class PocketRequest {
    uint8_t outgoing[140]{},incoming[140]{};
    uint16_t length_=0,received_=0;
    uint32_t token=0,elapsed=0;
    bool pending_=false,ready_=false;
    PocketError error_=PocketError::OK;
public:
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
        PocketFrame frame{outgoing,incoming,length,length_at};
        pocketstation.transfer(port,&frame,1);received_=frame.received;
        ready_=true;if(received_<2)error_=PocketError::NoCard;return true;
    }
    bool poll(uint32_t microseconds){
        if(!pending_)return ready_;
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
