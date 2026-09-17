#include "native_music_data.hpp"
#include "instrument_preparation.hpp"
#include "instrument_allocator.hpp"
#include "spu_envelope.hpp"
#include "instrument_preview.h"
#include <vector>
#include <map>
#include <array>
#include <cstring>
#include <cmath>
#include <algorithm>
#include <new>
#include <memory>

namespace {
using namespace epok;
using namespace epok::native_music;
struct CompileStats {uint32_t error=0,commands=0,tones=0,peak=0,steals=0,adapted=0,automation=0;};
struct Voice {
    instrument::synth::State synth;Envelope envelope;
    Tone tone{};uint16_t note=0,zone=0;uint8_t channel=0;
    double position=0;uint32_t frames_fraction=0;bool active=false,released=false;
};
struct Compiler {
    sequence::Kernel kernel;instrument::BankView bank;uint16_t limit=16;
    Voice voices[24];instrument::allocation::Slot slots[24]{};
    std::vector<Command> commands;std::vector<Tone> tones;CompileStats stats;
    std::map<std::array<uint32_t,5>,EnvelopeFit> fits;
    uint32_t now=0,age=0;bool looping=false;
    bool emit(uint8_t op,uint8_t lane=0,uint16_t value=0){
        if(40+(commands.size()+1)*8+tones.size()*16>256*1024){stats.error=1;return false;}
        commands.push_back({now,op,lane,value});return true;
    }
    void event(const sequence::Event& e){if(e.op==sequence::LoopStart)emit(Mark);}
    bool active(uint16_t note)const{for(const auto& v:voices)if(v.active && v.note==note)return true;return false;}
    void finish(unsigned lane){const auto note=voices[lane].note;emit(Cut,uint8_t(lane));voices[lane].active=false;slots[lane]={};if(!active(note))kernel.retire(note);}
    void cut(uint16_t note){for(unsigned n=0;n<24;++n)if(voices[n].active && voices[n].note==note){emit(Cut,uint8_t(n));voices[n].active=false;slots[n]={};}}
    void release(uint16_t note){for(unsigned n=0;n<24;++n){auto& v=voices[n];if(!v.active || v.note!=note || v.released)continue;
        v.released=true;v.envelope.key_off();v.synth.release();emit(Release,uint8_t(n));}}
    Tone parameters(Voice& v){
        const auto out=v.synth.hardware_output();const auto& z=bank.zone(v.zone);
        const double pitch=double(bank.sample(z.sample).rate)*4096/44100*std::exp2(double(out.pitch_cents_x100)/120000);
        Tone t=v.tone;t.sample=z.sample;t.pitch=uint16_t(std::lround(std::clamp(pitch,1.0,16383.0)));
        const uint32_t gain=std::min(16383u,uint32_t(out.gain_q15)/2),pan=uint32_t(out.pan_permille+500);
        t.left=uint16_t(gain*(1000-pan)/1000);t.right=uint16_t(gain*pan/1000);
        const auto& channel=kernel.channels[v.channel];
        t.loop=z.loop_mode;t.send=uint8_t(bank.reverb_preset() && out.reverb_send_permille && (!channel.reverb_set || channel.reverb));return t;
    }
    void refresh(unsigned n){auto& v=voices[n];const auto t=parameters(v);
        if(t.pitch!=v.tone.pitch){emit(Pitch,uint8_t(n),t.pitch);++stats.automation;}
        if(t.left!=v.tone.left){emit(Left,uint8_t(n),t.left);++stats.automation;}
        if(t.right!=v.tone.right){emit(Right,uint8_t(n),t.right);++stats.automation;}
        if(t.send!=v.tone.send){emit(Send,uint8_t(n),t.send);++stats.automation;}
        v.tone=t;
    }
    void update(uint16_t note,const sequence::Channel& channel){for(unsigned n=0;n<24;++n){auto& v=voices[n];if(!v.active || v.note!=note)continue;
        const auto before=v.synth.hardware_parameters();
        if(v.synth.update_controls(instrument::preparation::controls(channel))!=instrument::synth::Error::None){stats.error=2;return;}
        const auto after=v.synth.hardware_parameters();
        // An authored controller which retimes an already-playing envelope
        // cannot silently keep the original ADSR. Preserve the source and fail
        // conversion explicitly; ordinary expression/pan/bend are compiled.
        if(before.attack!=after.attack || before.hold!=after.hold || before.decay!=after.decay || before.sustain!=after.sustain || before.release!=after.release){stats.error=9;return;}
        refresh(n);}}
    bool start(uint16_t note,const sequence::Note& incoming,const sequence::Channel& channel){
        uint16_t matching[128];unsigned count=0;uint32_t exclusive=0;
        for(unsigned z=0;z<bank.zone_count();++z){const auto& p=bank.zone(uint16_t(z));
            if(p.bank==channel.bank && p.program==channel.program && p.percussion==uint8_t(incoming.channel==9) &&
                incoming.key>=p.key_lo && incoming.key<=p.key_hi && incoming.velocity>=p.velocity_lo && incoming.velocity<=p.velocity_hi)matching[count++]=uint16_t(z);}
        if(!count || count>limit){stats.error=3;return false;}
        for(unsigned n=0;n<24;++n){if(!voices[n].active || voices[n].channel!=incoming.channel)continue;
            const auto& old=bank.zone(voices[n].zone);for(unsigned z=0;z<count;++z){const auto& p=bank.zone(matching[z]);
                if(p.exclusive_class && p.exclusive_class==old.exclusive_class && p.bank==old.bank && p.program==old.program && p.percussion==old.percussion)exclusive|=1u<<n;}}
        for(unsigned n=0;n<24;++n)if((exclusive&(1u<<n)) && voices[n].active){const auto id=voices[n].note;cut(id);kernel.retire(id);}
        const auto plan=instrument::allocation::reserve(slots,{0,128,uint8_t(count),uint8_t(limit),uint8_t(limit),1});
        if(!plan.fits){stats.error=4;return false;}
        for(unsigned n=0;n<24;++n)if((plan.evict_mask&(1u<<n)) && voices[n].active){const auto id=voices[n].note;cut(id);kernel.retire(id);++stats.steals;}
        // Group admission is atomic at runtime, including when SFX own voices.
        emit(Group,uint8_t(note),uint16_t(count));unsigned z=0;
        for(unsigned n=0;n<24;++n)if(plan.start_mask&(1u<<n)){
            auto& v=voices[n];v=Voice{};v.active=true;v.note=note;v.zone=matching[z++];v.channel=incoming.channel;
            if(v.synth.start_validated(bank,v.zone,incoming.key,incoming.velocity,instrument::preparation::controls(channel))!=instrument::synth::Error::None){stats.error=2;return false;}
            const auto env=v.synth.hardware_parameters();
            const std::array<uint32_t,5> key{env.attack,env.hold,env.decay,env.sustain,env.release};
            auto found=fits.find(key);if(found==fits.end())found=fits.emplace(key,fit_envelope(env.attack,env.hold,env.decay,env.sustain,env.release)).first;
            v.tone.adsr1=found->second.lo;v.tone.adsr2=found->second.hi;
            if(env.delay>1000 || env.hold>1000)++stats.adapted;
            v.tone=parameters(v);v.envelope.key_on(v.tone.adsr1,v.tone.adsr2);
            auto t=std::find_if(tones.begin(),tones.end(),[&](const Tone& p){return std::memcmp(&p,&v.tone,sizeof(Tone))==0;});
            uint16_t index=uint16_t(t-tones.begin());if(t==tones.end()){if(tones.size()>=65535){stats.error=1;return false;}tones.push_back(v.tone);}
            emit(Start,uint8_t(n),index);slots[n]={true,true,0,uint8_t(note),128,1,++age};
        }
        uint32_t playing=0;for(const auto& v:voices)playing+=v.active;stats.peak=std::max(stats.peak,playing);return !stats.error;
    }
    void advance(uint32_t elapsed){
        for(unsigned n=0;n<24;++n){auto& v=voices[n];if(!v.active)continue;
            const uint64_t frames=uint64_t(elapsed)*44100+v.frames_fraction;v.frames_fraction=uint32_t(frames%1000000);
            v.envelope.advance(uint32_t(frames/1000000));v.synth.advance(elapsed);
            v.position+=double(elapsed)*44100/1000000*v.tone.pitch/4096;
            const auto& s=bank.sample(v.tone.sample);const bool loop=v.tone.loop==1 || (v.tone.loop==3 && !v.released);
            if(loop && v.position>=s.loop_end)v.position=s.loop_start+std::fmod(v.position-s.loop_start,double(s.loop_end-s.loop_start));
            if(v.envelope.done() || (!loop && v.position>=s.frames)){finish(n);continue;}
            refresh(n);
        }
    }
    bool compile(const sequence::Event* input,uint32_t count,uint16_t ppqn){
        if(!bank.valid() || !input || !count || count>65536 || !ppqn || ppqn>32767 || !limit || limit>24)return false;
        bool marked=false;uint32_t mark=0;
        for(uint32_t n=0;n<count;++n){const auto& e=input[n];
            if(!sequence::valid_event(e) || (n && e.tick<input[n-1].tick))return false;
            if(e.op==sequence::LoopStart){if(marked)return false;marked=true;mark=e.tick;}
            if(e.op==sequence::End || e.op==sequence::LoopEnd){
                if(n+1!=count || (e.op==sequence::End && marked) || (e.op==sequence::LoopEnd && (!marked || e.tick<=mark)))return false;
            }
        }
        if(input[count-1].op!=sequence::End && input[count-1].op!=sequence::LoopEnd)return false;
        std::vector<sequence::Event> events(input,input+count);
        looping=events.back().op==sequence::LoopEnd;if(looping)events.back().op=sequence::End;
        kernel.begin_validated(events.data(),count,ppqn,sequence::Kernel::MaxVoices);
        uint32_t next_control=4000;
        while((kernel.running || kernel.active()) && !stats.error){
            if(now>600000000){stats.error=5;break;}
            uint32_t next=next_control;
            if(kernel.running){const auto& e=events[kernel.cursor];const uint64_t due=kernel.event_time+uint64_t(e.tick-kernel.tick)*kernel.tempo;
                const uint64_t us=(due+ppqn-1)/ppqn;if(us>600000000){stats.error=5;break;}next=std::min(next,uint32_t(us));}
            if(next<now){stats.error=6;break;}const uint32_t elapsed=next-now;now=next;
            advance(elapsed);kernel.advance(elapsed,*this);
            if(kernel.error!=sequence::Error::None){stats.error=100+uint32_t(kernel.error);break;}
            if(now==next_control)next_control+=4000;
            if(looping && !kernel.running){for(unsigned n=0;n<24;++n)if(voices[n].active)finish(n);break;}
        }
        emit(looping?Loop:End);stats.commands=uint32_t(commands.size());stats.tones=uint32_t(tones.size());return !stats.error && !tones.empty();
    }
};
void put16(std::vector<uint8_t>& out,uint16_t v){out.push_back(uint8_t(v));out.push_back(uint8_t(v>>8));}
void put32(std::vector<uint8_t>& out,uint32_t v){put16(out,uint16_t(v));put16(out,uint16_t(v>>16));}
struct Result {std::vector<uint8_t> bytes;CompileStats stats;};
}
extern "C" void* epok_native_music_compile(const epok::sequence::Event* events,uint32_t count,uint16_t ppqn,uint16_t voices,
    const uint8_t* bank,uint32_t bytes){
    try{
        auto compiler=std::make_unique<Compiler>();compiler->bank={bank,bytes};compiler->limit=voices;
        auto result=std::make_unique<Result>();
        if(!compiler->compile(events,count,ppqn)){result->stats=compiler->stats;if(!result->stats.error)result->stats.error=7;return result.release();}
        auto& out=result->bytes;out={'E','P','S','Q'};put16(out,3);put16(out,40);put16(out,ppqn);put16(out,voices);
        put32(out,uint32_t(compiler->commands.size()));put32(out,compiler->now);put32(out,uint32_t(compiler->tones.size()));out.resize(40);
        for(const auto& e:compiler->commands){put32(out,e.time);out.push_back(e.op);out.push_back(e.lane);put16(out,e.value);}
        for(const auto& p:compiler->tones){put16(out,p.sample);put16(out,p.pitch);put16(out,p.left);put16(out,p.right);put16(out,p.adsr1);put16(out,p.adsr2);out.push_back(p.loop);out.push_back(p.send);put16(out,0);}
        result->stats=compiler->stats;
        if(!epok::native_music::View{out.data(),uint32_t(out.size())}.valid(compiler->bank.sample_count()))result->stats.error=8;
        return result.release();
    }catch(...){return nullptr;}
}
extern "C" const uint8_t* epok_native_music_bytes(void* handle,uint32_t* size,CompileStats* stats){
    if(!handle)return nullptr;auto& r=*static_cast<Result*>(handle);*size=uint32_t(r.bytes.size());*stats=r.stats;return r.bytes.data();
}
extern "C" void epok_native_music_destroy(void* handle){delete static_cast<Result*>(handle);}

namespace {
struct Audition {
    View stream;instrument::BankView bank;const EpokInstrumentPcm* pcm=nullptr;
    struct Playing {Envelope envelope;Tone tone{};double position=0;uint8_t note=0;bool active=false,released=false;};
    Playing voices[24];uint32_t cursor=0,loop_cursor=0,loop_time=0;uint64_t frames=0,time_base=0;
    EpokInstrumentStats stats{};bool ended=false;uint8_t group_note=0;
    void dispatch(uint64_t time){
        unsigned limit=0;
        while(!ended && cursor<stream.count() && stream.commands()[cursor].time<=time-time_base){
            if(++limit>2048){stats.error=1;return;}
            const auto e=stream.commands()[cursor++];
            if(e.op==Group){group_note=e.lane;continue;}
            if(e.op==Mark){loop_cursor=cursor;loop_time=e.time;continue;}
            if(e.op==Loop){for(auto& v:voices)v.active=false;time_base+=e.time-loop_time;cursor=loop_cursor;++stats.loops;continue;}
            if(e.op==End){ended=true;continue;}
            auto& v=voices[e.lane];
            if(e.op==Start){v=Playing{};v.active=true;v.note=group_note;v.tone=stream.patches()[e.value];v.envelope.key_on(v.tone.adsr1,v.tone.adsr2);}
            else if(!v.active)continue;
            else if(e.op==Release){v.released=true;v.envelope.key_off();}
            else if(e.op==Cut)v.active=false;
            else if(e.op==Pitch)v.tone.pitch=e.value;
            else if(e.op==Left)v.tone.left=e.value;
            else if(e.op==Right)v.tone.right=e.value;
            else if(e.op==Send)v.tone.send=uint8_t(e.value);
        }
    }
    int render(int16_t* output,uint32_t count){
        for(uint32_t f=0;f<count;++f){dispatch(frames*1000000/44100);if(stats.error)return int(stats.error);
            double left=0,right=0;uint32_t active=0,logical=0;bool notes[128]{};
            for(auto& v:voices){if(!v.active)continue;++active;if(!notes[v.note]){notes[v.note]=true;++logical;}const auto& sample=bank.sample(v.tone.sample);const auto& data=pcm[v.tone.sample];
                const bool loop=v.tone.loop==1 || (v.tone.loop==3 && !v.released);
                if(loop && v.position>=sample.loop_end){v.position=sample.loop_start+std::fmod(v.position-sample.loop_start,double(sample.loop_end-sample.loop_start));++stats.sample_loops;}
                if(v.position>=sample.frames || v.envelope.done()){v.active=false;continue;}
                const auto position=uint32_t(v.position);auto next=position+1;
                if(loop && next>=sample.loop_end)next=sample.loop_start;else if(next>=sample.frames)next=position;
                const double frac=v.position-position;
                const double value=(data.framesdata[position]*(1-frac)+data.framesdata[next]*frac)*double(v.envelope.level)/32767;
                left+=value*v.tone.left/16384;right+=value*v.tone.right/16384;
                v.envelope.advance(1);v.position+=double(v.tone.pitch)/4096;
            }
            stats.physical_peak=std::max(stats.physical_peak,active);stats.logical_peak=std::max(stats.logical_peak,logical);
            for(unsigned c=0;c<2;++c){const double value=(c?right:left)*32768;if(value<-32768 || value>32767)++stats.clipped;output[f*2+c]=int16_t(std::clamp(std::lround(value),-32768l,32767l));}
            ++frames;
        }return 0;
    }
};
}
extern "C" void* epok_native_preview_create(const uint8_t* stream,uint32_t length,const uint8_t* bank,uint32_t size,const EpokInstrumentPcm* pcm,uint32_t samples){
    try{auto out=std::make_unique<Audition>();out->stream={stream,length};out->bank={bank,size};out->pcm=pcm;
        if(!out->bank.valid() || !out->stream.valid(out->bank.sample_count()) || samples!=out->bank.sample_count() || !pcm)return nullptr;
        for(unsigned n=0;n<samples;++n)if(!pcm[n].framesdata || pcm[n].frames!=out->bank.sample(uint16_t(n)).frames)return nullptr;
        return out.release();}catch(...){return nullptr;}
}
extern "C" void epok_native_preview_destroy(void* handle){delete static_cast<Audition*>(handle);}
extern "C" int epok_native_preview_render(void* handle,int16_t* out,uint32_t frames){return handle && out?static_cast<Audition*>(handle)->render(out,frames):1;}
extern "C" EpokInstrumentStats epok_native_preview_stats(const void* handle){return handle?static_cast<const Audition*>(handle)->stats:EpokInstrumentStats{1};}
