#pragma once
#include "DungeonEconomyPolicyBase.hpp"

// Project decisions, not a native service. Matching Lua and graph implementations
// contain these decisions independently; the base only copies authored data.
class EPOK_CLASS(Blueprintable,Id="d3ed89cb-587b-57cb-a136-8172feb4c774") DungeonEconomyPolicy:public DungeonEconomyPolicyBase {
    DungeonSupplyDecision out(uint32_t action=0,uint32_t notice=0,int32_t result=0,uint32_t amount=0,uint32_t heal=0,bool changed=false)const{
        return decision(action,notice,result,amount,heal,tx.serial,tx.total,changed);
    }
    void intent(uint32_t phase,uint32_t heal=0,bool ack=false,uint32_t serial=0,uint32_t total=0){tx={phase,0,serial,total,heal,ack,0.0};}
    bool durable(uint32_t serial,uint32_t total)const{
        for(uint32_t i=0;i<8;++i){const auto r=saved_receipt(i);if(r.serial==serial&&r.total>=total&&r.total)return true;}
        return false;
    }
public:
    bool valid(DungeonEconomySave value)const override{
        if(value.signature!=0x41535431u||value.version!=1||value.ore>tuning.balance_limit||value.credits>tuning.balance_limit||
            value.medkits>tuning.medkit_cap||value.tanks>tuning.tank_cap||value.buster>tuning.buster_cap)return false;
        const DungeonSupplyReceipt receipts[]={value.r0,value.r1,value.r2,value.r3,value.r4,value.r5,value.r6,value.r7};
        for(const auto r:receipts)if(r.total>tuning.balance_limit)return false;
        return true;
    }
    uint32_t price(DungeonEconomySave value,uint32_t item)const override{
        if(item==0)return tuning.medkit_price;if(item==1)return tuning.tank_price;
        if(value.buster==UINT32_MAX||tuning.buster_price>UINT32_MAX/(value.buster+1))return 0;
        return tuning.buster_price*(value.buster+1);
    }
    uint32_t damage_bonus()const override{
        if(tuning.buster_damage&&saved.buster>UINT32_MAX/tuning.buster_damage)return UINT32_MAX;
        return saved.buster*tuning.buster_damage;
    }
    uint32_t find_receipt(uint32_t serial)override{
        for(uint32_t i=0;i<8;++i){const auto r=pending_receipt(i);if(r.serial==serial&&r.total)return i;}
        for(uint32_t i=0;i<8;++i){const auto r=pending_receipt(i);if(!r.total){write_receipt(i,serial,0);return i;}}
        return 8;
    }
    DungeonSupplyTransfer transfer(uint32_t serial,uint32_t total,uint32_t claimed)override{
        if(total>tuning.balance_limit||claimed>total)return {-1,0};
        if(!valid(pending))return {-5,0};
        const auto index=find_receipt(serial);if(index>=8)return {-2,0};
        const auto previous_receipt=pending_receipt(index);
        const auto previous=previous_receipt.total>claimed?previous_receipt.total:claimed;
        if(total<previous)return {-3,0};
        const uint32_t amount=total-previous;if(amount>tuning.balance_limit-pending.ore)return {-4,0};
        pending.ore+=amount;write_receipt(index,serial,total);return {0,amount};
    }
    bool sell()override{
        if(!valid(pending)||!pending.ore||!tuning.sale_rate||pending.ore>(tuning.balance_limit-pending.credits)/tuning.sale_rate)return false;
        pending.credits+=pending.ore*tuning.sale_rate;pending.ore=0;return true;
    }
    bool buy(uint32_t item)override{
        if(item>2||!valid(pending))return false;
        auto& count=item==0?pending.medkits:item==1?pending.tanks:pending.buster;
        const auto cap=item==0?tuning.medkit_cap:item==1?tuning.tank_cap:tuning.buster_cap;
        if(count>=cap)return false;
        if(item==2&&(pending.buster==UINT32_MAX||tuning.buster_price>UINT32_MAX/(pending.buster+1)))return false;
        const uint32_t cost=price(pending,item);if(pending.credits<cost)return false;
        pending.credits-=cost;++count;return true;
    }
    DungeonSupplyDecision activate(uint32_t menu,uint32_t selection,bool can_heal)override{
        if(tx.phase)return out(0,26);
        if(menu==1){
            if(selection==1&&!loaded)return out(0,1);
            return out(selection==0?2u:selection==1?3u:1u);
        }
        if(!loaded)return out(selection==4||(menu==3&&selection==3)?1u:0u,selection==4||(menu==3&&selection==3)?0u:1u);
        if(!valid(saved))return out(0,2);
        pending=saved;intent(0);
        if(menu==2){
            if(selection==4)return out(1);
            if(!selection){if(!sell())return out(0,3);intent(1);return out(4,11,0,0,0,true);}
            if(!buy(selection-1))return out(0,selection==3&&saved.buster==tuning.buster_cap?5u:4u);
            intent(1);return out(4,selection==3?10u:9u,0,0,0,true);
        }
        if(menu==3){
            if(selection==3)return out(1);if(selection==2)return out(0,6);if(selection>3)return out(0,27);
            auto& count=selection?pending.tanks:pending.medkits;
            if(!count)return out(0,7);if(!can_heal)return out(0,8);
            --count;intent(1,selection?tuning.tank_heal:tuning.medkit_heal);
            return out(4,selection?13u:12u,0,0,0,true);
        }
        return out();
    }
    DungeonSupplyDecision receive(bool running,uint32_t signature,uint32_t version,uint32_t serial,uint32_t total,uint32_t claimed)override{
        if(tx.phase)return out(0,26);if(!loaded)return out(0,1);
        if(!running)return out(0,14);if(signature!=0x41535431u||version!=1)return out(0,15);
        pending=saved;const auto change=transfer(serial,total,claimed);
        if(change.result<0)return out(0,change.result==-2?16u:17u,change.result);
        if(!change.amount){
            if(claimed<total&&durable(serial,total)){intent(3,0,true,serial,total);return out(5,19);}
            return out(0,18);
        }
        intent(1,0,true,serial,total);return out(4,19,0,change.amount,0,true);
    }
    bool can_save()const override{return loaded&&tx.phase==1&&valid(pending);}
    DungeonSupplyDecision save_started(uint32_t request,bool accepted)override{
        if(tx.phase!=1)return out(0,25);
        if(!accepted||!can_save()){intent(0);return out(0,30);}
        tx.phase=2;tx.request=request;return out(0,20);
    }
    DungeonSupplyDecision save_completed(uint32_t request,bool succeeded)override{
        if(tx.phase!=2||request!=tx.request)return out(0,25);
        if(!succeeded){intent(0);return out(0,21);}
        saved=pending;const auto heal=tx.heal;tx.heal=0;
        if(tx.ack_pending&&durable(tx.serial,tx.total)){tx.phase=3;return out(5,22,0,0,heal,true);}
        tx.phase=0;return out(0,22,0,0,heal,true);
    }
    DungeonSupplyDecision ack_started(bool accepted)override{
        if(tx.phase!=3||!tx.ack_pending||!durable(tx.serial,tx.total))return out(0,25);
        if(!accepted){tx.phase=0;return out(0,23);}
        tx.phase=4;tx.ack_time=0.0;return out();
    }
    DungeonSupplyDecision ack_sent(bool succeeded)override{
        if(tx.phase!=4)return out(0,25);
        tx.phase=succeeded?5u:0u;return out(0,succeeded?0u:23u);
    }
    DungeonSupplyDecision advance_ack(epok::Fixed dt)override{
        if(dt<=epok::Fixed(0.0)||dt>epok::Fixed(1.0)||(tx.phase!=5&&tx.phase!=6))return out();
        const auto before=tx.ack_time;tx.ack_time+=dt;
        if(tx.phase==5&&before.raw()/1365!=tx.ack_time.raw()/1365){tx.phase=6;return out(6);}
        return out();
    }
    DungeonSupplyDecision receipt_checked(bool succeeded,uint32_t serial,uint32_t signature,uint32_t claimed)override{
        if(tx.phase!=6)return out(0,25);
        if(!succeeded||serial!=tx.serial){tx.phase=0;return out(0,23);}
        if(signature==0x41535431u&&claimed>=tx.total){tx.phase=0;tx.ack_pending=false;return out(0,24);}
        if(tx.ack_time>epok::Fixed(4.0)){tx.phase=0;return out(0,23);}
        tx.phase=5;return out();
    }
    DungeonSupplyDecision load_result(DungeonEconomySave candidate,bool succeeded,bool missing)override{
        if(tx.phase)return out(0,26);
        if(missing){saved=empty_save();pending=saved;loaded=true;intent(0);return out(0,29,0,0,0,true);}
        if(!succeeded){loaded=false;return out(0,21);}
        if(!valid(candidate)){loaded=false;return out(0,2);}
        saved=candidate;pending=saved;loaded=true;intent(0);return out(0,28,0,0,0,true);
    }
    DungeonSupplyDecision leave()override{
        if(tx.phase>1)return out(0,26);
        pending=saved;intent(0);return out();
    }
    DungeonSupplyDecision pocket_begin(uint32_t action,bool app_available,uint32_t port)override{
        if(pocket.operation)return out(0,26);
        if(port>1)return out(0,35);
        if(action!=2&&action!=3&&action!=5)return out(0,27);
        if((action==2||action==3)&&tx.phase)return out(0,26);
        if(action==3&&!loaded)return out(0,1);
        if(!app_available){if(action==5)ack_started(false);return out(0,action==5?23u:31u);}
        if(action==5){const auto gate=ack_started(true);if(tx.phase!=4)return gate;}
        ++pocket.epoch;if(!pocket.epoch)pocket.epoch=1;
        pocket.operation=action==2?4u:action==3?5u:6u;pocket.port=port;pocket.error=0;
        if(action==2)pocket.progress=101;
        return out(action==2?7u:action==3?8u:9u,action==2?37u:action==3?38u:39u);
    }
    DungeonSupplyDecision pocket_progress(uint32_t epoch,uint32_t percent)override{
        if(epoch!=pocket.epoch||pocket.operation!=4)return out(0,25);
        if(percent>100||percent==pocket.progress)return out();
        pocket.progress=percent;return out(0,32,0,percent);
    }
    DungeonSupplyDecision pocket_request_failed(uint32_t epoch,uint32_t error)override{
        if(epoch!=pocket.epoch||pocket.operation<4||pocket.operation>7)return out(0,25);
        const auto previous=pocket.operation;pocket.operation=0;pocket.error=error;
        if(previous==6){ack_sent(false);return out(0,23);}
        if(previous==7){receipt_checked(false,tx.serial,0,0);return out(0,23);}
        return out(0,36);
    }
    DungeonSupplyDecision pocket_observe(DungeonPocketObservation sample)override{
        if(sample.epoch!=pocket.epoch||pocket.operation<4||pocket.operation>7)return out(0,25);
        if(!sample.ready){if(pocket.operation==4)return pocket_progress(sample.epoch,sample.percent);return out();}
        if(sample.error)return pocket_request_failed(sample.epoch,sample.error);
        const auto previous=pocket.operation;pocket.operation=0;pocket.error=0;
        if(previous==4){++pocket.installed;return out(0,33,0,0,0,true);}
        if(previous==5)return receive(sample.running_block==sample.expected_block,sample.signature,sample.version,sample.serial,sample.total,sample.claimed);
        if(previous==6){const auto result=ack_sent(true);if(tx.phase==5)pocket.operation=3;return result;}
        const auto result=receipt_checked(true,sample.serial,sample.signature,sample.claimed);if(tx.phase==5)pocket.operation=3;return result;
    }
    DungeonSupplyDecision pocket_advance(epok::Fixed dt)override{
        if(pocket.operation!=3&&pocket.operation!=7)return out();
        const auto result=advance_ack(dt);
        if(result.action!=6)return result;
        ++pocket.epoch;if(!pocket.epoch)pocket.epoch=1;
        pocket.operation=7;return out(10);
    }
    void wire_fail(uint32_t error,uint32_t detail) {
        wire_set_word(14u, error);
        wire_set_word(15u, detail);
        wire_set_word(13u, 1u);
        wire_set_word(1u, 0u);
        return;
    }
    void wire_finish() {
        wire_fail(0u, 0u);
        return;
    }
    void wire_packet(uint32_t command) {
        wire_set_word(24u, 0u);
        for (uint32_t repeat_1=0; repeat_1<140u; ++repeat_1) {
            wire_set_byte((2448u + wire_word(24u)), 0u);
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_byte((2448u + 0u), 129u);
        wire_set_byte((2448u + 1u), command);
        return;
    }
    void wire_send(uint32_t next,uint32_t length,int32_t length_at) {
        if (!(wire_channel_exists())) {
            wire_fail(11u, 0u);
            return;
        }
        if (!(wire_channel()->clear_request())) {
            wire_fail(12u, 0u);
            return;
        }
        wire_set_word(24u, 0u);
        for (uint32_t repeat_2=0; repeat_2<140u; ++repeat_2) {
            if ((wire_word(24u) < length)) {
                if (!(wire_channel()->set_request_byte(wire_word(24u), wire_byte((2448u + wire_word(24u)))))) {
                    wire_fail(11u, 0u);
                    return;
                }
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_word(1u, next);
        wire_assign_token(wire_channel()->begin(length, length_at));
        if ((wire_token().request == 0u)) {
            wire_fail(wire_error_code(wire_channel()->last_rejection()), 0u);
        }
        return;
    }
    void wire_read(uint32_t next,uint32_t address) {
        wire_set_word(10u, address);
        wire_packet(82u);
        wire_set_byte((2448u + 4u), (address / 256u));
        wire_set_byte((2448u + 5u), (address % 256u));
        wire_send(next, 140u, -1);
        return;
    }
    void wire_status(uint32_t next) {
        wire_packet(90u);
        wire_send(next, 21u, -1);
        return;
    }
    void wire_values() {
        wire_packet(91u);
        wire_set_byte((2448u + 2u), 130u);
        wire_send(4u, 70u, 4u);
        return;
    }
    void wire_launch(uint32_t next) {
        wire_packet(89u);
        wire_set_byte((2448u + 4u), wire_word(9u));
        wire_send(next, 9u, -1);
        return;
    }
    void wire_wait_start() {
        wire_set_word(17u, (wire_word(17u) + 1u));
        if ((wire_word(17u) > 120u)) {
            wire_fail(5u, 40u);
            return;
        }
        wire_status(18u);
        return;
    }
    bool wire_valid_sector() {
        if (((((wire_word(23u) != 140u) || (wire_byte((0u + 139u)) != 71u)) || (wire_byte((0u + 8u)) != (wire_word(10u) / 256u))) || (wire_byte((0u + 9u)) != (wire_word(10u) % 256u)))) {
            wire_fail(5u, 0u);
            return false;
        }
        wire_set_word(25u, ((wire_word(10u) / 256u) ^ (wire_word(10u) % 256u)));
        wire_set_word(24u, 0u);
        for (uint32_t repeat_3=0; repeat_3<128u; ++repeat_3) {
            wire_set_word(25u, (wire_word(25u) ^ wire_byte((0u + (wire_word(24u) + 10u)))));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if ((wire_word(25u) != wire_byte((0u + 138u)))) {
            wire_fail(5u, 0u);
            return false;
        }
        return true;
    }
    bool wire_parse_status() {
        if (((wire_word(23u) != 21u) || (wire_byte((0u + 2u)) != 18u))) {
            wire_fail(2u, 0u);
            return false;
        }
        wire_set_word(28u, ((wire_byte((0u + 3u)) * 256u) + wire_byte((0u + 4u))));
        wire_set_word(29u, wire_u32(9u));
        return true;
    }
    void wire_capture_status() {
        wire_set_word(21u, wire_word(28u));
        wire_set_word(22u, wire_word(29u));
        return;
    }
    uint32_t wire_source(uint32_t at) {
        if ((((wire_word(11u) != 0u) && (wire_word(2u) == 0u)) && (wire_word(4u) == (wire_app_save_offset() / 128u)))) {
            return wire_byte((2304u + at));
        }
        return wire_app_image_byte((((wire_word(2u) * 8192u) + (wire_word(4u) * 128u)) + at));
    }
    void wire_write(uint32_t next,uint32_t address,uint32_t source) {
        wire_packet(87u);
        wire_set_byte((2448u + 4u), (address / 256u));
        wire_set_byte((2448u + 5u), (address % 256u));
        wire_set_word(25u, ((address / 256u) ^ (address % 256u)));
        wire_set_word(24u, 0u);
        for (uint32_t repeat_4=0; repeat_4<128u; ++repeat_4) {
            if ((source == 0u)) {
                wire_set_word(30u, wire_source(wire_word(24u)));
            } else {
                wire_set_word(30u, wire_byte((source + wire_word(24u))));
            }
            wire_set_byte((2448u + (wire_word(24u) + 6u)), wire_word(30u));
            wire_set_word(25u, (wire_word(25u) ^ wire_word(30u)));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_byte((2448u + 134u), wire_word(25u));
        wire_send(next, 138u, -1);
        return;
    }
    void wire_write_data() {
        wire_write(6u, ((wire_byte((2432u + wire_word(2u))) * 64u) + wire_word(4u)), 0u);
        return;
    }
    bool wire_named(uint32_t block) {
        wire_set_word(24u, 0u);
        for (uint32_t repeat_5=0; repeat_5<20u; ++repeat_5) {
            if ((wire_byte((((256u + (block * 128u)) + 10u) + wire_word(24u))) != wire_app_name_byte(wire_word(24u)))) {
                return false;
            }
            if ((wire_app_name_byte(wire_word(24u)) == 0u)) {
                return true;
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        return (wire_app_name_byte(20u) == 0u);
    }
    void wire_allocate() {
        wire_set_word(3u, (wire_app_size() / 8192u));
        wire_set_word(26u, 0u);
        wire_set_word(24u, 0u);
        for (uint32_t repeat_6=0; repeat_6<15u; ++repeat_6) {
            wire_set_word(31u, (wire_word(24u) + 1u));
            if (((wire_word(26u) < wire_word(3u)) && ((wire_byte((256u + (wire_word(31u) * 128u))) / 16u) == 10u))) {
                wire_set_byte((2432u + wire_word(26u)), wire_word(31u));
                wire_set_word(26u, (wire_word(26u) + 1u));
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if ((wire_word(26u) != wire_word(3u))) {
            if ((wire_word(8u) != 0u)) {
                wire_fail(8u, 41u);
                return;
            } else {
                wire_fail(8u, 0u);
                return;
            }
        }
        wire_set_word(9u, wire_byte(2432u));
        wire_set_word(2u, 0u);
        wire_set_word(4u, 0u);
        wire_write_data();
        return;
    }
    void wire_existing() {
        wire_set_word(4u, 0u);
        wire_set_word(18u, 2166136261u);
        wire_set_word(19u, 2166136261u);
        wire_set_word(20u, 2166136261u);
        wire_read(13u, (wire_word(8u) * 64u));
        return;
    }
    void wire_directory_ready() {
        wire_set_word(27u, 0u);
        wire_set_word(9u, 0u);
        wire_set_word(26u, 0u);
        wire_set_word(2u, 0u);
        for (uint32_t repeat_7=0; repeat_7<15u; ++repeat_7) {
            wire_set_word(31u, (wire_word(2u) + 1u));
            if ((wire_byte((256u + (wire_word(31u) * 128u))) == 81u)) {
                if (wire_named(wire_word(31u))) {
                    wire_set_word(27u, (wire_word(27u) + 1u));
                    wire_set_word(26u, wire_word(31u));
                }
            }
            wire_set_word(2u, (wire_word(2u) + 1u));
        }
        if ((wire_word(27u) > 1u)) {
            wire_fail(5u, 42u);
            return;
        }
        wire_set_word(9u, wire_word(26u));
        if ((wire_word(9u) != 0u)) {
            if ((wire_word(0u) != 0u)) {
                wire_status(3u);
                return;
            }
            if ((wire_app_save_offset() == 0u)) {
                wire_launch(8u);
                return;
            }
            if (((((wire_app_size() != 8192u) || (wire_u32(((256u + (wire_word(9u) * 128u)) + 4u)) != 8192u)) || (wire_byte(((256u + (wire_word(9u) * 128u)) + 8u)) != 255u)) || (wire_byte(((256u + (wire_word(9u) * 128u)) + 9u)) != 255u))) {
                wire_fail(5u, 43u);
                return;
            }
            wire_set_word(8u, wire_word(9u));
            wire_status(10u);
            return;
        }
        if ((wire_word(0u) != 0u)) {
            wire_fail(7u, 0u);
            return;
        }
        wire_allocate();
        return;
    }
    void wire_commit_frame(uint32_t item) {
        wire_set_word(31u, (256u + (wire_byte((2432u + item)) * 128u)));
        wire_set_word(24u, 0u);
        for (uint32_t repeat_8=0; repeat_8<128u; ++repeat_8) {
            wire_set_byte((wire_word(31u) + wire_word(24u)), 0u);
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if ((item == 0u)) {
            wire_set_byte(wire_word(31u), 81u);
            wire_set_u32((wire_word(31u) + 4u), wire_app_size());
            wire_set_word(24u, 0u);
            for (uint32_t repeat_9=0; repeat_9<20u; ++repeat_9) {
                wire_set_byte(((wire_word(31u) + 10u) + wire_word(24u)), wire_app_name_byte(wire_word(24u)));
                wire_set_word(24u, (wire_word(24u) + 1u));
            }
        } else {
            if (((item + 1u) == wire_word(3u))) {
                wire_set_byte(wire_word(31u), 83u);
            } else {
                wire_set_byte(wire_word(31u), 82u);
            }
        }
        if (((item + 1u) < wire_word(3u))) {
            wire_set_byte((wire_word(31u) + 8u), (wire_byte(((2432u + item) + 1u)) - 1u));
            wire_set_byte((wire_word(31u) + 9u), 0u);
        } else {
            wire_set_byte((wire_word(31u) + 8u), 255u);
            wire_set_byte((wire_word(31u) + 9u), 255u);
        }
        wire_set_word(25u, 0u);
        wire_set_word(24u, 0u);
        for (uint32_t repeat_10=0; repeat_10<127u; ++repeat_10) {
            wire_set_word(25u, (wire_word(25u) ^ wire_byte((wire_word(31u) + wire_word(24u)))));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_byte((wire_word(31u) + 127u), wire_word(25u));
        return;
    }
    void wire_commit() {
        wire_set_word(2u, 0u);
        for (uint32_t repeat_11=0; repeat_11<15u; ++repeat_11) {
            if ((wire_word(2u) < wire_word(3u))) {
                wire_commit_frame(wire_word(2u));
            }
            wire_set_word(2u, (wire_word(2u) + 1u));
        }
        wire_set_word(2u, (wire_word(3u) - 1u));
        wire_write(7u, wire_byte((2432u + wire_word(2u))), (256u + (wire_byte((2432u + wire_word(2u))) * 128u)));
        return;
    }
    void wire_retire() {
        wire_set_word(31u, (256u + (wire_word(8u) * 128u)));
        wire_set_byte(wire_word(31u), 161u);
        wire_set_word(25u, 0u);
        wire_set_word(24u, 0u);
        for (uint32_t repeat_12=0; repeat_12<127u; ++repeat_12) {
            wire_set_word(25u, (wire_word(25u) ^ wire_byte((wire_word(31u) + wire_word(24u)))));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_byte((wire_word(31u) + 127u), wire_word(25u));
        wire_write(16u, wire_word(8u), wire_word(31u));
        return;
    }
    bool protocol_start(uint32_t mode,uint32_t amount,uint32_t expected_serial) override {
        if (((wire_word(13u) == 0u) && (wire_word(1u) != 0u))) {
            return false;
        }
        if (!(wire_channel_exists())) {
            wire_fail(11u, 0u);
            return false;
        }
        wire_set_word(0u, 0u);
        wire_set_word(1u, 0u);
        wire_set_word(2u, 0u);
        wire_set_word(3u, 0u);
        wire_set_word(4u, 0u);
        wire_set_word(5u, 0u);
        wire_set_word(6u, 0u);
        wire_set_word(7u, 0u);
        wire_set_word(8u, 0u);
        wire_set_word(9u, 0u);
        wire_set_word(10u, 0u);
        wire_set_word(11u, 0u);
        wire_set_word(12u, 0u);
        wire_set_word(13u, 0u);
        wire_set_word(14u, 0u);
        wire_set_word(15u, 0u);
        wire_set_word(16u, 0u);
        wire_set_word(17u, 0u);
        wire_set_word(18u, 0u);
        wire_set_word(19u, 0u);
        wire_set_word(20u, 0u);
        wire_set_word(21u, 0u);
        wire_set_word(22u, 0u);
        wire_set_word(23u, 0u);
        wire_set_word(24u, 0u);
        wire_set_word(25u, 0u);
        wire_set_word(26u, 0u);
        wire_set_word(27u, 0u);
        wire_set_word(28u, 0u);
        wire_set_word(29u, 0u);
        wire_set_word(30u, 0u);
        wire_set_word(31u, 0u);
        wire_set_word(0u, mode);
        wire_set_word(7u, amount);
        wire_set_word(6u, expected_serial);
        if (((((((mode > 3u) || !(wire_app_available())) || (wire_app_size() == 0u)) || ((wire_app_size() % 8192u) != 0u)) || (wire_app_size() > 122880u)) || ((wire_app_save_offset() != 0u) && (((wire_app_save_offset() % 128u) != 0u) || (wire_app_save_offset() > (wire_app_size() - 128u)))))) {
            wire_fail(11u, 0u);
            return false;
        }
        if ((mode >= 2u)) {
            wire_status(3u);
        } else {
            wire_packet(88u);
            wire_send(1u, 5u, -1);
        }
        return (wire_word(13u) == 0u);
    }
    void protocol_cancel() override {
        if (wire_channel_exists()) {
            wire_channel()->cancel(wire_token());
        }
        wire_fail(11u, 44u);
        return;
    }
    uint32_t protocol_percent() override {
        if (((wire_word(13u) != 0u) && (wire_word(14u) == 0u))) {
            return 100u;
        }
        if ((wire_word(3u) == 0u)) {
            return 0u;
        }
        return ((wire_word(5u) * 99u) / (wire_word(3u) * 64u));
    }
    DungeonPocketObservation protocol_observation() override {
        return wire_make_observation((wire_word(13u) != 0u), wire_epoch(), wire_word(14u), wire_word(21u), wire_word(9u), wire_word(22u), wire_u32(2608u), wire_u32(2616u), wire_u32(2600u), wire_u32(2612u), protocol_percent());
    }
    void wire_phase_probe() {
        if (((wire_word(23u) != 5u) || (wire_byte((0u + 2u)) != 2u))) {
            wire_fail(2u, 0u);
            return;
        }
        wire_set_word(2u, 0u);
        wire_read(2u, 0u);
        return;
    }
    void wire_phase_directory() {
        if (!(wire_valid_sector())) {
            return;
        }
        wire_set_word(25u, 0u);
        wire_set_word(24u, 0u);
        for (uint32_t repeat_13=0; repeat_13<128u; ++repeat_13) {
            wire_set_byte(((256u + (wire_word(2u) * 128u)) + wire_word(24u)), wire_byte((0u + (wire_word(24u) + 10u))));
            if ((wire_word(24u) < 127u)) {
                wire_set_word(25u, (wire_word(25u) ^ wire_byte((0u + (wire_word(24u) + 10u)))));
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if ((wire_word(25u) != wire_byte(((256u + (wire_word(2u) * 128u)) + 127u)))) {
            wire_fail(5u, 0u);
            return;
        }
        if (((wire_word(2u) == 0u) && ((wire_byte(256u) != 77u) || (wire_byte(257u) != 67u)))) {
            wire_fail(6u, 0u);
            return;
        }
        wire_set_word(2u, (wire_word(2u) + 1u));
        if ((wire_word(2u) < 16u)) {
            wire_read(2u, wire_word(2u));
        } else {
            wire_directory_ready();
        }
        return;
    }
    void wire_phase_upgrade_status() {
        if (!(wire_parse_status())) {
            return;
        }
        wire_capture_status();
        if ((wire_word(21u) == wire_word(8u))) {
            wire_packet(89u);
            wire_set_byte((2448u + 5u), (48u + wire_word(8u)));
            wire_send(11u, 9u, -1);
        } else {
            wire_existing();
        }
        return;
    }
    void wire_phase_stop() {
        if (((wire_word(23u) != 9u) || (wire_byte((0u + 2u)) != 6u))) {
            wire_fail(5u, 0u);
            return;
        }
        wire_status(12u);
        return;
    }
    void wire_phase_stopped() {
        if (!(wire_parse_status())) {
            return;
        }
        if ((wire_word(29u) != wire_word(22u))) {
            wire_fail(5u, 45u);
            return;
        }
        if ((wire_word(28u) == wire_word(8u))) {
            wire_set_word(16u, (wire_word(16u) + 1u));
            if ((wire_word(16u) >= 120u)) {
                wire_fail(9u, 0u);
                return;
            }
            wire_status(12u);
        } else {
            wire_existing();
        }
        return;
    }
    void wire_phase_existing() {
        if (!(wire_valid_sector())) {
            return;
        }
        wire_set_word(24u, 0u);
        for (uint32_t repeat_14=0; repeat_14<128u; ++repeat_14) {
            if ((wire_word(4u) != 42u)) {
                wire_set_word(18u, ((wire_word(18u) ^ wire_byte((0u + (wire_word(24u) + 10u)))) * 16777619u));
            } else {
                wire_set_byte((2304u + wire_word(24u)), wire_byte((0u + (wire_word(24u) + 10u))));
            }
            if ((wire_word(4u) != (wire_app_save_offset() / 128u))) {
                wire_set_word(19u, ((wire_word(19u) ^ wire_byte((0u + (wire_word(24u) + 10u)))) * 16777619u));
                wire_set_word(20u, ((wire_word(20u) ^ wire_app_image_byte(((wire_word(4u) * 128u) + wire_word(24u)))) * 16777619u));
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_word(4u, (wire_word(4u) + 1u));
        if ((wire_word(4u) < 64u)) {
            wire_read(13u, ((wire_word(8u) * 64u) + wire_word(4u)));
            return;
        }
        if ((wire_word(19u) == wire_word(20u))) {
            wire_launch(8u);
            return;
        }
        if ((wire_word(18u) != 2567055923u)) {
            wire_fail(5u, 46u);
            return;
        }
        if ((((wire_u32(2304u) > 1000000u) || (wire_u32(2308u) > 1000000u)) || (wire_u32(2312u) > wire_u32(2308u)))) {
            wire_fail(5u, 47u);
            return;
        }
        wire_set_word(11u, 1u);
        wire_status(14u);
        return;
    }
    void wire_phase_identity() {
        if (!(wire_parse_status())) {
            return;
        }
        if ((wire_word(29u) != wire_word(22u))) {
            wire_fail(5u, 45u);
            return;
        }
        if ((wire_word(28u) == wire_word(8u))) {
            wire_fail(9u, 0u);
            return;
        }
        wire_allocate();
        return;
    }
    void wire_phase_verify() {
        if (!(wire_valid_sector())) {
            return;
        }
        wire_set_word(24u, 0u);
        for (uint32_t repeat_15=0; repeat_15<128u; ++repeat_15) {
            if ((wire_byte((0u + (wire_word(24u) + 10u))) != wire_source(wire_word(24u)))) {
                wire_fail(5u, 48u);
                return;
            }
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        wire_set_word(4u, (wire_word(4u) + 1u));
        if ((wire_word(4u) == 64u)) {
            wire_set_word(4u, 0u);
            wire_set_word(2u, (wire_word(2u) + 1u));
        }
        if ((wire_word(2u) < wire_word(3u))) {
            wire_read(15u, ((wire_byte((2432u + wire_word(2u))) * 64u) + wire_word(4u)));
        } else {
            wire_commit();
        }
        return;
    }
    void wire_phase_retire() {
        if (((wire_word(23u) != 138u) || (wire_byte((0u + 137u)) != 71u))) {
            wire_fail(4u, 0u);
            return;
        }
        wire_launch(8u);
        return;
    }
    void wire_phase_status() {
        if (!(wire_parse_status())) {
            return;
        }
        wire_capture_status();
        if (((wire_word(6u) != 0u) && (wire_word(22u) != wire_word(6u)))) {
            wire_fail(5u, 0u);
            return;
        }
        if (((wire_word(0u) == 1u) && (wire_word(21u) != wire_word(9u)))) {
            if ((wire_word(21u) != 0u)) {
                wire_fail(5u, 49u);
                return;
            }
            wire_set_word(12u, 1u);
            wire_launch(17u);
            return;
        }
        if ((wire_word(0u) == 2u)) {
            wire_packet(92u);
            wire_set_byte((2448u + 2u), 128u);
            wire_set_byte((2448u + 4u), 5u);
            wire_set_byte((2448u + 6u), 209u);
            wire_set_u32(2455u, wire_word(7u));
            wire_send(9u, 12u, -1);
        } else {
            wire_values();
        }
        return;
    }
    void wire_phase_start() {
        if (((wire_word(23u) != 9u) || (wire_byte((0u + 2u)) != 6u))) {
            wire_fail(5u, 0u);
            return;
        }
        wire_wait_start();
        return;
    }
    void wire_phase_await() {
        if (!(wire_parse_status())) {
            return;
        }
        if ((wire_word(29u) != wire_word(22u))) {
            wire_fail(5u, 50u);
            return;
        }
        if ((wire_word(28u) == wire_word(9u))) {
            wire_capture_status();
            wire_values();
        } else {
            if ((wire_word(28u) == 0u)) {
                wire_wait_start();
            } else {
                wire_fail(5u, 51u);
                return;
            }
        }
        return;
    }
    void wire_phase_shared() {
        if (((((wire_word(23u) != 70u) || (wire_byte((0u + 2u)) != 255u)) || (wire_byte((0u + 3u)) != 0u)) || (wire_byte((0u + 4u)) != 64u))) {
            if ((wire_word(12u) != 0u)) {
                wire_wait_start();
            } else {
                wire_fail(5u, 0u);
                return;
            }
            return;
        }
        wire_set_word(24u, 0u);
        for (uint32_t repeat_16=0; repeat_16<64u; ++repeat_16) {
            wire_set_byte((2600u + wire_word(24u)), wire_byte((0u + (wire_word(24u) + 5u))));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if (((wire_word(12u) != 0u) && ((wire_u32(2608u) != 1095980081u) || (wire_u32(2616u) != 1u)))) {
            wire_wait_start();
        } else {
            wire_status(5u);
        }
        return;
    }
    void wire_phase_confirm() {
        if (!(wire_parse_status())) {
            return;
        }
        if (((wire_word(29u) != wire_word(22u)) || (wire_word(28u) != wire_word(21u)))) {
            wire_fail(5u, 0u);
            return;
        }
        wire_finish();
        return;
    }
    void wire_phase_data() {
        if (((wire_word(23u) != 138u) || (wire_byte((0u + 137u)) != 71u))) {
            wire_fail(4u, 0u);
            return;
        }
        wire_set_word(5u, (wire_word(5u) + 1u));
        wire_set_word(4u, (wire_word(4u) + 1u));
        if ((wire_word(4u) == 64u)) {
            wire_set_word(4u, 0u);
            wire_set_word(2u, (wire_word(2u) + 1u));
        }
        if ((wire_word(2u) < wire_word(3u))) {
            wire_write_data();
        } else {
            wire_set_word(2u, 0u);
            wire_set_word(4u, 0u);
            wire_read(15u, (wire_byte(2432u) * 64u));
        }
        return;
    }
    void wire_phase_commit() {
        if (((wire_word(23u) != 138u) || (wire_byte((0u + 137u)) != 71u))) {
            wire_fail(4u, 0u);
            return;
        }
        if ((wire_word(2u) != 0u)) {
            wire_set_word(2u, (wire_word(2u) - 1u));
            wire_write(7u, wire_byte((2432u + wire_word(2u))), (256u + (wire_byte((2432u + wire_word(2u))) * 128u)));
            return;
        }
        if ((wire_word(11u) != 0u)) {
            wire_retire();
        } else {
            wire_launch(8u);
        }
        return;
    }
    void wire_phase_launch() {
        if (((wire_word(23u) != 9u) || (wire_byte((0u + 2u)) != 6u))) {
            wire_fail(5u, 0u);
            return;
        }
        wire_finish();
        return;
    }
    void wire_phase_send() {
        if (((((wire_word(23u) != 12u) || (wire_byte((0u + 2u)) != 255u)) || (wire_byte((0u + 3u)) != 1u)) || (wire_byte((0u + 5u)) != 5u))) {
            wire_fail(5u, 0u);
            return;
        }
        wire_finish();
        return;
    }
    bool protocol_tick(uint32_t microseconds) override {
        if (((wire_word(13u) != 0u) || (wire_word(1u) == 0u))) {
            return (wire_word(13u) != 0u);
        }
        if (!(wire_channel_exists())) {
            wire_fail(11u, 0u);
            return true;
        }
        wire_channel()->poll(wire_token(), microseconds);
        const auto sample = wire_channel()->snapshot(wire_token());
        if (!(sample.valid)) {
            wire_fail(11u, 0u);
            return true;
        }
        if (!(sample.ready)) {
            return false;
        }
        wire_set_word(14u, wire_error_code(sample.error));
        if ((wire_word(14u) != 0u)) {
            wire_fail(wire_word(14u), 0u);
            return true;
        }
        wire_set_word(23u, sample.received);
        wire_set_word(24u, 0u);
        for (uint32_t repeat_17=0; repeat_17<140u; ++repeat_17) {
            wire_set_byte(wire_word(24u), wire_channel()->reply_byte(wire_token(), wire_word(24u)));
            wire_set_word(24u, (wire_word(24u) + 1u));
        }
        if ((wire_word(1u) == 1u)) {
            wire_phase_probe();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 2u)) {
            wire_phase_directory();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 3u)) {
            wire_phase_status();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 4u)) {
            wire_phase_shared();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 5u)) {
            wire_phase_confirm();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 6u)) {
            wire_phase_data();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 7u)) {
            wire_phase_commit();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 8u)) {
            wire_phase_launch();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 9u)) {
            wire_phase_send();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 10u)) {
            wire_phase_upgrade_status();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 11u)) {
            wire_phase_stop();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 12u)) {
            wire_phase_stopped();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 13u)) {
            wire_phase_existing();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 14u)) {
            wire_phase_identity();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 15u)) {
            wire_phase_verify();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 16u)) {
            wire_phase_retire();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 17u)) {
            wire_phase_start();
            return (wire_word(13u) != 0u);
        }
        if ((wire_word(1u) == 18u)) {
            wire_phase_await();
            return (wire_word(13u) != 0u);
        }
        wire_fail(11u, 0u);
        return true;
    }
    DungeonSupplyDecision pocket_execute(uint32_t action,uint32_t port) override {
        const auto gate = pocket_begin(action, wire_app_available(), port);
        if ((((gate.action != 7u) && (gate.action != 8u)) && (gate.action != 9u))) {
            return gate;
        }
        wire_configure_channel(port, false);
        if ((action == 2u)) {
            protocol_start(0u, 0u, 0u);
        }
        if ((action == 3u)) {
            protocol_start(1u, 0u, 0u);
        }
        if ((action == 5u)) {
            protocol_start(2u, tx.total, tx.serial);
        }
        if ((wire_word(13u) != 0u)) {
            return pocket_request_failed(pocket.epoch, wire_word(14u));
        }
        return gate;
    }
    DungeonSupplyDecision pocket_update(epok::Fixed dt,uint32_t microseconds) override {
        if ((pocket.operation >= 4u)) {
            if ((pocket.operation == 7u)) {
                advance_ack(dt);
            }
            protocol_tick(microseconds);
            if ((((wire_word(13u) != 0u) && (wire_word(14u) != 0u)) && ((pocket.operation == 4u) || (pocket.operation == 5u)))) {
                const auto failed = pocket_request_failed(pocket.epoch, wire_word(14u));
                if ((wire_word(15u) != 0u)) {
                    return decision(0u, wire_word(15u), 0u, 0u, 0u, tx.serial, tx.total, false);
                }
                return failed;
            }
            return pocket_observe(protocol_observation());
        }
        if ((pocket.operation == 3u)) {
            const auto next = pocket_advance(dt);
            if ((next.action == 10u)) {
                wire_configure_channel(pocket.port, false);
                if (!(protocol_start(3u, 0u, tx.serial))) {
                    return pocket_request_failed(pocket.epoch, wire_word(14u));
                }
                return decision(0u, 0u, 0u, 0u, 0u, tx.serial, tx.total, false);
            }
            return next;
        }
        return decision(0u, 0u, 0u, 0u, 0u, tx.serial, tx.total, false);
    }
};
