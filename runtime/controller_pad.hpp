#pragma once
// Epok's single SIO0 pad owner. Direct multitap addressing (01h..04h) keeps
// configuration and input packets on the same path, including an empty slot A.
// Adapted from PsyQo AdvancedPad, Copyright (c) 2025 PCSX-Redux authors.
// SPDX-License-Identifier: MIT
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
#include "psyqo/advancedpad.hh"
#pragma push_macro("DPCR")
#pragma push_macro("DICR")
#undef DPCR
#undef DICR
#include "psyqo/hardware/cpu.hh"
#pragma pop_macro("DICR")
#pragma pop_macro("DPCR")
#include "psyqo/hardware/sio.hh"
#include "psyqo/kernel.hh"
#include "psyqo/sio0-bus.hh"
#include "rumble.hpp"

namespace epok {
class ControllerPad {
public:
    using Pad=psyqo::AdvancedPad::Pad;
    using Button=psyqo::AdvancedPad::Button;
    // Result: 0 okay, 1 bus owned, 2 RX timeout, 3 ACK timeout,
    // 4 absent ID, 5 oversized reply, 6 invalid ID-high byte.
    struct TransferTrace {
        uint8_t command=0,index=0,id=0xff,header=0xff,result=0;
        uint32_t status=0;
    };
private:
    enum class Phase:uint8_t { Read,Enter,Identify,Map,Verify,Exit,Confirm,RejectExit };
    struct State {
        uint16_t buttons=0xffff;
        uint8_t type=0xff,adc[4]={128,128,128,128},large=0,lease=0;
        bool connected=false,small=false,ready=false,unsupported=false;
        Phase phase=Phase::Read;
    };
    State pads_[4]{};
    TransferTrace traces_[4]{};
    bool enabled_=false;
    uint32_t host_sequence_=0;
    uint32_t last_poll_clock_=0;
    bool polled_=false,have_poll_clock_=false;
    static void delay(unsigned n){for(unsigned i=0;i<n;++i)asm volatile("");}
    static bool byte(uint8_t out,uint8_t& in){
        using namespace psyqo::Hardware;
        CPU::IReg.access()=~static_cast<uint32_t>(CPU::IRQ::Controller);
        SIO::Ctrl|=SIO::Control::CTRL_ERRRES;
        SIO::Data=out;
        for(unsigned n=0;n<0x3000;++n)if(SIO::Stat&SIO::Status::STAT_RXRDY){in=SIO::Data;return true;}
        return false;
    }
    static bool ack(){
        using namespace psyqo::Hardware;
        unsigned n=0;
        // The resident serial monitor can service/acknowledge I_STAT while we
        // poll. SIO0's own ACK latch survives that unrelated IRQ dispatch.
        while(!(SIO::Stat&SIO::Status::STAT_IRQ))if(++n>=0x137)return false;
        n=0;
        while(SIO::Stat&SIO::Status::STAT_ACK)if(++n>=0x137)return false;
        return true;
    }
    // Every exit releases DTR, even when RX, ACK, an address or a reply fails.
    static bool exchange(unsigned slot,uint8_t command,const uint8_t* payload,uint8_t* reply,TransferTrace& trace){
        using namespace psyqo::Hardware;
        trace={command,0,0xff,0xff,0,0};
        if(psyqo::SIO0Bus::owned()){trace.result=1;return false;}
        const bool controller_irq=CPU::IMask.isSet(CPU::IRQ::Controller);
        CPU::IMask.clear(CPU::IRQ::Controller);CPU::flushWriteQueue();
        SIO::Ctrl=SIO::Control::CTRL_DTR;
        SIO::Baud=0x88;SIO::Mode=0x0d;
        for(unsigned n=0;n<16&&(SIO::Stat&SIO::Status::STAT_RXRDY);++n)SIO::Data.throwAway();
        SIO::Ctrl|=SIO::Control::CTRL_TXEN|SIO::Control::CTRL_ACKIRQEN;
        delay(100);
        // A digital reply advertises five bytes, but SCPH-1200 configuration
        // entry still needs all nine command bytes clocked out. Only bytes
        // inside the advertised reply request an ACK; padding has no ACK.
        const unsigned command_length=command==0x42?5:9;
        unsigned reply_length=5,length=command_length;bool ok=true;
        for(unsigned i=0;i<length;++i){
            trace.index=uint8_t(i);
            const uint8_t out=i==0?uint8_t(slot+1):i==1?command:i==2?0:payload[i-3];
            if(!byte(out,reply[i])){trace.result=2;ok=false;break;}
            if(i==1){
                trace.id=reply[1];
                if(reply[1]==0xff){trace.result=4;ok=false;break;}
                unsigned words=reply[1]&15;if(!words)words=16;
                reply_length=3+words*2;
                if(reply_length>35){trace.result=5;ok=false;break;}
                length=reply_length>command_length?reply_length:command_length;
            }
            if(i==2)trace.header=reply[2];
            if(i+1<reply_length&&!ack()){trace.result=3;ok=false;break;}
        }
        trace.status=SIO::Stat;
        SIO::Ctrl=0;
        CPU::IReg.access()=~static_cast<uint32_t>(CPU::IRQ::Controller);
        if(controller_irq)CPU::IMask.set(CPU::IRQ::Controller);
        CPU::flushWriteQueue();
        // Resident loaders can leave an ordinary pad's ID-high byte at zero.
        // Preserve its normal input while configuration replies stay strict.
        const bool normal_header=(command==0x42||command==0x43)&&
            ((reply[1]&0xf0)==0x40||reply[1]==0x53||reply[1]==0x73);
        if(ok&&reply[2]!=0x5a&&!(normal_header&&reply[2]==0)){trace.result=6;ok=false;}
        return ok;
    }
    static bool normal(State& s,const uint8_t* reply){
        const uint8_t type=reply[1];
        if(type==0xf3||type==0x80||type==0xff||!(type&15))return false;
        if(s.connected&&(s.type&0xf0)!=(type&0xf0)){s.ready=false;s.unsupported=false;}
        s.connected=true;s.type=type;s.buttons=uint16_t(reply[3])|(uint16_t(reply[4])<<8);
        const bool analog=type==0x53||type==0x73;
        for(unsigned a=0;a<4;++a)s.adc[a]=analog?reply[5+a]:128;
        // Keep ordinary buttons on mice/lightguns/other accessories, without
        // sending DualShock configuration commands to those device classes.
        if((type&0xf0)!=0x40&&!analog)s.unsupported=true;
        return true;
    }
    void poll(unsigned p){
        auto& s=pads_[p];uint8_t reply[35]={};uint8_t data[32]={};
        uint8_t command=0x42;
        switch(s.phase){
        case Phase::Enter: command=0x43;data[0]=1;break;
        case Phase::Identify:command=0x45;break;
        case Phase::Map:case Phase::Verify:
            command=0x4d;data[0]=0;data[1]=1;for(unsigned i=2;i<6;++i)data[i]=0xff;break;
        case Phase::Exit:case Phase::RejectExit:
            command=0x43;for(unsigned i=1;i<6;++i)data[i]=0x5a;break;
        default:if(s.ready&&s.lease){data[0]=s.small?1:0;data[1]=s.large;}break;
        }
        const bool valid=exchange(p,command,data,reply,traces_[p]);
        if(s.phase==Phase::Read||s.phase==Phase::Confirm){
            // A previous program may have left the multitap in buffered mode.
            // The zero third byte above clears it; the next direct read recovers.
            if(valid&&reply[1]==0x80)return;
            if(!valid||!normal(s,reply)){
                if(valid&&reply[1]==0xf3){s.ready=false;s.phase=Phase::RejectExit;return;}
                s=State{};return;
            }
            if(s.phase==Phase::Confirm){s.ready=true;s.phase=Phase::Read;}
            else if(enabled_&&!s.ready&&!s.unsupported)s.phase=Phase::Enter;
            return;
        }
        if(s.phase==Phase::RejectExit){s.phase=Phase::Read;s.unsupported=true;s.ready=false;s.large=0;s.small=false;s.lease=0;return;}
        if(!valid||(s.phase!=Phase::Enter&&reply[1]!=0xf3)){
            s.phase=Phase::RejectExit;s.ready=false;return;
        }
        switch(s.phase){
        case Phase::Enter:s.phase=Phase::Identify;break;
        case Phase::Identify:s.phase=Phase::Map;break;
        case Phase::Map:s.phase=Phase::Verify;break;
        case Phase::Verify:s.phase=reply[3]==0&&reply[4]==1?Phase::Exit:Phase::RejectExit;break;
        case Phase::Exit:s.phase=Phase::Confirm;break;
        default:break;
        }
    }
    void frame(){
        if(epok_rumble_control[0]==0x4550524du){
            const uint32_t sequence=epok_rumble_control[2];
            if(sequence!=host_sequence_||!(epok_rumble_control[1]&1))stopRumble();
            host_sequence_=sequence;
        }
        if(!psyqo::SIO0Bus::owned()){
            for(unsigned p=0;p<4;++p)poll(p);
            polled_=true;
        }
        // Submit a one-frame pulse before consuming its last lease frame. The
        // lease still ages while a card owns SIO0, so blocked output expires.
        for(auto& s:pads_)if(s.lease&&!--s.lease){s.large=0;s.small=false;}
    }
public:
    // A long card transaction or blocked game frame may let the controller's
    // own watchdog forget its configuration. Re-probe rather than keeping a
    // stale "ready" flag when polling resumes. The clock is GPU microseconds.
    void synchronize(uint32_t now){
        if(have_poll_clock_&&now-last_poll_clock_>500000){
            stopRumble();
            for(auto& s:pads_)if(s.connected){s.ready=false;s.phase=Phase::Read;if((s.type&0xf0)==0x40||s.type==0x53||s.type==0x73)s.unsupported=false;}
        }
        if(polled_){last_poll_clock_=now;have_poll_clock_=true;polled_=false;}
    }
    void initialize(bool enable_rumble=false){
        enabled_=enable_rumble;
        using namespace psyqo::Hardware;
        SIO::Ctrl=SIO::Control::CTRL_IR;SIO::Baud=0x88;SIO::Mode=0x0d;SIO::Ctrl=0;
        // This replaces AdvancedPad initialization; there is only one pad poller.
        psyqo::Kernel::Internal::addOnFrame([this](){frame();});
    }
    bool isPadConnected(Pad pad)const{return unsigned(pad)<4&&pads_[unsigned(pad)].connected;}
    bool isButtonPressed(Pad pad,Button button)const{return unsigned(pad)<4&&unsigned(button)<16&&!(pads_[unsigned(pad)].buttons&(1u<<unsigned(button)));}
    uint8_t getPadType(Pad pad)const{return unsigned(pad)<4?pads_[unsigned(pad)].type:0xff;}
    uint8_t getAdc(Pad pad,unsigned axis)const{return unsigned(pad)<4&&axis<4?pads_[unsigned(pad)].adc[axis]:128;}
    bool rumbleSupported(unsigned p)const{return p<4&&pads_[p].ready;}
    bool rumbleProbing(unsigned p)const{return enabled_&&p<4&&pads_[p].connected&&!pads_[p].ready&&!pads_[p].unsupported;}
    const TransferTrace& trace(unsigned p=0)const{return traces_[p<4?p:0];}
    void setRumble(unsigned p,uint8_t large,bool small,uint32_t remaining_us){
        if(p>=4)return;
        auto& s=pads_[p];
        s.large=large;s.small=small;
        const uint32_t frames=(remaining_us+16665)/16666;
        s.lease=(large||small)?uint8_t(frames>15?15:frames):0;
    }
    void stopRumble(){for(unsigned p=0;p<4;++p)setRumble(p,0,false,0);}
};
}
