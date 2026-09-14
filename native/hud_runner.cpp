// A separate native executable compiled from the current scene and its actual
// controllers. Binary stdout is reserved for frames; script diagnostics use stderr.
#include <cstdio>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
extern "C" __declspec(dllimport) unsigned long __stdcall SetErrorMode(unsigned long);
#else
#include <unistd.h>
#endif
#include "scene.hh"
#include "hud_commands.hpp"
#include "hud-config.hh"

namespace epok {
MusicStats music_stats;PerformanceStats performance_stats;LightingEnvironment lighting_environment;
LightingStats lighting_stats;MeshStats mesh_stats;
AudioSource* music_active=nullptr;AudioSource* music_requested=nullptr;
static std::string requested_scene;
static std::map<const AudioSource*,bool> audio_playing;
void AudioSource::play(){audio_playing[this]=true;}
void AudioSource::stop(){audio_playing[this]=false;}
bool AudioSource::is_playing() const {auto i=audio_playing.find(this);return i!=audio_playing.end()&&i->second;}
void remove_runtime_owner(size_t){}
void reset_runtime_services(){}
bool request_scene(const char* name){if(!name||!*name||!requested_scene.empty())return false;requested_scene=name;return true;}
bool request_scene(size_t index){return request_scene(std::to_string(index).c_str());}
bool request_scene(const char* name,const TransitionOptions&){return request_scene(name);}
bool request_scene(size_t index,const TransitionOptions&){return request_scene(index);}
bool scene_loading(){return !requested_scene.empty();}
size_t current_scene(){return 0;}
}
#include "lifecycle.hpp"

class PreviewCards final:public epok::MemoryCardDriver {
    std::function<void()> pending;
    std::map<std::string,std::vector<uint8_t>> files;
public:
    bool idle()const override{return !pending;}
    void pump(){if(pending){auto next=std::move(pending);pending={};next();}}
    void probe(unsigned,void* owner,Completion cb)override{pending=[=]{cb(owner,epok::CardError::OK);};}
    void read(unsigned port,const char* name,void* bytes,uint32_t capacity,uint32_t* size,void* owner,Completion cb)override{
        std::string key=std::to_string(port)+":"+name;
        pending=[=,this]{auto it=files.find(key);if(it==files.end()){cb(owner,epok::CardError::FileNotFound);return;}if(it->second.size()>capacity){cb(owner,epok::CardError::FileTooLarge);return;}*size=uint32_t(it->second.size());memcpy(bytes,it->second.data(),*size);cb(owner,epok::CardError::OK);};
    }
    void write(unsigned port,const char* name,const char*,const epok::CardIcon&,const void* bytes,uint32_t size,void* owner,Completion cb)override{
        std::string key=std::to_string(port)+":"+name;auto p=static_cast<const uint8_t*>(bytes);std::vector<uint8_t> copy(p,p+size);
        pending=[=,this]{files[key]=copy;cb(owner,epok::CardError::OK);};
    }
    void list(unsigned,epok::CardFile*,uint32_t* count,void* owner,Completion cb)override{pending=[=]{*count=0;cb(owner,epok::CardError::OK);};}
};
static FILE* frames=nullptr;
static bool write32(uint32_t value){uint8_t bytes[4];for(unsigned i=0;i<4;++i)bytes[i]=uint8_t(value>>(i*8));return fwrite(bytes,1,4,frames)==4;}
static bool read32(uint32_t& value){uint8_t bytes[4];if(fread(bytes,1,4,stdin)!=4)return false;value=0;for(unsigned i=0;i<4;++i)value|=uint32_t(bytes[i])<<(i*8);return true;}
// The phase this process runs in. Edit never advances game time and never calls
// BeginPlay/start/update; Simulate does. It is fixed at startup by --edit and
// reported in every frame header so the editor cannot disagree with the child.
static uint32_t preview_capabilities=0;
static void frame(){
    EpokHudSink sink;for(size_t i=0;i<epok::texture_count;++i){sink.dimensions.push_back(epok::texture_assets[i].width);sink.dimensions.push_back(epok::texture_assets[i].height);}
    epok::hud_core::Compiler compiler(sink,epok::display_width,epok::display_height,{epok::hud_layout_budget,epok::hud_rectangle_budget,epok::hud_text_budget,epok::hud_glyph_budget});
    int first[epok::objects.size()],next[epok::objects.size()];compiler.draw(epok::objects.data(),epok::object_count,first,next);
    auto s=compiler.stats;
    write32(EPOK_HUD_PREVIEW_MAGIC);write32(EPOK_HUD_PREVIEW_PROTOCOL_VERSION);write32(preview_capabilities);
    write32(epok::performance_stats.frame);write32(epok::screen_fade);write32(uint32_t(epok::object_count));
    write32(s.rectangles);write32(s.glyphs);write32(s.texts);write32(s.images);write32(s.dropped);
    write32(uint32_t(sink.commands.size()));write32(uint32_t(epok::requested_scene.size()));
    for(const auto& command:sink.commands)for(auto value:command.v)write32(uint32_t(value));
    fwrite(epok::requested_scene.data(),1,epok::requested_scene.size(),frames);fflush(frames);
}
int main(int argc,char** argv){
    const bool editing=argc>1&&strcmp(argv[1],"--edit")==0;
    // Blueprint is never reported: this executable links C++ controllers only.
    preview_capabilities=editing?0u:EPOK_HUD_PREVIEW_CAP_SIMULATE;
#ifdef _WIN32
    SetErrorMode(0x0001|0x0002); // SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX
    _setmode(_fileno(stdin),_O_BINARY);_setmode(_fileno(stdout),_O_BINARY);
    frames=_fdopen(_dup(_fileno(stdout)),"wb");_dup2(_fileno(stderr),_fileno(stdout));
#else
    frames=fdopen(dup(fileno(stdout)),"wb");dup2(fileno(stderr),fileno(stdout));
#endif
    if(!frames)return 2;
    PreviewCards cards;epok::memory_card.attach(cards);
    epok::time.reset(0);epok::input.reset();
    epok::editor_preview_active=editing;
    if(editing)epok::initialize_editor_preview();else {epok::initialize_components();if(epok::load_actor_bank(epok::actor_table,epok::objects.data(),epok::object_count)!=epok::actor_table.count)return 4;}
    frame();
    uint32_t now=0,elapsed=0,buttons=0;
    while(read32(elapsed)&&read32(buttons)){
        if(elapsed>100000||buttons>65535)return 3;
        if(!editing&&epok::requested_scene.empty()){
            now+=elapsed;epok::input.sample(0,true,uint16_t(buttons));
            const auto steps=epok::time.advance(now);
            epok::level.frame_update(epok::time.frame_microseconds);
            if(epok::time.paused()||epok::scene_loading())epok::input.discard_edges();
            for(unsigned i=0;i<steps&&!epok::time.paused()&&!epok::scene_loading();++i){
                epok::time.begin_tick();epok::input.begin_tick();
                epok::level.tick(epok::Fixed(epok::time.delta_raw,epok::Fixed::RAW));
                epok::input.end_tick();
            }
            cards.pump();++epok::performance_stats.frame;
        }
        frame();
    }
}
