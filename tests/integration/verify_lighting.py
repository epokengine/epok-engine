"""Build isolated PSX scenes and verify lighting using the emulator's actual VRAM."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse,copy,json,pathlib,re,socket,struct,subprocess,time,urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[2]
ART=ROOT/'artifacts'
ART.mkdir(exist_ok=True)
FLAGS=subprocess.CREATE_NO_WINDOW if hasattr(subprocess,'CREATE_NO_WINDOW') else 0
def entity(name,kind='Mesh',position=(0,0,0),rotation=(0,0,0),scale=(1,1,1)):
    return dict(name=name,kind=kind,position=position,rotation=rotation,scale=scale,material=dict(color=[1,1,1],unlit=False))
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--exe',type=pathlib.Path,default=ROOT/'target/debug/epok-editor.exe');args=parser.parse_args()
    with socket.socket() as probe:probe.bind(('127.0.0.1',8077))
    folder=ROOT/'.epok'/('lighting-verify-'+str(time.time_ns()))
    subprocess.run([str(args.exe.resolve()), '--create-project', str(folder)], check=True, creationflags=FLAGS)
    settings_path = project_manifest(folder)
    settings = documents.loads(settings_path.read_text())
    settings['rendering'] = dict(width=320, height=240)  # Fixed pixel-reference fixture.
    settings['startup_scene'] = 'assets/scenes/SampleScene.epokmap'
    documents.write_text(settings_path, json.dumps(settings))

    camera=entity('Main Camera','Camera',(0,1,-6),(8,0,0))
    cube=entity('Receiver',position=(0,0,0),rotation=(0,20,0))
    sun=entity('Sun','Empty',rotation=(30,-25,0));sun['light']=dict(kind='Directional',mode='Mixed',intensity=0.6,color=[1,0.7,0.4])
    base=dict(version=1,name='Lighting verification',environment=dict(ambient=[0.1]*3),entities=[camera,cube,sun])
    def request(path,post=False):
        req=urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None)
        with urllib.request.urlopen(req,timeout=3) as response:return response.read()
    def screen(raw):return b''.join(raw[y*2048:y*2048+640] for y in range(240))
    stats={}
    def capture(scene,label):
        documents.write_text(folder/'assets/scenes/SampleScene.epokmap', json.dumps(scene,indent=2))
        with (ART/f'lighting-{label}.log').open('w') as log:
            p=subprocess.Popen([str(args.exe.resolve()),'--project',str(folder),'--play-psx','--stop-after','5'],stdout=log,stderr=log,creationflags=FLAGS)
            try:
                deadline=time.monotonic()+40
                while True:
                    if p.poll() is not None:raise AssertionError(f'Build/boot failed: lighting-{label}.log')
                    try:
                        if not json.loads(request('execution-flow'))['running']:request('execution-flow?function=resume',True)
                        break
                    except (OSError,ValueError):pass
                    if time.monotonic()>deadline:raise TimeoutError('Emulator did not start')
                    time.sleep(0.1)
                time.sleep(1.3);request('execution-flow?function=pause',True)
                raw=screen(request('gpu/vram/raw'));(ART/f'lighting-{label}.vram').write_bytes(raw)
                symbols=(folder/'.epok/build/epok.map').read_text()
                address=int(re.search(r'0x([0-9a-f]+)\s+epok::lighting_stats',symbols)[1],16)&0x1fffff
                ram=request('cpu/ram/raw');stats[label]=struct.unpack_from('<9I',ram,address)
                assert stats[label][5]==0,'Triangle pool exhausted'
                assert p.wait(timeout=15)==0
                return struct.unpack('<76800H',raw)
            finally:
                if p.poll() is None:p.wait(timeout=45)
    reports=[]
    dynamic=capture(base,'directional-gte')
    baked=copy.deepcopy(base);baked['entities'][1]['lighting']=dict(receive='Baked',static_geometry=True)
    offline=capture(baked,'directional-baked')
    def rgb(pixel):return [pixel&31,(pixel>>5)&31,(pixel>>10)&31]
    background=dynamic[0]
    occupied=[i for i,v in enumerate(dynamic) if v!=background]
    assert len(occupied)>500,'No lit object rendered'
    error=max(abs(a-b) for i in occupied for a,b in zip(rgb(dynamic[i]),rgb(offline[i])))
    assert error<=2,f'GTE and baked colors diverge: {error}/31'
    reports.append(f'PASS baked vs GTE directional lighting: max channel error {error}/31')
    assert stats['directional-baked'][2]==0 and stats['directional-gte'][2]==6
    reports.append('PASS baked renderer performs zero GTE lighting operations; dynamic cube uses six normals')
    # Additional directional sources do not multiply the target work or brightness.
    crowded=copy.deepcopy(base);crowded['entities'].extend([copy.deepcopy(sun) for _ in range(7)])
    many=capture(crowded,'bounded-directionals');assert dynamic==many
    reports.append('PASS eight directional lights retain the one-directional-per-object budget')
    inherited=copy.deepcopy(base)
    parent=entity('Nonuniform parent','Empty',rotation=(10,0,25),scale=(1.5,0.8,1.2))
    inherited['entities'].append(parent);inherited['entities'][1]['parent']=3
    lit_inherited=capture(inherited,'inherited-gte')
    inherited['entities'][1]['lighting']=dict(receive='Baked',static_geometry=True)
    baked_inherited=capture(inherited,'inherited-baked')
    error=max(abs(a-b) for v,w in zip(lit_inherited,baked_inherited) for a,b in zip(rgb(v),rgb(w)))
    assert error<=2,f'Inverse-transpose normals diverge under inherited scale/shear: {error}'
    reports.append(f'PASS inherited rotation / nonuniform scale normals: max baked/GTE error {error}/31')
    point=entity('Point','Empty',(0,0,-2));point['light']=dict(kind='Point',mode='Realtime',color=[0,0,1],intensity=1,range=6)
    local=copy.deepcopy(base);local['entities']=[camera,cube,point]
    blue=capture(local,'point');assert sum(1 for v in blue if rgb(v)[2]>rgb(v)[0]+5)>500
    far=copy.deepcopy(local);far['entities'][2]['position']=[0,0,-20]
    dark=capture(far,'point-outside-range');assert blue!=dark
    reports.append('PASS point attenuation and range rejection in PSX VRAM')
    # Prove the component API creates the same light as the scene exporter.
    script=folder/'assets/scripts'
    documents.write_text(script/'CreateLight.epokscript', json.dumps(dict(name='CreateLight',properties=[])))
    documents.write_text(script/'CreateLight.hpp', '#pragma once\n#include "epok.hpp"\nclass CreateLight: public epok::Behaviour {public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override{};};\n')
    documents.write_text(script/'CreateLight.cpp', '''#include "CreateLight.hpp"
using namespace epok;
void CreateLight::start(Transform&){auto* e=create_entity("Point");if(!e)return;e->transform.position[2]=-2.0;auto& l=e->add<Light>();l.type=LightType::Point;l.color[0]=l.color[1]=0;l.color[2]=255;l.intensity=1.0;l.range=6.0;}
''')
    native=copy.deepcopy(local);native['entities']=native['entities'][:2];native['entities'][1]['script']=dict(name='CreateLight',properties={})
    created=capture(native,'cpp-created');assert created==blue
    reports.append('PASS C++ create_entity / add<Light>() matches authored light pixel for pixel')
    # Baked shadows modify the ground while the dynamic receiver remains lit.
    shadow=copy.deepcopy(base);shadow['entities'][1]['lighting']=dict(static_geometry=True)
    floor=entity('Ground',position=(0,-0.65,0),scale=(5,0.2,5));floor['lighting']=dict(static_geometry=True,receive='Baked')
    shadow['entities'].append(floor);shadow['entities'][2]['rotation']=[65,0,0]
    cast=capture(shadow,'baked-shadow');shadow['entities'][2]['light']['shadows']=False
    no_shadow=capture(shadow,'no-shadow');assert sum(a!=b for a,b in zip(cast,no_shadow))>30
    reports.append('PASS static occluder changes baked shadow in PSX VRAM')
    blob_scene=copy.deepcopy(shadow);blob_scene['entities'][1]['blob_shadow']=dict(radius=1.0,strength=0.5)
    blob=capture(blob_scene,'blob-shadow');assert sum(a!=b for a,b in zip(blob,no_shadow))>30
    assert stats['blob-shadow'][6]==8,stats['blob-shadow']
    reports.append('PASS moving blob shadow renders with eight triangles on a horizontal floor')
    shadow['entities'][2]['light']['shadows']=True
    documents.write_text(folder/'assets/scenes/SampleScene.epokmap', json.dumps(shadow,indent=2))
    subprocess.run([str(args.exe.resolve()),'--project',str(folder),'--bake-lighting'],check=True,creationflags=FLAGS)
    cached=documents.loads((folder/'assets/scenes/SampleScene.epokmap').read_text());assert cached['bake']['colors'][3]
    reports.append('PASS offline bake cache saved with scene')
    with socket.socket() as probe:probe.bind(('127.0.0.1',8077))
    reports.append('PASS owned emulator cleanup');reports.append(f'Fixture: {folder}')
    for label in ('directional-baked','directional-gte','point','blob-shadow'):
        reports.append(f'PSX counters {label}: {stats[label][7]} lighting scanlines, {stats[label][8]} CPU frame scanlines (~64 us/line; excludes GPU wait)')
    report='\n'.join(reports)+'\n';documents.write_text(ART/'lighting-verification.txt', report);print(report)
if __name__=='__main__':main()
