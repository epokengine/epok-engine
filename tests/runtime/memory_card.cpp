#include <cassert>
#include <cstdio>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <vector>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/memory_card.hpp"
#include "../../runtime/input.hpp"
using namespace epok;
// An asynchronous driver deliberately retains borrowed pointers until pump(),
// as the real SDK does. Only sector transport is replaced; service code is real.
class Driver:public MemoryCardDriver {
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
void persistence_and_recovery(){
    MemoryCardService card;Driver driver;assert(card.attach(driver));
    char name[]="BASLUS-99999GAME",title[]="Owned title";uint8_t payload[]={1,2,3,4,5};
    assert(card.write(name,title,payload,sizeof(payload),1));
    std::memset(name,'X',sizeof(name)-1);std::memset(title,'Y',sizeof(title)-1);std::memset(payload,0,sizeof(payload));
    assert(!card.read("Other")&&card.status().last_rejection==CardError::Busy);driver.drain();
    assert(card.status().state==CardState::Succeeded&&card.size()==5&&card.data()[0]==1);assert(driver.port_seen==1&&driver.title_seen=="Owned title");
    assert(driver.records.count("BASLUS-99999GAME-A")==1);
    const uint8_t second[]={9,8,7,6};assert(card.write("BASLUS-99999GAME","Next",second,4));driver.drain();assert(card.data()[0]==9&&driver.records.size()==2);
    // Corrupt the newer record's sequence header: CRC rejects it, then A wins.
    driver.records["BASLUS-99999GAME-B"][8]^=0x20;
    assert(card.read("BASLUS-99999GAME"));driver.drain();assert(card.status().state==CardState::Succeeded&&card.size()==5&&card.data()[0]==1);
    // Interrupted inactive-copy overwrite cannot damage the last good record.
    driver.interrupt_write=true;assert(card.write("BASLUS-99999GAME","Retry",second,4));driver.drain();assert(card.status().error==CardError::Timeout);
    driver.interrupt_write=false;assert(card.read("BASLUS-99999GAME"));driver.drain();assert(card.data()[0]==1);
    driver.corrupt_write=true;assert(card.write("BASLUS-99999GAME","Verify",second,4));driver.drain();assert(card.status().error==CardError::VerificationFailed);
    driver.corrupt_write=false;assert(card.write("BASLUS-99999GAME","Good",second,4));driver.drain();assert(card.status().state==CardState::Succeeded&&card.data()[0]==9);
    assert(card.list());driver.drain();assert(card.file_count()==2);
}
void errors_and_bounds(){
    MemoryCardService card;assert(!card.probe()&&card.status().last_rejection==CardError::NotReady);Driver driver;card.attach(driver);
    assert(!card.probe(2)&&card.status().last_rejection==CardError::BadPort);
    assert(!card.read("1234567890123456789")&&card.status().last_rejection==CardError::NameTooLong);
    assert(!card.write("TEST","Title",nullptr,1));assert(!card.write("TEST","Title",nullptr,4097));
    assert(card.read("Missing"));driver.drain();assert(card.status().error==CardError::FileNotFound);
    driver.records["Corrupt-A"]={1,2,3};assert(card.read("Corrupt"));driver.drain();assert(card.status().error==CardError::BadData);
    driver.card_error=CardError::NotFormatted;assert(card.probe());driver.drain();assert(card.status().error==CardError::NotFormatted&&driver.writes==0);
    driver.card_error=CardError::NoCard;assert(card.write("TEST","Title",nullptr,0));driver.drain();assert(card.status().error==CardError::NoCard&&driver.writes==0);
    driver.card_error=CardError::OK;std::vector<uint8_t> max(MemoryCardService::max_payload,0x5a);assert(card.write("TEST","Title",max.data(),uint32_t(max.size())));driver.drain();assert(card.size()==max.size()&&card.data()[4095]==0x5a);
    auto completed=card.status().completed;assert(card.write("TEST","Empty",nullptr,0));driver.drain();assert(card.size()==0&&card.status().completed!=completed);
    // A freshly attached process-wide service can read saves made by the old one.
    MemoryCardService after_scene_reset;after_scene_reset.attach(driver);assert(after_scene_reset.read("TEST"));driver.drain();assert(after_scene_reset.status().state==CardState::Succeeded&&after_scene_reset.size()==0);
}
struct MockPad {
    enum class Pad:unsigned {First=0,Second=4};enum Button{Select=0};
    bool isPadConnected(Pad p)const {assert(unsigned(p)==0||unsigned(p)==4);return true;}
    bool isButtonPressed(Pad p,Button b)const{return unsigned(p)==4&&unsigned(b)==14;}
};
void physical_ports(){epok::Input input;MockPad pad;input.poll(pad,4);input.begin_tick();assert(!input.held(epok::Button::Cross,0));assert(input.pressed(epok::Button::Cross,1));}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    persistence_and_recovery();errors_and_bounds();physical_ports();std::puts("Memory Card async snapshots, paired save recovery, errors and physical pad ports passed.");
}
