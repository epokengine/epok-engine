"""P10 actual-project/exact-export scenario adapter draft. No fixture substitution.

Basic/Sample execute unchanged images under an external Redux observer, using
layouts from each exact ELF and120 observed simulation ticks. Other four routes
remain explicitly missing until their actual-template drivers are adapted.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import yaml
from layout import inspect

HERE=Path(__file__).resolve().parent
METRICS='slow_frames max_frame_us simulation_steps cpu_work_units dropped_work rejected_work missing_sounds heap_peak_bytes stack_peak_bytes vram_peak_bytes hud_drops max_runtime_actors max_components_per_actor'.split()
TEMPLATES=['basic','sample','third-person','platformer-2d','dungeon','dungeon-complete']
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def digest(value):return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()
def read(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def write(p,v):p=Path(p);p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def documents(p):return yaml.safe_load(Path(p).read_text(encoding='utf-8'))
def contained(p,base):return Path(p).resolve().is_relative_to(Path(base).resolve())
def inventory(project):
    result={}
    for folder in ['assets','ProjectSettings']:
        for p in (project/folder).rglob('*'):
            if p.is_symlink():raise ValueError('Linked source requires separate reviewed provenance')
            if p.is_file():result[p.relative_to(project).as_posix()]=sha(p)
    for p in project.glob('*.epokproject'):result[p.name]=sha(p)
    return result

def validate_binding(root,project,case,mode,dependency,export_root,export_manifest):
    template,flavor,selected=case.split('/')
    if template not in TEMPLATES or flavor not in ['cpp','blueprint','lua'] or mode!=selected or mode!='native_cpp':raise ValueError('Wrong template/provider/mode')
    if not re.fullmatch('[a-f0-9]{64}',dependency):raise ValueError('Invalid row dependency hash')
    project=Path(project).resolve(strict=True);root=Path(root).resolve(strict=True)
    manifests=list(project.glob('*.epokproject'))
    if len(manifests)!=1:raise ValueError('Exactly one actual project manifest required')
    manifest=documents(manifests[0]);installation=documents(project/'ProjectSettings/Template.epoksettings')
    if (installation['template'],installation['gameplay'],manifest['lua_execution'])!=(template,flavor,'native_cpp'):raise ValueError('Foreign project or wrong installed language')
    scene_file=project/manifest['startup_scene']
    if not contained(scene_file,project):raise ValueError('Scene escapes actual project')
    scene=documents(scene_file)
    if max((len(a.get('components',[])) for a in scene['actors']),default=0)>8:raise ValueError('Authored component limit exceeded')
    export_root=Path(export_root).resolve(strict=True);export_manifest=Path(export_manifest).resolve(strict=True)
    if contained(export_root,root) or contained(export_root,project):raise ValueError('Export must be relocated outside original project/repository')
    exported=read(export_manifest)
    if Path(exported['relocated']).resolve()!=export_root:raise ValueError('Foreign relocated export path')
    marker=read(export_root/'.p10-export.json')
    if marker.get('dependency_sha256')!=dependency:raise ValueError('Export belongs to a different dependency row')
    executable=project/'.epok/build/epok.ps-exe'
    if not executable.is_file() or sha(executable)!=exported['executable_sha256'] or sha(export_root/'epok.ps-exe')!=sha(executable):raise ValueError('Export/source executable identity mismatch')
    for relative,expected in exported.get('external_assets',{}).items():
        source=project/'.epok/build'/relative;target=export_root/relative
        if not contained(target,export_root) or not contained(source,project):raise ValueError('Escaping external asset')
        if not source.is_file() or sha(source)!=expected or sha(target)!=expected:raise ValueError('Wrong exported external asset')
    for relative,expected in exported.get('inputs',{}).items():
        path=export_root/relative
        if not contained(path,export_root) or not path.is_file() or sha(path)!=expected:raise ValueError('Export input changed after certification: '+relative)
    return {'template':template,'flavor':flavor,'project':project,'manifest':manifest,'installation':installation,'scene':scene,
            'source_inventory':inventory(project),'export_root':export_root,'export_manifest':export_manifest,'executable':executable}

def select_actor(binding):
    template=binding['template'];scene=binding['scene'];roles=binding['installation']['roles']
    if template=='basic':
        identity=roles['game_root']['class_id']
        selected=[a for a in scene['actors'] if a['class'].get('class_id')==identity]
        if len(selected)!=1:raise ValueError('Actual inert GameRoot not unique')
        actor=selected[0]
        if len(actor.get('components',[]))!=1 or not actor['components'][0].get('root'):raise ValueError('Basic GameRoot must keep its ordinary single spatial root')
        if any(c['class']['name']=='epok::Mesh3DComponent' for a in scene['actors'] for c in a.get('components',[])):raise ValueError('Basic empty world unexpectedly contains a mesh')
    elif template=='sample':
        identity=roles['spinner']['class_id']
        selected=[(a,c) for a in scene['actors'] for c in a.get('components',[]) if c['class'].get('class_id')==identity]
        if len(selected)!=1:raise ValueError('Actual selected Spinner component not unique')
        actor,spinner=selected[0]
        if spinner.get('properties',{}).get('speed',45)!=45:raise ValueError('Default source scenario requires selected Spinner speed45; edited route must be explicit')
    else:raise ValueError('Actual-template dynamic driver pending for '+template)
    if sum(a['name']==actor['name'] for a in scene['actors'])!=1:raise ValueError('Observer mapping requires a unique current source actor label')
    return actor

def lua_literal(value):
    if isinstance(value,bool):return 'true' if value else 'false'
    if isinstance(value,dict):return '{'+','.join('['+json.dumps(str(k))+']='+lua_literal(v) for k,v in value.items())+'}'
    if isinstance(value,list):return '{'+','.join(lua_literal(v) for v in value)+'}'
    return json.dumps(value)

def capture(root,binding,program,output,diagnostic_memory=False,steps=120):
    """Future execution requires the integrator's console lease. Never builds."""
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    program=Path(program);elf=program/'epok.elf';executable=program/'epok.ps-exe'
    if sha(executable)!=sha(binding['executable']):raise ValueError('Capture image differs from selected final program')
    if not elf.is_file():raise ValueError('Exact linked debug ELF required; no inferred runtime ABI')
    layout=inspect(root,elf,binding['template']);actor=select_actor(binding)
    config=dict(layout,template=binding['template'],actor_name=actor['name'],steps=steps,
                output=str((output/'observation.json').resolve()).replace('\\','/'),diagnostic_memory=diagnostic_memory,
                source_inventory_sha256=digest(binding['source_inventory']),executable_sha256=sha(executable),
                performance_words=layout['elf_symbol_sizes']['epok::performance_stats']//4,
                hud_words=layout['elf_symbol_sizes']['epok::hud_stats']//4)
    write(output/'config.json',config)
    source=''
    if diagnostic_memory:
        spec=importlib.util.spec_from_file_location('p10_memory',root/'tools/profile_runtime_memory.py')
        profiler=importlib.util.module_from_spec(spec);sys.modules[spec.name]=profiler;spec.loader.exec_module(profiler)
        memory,writes=profiler.elf_probe_config(elf,root/'.tools/mips/bin')
        write(output/'memory-config.json',memory)
        source+=profiler.lua_probe(memory,writes)
    source+=(HERE/'observe.lua').read_text(encoding='utf-8').replace('@CONFIG@',lua_literal(config))
    (output/'observe.lua').write_text('-- Automated run: keep the SPU running and silence only its output.\npcall(function() PCSX.settings.spu.Mute=true end)\n'+source,encoding='utf-8')
    emulator=next((p for p in [root/'.tools/redux/pcsx-redux.main',root/'.tools/redux/pcsx-redux.exe'] if p.is_file()),None)
    bios=next((p for p in [root/'.tools/redux/openbios-fastboot.bin',root/'.tools/redux/openbios.bin'] if p.is_file()),None)
    if emulator is None or bios is None:raise ValueError('Pinned PCSX-Redux and BIOS missing')
    portable=output/'portable';portable.mkdir()
    command=[str(emulator),'-portable',str(portable),'-logfile',str(output/'runtime.log'),'-run','-stdout','-fastboot','-noupdate','-interpreter','-softgpu','-2mb','-no-gdb','-no-ui','-bios',str(bios),'-loadexe',str(executable),'-dofile',str(output/'observe.lua'),'-debugger']
    command+=['-pcdrv','-pcdrvbase',str(program)] if (program/'GEOMETRY.BIN').exists() else ['-no-pcdrv']
    with (output/'process.log').open('w',encoding='utf-8') as log:
        process=subprocess.Popen(command,cwd=portable,stdout=log,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        try:code=process.wait(timeout=600 if diagnostic_memory else 180)
        except subprocess.TimeoutExpired:
            process.terminate();process.wait(timeout=15);raise ValueError('Actual-image observer timed out; preserve logs, no retry')
    write(output/'command.json',{'argv':command,'exit_code':code,'executable_sha256':sha(executable),'elf_sha256':sha(elf),'source_inventory_sha256':digest(binding['source_inventory'])})
    if code!=0:raise ValueError('Actual-image observer failed; see '+str(output/'process.log'))
    observed=read(output/'observation.json')
    if observed.get('clock_observer',{}).get('address')!=layout['tick_counter_address']:raise ValueError('Observation clock address differs from this exact ELF/DWARF')
    validate_observation(observed,binding,steps,diagnostic_memory)
    if inventory(binding['project'])!=binding['source_inventory']:raise ValueError('Actual authored project changed while observing it')
    return observed

def validate_observation(observed,binding,steps,diagnostic):
    if observed.get('executable_sha256')!=sha(binding['executable']) or observed.get('source_inventory_sha256')!=digest(binding['source_inventory']):raise ValueError('Observation belongs to another source/image')
    if observed.get('reason')!='exact-steps' or observed.get('errors') or observed.get('diagnostic_memory') is not diagnostic:raise ValueError('Incomplete/error/wrong observation phase')
    trace=observed.get('trace',[])
    if [r['step'] for r in trace]!=list(range(1,steps+1)):raise ValueError('Missing actual equal-step observations')
    if type(observed.get('cpu_work_units')) is not int or observed['cpu_work_units']<=0:raise ValueError('CPU work unmeasured')
    if not observed.get('frames') or any(type(r.get('cpu_cycles')) is not int or r['cpu_cycles']<=0 for r in observed['frames']):raise ValueError('Frame CPU work unmeasured')
    expected_boundary='time-write-and-frame-return' if binding['template']=='basic' else 'selected-actor-tick-return'
    clock=observed.get('clock_observer',{})
    if clock.get('kind')!=expected_boundary or type(clock.get('address')) is not int or not 0x80000000<=clock['address']<0x80200000:raise ValueError('Exact compiled clock observation missing')
    if binding['template']=='basic' and (type(clock.get('write_callbacks')) is not int or clock['write_callbacks']<=0):raise ValueError('Basic Time write observation never triggered')
    observations=observed.get('step_observations',[])
    if [r.get('step') for r in observations]!=list(range(1,steps+1)):raise ValueError('Missing actual per-step clock observations')
    allowed={'time-write','frame-return'} if binding['template']=='basic' else {'tick-return'}
    if any(r.get('boundary') not in allowed or type(r.get('frame')) is not int or r['frame']<=0 or type(r.get('cpu_work_units')) is not int or r['cpu_work_units']<=0 for r in observations):raise ValueError('Unmeasured per-step boundary/CPU work')
    if any(b['cpu_work_units']<=a['cpu_work_units'] or b['frame']<a['frame'] for a,b in zip(observations,observations[1:])):raise ValueError('Per-step CPU/frame sequence did not advance')
    frames=observed['frames']
    if [r.get('observer_frame') for r in frames]!=list(range(1,len(frames)+1)) or clock.get('frames')!=len(frames):raise ValueError('Actual frame observation sequence incomplete')
    for index,row in enumerate(frames):
        if type(row.get('frame_sequence')) is not int or (index and row['frame_sequence']!=frames[index-1]['frame_sequence']+1):raise ValueError('Published frame sequence incomplete')
        if any(type(row.get(key)) is not int for key in ['ticks','start_ticks','executed_steps','requested_steps']):raise ValueError('Simulation clock/frame counters missing')
        if not 0<=row['executed_steps']<=8 or row['executed_steps']!=row['ticks']-row['start_ticks'] or row['requested_steps']!=row['executed_steps']:raise ValueError('Simulation clock/frame counters disagree')
        if index and row['start_ticks']!=frames[index-1]['ticks']:raise ValueError('Simulation clock discontinuity between frames')
    if observations[-1]['cpu_work_units']!=observed['cpu_work_units'] or observed['cpu_work_units']>sum(r['cpu_cycles'] for r in frames):raise ValueError('Exact-step CPU work does not match actual frame work')
    if frames[0]['start_ticks']!=0 or frames[-1]['ticks']<steps or any(r['frame']>len(frames) for r in observations):raise ValueError('Observed clock misses start or requested steps')
    cumulative=[0]
    for row in frames:cumulative.append(cumulative[-1]+row['cpu_cycles'])
    for row in observations:
        frame=frames[row['frame']-1]
        if not frame['start_ticks']<row['step']<=frame['ticks'] or not cumulative[row['frame']-1]<row['cpu_work_units']<=cumulative[row['frame']]:raise ValueError('Step observation is outside its actual clock/CPU frame')
        if row['boundary']=='frame-return' and row['step']!=frame['ticks']:raise ValueError('Frame-return cannot invent intermediate step observations')
    for r in trace:
        if not r.get('owner'):raise ValueError('Runtime owner missing')
        if binding['template']=='basic':
            if r['position']!=[0,0,0] or r['rotation']!=[0,0,0] or r['scale']!=[4096]*3:raise ValueError('Actual GameRoot was not inert')
        elif r['rotation']!=[0,45*(68*r['step']+16*r['step']//60),0]:raise ValueError('Actual selected Spinner differs from fixed60Hz independent arithmetic')

def base_proof(args,binding,required):
    return {'schema':1,'status':'missing','case':args.case,'dependency_sha256':args.dependency,'provider':binding['flavor'],'mode':'native_cpp',
        'executable_sha256':sha(binding['executable']),'export_manifest_sha256':sha(binding['export_manifest']),
        'normalized_trace_sha256':None,'scenarios':{name:{'passed':False} for name in required},'metrics':{name:None for name in METRICS},
        'streaming_passed':False,'music_passed':False,'peak_coverage_complete':False,'capture_lifecycle_passed':False,
        'ownership_edit_observed':False,'export_scenario_passed':False,
        'baseline_comparison':{'matched_route_settings':False,'instrumentation_same':False,'steps_equal':False},
        'rule_edit_checks':{'guard':False,'shop':False,'run_flow':False},'evidence_files':{},
        'source_inventory_sha256':digest(binding['source_inventory']),'actual_project':str(binding['project']),
        'actual_relocated_export':str(binding['export_root']),'missing':[]}

def main(argv=None):
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,required=True);parser.add_argument('--editor',type=Path)
    for name in ['project','export-root','export-manifest','output']:parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--case',required=True);parser.add_argument('--mode',required=True);parser.add_argument('--dependency',required=True)
    parser.add_argument('--queue',type=Path);parser.add_argument('--observe',action='store_true',help='Execute current unchanged Basic/Sample image and relocated image; requires integrator console lease')
    args=parser.parse_args(argv);args.root=args.root.resolve();args.output=args.output.resolve()
    binding=validate_binding(args.root,args.project,args.case,args.mode,args.dependency,args.export_root,args.export_manifest)
    queue=args.queue or args.root/'artifacts/gameplay-helpers/p10/current-acceptance-queue.json'
    required=read(queue)['matrix_contract']['scenarios'][binding['template']] if queue.exists() else []
    proof=base_proof(args,binding,required)
    work=args.output.with_name(args.output.stem+'-evidence')
    if work.exists():raise ValueError('Preserve earlier scenario evidence; choose a new output/dependency attempt')
    work.mkdir(parents=True)
    write(work/'source-inventory.json',binding['source_inventory'])
    write(work/'binding.json',{'project':str(binding['project']),'case':args.case,'dependency_sha256':args.dependency,'export_manifest_sha256':sha(binding['export_manifest']),'executable_sha256':sha(binding['executable'])})
    if binding['template'] in ['basic','sample'] and args.observe:
        original=capture(args.root,binding,binding['project']/'.epok/build',work/'original')
        exported=capture(args.root,binding,binding['export_root'],work/'exported')
        if original['trace']!=exported['trace']:raise ValueError('Relocated actual image changes semantic outputs')
        diagnostic=capture(args.root,binding,binding['project']/'.epok/build',work/'memory',True)
        if original['trace']!=diagnostic['trace']:raise ValueError('Memory observer changed actual equal-step semantic outputs')
        memory=json.loads(diagnostic['memory'])
        frames=original['frames'];times=[(f['cpu_cycles']*1_000_000+33_868_799)//33_868_800 for f in frames]
        rendering=binding['manifest']['rendering'];field_height=rendering['height']//2 if rendering['height']==480 else rendering['height']
        framebuffer=rendering['width']*field_height*4
        proof['metrics'].update(slow_frames=sum(t>25000 for t in times),max_frame_us=max(times),simulation_steps=len(original['trace']),cpu_work_units=original['cpu_work_units'],
             dropped_work=max(f['dropped_primitives']+f['dropped_steps'] for f in frames),rejected_work=max(f['actors'][2] for f in frames),
             heap_peak_bytes=memory['heap_peak_footprint_bytes'],stack_peak_bytes=memory['stack_peak_bytes'],
             vram_peak_bytes=framebuffer+max(max(f['resident_texture_bytes'],f['active_texture_bytes']) for f in frames),
             hud_drops=max(f['hud_dropped'] for f in frames),max_runtime_actors=max(f['actors'][5] for f in frames),
             max_components_per_actor=max(f['max_components_per_actor'] for f in frames))
        proof.update(normalized_trace_sha256=digest(original['trace']),capture_lifecycle_passed=True,export_scenario_passed=True)
        proof['observed_functional_scenarios']={'inert-root' if binding['template']=='basic' else 'selected-spinner-default':True,'equal_steps':120,'original_export_semantics_match':True}
        proof['memory_observer_coverage']=memory
        proof['measurement_scope']={'frame_time':'ActualGameScene entry/return targetCPUcycles, normal observer only; memory run excluded',
             'cpu_work':'Cumulative actual frame work ending at observed simulationstep120; no inferred intermediate transform rows',
             'vram':'Observed static resident/active texture high water + actual configured double field-buffer footprint',
             'target_instructions_added':0,'target_bss_added':0}
        proof['missing'] += ['Chosen-language edit/extension scenario and generic helper editor workflow not yet observed by this default-source route.',
                              'Matched accepted baseline route/instrumentation/equal-step comparison must be supplied; export equality alone is not that baseline.',
                              'Whole startup/dynamic peaks, audio/streaming applicability and exact required named scenario mapping still require reviewed coverage; do not autocertify.']
    else:
        proof['missing'].append('Actual-project driver pending for '+binding['template'] if binding['template'] not in ['basic','sample'] else 'Pass--observe under integrator console lease for actual image execution')
    if not required:proof['missing'].append('Reviewed queue scenario names missing')
    for p in work.rglob('*'):
        if p.is_file():proof['evidence_files'][str(p.resolve())]=sha(p)
    write(args.output,proof)
    print(json.dumps({'status':proof['status'],'output':str(args.output),'observed':proof.get('observed_functional_scenarios'),'missing':proof['missing']},indent=2))
    return 0

if __name__=='__main__':raise SystemExit(main())
