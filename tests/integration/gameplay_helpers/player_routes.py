"""Observe unchanged final18 ThirdPerson/Platformer images and relocated exports.

Only the input-device state is injected, at real simulation-step boundaries.
Target rules, poses, colliders, cameras, pickups and component bindings are not
replaced. Missing authoring/baseline/full-surface gates remain explicitly missing.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import adapter
import layout
from physical_support import plan as physical_plan,validate as validate_physical,normalized as normalized_physical
from capture_reuse import reconcile as reconcile_capture

HERE=Path(__file__).resolve().parent
STEPS=240
BASE_OBSERVER_SHA='b5fbcd382019896304baa502f62f94f8da86156b950422eaee6e1a43223ba43b'
TEMPLATES=('third-person','platformer-2d')
_LAYOUTS={}


def timeline(template):
    """P7 default-source input sequence; modal capture is a separate missing gate."""
    if template not in TEMPLATES:raise ValueError('Unsupported player route')
    result=[]
    for tick in range(STEPS):
        buttons=0;right_x=128
        if template=='third-person':
            if 10<=tick<80 or 110<=tick<160:buttons|=1<<4
            if 80<=tick<110 or 160<=tick<190:buttons|=1<<5
            if tick in (25,125):buttons|=1<<14
            if 40<=tick<60:right_x=200
        else:
            if 10<=tick<45:buttons|=1<<7
            if 45<=tick<210:buttons|=1<<5
            if tick in (60,130):buttons|=1<<14
        result.append({'step':tick+1,'buttons':buttons,'adc':[128,128,right_x,128]})
    return result


def roles(binding):
    scene=binding['scene'];installed=binding['installation']['roles'];actors=scene['actors']
    def unique(values,description):
        if len(values)!=1:raise ValueError('Actual source role not unique: '+description)
        return values[0]
    def actor_role(role):return unique([a for a in actors if a['class']['class_id']==installed[role]['class_id']],role)
    def component(actor,role=None,cpp=None):
        return unique([c for c in actor['components'] if c['class'].get('class_id')==installed[role]['class_id']] if role else
                      [c for c in actor['components'] if c['class']['name']==cpp],role or cpp)
    player=actor_role('player');animation=component(player,'locomotion');component(player,'controller')
    result={'player_name':player['name'],'animation_name':animation['name'],'player_id':player['id']}
    if binding['template']=='third-person':
        camera=actor_role('camera');component(camera,'camera_input')
        if player.get('properties',{}).get('camera')!=camera['id'] or camera.get('properties',{}).get('target')!=player['id'] or camera.get('attach'):
            raise ValueError('Actual camera/player typed bindings differ')
        motor=component(player,cpp='epok::CharacterMotor3DComponent')
        result.update(camera_name=camera['name'],motor_name=motor['name'])
    else:
        camera=unique([a for a in actors if any(c['class']['class_id']==installed['camera']['class_id'] for c in a['components'])],'camera owner')
        view=component(camera,'camera');motor=component(player,cpp='epok::CharacterMotor2DComponent')
        body=component(player,cpp='epok::CharacterBody2DComponent')
        if camera.get('attach',{}).get('actor')!=player['id'] or player.get('properties',{}).get('camera')!=view['id']:
            raise ValueError('Actual attached 2D camera binding differs')
        coins=[a for a in actors if any(c['class']['class_id']==installed['pickup']['class_id'] for c in a['components'])]
        if len(coins)!=8:raise ValueError('Actual eight selected-language coin components required')
        for coin in coins:
            pickup=component(coin,'pickup');props=pickup.get('properties') or {}
            if props.get('accepted_initiator')!=player['id'] or props.get('auto_collect') is not True:raise ValueError('Actual coin/player binding differs')
        lifts=[a for a in actors if any(c['class']['name']=='epok::PathFollow2DComponent' for c in a['components'])]
        lift=unique(lifts,'moving path platform')
        result.update(camera_name=camera['name'],camera_component_name=view['name'],motor_name=motor['name'],body_name=body['name'],
                      coin_names=[a['name'] for a in coins],lift_name=lift['name'])
    names=[a['name'] for a in actors]
    for name in [result['player_name'],result['camera_name'],*result.get('coin_names',[]),*([result['lift_name']] if 'lift_name' in result else [])]:
        if names.count(name)!=1 or len(name.encode('utf-8'))>32:raise ValueError('External observer needs unique actual bounded actor labels')
    return result


def actor_active_bit(debug):
    """Read the compiler's bitfield position; do not guess Actor's flag byte."""
    entries=[];current=None
    for line in debug.splitlines():
        match=re.match(r'^\s*<(\d+)><([0-9a-fA-F]+)>:.*\((DW_TAG_\w+)\)',line)
        if match:
            current={'depth':int(match[1]),'tag':match[3],'attrs':{}};entries.append(current)
        else:
            match=re.search(r'DW_AT_(\w+)\s*:\s*(.*)$',line)
            if match and current:current['attrs'][match[1]]=match[2].strip()
    def name(value):return value.rsplit('): ',1)[-1] if '): ' in value else re.sub(r'^\([^)]*\)\s*','',value)
    def number(value):
        value=re.sub(r'^\([^)]*\)\s*','',value)
        if not re.fullmatch(r'(?:0x[0-9a-fA-F]+|\d+)',value):raise ValueError('Unsupported exact flag encoding')
        return int(value,0)
    positions=set()
    for i,entry in enumerate(entries):
        if entry['tag'] not in ('DW_TAG_class_type','DW_TAG_structure_type') or name(entry['attrs'].get('name',''))!='Actor':continue
        for child in entries[i+1:]:
            if child['depth']<=entry['depth']:break
            attrs=child['attrs']
            if child['depth']==entry['depth']+1 and child['tag']=='DW_TAG_member' and name(attrs.get('name',''))=='m_active':
                if number(attrs.get('bit_size','0'))!=1 or 'data_bit_offset' not in attrs:raise ValueError('Missing exact Actor active bitfield layout')
                positions.add(number(attrs['data_bit_offset']))
    if len(positions)!=1:raise ValueError('Missing/ambiguous Actor active bitfield')
    position=positions.pop();return {'byte':position//8,'bit':position%8}


def inspect_layout(root,elf,template):
    # This reuses only the common compiled ABI reader, not Sample's scenario or
    # acceptance. Both player templates require the real tick_actor dispatcher.
    key=(str(root.resolve()),adapter.sha(elf),template)
    if key in _LAYOUTS:return _LAYOUTS[key]
    result=layout.inspect(root,elf,'sample')
    debug=layout.tool(root,'readelf','--debug-dump=info','--wide',elf)
    structures=layout.dwarf_structures(debug);syms=layout.symbols(root,elf)
    required={'ObjectRegistry':['slots','capacity'],'ObjectSlot':['instance','type','generation','state'],
      'ClassDescriptor':['family'],'ObjectId':['index','generation'],'Object':['m_id'],
      'Actor':['m_name','m_root','m_components','m_component_count','m_data'],
      'ActorComponent':['m_name'],'Input':['ports'],'State':['held','pending_pressed','pending_released','pressed','released','frame_pressed','frame_released','connected','analog','axes'],
      'GameplayControls':['m_edges'],'Edges':['frame_pressed','frame_released','pending_pressed','pending_released','tick_pressed','tick_released'],
      'LocomotionSample':['status','state','clip','rate'],'Runtime':['sample','phase','motor','visual','bank','updating','manual']}
    if template=='third-person':
        required.update(CharacterMotor3DComponent=['sample_'],CharacterMotionSample3D=['status','velocity','displacement','grounded','jumped','blocked'],
          GameplayVector3=['x','y','z'],CameraRig3D=['sample_'],CameraRigSample=['status','position','yaw','pitch','distance','blocked'],
          SkeletalLocomotionComponent=['runtime_'])
    else:
        required.update(Actor2D=['root'],SceneComponent2D=['transform','attach_parent'],Transform2D=['position','rotation','scale'],
          CharacterMotor2DComponent=['sample_'],CharacterMotionSample2D=['status','velocity','displacement','grounded','jumped','blocked'],
          GameplayVector2=['x','y'],CharacterBody2DComponent=['last_collider','last_hit_tile','last_tile','on_floor'],
          Camera2DComponent=['position','initialized','current'],SpriteLocomotion2DComponent=['runtime_'])
        required['CharacterBody2DComponent'].append('on_ceiling')
        result['actor_active_bit']=actor_active_bit(debug)
    result['layouts'].update({name:layout.require_layout(structures,name,members) for name,members in required.items()})
    if result['layouts']['ObjectId']['size']!=4:raise ValueError('Unsupported compiled ObjectId representation')
    for name in ['epok::active_object_registry','epok::input','epok::gameplay_controls()::value']:
        if name not in syms:raise ValueError('Missing exact input/registry symbol: '+name)
        result['symbols'][name]=syms[name]['address']
    result['observation_boundary']='player-next-tick-and-frame-return'
    _LAYOUTS[key]=result
    return result


def observer_source(config):
    base=HERE/'observe.lua'
    if adapter.sha(base)!=BASE_OBSERVER_SHA:raise ValueError('Frozen common observer changed; review player observer composition')
    source=base.read_text(encoding='utf-8')
    def replace_once(before,after):
        nonlocal source
        if source.count(before)!=1:raise ValueError('Common observer composition contract changed')
        source=source.replace(before,after,1)
    start=source.index("assert((config.template=='basic'")
    end=source.index('local actor_size=',start)
    source=source[:start]+"assert(config.observation_boundary=='player-next-tick-and-frame-return','Wrong player clock boundary')\n"+source[end:]
    start=source.index('local function snapshot_step(');end=source.index('local function counters()',start)
    policy=(HERE/'player_observe.lua').read_text(encoding='utf-8')
    if config.get('physical_route'):
        extension=(HERE/'physical_routes.lua').read_text(encoding='utf-8')
        marker='local function inject_input(tick)'
        assert policy.count(marker)==1
        policy=policy.replace(marker,extension+'\n'+marker,1)
        policy=policy.replace("local sample=assert(config.timeline[tick],'Missing exact input step')","local sample=physical_input(tick)",1)
        policy=policy.replace(' trace[#trace+1]=row',' physical_record(row,tick)\n trace[#trace+1]=row',1)
        if 'local sample=physical_input(tick)' not in policy or policy.count('\n physical_record(row,tick)\n')!=1:raise ValueError('Reviewed physical observer composition changed')
    source=source[:start]+policy+'\n'+source[end:]
    start=source.index(' for n=0,u32(config.symbols[\'epok::object_count\'])-1 do',source.index('local function counters()'))
    end=source.index(' return {ticks=',start)
    source=source[:start]+''' for _,value in ipairs(registry_actors())do maximum_components=math.max(maximum_components,u8(value+offset('Actor','m_component_count')))end
'''+source[end:]
    replace_once('step_observations=step_observations,clock_observer=',
                 'input_trace=input_trace,input_writes=input_writes,step_observations=step_observations,clock_observer=')
    if config.get('physical_route'):
        replace_once('input_trace=input_trace,input_writes=input_writes,step_observations=',
                     'physical_route=config.physical_route,physical_trace=physical_trace,pose_setups=pose_setups,physical_setup_observations=physical_setup_observations,input_trace=input_trace,input_writes=input_writes,step_observations=')
    start=source.index("if config.template=='sample' then\n assert(config.tick_entry")
    end=source.index('P10_FRAME=',start)
    source=source[:start]+'''P10_TICK=PCSX.addBreakpoint(config.tick_entry,'Exec',4,'P10 real player simulation input',guarded(function()
 if finished then return end
 local tick=u32(ticks_address)
 if tick==last_injected_tick then return end
 if last_injected_tick>0 then snapshot_step(last_injected_tick,'next-tick')end
 assert(tick==last_injected_tick+1,'Player input boundary skipped an actual Time tick')
 if tick<=config.steps then inject_input(tick)end
 last_injected_tick=tick
end))
'''+source[end:]
    replace_once("  if config.template=='basic' then snapshot_step('frame-return')end",
                 "  if last_injected_tick>0 then snapshot_step(last_injected_tick,'frame-return')end")
    return source.replace('@CONFIG@',adapter.lua_literal(config))


def validate_observation(value,binding):
    if value.get('reason')!='exact-steps' or value.get('errors'):raise ValueError('Incomplete/error actual player route')
    if value.get('executable_sha256')!=adapter.sha(binding['executable']) or value.get('source_inventory_sha256')!=adapter.digest(binding['source_inventory']):raise ValueError('Foreign actual player source/image')
    trace=value.get('trace',[]);inputs=value.get('input_trace',[]);frames=value.get('frames',[]);observations=value.get('step_observations',[])
    physical=value.get('physical_route');steps=physical_plan(binding)['steps'] if physical else STEPS
    if [r.get('step') for r in trace]!=list(range(1,steps+1)) or inputs[:STEPS]!=timeline(binding['template']):raise ValueError('Missing actual steps or different input timeline')
    if len(inputs)!=steps or any(r.get('step')!=i+1 for i,r in enumerate(inputs)):raise ValueError('Actual extended input sequence incomplete')
    if [r.get('step') for r in observations]!=list(range(1,steps+1)):raise ValueError('Missing actual step observations')
    clock=value.get('clock_observer',{})
    if clock.get('kind')!='player-next-tick-and-frame-return' or clock.get('write_callbacks')!=steps or clock.get('frames')!=len(frames):raise ValueError('Actual input/clock observation missing')
    if type(value.get('input_writes',{}).get('count')) is not int or value['input_writes']['count']<=0:raise ValueError('Actual device input writes unobserved')
    if not frames or [r.get('observer_frame') for r in frames]!=list(range(1,len(frames)+1)):raise ValueError('Missing actual frame sequence')
    total=0;previous_tick=0
    for index,row in enumerate(frames):
        if type(row.get('cpu_cycles')) is not int or row['cpu_cycles']<=0:raise ValueError('CPU work unmeasured')
        if any(type(row.get(k)) is not int for k in ['frame_sequence','start_ticks','ticks','executed_steps','requested_steps']):raise ValueError('Frame/clock counters missing')
        if row['start_ticks']!=previous_tick or row['ticks']-previous_tick!=row['executed_steps'] or row['requested_steps']!=row['executed_steps'] or not 0<=row['executed_steps']<=8:raise ValueError('Actual clock sequence inconsistent')
        if index and row['frame_sequence']!=frames[index-1]['frame_sequence']+1:raise ValueError('Published frame sequence inconsistent')
        total+=row['cpu_cycles'];previous_tick=row['ticks']
    if type(value.get('cpu_work_units')) is not int or not 0<value['cpu_work_units']<=total or previous_tick<steps:raise ValueError('Exact-step CPU work missing')
    if observations[-1]['cpu_work_units']!=value['cpu_work_units'] or any(b['cpu_work_units']<=a['cpu_work_units'] for a,b in zip(observations,observations[1:])):raise ValueError('Step CPU sequence inconsistent')
    cumulative=[0]
    for frame in frames:cumulative.append(cumulative[-1]+frame['cpu_cycles'])
    for row in observations:
        if row.get('boundary') not in ['next-tick','frame-return'] or type(row.get('frame')) is not int or not 1<=row['frame']<=len(frames):raise ValueError('Actual step boundary unobserved')
        frame=frames[row['frame']-1]
        if row.get('clock_read')!=row['step']+(row['boundary']=='next-tick'):raise ValueError('Step is not bound to the actual Time read')
        if not frame['start_ticks']<row['step']<=frame['ticks'] or not cumulative[row['frame']-1]<row['cpu_work_units']<=cumulative[row['frame']]:raise ValueError('Step CPU/clock outside observed frame')
    core=trace[:STEPS]
    for row in core:
        if row.get('motion_status')!=0 or row.get('animation_status')!=0 or not row.get('camera_valid'):raise ValueError('Actual player motor/animation/camera failed')
    if not any(r['jumped'] for r in core) or not any(r['grounded'] for r in core) or len({tuple(r['position']) for r in core})<2:raise ValueError('Actual movement/jump/ground behavior absent')
    if len({tuple(r['camera_position']) for r in core})<2:raise ValueError('Actual following camera did not move')
    if binding['template']=='third-person':
        if len({r['camera_yaw'] for r in trace[40:70]})<2:raise ValueError('Actual camera orbit input ignored')
        if not {0,1,2,3,4,5}<={r['animation_state'] for r in core}:raise ValueError('Actual six-state locomotion coverage incomplete')
    else:
        if not any(not all(r['coins_active']) for r in core):raise ValueError('Actual selected-language coin callback never collected')
        if len({tuple(r['lift_position']) for r in core})<2 or not any(r['hit_tile'] for r in core):raise ValueError('Actual platform/tile behavior unobserved')
    if physical:validate_physical(value,binding)
    return {'movement':True,'jump':True,'following_camera':True,'selected_animation':True,
            **({'orbit_camera':True} if binding['template']=='third-person' else {'selected_coin':True,'moving_platform':True,'tile_contact':True})}


def capture(root,binding,program,output,diagnostic=False,physical=False):
    output.mkdir(parents=True,exist_ok=False);elf=program/'epok.elf';exe=program/'epok.ps-exe'
    if not elf.is_file() or adapter.sha(exe)!=adapter.sha(binding['executable']):raise ValueError('Exact actual image/debug ELF missing')
    compiled=inspect_layout(root,elf,binding['template'])
    selected=roles(binding)
    config=dict(compiled,**selected,template=binding['template'],steps=STEPS,timeline=timeline(binding['template']),
      actor_name=selected['player_name'],diagnostic_memory=diagnostic,output=str((output/'observation.json').resolve()).replace('\\','/'),
      source_inventory_sha256=adapter.digest(binding['source_inventory']),executable_sha256=adapter.sha(exe),
      performance_words=compiled['elf_symbol_sizes']['epok::performance_stats']//4,hud_words=compiled['elf_symbol_sizes']['epok::hud_stats']//4)
    if physical:config['physical_route']=physical_plan(binding);config['steps']=config['physical_route']['steps']
    adapter.write(output/'config.json',config);source=''
    if diagnostic:
        spec=importlib.util.spec_from_file_location('player_memory',root/'tools/profile_runtime_memory.py');module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module;spec.loader.exec_module(module)
        memory,writes=module.elf_probe_config(elf,root/'.tools/mips/bin');adapter.write(output/'memory-config.json',memory);source+=module.lua_probe(memory,writes)
    (output/'observe.lua').write_text('-- Automated run: keep the SPU running and silence only its output.\npcall(function() PCSX.settings.spu.Mute=true end)\n'+source+observer_source(config),encoding='utf-8')
    emulator=next((p for p in [root/'.tools/redux/pcsx-redux.main',root/'.tools/redux/pcsx-redux.exe'] if p.is_file()),None)
    bios=next((p for p in [root/'.tools/redux/openbios-fastboot.bin',root/'.tools/redux/openbios.bin'] if p.is_file()),None)
    if not emulator or not bios:raise ValueError('Pinned emulator/BIOS missing')
    portable=output/'portable';portable.mkdir()
    command=[str(emulator),'-portable',str(portable),'-logfile',str(output/'runtime.log'),'-run','-stdout','-fastboot','-noupdate','-interpreter','-softgpu','-2mb','-no-gdb','-no-ui','-bios',str(bios),'-loadexe',str(exe),'-dofile',str(output/'observe.lua'),'-debugger']
    command+=['-pcdrv','-pcdrvbase',str(program)] if (program/'GEOMETRY.BIN').exists() else ['-no-pcdrv']
    with (output/'process.log').open('w',encoding='utf-8') as log:
        process=subprocess.Popen(command,cwd=portable,stdout=log,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        try:code=process.wait(timeout=900 if diagnostic else 240)
        except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=15);raise ValueError('Actual player route timed out; preserve logs')
    adapter.write(output/'command.json',{'argv':command,'exit_code':code,'executable_sha256':adapter.sha(exe),'elf_sha256':adapter.sha(elf)})
    if code:raise ValueError('Actual player route failed; see '+str(output/'process.log'))
    value=adapter.read(output/'observation.json')
    if value.get('diagnostic_memory') is not diagnostic or value.get('clock_observer',{}).get('address')!=compiled['tick_counter_address']:raise ValueError('Wrong actual clock/diagnostic observation')
    expected_ranges=[{'address':compiled['symbols']['epok::input']+compiled['layouts']['Input']['members']['ports'],'bytes':compiled['layouts']['State']['size']},
                     {'address':compiled['symbols']['epok::gameplay_controls()::value']+compiled['layouts']['GameplayControls']['members']['m_edges'],'bytes':compiled['layouts']['Edges']['size']}]
    if value.get('input_writes',{}).get('ranges')!=expected_ranges:raise ValueError('Input writes are not bound to this exact compiled device state')
    validate_observation(value,binding)
    if adapter.inventory(binding['project'])!=binding['source_inventory']:raise ValueError('Actual authored sources changed during route')
    return value


def measured_metrics(observed,memory,render):
    # All normal frames count toward budgets, including startup, neutral/setup
    # frames and the physical tail. Only equal-step CPU comparison uses prefix240.
    frames=observed['frames']
    prefix=[f for f in frames if f['start_ticks']<STEPS]
    if not frames or not prefix or len(observed['step_observations'])<STEPS:
        raise ValueError('Missing full measured frames or exact prefix CPU observations')
    microseconds=lambda f:(f['cpu_cycles']*1000000+33868799)//33868800
    times=[microseconds(f)for f in frames];prefix_times=[microseconds(f)for f in prefix]
    baseline=dict(simulation_steps=STEPS,cpu_work_units=observed['step_observations'][STEPS-1]['cpu_work_units'],
        observed_frames=len(prefix),slow_frames=sum(t>25000 for t in prefix_times),max_frame_us=max(prefix_times))
    height=render['height']//2 if render['height']==480 else render['height']
    metrics=dict(slow_frames=sum(t>25000 for t in times),max_frame_us=max(times),
        simulation_steps=baseline['simulation_steps'],cpu_work_units=baseline['cpu_work_units'],
        dropped_work=max(f['dropped_primitives']+f['dropped_steps'] for f in frames),
        rejected_work=max(f['actors'][2]for f in frames),
        heap_peak_bytes=memory['heap_peak_footprint_bytes'],stack_peak_bytes=memory['stack_peak_bytes'],
        vram_peak_bytes=render['width']*height*4+max(max(f['resident_texture_bytes'],f['active_texture_bytes'])for f in frames),
        hud_drops=max(f['hud_dropped']for f in frames),max_runtime_actors=max(f['actors'][5]for f in frames),
        max_components_per_actor=max(f['max_components_per_actor']for f in frames))
    return metrics,baseline


def main(argv=None):
    parser=argparse.ArgumentParser()
    for name in ['root','project','export-root','export-manifest','output']:parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--editor',type=Path);parser.add_argument('--queue',type=Path)
    for name in ['case','mode','dependency']:parser.add_argument('--'+name,required=True)
    parser.add_argument('--surfaces',action='store_true',help='Add explicit setup + separately measured P7 physical scenarios; keep the240-step default prefix unchanged')
    parser.add_argument('--observe',action='store_true');args=parser.parse_args(argv)
    args.root=args.root.resolve();args.output=args.output.resolve()
    binding=adapter.validate_binding(args.root,args.project,args.case,args.mode,args.dependency,args.export_root,args.export_manifest)
    if binding['template'] not in TEMPLATES:raise ValueError('Foreign player route template')
    required=adapter.read(args.queue or args.root/'artifacts/gameplay-helpers/p10/current-acceptance-queue.json')['matrix_contract']['scenarios'][binding['template']]
    proof=adapter.base_proof(args,binding,required);work=args.output.with_name(args.output.stem+'-evidence');work.mkdir(parents=True,exist_ok=False)
    adapter.write(work/'source-inventory.json',binding['source_inventory']);adapter.write(work/'source-roles.json',roles(binding))
    if args.observe:
        original=capture(args.root,binding,binding['project']/'.epok/build',work/'original',physical=args.surfaces)
        exported=capture(args.root,binding,binding['export_root'],work/'exported',physical=args.surfaces)
        normalize=normalized_physical if args.surfaces else lambda value:value['trace']
        if normalize(original)!=normalize(exported) or original['input_trace']!=exported['input_trace']:raise ValueError('Relocated actual player image changes equal-step semantics')
        memory=capture(args.root,binding,binding['project']/'.epok/build',work/'memory',True,physical=args.surfaces)
        if normalize(original)!=normalize(memory):raise ValueError('Diagnostic memory observer changed actual semantic trace')
        peak=json.loads(memory['memory'])
        metrics,baseline=measured_metrics(original,peak,binding['manifest']['rendering'])
        proof['metrics'].update(metrics)
        proof['baseline_frame_metrics']=baseline
        proof['budget_observation_scope']={'normal_frames':len(original['frames']),
            'simulation_steps':len(original['step_observations']),
            'cpu_work_units':original['step_observations'][-1]['cpu_work_units'],
            'includes_startup_setup_and_physical_phases':True,'diagnostic_frames_used_for_frame_time':False}
        proof.update(normalized_trace_sha256=adapter.digest(normalize(original)),capture_lifecycle_passed=True,export_scenario_passed=True,
                     observed_functional_scenarios=validate_observation(original,binding),input_timeline_sha256=adapter.digest(original['input_trace']),memory_observer_coverage=peak)
        proof['measurement_scope']={'input':'Exact per-Time-step device state replay; only Input.State/GameplayControls.Edges writes, no target calls or world writes',
          'cpu':'Every actual normal GameScene frame counts toward25ms/drop/VRAM/actor budgets; exact prefix240 cumulative CPU is separate; diagnostic frame time excluded',
          'target_instructions_added':0,'target_bss_added':0}
        if args.surfaces:
            proof['physical_scenarios']=validate_physical(original,binding)
            proof['cold_restart_passed']=normalize(original)==normalize(exported)
            proof['restart_scope']='Independent original/export fresh-process startup; no nonexistent death/restart binding is invented'
            proof['measurement_scope'].update(input='Original240-step input/device-only route unchanged; explicit tail position-only setup, never measured-phase pose/velocity/rule forcing',
              cpu='All normal frames including physical setup/tail count toward performance/resource budgets; only exact prefix240 CPU work and baseline_frame_metrics use the original route',
              physical_comparison='Phase-local actual motor/camera/tile/carry samples; setup rows and addresses excluded')
            proof['input_capture_reference']=reconcile_capture(args.root,binding)
            proof['input_capture_mechanism_passed']=proof['input_capture_reference']['status']=='reused-unaffected-capture-mechanism'
            proof['evidence_files'].update(proof['input_capture_reference']['evidence_files'])
        proof['missing']+=['Chosen-language editing/inheritance and common-editor helper workflow remain unobserved.',
          'Matched accepted baseline route/instrumentation/equal-step CPU comparison remains missing.',
          'Whole startup/peak coverage, sound/streaming applicability and exact required named-scenario coverage require reviewed evidence.',
          *([] if args.surfaces and proof['input_capture_mechanism_passed'] else ['Input capture mechanism reference missing/changed; no private capture/rule state is forced to fabricate this gate.']),
          *([] if args.surfaces else ['Full one-way traversal and moving-lift carry (2D), or ramp/wall/boom-occlusion route (3D), require --surfaces with explicit pre-measurement setups.'])]
    else:proof['missing'].append('Run --observe under the integrator console lease; current program was not executed.')
    collectors={Path(__file__).resolve(),Path(adapter.__file__).resolve(),Path(layout.__file__).resolve(),HERE/'player_observe.lua',HERE/'observe.lua'}
    if args.surfaces:collectors|={HERE/'physical_support.py',HERE/'physical_routes.lua',HERE/'capture_reuse.py'}
    for p in collectors:
        if not p.is_file():raise ValueError('Collector provenance source missing: '+str(p))
        proof['evidence_files'][str(p.resolve())]=adapter.sha(p)
    for p in work.rglob('*'):
        if p.is_file():proof['evidence_files'][str(p.resolve())]=adapter.sha(p)
    adapter.write(args.output,proof);print(json.dumps({'status':proof['status'],'observed':proof.get('observed_functional_scenarios'),'missing':proof['missing']},indent=2))
    return 0


if __name__=='__main__':raise SystemExit(main())
