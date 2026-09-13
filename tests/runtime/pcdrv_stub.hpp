#pragma once
#include <cstring>
#define PCDRV_SEEK_SET 0
#define PCDRV_SEEK_END 2
namespace fake_host {
inline int size=4*65536,position=0,reads=0;
inline bool missing=false,short_read=false,corrupt=false,seek_error=false;
}
inline int PCinit(){return 0;}
inline int PCopen(const char* path,int flags,int){assert(std::strcmp(path,"GEOMETRY.BIN")==0&&flags==0);return fake_host::missing?-1:7;}
inline int PCclose(int){return 0;}
inline int PClseek(int fd,int offset,int mode){
    assert(fd==7);if(fake_host::seek_error)return -1;
    return fake_host::position=mode==PCDRV_SEEK_END?fake_host::size+offset:offset;
}
inline int PCread(int fd,void* buffer,int bytes){
    assert(fd==7&&bytes==65536);++fake_host::reads;
    std::memset(buffer,fake_host::position/65536,bytes);
    if(fake_host::corrupt)static_cast<uint8_t*>(buffer)[1234]^=1;
    fake_host::position+=bytes;return fake_host::short_read?bytes-1:bytes;
}
