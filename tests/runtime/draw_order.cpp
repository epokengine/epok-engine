#if defined(_WIN32)
#define _CRT_SECURE_NO_WARNINGS // Read-only standard getenv in this single-threaded test.
#endif
#include "../../runtime/draw_order.hpp"
#include <cassert>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <vector>

static std::vector<uint8_t> load(const char* path) {
    std::ifstream stream(path,std::ios::binary);assert(stream);
    return {std::istreambuf_iterator<char>(stream),std::istreambuf_iterator<char>()};
}
static uint32_t word(const std::vector<uint8_t>& data,size_t& at,unsigned bytes) {
    assert(at+bytes<=data.size());uint32_t value=0;
    for(unsigned i=0;i<bytes;++i)value|=uint32_t(data[at++])<<(i*8);
    return value;
}
static void fixture(const char* source,const char* encoded) {
    const auto raw=load(source), cooked=load(encoded);
    assert(raw.size()>=12&&cooked.size()>=16);
    assert(std::string(raw.begin(),raw.begin()+8)=="EPOKSEQ1");
    assert(std::string(cooked.begin(),cooked.begin()+8)=="EPOKORD1");
    size_t p=8,q=8;
    const auto count=word(raw,p,4),samples=word(cooked,q,4),capacity=word(cooked,q,4);
    assert(count==samples&&capacity<=65535);
    const size_t payload=16+samples*12;assert(payload<=cooked.size());
    std::vector<std::vector<uint32_t>> expected;
    std::vector<epok::DrawOrderSpan> spans;
    std::vector<uint32_t> a(capacity),b(capacity);size_t previous=0,total=0,end=0;
    for(size_t sample=0;sample<samples;++sample){
        const auto n=word(raw,p,2),cut=word(raw,p,2);
        epok::DrawOrderSpan span{};
        span.offset=word(cooked,q,4);span.bytes=word(cooked,q,4);span.count=uint16_t(word(cooked,q,2));
        const auto reset=word(cooked,q,2);assert(reset<=1);span.reset=reset!=0;
        assert(span.count==n&&(!cut||span.reset)&&span.reset==(cut||sample%32==0));
        assert(span.offset==end&&payload+span.offset+span.bytes<=cooked.size());end+=span.bytes;
        std::vector<uint32_t> wanted;for(size_t i=0;i<n;++i)wanted.push_back(word(raw,p,4));
        assert(epok::decode_draw_order(cooked.data()+payload+span.offset,span.bytes,
            a.data(),span.reset?0:previous,b.data(),capacity,n));
        assert(std::equal(wanted.begin(),wanted.end(),b.begin()));
        total+=n;expected.push_back(wanted);spans.push_back(span);a.swap(b);previous=n;
    }
    assert(p==raw.size()&&payload+end==cooked.size());
    // Random/reverse navigation must start from the closest reset, not depend
    // on whichever buffer happened to be displayed before a jump or camera cut.
    for(size_t target=samples;target-->0;){
        size_t first=target;while(first&&!spans[first].reset)--first;
        assert(spans[first].reset&&target-first<32);previous=0;
        for(size_t i=first;i<=target;++i){
            const auto& span=spans[i];
            assert(epok::decode_draw_order(cooked.data()+payload+span.offset,span.bytes,
                a.data(),span.reset?0:previous,b.data(),capacity,span.count));
            a.swap(b);previous=span.count;
        }
        assert(previous==expected[target].size()&&std::equal(expected[target].begin(),expected[target].end(),a.begin()));
    }
    std::cout<<"Ordered fixture: "<<samples<<" exact lists, "<<total<<" entries; all reverse seeks match.\n";
}
int main(){
    const uint32_t old[]={0,7,1000,2000,262143};uint32_t output[8]{};
    const uint8_t moved[]={0,4,2,0,0,3,1,17}; // max,0,7,2000,new17
    assert(epok::decode_draw_order(moved,sizeof(moved),old,5,output,8,5));
    const uint32_t wanted[]={262143,0,7,2000,17};
    for(unsigned i=0;i<5;++i)assert(output[i]==wanted[i]);
    for(size_t n=0;n<sizeof(moved);++n)assert(!epok::decode_draw_order(moved,n,old,5,output,8,5));
    assert(!epok::decode_draw_order(moved,sizeof(moved),old,5,output,4,5));
    assert(!epok::decode_draw_order(moved,sizeof(moved),old,5,output,8,4));
    assert(!epok::decode_draw_order(moved,sizeof(moved),old,5,const_cast<uint32_t*>(old),5,5));
    assert(!epok::decode_draw_order(moved,sizeof(moved),nullptr,5,output,8,5));
    assert(!epok::decode_draw_order(moved,sizeof(moved),old,5,nullptr,8,5));
    assert(epok::decode_draw_order(nullptr,0,nullptr,0,nullptr,0,0));
    const uint8_t maximum_token[]={1,255,255,15};
    assert(epok::decode_draw_order(maximum_token,4,nullptr,0,output,8,1)&&output[0]==262143);
    const uint8_t archive_token[]={1,255,255,255,255,1};
    assert(epok::decode_draw_order(archive_token,6,nullptr,0,output,8,1)&&output[0]==0x1fffffffu);
    assert(!epok::decode_draw_order(nullptr,1,nullptr,0,output,8,1));
    std::vector<uint32_t> many(65535),copied(65535);
    for(size_t i=0;i<many.size();++i)many[i]=uint32_t(i);
    const uint8_t maximum_copy[]={252,255,7,0};
    assert(epok::decode_draw_order(maximum_copy,4,many.data(),many.size(),copied.data(),copied.size(),copied.size()));
    assert(many==copied);
    assert(!epok::decode_draw_order(maximum_copy,4,many.data(),65536,copied.data(),copied.size(),copied.size()));
    for(const auto& invalid:std::vector<std::vector<uint8_t>>{{0,5},{2,4},{1,128},{1,128,0},
        {1,255,255,255,255,2},{1,255,255,255,255,129},{128,128,128,0},{255,255,15,0}})
        assert(!epok::decode_draw_order(invalid.data(),invalid.size(),old,5,output,8,1));
    const char* source=std::getenv("EPOK_DRAW_ORDER_SEQUENCE"),*encoded=std::getenv("EPOK_DRAW_ORDER_ENCODED");
    assert(bool(source)==bool(encoded));if(source)fixture(source,encoded);
    std::cout<<"Ordered draw decoder: moves, literals, checkpoints, boundaries, aliasing and malformed streams pass.\n";
}
