"""One economy-provider circuit with the real23-word save and selected Pocket protocol.

The fixture owns a bounded simulated card transport and a short mailbox exchange;
it never writes user cards. Actual Pocket program/install acceptance remains in
verify_dungeon_complete_economy.py. Rules run in the selected authored language.
"""
import argparse
import sys
from dungeon_economy_ui_scenarios import install_ui_extension, stage_ui_assets, prepare_ui_project
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import uuid

from gameplay_probe import binary_reuse_matches, run_modes
from verify_camera_rigs import ROOT, EDITOR, documents, write

NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "DungeonEconomyConformance/" + name))
DRIVER = identity("driver")
POLICY = "d3ed89cb-587b-57cb-a136-8172feb4c774"
NATIVE = ROOT / "resources/templates/dungeon-complete/assets/scripts/DungeonCombat"
FLAVORS = ROOT / "resources/templates/dungeon-complete-flavors"
MAGIC, WORDS, POCKET_MASK = 0x45434F4E, 86, (1 << 16) - 1
PREFIX_ROWS = 181
PROTOCOL_BURST = 4
OPS = {name: n + 1 for n, name in enumerate(("reset", "load", "activate", "receive", "save_started",
    "save_completed", "ack_started", "ack_sent", "advance_ack", "receipt_checked", "leave", "transfer",
    "sell", "buy", "find_receipt", "edit_tuning", "card_start", "card_finish", "card_read", "card_mutate", "pocket",
    "protocol_setup", "protocol_step", "protocol_action"))}

# Test-card configuration, not a project protocol fallback. Every packet and
# transition is produced by the selected policy; this bounded card answers it.
CURRENT, LEGACY, RUNNING, OTHER, DUPLICATE, FULL, UNFORMATTED, BAD_DIRECTORY, BAD_SECTOR, BAD_WRITE, NOT_POCKET, REFUSE, UNKNOWN, CHANGE_CONFIRM, CHANGE_IDENTITY, RUN_OTHER, HOLD_START, NOT_READY, HOLD_STOP, INVALID_SAVE, UPGRADE_FULL = (1 << n for n in range(21))
PROTOCOL_CASES = [
    ("fresh-install",0,0,147,0,0), ("reinstall-current",0,CURRENT|RUNNING,85,0,0),
    ("inspect-running",1,CURRENT|RUNNING,20,0,0), ("inspect-starts-existing",1,CURRENT,22,0,0),
    ("inspect-refuses-other-app",1,CURRENT|OTHER,18,5,0), ("other-app-started-during-transfer",1,CURRENT|RUN_OTHER,20,5,0),
    ("start-timeout",1,CURRENT|HOLD_START,139,5,0), ("shared-init-timeout",1,CURRENT|NOT_READY,259,5,0),
    ("card-changed-confirm",1,CURRENT|RUNNING|CHANGE_CONFIRM,20,5,0), ("receipt-send",2,CURRENT|RUNNING,2,0,0),
    ("receipt-status",3,CURRENT|RUNNING,3,0,0), ("receipt-wrong-identity",2,CURRENT|RUNNING,1,5,0),
    ("inspect-missing",1,0,17,7,0), ("duplicate-heads",0,CURRENT|DUPLICATE,17,5,0),
    ("full-card",0,FULL,17,8,0), ("not-formatted",0,UNFORMATTED,2,6,0),
    ("directory-checksum",0,BAD_DIRECTORY,2,5,0), ("sector-checksum",0,BAD_SECTOR,2,5,0),
    ("verify-failed",0,BAD_WRITE,82,5,0), ("not-pocket",0,NOT_POCKET,1,2,0),
    ("disconnected-directory",0,0,8,1,8), ("launch-refused",0,REFUSE,147,5,0),
    ("unknown-upgrade-refused",0,CURRENT|UNKNOWN,82,5,0), ("legacy-upgrade-preserves-save",0,LEGACY,214,0,0),
    ("stop-then-migrate",0,LEGACY|RUNNING,216,0,0), ("invalid-legacy-save",0,LEGACY|INVALID_SAVE,82,5,0),
    ("changed-before-upgrade-allocation",0,LEGACY|CHANGE_IDENTITY,83,5,0), ("upgrade-full-keeps-old",0,LEGACY|UPGRADE_FULL,83,8,0),
    ("disconnect-before-new-head",0,LEGACY,212,1,212), ("disconnect-before-old-retire",0,LEGACY,213,1,213),
    ("changed-during-stop",0,CURRENT|UNKNOWN|RUNNING|CHANGE_IDENTITY,20,5,0), ("stop-timeout",0,CURRENT|UNKNOWN|RUNNING|HOLD_STOP,139,9,0),
]


def seeds():
    def save(ore=20, credits=900, medkits=2, tanks=1, buster=1, receipts=None):
        return [0x41535431, 1, ore, credits, medkits, tanks, buster] + sum(
            ([serial, total] for serial, total in (receipts or [(101, 10)] + [(0, 0)] * 7)), [])
    invalid = save(); invalid[0] = 0
    return [save(), save(receipts=[(n + 1, 10) for n in range(8)]), save(ore=1000000), invalid,
            save(ore=0, credits=0, medkits=0, tanks=0, buster=0),
            save(ore=1, credits=1000000, medkits=9, tanks=9, buster=3),
            save(receipts=[(0x80000001 + n, 101 + n) for n in range(8)])]


def commands():
    result = []
    def add(label, op, *args):
        assert len(args) <= 6
        result.append({"label": label, "op": op, "words": [OPS[op], *args] + [0] * (6 - len(args))})
    add("unloaded-card-error", "load", 0, 0, 0)
    for menu, selection in ((1, 0), (1, 1), (1, 2), (2, 0), (2, 4), (3, 3)):
        add(f"unloaded-menu-{menu}-{selection}", "activate", menu, selection, 1)
    add("missing-save-creates-empty-record", "load", 0, 0, 1)
    add("invalid-record-rejected", "load", 3, 1, 0)
    add("all-23-words-roundtrip", "load", 6, 1, 0)
    add("restore-wallet", "reset", 0)
    for menu, selection, heal in ((3, 0, 0), (3, 2, 1), (3, 7, 1), (2, 4, 1)):
        add(f"menu-rejection-{menu}-{selection}-{heal}", "activate", menu, selection, heal)
    add("prepare-medkit", "activate", 3, 0, 1)
    add("no-ack-before-durable-save", "ack_started", 1)
    add("save-rejected-preserves-inventory", "save_started", 5, 0)
    add("prepare-medkit-again", "activate", 3, 0, 1)
    add("save-accepted", "save_started", 6, 1)
    add("leave-refused-during-save", "leave")
    add("stale-save-completion", "save_completed", 5, 1)
    add("failed-save-preserves-inventory", "save_completed", 6, 0)
    add("prepare-medkit-third", "activate", 3, 0, 1)
    add("save-accepted-third", "save_started", 7, 1)
    add("heal-after-durable-save", "save_completed", 7, 1)
    add("duplicate-completion-never-reheals", "save_completed", 7, 1)
    for item in range(3):
        add(f"shop-prepare-item-{item}", "activate", 2, item + 1, 1)
        add(f"shop-save-start-{item}", "save_started", 20 + item, 1)
        add(f"shop-save-complete-{item}", "save_completed", 20 + item, 1)
    add("shop-sell-ore", "activate", 2, 0, 1)
    add("cancel-uncommitted-sale", "leave")
    for seed, label, serial, total, claimed in ((0, "bad-total", 1, 1000001, 0),
        (0, "claimed-exceeds-total", 1, 10, 11), (1, "full-ledger", 9, 11, 0),
        (0, "older-card", 101, 9, 0), (2, "balance-overflow", 101, 11, 0),
        (3, "invalid-pending", 101, 11, 0), (0, "duplicate", 101, 10, 10),
        (0, "known-device-growth", 101, 12, 0), (0, "new-device", 202, 15, 0),
        (6, "high-bit-serial", 0x80000008, 110, 0)):
        add("reset-" + label, "reset", seed)
        add("transfer-" + label, "transfer", serial, total, claimed)
    for seed in (0, 4, 5, 3):
        add(f"reset-sell-{seed}", "reset", seed); add(f"sell-{seed}", "sell")
        for item in range(4):
            add(f"reset-buy-{seed}-{item}", "reset", seed)
            add(f"buy-{seed}-{item}", "buy", item)
    add("find-empty-receipt", "reset", 0); add("find-receipt", "find_receipt", 777)
    add("find-full-ledger", "reset", 1); add("find-no-slot", "find_receipt", 777)
    add("restore-for-receive", "reset", 0)
    for label, args in (("program-not-running", (0, 0x41535431, 1, 101, 12, 0)),
                        ("wrong-signature", (1, 0, 1, 101, 12, 0)),
                        ("wrong-version", (1, 0x41535431, 2, 101, 12, 0))):
        add(label, "receive", *args)
    add("prepare-reward", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("duplicate-receive-busy", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("native-card-start", "card_start", 0)
    add("accepted-request-owns-metadata-and-payload", "card_mutate")
    add("native-card-durable-complete", "card_finish")
    add("ack-rejected-keeps-durable-receipt", "ack_started", 0)
    add("duplicate-reward-only-reacks", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("ack-accepted", "ack_started", 1); add("ack-send-failed", "ack_sent", 0)
    add("retry-receive", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("ack-accepted-retry", "ack_started", 1); add("ack-send-succeeded", "ack_sent", 1)
    add("zero-ack-delta", "advance_ack", 0); add("negative-ack-delta", "advance_ack", 0xffffffff)
    add("oversize-ack-delta", "advance_ack", 4097)
    add("ack-poll-ready", "advance_ack", 1365)
    add("wrong-receipt-serial", "receipt_checked", 1, 102, 0x41535431, 12)
    add("retry-after-wrong-device", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("ack-start-final", "ack_started", 1); add("ack-send-final", "ack_sent", 1)
    add("ack-poll-final", "advance_ack", 1365)
    add("durable-ack-confirmed", "receipt_checked", 1, 101, 0x41535431, 12)
    add("same-claimed-reward-does-not-reaward", "receive", 1, 0x41535431, 1, 101, 12, 12)
    add("reload-native-23word-payload", "card_read")
    for failure, label in ((1, "interrupted"), (2, "disconnected"), (3, "corrupt-inactive-copy")):
        add(label + "-prepare", "receive", 1, 0x41535431, 1, 101, 13, 12)
        add(label + "-start", "card_start", failure)
        add(label + "-complete", "card_finish")
        add(label + "-reload-old-copy", "card_read")
    add("timeout-reward", "receive", 1, 0x41535431, 1, 101, 12, 0)
    add("timeout-ack-start", "ack_started", 1); add("timeout-ack-sent", "ack_sent", 1)
    for n in range(13):
        add(f"ack-timeout-advance-{n}", "advance_ack", 1365)
        add(f"ack-timeout-poll-{n}", "receipt_checked", 1, 101, 0, 0)
    add("after-timeout-reload", "card_read")
    add("editable-tuning", "edit_tuning")
    add("edited-wallet", "reset", 0)
    add("edited-shop-purchase", "activate", 2, 1, 1)
    add("edited-price-save-start", "save_started", 900, 1)
    add("edited-price-save-complete", "save_completed", 900, 1)
    add("edited-heal", "activate", 3, 0, 1)
    add("edited-heal-save-start", "save_started", 901, 1)
    add("edited-heal-save-complete", "save_completed", 901, 1)
    add("native-pocket-owned-handle-contract", "pocket")
    assert len(result) == PREFIX_ROWS
    for case, (label, _, _, packets, _, _) in enumerate(PROTOCOL_CASES):
        add("protocol-" + label + "-setup", "protocol_setup", case)
        for step in range((packets + PROTOCOL_BURST - 1) // PROTOCOL_BURST):
            add(f"protocol-{label}-step-{step}", "protocol_step")
    # Same circuit, now with the chosen implementation driving real packets
    # across the durable-save and reconnect/duplicate boundary.
    add("protocol-flow-inspect", "protocol_setup", 2)
    for step in range(5): add(f"protocol-flow-inspect-{step}", "protocol_step")
    add("protocol-flow-native-save-start", "card_start", 0)
    add("protocol-flow-native-save-complete", "card_finish")
    add("protocol-flow-send-durable-receipt", "protocol_action", 5)
    for step in range(6): add(f"protocol-flow-receipt-{step}", "protocol_step")
    add("protocol-flow-duplicate-inspect", "protocol_action", 3)
    for step in range(5): add(f"protocol-flow-duplicate-{step}", "protocol_step")
    assert {entry["op"] for entry in result} == set(OPS)
    return result


STEPS = len(commands())


@lru_cache(maxsize=1)
def legacy_tail():
    image = bytes(((n * 13 + n // 7) ^ 0xa5) & 255 for n in range(8192))
    value = 2166136261
    for n, byte in enumerate(image[:8187]):
        if n // 128 != 42: value = ((value ^ byte) * 16777619) & 0xffffffff
    inverse = pow(16777619, -1, 1 << 32)
    for extra in range(256):
        h = ((value ^ extra) * 16777619) & 0xffffffff
        prefixes = {((((h ^ a) * 16777619) & 0xffffffff) ^ b) * 16777619 & 0xffffffff: (a, b)
                    for a in range(256) for b in range(256)}
        for d in range(256):
            for c in range(256):
                before = ((((0x99022a33 * inverse) & 0xffffffff) ^ d) * inverse & 0xffffffff) ^ c
                if before in prefixes: return [extra, *prefixes[before], c, d]
    raise AssertionError("No bounded deterministic legacy fingerprint fixture")


def protocol_source():
    rows = ",\n".join("{" + ",".join(str(n) + "u" for n in (mode, flags, packets, error, disconnect)) + "}"
                        for _, mode, flags, packets, error, disconnect in PROTOCOL_CASES)
    constants = ",".join(name + "=" + str(globals()[name]) + "u" for name in (
        "CURRENT", "LEGACY", "RUNNING", "OTHER", "DUPLICATE", "FULL", "UNFORMATTED", "BAD_DIRECTORY", "BAD_SECTOR",
        "BAD_WRITE", "NOT_POCKET", "REFUSE", "UNKNOWN", "CHANGE_CONFIRM", "CHANGE_IDENTITY", "RUN_OTHER", "HOLD_START",
        "NOT_READY", "HOLD_STOP", "INVALID_SAVE", "UPGRADE_FULL"))
    return r'''
namespace economy_protocol_probe {
using namespace epok;
enum Flags:uint32_t {FLAG_CONSTANTS};
inline constexpr uint32_t producer=0x51af0273u;
inline const uint32_t cases[][5]={PROTOCOL_ROWS};
inline uint8_t image[8192]{};
inline PocketApp app{};
inline uint32_t scenario=0,state_hash=2166136261u,model_errors=0;
inline void mix(uint32_t& hash,uint32_t value){hash=(hash^value)*16777619u;}
struct Card {
    uint8_t directory[16][128]{},program[2][8192]{};
    uint32_t flags=0,disconnect=0,id=producer,running=0,total=320,claimed=5,signature=0x41535431u,version=1;
    uint32_t packets=0,packet_hash=2166136261u,statuses=0,writes=0;
    static void put(uint8_t* out,uint32_t value){for(uint32_t k=0;k<4;++k)out[k]=uint8_t(value>>(8*k));}
    void checksum(uint32_t b){directory[b][127]=0;for(uint32_t i=0;i<127;++i)directory[b][127]^=directory[b][i];}
    uint8_t byte(uint32_t address,uint32_t at)const{
        if(address<16)return directory[address][at];
        if(address>=64&&address<192)return program[(address/64)-1][(address%64)*128+at];return 0;
    }
    void set(uint32_t address,uint32_t at,uint8_t value){
        if(address<16)directory[address][at]=value;
        else if(address>=64&&address<192)program[(address/64)-1][(address%64)*128+at]=value;
        else model_errors|=1;
    }
    void place(uint32_t b){
        directory[b][0]=0x51;put(directory[b]+4,8192);directory[b][8]=directory[b][9]=255;
        const char* name="BEEPOKPASTEROID";for(uint32_t i=0;name[i];++i)directory[b][10+i]=uint8_t(name[i]);checksum(b);
        for(uint32_t i=0;i<8192;++i)program[b-1][i]=image[i];
    }
    void reset(uint32_t options,uint32_t at){
        *this=Card{};flags=options;disconnect=at;directory[0][0]='M';directory[0][1]='C';checksum(0);
        for(uint32_t b=1;b<16;++b){directory[b][0]=0xa0;checksum(b);}
        if(flags&(CURRENT|LEGACY))place(1);
        if(flags&DUPLICATE)place(2);
        if(flags&LEGACY){const uint8_t tail[5]={LEGACY_TAIL};for(uint32_t i=0;i<5;++i)program[0][8187+i]=tail[i];
            put(program[0]+42*128,91);put(program[0]+42*128+4,320);put(program[0]+42*128+8,5);}
        if(flags&UNKNOWN)program[0][3]^=1;
        if(flags&INVALID_SAVE)put(program[0]+42*128,1000001);
        if(flags&FULL)for(uint32_t b=1;b<16;++b){directory[b][0]=0x51;checksum(b);}
        if(flags&UPGRADE_FULL)for(uint32_t b=2;b<16;++b){directory[b][0]=0x51;checksum(b);}
        if(flags&UNFORMATTED){directory[0][0]=0;checksum(0);}
        if(flags&BAD_DIRECTORY)directory[0][127]^=1;
        running=flags&RUNNING?1:flags&OTHER?2:0;
    }
    void answer(){
        auto& box=epok_pocket_mailbox;if(box.request==box.answered)return;
        const uint32_t n=box.lengths[0];if(!n||n>140){model_errors|=2;return;}
        uint8_t tx[140]{},reply[140]{};for(uint32_t i=0;i<n;++i)tx[i]=box.send[i];
        ++packets;mix(packet_hash,n);for(uint32_t i=0;i<n;++i)mix(packet_hash,tx[i]);
        reply[0]=255;uint32_t received=n;
        if(disconnect&&packets==disconnect)received=0;
        else switch(tx[1]){
        case 0x58:reply[2]=flags&NOT_POCKET?0:2;break;
        case 0x52:{uint32_t address=uint32_t(tx[4])*256+tx[5];uint8_t sum=tx[4]^tx[5];
            reply[8]=tx[4];reply[9]=tx[5];for(uint32_t i=0;i<128;++i){reply[10+i]=byte(address,i);sum^=reply[10+i];}
            reply[138]=sum;reply[139]=flags&BAD_SECTOR?0:0x47;break;}
        case 0x57:{uint32_t address=uint32_t(tx[4])*256+tx[5];uint8_t sum=tx[4]^tx[5];
            for(uint32_t i=0;i<128;++i)sum^=tx[6+i];if(sum!=tx[134])model_errors|=4;
            if(!(flags&BAD_WRITE))for(uint32_t i=0;i<128;++i)set(address,i,tx[6+i]);++writes;reply[137]=0x47;break;}
        case 0x5a:++statuses;if(((flags&CHANGE_CONFIRM)||(flags&CHANGE_IDENTITY))&&statuses==2)++id;
            reply[2]=0x12;reply[3]=uint8_t(running>>8);reply[4]=uint8_t(running);put(reply+9,id);break;
        case 0x59:reply[2]=flags&REFUSE?0:6;if(!(flags&REFUSE)){
            if(tx[5]>=0x30){if(!(flags&HOLD_STOP))running=0;}
            else if(!(flags&HOLD_START))running=flags&RUN_OTHER?9:tx[4];}break;
        case 0x5b:reply[2]=flags&NOT_READY?0:255;reply[4]=64;put(reply+5,total);put(reply+13,signature);put(reply+17,claimed);put(reply+21,version);break;
        case 0x5c:if(tx[2]!=0x80||tx[4]!=5||tx[6]!=0xd1)model_errors|=8;
            claimed=uint32_t(tx[7])|uint32_t(tx[8])<<8|uint32_t(tx[9])<<16|uint32_t(tx[10])<<24;
            reply[2]=255;reply[3]=1;reply[5]=5;break;
        default:model_errors|=16;break;
        }
        for(uint32_t i=0;i<140;++i)box.reply[i]=reply[i];box.received[0]=uint16_t(received);box.answered=box.request;
    }
    uint32_t program_hash()const{uint32_t h=2166136261u;for(uint32_t b=0;b<2;++b)for(uint32_t i=0;i<8192;++i)mix(h,program[b][i]);return h;}
};
inline Card card;
inline void fold(const DungeonEconomyPolicyBase& p,const DungeonSupplyDecision& d){
    for(uint32_t n=0;n<32;++n)mix(state_hash,p.wire_word(n));
    const uint32_t values[]={p.pocket.operation,p.pocket.epoch,p.pocket.installed,p.pocket.error,p.pocket.progress,
        p.tx.phase,p.tx.serial,p.tx.total,uint32_t(p.tx.ack_pending),uint32_t(p.tx.ack_time.raw()),p.saved.ore,p.pending.ore,
        d.action,d.notice,uint32_t(d.result),d.amount,d.heal,d.serial,d.total,uint32_t(d.changed)};
    for(auto value:values)mix(state_hash,value);
}
inline DungeonSupplyDecision setup(DungeonEconomyPolicyBase& p,PocketTransactionComponent& io,uint32_t selected){
    p.protocol_cancel();p.assign_pocket({0,0,0,0,0,101});p.assign_saved({});p.assign_pending(p.saved);p.assign_tx({});p.assign_loaded(true);
    p.pocket_channel=io.id();scenario=selected;state_hash=2166136261u;model_errors=0;
    for(uint32_t i=0;i<8192;++i)image[i]=uint8_t((i*13+i/7)^0xa5);
    app.filename="BEEPOKPASTEROID";app.image=image;app.size=8192;app.save_offset=40*128;p.wire_assign_image(app);
    const auto* c=cases[selected];card.reset(c[1],c[4]);auto& box=epok_pocket_mailbox;box.magic=PocketMailbox::magic_value;box.answered=box.request;
    DungeonSupplyDecision decision{};
    if(c[0]>=2){p.saved.ore=315;p.saved.r0={producer,320};p.assign_pending(p.saved);p.assign_tx(p.make_tx(c[0]==2?3:6,0,producer+(selected==11?1:0),320,0,true,0.0));
        if(selected==11){p.saved.r0.serial=producer+1;p.assign_pending(p.saved);}
        if(c[0]==3){p.assign_pocket({7,0,1,0,0,101});p.protocol_start(3,0,producer);}
        else decision=p.pocket_execute(5,0);
    }else decision=p.pocket_execute(c[0]==0?2:3,0);
    fold(p,decision);return decision;
}
inline DungeonSupplyDecision step(DungeonEconomyPolicyBase& p){
    DungeonSupplyDecision d{};for(uint32_t n=0;n<PROTOCOL_BURST;++n){
        if(!p.pocket.operation)break;card.answer();d=p.pocket_update(Fixed(137,Fixed::RAW),16667);fold(p,d);
    }return d;
}
}
'''.replace("FLAG_CONSTANTS", constants).replace("PROTOCOL_ROWS", rows).replace("LEGACY_TAIL", ",".join(str(n) for n in legacy_tail())).replace("PROTOCOL_BURST", str(PROTOCOL_BURST))


def source():
    seed_rows = ",\n".join("    {" + ",".join(str(n) + "u" for n in row) + "}" for row in seeds())
    rows = ",\n".join("    {" + ",".join(str(n) + "u" for n in entry["words"]) + "}" for entry in commands())
    return r'''#pragma once
#include "DungeonEconomyPolicyBase.hpp"
#include "device_io.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
namespace economy_probe {
using namespace epok;
inline const uint32_t inputs[][7]={COMMAND_ROWS};
inline const uint32_t seed_words[][23]={SEED_ROWS};
inline DungeonEconomySave seed(uint32_t n){DungeonEconomySave value;auto* bytes=reinterpret_cast<uint8_t*>(&value);
    for(uint32_t i=0;i<23;++i)for(uint32_t k=0;k<4;++k)bytes[4*i+k]=uint8_t(seed_words[n][i]>>(8*k));return value;}
inline uint32_t hash_bytes(const char* value){uint32_t hash=2166136261u;for(uint32_t n=0;value[n];++n)hash=(hash^uint8_t(value[n]))*16777619u;return hash;}
// Process-lifetime adapter, fixed two-record journal, no user card or hardware.
class Card:public MemoryCardDriver {
    uint32_t operation_=0,capacity_=0,size_=0;void* output_=nullptr;const uint8_t* data_=nullptr;
    uint32_t* transferred_=nullptr;void* owner_=nullptr;Completion complete_=nullptr;
    char name_[21]{},title_[129]{};uint8_t records_[2][112]{};uint32_t lengths_[2]{};
    static void text(char* to,const char* from,uint32_t cap){uint32_t i=0;for(;i+1<cap&&from[i];++i)to[i]=from[i];to[i]=0;}
    void queue(uint32_t op,unsigned port,const char* name,void* owner,Completion complete){operation_=op;port_seen=port;text(name_,name,21);owner_=owner;complete_=complete;}
public:
    uint32_t failure=0,writes=0,port_seen=99,name_hash=0,title_hash=0;
    bool idle()const override{return !operation_;}
    void probe(unsigned port,void* owner,Completion complete)override{queue(1,port,"",owner,complete);}
    void read(unsigned port,const char* name,void* output,uint32_t capacity,uint32_t* transferred,void* owner,Completion complete)override{
        output_=output;capacity_=capacity;transferred_=transferred;queue(2,port,name,owner,complete);}
    void write(unsigned port,const char* name,const char* title,const CardIcon&,const void* data,uint32_t size,void* owner,Completion complete)override{
        data_=static_cast<const uint8_t*>(data);size_=size;text(title_,title,129);queue(3,port,name,owner,complete);}
    void list(unsigned port,CardFile*,uint32_t* count,void* owner,Completion complete)override{*count=0;queue(1,port,"",owner,complete);}
    bool pump(){
        if(!operation_)return false;const auto operation=operation_;operation_=0;
        auto complete=complete_;auto owner=owner_;CardError error=CardError::OK;
        uint32_t length=0;while(name_[length])++length;const uint32_t slot=length&&name_[length-1]=='B'?1:0;
        if(failure==2)error=CardError::NoCard;
        else if(operation==2){
            if(!lengths_[slot])error=CardError::FileNotFound;
            else{*transferred_=lengths_[slot]<capacity_?lengths_[slot]:capacity_;auto* bytes=static_cast<uint8_t*>(output_);
                for(uint32_t n=0;n<*transferred_;++n)bytes[n]=records_[slot][n];}
        }else if(operation==3){
            ++writes;name_hash=hash_bytes(name_);title_hash=hash_bytes(title_);
            lengths_[slot]=failure==1?size_/2:size_;
            if(lengths_[slot]>112){lengths_[slot]=0;error=CardError::FileTooLarge;}
            else{for(uint32_t n=0;n<lengths_[slot];++n)records_[slot][n]=data_[n];
                if(failure==3&&lengths_[slot])records_[slot][lengths_[slot]-1]^=128;
                if(failure==1)error=CardError::Timeout;}
        }
        complete(owner,error);return true;
    }
    void drain(){for(uint32_t n=0;n<8&&pump();++n){}}
};
inline Card card;
inline uint32_t pocket_contract(PocketTransactionComponent& a,PocketTransactionComponent& b){
    auto& box=epok_pocket_mailbox;const auto saved_magic=box.magic,saved_request=box.request,saved_answered=box.answered;
    box.magic=PocketMailbox::magic_value;box.request=box.answered=0;uint32_t mask=0;
    auto check=[&](uint32_t bit,bool ok){if(ok)mask|=1u<<bit;};
    auto answer=[&](uint32_t size){box.received[0]=uint16_t(size);box.reply[0]=0xff;box.reply[2]=2;box.answered=box.request;};
    check(0,a.clear_request()&&a.set_request_byte(0,0x81)&&a.set_request_byte(1,0x58));
    check(1,!a.set_request_byte(140,1)&&!a.set_request_byte(0,256));
    check(2,!a.begin(0,-1).request&&!a.begin(141,-1).request&&!a.begin(5,-2).request&&!a.begin(5,5).request);
    a.port=2;check(3,!a.begin(5,-1).request&&a.last_rejection()==PocketError::BadPort);a.port=1;
    const auto first=a.begin(5,-1);const auto busy=a.snapshot(first);
    check(4,first.request&&first.component==a.id()&&busy.valid&&busy.busy&&!busy.ready);
    check(5,!b.snapshot(first).valid&&!b.poll(first,1)&&!b.cancel(first));
    check(6,!a.clear_request()&&!a.set_request_byte(1,0)&&!a.begin(5,-1).request&&a.last_rejection()==PocketError::Busy);
    answer(5);a.poll(first,1);const auto done=a.snapshot(first);
    check(7,done.valid&&done.ready&&!done.busy&&done.received==5&&done.error==PocketError::OK&&a.reply_byte(first,2)==2&&a.reply_byte(first,5)==0);
    const auto next=a.begin(5,-1);check(8,next.request!=first.request&&!a.snapshot(first).valid);
    check(9,a.cancel(next)&&!a.snapshot(next).valid&&!b.begin(5,-1).request&&b.last_rejection()==PocketError::Refused);
    answer(5);const auto theirs=b.begin(5,-1);answer(5);check(10,theirs.request&&b.poll(theirs,1)&&b.snapshot(theirs).ready);
    const auto timeout=a.begin(5,-1);check(11,timeout.request&&a.poll(timeout,0xffffffffu)&&a.snapshot(timeout).error==PocketError::Timeout);answer(5);
    const auto disabled=a.begin(5,-1);auto* owner=a.get_owner();owner->set_active(false);
    check(12,disabled.request&&!a.snapshot(disabled).valid);answer(5);owner->set_active(true);
    const auto enabled=a.begin(5,-1);answer(1);a.poll(enabled,1);
    check(13,enabled.request!=disabled.request&&a.snapshot(enabled).error==PocketError::NoCard);
    const auto ended=a.begin(5,-1);a.end_play(EndPlayReason::LevelUnloaded);answer(5);
    check(14,ended.request&&!a.snapshot(ended).valid);a.begin_play();
    check(15,!b.snapshot({a.id(),theirs.request}).valid&&!a.poll({},1)&&!a.cancel({}));
    box.request=saved_request;box.answered=saved_answered;box.magic=saved_magic;return mask;
}
}
PROTOCOL_SOURCE
extern "C" {inline uint32_t dungeon_economy_policy_probe[8]={};inline uint32_t dungeon_economy_policy_trace[STEP_COUNT][86]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") DungeonEconomyPolicyProbe:public epok::Actor3D {
    uint32_t frame_=0,hash_=2166136261u,fail_=0,pocket_=0;
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEconomyPolicyBase") epok::ObjectId policy{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::PocketTransactionComponent") epok::ObjectId first{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::PocketTransactionComponent") epok::ObjectId second{};
    void on_frame(uint32_t)override{
        using namespace epok;using namespace economy_probe;if(frame_>=STEP_COUNT)return;
        auto* registry=active_object_registry;auto* p=registry?registry->resolve<DungeonEconomyPolicyBase>(policy):nullptr;
        auto* a=registry?registry->resolve<PocketTransactionComponent>(first):nullptr;
        auto* b=registry?registry->resolve<PocketTransactionComponent>(second):nullptr;if(!p||!a||!b)return;
        if(!frame_){if(!memory_card.attach(card)||sizeof(DungeonEconomySave)!=92)fail_|=1;}
        const auto* q=inputs[frame_];DungeonSupplyDecision d{};DungeonSupplyTransfer transfer{};
        switch(q[0]){
        case 1:p->assign_saved(seed(q[1]));p->assign_pending(p->saved);p->assign_tx({});p->assign_loaded(true);break;
        case 2:d=p->load_result(seed(q[1]),q[2]!=0,q[3]!=0);break;
        case 3:d=p->activate(q[1],q[2],q[3]!=0);break;
        case 4:d=p->receive(q[1]!=0,q[2],q[3],q[4],q[5],q[6]);break;
        case 5:d=p->save_started(q[1],q[2]!=0);break;
        case 6:d=p->save_completed(q[1],q[2]!=0);break;
        case 7:d=p->ack_started(q[1]!=0);break;
        case 8:d=p->ack_sent(q[1]!=0);break;
        case 9:d=p->advance_ack(Fixed(int32_t(q[1]),Fixed::RAW));break;
        case 10:d=p->receipt_checked(q[1]!=0,q[2],q[3],q[4]);break;
        case 11:d=p->leave();break;
        case 12:transfer=p->transfer(q[1],q[2],q[3]);break;
        case 13:transfer.amount=p->sell();break;
        case 14:transfer.amount=p->buy(q[1]);break;
        case 15:transfer.amount=p->find_receipt(q[1]);break;
        case 16:p->tuning={2000000,12,12,5,7,17,63,91,44,123,11};break;
        case 17:{card.failure=q[1];p->stage_card_metadata();p->stage_pending_payload();
            const bool accepted=MemoryCardLibrary::write_staged_named(92,1);d=p->save_started(MemoryCardLibrary::snapshot().request,accepted);break;}
        case 18:{card.drain();const auto status=MemoryCardLibrary::snapshot();d=p->save_completed(status.completed,status.state==CardState::Succeeded);card.failure=0;break;}
        case 19:{p->stage_card_metadata();const bool accepted=MemoryCardLibrary::read_staged_name(1);card.drain();const auto status=MemoryCardLibrary::snapshot();
            d=p->load_result(p->loaded_payload(),accepted&&status.state==CardState::Succeeded,status.error==CardError::FileNotFound);break;}
        case 20:if(MemoryCardLibrary::set_staged_name_byte(18,'X')||MemoryCardLibrary::set_staged_name_byte(0,256)||
                MemoryCardLibrary::set_staged_title_byte(128,'X')||MemoryCardLibrary::set_staged_title_byte(0,256))fail_|=16;
            MemoryCardLibrary::clear_staged_metadata();MemoryCardLibrary::set_staged_name_byte(0,'X');
            MemoryCardLibrary::set_staged_title_byte(0,'X');MemoryCardLibrary::clear_staged_payload();
            transfer.amount=MemoryCardLibrary::read_staged_name(0);if(transfer.amount||MemoryCardLibrary::snapshot().last_rejection!=CardError::Busy)fail_|=2;break;
        case 21:pocket_=pocket_contract(*a,*b);if(pocket_!=65535)fail_|=4;break;
        case 22:d=economy_protocol_probe::setup(*p,*a,q[1]);break;
        case 23:d=economy_protocol_probe::step(*p);if(economy_protocol_probe::model_errors)fail_|=32;break;
        case 24:d=p->pocket_execute(q[1],0);economy_protocol_probe::fold(*p,d);break;
        }
        const auto status=MemoryCardLibrary::snapshot();uint32_t payload_hash=2166136261u;
        for(uint32_t n=0;n<23;++n)payload_hash=(payload_hash^MemoryCardLibrary::loaded_word(n))*16777619u;
        uint32_t row[86]{},at=0;auto put=[&](uint32_t value){row[at++]=value;};
        put(frame_);put(q[0]);put(d.action);put(d.notice);put(uint32_t(d.result));put(d.amount);put(d.heal);put(d.serial);put(d.total);put(d.changed);
        for(uint32_t which=0;which<2;++which){const auto* bytes=reinterpret_cast<const uint8_t*>(which?&p->pending:&p->saved);
            for(uint32_t n=0;n<23;++n){uint32_t value=0;for(uint32_t k=0;k<4;++k)value|=uint32_t(bytes[n*4+k])<<(8*k);put(value);}}
        put(p->tx.phase);put(p->tx.request);put(p->tx.serial);put(p->tx.total);put(p->tx.heal);put(p->tx.ack_pending);put(uint32_t(p->tx.ack_time.raw()));put(p->loaded);
        put(p->valid(p->saved));put(p->can_save());put(p->price(p->saved,0));put(p->price(p->saved,1));put(p->price(p->saved,2));put(p->damage_bonus());
        put(uint32_t(transfer.result));put(transfer.amount);put(uint32_t(status.state));put(uint32_t(status.operation));put(uint32_t(status.error));put(uint32_t(status.last_rejection));
        put(status.request);put(status.completed);put(status.payload_bytes);put(status.file_count);put(card.name_hash);put(card.title_hash);put(card.port_seen);put(card.writes);put(payload_hash);put(pocket_);
        if(q[0]>=22){using namespace economy_protocol_probe;
            const uint32_t tail[]={p->wire_word(1),p->wire_word(14),p->wire_word(13),p->protocol_percent(),p->pocket.operation,p->pocket.epoch,
                p->wire_word(9),p->wire_word(21),p->wire_word(22),economy_protocol_probe::card.packets,economy_protocol_probe::card.packet_hash,
                state_hash,uint32_t(economy_protocol_probe::card.directory[1][0])|uint32_t(economy_protocol_probe::card.directory[2][0])<<8,economy_protocol_probe::card.claimed};
            for(uint32_t n=0;n<14;++n)row[72+n]=tail[n];
            row[70]=economy_protocol_probe::card.program_hash();row[71]=model_errors;}
        if(at!=86)fail_|=8;for(uint32_t n=0;n<86;++n){dungeon_economy_policy_trace[frame_][n]=row[n];hash_=(hash_^row[n])*16777619u;}
        if(++frame_==STEP_COUNT){const uint32_t summary[]={0x45434F4Eu+EPOK_LUA_MODE,STEP_COUNT,fail_,STEP_COUNT,hash_,card.writes,pocket_,23};
            for(uint32_t n=0;n<8;++n)dungeon_economy_policy_probe[n]=summary[n];}
    }
};
'''.replace("COMMAND_ROWS", rows).replace("SEED_ROWS", seed_rows).replace("STEP_COUNT", str(STEPS)).replace("DRIVER_ID", DRIVER).replace("PROTOCOL_SOURCE", protocol_source())


def common_files(project):
    files = list((ROOT / "runtime").glob("*.hpp")) + list((ROOT / "runtime").glob("*.cpp"))
    files += list((ROOT / "src").glob("*.rs"))
    files += [NATIVE / name for name in ("DungeonEconomyPolicy.hpp", "DungeonEconomyPolicyBase.hpp", "DungeonInventory.hpp", "DungeonRulesBinding.hpp")]
    files += [project / "assets/scripts/DungeonEconomyPolicyProbe.hpp"]
    return sorted(set(files))


def common_contract(project):
    digest = hashlib.sha256()
    for path in common_files(project):
        digest.update(str(path.relative_to(ROOT) if path.is_relative_to(ROOT) else path).encode("utf-8"))
        digest.update(path.read_bytes())
    return digest.hexdigest()


def assert_scenario_contracts(trace):
    """Scenario requirements independent of the selected/native implementation."""
    assert len(trace) == STEPS * WORDS
    rows = {entry["label"]: trace[n * WORDS:(n + 1) * WORDS] for n, entry in enumerate(commands())}
    def row(label): return rows[label]
    assert row("all-23-words-roundtrip")[10:33] == seeds()[6]
    assert row("all-23-words-roundtrip")[33:56] == seeds()[6]
    assert row("prepare-medkit")[14] == 2 and row("prepare-medkit")[37] == 1
    assert row("no-ack-before-durable-save")[3] == 25
    assert row("save-rejected-preserves-inventory")[14] == 2
    assert row("failed-save-preserves-inventory")[14] == 2
    assert row("stale-save-completion")[3] == 25
    assert row("heal-after-durable-save")[6] == 35 and row("heal-after-durable-save")[14] == 1
    assert row("duplicate-completion-never-reheals")[6] == 0
    assert row("prepare-reward")[5] == 2 and row("prepare-reward")[12] == 20
    assert row("prepare-reward")[35] == 22 and row("prepare-reward")[56] == 1
    assert row("native-card-start")[56] == 2 and row("native-card-start")[72] == 1
    assert row("accepted-request-owns-metadata-and-payload")[75] == 128
    assert row("native-card-durable-complete")[2] == 5 and row("native-card-durable-complete")[56] == 3
    assert row("native-card-durable-complete")[12] == 22 and row("native-card-durable-complete")[78] == 92
    def text_hash(value):
        result = 2166136261
        for byte in value.encode("ascii"): result = ((result ^ byte) * 16777619) & 0xffffffff
        return result
    assert row("native-card-durable-complete")[80:83] == [text_hash("EPOK-DUNGEON-A"), text_hash("Dungeon - Asteroid Supply"), 1]
    expected = seeds()[0].copy(); expected[2] = 22; expected[8] = 12
    assert row("reload-native-23word-payload")[10:33] == expected
    assert row("reload-native-23word-payload")[33:56] == expected
    assert row("duplicate-reward-only-reacks")[5] == 0 and row("duplicate-reward-only-reacks")[12] == 22
    assert row("durable-ack-confirmed")[3] == 24 and row("durable-ack-confirmed")[61] == 0
    for label, error in (("interrupted", 5), ("disconnected", 1), ("corrupt-inactive-copy", 131)):
        assert row(label + "-complete")[74] == error and row(label + "-complete")[12] == 22
        assert row(label + "-reload-old-copy")[10:33] == expected
    assert row("ack-timeout-poll-12")[3] == 23 and row("ack-timeout-poll-12")[61] == 1
    assert row("edited-price-save-complete")[13:15] == [883, 3]
    assert row("edited-heal-save-complete")[6] == 44
    assert row("edited-heal-save-complete")[66:70] == [17, 63, 182, 11]
    assert row("native-pocket-owned-handle-contract")[85] == POCKET_MASK
    errors = {row("transfer-" + label)[70] for label in
              ("bad-total", "full-ledger", "older-card", "balance-overflow", "invalid-pending")}
    assert errors == {(-n) & 0xffffffff for n in range(1, 6)}
    for label, _, _, packets, error, _ in PROTOCOL_CASES:
        final = row(f"protocol-{label}-step-{(packets + PROTOCOL_BURST - 1) // PROTOCOL_BURST - 1}")
        assert final[71] == 0 and final[73:75] == [error, 1], (label, final[71:86])
        assert final[81] == packets and final[82] != 0 and final[83] != 0, (label, final[71:86])
    def end(label):
        packets = next(case[3] for case in PROTOCOL_CASES if case[0] == label)
        return row(f"protocol-{label}-step-{(packets + PROTOCOL_BURST - 1) // PROTOCOL_BURST - 1}")
    assert end("fresh-install")[78] == 1 and end("fresh-install")[84] == 0xa051
    assert end("legacy-upgrade-preserves-save")[78] == 2 and end("legacy-upgrade-preserves-save")[84] == 0x51a1
    assert end("disconnect-before-new-head")[84] == 0xa051
    assert end("disconnect-before-old-retire")[84] == 0x5151
    for label in ("inspect-running", "inspect-starts-existing"):
        assert end(label)[2] == 4 and end(label)[12] == 0 and end(label)[35] == 315
    assert end("receipt-send")[85] == 320 and end("receipt-send")[12] == 315
    image = bytearray(((n * 13 + n // 7) ^ 0xa5) & 255 for n in range(8192))
    legacy = image.copy(); legacy[-5:] = bytes(legacy_tail())
    for offset, value in ((0, 91), (4, 320), (8, 5)): legacy[42 * 128 + offset:42 * 128 + offset + 4] = value.to_bytes(4, "little")
    upgraded = image.copy(); upgraded[40 * 128:41 * 128] = legacy[42 * 128:43 * 128]
    def byte_hash(data):
        value = 2166136261
        for byte in data: value = ((value ^ byte) * 16777619) & 0xffffffff
        return value
    assert end("fresh-install")[70] == byte_hash(image + bytes(8192))
    assert end("legacy-upgrade-preserves-save")[70] == byte_hash(legacy + upgraded)
    assert row("protocol-flow-inspect-4")[2] == 4 and row("protocol-flow-inspect-4")[12] == 0
    assert row("protocol-flow-native-save-complete")[2] == 5 and row("protocol-flow-native-save-complete")[12] == 315
    assert row("protocol-flow-receipt-5")[85] == 320 and row("protocol-flow-receipt-5")[61] == 0
    assert row("protocol-flow-duplicate-4")[3] == 18 and row("protocol-flow-duplicate-4")[12] == 315


def retained_prefix(trace):
    """Old captures prove this unchanged behavioral prefix, not new code/IO."""
    archive = ROOT / "artifacts/gameplay-helpers/p9/economy/protocol-prefix-181"
    provenance = json.loads((archive / "provenance.json").read_text(encoding="utf-8"))
    receipts = {}
    for relative, digest in provenance["files"].items():
        if not relative.endswith("-dungeon_economy_policy_trace.json"): continue
        path = archive / relative
        assert hashlib.sha256(path.read_bytes()).hexdigest() == digest
        prior = json.loads(path.read_text(encoding="utf-8"))
        assert len(prior) == PREFIX_ROWS * WORDS and trace[:PREFIX_ROWS * WORDS] == prior, relative
        receipts[relative] = digest
    assert len(receipts) == 5, "Retain all five earlier approved provider/mode captures"
    return {"scope": "Historical181-row prefix equals current trace; does not certify added protocol or current executable", "rows": PREFIX_ROWS, "captures": receipts}


def host_oracle(base_out, allow_build=True):
    out = base_out / "host-oracle"
    probe = source()
    inputs = {str(p.relative_to(ROOT)).replace("\\", "/"): hashlib.sha256(p.read_bytes()).hexdigest()
              for p in sorted(list((ROOT / "runtime").glob("*.hpp")) + [NATIVE / "DungeonEconomyPolicy.hpp",
                  NATIVE / "DungeonEconomyPolicyBase.hpp", NATIVE / "DungeonRulesBinding.hpp", NATIVE / "DungeonInventory.hpp",
                  ROOT / "tests/runtime/actor_scene_fixture.hpp", ROOT / "tests/fixtures/dungeon-legacy/DungeonInventory.hpp"])}
    signature = hashlib.sha256(json.dumps({"probe": probe, "inputs": inputs,
        "fixture_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}, sort_keys=True).encode()).hexdigest()
    receipt_path, trace_path = out / "receipt.json", out / "trace.json"
    if receipt_path.exists() and trace_path.exists():
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt["signature"] == signature and receipt["trace_sha256"] == hashlib.sha256(trace_path.read_bytes()).hexdigest():
            result = json.loads(trace_path.read_text(encoding="utf-8")); assert_scenario_contracts(result); return result
    assert allow_build, "Prepare current host oracle before checking retained captures"
    out.mkdir(parents=True, exist_ok=True)
    write(out / "DungeonEconomyPolicyProbe.hpp", probe)
    host = r'''#define EPOK_WORLD_GAMEPLAY 1
#include "DungeonEconomyPolicy.hpp"
struct HostPolicy:public DungeonEconomyPolicy {
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("d3ed89cb-587b-57cb-a136-8172feb4c774");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
};
#define EPOK_TEST_EXTRA_CLASSES \
 {DungeonEconomyPolicyBase::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassAbstract}, \
 {HostPolicy::static_class_id,DungeonEconomyPolicyBase::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,0,&object_construct<HostPolicy>,&object_destruct,sizeof(HostPolicy),alignof(HostPolicy),&ObjectPool<HostPolicy,1>::acquire,&ObjectPool<HostPolicy,1>::release}, \
 {PocketTransactionComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,0,&object_construct<PocketTransactionComponent>,&object_destruct,sizeof(PocketTransactionComponent),alignof(PocketTransactionComponent),&ObjectPool<PocketTransactionComponent,2>::acquire,&ObjectPool<PocketTransactionComponent,2>::release},
#include "actor_scene_fixture.hpp"
#include "DungeonEconomyPolicyProbe.hpp"
namespace frozen {
#include "DungeonLegacyInventory.hpp"
}
#include <cstdio>
#include <cstring>
int main(){
    using namespace epok;test_reset_scene();
    auto* policy=test_level.add_component<HostPolicy>(*test_registry.resolve<Actor3D>(test_owner(0)),"Policy");assert(policy);
    auto* first=test_level.add_component<PocketTransactionComponent>(*test_registry.resolve<Actor3D>(test_owner(1)),"First");assert(first);
    auto* second=test_level.add_component<PocketTransactionComponent>(*test_registry.resolve<Actor3D>(test_owner(2)),"Second");assert(second);
    DungeonEconomyPolicyProbe probe;probe.policy=policy->id();probe.first=first->id();probe.second=second->id();
    uint32_t checks=0;bool edited=false;
    for(uint32_t frame=0;frame<STEP_COUNT;++frame){
        const auto* q=economy_probe::inputs[frame];frozen::dungeon_inventory::Save old{};std::memcpy(&old,&policy->pending,sizeof(old));
        const bool independent=!edited&&policy->valid(policy->pending)&&(q[0]==12||q[0]==13||q[0]==14);
        int expected=0;if(independent){
            expected=q[0]==12?frozen::dungeon_inventory::transfer(old,q[1],q[2],q[3]):q[0]==13?int(frozen::dungeon_inventory::sell(old)):int(frozen::dungeon_inventory::buy(old,q[1]));}
        probe.on_frame(0);if(q[0]==16)edited=true;
        if(independent){const auto* row=dungeon_economy_policy_trace[frame];assert(!std::memcmp(&old,&policy->pending,sizeof(old)));
            if(q[0]==12){assert(int32_t(row[70])==(expected<0?expected:0));assert(row[71]==uint32_t(expected>0?expected:0));}
            else assert(row[71]==uint32_t(expected));++checks;}
    }
    assert(dungeon_economy_policy_probe[0]==0x45434F4E&&dungeon_economy_policy_probe[2]==0);
    assert(dungeon_economy_policy_probe[6]==65535&&checks>=20);
    std::fprintf(stderr,"PASS %u curated frozen-legacy inventory checkpoints; %u scenario rows; 16 native Pocket contracts\n",checks,STEP_COUNT);
    std::printf("[");for(uint32_t frame=0;frame<STEP_COUNT;++frame)for(uint32_t n=0;n<86;++n)
        std::printf("%s%u",(frame||n)?",":"",dungeon_economy_policy_trace[frame][n]);std::puts("]");
}
'''.replace("STEP_COUNT", str(STEPS))
    cpp_path = out / "EconomyOracle.cpp"; write(cpp_path, host)
    shutil.copyfile(ROOT / "tests/fixtures/dungeon-legacy/DungeonInventory.hpp", out / "DungeonLegacyInventory.hpp")
    runner = ROOT / "tests/runtime/verify_spatial.py"
    code = runner.read_text(encoding="utf-8")
    selection = next(marker for marker in ("arguments = _sys.argv[1:]", "if len(_sys.argv)") if marker in code)
    code = code[:code.index(selection)] + "\nSOURCES=[_Path(" + repr(str(cpp_path)) + ")]\ntest_arguments=[]\n" + code[code.index("env = os.environ.copy()") :]
    code = code.replace("    for source in SOURCES:", "    includes.extend([ROOT / 'tests/runtime', _Path(" + repr(str(NATIVE)) + ")])\n    for source in SOURCES:")
    code = code.replace("        subprocess.run(command, cwd=temp, env=env, check=True)",
        "        subprocess.run(command, cwd=temp, env=env, check=True, stdout=log, stderr=subprocess.STDOUT)")
    code, replacements = re.subn(r"        subprocess\.run\(\[str\(target\)(?:, \*test_arguments)?\], cwd=temp, check=True, timeout=30\)",
        "        subprocess.run([str(target)], cwd=temp, check=True, timeout=30, stdout=trace, stderr=log)", code)
    assert replacements == 1
    namespace = {"__file__": str(runner), "__name__": "__main__"}
    with (out / "compile.log").open("w", encoding="utf-8") as log, trace_path.open("w", encoding="utf-8") as trace:
        namespace.update(log=log, trace=trace); exec(code, namespace)
    result = json.loads(trace_path.read_text(encoding="utf-8")); assert len(result) == STEPS * WORDS
    assert all(isinstance(value, int) and 0 <= value <= 0xffffffff for value in result)
    assert_scenario_contracts(result)
    prefix = retained_prefix(result)
    receipt_path.write_text(json.dumps({"signature": signature, "trace_sha256": hashlib.sha256(trace_path.read_bytes()).hexdigest(),
        "scope": "Current native host scenario reference, with curated frozen-legacy inventory checks; no independent oracle claim for new transaction methods",
        "inputs": inputs, "compiler": namespace["compiler"], "rows": STEPS, "words": len(result),
        "native_pocket_contracts": 16, "selected_protocol_scenarios": len(PROTOCOL_CASES), "retained_prefix": prefix,
        "protocol_oracle": "Current native scenario reference plus separately SHA-reused independent old-protocol2375-packet proof; packet and intermediate-state hashes include all selected-language updates",
        "user_card_io": False}, indent=2) + "\n", encoding="utf-8")
    write(base_out / "scenario-inventory.json", {"words_per_row": WORDS, "save_words": 23, "cases": commands(),
        "protocol_burst_per_driver_frame": PROTOCOL_BURST, "protocol_cases": PROTOCOL_CASES,
        "coverage": "All selected economy and real packet protocol methods, durable/rejected/stale saves, duplicate receipt/ACK, timeout, tuning, journal failures, direct typed owned IO, install/current/legacy migration/interrupt/readback/card identity"})
    return result


def validate_captures(base_out, out, flavor, selected, fingerprint):
    reference = host_oracle(base_out, allow_build=False)
    for mode in selected:
        actual = json.loads((out / (mode + "-dungeon_economy_policy_trace.json")).read_text(encoding="utf-8"))
        assert len(actual) == STEPS * WORDS
        assert_scenario_contracts(actual)
        retained_prefix(actual)
        observation = json.loads((out / (mode + "-observation.json")).read_text(encoding="utf-8"))
        expected = MAGIC + {"native_cpp": 0, "vm_bytecode": 1, "vm_source": 2}[mode]
        values = observation["values"]
        assert values[:4] == [expected, STEPS, 0, STEPS] and values[5] > 0 and values[6:] == [POCKET_MASK, 23], values
        digest = 2166136261
        for value in actual: digest = ((digest ^ value) * 16777619) & 0xffffffff
        assert digest == values[4]
        if actual != reference:
            index = next(n for n, (a, b) in enumerate(zip(actual, reference)) if a != b)
            raise AssertionError((flavor, mode, commands()[index // WORDS]["label"], index % WORDS, actual[index], reference[index]))
    write(out / "conformance.json", {"provider": flavor, "modes": selected, "rows": STEPS, "words_per_case": STEPS * WORDS,
        "native_policy_fallback": False, "common_contract_sha256": fingerprint,
        "host_reference_sha256": hashlib.sha256((base_out / "host-oracle/trace.json").read_bytes()).hexdigest(),
        "retained_prefix_rows": PREFIX_ROWS, "selected_protocol_scenarios": len(PROTOCOL_CASES),
        "transport": "Bounded fixture-owned journal and Pocket card; chosen C++/BP/Lua drives actual generic IO methods, packet bytes and intermediate state hashes; actual Pocket application/hardware route separately required"})
    print("PASS unified economy", flavor, selected, STEPS * WORDS, "words", flush=True)


def verify(base_out, flavor, modes=None, prepare_only=False, check_only=False):
    out = base_out / flavor; out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    selected = modes or (["native_cpp", "vm_bytecode", "vm_source"] if flavor == "lua" else ["native_cpp"])
    if check_only:
        inventory = documents.loads((out / "source-inventory.json").read_text(encoding="utf-8"))
        fingerprint = common_contract(project)
        assert inventory["common_contract_sha256"] == fingerprint or binary_reuse_matches(out, inventory["common_contract_sha256"], fingerprint)
        for relative, expected in inventory["files"].items(): assert hashlib.sha256((project / relative).read_bytes()).hexdigest() == expected, relative
        authored = NATIVE / "DungeonEconomyPolicy.hpp" if flavor == "cpp" else FLAVORS / flavor / ("DungeonEconomyPolicy.lua" if flavor == "lua" else "DungeonEconomyPolicy.epokbp")
        assert inventory["authored_policy_sha256"] == hashlib.sha256(authored.read_bytes()).hexdigest(), "Current selected-language rules have changed"
        validate_captures(base_out, out, flavor, selected, fingerprint); return
    def run(*args, label):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
        (out / label).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()
    host_oracle(base_out)
    if not project.exists(): run("--create-project", project, "--template", "basic", "--name", "Economy policies " + flavor, label="create.log")
    scripts = project / "assets/scripts"
    for name in ("DungeonRulesBinding.hpp", "DungeonInventory.hpp", "DungeonEconomyPolicyBase.hpp"):
        shutil.copyfile(NATIVE / name, scripts / name)
    if flavor == "cpp": shutil.copyfile(NATIVE / "DungeonEconomyPolicy.hpp", scripts / "DungeonEconomyPolicy.hpp")
    elif flavor == "blueprint": write(project / "assets/Blueprints/DungeonEconomyPolicy.epokbp", (FLAVORS / "blueprint/DungeonEconomyPolicy.epokbp").read_text(encoding="utf-8"))
    else:
        write(scripts / "DungeonEconomyPolicy.lua", (FLAVORS / "lua/DungeonEconomyPolicy.lua").read_text(encoding="utf-8"))
        write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/DungeonEconomyPolicy.lua": POLICY}})
    assert flavor == "cpp" or not (scripts / "DungeonEconomyPolicy.hpp").exists(), "Native policy leak"
    stage_ui_assets(sys.modules[__name__],project,flavor)
    write(scripts / "DungeonEconomyPolicyProbe.hpp", source())
    metadata = json.loads(run("--project", project, "--reflect", label="reflection.json"))
    classes = {entry["cpp_name"]: entry for entry in metadata["classes"]}
    def component(name, key, root=False):
        value = {"id": identity(key), "name": key, "class": {"name": name, "class_id": classes[name]["id"]}, "properties": {}, "overrides": []}
        if root: value["root"] = True
        return value
    def actor(key):
        return {"id": identity(key), "name": key, "class": {"name": "epok::Actor3D", "class_id": classes["epok::Actor3D"]["id"]},
                "properties": {}, "overrides": [], "components": [component("epok::SceneComponent3D", key + "/root", True)]}
    driver, first, second = actor("driver-actor"), actor("first-device"), actor("second-device")
    driver["class"] = {"name": "DungeonEconomyPolicyProbe", "class_id": DRIVER}
    driver["properties"] = {"policy": identity("policy"), "first": identity("first"), "second": identity("second")}
    driver["overrides"] = list(driver["properties"])
    driver["components"].append({"id": identity("policy"), "name": "Economy rules", "class": {"name": "DungeonEconomyPolicy", "class_id": POLICY}, "properties": {"pocket_channel": identity("first")}, "overrides": ["pocket_channel"]})
    first["components"].append(component("epok::PocketTransactionComponent", "first"))
    second["components"].append(component("epok::PocketTransactionComponent", "second"))
    descriptor = next(project.glob("*.epokproject")); manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]; scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    prepare_ui_project(sys.modules[__name__],project,flavor,scene,driver,first,second)
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    inventory = {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
        for folder in (scripts, project / "assets/Blueprints", project / "assets/Classes", project / "assets/Localization", project / "assets/UI") if folder.exists() for p in folder.rglob("*") if p.is_file()}
    fingerprint = common_contract(project)
    write(out / "source-inventory.json", {"provider": flavor, "native_policy_fallback": False, "files": inventory,
        "common_contract_sha256": fingerprint, "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "trace_words": WORDS, "steps": STEPS,
        "authored_policy_sha256": hashlib.sha256((NATIVE / "DungeonEconomyPolicy.hpp" if flavor == "cpp" else FLAVORS / flavor / ("DungeonEconomyPolicy.lua" if flavor == "lua" else "DungeonEconomyPolicy.epokbp")).read_bytes()).hexdigest()})
    if prepare_only: print("Prepared unified economy", flavor, flush=True); return
    def check(values, expected):
        assert values[:4] == [expected, STEPS, 0, STEPS] and values[5] > 0 and values[6:] == [POCKET_MASK, 23], values
    run_modes(out, project, "economy-" + flavor, selected, "dungeon_economy_policy_probe", MAGIC, check,
              provider_count=1, trace=("dungeon_economy_policy_trace", STEPS * WORDS), stop_after_seconds=120)
    validate_captures(base_out, out, flavor, selected, fingerprint)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p9/economy/providers")
    parser.add_argument("--flavor", action="append", choices=("cpp", "blueprint", "lua"))
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--host-oracle-only", action="store_true")
    args = parser.parse_args(); out = args.output.resolve()
    if args.host_oracle_only:
        result = host_oracle(out); print("PASS economy host reference", len(result), "words; no editor or console work", flush=True); return
    for flavor in args.flavor or ("cpp", "blueprint", "lua"): verify(out, flavor, args.mode, args.prepare_only, args.check_only)


if __name__ == "__main__":
    install_ui_extension(sys.modules[__name__])
    main()
