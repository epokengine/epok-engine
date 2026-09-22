"""Native HUD regression: authored components == C++ creation, then animate via get<T>()."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import json
import pathlib
import socket
import struct
import subprocess
import time
import urllib.request
import hashlib
import uuid
from imaging import png

ROOT=pathlib.Path(__file__).resolve().parents[2]
ART=ROOT/'artifacts'
ART.mkdir(exist_ok=True)
FLAGS=subprocess.CREATE_NO_WINDOW if hasattr(subprocess,'CREATE_NO_WINDOW') else 0

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--exe',type=pathlib.Path,default=ROOT/'target/debug/epok-editor.exe');args=parser.parse_args()
    with socket.socket() as probe:probe.bind(('127.0.0.1',8077))
    folder=ROOT/'.epok'/('hud-verify-'+str(time.time_ns()))
    subprocess.run([str(args.exe.resolve()), '--create-project', str(folder)], check=True, creationflags=FLAGS)
    settings_path = project_manifest(folder)
    settings = documents.loads(settings_path.read_text())
    settings['rendering'] = dict(width=320, height=240)  # Fixed pixel-reference fixture.
    settings['startup_scene'] = 'assets/scenes/SampleScene.epokmap'
    documents.write_text(settings_path, json.dumps(settings))

    def entity(name,parent=None):return dict(name=name,kind='Empty',parent=parent,position=[0,0,0],rotation=[0,0,0],scale=[1,1,1])
    def rect(x,y,w,h):return dict(anchor_min=[0,1],anchor_max=[0,1],pivot=[0,1],position=[x,y],size=[w,h])
    canvas=entity('Canvas');canvas['canvas']={'enabled':True}
    panel=entity('Panel',0);panel['rect']=rect(12,-12,180,64);panel['image']={'color':[0,0,1]}
    text=entity('Label',1);text['rect']=rect(8,-6,160,16);text['text']={'text':'HUD 075','color':[1,1,1]}
    bar=entity('Health',1);bar['rect']=rect(8,-32,160,16);bar['progress']={'value':0.75,'color':[0,1,0],'background':[1,0,0]}
    authored=[canvas,panel,text,bar]
    script=folder/'assets/scripts'
    documents.write_text(script/'HudDemo.hpp', '''#pragma once
#include "epok.hpp"
class HudDemo final:public epok::Behaviour {
    epok::Fixed phase=0.0;
public:
    epok::Fixed animate=0.0;
    void start(epok::Transform&) override;
    void update(epok::Transform&,epok::Fixed dt) override;
};
''')
    documents.write_text(script/'HudDemo.epokscript', json.dumps(dict(name='HudDemo',properties=[dict(name='animate',default=0)])))
    documents.write_text(script/'HudDemo.cpp', '''#include "HudDemo.hpp"
using namespace epok;
static void rect(Entity& e,Fixed x,Fixed y,Fixed w,Fixed h){
    auto& r=e.add<RectTransform>();r.anchor_min[0]=r.anchor_max[0]=r.pivot[0]=0.0;
    r.anchor_min[1]=r.anchor_max[1]=r.pivot[1]=1.0;r.position[0]=x;r.position[1]=y;r.size[0]=w;r.size[1]=h;
}
void HudDemo::start(Transform&){
    if(find_entity("Canvas"))return;
    auto* c=create_entity("Canvas");if(!c)return;c->add<Canvas>();
    auto* p=create_entity("Panel",c);if(!p)return;rect(*p,12.0,-12.0,180.0,64.0);auto& image=p->add<Image>();image.color[0]=image.color[1]=0;image.color[2]=255;
    auto* t=create_entity("Label",p);if(!t)return;rect(*t,8.0,-6.0,160.0,16.0);t->add<Text>().set_text("HUD 075");
    auto* b=create_entity("Health",p);if(!b)return;rect(*b,8.0,-32.0,160.0,16.0);auto& bar=b->add<ProgressBar>();bar.value=0.75;bar.color[0]=bar.color[2]=0;bar.color[1]=255;bar.background[0]=255;bar.background[1]=bar.background[2]=0;
}
void HudDemo::update(Transform&,Fixed dt){
    if(animate.raw()==0)return;
    phase+=dt;if(phase.raw()>4096)phase-=1.0;
    if(auto* bar=entity().get<ProgressBar>())bar->value=phase;
    if(auto* label=find_entity("Label"))if(auto* text=label->get<Text>())text->set_text(phase.raw()>2048?"HIGH":"LOW");
}
''')
    def write_scene(entities):
        documents.write_text(folder/'assets/scenes/SampleScene.epokmap', json.dumps(dict(version=1,name='HUD verification',entities=entities),indent=2))
    def request(path,post=False):
        req=urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None)
        with urllib.request.urlopen(req,timeout=2) as r:return r.read()
    def screen(raw):return b''.join(raw[y*2048:y*2048+640] for y in range(240))
    def capture(entities,label,animated=False):
        write_scene(entities)
        with (ART/f'hud-{label}.log').open('w') as log:
            process=subprocess.Popen([str(args.exe.resolve()),'--project',str(folder),'--play-psx','--stop-after','7'],stdout=log,stderr=log,creationflags=FLAGS)
            try:
                deadline=time.monotonic()+30
                while True:
                    if process.poll() is not None:raise AssertionError(f'Build/boot failed: hud-{label}.log')
                    try:
                        state=json.loads(request('execution-flow'))
                        if not state['running']:request('execution-flow?function=resume',True)
                        break
                    except (OSError,ValueError):pass
                    if time.monotonic()>deadline:raise TimeoutError('Emulator did not start')
                    time.sleep(0.1)
                time.sleep(1.3);request('execution-flow?function=pause',True);first=screen(request('gpu/vram/raw'))
                colors=struct.unpack('<76800H',first)
                assert colors.count(31<<5)>400,'No green ProgressBar in VRAM'
                assert colors.count(31<<10)>2000,'No blue Panel in VRAM'
                assert colors.count(32767)>20,'No HUD text in VRAM'
                if animated:
                    request('execution-flow?function=resume',True);time.sleep(0.35);request('execution-flow?function=pause',True)
                    second=screen(request('gpu/vram/raw'));assert first!=second,'C++ component updates did not change HUD'
                assert process.wait(timeout=12)==0
                return first
            finally:
                if process.poll() is None:process.wait(timeout=40)
    first=capture(authored,'authored')
    controller=entity('Controller');controller['script']=dict(name='HudDemo',properties={'animate':0})
    second=capture([controller],'cpp-created')
    assert first==second,'Authored and C++-created HUD differ'
    bar['script']=dict(name='HudDemo',properties={'animate':1})
    capture(authored,'animated',True)
    bar.pop('script');write_scene(authored)
    # Exercise atlas sampling, fixed nine-slice borders, Spanish glyphs,
    # character wrapping and screen-edge clipping in the actual PSX renderer.
    ident=str(uuid.uuid4())
    colors=[(255,0,0),(0,255,0),(0,0,255),
            (255,0,255),(255,255,0),(0,255,255),
            (0,0,255),(0,255,0),(255,0,0)]
    def cell(v):return 0 if v<2 else 2 if v>=6 else 1
    rgba=bytes(c for y in range(8) for x in range(8) for c in (*colors[cell(y)*3+cell(x)],255))
    source='assets/textures/nine-slice.png'
    texture=png(8,8,rgba)
    (folder/'assets/textures').mkdir(exist_ok=True)
    (folder/source).write_bytes(texture)
    meta=json.dumps(dict(version=2,id=ident,kind='Texture',importer_version=1,source=source,
                         source_hash=hashlib.sha256(texture).hexdigest(),settings={'type':'Texture'})).encode()
    (folder/'assets/textures/nine-slice.epokasset').write_bytes(b'EPOKAS01'+struct.pack('<II',len(meta),len(texture))+meta+texture)
    sliced=entity('Nine slice',0);sliced['rect']=rect(210,-12,80,64)
    sliced['image']=dict(texture=ident,borders=[2,2,2,2],color=[1,1,1])
    spanish=entity('Spanish wrap',0);spanish['rect']=rect(12,-100,32,32)
    spanish['text']=dict(text='ÁÑü¿¡a',wrap=True,color=[1,1,1])
    clipped=entity('Clipped image',0);clipped['rect']=rect(310,-100,40,40)
    clipped['image']=dict(texture=ident,color=[1,1,1])
    native=capture(authored+[sliced,spanish,clipped],'atlas-spanish')
    def pixel(x,y):return struct.unpack_from('<H',native,(y*320+x)*2)[0]&0x7fff
    for x,y,rgb in [(210,12,colors[0]),(250,12,colors[1]),(289,12,colors[2]),
                    (210,40,colors[3]),(250,40,colors[4]),(289,40,colors[5]),
                    (210,75,colors[6]),(250,75,colors[7]),(289,75,colors[8])]:
        expected=(rgb[0]//8)|((rgb[1]//8)<<5)|((rgb[2]//8)<<10)
        assert pixel(x,y)==expected,('Nine-slice border/center',x,y,pixel(x,y),expected)
    cells=[bytes(pixel(12+col*8+x,100+row*16+y)==32767 for y in range(16) for x in range(8))
           for row,col in [(0,0),(0,1),(0,2),(0,3),(1,0),(1,1)]]
    assert all(any(glyph) for glyph in cells),'Spanish glyph/wrapped second row missing'
    assert len(set(cells))==6,'Spanish glyphs substituted or collapsed'
    assert pixel(319,105)!=pixel(300,105),'Screen-edge image clipping lost the visible part'
    with socket.socket() as probe:probe.bind(('127.0.0.1',8077))
    report='PASS native Panel, Text and ProgressBar\nPASS authored HUD equals C++ create_entity/add<T> pixel-for-pixel\nPASS entity().get<T>() and find_entity() animate native HUD\nPASS textured atlas, nine-slice borders, six distinct Spanish glyphs, wrapping and clipping\nPASS emulator cleanup\nFixture: '+str(folder)+'\n'
    documents.write_text(ART/'hud-verification.txt', report);print(report)

if __name__=='__main__':main()
