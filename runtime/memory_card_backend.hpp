#pragma once
#include "memory_card.hpp"
#include "psyqo/memory-card.hh"
#include "psyqo/memory-card-filesystem.hh"

namespace epok {
class PsyqoMemoryCardDriver final:public MemoryCardDriver {
    psyqo::MemoryCard card;
    psyqo::MemoryCardFileSystem filesystem{card};
    psyqo::MemoryCardFileSystem::FileEntry directory[15];
    uint32_t directory_count=0;
    static psyqo::MemoryCard::Port port(unsigned p){return static_cast<psyqo::MemoryCard::Port>(p);}
    static CardError error(psyqo::MemoryCard::Error e){return static_cast<CardError>(e);}
public:
    void prepare(){card.prepare();memory_card.attach(*this);}
    bool idle()const override{return filesystem.isIdle();}
    void probe(unsigned p,void* owner,Completion complete)override {
        filesystem.getCardState(port(p),[owner,complete](auto e){complete(owner,error(e));});
    }
    void read(unsigned p,const char* name,void* buffer,uint32_t capacity,uint32_t* length,void* owner,Completion complete)override {
        filesystem.readFile(port(p),name,buffer,capacity,length,[owner,complete](auto e){complete(owner,error(e));});
    }
    void write(unsigned p,const char* name,const char* title,const CardIcon& icon,const void* data,uint32_t size,void* owner,Completion complete)override {
        psyqo::MemoryCardFileSystem::Icon native{};native.frameCount=icon.frame_count;
        for(unsigned i=0;i<16;++i)native.clut[i]=icon.palette[i];
        for(unsigned f=0;f<3;++f)for(unsigned i=0;i<128;++i)native.pixels[f][i]=icon.pixels[f][i];
        filesystem.writeFile(port(p),name,title,native,data,size,[owner,complete](auto e){complete(owner,error(e));});
    }
    void list(unsigned p,CardFile* output,uint32_t* count,void* owner,Completion complete)override {
        directory_count=0;
        filesystem.listFiles(port(p),directory,15,&directory_count,[this,output,count,owner,complete](auto e){
            *count=0;
            if(e==psyqo::MemoryCard::Error::OK) {
                *count=directory_count>15?15:directory_count;
                for(unsigned i=0;i<*count;++i){for(unsigned c=0;c<21;++c)output[i].name[c]=directory[i].name[c];output[i].name[20]=0;output[i].blocks=directory[i].sizeInBlocks;}
            }
            complete(owner,error(e));
        });
    }
};
static_assert(unsigned(CardError::BadPort)==unsigned(psyqo::MemoryCard::Error::BadPort));
}
