"""Import real MP3, author a CD and inspect decoded XA through PCSX-Redux."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import hashlib, json, pathlib, re, shutil, socket, struct, subprocess, time, urllib.request

ROOT=pathlib.Path(__file__).resolve().parents[2]
EXE=ROOT/'target/debug/epok-editor.exe'
ART=ROOT/'artifacts'
FLAGS=subprocess.CREATE_NO_WINDOW if hasattr(subprocess,'CREATE_NO_WINDOW') else 0


def verify_xa_golden(folder):
    # psxavenc 0.3.1 has process-dependent values in the 20 unused bytes after
    # each audio sector's 18 sound groups (and therefore its EDC). Preserve and
    # report that upstream limitation. Every header, sound group, interleave
    # padding sector and Epok end marker must match the legacy source baseline.
    expected = {
        226592: 'ac2493ce9bd07196681f893036354964c56eb5e8b689870d82d4f470ee3dd24c',
        1067552: '902402226a74a767e07af03aa6a1a70f7a49d1f25e4455d2c780520d0a201788',
    }
    found = {}
    for path in (folder/'.epok/build/music').glob('*.XA'):
        data = path.read_bytes()
        defined = bytearray()
        for at in range(0, len(data), 2336):
            sector = data[at:at + 2336]
            defined.extend(sector[:2312] if sector[2] & 4 and sector[1] == 0 else sector)
        found[len(data)] = hashlib.sha256(defined).hexdigest()
    assert found == expected, ('Legacy XA audio/layout regression', found)

def main():
    ART.mkdir(exist_ok=True)
    folder=ROOT/'.epok'/('bgm-verify-'+str(time.time_ns()))
    documents.write_text(ART/'bgm-project.txt', str(folder))
    def run(*args,success=True):
        result=subprocess.run([str(EXE),*map(str,args)],capture_output=True,text=True,timeout=180,creationflags=FLAGS)
        with (ART/'bgm-build.log').open('a',encoding='utf-8') as log:log.write(result.stdout+result.stderr)
        assert (result.returncode==0)==success,result.stdout+result.stderr
        return result.stdout
    run('--create-project',folder,'--template','sample')
    source=folder/'assets/music.mp3';shutil.copyfile(ROOT/'tests/fixtures/stereo-tone.mp3',source)
    run('--project',folder,'--import-audio','assets/music.mp3','--audio-usage','bgm','--loop')
    run('--project',folder,'--import-audio','assets/music.mp3','--asset','assets/effect.epokasset','--trim-end','0.1','--loop')
    run('--project',folder,'--import-audio','assets/music.mp3','--asset','assets/once.epokasset','--audio-usage','bgm','--rate','18900','--channels','1','--trim-end','0.6')
    records=json.loads(run('--project',folder,'--scan-assets'))['assets']
    ids={r['path']:r['id'] for r in records}
    scene_path=folder/'assets/scenes/SampleScene.epokmap';scene=documents.loads(scene_path.read_text())
    scene['entities'][0]['audio']=dict(clip=ids['assets/music.epokasset'],volume=0.5,priority=200)
    scene['entities'][1]['audio']=dict(clip=ids['assets/effect.epokasset'])
    scene['entities'][2]['audio']=dict(clip=ids['assets/once.epokasset'],play_on_start=False,priority=0)
    documents.write_text(scene_path, json.dumps(scene))
    documents.write_text(folder/'assets/scripts/Spinner.cpp', r'''
#include "Spinner.hpp"
#include "common/hardware/spu.h"
#include "common/hardware/dma.h"
uint32_t bgm_probe[18]={};
alignas(4) int16_t bgm_capture[1024]={};
uint32_t control_stage=0,control_frame=0,control_starts=0;
void Spinner::start(epok::Transform&){}
void Spinner::update(epok::Transform&,epok::Fixed){
    ++bgm_probe[0];
    auto& m=epok::music_stats;
    bgm_probe[1]=m.state;bgm_probe[2]=m.starts;bgm_probe[3]=m.ends;bgm_probe[4]=m.loops;bgm_probe[5]=m.errors;
    bgm_probe[12]=m.error_code;
    if(m.state==4 && ++bgm_probe[6]==60){
        uint16_t ctrl=SPU_CTRL;SPU_RAM_DTA=0;SPU_CTRL=0xc031;
        DMA_CTRL[DMA_SPU].MADR=(uint32_t)bgm_capture;DMA_CTRL[DMA_SPU].BCR=(32<<16)|16;DMA_CTRL[DMA_SPU].CHCR=0x01000200;
        uint32_t timeout=1000000;while((DMA_CTRL[DMA_SPU].CHCR&0x01000000) && --timeout){}
        bgm_probe[7]=timeout!=0;SPU_CTRL=ctrl;
        bgm_probe[8]=SPU_VOICES[0].currentVolume;bgm_probe[9]=SPU_VOICES[1].currentVolume;
        bgm_probe[10]=SPU_VOL_CD_LEFT;bgm_probe[11]=SPU_CTRL;
        // An idle ADSR envelope need not be zero (firmware may have used the
        // voice). Audible output depends on its channel gains.
        bgm_probe[14]=SPU_VOICES[1].volumeLeft|SPU_VOICES[1].volumeRight;
        for(int i=0;i<24;++i)if(SPU_VOICES[i].volumeLeft||SPU_VOICES[i].volumeRight)++bgm_probe[15];
        bgm_probe[16]=epok::find_entity(EFFECT_ENTITY)->audio.is_playing();
        bgm_probe[17]=epok::find_entity(ONCE_ENTITY)->audio.is_playing();
    }
    auto& track=epok::find_entity(MUSIC_ENTITY)->audio;
    auto& once=epok::find_entity(ONCE_ENTITY)->audio;
    if(control_stage==0 && m.loops>=1 && track.is_playing()) {track.stop();control_stage=1;}
    else if(control_stage==1 && m.state==2 && !track.is_playing()) {bgm_probe[13]|=1;track.play();control_stage=2;}
    else if(control_stage==2 && track.is_playing()) {bgm_probe[13]|=2;control_starts=m.starts;track.play();control_stage=3;}
    else if(control_stage==3 && m.starts>control_starts && track.is_playing()) {bgm_probe[13]|=4;once.play();control_frame=bgm_probe[0];control_stage=4;}
    else if(control_stage==4 && bgm_probe[0]>control_frame+20) {
        if(track.is_playing() && !once.is_playing())bgm_probe[13]|=8;
        once.priority=255;once.play();control_stage=5;
    }
    else if(control_stage==5 && once.is_playing()) {bgm_probe[13]|=16;control_stage=6;}
    else if(control_stage==6 && m.state==2 && !once.is_playing()) {bgm_probe[13]|=32;control_stage=7;}
}
'''.replace('MUSIC_ENTITY',json.dumps(scene['entities'][0]['name'])).replace('ONCE_ENTITY',json.dumps(scene['entities'][2]['name'])).replace('EFFECT_ENTITY',json.dumps(scene['entities'][1]['name'])))
    # Recover the music from its portable package with the editor closed and no source/cache.
    assert not (folder/'.epok/imported').exists(), 'Import must not cook a target'
    run('--project',folder,'--build-psx')
    moved=folder/'assets/moved.epokasset';(folder/'assets/music.epokasset').rename(moved);source.unlink()
    cache=(folder/'.epok/imported').resolve()
    assert cache.is_relative_to(folder.resolve()) and folder.resolve().is_relative_to(ROOT.resolve())
    shutil.rmtree(cache)
    run('--project',folder,'--build-psx')
    saved=moved.read_bytes()
    (folder/'assets/broken.mp3').write_bytes(b'not audio')
    run('--project',folder,'--reimport-asset','assets/moved.epokasset','--import-audio','assets/broken.mp3',success=False)
    assert moved.read_bytes()==saved,'Failed reimport changed the existing package'
    run('--project',folder,'--reimport-asset','assets/moved.epokasset','--snapshot','--rate','37800')
    records=json.loads(run('--project',folder,'--scan-assets'))['assets']
    assert any(r['path']=='assets/moved.epokasset' and r['id']==ids['assets/music.epokasset'] for r in records)
    scene['entities'][0]['audio']['pitch']=0.5;documents.write_text(scene_path, json.dumps(scene))
    run('--project',folder,'--build-psx',success=False)
    scene['entities'][0]['audio']['pitch']=1
    scene['entities'][2]['audio']['play_on_start']=True;documents.write_text(scene_path, json.dumps(scene))
    run('--project',folder,'--build-psx',success=False)
    scene['entities'][2]['audio']['play_on_start']=False;documents.write_text(scene_path, json.dumps(scene))
    build=folder/'.epok/build'
    verify_xa_golden(folder)
    assert (build/'epok.cue').is_file() and (build/'epok.bin').stat().st_size>500000
    assert (build/'epok.ps-exe').stat().st_size<512000
    print('PASS MP3 -> SFX and XA, offline move/source deletion, cache recovery, BIN/CUE authoring',flush=True)
    with socket.socket() as port:port.bind(('127.0.0.1',8077))
    def request(path,post=False):
        req=urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None)
        with urllib.request.urlopen(req,timeout=3) as response:return response.read()
    with (ART/'bgm-emulator.log').open('w') as log:
        process=subprocess.Popen([str(EXE),'--project',str(folder),'--play-psx','--stop-after','45'],stdout=log,stderr=log,creationflags=FLAGS)
        try:
            deadline=time.monotonic()+60
            while True:
                assert process.poll() is None,'Build/boot failed: artifacts/bgm-emulator.log'
                try:
                    if json.loads(request('execution-flow'))['running']:break
                except (OSError,ValueError):pass
                assert time.monotonic()<deadline,'Emulator did not start'
                time.sleep(0.1)
            symbols=(build/'epok.map').read_text()
            def address(name):return int(re.search(r'0x([0-9a-f]+)\s+'+name+r'\b',symbols)[1],16)&0x1fffff
            # Fixed simulation ticks and CD progress follow emulated time, not
            # host wall time. Wait for the observable final one-shot completion.
            deadline=time.monotonic()+35
            while True:
                ram=request('cpu/ram/raw')
                values=struct.unpack_from('<18I',ram,address('bgm_probe'))
                if values[13]==63 or values[5]:break
                assert process.poll() is None,'Emulator exited before BGM scenario completed'
                assert time.monotonic()<deadline,('BGM scenario timed out',values)
                time.sleep(0.25)
            request('execution-flow?function=pause',True)
            ram=request('cpu/ram/raw')
            values=struct.unpack_from('<18I',ram,address('bgm_probe'))
            documents.write_text(ART/'bgm-probe.json', json.dumps(values))
            assert values[2]>=2 and values[3]>=1 and values[4]>=1 and values[5]==0,('XA start/end/loop/errors',values)
            assert values[7]==1 and values[8]>0 and values[14]==0 and values[15]==1,('Capture DMA / separate SFX voices',values)
            assert values[16]==1 and values[17]==0,('SFX ownership / inactive one-shot',values)
            assert values[13]==63,('Stop/restart/retrigger/priority/switch/one-shot',values)
            decoded=struct.unpack_from('<1024h',ram,address('bgm_capture'))
            for channel in (decoded[:512],decoded[512:]):
                assert min(channel)<-1000 and max(channel)>1000,('No decoded XA audio',max(map(abs,channel)),values)
            assert decoded[:512]!=decoded[512:],'Stereo channels collapsed'
            (ART/'bgm-capture.pcm').write_bytes(struct.pack('<1024h',*decoded))
            assert process.wait(timeout=50)==0
            print('PASS CD boot, stereo XA decode, loop, SPU effect, stop/restart/retrigger, priority and mono one-shot switch',flush=True)
        finally:
            if process.poll() is None:process.wait(timeout=45)

if __name__=='__main__':main()
