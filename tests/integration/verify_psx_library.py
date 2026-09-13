"""Owned SF2 layers + RPN pitch through the actual EPSB v2 PSX service.

The emulator capture is the hardware voice-1 buffer; no host preview PCM is
substituted. Run sequentially with other builds/emulator tests.
"""
from pathlib import Path
import json
import math
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
from spu_capture_tone import fit as fit_tone

EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PROFILE = "--profile" in sys.argv

def chunk(tag, data):
    return tag + struct.pack("<I", len(data)) + data + bytes(len(data) & 1)

def font():
    def name(text, size=20):
        return text.encode().ljust(size, b"\0")
    def table(zones):
        bags, generators = bytearray(), bytearray()
        for zone in zones:
            bags += struct.pack("<HH", len(generators)//4, 0)
            generators += b"".join(struct.pack("<Hh", *g) for g in zone)
        bags += struct.pack("<HH", len(generators)//4, 0)
        return bytes(bags), bytes(10), bytes(generators)+bytes(4)
    pbag,pmod,pgen = table([[(41,0)]])
    # Two independently panned layers, continuous sample loop, authored release.
    ibag,imod,igen = table([[(17,pan),(16,200),(33,-32768),(34,-12000),(35,-32768),
                           (38,-3600),(54,1),(53,0)] for pan in (-250,250)])
    phdr = b"".join(name(label)+struct.pack("<HHHIII",0,0,bag,0,0,0) for label,bag in [("Layered sine",0),("EOP",1)])
    inst = name("Layered sine")+struct.pack("<H",0)+name("EOI")+struct.pack("<H",2)
    frames, rate = 5600,22000
    pcm = b"".join(struct.pack("<h",round(14000*math.sin(i*2*math.pi/50))) for i in range(frames))
    shdr = name("Owned A4")+struct.pack("<IIIIIBbHH",0,frames,0,frames,rate,69,0,0,1)+name("EOS",46)
    info = chunk(b"LIST", b"INFO"+chunk(b"ifil",struct.pack("<HH",2,4))+chunk(b"INAM",b"Epok owned fixture\0"))
    sdta = chunk(b"LIST",b"sdta"+chunk(b"smpl",pcm+bytes(92)))
    pdta = chunk(b"LIST",b"pdta"+b"".join(chunk(k,v) for k,v in [(b"phdr",phdr),(b"pbag",pbag),(b"pmod",pmod),(b"pgen",pgen),(b"inst",inst),(b"ibag",ibag),(b"imod",imod),(b"igen",igen),(b"shdr",shdr)]))
    return chunk(b"RIFF",b"sfbk"+info+sdta+pdta)

def main():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1",8077))
    stamp=time.time_ns()
    art=ROOT/"artifacts/midi-completion"/f"p4-psx-library-{stamp}"
    art.mkdir(parents=True)
    project=ROOT/".epok"/f"psx-library-{stamp}"
    print(f"Project: {project}\nEvidence: {art}",flush=True)
    def run(*args):
        result=subprocess.run([str(EXE),*map(str,args)],capture_output=True,text=True,timeout=300,creationflags=FLAGS)
        with (art/"commands.log").open("a",encoding="utf-8") as log:
            log.write(f"{args}\nexit={result.returncode}\n{result.stdout}{result.stderr}\n")
        assert result.returncode==0,result.stdout+result.stderr
        return result.stdout
    def ids():
        return {r["path"]:r["id"] for r in json.loads(run("--project",project,"--scan-assets"))["assets"]}
    run("--create-project",project,"--template","sample")
    run("--project",project,"--create-starter-bank","assets/Retro.epokasset")
    (project/"assets/Layered.sf2").write_bytes(font())
    run("--project",project,"--import-sound-bank","assets/Layered.sf2")
    bank=ids()["assets/Layered.epokasset"]
    # RPN0 = 12 semitones; bend 9600 gives +206.25 cents at key 69.
    track=bytes.fromhex("00b0650000b0640000b0060c00b05b7f00e0004b009045646080450000b05b0060ff2f00")
    (project/"assets/song.mid").write_bytes(b"MThd"+struct.pack(">IHHH",6,0,1,96)+b"MTrk"+struct.pack(">I",len(track))+track)
    recipe={"preset":"Custom","max_sample_rate":22000,"effects":"Room","reverb_depth_permille":250}
    documents.write_text(project/"assets/recipe.json",json.dumps(recipe))
    run("--project",project,"--import-audio","assets/song.mid","--sound-bank",bank,"--sequence-loop","whole","--voice-limit","16","--psx-music-recipe","assets/recipe.json")
    records=ids(); song=records["assets/song.epokasset"]; sample=records["assets/Retro-triangle.epokasset"]
    run("--project",project,"--reimport-asset","assets/Retro-triangle.epokasset","--snapshot","--loop")
    scene_path=project/"assets/scenes/SampleScene.epokmap"
    scene=documents.loads(scene_path.read_text())
    spinner=next(e for e in scene["entities"] if (e.get("script") or {}).get("name")=="Spinner")
    spinner["audio"]={"clip":song,"volume":0.5}
    scene["entities"][0]["audio"]={"clip":sample,"volume":0.1,"play_on_start":False}
    documents.write_text(scene_path,json.dumps(scene))
    script=r'''
#include "Spinner.hpp"
#include "common/hardware/spu.h"
#include "common/hardware/dma.h"
#include "common/hardware/counters.h"
uint32_t library_probe[16]={};
uint32_t library_cost[8]={};
alignas(4) int16_t library_capture[20][512]={};
alignas(4) int16_t library_wet[512]={};
epok::AudioSource library_sfx;
PROFILE_HELPER
void Spinner::start(epok::Transform&){
    entity().audio.stop();library_sfx.enabled=true;library_sfx.clip=DUMMY_CLIP;
    library_sfx.volume=0.0;library_sfx.priority=255;library_sfx.play();entity().audio.play();
}
void Spinner::update(epok::Transform&,epok::Fixed){
    auto& source=entity().audio;const auto frame=++library_probe[0];
    if(frame==40){source.stop();library_probe[1]=!source.is_playing();}
    if(frame==45){source.play();library_probe[2]=source.is_playing();}
    if(frame>=70 && frame<90){
        library_probe[3]=source.is_playing();
        library_probe[4]=SPU_VOICES[1].sampleRate;
        library_probe[5]=SPU_VOICES[2].sampleRate;
        library_probe[6]|=uint32_t(*(volatile uint16_t*)0x1f801d98);
        const uint16_t control=SPU_CTRL;library_probe[7]=control;
        SPU_RAM_DTA=0x800>>3;SPU_CTRL=(control&~0x30)|0x30;
        DMA_CTRL[DMA_SPU].MADR=uint32_t(uintptr_t(library_capture[frame-70]));
        DMA_CTRL[DMA_SPU].BCR=(16<<16)|16;DMA_CTRL[DMA_SPU].CHCR=0x01000200;
        uint32_t timeout=1000000;while((DMA_CTRL[DMA_SPU].CHCR&0x01000000) && --timeout){}
        library_probe[8]=timeout!=0;SPU_CTRL=control;
        if(frame==89)library_probe[12]=uint32_t(*(volatile uint16_t*)0x1f801d98);
    }
    if(frame==94){
        const uint16_t control=SPU_CTRL;SPU_RAM_DTA=0x7d940>>3;SPU_CTRL=(control&~0x30)|0x30;
        DMA_CTRL[DMA_SPU].MADR=uint32_t(uintptr_t(library_wet));
        DMA_CTRL[DMA_SPU].BCR=(16<<16)|16;DMA_CTRL[DMA_SPU].CHCR=0x01000200;
        uint32_t timeout=1000000;while((DMA_CTRL[DMA_SPU].CHCR&0x01000000) && --timeout){}
        library_probe[9]=timeout!=0;SPU_CTRL=control;
    }
    if(frame==95){source.enabled=false;}
    if(frame==97){library_probe[10]=!source.is_playing();source.enabled=true;source.play();}
    PROFILE_CALL
}
'''.replace("DUMMY_CLIP",str(sorted([song,sample]).index(sample)))
    profile_helper=r'''
#include "instrument_synth.hpp"
#include "sequence_kernel.hpp"
#include "sequence_lock.hpp"
namespace epok {extern const uint8_t sequence_bank_data_0[];extern const uint8_t sequence_data_SONG_CLIP[];}
void measure_library(){
    static epok::instrument::synth::State state;
    static epok::sequence::Kernel kernel;
    const epok::instrument::BankView bank{epok::sequence_bank_data_0,epok::instrument::u32(epok::sequence_bank_data_0+28)};
    const auto* sequence=epok::sequence_data_SONG_CLIP;
    epok::instrument::synth::Controls controls;controls.bend_range_cents=1200;controls.bend=9600;
    const auto measure=[](unsigned index,auto call){epok::SequenceLock lock;const uint16_t start=COUNTERS[2].value;call();library_cost[index]=uint16_t(COUNTERS[2].value-start);};
    measure(0,[&]{state.start(bank,0,69,100,controls);});
    measure(1,[&]{state.start_validated(bank,0,69,100,controls);});
    measure(2,[&]{state.advance(1000);});
    measure(3,[&]{controls.cc[1]=1;state.update_controls(controls);});
    measure(4,[&]{kernel.begin_validated(reinterpret_cast<const epok::sequence::Event*>(sequence+40),epok::instrument::u32(sequence+12),epok::instrument::u16(sequence+8),128);});
    measure(5,[&]{state.reset();});
}
'''.replace("SONG_CLIP",str(sorted([song,sample]).index(song)))
    script=script.replace("PROFILE_HELPER",profile_helper if PROFILE else "").replace("PROFILE_CALL","if(frame==130)measure_library();" if PROFILE else "")
    documents.write_text(project/"assets/scripts/Spinner.cpp",script)
    run("--project",project,"--build-psx")
    build=project/".epok/build"
    cook=json.loads((build/"audio/sequence-report.json").read_text())
    def request(path,post=False):
        req=urllib.request.Request("http://127.0.0.1:8077/api/v1/"+path,data=b"" if post else None)
        with urllib.request.urlopen(req,timeout=3) as response:return response.read()
    with (art/"emulator.log").open("w") as log:
        process=subprocess.Popen([str(EXE),"--project",str(project),"--play-psx","--stop-after","9"],stdout=log,stderr=log,creationflags=FLAGS)
        try:
            deadline=time.monotonic()+80
            while True:
                assert process.poll() is None,"Emulator boot failed"
                try:
                    if json.loads(request("execution-flow"))["running"]:break
                except (OSError,ValueError):pass
                assert time.monotonic()<deadline,"Emulator did not start"
                time.sleep(.1)
            time.sleep(3.5)
            request("execution-flow?function=pause",True)
            symbols=(build/"epok.map").read_text(); ram=request("cpu/ram/raw")
            def address(name):
                match=re.search(r"0x([0-9a-f]+)\s+"+re.escape(name)+r"\b",symbols)
                assert match,name
                return int(match[1],16)&0x1fffff
            probe=struct.unpack_from("<16I",ram,address("library_probe"))
            stats=struct.unpack_from("<16I",ram,address("epok::music_sequence_stats"))
            decoded=struct.unpack_from("<10240h",ram,address("library_capture"))
            wet=struct.unpack_from("<512h",ram,address("library_wet"))
            service_us=stats[14]*625/2646; gap_us=stats[13]
            data={"project":str(project),"probe":probe,"stats":stats,"service_max_us":service_us,"gap_max_us":gap_us,
                  "voice_capture_peak":max(map(abs,decoded)),"reverb_buffer_peak":max(map(abs,wet)),"cook":cook}
            tone=fit_tone(decoded)
            tone["expected_hz"]=440*2**(206.25/1200)
            tone["error_cents"]=1200*math.log2(tone["frequency_hz"]/tone["expected_hz"])
            data["captured_tone"]=tone
            if PROFILE:data["intrusive_profile_us"]=[x*625/2646 for x in struct.unpack_from("<8I",ram,address("library_cost"))]
            documents.write_text(art/"evidence.json",json.dumps(data,indent=2))
            (art/"voice1.pcm").write_bytes(struct.pack("<10240h",*decoded))
            print(json.dumps({k:v for k,v in data.items() if k!="cook"},indent=2),flush=True)
            assert probe[0]>100 and probe[1:4]==(1,1,1) and probe[10]==1,probe
            expected=round(4096*22000/44100*2**(206.25/1200))
            assert abs(probe[4]-expected)<=1 and probe[4]==probe[5],(expected,probe)
            assert probe[6]&6==6 and probe[7]&0x80 and probe[8:10]==(1,1),probe
            assert probe[12]==0,"explicit CC91 zero must clear the hardware sends"
            assert max(decoded)>500 and min(decoded)<-500 and max(map(abs,wet))>0,data
            assert tone["relative_rms_error"]<.01 and abs(tone["error_cents"])<=2+1200*math.log2((expected+1)/expected),tone
            assert stats[11]==0 and stats[12]==0 and stats[14]>0,stats
            if not PROFILE:
                assert service_us<=2000 and gap_us<=3000,f"Timing gate failed: service {service_us:.3f} us / 2000, gap {gap_us} us / 3000; {art/'evidence.json'}"
            assert process.wait(timeout=20)==0
            print("DIAGNOSTIC ONLY: intrusive microprofile; timing gate not evaluated" if PROFILE else "PASS EPSB v2 layered notes, RPN pitch, Room memory/send, IRQ budget and lifecycle",flush=True)
        finally:
            if process.poll() is None:process.wait(timeout=45)

if __name__=="__main__":main()
