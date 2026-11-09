#pragma once
#include "gameplay_api.hpp"
#include "pocketstation_async.hpp"

namespace epok {
struct EPOK_VALUE(Id="89a6d37a-e8c0-56ff-be3e-add9b8ecd1fe") PocketTransferHandle {
    ObjectId component{};
    uint32_t request=0;
};
struct EPOK_VALUE(Id="3999908b-fca5-5747-82d5-91f4ef54a607") PocketTransferSnapshot {
    bool valid=false,busy=false,ready=false;
    PocketError error=PocketError::BadArgument;
    uint32_t request=0,received=0;
};

// One owned bounded bus transaction. Packet formats, installation policy,
// producer identity, rewards and acknowledgement decisions belong to projects.
class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D|UI,Capability=world-gameplay,Id="abd3923b-cc40-589c-b755-5d607835a22f") PocketTransactionComponent:public ActorComponent {
    PocketRequest transfer_;
    uint8_t staged_[140]{};
    uint32_t sequence_=0,current_=0;
    PocketError rejection_=PocketError::OK;
    bool available()const{
        auto* self=const_cast<PocketTransactionComponent*>(this);
        auto* owner=self->get_owner();
        return active_object_registry&&active_object_registry->resolve<PocketTransactionComponent>(id())==this
            &&!m_ended&&owner&&!owner->pending_destruction()&&is_active(self);
    }
    bool matches(PocketTransferHandle value)const{return value.component==id()&&value.request&&value.request==current_;}
    void reset_transfer(){transfer_=PocketRequest{};current_=0;}
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("abd3923b-cc40-589c-b755-5d607835a22f");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) uint32_t port=0;
    EPOK_PROPERTY(EditAnywhere) bool automatic_poll=false;
    EPOK_FUNCTION(BlueprintPure) PocketError last_rejection()const{return rejection_;}
    EPOK_FUNCTION(BlueprintCallable) bool clear_request(){
        if(!available()||transfer_.busy())return false;
        for(auto& byte:staged_)byte=0;return true;
    }
    EPOK_FUNCTION(BlueprintCallable) bool set_request_byte(uint32_t index,uint32_t value){
        if(!available()||transfer_.busy()||index>=140||value>255)return false;
        staged_[index]=uint8_t(value);return true;
    }
    EPOK_FUNCTION(BlueprintCallable,AsyncRequest) PocketTransferHandle begin(uint32_t length,int32_t length_at){
        if(!available()){rejection_=PocketError::BadArgument;return {};}
        if(transfer_.busy()){rejection_=PocketError::Busy;return {};}
        if(port>1){rejection_=PocketError::BadPort;return {};}
        if(!length||length>140||length_at< -1||length_at>=int32_t(length)){rejection_=PocketError::BadArgument;return {};}
        current_=0;
        if(!transfer_.begin(port,staged_,uint16_t(length),int16_t(length_at))){rejection_=transfer_.error();return {};}
        if(!++sequence_)++sequence_;current_=sequence_;rejection_=PocketError::OK;
        return {id(),current_};
    }
    EPOK_FUNCTION(BlueprintCallable) bool poll(PocketTransferHandle value,uint32_t microseconds){
        if(!available()||!matches(value))return false;
        // A whole timeout is sufficient; widening an elapsed counter must not
        // wrap when a caller supplies an arbitrarily large positive delta.
        return transfer_.poll(microseconds>5000000u?5000000u:microseconds);
    }
    EPOK_FUNCTION(BlueprintPure) PocketTransferSnapshot snapshot(PocketTransferHandle value)const{
        if(!available()||!matches(value))return {};
        return {true,transfer_.busy(),transfer_.ready(),transfer_.error(),current_,transfer_.received()};
    }
    EPOK_FUNCTION(BlueprintPure) uint32_t reply_byte(PocketTransferHandle value,uint32_t index)const{
        return available()&&matches(value)&&transfer_.ready()&&index<transfer_.received()?transfer_.reply()[index]:0;
    }
    EPOK_FUNCTION(BlueprintCallable) bool cancel(PocketTransferHandle value){
        if(!available()||!matches(value))return false;
        reset_transfer();return true;
    }
    void tick(Fixed dt)override{
        if(automatic_poll&&current_&&dt.raw()>0){
            const uint64_t elapsed=uint64_t(uint32_t(dt.raw()))*1000000u/4096u;
            poll({id(),current_},uint32_t(elapsed>5000000u?5000000u:elapsed));
        }
    }
    void begin_play()override{reset_transfer();rejection_=PocketError::OK;}
    void on_disable()override{reset_transfer();}
    void end_play(EndPlayReason)override{reset_transfer();for(auto& byte:staged_)byte=0;}
};
}
