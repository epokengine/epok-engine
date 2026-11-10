"""Retained real-language edit/inheritance/helper workflows; no live-row edits.

--prepare uses public editor class/catalog/placement commands on source-only
copies of exact current projects. --build and --observe require root's console
lease. This script never exports, rewrites generated C++, or accepts RAM/perf.
"""
import argparse
import copy
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import uuid

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p / 'tools/epok_documents.py').is_file())
COMMON=HERE
sys.path.insert(0,str(COMMON));sys.path.insert(0,str(ROOT/'tools'))
import adapter
import layout
import epok_documents as documents

NAMESPACE=uuid.UUID('18263f9d-d972-530a-91ac-739c4ed39dc4')
BASE_SHA='b5fbcd382019896304baa502f62f94f8da86156b950422eaee6e1a43223ba43b'
DERIVED='P10AuthoredExtension'
TRIGGER='P10EditableTrigger'
STEPS=120


def stable(name):return str(uuid.uuid5(NAMESPACE,'p10-authored-workflow:'+name))
def put(path,value):documents.write_text(path,documents.dumps(value) if isinstance(value,dict) else value)
def scene(project):
    manifest_path=next(project.glob('*.epokproject'));manifest=documents.loads(manifest_path.read_text(encoding='utf-8'))
    return project/manifest['startup_scene'],manifest
def installed(project):return documents.loads((project/'ProjectSettings/Template.epoksettings').read_text(encoding='utf-8'))
def class_named(registry,name):
    found=[c for c in registry['classes'] if c['cpp_name']==name]
    if len(found)!=1:raise ValueError('Real compiled catalog class missing/ambiguous: '+name)
    return found[0]


def clone_sources(original,project):
    inputs=adapter.inventory(original)
    if project.exists():raise ValueError('Preserve existing retained edit fixture; use its workflow receipt')
    project.mkdir(parents=True)
    for relative,expected in inputs.items():
        source=original/relative;target=project/relative
        if not adapter.contained(target,project):raise ValueError('Escaping authored source')
        target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
        if adapter.sha(target)!=expected:raise ValueError('Source-only clone changed an authored input')
    if (project/'.epok').exists():raise ValueError('Original build/cache must not be cloned')
    return inputs


def edit_selected_source(project,template,flavor,registry,graph_reference):
    name='GameRoot' if template=='basic' else 'Spinner'
    if flavor=='cpp':
        path=project/'assets/scripts'/(name+'.hpp');source=path.read_text(encoding='utf-8')
        if template=='basic':
            at=source.rfind('};');assert at>0
            source=source[:at]+'''    EPOK_PROPERTY(EditAnywhere) epok::Fixed speed=45.0;
    void tick(epok::Fixed dt) override {
        if(auto* value=data()) value->transform.rotation[1]+=speed*dt;
    }
'''+source[at:]
        else:
            source,count=re.subn(r'\bspeed\s*=\s*45\.0\s*;', 'speed=90.0;',source)
            if count!=1:raise ValueError('Reviewed default Spinner45 source changed')
        put(path,source)
    elif flavor=='lua':
        path=project/'assets/scripts'/(name+'.lua');source=path.read_text(encoding='utf-8')
        if template=='basic':
            before='return GameRoot'
            if source.count(before)!=1:raise ValueError('Actual GameRoot declaration changed')
            source=source.replace(before,'''GameRoot.speed = 45.0

---@override
function GameRoot:tick(delta_seconds)
    self.rotation.y = self.rotation.y + self.speed * delta_seconds
end

return GameRoot''')
        else:
            source,count=re.subn(r'\bSpinner\.speed\s*=\s*45\.0\b','Spinner.speed = 90.0',source)
            if count!=1:raise ValueError('Reviewed default Lua Spinner45 changed')
        put(path,source)
    else:
        path=project/'assets/Blueprints'/(name+'.epokbp');asset=documents.loads(path.read_text(encoding='utf-8'))
        if template=='basic':
            # Ordinary authored graph, using the same public transform nodes as
            # shipped Spinner, with Self (Actor) instead of Get Owner (Component).
            reference=documents.loads(graph_reference.read_text(encoding='utf-8'))
            tick=copy.deepcopy(next(f for f in reference['functions'] if f['name']=='tick'))
            speed=copy.deepcopy(next(v for v in reference['variables'] if v['name']=='speed'))
            ids={tick['id'],speed['id'],*(n['id'] for n in tick['nodes'])}
            mapping={value:stable('basic-graph:'+value) for value in ids}
            def remap(value):
                if isinstance(value,str):return mapping.get(value,value)
                if isinstance(value,list):return [remap(v) for v in value]
                if isinstance(value,dict):return {mapping.get(k,k):remap(v) for k,v in value.items()}
                return value
            tick=remap(tick);speed=remap(speed);speed['default']=45.0
            actor=class_named(registry,'epok::Actor');event=next(f for f in actor['functions'] if f['name']=='tick' and f.get('event'))
            tick['override_id']=event['id']
            for node in tick['nodes']:
                if node.get('kind',{}).get('operation',{}).get('kind')=='get_owner':node['kind']['operation']['kind']='self_object'
            asset['functions']=[tick];asset['variables']=[speed]
        else:
            variable=next(v for v in asset['variables'] if v['name']=='speed')
            if variable['default']!=45.0:raise ValueError('Reviewed Blueprint Spinner45 default changed')
            variable['default']=90.0
        put(path,asset)
    return path


class Workflow:
    def __init__(self,root,editor,original,output,graph_reference):
        self.root=root.resolve();self.editor=editor.resolve();self.original=original.resolve();self.output=output.resolve()
        self.project=self.output/'project';self.marker=self.output/'workflow.json';self.graph_reference=graph_reference.resolve()
        if adapter.contained(self.output,self.original):raise ValueError('Retained edit workflow must stay outside the original final18 project/cache')
        self.output.mkdir(parents=True,exist_ok=True)
        self.installation=installed(self.original);self.template=self.installation['template'];self.flavor=self.installation['gameplay']
        if self.template not in ['basic','sample'] or self.flavor not in ['cpp','blueprint','lua']:raise ValueError('Foreign small-starter workflow')
        if scene(self.original)[1].get('lua_execution')!='native_cpp':raise ValueError('Chosen-language workflow requires the final native AOT mode')
        self.commands=[]
    def run(self,*arguments):
        command=[str(self.editor),*map(str,arguments)]
        result=subprocess.run(command,cwd=self.root,capture_output=True,text=True,encoding='utf-8',timeout=900)
        log=self.output/('editor-'+str(len(list(self.output.glob('editor-*.log'))))+'.log')
        log.write_text(result.stdout+result.stderr,encoding='utf-8')
        receipt=dict(argv=command,exit_code=result.returncode,log=str(log),log_sha256=adapter.sha(log));self.commands.append(receipt)
        if result.returncode:raise ValueError('Actual editor API failed: '+str(log))
        return result.stdout.strip()
    def catalog(self):
        value=json.loads(self.run('--project',self.project,'--class-registry','--reflect'))
        if value.get('catalog_compiled') is not True:raise ValueError('Real all-provider catalog required')
        return value
    def verify_original(self,origin):
        if adapter.inventory(self.original)!=origin:raise ValueError('Original final18 authored project changed; edit fixture may not certify this row')
    def prepare(self):
        if self.marker.exists():
            receipt=adapter.read(self.marker);self.verify_original(receipt['original_inventory'])
            if adapter.inventory(self.project)!=receipt['edited_inventory']:raise ValueError('Retained workflow source changed; preserve it for review')
            return receipt
        origin=clone_sources(self.original,self.project)
        before=self.catalog();role=self.installation['roles']['game_root' if self.template=='basic' else 'spinner'];base=class_named(before,role['name'])
        selected_source=edit_selected_source(self.project,self.template,self.flavor,before,self.graph_reference)
        self.catalog() # Type-check the actual edited selected-language body.
        flag={'cpp':'--new-script','blueprint':'--new-blueprint','lua':'--new-lua-class'}[self.flavor]
        self.run('--project',self.project,flag,DERIVED,'--parent',role['class_id'] if self.flavor=='blueprint' else role['name'])
        registry=self.catalog();derived=class_named(registry,DERIVED)
        if derived.get('parent')!=base['id']:raise ValueError('Public class creator did not preserve actual selected-language parent')
        second_name='P10InheritedRule'
        target=derived if self.template=='basic' else class_named(registry,'epok::Actor3D')
        self.run('--project',self.project,'--add-actor',target['id']+':'+second_name)
        path,manifest=scene(self.project);value=documents.loads(path.read_text(encoding='utf-8'))
        if self.template=='basic':primary=next(a for a in value['actors'] if a['class']['class_id']==base['id'])
        else:primary=next(a for a in value['actors'] if any(c['class']['class_id']==base['id'] for c in a['components']))
        secondary=next(a for a in value['actors'] if a['name']==second_name)
        speed=45 if self.template=='basic' else 90
        if self.template=='basic':secondary.setdefault('properties',{})['speed']=speed*2;secondary['overrides']=list(secondary['properties'])
        else:
            secondary['components'].append(dict(id=stable(self.flavor+':derived-component'),name='Authored inherited rule',
              **{'class':{'name':DERIVED,'class_id':derived['id']}},properties={'speed':speed*2},overrides=['speed']))
        put(path,value)
        helper=class_named(registry,'epok::TriggerBox3D')
        # Public placement validates the entire scene, including the new derived
        # component and its inherited property. This is not a synthetic fixture.
        self.run('--project',self.project,'--add-actor',helper['id']+':'+TRIGGER)
        value=documents.loads(path.read_text(encoding='utf-8'));trigger=next(a for a in value['actors'] if a['name']==TRIGGER)
        sensor=next(c for c in trigger['components'] if c['class']['name']=='epok::InteractionSensor3DComponent')
        sensor.setdefault('properties',{}).update(width=4.0,height=3.0,depth=2.0);sensor['overrides']=list(sensor['properties']);put(path,value)
        final=self.catalog();class_named(final,DERIVED)
        receipt=dict(schema=1,status='prepared-runtime-missing',original=str(self.original),project=str(self.project),template=self.template,flavor=self.flavor,
          original_inventory=origin,edited_inventory=adapter.inventory(self.project),editor_sha256=adapter.sha(self.editor),
          selected_source=str(selected_source.relative_to(self.project)),selected_source_sha256=adapter.sha(selected_source),
          base_class_id=base['id'],derived_class_id=derived['id'],parentage_validated=True,primary_name=primary['name'],secondary_name=second_name,
          primary_speed=speed,secondary_speed=speed*2,helper_name=TRIGGER,helper_class_id=helper['id'],helper_half_extents=[8192,6144,4096],
          graph_reference_sha256=adapter.sha(self.graph_reference) if self.template=='basic' and self.flavor=='blueprint' else None,
          commands=self.commands,original_cache_modified=False)
        self.verify_original(origin);adapter.write(self.marker,receipt);return receipt
    def build(self,receipt):
        self.verify_original(receipt['original_inventory'])
        if adapter.inventory(self.project)!=receipt['edited_inventory']:raise ValueError('Edited authored input changed before build')
        self.run('--project',self.project,'--analyze-memory')
        if adapter.inventory(self.project)!=receipt['edited_inventory']:raise ValueError('Build changed edited authored inputs')
        build=self.project/'.epok/build';adapter.write(self.output/'build.json',dict(editor_sha256=adapter.sha(self.editor),
          executable_sha256=adapter.sha(build/'epok.ps-exe'),elf_sha256=adapter.sha(build/'epok.elf'),source_inventory_sha256=adapter.digest(receipt['edited_inventory']),
          command=self.commands[-1],scope='Edited-image build only; no peak/performance/audio acceptance'))


def observer_source(config):
    base=COMMON/'observe.lua'
    if adapter.sha(base)!=BASE_SHA:raise ValueError('Frozen observer changed; review edit workflow composition')
    source=base.read_text(encoding='utf-8')
    start=source.index("assert((config.template=='basic'");end=source.index('local actor_size=',start)
    source=source[:start]+"assert(config.observation_boundary=='selected-actor-tick-return','Edited rule requires real tick callbacks')\n"+source[end:]
    start=source.index('local function snapshot_step(');end=source.index('local function counters()',start)
    source=source[:start]+(HERE/'angular_observer.lua').read_text(encoding='utf-8')+'\n'+source[end:]
    start=source.index("if config.template=='sample' then\n assert(config.tick_entry");end=source.index('P10_FRAME=',start)
    source=source[:start]+'''P10_TICK=PCSX.addBreakpoint(config.tick_entry,'Exec',4,'P10 real authored extension tick',guarded(function()
 at_return('tick',function()snapshot_step('tick-return')end)
end))
'''+source[end:]
    source=source.replace("  if config.template=='basic' then snapshot_step('frame-return')end",'')
    return source.replace('@CONFIG@',adapter.lua_literal(config))


def validate_observation(value,receipt,executable):
    if value.get('reason')!='exact-steps' or value.get('errors') or value.get('executable_sha256')!=adapter.sha(executable):raise ValueError('Edited actual image observation incomplete/foreign')
    if value.get('source_inventory_sha256')!=adapter.digest(receipt['edited_inventory']):raise ValueError('Edited source provenance mismatch')
    trace=value.get('trace',[])
    if [r.get('step') for r in trace]!=list(range(1,121)):raise ValueError('Actual authored edits did not produce every observed step')
    for row in trace:
        total=68*row['step']+16*row['step']//60
        if row.get('owner') is not True or row.get('secondary_owner') is not True:raise ValueError('Authored ordinary actor owners missing')
        if row['rotation']!=[0,receipt['primary_speed']*total,0] or row.get('secondary_rotation')!=[0,receipt['secondary_speed']*total,0]:raise ValueError('Selected-language edit/inherited runtime rule differs from independent60Hz arithmetic')
        if row.get('helper_enabled') is not True or row.get('helper_trigger') is not True or row.get('helper_half_extents')!=receipt['helper_half_extents']:raise ValueError('Actual placed helper did not apply edited inspector tuning')
    if not value.get('frames') or type(value.get('cpu_work_units')) is not int or value['cpu_work_units']<=0:raise ValueError('Actual edited execution unobserved')


def observe(workflow,receipt,baseline=None):
    root=workflow.root;build=workflow.project/'.epok/build';expected=adapter.read(workflow.output/'build.json')
    if expected['executable_sha256']!=adapter.sha(build/'epok.ps-exe') or expected['source_inventory_sha256']!=adapter.digest(adapter.inventory(workflow.project)):raise ValueError('Edited linked image is stale')
    compiled=layout.inspect(root,build/'epok.elf','sample')
    structures=layout.dwarf_structures(layout.tool(root,'readelf','--debug-dump=info','--wide',build/'epok.elf'))
    compiled['layouts']['ActorData']=layout.require_layout(structures,'ActorData',['transform','name','owner','alive','audio','collider'])
    compiled['layouts']['Collider']=layout.require_layout(structures,'Collider',['enabled','trigger','half_extents'])
    output=workflow.output/'runtime';output.mkdir(exist_ok=False)
    config=dict(compiled,template=workflow.template,actor_name=receipt['primary_name'],secondary_name=receipt['secondary_name'],
      helper_name=receipt['helper_name'],primary_speed=receipt['primary_speed'],secondary_speed=receipt['secondary_speed'],steps=120,
      diagnostic_memory=False,source_inventory_sha256=adapter.digest(receipt['edited_inventory']),executable_sha256=adapter.sha(build/'epok.ps-exe'),
      performance_words=compiled['elf_symbol_sizes']['epok::performance_stats']//4,hud_words=compiled['elf_symbol_sizes']['epok::hud_stats']//4,
      output=str((output/'observation.json').resolve()).replace('\\','/'))
    adapter.write(output/'config.json',config);(output/'observe.lua').write_text('-- Automated run: keep the SPU running and silence only its output.\npcall(function() PCSX.settings.spu.Mute=true end)\n'+observer_source(config),encoding='utf-8')
    emulator=next(p for p in [root/'.tools/redux/pcsx-redux.main',root/'.tools/redux/pcsx-redux.exe'] if p.is_file())
    bios=next(p for p in [root/'.tools/redux/openbios-fastboot.bin',root/'.tools/redux/openbios.bin'] if p.is_file())
    portable=output/'portable';portable.mkdir()
    command=[str(emulator),'-portable',str(portable),'-logfile',str(output/'runtime.log'),'-run','-stdout','-fastboot','-noupdate','-interpreter','-softgpu','-2mb','-no-gdb','-no-ui','-bios',str(bios),'-loadexe',str(build/'epok.ps-exe'),'-dofile',str(output/'observe.lua'),'-debugger']
    command+=['-pcdrv','-pcdrvbase',str(build)] if (build/'GEOMETRY.BIN').exists() else ['-no-pcdrv']
    with (output/'process.log').open('w',encoding='utf-8') as log:
        process=subprocess.Popen(command,cwd=portable,stdout=log,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        try:code=process.wait(timeout=240)
        except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=15);raise ValueError('Edited workflow timed out; retain evidence')
    if code:raise ValueError('Edited-image observer failed; retain runtime logs')
    value=adapter.read(output/'observation.json');validate_observation(value,receipt,build/'epok.ps-exe')
    if adapter.inventory(workflow.project)!=receipt['edited_inventory']:raise ValueError('Authored edited fixture changed during execution')
    reused=None
    if baseline:
        original=workflow.original;bound={'template':workflow.template,'source_inventory':receipt['original_inventory'],'executable':original/'.epok/build/epok.ps-exe'}
        prior=adapter.read(baseline);adapter.validate_observation(prior,bound,120,False)
        if workflow.template=='sample' and any(now['rotation'][1]!=old['rotation'][1]*2 for now,old in zip(value['trace'],prior['trace'])):raise ValueError('Authored speed edit did not double accepted baseline outputs')
        reused=dict(path=str(baseline.resolve()),sha256=adapter.sha(baseline),scope='Current source/image functional default baseline only; not a matched performance baseline')
    workflow.verify_original(receipt['original_inventory'])
    result=dict(schema=1,status='authored-edit-extension-helper-observed',template=workflow.template,flavor=workflow.flavor,
      source_edit_observed=True,selected_language=workflow.flavor,inheritance_observed=True,public_class_creation_observed=True,
      public_helper_placement_observed=True,helper_tuning_observed=True,normalized_trace_sha256=adapter.digest(value['trace']),
      executable_sha256=expected['executable_sha256'],original_inventory_sha256=adapter.digest(receipt['original_inventory']),edited_inventory_sha256=adapter.digest(receipt['edited_inventory']),
      baseline_functional_reuse=reused,original_sources_and_cache_preserved=True,
      ram_performance_audio_accepted=False,missing=['Interactive designer manipulation is not claimed by the headless public API workflow.','Edited scene startup/peak/audio/performance/standalone gates are not measured or auto-accepted.'],
      evidence_files={str(p.resolve()):adapter.sha(p) for p in workflow.output.rglob('*') if p.is_file() and '.epok' not in p.parts and p.name!='result.json'})
    adapter.write(workflow.output/'result.json',result);return result


def main(argv=None):
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=ROOT);parser.add_argument('--editor',type=Path,default=ROOT/'target/debug/epok-editor.exe')
    parser.add_argument('--project',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--graph-reference',type=Path,default=ROOT/'artifacts/gameplay-helpers/p10/final18/projects/sample--blueprint/assets/Blueprints/Spinner.epokbp')
    parser.add_argument('--baseline-observation',type=Path)
    for phase in ['prepare','build','observe']:parser.add_argument('--'+phase,action='store_true')
    args=parser.parse_args(argv)
    if not any([args.prepare,args.build,args.observe]):raise ValueError('Select an explicit workflow phase')
    workflow=Workflow(args.root,args.editor,args.project,args.output,args.graph_reference)
    receipt=workflow.prepare() if args.prepare else adapter.read(workflow.marker)
    if args.build:workflow.build(receipt)
    if args.observe:print(json.dumps(observe(workflow,receipt,args.baseline_observation),indent=2))
    return 0


if __name__=='__main__':raise SystemExit(main())
