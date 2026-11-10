#include "actor_scene_fixture.hpp"
#ifndef DRAFT_POLICY_HEADER
#define DRAFT_POLICY_HEADER "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonEconomyPolicy.hpp"
#endif
#include DRAFT_POLICY_HEADER
namespace legacy_reference {
#include "../fixtures/dungeon-legacy/DungeonInventory.hpp"
}
namespace old_inventory=legacy_reference::dungeon_inventory;
#include <cstring>
#include <cstdio>
using namespace epok;
using namespace old_inventory;
// Generated graphs check that their receiver survived each native call. Adopt
// the actual authored class rather than disabling those lifecycle checks.
struct RegisteredPolicy:public DungeonEconomyPolicy {
    inline static const ClassDescriptor descriptor{DungeonEconomyPolicy::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0};
    RegisteredPolicy(){assert(test_registry.adopt(*this,descriptor).valid());}
    ~RegisteredPolicy(){assert(test_registry.release(id()));}
};
struct LegacyGame {bool allowed=true;bool can_heal_player()const{return allowed;}};
enum class DungeonSound {UiError};
inline void cue(DungeonSound){}
struct {uint32_t transfers=0,error=0;} economy_telemetry;
struct LegacyEconomy {
    enum Menu {Closed,Pocket,Shop,Inventory};
    Menu menu=Closed;uint32_t selected=0;bool loaded=false,ack_pending=false,accept=true;
    Save saved{},pending{};int heal_pending=0;uint32_t ack_total=0,ack_serial=0,action=0;
    const char* notice="";
    struct Link {struct Status {uint32_t running_block=0,serial=0;} status;uint32_t block=0;int32_t values[8]{};} link;
    void message(const char* value){notice=value;}
    void load(){action=1;}
    void install(){action=2;}
    void transfer(){if(!loaded){message("Load your save first. Check memory card slot 1.");return;}action=3;}
    void acknowledge(){action=5;}
    bool commit(const char* value){notice=value;if(!accept){heal_pending=0;return false;}action=4;return true;}
    void activate(LegacyGame& game);void receive_transfer();
};
#include "../fixtures/dungeon-legacy/DungeonEconomyDecisions.inc"
static uint32_t seed=0x3518aa17u;
static uint32_t random_word(){seed=seed*1664525u+1013904223u;return seed;}
static DungeonEconomySave snapshot(const Save& value){DungeonEconomySave result;std::memcpy(&result,&value,sizeof(value));return result;}
static Save legacy(const DungeonEconomySave& value){Save result;std::memcpy(&result,&value,sizeof(value));return result;}
static bool same(const Save& left,const DungeonEconomySave& right){return !std::memcmp(&left,&right,sizeof(left));}
static Save random_save(){
    Save value;value.ore=random_word()%1000001u;value.credits=random_word()%1000001u;
    value.medkits=random_word()%10;value.tanks=random_word()%10;value.buster=random_word()%4;
    const auto occupied=random_word()%9;
    for(uint32_t n=0;n<8;++n){value.receipts[n].serial=random_word();value.receipts[n].total=n<occupied?random_word()%1000000+1:0;}
    return value;
}
int main(){
    active_object_registry=&test_registry;
    static_assert(sizeof(Save)==92&&sizeof(DungeonEconomySave)==sizeof(Save));
    unsigned checks=0,actions[7]{},errors[6]{},zero=0,awards=0;
    for(uint32_t iteration=0;iteration<100000;++iteration){
        auto original=random_save();RegisteredPolicy rules;rules.pending=snapshot(original);rules.saved=rules.pending;
        assert(valid(original)==rules.valid(rules.pending));++checks;
        uint32_t serial=iteration%3?random_word():original.receipts[iteration%8].serial;
        uint32_t total=random_word()%1000001,claimed=total?random_word()%(total+1):0;
        if(iteration%13==0)total=UINT32_MAX;if(iteration%17==0)claimed=UINT32_MAX;
        if(iteration%19==0){serial=original.receipts[0].serial;total=original.receipts[0].total;claimed=0;}
        const auto expected=old_inventory::transfer(original,serial,total,claimed);
        const auto actual=rules.transfer(serial,total,claimed);
        assert(actual.result==(expected<0?expected:0));assert(actual.amount==(expected>0?uint32_t(expected):0));assert(same(original,rules.pending));++checks;
        if(expected<0)++errors[-expected];else if(expected)++awards;else ++zero;
        original=random_save();rules.pending=snapshot(original);
        assert(sell(original)==rules.sell());assert(same(original,rules.pending));++checks;
        for(unsigned item=0;item<5;++item){
            auto bought=random_save();rules.pending=snapshot(bought);
            assert(price(bought,item)==rules.price(rules.pending,item));assert(buy(bought,item)==rules.buy(item));assert(same(bought,rules.pending));checks+=2;
        }
    }
    for(unsigned iteration=0;iteration<40000;++iteration){
        LegacyEconomy old;LegacyGame game;old.saved=random_save();old.pending=old.saved;
        old.loaded=iteration%5!=0;old.menu=LegacyEconomy::Menu(1+iteration%3);old.selected=random_word()%(old.menu==LegacyEconomy::Pocket?3:old.menu==LegacyEconomy::Shop?5:4);game.allowed=iteration%7!=0;
        RegisteredPolicy rules;rules.saved=snapshot(old.saved);rules.pending=rules.saved;rules.loaded=old.loaded;
        old.activate(game);const auto decision=rules.activate(old.menu,old.selected,game.allowed);
        assert(decision.action==old.action&&same(old.saved,rules.saved)&&same(old.pending,rules.pending));
        assert(rules.tx.heal==uint32_t(old.heal_pending)&&rules.tx.ack_pending==old.ack_pending);
        assert(decision.heal==0&&rules.tx.phase==(old.action==4?1u:0u));++actions[decision.action];++checks;
    }
    for(unsigned iteration=0;iteration<40000;++iteration){
        LegacyEconomy old;old.saved=random_save();old.pending=old.saved;old.loaded=true;
        old.link.status.serial=iteration%3?random_word():old.saved.receipts[iteration%8].serial;
        old.link.block=7;old.link.status.running_block=iteration%23?7:8;
        old.link.values[2]=iteration%29?magic:0;old.link.values[4]=iteration%31?1:0;
        old.link.values[0]=int32_t(random_word()%1000001);old.link.values[3]=int32_t(random_word()%(uint32_t(old.link.values[0])+1));
        RegisteredPolicy rules;rules.saved=snapshot(old.saved);rules.pending=rules.saved;rules.loaded=true;
        old.receive_transfer();const auto decision=rules.receive(old.link.status.running_block==old.link.block,uint32_t(old.link.values[2]),uint32_t(old.link.values[4]),old.link.status.serial,uint32_t(old.link.values[0]),uint32_t(old.link.values[3]));
        assert(decision.action==old.action&&same(old.saved,rules.saved)&&same(old.pending,rules.pending));
        if(old.action){assert(rules.tx.ack_pending==old.ack_pending&&rules.tx.serial==old.ack_serial&&rules.tx.total==old.ack_total);}
        ++checks;
    }
    // Full ledger, known-device growth, duplicate, reinstall and balance overflow.
    RegisteredPolicy p;p.loaded=true;
    for(unsigned n=0;n<8;++n){p.pending=p.saved;p.write_receipt(n,n+1,10);p.saved=p.pending;}
    assert(p.receive(true,magic,1,9,10,0).result==-2);
    auto decision=p.receive(true,magic,1,1,11,0);assert(decision.action==4&&decision.amount==1&&p.saved.ore==0);
    // Neither preparation nor a rejected/stale completion may acknowledge.
    assert(p.ack_started(true).notice==25&&p.save_completed(10,true).notice==25);
    assert(p.save_started(10,true).notice==20&&p.tx.phase==2&&p.saved.ore==0);
    assert(p.ack_started(true).notice==25&&p.receive(true,magic,1,1,11,0).notice==26);
    assert(p.save_completed(9,true).notice==25&&p.saved.ore==0);
    assert(p.save_completed(10,false).notice==21&&p.saved.ore==0&&!p.tx.ack_pending);
    decision=p.receive(true,magic,1,1,11,0);assert(decision.amount==1);p.save_started(11,true);
    decision=p.save_completed(11,true);assert(decision.action==5&&p.saved.ore==1&&p.tx.phase==3);
    assert(p.save_completed(11,true).notice==25);assert(p.ack_started(false).notice==23&&p.tx.ack_pending);
    decision=p.receive(true,magic,1,1,11,0);assert(decision.action==5&&decision.amount==0&&p.saved.ore==1);
    p.ack_started(true);assert(p.tx.phase==4);p.ack_sent(false);assert(!p.tx.phase&&p.tx.ack_pending);
    p.receive(true,magic,1,1,11,0);p.ack_started(true);p.ack_sent(true);
    unsigned polls=0;for(unsigned n=0;n<180&&p.tx.phase;++n){
        const auto before=p.tx.ack_time.raw();const auto phase=p.tx.phase;const auto result=p.advance_ack(Fixed(137,Fixed::RAW));
        assert((result.action==6)==(phase==5&&before/1365!=(before+137)/1365));
        if(result.action==6){++polls;p.receipt_checked(true,1,0,0);}
    }
    assert(polls>10&&!p.tx.phase&&p.tx.ack_pending&&p.saved.ore==1);
    RegisteredPolicy restarted;assert(restarted.load_result(p.saved,true,false).notice==28);
    assert(restarted.receive(true,magic,1,1,11,0).action==5&&restarted.saved.ore==1);
    restarted.ack_started(true);restarted.ack_sent(true);
    while(restarted.tx.phase==5)restarted.advance_ack(Fixed(137,Fixed::RAW));
    assert(restarted.receipt_checked(true,1,magic,11).notice==24&&!restarted.tx.ack_pending);
    assert(restarted.receive(true,magic,1,1,10,0).result==-3&&restarted.saved.ore==1);
    restarted.saved.ore=1000000;assert(restarted.receive(true,magic,1,1,12,0).result==-4);
    // Healing happens exactly once after a successful durable write.
    p.saved={};p.pending={};p.tx={};p.loaded=true;p.saved.medkits=1;p.saved.tanks=1;
    assert(p.activate(3,0,false).notice==8&&p.saved.medkits==1);
    assert(p.activate(3,0,true).action==4&&p.tx.heal==35&&p.saved.medkits==1);
    assert(!p.save_started(21,false).heal&&p.saved.medkits==1&&!p.tx.phase);
    p.activate(3,0,true);p.save_started(22,true);assert(p.leave().notice==26);
    assert(p.save_completed(22,true).heal==35&&p.saved.medkits==0&&!p.save_completed(22,true).heal);
    p.activate(3,1,true);p.save_started(23,true);assert(p.save_completed(23,false).heal==0&&p.saved.tanks==1);
    p.activate(3,1,true);assert(!p.leave().action&&!p.tx.phase&&p.pending.tanks==p.saved.tanks);
    auto corrupt=p.saved;corrupt.signature=0;assert(!p.valid(corrupt));const auto before=p.saved;
    assert(p.load_result(corrupt,true,false).notice==2&&!p.loaded&&!std::memcmp(&before,&p.saved,sizeof(before)));
    assert(p.load_result({},false,true).notice==29&&p.loaded&&p.valid(p.saved)&&p.saved.ore==0);
    // Editable prices/caps plus integer-overflow refusal.
    p.tuning.medkit_price=17;p.saved.credits=100;assert(p.activate(2,1,true).action==4&&p.pending.credits==83);p.leave();
    p.tuning.buster_cap=UINT32_MAX;p.tuning.buster_price=2;p.pending=p.saved;p.pending.buster=UINT32_MAX-1;
    assert(!p.buy(2));p.saved.buster=UINT32_MAX;assert(p.damage_bonus()==UINT32_MAX);
    p.pending=p.saved;p.tuning.sale_rate=0;assert(!p.sell());
    // The shared adapter copies exactly the old 92-byte schema and metadata;
    // it performs no transfer, purchase, validation or acknowledgement policy.
    p.stage_card_metadata();
    assert(!std::strcmp(gameplay_card_metadata().name,"EPOK-DUNGEON"));
    assert(!std::strcmp(gameplay_card_metadata().title,"Dungeon - Asteroid Supply"));
    p.stage_pending_payload();uint32_t expected_words[23];std::memcpy(expected_words,&p.pending,sizeof(expected_words));
    for(unsigned n=0;n<23;++n)assert(MemoryCardLibrary::staged_word(n)==expected_words[n]);
    assert(errors[1]&&errors[2]&&errors[3]&&errors[4]&&zero&&awards&&actions[4]);
    std::printf("PASS %u exact legacy validity/transfer/trade/price/activate/receive checkpoints; save-before-ack, interruption, duplicate, reinstall, full-ledger, heal-once, timeout, reload and overflow assertions; polls=%u\n",checks,polls);
}
