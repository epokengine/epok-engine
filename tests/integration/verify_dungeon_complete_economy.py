"""PSX integration test with bounded, state-driven controller edge injection.

Only validation RAM is patched: pad edges and player position. All currency and
items must go through the real link, inventory, shop and memory-card services.
Original memory cards are restored after both test runs.
"""
from pathlib import Path
import json,os,re,shutil,socket,struct,subprocess,sys,time,urllib.request,yaml
import argparse,struct
import dungeon_complete_capture as capture
ENGINE=Path(__file__).resolve().parents[2]
EDITOR=ENGINE/'target/debug'/('epok-editor.exe' if os.name=='nt' else 'epok-editor')
NM=ENGINE/('.tools/mips/bin/mipsel-none-elf-nm'+('.exe' if os.name=='nt' else ''))
sys.path.insert(0,str(ENGINE/'tools'))
import epok_documents as documents
from package_dungeon_complete import asset
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--project',type=Path,required=True)
parser.add_argument('--fresh-install',action='store_true')
parser.add_argument('--detail',action='store_true')
args=parser.parse_args()
P=args.project.resolve(strict=True)
manifest=documents.loads(next(P.glob('*.epokproject')).read_text())
assert manifest['play']['target']=='embedded' and manifest['play']['data']=='host', 'Use the template emulator profile, never physical serial hardware'
capture.P=P
capture.OUT=P/'.epok/validation/dungeon-complete'
capture.OUT.mkdir(parents=True,exist_ok=True)
OUT=capture.OUT/('dynamic-'+time.strftime('%Y%m%d-%H%M%S'));OUT.mkdir(parents=True)
capture.OUT=OUT

# Compile the real Pocket Lua source and use the exact staged image (including
# its authored icons), so the safe installer recognizes the fixture's program.
_,payload=asset((P/'assets/PocketStation/AsteroidCatcher.epokasset').read_bytes())
app=documents.loads(payload)
(OUT/'miner.plua').write_text(next(f['text'] for f in app['files'] if f['name'].endswith('.plua')))
os.environ['EPOK_DUNGEON_MINER_SOURCE']=str(OUT/'miner.plua')
header=(P/'.epok/build/pocket-apps.hh').read_text()
body=re.search(r'pocket_app_image_0[^{}]*\{([^}]*)\}',header)[1]
image=bytes(int(v.strip(),0) for v in body.split(',') if v.strip())
(OUT/'miner.bin').write_bytes(image)
os.environ['EPOK_DUNGEON_MINER_IMAGE']=str(OUT/'miner.bin')
library=next((ENGINE/'target/debug/deps').glob('libepok_pocketstation-*.rlib'))
subprocess.run(['rustc','--edition=2021',str(ENGINE/'tests/fixtures/dungeon_complete_miner.rs'),'--extern','epok_pocketstation='+str(library),'-L','dependency='+str(ENGINE/'target/debug/deps'),'-o',str(P/'.epok/build/verify-asteroid-miner.exe')],check=True)
cards=[P/'.epok/emulator/memcard1.mcd',P/'.epok/emulator/memcard2.mcd',P/'UserSettings/PocketStation.mcd']
originals={};process=None;symbols={};events=[]

def request(path,data=None):
    return urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=data),timeout=3).read()
def ram():return request('cpu/ram/raw')
def write(offset,data):return request(f'cpu/ram/raw?offset={offset}&size={len(data)}',data)
def economy(data=None):return struct.unpack_from('<14I',data or ram(),symbols['economy_telemetry'])
def health():return struct.unpack_from('<I',ram(),symbols['dungeon_telemetry']+8)[0]
def wait(predicate,label,timeout=35):
    deadline=time.monotonic()+timeout
    while not predicate():
        if process.poll() is not None:raise RuntimeError('Emulator stopped: '+label)
        if time.monotonic()>deadline:raise TimeoutError((label,economy()))
        time.sleep(.15)
def ready():wait(lambda:economy()[13]==0,'memory card completion')
def press(button):
    request('execution-flow?function=pause',b'')
    offset=symbols['input']+2
    prior=struct.unpack_from('<H',ram(),offset)[0]
    write(offset,struct.pack('<H',prior|(1<<button)))
    request('execution-flow?function=resume',b'');time.sleep(.18)
def check(name,expected):
    ready();data=capture.snapshot(name,quiet=True);e=data['economy_telemetry']
    for k,v in expected.items():assert e[k]==v,(name,k,v,e)
    assert data['hud_stats'][4]==0,(name,'HUD dropped',data['hud_stats'])
    audio=data['dungeon_audio_telemetry'];assert audio[-1]==0,(name,'Missing sound asset',audio)
    music=struct.unpack_from('<16I',ram(),symbols['music'])
    assert music[10]==1 and music[11]==0 and music[12]==0,(name,'Music stopped or clock failed',music)
    events.append({'name':name,'economy':e,'audio':audio});print('PASS',name,flush=True)
def start(seconds,name):
    global process,symbols
    log=(OUT/(name+'.log')).open('w',encoding='utf-8')
    env=os.environ.copy();env.pop('EPOK_PROFILE_DETAIL',None)
    if '--detail' in sys.argv:env['EPOK_PROFILE_DETAIL']='1'
    process=subprocess.Popen([str(EDITOR),'--project',str(P),'--play-psx','--stop-after',str(seconds)],
        cwd=ENGINE,stdout=log,stderr=log,env=env,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
    deadline=time.monotonic()+180
    while True:
        if process.poll() is not None:raise RuntimeError((OUT/(name+'.log')).read_text()[-2500:])
        try:
            if json.loads(request('execution-flow'))['running']:break
        except OSError:pass
        if time.monotonic()>deadline:raise TimeoutError('PSX boot')
        time.sleep(.2)
    text=(P/'.epok/build/epok.map').read_text()
    names={'economy_telemetry':'economy_telemetry','dungeon_telemetry':'dungeon_telemetry','input':'epok::input','objects':'epok::objects','flow':'dungeon_flow_telemetry','music':'epok::music_sequence_stats'}
    symbols={n:int(re.findall(r'0x([0-9a-fA-F]+)\s+'+re.escape(full)+r'\s*$',text,re.M)[-1],16)&0x1fffff
             for n,full in names.items()}
    wait(lambda:economy()[12]==1,'initial save load');ready()
    wait(lambda:struct.unpack_from('<I',ram(),symbols['flow'])[0]==4,'intro and control unlock')
def profile(label):
    rows=[]
    for i in range(12):
        data=capture.snapshot(label+'-'+str(i),quiet=True);rows.append(data);time.sleep(.1)
    (OUT/(label+'-profile.json')).write_text(json.dumps(rows,indent=2))
def teleport(x,y,z):
    header=(P/'.epok/build/scene.hh').read_text()
    record=re.search(r'"Player Aqua",(?:true|false),-?\d+,-?\d+,-?\d+,actor_components_(\d+)',header)[1]
    index=int(re.search(r'actor_components_'+record+r'\[\]=\{\{UINT64_C\(\d+\),"[^"]+",(?:true|false),-?\d+,(-?\d+),',header)[1])
    count=int(re.search(r'std::array<ActorData,\s*(\d+)> objects',header)[1])
    nm=subprocess.check_output([str(NM),'-S','--defined-only',str(P/'.epok/build/epok.elf')],text=True)
    match=re.search(r'^([0-9a-f]+) ([0-9a-f]+) \w _ZN4epok7objectsE$',nm,re.M)
    stride=int(match[2],16)//count
    assert stride>=1000 and stride*count==int(match[2],16)
    # ActorData starts with three bools, an aligned parent index, then Transform.
    offset=symbols['objects']+index*stride+8
    request('execution-flow?function=pause',b'')
    write(offset,struct.pack('<3i',round(x*4096),round(y*4096),round(z*4096)))
    request('execution-flow?function=resume',b'');time.sleep(.3)

with socket.socket() as probe:probe.bind(('127.0.0.1',8077))
for i,path in enumerate(cards):
    originals[path]=path.exists()
    if path.exists():shutil.copy2(path,OUT/f'original-{i}.mcd')
try:
    if '--fresh-install' in sys.argv:
        subprocess.run([str(P/'.epok/build/verify-asteroid-miner.exe'),'--blank',str(cards[2])],check=True)
        start(35,'fresh-install');press(14);check('fresh-pocket',{0:1})
        press(14);wait(lambda:economy()[9]==1,'fresh minigame installation');ready()
        check('fresh-installed',{0:1,9:1,10:0})
        process.wait(timeout=45);assert process.returncode==0
        subprocess.run([str(P/'.epok/build/verify-asteroid-miner.exe'),'--check-installed',str(cards[2])],check=True)
        (OUT/'result.json').write_text(json.dumps({'passed':True,'fresh_install':True,'events':events},indent=2))
        sys.exit(0)
    subprocess.run([str(P/'.epok/build/verify-asteroid-miner.exe'),str(cards[2])],check=True)
    start(130,'psx')
    check('entry-room',{0:0,4:0})
    profile('entry')
    press(14);check('pocket',{0:1,2:0})
    press(5);press(14);check('slot-2-no-card',{2:1,10:1})
    press(7);press(6);press(14);wait(lambda:economy()[11]>=1,'ore save');ready()
    check('transferred-from-native-menu',{3:40,8:1,9:0,10:0,11:1})
    press(14);check('duplicate-transfer',{3:40,8:1,11:1})
    press(4);press(14);wait(lambda:economy()[9]==1,'existing app installation check');check('installed-data-preserved',{3:40,9:1,10:0})
    press(13);teleport(2.1,0,-5)
    press(14);check('shop',{0:2})
    press(14);wait(lambda:economy()[11]>=2,'sale save');check('sold',{3:0,4:400})
    press(6);press(14);wait(lambda:economy()[11]>=3,'medkit save');check('bought-medkit',{4:370,5:1})
    press(6);press(14);wait(lambda:economy()[11]>=4,'tank save');check('bought-tank',{4:290,6:1})
    press(6);press(14);wait(lambda:economy()[11]>=5,'buster save');check('bought-buster',{4:170,7:1})
    press(14);check('insufficient-funds',{4:170,7:1})
    press(13);press(12);press(14);check('full-health-medkit',{0:3,5:1,6:1})
    press(6);press(14);check('full-health-tank',{0:3,5:1,6:1})
    press(13);teleport(-4,0,19);profile('combat');wait(lambda:health()<100,'enemy damage',12)
    before=health();press(12);press(14);wait(lambda:economy()[11]>=6,'healing save');ready()
    check('used-medkit',{0:3,5:0,6:1});assert health()==min(100,before+35),(before,health())
    process.wait(timeout=140);assert process.returncode==0
    start(15,'reload');press(12);check('cold-save-reload',{0:3,3:0,4:170,5:0,6:1,7:1,12:1})
    process.wait(timeout=20);assert process.returncode==0
    (OUT/'result.json').write_text(json.dumps({'passed':True,'events':events},indent=2))
    print('PASS full PocketStation / PSX economy circuit.',flush=True)
finally:
    if process is not None and process.poll() is None:
        try:process.wait(timeout=140)
        except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=10)
    for i,path in enumerate(cards):
        if path.exists():shutil.copy2(path,OUT/f'test-result-{i}.mcd')
        if originals[path]:shutil.copy2(OUT/f'original-{i}.mcd',path)
        elif path.exists():os.replace(path,OUT/f'generated-{i}.mcd')
    print('Original cards restored. Evidence:',OUT,flush=True)
