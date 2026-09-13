#pragma once
#include <stddef.h>
#include <stdint.h>

namespace epok {
enum class CardError : uint8_t {
    OK=0,NoCard,NotFormatted,BadChecksum,BadSector,Timeout,Unconnected,
    ProtocolError,DirectoryFull,OutOfSpace,FileNotFound,FileExists,NameTooLong,
    FileTooLarge,SerializeOverflow,BadData,BadPort,
    Busy=128,NotReady,InvalidArgument,VerificationFailed
};
enum class CardOperation : uint8_t { None,Probe,Read,Write,List };
enum class CardState : uint8_t { Idle,Busy,Succeeded,Failed };
struct CardStatus {
    CardState state=CardState::Idle;CardOperation operation=CardOperation::None;
    CardError error=CardError::OK,last_rejection=CardError::OK;
    uint32_t request=0,completed=0,payload_bytes=0;
};
struct CardIcon {uint8_t frame_count=1;uint16_t palette[16]={};uint8_t pixels[3][128]={};};
struct CardFile {char name[21]={};uint16_t blocks=0;};
// The adapter is process-owned. Completion callbacks never reference a scene,
// Behaviour, caller payload, filename, title, icon or result buffer.
class MemoryCardDriver {
public:
    using Completion=void(*)(void*,CardError);
    virtual bool idle() const=0;
    virtual void probe(unsigned,void*,Completion)=0;
    virtual void read(unsigned,const char*,void*,uint32_t,uint32_t*,void*,Completion)=0;
    virtual void write(unsigned,const char*,const char*,const CardIcon&,const void*,uint32_t,void*,Completion)=0;
    virtual void list(unsigned,CardFile*,uint32_t*,void*,Completion)=0;
};

class MemoryCardService {
public:
    static constexpr uint32_t max_payload=4096;
    static constexpr uint32_t header_size=20;
private:
    enum class Phase { None,Probe,First,Second,Write,Verify,List };
    MemoryCardDriver* driver=nullptr;CardStatus status_;Phase phase=Phase::None;
    unsigned port_=0;char base_[19]={},filename_[21]={},title_[129]={};CardIcon icon_;
    alignas(4) uint8_t scratch_[header_size+max_payload]={};
    uint8_t pending_[max_payload]={},result_[max_payload]={};
    CardFile files_[15];uint32_t file_count_=0,transferred_=0,pending_size_=0;
    uint32_t best_sequence_=0,write_sequence_=0,write_checksum_=0;int best_slot_=-1,write_slot_=0;
    bool corrupt_=false;
    static void copy(void* to,const void* from,size_t n) {auto* d=static_cast<uint8_t*>(to);auto* s=static_cast<const uint8_t*>(from);for(size_t i=0;i<n;++i)d[i]=s[i];}
    static bool string_copy(char* out,size_t capacity,const char* value) {
        if(!value)return false;size_t n=0;while(n<capacity&&value[n])++n;
        if(n>=capacity)return false;for(size_t i=0;i<=n;++i)out[i]=value[i];return true;
    }
    static uint32_t get32(const uint8_t* p){return uint32_t(p[0])|(uint32_t(p[1])<<8)|(uint32_t(p[2])<<16)|(uint32_t(p[3])<<24);}
    static void put32(uint8_t* p,uint32_t n){for(unsigned i=0;i<4;++i)p[i]=uint8_t(n>>(i*8));}
    static uint32_t checksum_update(uint32_t crc,const uint8_t* data,uint32_t n) {
        for(uint32_t i=0;i<n;++i){crc^=data[i];for(unsigned bit=0;bit<8;++bit)crc=(crc>>1)^(0xedb88320u&uint32_t(-int32_t(crc&1)));}
        return crc;
    }
    static uint32_t checksum(const uint8_t* record,uint32_t size) {
        return ~checksum_update(checksum_update(0xffffffffu,record,16),record+header_size,size);
    }
    bool reject(CardError error){status_.last_rejection=error;return false;}
    bool can_begin(unsigned port) {
        if(busy())return reject(CardError::Busy);
        if(!driver)return reject(CardError::NotReady);
        if(!driver->idle())return reject(CardError::Busy);
        if(port>1)return reject(CardError::BadPort);
        return true;
    }
    void begin(unsigned port,CardOperation operation) {
        port_=port;status_.operation=operation;status_.state=CardState::Busy;
        status_.error=status_.last_rejection=CardError::OK;status_.payload_bytes=0;
        ++status_.request;if(!status_.request)++status_.request;
        transferred_=file_count_=0;best_slot_=-1;best_sequence_=0;corrupt_=false;
    }
    void finish(CardError error) {
        status_.error=error;status_.state=error==CardError::OK?CardState::Succeeded:CardState::Failed;
        status_.completed=status_.request;phase=Phase::None;
        if(error!=CardError::OK)status_.payload_bytes=0;
    }
    void filename(int slot) {
        size_t n=0;while(base_[n]){filename_[n]=base_[n];++n;}
        filename_[n++]='-';filename_[n++]=slot?'B':'A';filename_[n]=0;
    }
    static void completion(void* owner,CardError error){static_cast<MemoryCardService*>(owner)->complete(error);}
    void scan(int slot) {
        phase=slot?Phase::Second:Phase::First;filename(slot);transferred_=0;
        driver->read(port_,filename_,scratch_,sizeof(scratch_),&transferred_,this,completion);
    }
    bool valid_record(uint32_t& sequence,uint32_t& size) const {
        if(transferred_<header_size||scratch_[0]!='U'||scratch_[1]!='Q'||scratch_[2]!='M'||scratch_[3]!='C'||get32(scratch_+4)!=1)return false;
        sequence=get32(scratch_+8);size=get32(scratch_+12);
        return size<=max_payload&&size<=transferred_-header_size&&checksum(scratch_,size)==get32(scratch_+16);
    }
    void complete(CardError error) {
        if(phase==Phase::Probe||phase==Phase::List){finish(error);return;}
        if(phase==Phase::First||phase==Phase::Second) {
            const bool second=phase==Phase::Second;
            if(error!=CardError::OK&&error!=CardError::FileNotFound&&error!=CardError::BadData){finish(error);return;}
            uint32_t sequence=0,size=0;
            if(error==CardError::OK&&valid_record(sequence,size)) {
                if(best_slot_<0||(sequence!=best_sequence_&&uint32_t(sequence-best_sequence_)<0x80000000u)) {
                    best_slot_=second?1:0;best_sequence_=sequence;status_.payload_bytes=size;
                    copy(result_,scratch_+header_size,size);
                }
            } else if(error!=CardError::FileNotFound)corrupt_=true;
            if(!second){scan(1);return;}
            if(status_.operation==CardOperation::Read){finish(best_slot_>=0?CardError::OK:(corrupt_?CardError::BadData:CardError::FileNotFound));return;}
            write_slot_=best_slot_==0?1:0;write_sequence_=best_slot_<0?1:best_sequence_+1;
            filename(write_slot_);scratch_[0]='U';scratch_[1]='Q';scratch_[2]='M';scratch_[3]='C';
            put32(scratch_+4,1);put32(scratch_+8,write_sequence_);put32(scratch_+12,pending_size_);
            copy(scratch_+header_size,pending_,pending_size_);
            write_checksum_=checksum(scratch_,pending_size_);put32(scratch_+16,write_checksum_);phase=Phase::Write;
            driver->write(port_,filename_,title_,icon_,scratch_,header_size+pending_size_,this,completion);return;
        }
        if(phase==Phase::Write) {
            if(error!=CardError::OK){finish(error);return;}
            phase=Phase::Verify;transferred_=0;
            driver->read(port_,filename_,scratch_,sizeof(scratch_),&transferred_,this,completion);return;
        }
        if(phase==Phase::Verify) {
            if(error!=CardError::OK){finish(error);return;}
            uint32_t sequence=0,size=0;
            if(!valid_record(sequence,size)||sequence!=write_sequence_||size!=pending_size_||get32(scratch_+16)!=write_checksum_){finish(CardError::VerificationFailed);return;}
            copy(result_,scratch_+header_size,size);status_.payload_bytes=size;finish(CardError::OK);
        }
    }
public:
    MemoryCardService()=default;
    MemoryCardService(const MemoryCardService&)=delete;
    MemoryCardService& operator=(const MemoryCardService&)=delete;
    bool attach(MemoryCardDriver& value){if(busy())return false;driver=&value;return true;}
    bool busy()const{return status_.state==CardState::Busy;}
    const CardStatus& status()const{return status_;}
    const uint8_t* data()const{return result_;}
    uint32_t size()const{return status_.payload_bytes;}
    const CardFile* files()const{return files_;}
    uint32_t file_count()const{return file_count_;}
    bool probe(unsigned port=0) {
        if(!can_begin(port))return false;begin(port,CardOperation::Probe);phase=Phase::Probe;driver->probe(port_,this,completion);return true;
    }
    bool list(unsigned port=0) {
        if(!can_begin(port))return false;begin(port,CardOperation::List);phase=Phase::List;driver->list(port_,files_,&file_count_,this,completion);return true;
    }
    bool read(const char* name,unsigned port=0) {
        if(!can_begin(port))return false;
        if(!name||!name[0]||!string_copy(base_,sizeof(base_),name))return reject(CardError::NameTooLong);
        begin(port,CardOperation::Read);scan(0);return true;
    }
    bool write(const char* name,const char* title,const void* data,uint32_t size,unsigned port=0,const CardIcon* icon=nullptr) {
        if(!can_begin(port))return false;
        if(size>max_payload)return reject(CardError::FileTooLarge);
        if((size&&!data)||!title)return reject(CardError::InvalidArgument);
        if(!name||!name[0]||!string_copy(base_,sizeof(base_),name))return reject(CardError::NameTooLong);
        if(!string_copy(title_,sizeof(title_),title))return reject(CardError::InvalidArgument);
        if(icon&&(icon->frame_count<1||icon->frame_count>3))return reject(CardError::InvalidArgument);
        if(icon)icon_=*icon;
        else {icon_=CardIcon{};icon_.palette[1]=0x7fff;for(auto& byte:icon_.pixels[0])byte=0x11;}
        copy(pending_,data,size);pending_size_=size;
        begin(port,CardOperation::Write);scan(0);return true;
    }
};
inline MemoryCardService memory_card;
inline const char* card_error_message(CardError error) {
    switch(error) {
    case CardError::OK:return "OK";case CardError::NoCard:return "No card";case CardError::NotFormatted:return "Card not formatted";
    case CardError::BadChecksum:return "Card transfer checksum failed";case CardError::BadSector:return "Bad card sector";case CardError::Timeout:return "Card timed out";
    case CardError::Unconnected:return "Port unavailable";case CardError::ProtocolError:return "Card protocol error";case CardError::DirectoryFull:return "Card directory full";
    case CardError::OutOfSpace:return "Not enough free card blocks";case CardError::FileNotFound:return "Save not found";case CardError::FileExists:return "File already exists";
    case CardError::NameTooLong:return "Save name needs 1-18 characters";case CardError::FileTooLarge:return "Save exceeds 4096 bytes";case CardError::SerializeOverflow:return "Serialization overflow";
    case CardError::BadData:return "No valid save copy";case CardError::BadPort:return "Card port must be 0 or 1";case CardError::Busy:return "Card operation in progress";
    case CardError::NotReady:return "Card service not initialized";case CardError::InvalidArgument:return "Invalid save argument";case CardError::VerificationFailed:return "Save verification failed";
    }return "Unknown card error";
}
}
