#include "actor_scene_fixture.hpp"
#include "memory_card_fixture.hpp"
#include <cstdio>
using namespace epok;

static void metadata(const char* name,const char* title){
    MemoryCardLibrary::clear_staged_metadata();
    for(uint32_t i=0;name[i];++i)assert(MemoryCardLibrary::set_staged_name_byte(i,uint8_t(name[i])));
    for(uint32_t i=0;;++i){assert(MemoryCardLibrary::set_staged_title_byte(i,uint8_t(title[i])));if(!title[i])break;}
}
int main(){
    CardTestDriver driver;assert(memory_card.attach(driver));
    MemoryCardLibrary::clear_staged_metadata();
    assert(!MemoryCardLibrary::read_staged_name(0));
    assert(memory_card.status().last_rejection==CardError::NameTooLong);
    assert(!MemoryCardLibrary::set_staged_name_byte(18,'A'));
    assert(!MemoryCardLibrary::set_staged_name_byte(0,256));
    assert(!MemoryCardLibrary::set_staged_title_byte(128,'A'));
    assert(!MemoryCardLibrary::set_staged_title_byte(0,256));
    const char* name="GENERIC-NAMED-SAVE";
    metadata(name,"Project title");
    MemoryCardLibrary::clear_staged_payload();
    for(uint32_t i=0;i<23;++i)assert(MemoryCardLibrary::set_staged_word(i,0x12340000u+i));
    assert(MemoryCardLibrary::write_staged_named(92,1));
    const auto request=MemoryCardLibrary::snapshot().request;
    // Both metadata and all payload bytes belong to the accepted request.
    metadata("OTHER","Other title");MemoryCardLibrary::clear_staged_payload();
    assert(!MemoryCardLibrary::read_staged_name(0)&&memory_card.status().last_rejection==CardError::Busy);
    assert(MemoryCardLibrary::snapshot().request==request);
    driver.drain();assert(driver.port_seen==1&&driver.title_seen=="Project title");
    assert(driver.records.count(std::string(name)+"-A")==1);
    assert(memory_card.size()==92&&MemoryCardLibrary::loaded_word(22)==0x12340016u);
    metadata(name,"");assert(MemoryCardLibrary::read_staged_name(1));driver.drain();
    assert(MemoryCardLibrary::snapshot().completed!=request&&MemoryCardLibrary::loaded_word(0)==0x12340000u);
    // Exact maximum metadata lengths and full payload coexist, without borrowing
    // any of the payload's4096 bytes for the file name or BIOS title.
    MemoryCardLibrary::clear_staged_metadata();
    for(uint32_t i=0;i<18;++i)assert(MemoryCardLibrary::set_staged_name_byte(i,'N'));
    for(uint32_t i=0;i<128;++i)assert(MemoryCardLibrary::set_staged_title_byte(i,'T'));
    for(uint32_t i=0;i<1024;++i)assert(MemoryCardLibrary::set_staged_word(i,i^0xa5a5a5a5u));
    assert(!MemoryCardLibrary::write_staged_named(4097,0));
    assert(memory_card.status().last_rejection==CardError::FileTooLarge);
    assert(MemoryCardLibrary::write_staged_named(4096,0));driver.drain();
    assert(driver.records.count(std::string(18,'N')+"-A")==1&&driver.title_seen==std::string(128,'T'));
    assert(memory_card.size()==4096&&MemoryCardLibrary::loaded_word(1023)==(1023u^0xa5a5a5a5u));
    // An interrupted inactive-copy write and a disconnect retain the old copy.
    metadata(name,"Retry");MemoryCardLibrary::set_staged_word(0,99);driver.interrupt_write=true;
    assert(MemoryCardLibrary::write_staged_named(92,1));driver.drain();
    assert(memory_card.status().error==CardError::Timeout);driver.interrupt_write=false;
    assert(MemoryCardLibrary::read_staged_name(1));driver.drain();assert(MemoryCardLibrary::loaded_word(0)==0x12340000u);
    driver.card_error=CardError::NoCard;assert(MemoryCardLibrary::write_staged_named(92,1));driver.drain();
    assert(memory_card.status().error==CardError::NoCard);driver.card_error=CardError::OK;
    assert(MemoryCardLibrary::read_staged_name(1));driver.drain();assert(MemoryCardLibrary::loaded_word(22)==0x12340016u);
    puts("Named card API: bounds, full payload, async metadata ownership, interrupted writes and disconnect recovery passed");
}
