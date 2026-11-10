#pragma once
#include <cassert>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <vector>
#include "../../runtime/memory_card.hpp"
using namespace epok;
class CardTestDriver:public MemoryCardDriver {
    std::function<void()> pending;
public:
    std::map<std::string,std::vector<uint8_t>> records;
    CardError card_error=CardError::OK;bool interrupt_write=false,corrupt_write=false;
    unsigned port_seen=99,writes=0;std::string title_seen;
    bool idle()const override{return !pending;}
    void pump(){assert(pending);auto next=std::move(pending);pending={};next();}
    void drain(){for(unsigned i=0;pending&&i<12;++i)pump();assert(!pending);}
    void probe(unsigned port,void* owner,Completion complete)override {assert(idle());port_seen=port;pending=[=,this]{complete(owner,card_error);};}
    void read(unsigned port,const char* name,void* output,uint32_t capacity,uint32_t* size,void* owner,Completion complete)override {
        assert(idle());port_seen=port;pending=[=,this]{
            if(card_error!=CardError::OK){complete(owner,card_error);return;}
            auto it=records.find(name);if(it==records.end()){complete(owner,CardError::FileNotFound);return;}
            *size=uint32_t(it->second.size());if(*size>capacity)*size=capacity;
            std::memcpy(output,it->second.data(),*size);complete(owner,CardError::OK);
        };
    }
    void write(unsigned port,const char* name,const char* title,const CardIcon&,const void* data,uint32_t size,void* owner,Completion complete)override {
        assert(idle());port_seen=port;pending=[=,this]{
            if(card_error!=CardError::OK){complete(owner,card_error);return;}
            ++writes;title_seen=title;auto* bytes=static_cast<const uint8_t*>(data);
            records[name]=std::vector<uint8_t>(bytes,bytes+(interrupt_write?size/2:size));
            if(corrupt_write&&!records[name].empty())records[name].back()^=0x80;
            complete(owner,interrupt_write?CardError::Timeout:CardError::OK);
        };
    }
    void list(unsigned port,CardFile* output,uint32_t* count,void* owner,Completion complete)override {
        assert(idle());port_seen=port;pending=[=,this]{*count=0;for(auto& [name,data]:records){if(*count==15)break;std::strncpy(output[*count].name,name.c_str(),20);output[*count].blocks=1;++*count;}complete(owner,card_error);};
    }
};
