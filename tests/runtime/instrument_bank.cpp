#include "../../runtime/instrument_bank.hpp"
#include <cassert>
#include <cstdio>
#include <fstream>
#include <iterator>
#include <vector>
using namespace epok::instrument;
void w16(uint8_t* p, uint16_t v) { p[0] = uint8_t(v); p[1] = uint8_t(v >> 8); }
void w32(uint8_t* p, uint32_t v) { w16(p, uint16_t(v)); w16(p + 2, uint16_t(v >> 16)); }
int main(int argc, char** argv) {
    // Independent hand-built fixture, no Rust writer/helper involved.
    alignas(4) uint8_t data[320] = {'E','P','S','B',2,0,48,0,1,0,1,0};
    w32(data + 12, 48); w32(data + 16, 72); w32(data + 20, 216);
    w32(data + 24, 256); w32(data + 28, 320); w32(data + 32, 1); w32(data + 36, 2);
    for (auto [at, value] : {std::pair{48,256}, {52,64}, {56,11025}, {60,56}, {64,28}, {68,56}}) w32(data + at, value);
    data[72+7] = data[72+9] = 127; data[72+10] = 60; data[72+11] = data[72+12] = 255; data[72+13] = 1;
    w32(data + 72+20, 100); w16(data + 72+138, 1);
    for (int base : {72+40,72+72}) {
        for (int at : {0,4,8,12,20}) w32(data + base+at, uint32_t(-12000));
    }
    w32(data + 72+104, uint32_t(-12000)); w32(data + 72+120, uint32_t(-12000));
    w16(data + 216, 14|512); w16(data + 218, 16); w16(data + 220, Pitch); w32(data + 224, 12700);
    data[273] = 7; data[289] = 7; // one repeated data block, then terminal
    BankView bank{data, sizeof(data)};
    assert(bank.valid()); assert(bank.zone(0).root_key == 60); assert(bank.modulation(0).amount == 12700);
    for (uint32_t length = 0; length < sizeof(data); ++length) assert((!BankView{data,length}.valid()));
    w32(data+16, UINT32_MAX); assert(!bank.valid()); w32(data+16,72);
    w32(data+32,4097); assert(!bank.valid()); w32(data+32,1);
    w16(data+72+136,1); assert(!bank.valid()); w16(data+72+136,0);
    data[72+13]=2; assert(!bank.valid()); data[72+13]=1;
    data[272]=0x5c; assert(!bank.valid()); data[272]=0;
    data[273]=3; assert(!bank.valid()); data[273]=7;
    w16(data+220,0); assert(!bank.valid()); w16(data+220,Pitch);
    w16(data+216,0x8002); assert(!bank.valid()); w16(data+216,14|512);
    w32(data+72+40+12, uint32_t(-32768)); assert(!bank.valid()); w32(data+72+40+12, uint32_t(-12000));
    assert(bank.valid());
    for (int arg = 1; arg < argc; ++arg) {
        std::ifstream file(argv[arg], std::ios::binary);
        assert(file.good());
        std::vector<uint8_t> bytes{std::istreambuf_iterator<char>(file), {}};
        BankView exported{bytes.data(), uint32_t(bytes.size())};
        assert(exported.valid(false));
        std::printf("%s: %u samples, %u zones, %u modulators, %u SPU bytes, fits=%d\n",
            argv[arg], exported.sample_count(), exported.zone_count(), exported.modulation_count(), exported.sample_bytes(), exported.valid());
    }
    std::puts("instrument_bank: independent fixture, malformed bounds and exported payloads passed");
}
