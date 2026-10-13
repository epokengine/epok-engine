#if defined(_WIN32)
#define _CRT_SECURE_NO_WARNINGS
#endif
#include "../../runtime/ordered_draw.hpp"
#include <algorithm>
#include <cassert>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <memory>
#include <vector>

static void put16(std::vector<uint8_t>& data,size_t at,uint16_t v){data[at]=uint8_t(v);data[at+1]=uint8_t(v>>8);}
static void atomic_failure(){
    std::vector<uint8_t> page(65536);
    page[0]='O';page[1]='D';page[2]='R';page[3]='1';put16(page,4,4);put16(page,6,40);
    // Four checkpoint/literal samples, each containing two distinct values.
    for(unsigned i=0;i<4;++i){
        const size_t row=8+i*8;put16(page,row,uint16_t(40+i*3));put16(page,row+2,3);put16(page,row+4,2);put16(page,row+6,i?0:1);
        page[40+i*3]=3;page[41+i*3]=uint8_t(i+1);page[42+i*3]=uint8_t(i+11);
    }
    const epok::OrderedDrawBlock blocks[]={{7,0,0}};const uint16_t map[]={0,0,0,0};
    epok::OrderedDrawData data{blocks,map,nullptr,1,4,2,0};
    epok::OrderedDrawCursor<4> cursor;
    assert(!cursor.publish(nullptr,0xffff));
    auto ready=[&](uint32_t id)->const uint8_t*{return id==7?page.data():nullptr;};
    assert(cursor.prepare(&data,0,ready));assert(cursor.tokens()==nullptr&&!cursor.publish(&data,1));
    assert(cursor.publish(&data,0));assert(cursor.count()==2&&cursor.tokens()[0]==1);
    const auto* active=cursor.tokens();
    // Decode two good rows, then fail. Neither scratch alternation nor failure
    // may overwrite the published source list.
    page[49]=2;page[50]=9;page[51]=0; // copy beyond previous count
    assert(!cursor.prepare(&data,3,ready));
    assert(cursor.tokens()==active&&cursor.sample()==0&&active[0]==1&&active[1]==11);
    assert(!cursor.publish(&data,3));
    page[49]=3;page[50]=4;page[51]=14;
    assert(cursor.prepare(&data,3,ready));assert(cursor.tokens()==active&&active[0]==1);
    assert(cursor.publish(&data,3));assert(cursor.tokens()[0]==4&&cursor.tokens()[1]==14);
    assert(cursor.prepare(&data,1,ready));assert(cursor.publish(&data,1));assert(cursor.tokens()[0]==2);
    epok::OrderedDrawData next=data;
    auto missing=[](uint32_t)->const uint8_t*{return nullptr;};
    assert(!cursor.prepare(&next,0,missing));assert(cursor.owner()==&data&&cursor.sample()==1&&cursor.tokens()[0]==2);
    assert(cursor.prepare(&next,0,ready));assert(cursor.owner()==&data);
    assert(cursor.publish(&next,0));assert(cursor.owner()==&next&&cursor.tokens()[0]==1);
    assert(!cursor.prepare(&data,4,ready));
    for(unsigned mode=0;mode<4;++mode){
        auto broken=page;
        if(mode==0)broken[0]='X';
        if(mode==1)put16(broken,4,33);
        if(mode==2)put16(broken,8+3*8,65535);
        if(mode==3)put16(broken,8+3*8+4,5);
        assert(!cursor.prepare(&data,3,[&](uint32_t){return broken.data();}));
        assert(cursor.owner()==&next&&cursor.tokens()[0]==1&&cursor.tokens()[1]==11);
    }
    cursor.reset();assert(cursor.tokens()==nullptr&&cursor.count()==0);
}
static std::vector<uint8_t> load(const char* path){
    std::ifstream file(path,std::ios::binary);assert(file);return {std::istreambuf_iterator<char>(file),std::istreambuf_iterator<char>()};
}
static uint32_t read(const std::vector<uint8_t>& bytes,size_t& at,unsigned count){
    assert(at+count<=bytes.size());uint32_t value=0;for(unsigned i=0;i<count;++i)value|=uint32_t(bytes[at++])<<(8*i);return value;
}
static void fixture(const char* archive_path,const char* fixture_path){
    const auto archive=load(archive_path),file=load(fixture_path);assert(file.size()>=20&&archive.size()%65536==0);
    assert(std::string(file.begin(),file.begin()+8)=="EPOKPGD1");
    size_t at=8;const auto block_count=read(file,at,4),sample_count=read(file,at,4),capacity=read(file,at,4);
    assert(block_count&&block_count<65536&&sample_count&&sample_count<65536&&capacity<=4096);
    std::vector<epok::OrderedDrawBlock> blocks;
    for(size_t i=0;i<block_count;++i){epok::OrderedDrawBlock b{};b.page=read(file,at,4);b.offset=uint16_t(read(file,at,2));b.first=uint16_t(read(file,at,2));blocks.push_back(b);}
    std::vector<uint16_t> map;for(size_t i=0;i<sample_count;++i)map.push_back(uint16_t(read(file,at,2)));
    std::vector<std::vector<uint32_t>> lists;
    size_t total=0;
    for(size_t i=0;i<sample_count;++i){const auto count=read(file,at,2);assert(count<=capacity);std::vector<uint32_t> list;
        for(size_t j=0;j<count;++j)list.push_back(read(file,at,4));lists.push_back(list);total+=count;}
    assert(at==file.size());
    const epok::OrderedDrawData data{blocks.data(),map.data(),nullptr,uint16_t(block_count),uint16_t(sample_count),uint16_t(capacity),0};
    auto cursor=std::make_unique<epok::OrderedDrawCursor<4096>>();
    auto page=[&](uint32_t id)->const uint8_t*{return id<archive.size()/65536?archive.data()+size_t(id)*65536:nullptr;};
    auto seek=[&](uint16_t sample){
        const auto active_sample=cursor->sample();const auto* active=cursor->tokens();const auto count=cursor->count();
        std::vector<uint32_t> prior;if(active)prior.assign(active,active+count);
        assert(cursor->prepare(&data,sample,page));
        assert(cursor->sample()==active_sample&&cursor->tokens()==active);
        if(active)assert(std::equal(prior.begin(),prior.end(),active));
        assert(cursor->publish(&data,sample));assert(cursor->count()==lists[sample].size());
        assert(std::equal(lists[sample].begin(),lists[sample].end(),cursor->tokens()));
    };
    for(size_t i=0;i<sample_count;++i)seek(uint16_t(i));
    for(size_t i=sample_count;i-->0;)seek(uint16_t(i));
    uint32_t seed=94244;for(unsigned i=0;i<1000;++i){seed=seed*1664525u+1013904223u;seek(uint16_t(seed%sample_count));}
    std::cout<<"Paged ordered fixture: "<<sample_count<<" lists / "<<total<<" entries; full forward/reverse and 1000 random atomic seeks match.\n";
}
int main(){
    atomic_failure();
    const auto* archive=std::getenv("EPOK_ORDERED_ARCHIVE");const auto* input=std::getenv("EPOK_ORDERED_NATIVE_FIXTURE");
    assert(bool(archive)==bool(input));if(archive)fixture(archive,input);
    std::cout<<"Ordered page cursor: publication, malformed late deltas, missing pages, bounds and scene changes pass.\n";
}
