"""Real FBX -> portable assets -> MIPS -> PCSX-Redux animation verification.
Requires the configured local toolchain/emulator. Leaves an isolated review project.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import copy, json, os, pathlib, re, shutil, struct, subprocess, time, urllib.request, uuid
ROOT=pathlib.Path(__file__).resolve().parents[2]
# Only Windows builds carry the .exe suffix; macOS and Linux use the bare name.
EXE=ROOT/'target/debug'/('epok-editor.exe' if os.name=='nt' else 'epok-editor')
ART=ROOT/'artifacts'
FLAGS=getattr(subprocess,'CREATE_NO_WINDOW',0)
def package(path):
 b=path.read_bytes();ms,ss=struct.unpack_from('<II',b,8);return json.loads(b[16:16+ms]),b[16+ms:]
def run(*args):
 p=subprocess.run([str(EXE),*map(str,args)],capture_output=True,text=True,timeout=120,creationflags=FLAGS)
 with (ART/'skeletal-build.log').open('a',encoding='utf-8') as log:log.write(p.stdout+p.stderr)
 assert p.returncode==0,p.stdout+p.stderr
 return p.stdout
def request(path,post=False):
 req=urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None)
 with urllib.request.urlopen(req,timeout=3) as response:return response.read()
def expected_poses(assets,mesh_id,clip_id):
 data={m['id']:json.loads(s)['data'] for p,m,s in assets if m['kind']!='ModelSource'}
 mesh=data[mesh_id];skeleton=data[mesh['skeleton']];clip=data[clip_id];out=[]
 for frame in range(clip['frames']):
  matrices=[]
  for index,bone in enumerate(skeleton['bones']):
   track=clip['tracks'][index];p=track[0 if len(track)==1 else frame]
   x,y,z,w=[v/4096 for v in p['rotation']];s=[v/4096 for v in p['scale']]
   m=[[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w),0],[2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w),0],[2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y),0]]
   for r in range(3):
    for c in range(3):m[r][c]*=s[c]
    m[r][3]=p['translation'][r]/256
   if bone['parent']>=0:
    a=matrices[bone['parent']];m=[[sum(a[r][k]*m[k][c] for k in range(3))+(a[r][3] if c==3 else 0) for c in range(4)] for r in range(3)]
   matrices.append(m)
  out.append([sum(matrices[v['bone']][r][c]*v['position'][c] for c in range(3))+matrices[v['bone']][r][3]*4096 for v in mesh['vertices'] for r in range(3)])
 return out
def main():
 ART.mkdir(exist_ok=True)
 folder=ROOT/'.epok'/('skeletal-verify-'+str(time.time_ns()))
 documents.write_text(ART/'skeletal-project.txt', str(folder))
 run('--create-project',folder,'--name','Skeletal Preview','--template','basic')
 video_path=project_manifest(folder)
 video=documents.loads(video_path.read_text());video['rendering']=dict(width=320,height=240)
 documents.write_text(video_path, json.dumps(video))  # Fixed pixel-reference fixture.

 shutil.copyfile(ROOT/'resources/models/EpokMannequin.fbx',folder/'assets/EpokMannequin.fbx')
 run('--project',folder,'--import-fbx','assets/EpokMannequin.fbx','--animation-storage','rigid-gte')
 assets=[(p,*package(p)) for p in folder.glob('assets/**/*.epokasset')]
 mesh=next(m['id'] for p,m,s in assets if m['kind']=='SkeletalMesh')
 clip=next(m['id'] for p,m,s in assets if m['kind']=='AnimationClip' and 'Walk' in json.loads(s)['data']['name'])
 materials=[json.loads(s)['data']['color'] for p,m,s in assets if m['kind']=='Material']
 assert len(set(tuple(c) for c in materials))>=3,materials
 scene_path=folder/'assets/scenes/Main.epokmap'
 scene=documents.loads(scene_path.read_text())
 def component(actor,suffix):return next(c for c in actor['components'] if c['class']['name'].endswith(suffix))
 camera=next(a for a in scene['actors'] if any(c['class']['name'].endswith('Camera3DComponent') for c in a['components']))
 camera_transform=component(camera,'SceneComponent3D')
 camera_transform['properties'].update(position=[2.2,1.45,-3.2],rotation=[7,-34,0],scale=[1,1,1])
 sample=documents.loads((ROOT/'examples/sample-game/assets/scenes/SampleScene.epokmap').read_text())
 character=copy.deepcopy(next(a for a in sample['actors'] if any(c['class']['name'].endswith('Mesh3DComponent') for c in a['components'])))
 character['id']=str(uuid.uuid4())
 character['components']=[c for c in character['components'] if c['class']['name'].endswith(('SceneComponent3D','Mesh3DComponent'))]
 for c in character['components']:c['id']=str(uuid.uuid4())
 character['name']='Character'
 transform=component(character,'SceneComponent3D')
 transform['properties'].update(position=[0,0,0],rotation=[0,180,0],scale=[1,1,1])
 renderer=component(character,'Mesh3DComponent')
 renderer['properties']['skeletal_mesh']=dict(asset=mesh,clip=clip,looping=True,play_on_start=True)
 if 'skeletal_mesh' not in renderer['overrides']:renderer['overrides'].append('skeletal_mesh')
 scene['actors']=[camera,character]
 documents.write_text(scene_path, json.dumps(scene,indent=2))
 documents.write_text(folder/'UserSettings/SceneView.epokprefs', json.dumps(dict(center=[0,.85,0],yaw=-.45,pitch=.18,zoom=2.5)))
 run('--project',folder,'--build-psx')
 generated=(folder/'.epok/build/scene.hh').read_text()
 assert 'SkeletalStorage::RigidGte' in generated
 run('--project',folder,'--preview-model','assets/EpokMannequin.imported/Model.epokasset','--screenshot',ART/'skeletal-import-preview.png')
 def play(mode,validate_vertices):
  vertices=[];screens=[]
  with (ART/f'skeletal-emulator-{mode}.log').open('w') as log:
   process=subprocess.Popen([str(EXE),'--project',str(folder),'--play-psx','--stop-after','15'],stdout=log,stderr=log,creationflags=FLAGS)
   try:
    deadline=time.monotonic()+40
    while True:
     assert process.poll() is None,f'See skeletal-emulator-{mode}.log'
     try:
      if json.loads(request('execution-flow'))['running']:break
     except (OSError,ValueError):pass
     assert time.monotonic()<deadline
     time.sleep(.1)
    address=None
    if validate_vertices:
     symbols=(folder/'.epok/build/epok.map').read_text()
     matches=re.findall(r'0x([0-9a-f]+)\s+epok::skeletal_detail::scratch\b',symbols)
     assert matches,'Skeletal scratch symbol missing'
     address=int(matches[0],16)&0x1fffff
    for delay in [1.0,.43]:
     time.sleep(delay);request('execution-flow?function=pause',True)
     if address is not None:
      ram=request('cpu/ram/raw');vertices.append(ram[address+64*48:address+64*48+96*6])
     vram=request('gpu/vram/raw');screens.append(b''.join(vram[y*2048:y*2048+640] for y in range(240)))
     request('execution-flow?function=resume',True)
    assert screens[0]!=screens[1],f'{mode} animation did not reach the GPU'
    for screen in screens:
     pixels=struct.unpack('<76800H',screen)
     assert sum(1 for v in pixels if ((v>>10)&31)>18 and (v&31)<8)>200,'Blue character is missing'
    assert process.wait(timeout=25)==0
   finally:
    if process.poll() is None:process.wait(timeout=45)
  return vertices
 rigid_size=(folder/'.epok/build/epok.ps-exe').stat().st_size
 play('rigid-gte',False)
 run('--project',folder,'--reimport-asset','assets/EpokMannequin.imported/Model.epokasset','--animation-storage','baked-vertices')
 run('--project',folder,'--build-psx')
 generated=(folder/'.epok/build/scene.hh').read_text()
 assert 'SkeletalStorage::BakedVertices' in generated and 'skin_vertex_data_' in generated
 vertices=play('baked-vertices',True)
 assert vertices[0]!=vertices[1],'Baked vertex playback did not deform the mesh'
 expected=expected_poses(assets,mesh,clip)
 errors=[min(max(abs(a-b) for a,b in zip(struct.unpack('<288h',raw),pose)) for pose in expected) for raw in vertices]
 assert max(errors)<48,('Decoded baked pose differs from editor quantized pose',errors)
 (ART/'skeletal-native-poses.bin').write_bytes(b''.join(vertices))
 report=dict(vertices=96,triangles=144,clips=['Idle','Walk'],rigid_gte_frame_changed=True,baked_frame_changed=True,max_baked_position_error_meters=max(errors)/4096,rigid_ps_exe_bytes=rigid_size,baked_ps_exe_bytes=(folder/'.epok/build/epok.ps-exe').stat().st_size)
 documents.write_text(ART/'skeletal-verification.json', json.dumps(report,indent=2))
 run('--project',folder,'--screenshot-game','--screenshot',ART/'skeletal-psx.png')
 print('PASS FBX import, rigid GTE animation, compressed baked animation and GPU playback:',folder)
if __name__=='__main__':main()
