#pragma once
#include <stdint.h>

// The saved economy is independent of a dungeon run. Cumulative receipts make
// an interrupted PocketStation acknowledgement safe to retry on this save.
namespace dungeon_inventory {
constexpr uint32_t magic=0x41535431,limit=1000000;
struct Receipt {uint32_t serial=0,total=0;};
struct Save {
    uint32_t signature=magic,version=1,ore=0,credits=0,medkits=0,tanks=0,buster=0;
    Receipt receipts[8]{};
};
inline constexpr bool valid(const Save& s){
    if(s.signature!=magic||s.version!=1||s.ore>limit||s.credits>limit||s.medkits>9||s.tanks>9||s.buster>3)return false;
    for(const auto& r:s.receipts)if(r.total>limit)return false;
    return true;
}
inline constexpr int receipt(Save& s,uint32_t serial){
    for(int i=0;i<8;++i)if(s.receipts[i].serial==serial&&s.receipts[i].total)return i;
    for(int i=0;i<8;++i)if(!s.receipts[i].total){s.receipts[i].serial=serial;return i;}
    return -1;
}
inline constexpr int transfer(Save& s,uint32_t serial,uint32_t total,uint32_t claimed){
    if(total>limit||claimed>total)return -1;
    const int i=receipt(s,serial);if(i<0)return -2;
    const auto previous=s.receipts[i].total>claimed?s.receipts[i].total:claimed;
    if(total<previous)return -3; // Reinstalled or older card data: never replay a receipt.
    const uint32_t amount=total-previous;
    if(amount>limit-s.ore)return -4;
    s.ore+=amount;s.receipts[i].total=total;return int(amount);
}
inline constexpr bool sell(Save& s){
    if(!s.ore||s.ore>(limit-s.credits)/10)return false;
    s.credits+=s.ore*10;s.ore=0;return true;
}
inline constexpr uint32_t price(const Save& s,unsigned item){return item==0?30:item==1?80:120*(s.buster+1);}
inline constexpr bool buy(Save& s,unsigned item){
    if(item>2)return false;
    uint32_t& count=item==0?s.medkits:item==1?s.tanks:s.buster;
    const uint32_t cost=price(s,item);
    if(count>=(item==2?3u:9u)||s.credits<cost)return false;
    s.credits-=cost;++count;return true;
}
}
