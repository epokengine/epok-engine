"""Actual final18 Dungeon/Complete routes, unchanged images and selected rules.

Implementation draft: execute only under integration console lease. This script
never creates a replacement template/fixture. Dungeon observes natural pickup
and door callbacks after player-pose injection. Complete composes existing
actual-project guards/boss/economy/fresh-Pocket tests. Incomplete gates stay
missing, and any executed failure is preserved then raised, never retried.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import zipfile
import yaml
import adapter
import layout
import complete_observation

HERE=Path(__file__).resolve().parent
sha=adapter.sha
read=adapter.read
write=adapter.write

def actor_roles(binding):
    scene=binding['scene'];roles=binding['installation']['roles'];actors=scene['actors']
    def actor_for_class(identity,component=False):
        matches=[a for a in actors if (any(c['class'].get('class_id')==identity for c in a.get('components',[])) if component else a['class'].get('class_id')==identity)]
        if len(matches)!=1:raise ValueError('Dungeon role is absent/ambiguous: '+identity)
        return matches[0]
    player=actor_for_class(roles['player']['class_id'])
    key=actor_for_class(roles['key']['class_id'],True)
    door=actor_for_class(roles['door']['class_id'],True)
    key_component=next(c for c in key['components'] if c['class'].get('class_id')==roles['key']['class_id'])
    door_component=next(c for c in door['components'] if c['class'].get('class_id')==roles['door']['class_id'])
    k=key_component.get('properties',{});d=door_component.get('properties',{})
    if k.get('accepted_initiator',k.get('player'))!=player['id']:raise ValueError('Selected key is not bound to the actual player')
    if 'collected_flag' in k:
        if k['collected_flag']!=d.get('requirement_flag') or not k.get('auto_collect'):raise ValueError('Key/door ownership flag binding mismatch')
    elif d.get('key')!=key['id']:raise ValueError('Legacy key/door actor binding mismatch')
    names={role:a['name'] for role,a in [('player',player),('key',key),('door',door)]}
    if any(sum(a['name']==name for a in actors)!=1 for name in names.values()):raise ValueError('Nonunique runtime observer label')
    return names

def exact_program(binding,folder):
    folder=Path(folder).resolve(strict=True)
    for name in ['epok.ps-exe','epok.elf','epok.map','scene.hh']:
        if not (folder/name).is_file():raise ValueError('Exact program/debug/header required: '+str(folder/name))
    if sha(folder/'epok.ps-exe')!=sha(binding['executable']):raise ValueError('Foreign program image')
    # Both headers are tied to their source/export build receipts. The route
    # uses each exact header; reject an unexplained numeric-layout difference.
    original=binding['project']/'.epok/build/scene.hh'
    if sha(folder/'scene.hh')!=sha(original):raise ValueError('Exported header differs; review before runtime slot observation')
    return {'folder':folder,'exe_sha256':sha(folder/'epok.ps-exe'),'elf_sha256':sha(folder/'epok.elf'),
            'map_sha256':sha(folder/'epok.map'),'header_sha256':sha(folder/'scene.hh')}

def command(command,log,cwd,timeout=1800,env=None):
    if any(flag in command for flag in ['--create-project','--new-blueprint','--new-lua-class']):raise ValueError('A scenario may not substitute a fixture project')
    started=time.time()
    with Path(log).open('w',encoding='utf-8') as stream:
        process=subprocess.Popen(list(map(str,command)),cwd=cwd,stdout=stream,stderr=subprocess.STDOUT,env=env,
                                 creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        try:code=process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name=='nt':subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],capture_output=True,timeout=15)
            else:process.terminate()
            process.wait(timeout=15);code=124
    record={'argv':list(map(str,command)),'exit_code':code,'seconds':round(time.time()-started,3),'log_sha256':sha(log)}
    write(Path(log).with_suffix('.command.json'),record)
    return record

def dungeon_layout(root,elf):
    symbols=layout.symbols(root,elf)
    structures=layout.dwarf_structures(layout.tool(root,'readelf','--debug-dump=info','--wide',elf))
    specs={'ActorData':['parent','transform','name','owner','alive','active'],'Transform':['position','rotation','scale'],
           'Time':['ticks','dropped_steps'],'ResourceUsage':['dropped_primitives'],'HudStats':['dropped']}
    layouts={name:layout.require_layout(structures,name,fields) for name,fields in specs.items()}
    names=['epok::objects','epok::object_count','epok::time','epok::performance_stats','epok::actor_stats','epok::resource_usage','epok::hud_stats']
    if any(name not in symbols for name in names):raise ValueError('Required actual runtime counters absent')
    frames=[v['address'] for name,v in symbols.items() if name.endswith('GameScene::frame()') and 'lambda' not in name]
    if len(set(frames))!=1:raise ValueError('Missing/ambiguous actual frame callback')
    size=symbols['epok::objects']['size'];stride=layouts['ActorData']['size']
    if not size or size%stride or layouts['Transform']['size']!=36:raise ValueError('Unsupported compiled object/Fixed ABI')
    return {'symbols':{name:symbols[name]['address'] for name in names},'frame_entry':frames[0],
            'layouts':layouts,'slot_capacity':size//stride}

def capture_dungeon(root,binding,program,output):
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    program=exact_program(binding,program)
    config=dungeon_layout(root,program['folder']/'epok.elf')
    config.update(actor_names=actor_roles(binding),steps=120,output=str((output/'observation.json').resolve()).replace('\\','/'),
                  executable_sha256=program['exe_sha256'],header_sha256=program['header_sha256'],source_inventory_sha256=adapter.digest(binding['source_inventory']))
    write(output/'config.json',config)
    (output/'route.lua').write_text('-- Automated run: keep the SPU running and silence only its output.\npcall(function() PCSX.settings.spu.Mute=true end)\n'+(HERE/'dungeon_keydoor.lua').read_text(encoding='utf-8').replace('@CONFIG@',adapter.lua_literal(config)),encoding='utf-8')
    emulator=next((p for p in [root/'.tools/redux/pcsx-redux.main',root/'.tools/redux/pcsx-redux.exe'] if p.is_file()),None)
    bios=next((p for p in [root/'.tools/redux/openbios-fastboot.bin',root/'.tools/redux/openbios.bin'] if p.is_file()),None)
    if not emulator or not bios:raise ValueError('Pinned Redux/BIOS unavailable')
    portable=output/'portable';portable.mkdir()
    argv=[emulator,'-portable',portable,'-logfile',output/'runtime.log','-run','-stdout','-fastboot','-noupdate','-interpreter','-softgpu','-2mb','-no-gdb','-no-ui','-bios',bios,'-loadexe',program['folder']/'epok.ps-exe','-dofile',output/'route.lua','-debugger']
    argv+=['-pcdrv','-pcdrvbase',program['folder']] if (program['folder']/'GEOMETRY.BIN').exists() else ['-no-pcdrv']
    result=command(argv,output/'process.log',portable,180)
    if result['exit_code']!=0:raise ValueError('Actual Dungeon route failed; preserve '+str(output))
    observed=read(output/'observation.json');validate_dungeon(observed,binding,program)
    return observed

def validate_dungeon(observation,binding,program):
    if observation.get('executable_sha256')!=program['exe_sha256'] or observation.get('header_sha256')!=program['header_sha256'] or observation.get('source_inventory_sha256')!=adapter.digest(binding['source_inventory']):raise ValueError('Foreign source/image/header observation')
    rows=observation.get('rows',[])
    if observation.get('reason')!='key-door-complete' or observation.get('failures') or not rows:raise ValueError('Incomplete actual Dungeon route')
    if not rows[0]['key'] or not rows[0]['door'] or rows[-1]['key'] or rows[-1]['door']:raise ValueError('Key/door lifecycle not observed')
    if rows[-1]['step']<120 or any(b['step']<a['step'] for a,b in zip(rows,rows[1:])):raise ValueError('Invalid actual simulation step sequence')
    if any(type(r.get('cpu_cycles'))is not int or r['cpu_cycles']<=0 for r in rows):raise ValueError('Actual CPU work missing')
    key_edges=sum(a['key'] and not b['key'] for a,b in zip(rows,rows[1:]));door_edges=sum(a['door'] and not b['door'] for a,b in zip(rows,rows[1:]))
    if (key_edges,door_edges)!=(1,1):raise ValueError('Collection/opening must happen exactly once')

def normalize_dungeon(observation):
    rows=observation['rows']
    # Gameplay outcomes are provider-neutral; raw frame/tick/cycle data remains
    # backing evidence and cannot be dropped from the equal-step CPU gate.
    return {'boot_key':rows[0]['key'],'boot_door':rows[0]['door'],'collected':not rows[-1]['key'],'opened':not rows[-1]['door'],
            'key_transitions':sum(a['key']and not b['key']for a,b in zip(rows,rows[1:])),
            'door_transitions':sum(a['door']and not b['door']for a,b in zip(rows,rows[1:]))}

def package_current(binding,program,path):
    program=exact_program(binding,program);folder=program['folder']
    if not (folder/'GEOMETRY.BIN').is_file():raise ValueError('Complete actual external geometry missing')
    files={'epok.ps-exe':folder/'epok.ps-exe','debug/epok.elf':folder/'epok.elf','debug/epok.map':folder/'epok.map','GEOMETRY.BIN':folder/'GEOMETRY.BIN'}
    manifest={'schema':1,'loader':{'pcdrv_files':['GEOMETRY.BIN'],'pcdrv_required':True,'cd_required':False},
              'files':[{'path':name,'sha256':sha(file)}for name,file in files.items()]}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_STORED)as archive:
        for name,file in files.items():archive.write(file,name)
        archive.writestr('epok-package.json',json.dumps(manifest))
    return program

def new_evidence(project,glob,before):
    found=[p for p in (project/'.epok/validation').glob(glob) if p.is_dir() and str(p.resolve())not in before]
    if len(found)!=1:raise ValueError('Missing/ambiguous actual-project capture directory')
    return found[0]

def routes_match(first,second):
    # Disc routes sample different steps at frame ends. Every step both sampled must
    # hold the same state, and the samples must overlap for nearly the whole route.
    if any(first[k]!=second[k] for k in first if k!='completed_frame_observations'):return False
    a,b=first['completed_frame_observations'],second['completed_frame_observations']
    if isinstance(a,list):return a==b
    common=set(a)&set(b)
    return len(common)>=0.9*min(len(a),len(b)) and all(a[s]==b[s] for s in common)

def complete_route(root,binding,program,route,output,disc=None):
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    package=output/'exact-program.zip';exact=package_current(binding,program,package)
    backend='cd'if disc else'pcdrv'
    project=binding['project'];before={str(p.resolve())for p in (project/'.epok/validation').glob('stream-'+backend+'-'+route+'-*')}
    argv=[sys.executable,root/'tests/integration/verify_dungeon_complete.py','--project',project,'--route',route,'--frames',('12000'if route=='closure'else'1800'),'--deterministic-route','--capture-slow-frames','--package',package,'--header',exact['folder']/'scene.hh']
    if disc:argv+=['--cd',disc]
    result=command(argv,output/'driver.log',root,1500 if route=='closure'else 420)
    captured=new_evidence(project,'stream-'+backend+'-'+route+'-*',before)
    write(output/'capture-binding.json',{'actual_project':str(project),'capture':str(captured),'program':{k:str(v)for k,v in exact.items()},'package_sha256':sha(package)})
    for name in ['result.json','replay.json','last-counters.json','slow-frames.json','samples.json','scene.hh','current-program-contract.json','complete-observation.json']:
        if(captured/name).is_file():shutil.copy2(captured/name,output/name)
    if result['exit_code']!=0:raise ValueError('Actual Complete '+route+' failed, including unchanged25ms/drop/sound gates; '+str(output))
    data=read(output/'result.json')
    if sha(output/'scene.hh')!=exact['header_sha256']:raise ValueError('Actual Complete capture used wrong header')
    validate_complete(data,exact['exe_sha256'])
    return {'result':data,'replay':read(output/'replay.json'),'counters':read(output/'last-counters.json'),'observation':read(output/'complete-observation.json')}

# Emulated CD reads move a frame boundary by a few hundred microseconds from run to
# run, even for one image and one disc, so a frame can end one simulation step early
# or late. A disc route therefore keys its observations by simulation step, and drops
# the fields that follow frame duration rather than game state. The step count starts
# again at every restart, so the key also carries the restart count, and a key keeps
# every distinct state it showed, in order, across frames that ran no step.
FRAME_TIMED=('frame','image_color','image_enabled','borders','fade','music_volume')

def step_keyed(rows):
    keyed={}
    for row in rows:
        state={k:v for k,v in row.items() if k not in FRAME_TIMED}
        seen=keyed.setdefault(str(row['restarts'])+':'+str(row['step']),[])
        if not seen or seen[-1]!=state:seen.append(state)
    return keyed

def normalize_complete_route(value,disc=False):
    data=value['result'];replay=value['replay'];counters=value['counters']
    rows=value['observation']['rows']
    if disc:rows=step_keyed(rows)
    return {'route':data['route'],'states':data['states'],'maximum_guards':data['maximum_guards'],'shots':data['shots'],'boss_health':data['boss_health'],
            'events':replay['events'],'pad_changes':replay['pad_changes'],'stopped_frame':replay['stopped_frame'],
            'work_steps':sorted([{'phase':r['phase'],'steps':r['steps'],'frames':r['frames']}for r in replay['work']],key=lambda r:(r['phase'],r['steps'])),
            'game':counters['game'],'combat':counters['combat'],'flow':counters['flow'],'completed_frame_observations':rows}

def validate_complete(value,expected_image):
    if value.get('executable_sha256')!=expected_image:raise ValueError('Actual Complete capture used wrong image')
    # Frame time is reported, not blocking, for this template. See the contract.
    if any(value[k]for k in ['max_dropped_triangles','max_stream_skipped_chunks','max_stream_failed_chunks','missing_sound_assets']):raise ValueError('Unchanged actual Complete runtime gate failed')

def snapshot_export(binding,output):
    # Keep the certified image and the editor's disc of it with this evidence.
    folder=binding['project']/'.epok/build';output=Path(output);output.mkdir(parents=True,exist_ok=False)
    for name in ['epok.ps-exe','epok.elf','epok.map','scene.hh','GEOMETRY.BIN','epok.bin','epok.cue']:
        if not (folder/name).is_file():raise ValueError('Export image file missing: '+name)
        shutil.copy2(folder/name,output/name)
    if sha(output/'epok.ps-exe')!=sha(binding['executable']):raise ValueError('Project build is not the certified export image')
    return output

def author_disc(root,export_root,output):
    # The standalone export documents mkpsxiso for its disc. Author it from the
    # relocated folder, writing only into this evidence folder.
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    tool=root/('.tools/mkpsxiso/mkpsxiso-2.30-win64/mkpsxiso.exe'if os.name=='nt'else'.tools/linux/mkpsxiso/mkpsxiso-2.30-Linux/bin/mkpsxiso')
    result=command([tool,'-y','-q','-o',output/'epok.bin','-c',output/'epok.cue','disc.xml'],output/'mkpsxiso.log',export_root,600)
    if result['exit_code']!=0 or not (output/'epok.cue').is_file():raise ValueError('Standalone disc authoring failed; '+str(output))
    return output

def cards(project):
    candidates=[project/'.epok/emulator/memcard1.mcd',project/'.epok/emulator/memcard2.mcd',project/'UserSettings/PocketStation.mcd']
    return {str(p):sha(p)if p.exists()else None for p in candidates}

def complete_economy(root,binding,output,fresh=False):
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    project=binding['project'];before={str(p.resolve())for p in (project/'.epok/validation/dungeon-complete').glob('dynamic-*')}
    protected=cards(project);expected=sha(binding['executable']);program_before=exact_program(binding,project/'.epok/build')
    argv=[sys.executable,root/'tests/integration/verify_dungeon_complete_economy.py','--project',project]+(['--fresh-install']if fresh else[])
    result=command(argv,output/'driver.log',root,600)
    after=cards(project);write(output/'protected-cards.json',{'before':protected,'after':after,'preserved':protected==after})
    if protected!=after:raise ValueError('Actual-project memory cards were not restored')
    if sha(binding['executable'])!=expected:raise ValueError('Economy tool rebuilt a different image; no reuse permitted')
    program_after=exact_program(binding,project/'.epok/build')
    if any(program_before[k]!=program_after[k]for k in ['exe_sha256','elf_sha256','map_sha256','header_sha256']):raise ValueError('Economy image/debug/header changed during execution')
    captured=new_evidence(project,'dungeon-complete/dynamic-*',before)
    write(output/'capture-binding.json',{'actual_project':str(project),'capture':str(captured),'executable_sha256':expected,
          'header_sha256':sha(project/'.epok/build/scene.hh'),'source_inventory_sha256':adapter.digest(binding['source_inventory'])})
    if(captured/'result.json').exists():shutil.copy2(captured/'result.json',output/'result.json')
    if result['exit_code']!=0:raise ValueError('Actual economy/fresh-Pocket circuit failed; preserve '+str(output))
    value=read(output/'result.json')
    if value.get('passed')is not True:raise ValueError('Actual economy circuit did not pass')
    # These are the actual UI transaction/state checks, not the isolated896-row
    # policy fixture. No guard/shop/run-flow edit proof is inferred from them.
    names={event['name']for event in value['events']}
    required={'fresh-pocket','fresh-installed'}if fresh else{'slot-2-no-card','transferred-from-native-menu','duplicate-transfer','installed-data-preserved','sold','bought-medkit','bought-tank','bought-buster','insufficient-funds','used-medkit','cold-save-reload'}
    if not required<=names:raise ValueError('Actual UI/Pocket scenario event missing')
    return {'fresh_install':fresh,'events':[{'name':e['name'],'economy':e['economy']}for e in value['events']]}

def main(argv=None):
    parser=argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--editor',type=Path)
    for name in ['project','export-root','export-manifest','output']:parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--case',required=True);parser.add_argument('--mode',required=True);parser.add_argument('--dependency',required=True)
    parser.add_argument('--rule-edit-proof',type=Path,action='append',default=[])
    parser.add_argument('--execute',action='store_true');args=parser.parse_args(argv);args.root=args.root.resolve()
    binding=adapter.validate_binding(args.root,args.project,args.case,args.mode,args.dependency,args.export_root,args.export_manifest)
    if binding['template']not in ['dungeon','dungeon-complete']:raise ValueError('This adapter cannot certify another template')
    queue=read(args.root/'artifacts/gameplay-helpers/p10/current-acceptance-queue.json')
    proof=adapter.base_proof(args,binding,queue['matrix_contract']['scenarios'][binding['template']])
    work=args.output.with_name(args.output.stem+'-dungeon-evidence');work.mkdir(parents=True,exist_ok=False)
    write(work/'source-inventory.json',binding['source_inventory'])
    sources=[args.root/'tests/integration/verify_dungeon_complete.py',args.root/'tests/integration/verify_dungeon_complete_economy.py',args.root/'tests/integration/verify_dungeon_foundation.py',HERE/'dungeon_keydoor.lua',HERE/'dungeon_routes.py',HERE/'complete_observation.py',HERE/'complete_observation.lua']
    write(work/'driver-source-hashes.json',{str(p):sha(p)for p in sources})
    try:
        if not args.execute:proof['missing'].append('Current image execution not requested; use--execute only under integration console lease')
        elif binding['template']=='dungeon':
            first=capture_dungeon(args.root,binding,binding['project']/'.epok/build',work/'original')
            restarted=capture_dungeon(args.root,binding,binding['project']/'.epok/build',work/'cold-restart')
            exported=capture_dungeon(args.root,binding,binding['export_root'],work/'exported')
            normalized=normalize_dungeon(first)
            if normalized!=normalize_dungeon(restarted)or normalized!=normalize_dungeon(exported):raise ValueError('Actual cold-restart/export key-door behavior differs')
            proof['scenarios']['key/door']={'passed':True};proof['normalized_trace_sha256']=adapter.digest(normalized)
            rows=first['rows'];timings=[(r['cpu_cycles']*1_000_000+33_868_799)//33_868_800 for r in rows]
            proof['metrics'].update(slow_frames=sum(t>25000 for t in timings),max_frame_us=max(timings),simulation_steps=rows[-1]['step'],
               cpu_work_units=sum(r['cpu_cycles']for r in rows),dropped_work=max(r['dropped_steps']+r['dropped_primitives']for r in rows),
               rejected_work=max(r['actor_stats'][2]for r in rows),hud_drops=max(r['hud_drops']for r in rows),max_runtime_actors=max(r['actor_stats'][5]for r in rows))
            if proof['metrics']['slow_frames']or any(proof['metrics'][k]for k in ['dropped_work','rejected_work','hud_drops']):raise ValueError('Unchanged Dungeon25ms/drop/rejection gate failed')
            proof.update(capture_lifecycle_passed=True,export_scenario_passed=True)
            proof['observed_functional_scenarios']={'actual-key-door':True,'cold-process-restart-defaults':True,'export-key-door':True}
            proof['missing']+=['Regeneration and renaming/duplication authoring workflows are not cold process restart; require actual workflow proof.','Whole startup/heap/stack/VRAM/max-component coverage and matched equal-step baseline remain unmeasured.']
        else:
            # Play and the export read Complete's streamed geometry from disc; only the
            # build stage's memory analysis uses the Play profile over PCdrv. Both routes
            # run the certified image from disc: the original from the editor's disc, the
            # export from a disc authored in the relocated folder.
            image=snapshot_export(binding,work/'export-image')
            standalone=author_disc(args.root,binding['export_root'],work/'export-disc')
            proof['export_disc']={'editor_sha256':sha(image/'epok.bin'),'standalone_sha256':sha(standalone/'epok.bin')}
            observed={};exports={};semantic={}
            for route in ['guards','boss','closure']:
                value=complete_route(args.root,binding,image,route,work/('original-'+route),image/'epok.cue')
                exported=complete_route(args.root,binding,binding['export_root'],route,work/('export-'+route),standalone/'epok.cue')
                normal=normalize_complete_route(value,disc=True)
                if not routes_match(normal,normalize_complete_route(exported,disc=True)):raise ValueError('Actual exported '+route+' behavior/equal-step route differs')
                observed[route]=value;exports[route]=exported;semantic[route]=normal
            semantic['economy']=complete_economy(args.root,binding,work/'economy')
            semantic['fresh-install']=complete_economy(args.root,binding,work/'fresh-install',True)
            proof['normalized_trace_sha256']=adapter.digest(semantic)
            proof['scenarios']['guards/boss']={'passed':True};proof['scenarios']['inventory/shop/save/restart']={'passed':True}
            for name,passed in observed['closure']['result']['functional_scenarios'].items():
                proof['scenarios'][name]={'passed':passed,'evidence':'original/export closure per-completed-frame observation'}
            proof['scenarios']['terminal/install/status/transfer/ACK']={'passed':False,'observed':'Happy transfer/duplicate/fresh-install/no-card route; interrupted ACK still requires dedicated evidence'}
            # The driver checks active, error-free music and exact streaming in every route,
            # and asks routes of 4,000 frames or more to cross a MIDI loop. The 1,800-frame
            # guard and boss routes end before the first boundary, so the crossing is
            # required where it can happen: the closure, from both discs.
            music_loops=all(value['closure']['result']['music'][4]>0 for value in (observed,exports))
            proof['scenarios']['music/streaming']={'passed':music_loops,'observed':'Exact streamed pages/textures and active music in every route; MIDI loop crossed in the original and exported closure',
                'closure_loops':{'original':observed['closure']['result']['music'][4],'export':exports['closure']['result']['music'][4]}}
            proof.update(music_passed=music_loops,streaming_passed=True,capture_lifecycle_passed=True)
            values=list(observed.values());proof['metrics'].update(slow_frames=sum(v['result']['slow_frames']for v in values),max_frame_us=max(v['result']['max_frame_us']for v in values),
                simulation_steps=sum(r['steps']*r['frames']for v in values for r in v['replay']['work']),cpu_work_units=sum(r['cpu_scanlines']for v in values for r in v['replay']['work']),
                dropped_work=max(v['result']['max_dropped_triangles']for v in values),rejected_work=max(v['counters']['actors'][2]for v in values),
                missing_sounds=max(v['result']['missing_sound_assets']for v in values))
            for receipt in args.rule_edit_proof:
                check,files=complete_observation.load_rule_edit_receipt(receipt,binding)
                key={'encounter_policy':'guard','shop_widget':'shop','run_flow':'run_flow'}[check['role']]
                if proof['rule_edit_checks'][key]:raise ValueError('Duplicate rule-edit proof')
                proof['rule_edit_checks'][key]=check['passed'];proof['evidence_files'].update(files)
            proof['ownership_edit_observed']=all(proof['rule_edit_checks'].values())
            proof['observed_functional_scenarios']={'guards':True,'boss':True,'economy-UI-circuit':True,'fresh-Pocket-install':True,'Pocket-no-card-error':True,'exported-guards-boss':True}
            proof['missing']+=['Chosen-language guard/shop/run-flow edited-image proof remains pending; validate_rule_edit rejects source-only claims.','Interrupted ACK/retry/failure protocol and exported economy/Pocket circuit remain pending.','Heap/stack/VRAM/HUD/max-component peaks, exact baseline CPU comparison and complete phase coverage still require measurements.']
        if adapter.inventory(binding['project'])!=binding['source_inventory']:raise ValueError('Actual authored project changed during scenario')
    except BaseException as error:
        proof['status']='failed';proof['error']=str(error)
        for p in work.rglob('*'):
            if p.is_file():proof['evidence_files'][str(p.resolve())]=sha(p)
        write(args.output,proof);raise
    for p in work.rglob('*'):
        if p.is_file():proof['evidence_files'][str(p.resolve())]=sha(p)
    write(args.output,proof);print(json.dumps({'status':proof['status'],'observed':proof.get('observed_functional_scenarios'),'missing':proof['missing'],'output':str(args.output)},indent=2))
    return 0

if __name__=='__main__':raise SystemExit(main())
