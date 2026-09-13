"""Native video modes, projection, HUD edges/font and persisted emulator filtering."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import json, pathlib, struct, subprocess, time, urllib.request, zlib

ROOT=pathlib.Path(__file__).resolve().parents[2]
EXE=ROOT/'target/debug/epok-editor.exe'
ART=ROOT/'artifacts'
FLAGS=subprocess.CREATE_NO_WINDOW if hasattr(subprocess,'CREATE_NO_WINDOW') else 0

def request(path,post=False):
    req=urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None)
    with urllib.request.urlopen(req,timeout=3) as response:return response.read()

def main():
    ART.mkdir(exist_ok=True)
    folder=ROOT/'.epok'/('display-verify-'+str(time.time_ns()))
    def run(*args):
        result=subprocess.run([str(EXE),*map(str,args)],capture_output=True,text=True,timeout=120,creationflags=FLAGS)
        with (ART/'display-build.log').open('a',encoding='utf-8') as log:log.write(result.stdout+result.stderr)
        assert result.returncode==0,result.stdout+result.stderr
    run('--create-project',folder,'--name','Display Modes')
    documents.write_text(ART/'display-project.txt', str(folder))
    path=project_manifest(folder)
    settings=documents.loads(path.read_text())
    assert settings['rendering']==dict(width=640,height=480)
    def entity(name,kind='Empty',position=(0,0,0)):
        return dict(name=name,kind=kind,position=position,rotation=[0,0,0],scale=[1,1,1])
    camera=entity('Camera','Camera')
    cube=entity('Cube','Mesh',(0,0,4));cube['material']=dict(color=[0,1,0],unlit=True)
    canvas=entity('Canvas');canvas['canvas']={}
    red=entity('Top Left');red['parent']=2;red['rect']=dict(anchor_min=[0,1],anchor_max=[0,1],pivot=[0,1],position=[2,-2],size=[12,12]);red['image']=dict(color=[1,0,0])
    blue=entity('Bottom Right');blue['parent']=2;blue['rect']=dict(anchor_min=[1,0],anchor_max=[1,0],pivot=[1,0],position=[-2,2],size=[12,12]);blue['image']=dict(color=[0,0,1])
    label=entity('Label');label['parent']=2;label['rect']=dict(anchor_min=[.5,1],anchor_max=[.5,1],pivot=[.5,1],position=[0,-10],size=[80,16]);label['text']=dict(text='EPOK',color=[1,1,1])
    scene=dict(version=1,name='Display Modes',entities=[camera,cube,canvas,red,blue,label])
    documents.write_text(folder/settings['startup_scene'], json.dumps(scene))
    # Include all supported widths, both scan modes, and both extreme aspect ratios.
    for width,height in [(640,480),(320,240),(256,240),(368,240),(512,240),(640,240),(256,480),(320,480),(368,480),(512,480)]:
        settings['rendering']=dict(width=width,height=height);documents.write_text(path, json.dumps(settings))
        with (ART/f'display-{width}x{height}.log').open('w') as log:
            process=subprocess.Popen([str(EXE),'--project',str(folder),'--play-psx','--stop-after','5'],stdout=log,stderr=log,creationflags=FLAGS)
            try:
                deadline=time.monotonic()+40
                while True:
                    assert process.poll() is None,'See display log'
                    try:
                        if json.loads(request('execution-flow'))['running']:break
                    except (OSError,ValueError):pass
                    assert time.monotonic()<deadline
                    time.sleep(.1)
                time.sleep(.8);request('execution-flow?function=pause',True)
                raw=request('gpu/vram/raw')
                def pixel(x,y):return struct.unpack_from('<H',raw,(y*1024+x)*2)[0]&0x7fff
                assert pixel(7,7)==31,(width,height,'top-left HUD',pixel(7,7))
                assert pixel(width-7,height-7)==31<<10,(width,height,'bottom-right HUD')
                assert pixel(width//2,height//2)==31<<5,(width,height,'projection center')
                white=sum(pixel(x,y)==0x7fff for y in range(10,26) for x in range(width//2-40,width//2+40))
                assert white>50,(width,height,'font VRAM overlaps framebuffer',white)
                green=[(x,y) for y in range(height) for x in range(width) if pixel(x,y)==31<<5]
                xs,ys=zip(*green)
                assert abs((max(xs)-min(xs))/width-(max(ys)-min(ys))/height*.75)<.015,(width,height,'projection aspect')
                # Write raw GPU output as a native-resolution PNG, without image resampling.
                rows=bytearray()
                for y in range(height):
                    rows.append(0)
                    for x in range(width):
                        p=pixel(x,y);rows.extend(((p&31)*255//31,((p>>5)&31)*255//31,((p>>10)&31)*255//31))
                def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
                (ART/f'display-{width}x{height}.png').write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>2I5B',width,height,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))
                assert process.wait(timeout=15)==0
            finally:
                if process.poll() is None:process.wait(timeout=45)
        config=documents.loads((folder/'.epok/emulator/pcsx.json').read_text())
        prefs=pathlib.Path.home()/'AppData/Local/Epok/Editor.epokprefs'
        expected=documents.loads(prefs.read_text()).get('emulator_linear_filter',False) if prefs.exists() else False
        assert config['emulator']['LinearFiltering']==expected
        print(f'PASS {width}x{height}: projection, HUD anchors/font, native GPU pixels and emulator filter',flush=True)
    settings['rendering']=dict(width=640,height=480);documents.write_text(path, json.dumps(settings))
    print('PASS all 10 video modes; project:',folder,flush=True)

if __name__=='__main__':main()
