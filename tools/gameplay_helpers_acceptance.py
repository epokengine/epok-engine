"""Resumable final18 AOT acceptance ledger. Nothing passes from startup alone.

Prepare creates18 primary projects. Lua authors real rules, compiled to native
C++ AOT. VM engine regressions remain separate from this delivery runner.
Build/Play/scenario/export work shares one explicit exclusive console lock.
The ownership contracts and scenario adapters are reviewed JSON inputs. Missing
contracts, adapters, metrics or receipts remain missing, never a green result.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager, nullcontext
from datetime import datetime, timezone
import fnmatch
import hashlib
import json
import os
# Automated Play is silent; emulation and audio events are unchanged.
os.environ.setdefault('EPOK_MUTE_AUDIO','1')
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid

SCHEMA=1
STAGES=('generation','source','reflection','build','native_play','psx_play','scenario','export','resources')
EXECUTION={'native_cpp':0}
VOLATILE={'.epok','exports','UserSettings','.git','target','node_modules','__pycache__'}
SOURCE_EXTENSIONS={'.cpp','.hpp','.hh','.h','.c','.lua','.epokbp'}

def utc():return datetime.now(timezone.utc).isoformat()
def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def fingerprint(value):return hashlib.sha256(canonical(value)).hexdigest()
def read(path):return json.loads(Path(path).read_text(encoding='utf-8'))
def write(path,value):
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
    temporary=path.with_name(path.name+'.'+uuid.uuid4().hex+'.tmp')
    temporary.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    temporary.replace(path)
def contained(path,parent):
    try:Path(path).resolve().relative_to(Path(parent).resolve());return True
    except ValueError:return False
def files(root,exclude=VOLATILE):
    root=Path(root)
    if not root.exists():return
    for base,dirs,names in os.walk(root,followlinks=False):
        dirs[:]=sorted(d for d in dirs if d not in exclude and not (Path(base)/d).is_symlink())
        for name in sorted(names):
            path=Path(base)/name
            if path.is_symlink():raise RuntimeError('Linked input requires explicit provenance: '+str(path))
            yield path
def inventory(root,exclude=VOLATILE):return {p.relative_to(root).as_posix():digest(p) for p in files(root,exclude)}
def toolchain_inventory(root):
    result={}
    for path in files(root):
        relative=path.relative_to(root)
        if path.suffix.lower() in {'.exe','.dll','.so','.a','.o','.specs'} or path.name=='specs' or any(part in {'bin','libexec','include','include-fixed'} for part in relative.parts):
            result[relative.as_posix()]=digest(path)
    return result

# Objects, archives and dependency files are rebuilt in place from the sources and
# toolchain inventoried here. Hashing them made every Make run look like an SDK change.
SDK_BUILD_OUTPUTS=('.o','.a','.dep','.elf','.map')
def sdk_inventory(root):
    return {p.relative_to(root).as_posix():digest(p) for folder in ['psyqo','common','EASTL/include','EABase/include'] for p in files(root/folder) if p.suffix.lower() not in SDK_BUILD_OUTPUTS}

def adapter_source_inventory(root,adapters):
    result={}
    for adapter in adapters.values():
        if not adapter.get('command'):continue
        names=adapter.get('source_files',[])
        if not names:raise RuntimeError('Command adapter requires its reviewed source closure')
        for name in names:
            path=root/name
            linked=any(parent.is_symlink() or getattr(parent,'is_junction',lambda:False)()
                for parent in (path,*path.parents) if contained(parent,root))
            if not contained(path,root) or not path.is_file() or linked:
                raise RuntimeError('Missing/escaping adapter source: '+name)
            result[path.relative_to(root).as_posix()]=digest(path)
    return result
def identifier(row):return '/'.join(row[k] for k in ('template','flavor','mode'))
def label(row):return identifier(row).replace('/','--')
def stage(state,**values):return dict(status=state,**values)

@contextmanager
def console_lock(path):
    """Never steal a lock, including a possibly stale lock after a crash."""
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
    token=uuid.uuid4().hex
    try:fd=os.open(path,os.O_CREAT|os.O_EXCL|os.O_WRONLY)
    except FileExistsError:raise RuntimeError('Console queue already reserved: '+str(path))
    with os.fdopen(fd,'w',encoding='utf-8') as stream:json.dump(dict(pid=os.getpid(),token=token,started=utc()),stream)
    try:yield
    finally:
        if path.exists() and read(path).get('token')==token:path.unlink()

def source_snapshot(root,snapshot=None):
    if snapshot:
        values=read(snapshot)
        for name,expected in values.items():
            path=root/name
            if not path.is_file() or digest(path)!=expected:raise RuntimeError('Frozen dependency changed: '+name)
        return values
    values={}
    for folder in ('src','runtime','resources/templates','resources/templates/dungeon-complete-flavors'):
        for path in files(root/folder):values[path.relative_to(root).as_posix()]=digest(path)
    for name in ('Cargo.lock','Cargo.toml','build.rs'):
        if (root/name).is_file():values[name]=digest(root/name)
    return values

def semantic_project_inputs(values):
    # Root LuaLS configuration is editor support (src/lua_api_stub.rs), not
    # authored gameplay/compiler input. Do not ignore this name under assets,
    # scripts or ProjectSettings, nor any other project configuration.
    return {name:value for name,value in values.items() if name!='.luarc.json'}

def project_inputs(project,documents):
    values=semantic_project_inputs(inventory(project))
    descriptors=list(project.glob('*.epokproject'))
    if len(descriptors)!=1:raise RuntimeError('Exactly one project manifest is required')
    manifest=documents.loads(descriptors[0].read_text(encoding='utf-8'))
    # Mode is checked independently; final delivery requires native_cpp.
    normalized=dict(manifest);normalized.pop('lua_execution',None)
    values[descriptors[0].name]=fingerprint(normalized)
    return values,manifest,descriptors[0]

def normalize_scene(scene,class_roles):
    aliases={}
    for index,actor in enumerate(scene.get('actors',[])):
        aliases[actor['id']]='actor:'+str(index)
        for j,component in enumerate(actor.get('components',[])):aliases[component['id']]=f'actor:{index}/component:{j}'
    def visit(value):
        if isinstance(value,list):return [visit(v) for v in value]
        if isinstance(value,dict):
            result={}
            for key,item in value.items():
                if key=='class' and isinstance(item,dict):
                    identity=item.get('class_id');name=item.get('name')
                    result[key]={'role':class_roles.get(identity,class_roles.get(name,identity or name))}
                else:result[key]=visit(item)
            return result
        return aliases.get(value,value) if isinstance(value,str) else value
    return visit(scene)

def shared_resources(project,documents,class_roles):
    """Compare semantic resources, retaining identity/reference edges and payloads.

    C++/Lua class companions and Blueprint-embedded templates are storage variants
    of the same composition. Only known cache input digests and package source
    hashes are recomputed, after their actual resource inputs are retained.
    """
    project=Path(project);aliases={};packages={};scenes={};compositions={};raw={};provenance={}
    def identity(value):
        try:return isinstance(value,str) and str(uuid.UUID(value))==value.lower()
        except (ValueError,AttributeError):return False
    def register(value,label):
        if not identity(value):raise RuntimeError('Invalid resource identity: '+str(value))
        aliases.setdefault(value,label)
    def role(value):
        return class_roles.get(value,value)
    def collect(value,prefix):
        if isinstance(value,dict):
            if identity(value.get('id')):register(value['id'],prefix+'/id')
            for key,item in value.items():collect(item,prefix+'/'+key)
        elif isinstance(value,list):
            for index,item in enumerate(value):collect(item,prefix+'/'+str(index))
    def nonempty(template):
        return any(value not in [None,{},[]] for value in template.values())
    def composition(class_id,template,source):
        if not nonempty(template):return
        key=role(class_id)
        if key in compositions:raise RuntimeError('Duplicate composition for '+key)
        compositions[key]=template;provenance['composition:'+key]=source
    for path in files(project/'assets'):
        relative=path.relative_to(project).as_posix();suffix=path.suffix.lower()
        if suffix=='.epokclass':
            doc=documents.loads(path.read_text(encoding='utf-8'))
            if set(doc)-{'class','template','version'}:raise RuntimeError('Unclassified composition metadata: '+relative)
            composition(doc['class'],doc['template'],relative)
        elif suffix=='.epokbp':
            doc=documents.loads(path.read_text(encoding='utf-8'))
            composition(doc['id'],doc.get('template',{}),relative)
        elif suffix=='.epokmap':scenes[relative]=documents.loads(path.read_text(encoding='utf-8'))
        elif suffix not in SOURCE_EXTENSIONS:
            data=path.read_bytes()
            if suffix=='.epokasset':
                if len(data)<16 or data[:8]!=b'EPOKAS01':raise RuntimeError('Invalid asset package: '+relative)
                meta_len=int.from_bytes(data[8:12],'little');source_len=int.from_bytes(data[12:16],'little')
                if 16+meta_len+source_len!=len(data):raise RuntimeError('Invalid asset package length: '+relative)
                meta=json.loads(data[16:16+meta_len]);payload=data[16+meta_len:]
                if hashlib.sha256(payload).hexdigest()!=meta.get('source_hash'):raise RuntimeError('Asset source hash mismatch: '+relative)
                try:structured=json.loads(payload)
                except (ValueError,UnicodeDecodeError):structured=None
                packages[relative]=(meta,payload,structured)
            else:raw[relative]=hashlib.sha256(data).hexdigest()
    asset_ids={}
    for name,(meta,_,_) in packages.items():
        asset_id=meta['id']
        if asset_id in asset_ids:raise RuntimeError('Duplicate asset UUID: '+name+' and '+asset_ids[asset_id])
        asset_ids[asset_id]=name;register(asset_id,'asset:'+name)
    for name,scene in scenes.items():collect(scene,'scene:'+name)
    for name,(_,_,structured) in packages.items():collect(structured,'asset:'+name+'/payload')
    for name,template in compositions.items():collect(template,'composition:'+name)
    for class_id,name in class_roles.items():
        if identity(class_id):
            if class_id in aliases:raise RuntimeError('Class UUID collides with resource identity: '+class_id)
            aliases[class_id]='class:'+name
    def text(value):
        if value in aliases:return aliases[value]
        # Component member overrides are JSON pointer paths containing UUIDs.
        if value.startswith('/'):
            return '/'.join(aliases.get(segment,segment) for segment in value.split('/'))
        return value
    def visit(value):
        if isinstance(value,str):return text(value)
        if isinstance(value,list):return [visit(item) for item in value]
        if isinstance(value,dict):
            result={}
            for key,item in value.items():
                key=text(key)
                if key in result:raise RuntimeError('Normalized identity keys collide')
                if key=='class' and isinstance(item,dict):
                    class_id=item.get('class_id')
                    name=item.get('name')
                    result[key]={k:visit(v) for k,v in item.items() if k not in {'class_id','name'}}
                    result[key]['role']=role(class_id) if class_id else role(name)
                else:result[key]=visit(item)
            return result
        return value
    assets=dict(raw)
    for name,(metadata,payload,structured) in packages.items():
        metadata=visit(metadata)
        normalized_payload=visit(structured) if structured is not None else None
        payload_hash=fingerprint(normalized_payload) if structured is not None else hashlib.sha256(payload).hexdigest()
        metadata['source_hash']=payload_hash
        assets[name]={'metadata':metadata,'payload_sha256':payload_hash,'structured':structured is not None}
    normalized_scenes={}
    for name,scene in scenes.items():
        scene=visit(scene)
        for index,generator in enumerate(scene.get('dungeons',[])):
            output=generator.get('output')
            if not isinstance(output,dict):continue
            prefix=f'{name}/dungeons/{index}/output'
            if 'recipe' in output:
                provenance[prefix+'/recipe']=output['recipe']
                recipe=generator.get('recipe')
                if not isinstance(recipe,str) or not recipe.startswith('asset:'):raise RuntimeError('Unresolved dungeon recipe: '+prefix)
                recipe=recipe[len('asset:'):]
                if recipe not in assets:raise RuntimeError('Missing dungeon recipe payload: '+recipe)
                output['recipe']=assets[recipe]['payload_sha256']
            if 'fingerprint' in output:
                provenance[prefix+'/fingerprint']=output.pop('fingerprint')
                output['fingerprint']=fingerprint(generator)
        bake=scene.get('bake')
        if isinstance(bake,dict) and 'fingerprint' in bake:
            provenance[name+'/bake/fingerprint']=bake.pop('fingerprint')
            bake['fingerprint']=fingerprint(scene)
        normalized_scenes[name]=scene
    return {'assets':assets,'scenes':normalized_scenes,'compositions':visit(compositions)},provenance


def ownership_check(project,contract):
    asset_files={p.relative_to(project).as_posix() for p in files(project/'assets')}
    sources={name:digest(project/name) for name in asset_files if Path(name).suffix in SOURCE_EXTENSIONS}
    if not contract or contract.get('reviewed') is not True:
        return stage('missing',reason='Reviewed selected-rule/native-mechanism inventory contract required',sources=sources)
    required=contract.get('selected_rule_files',[])
    if not required:return stage('missing',reason='No selected-language rule files declared',sources=sources)
    for pattern in required:
        if not any(fnmatch.fnmatch(p,pattern) for p in sources):raise RuntimeError('Missing selected rule: '+pattern)
    for pattern in contract.get('required_asset_files',[]):
        if not any(fnmatch.fnmatch(p,pattern) for p in asset_files):raise RuntimeError('Missing required class composition: '+pattern)
    native=[p for p in sources if Path(p).suffix in {'.hpp','.cpp','.h','.c','.hh'}]
    allowed=contract.get('allowed_native_files',[])
    forbidden=contract.get('forbidden_files',[])
    for name in asset_files:
        if any(fnmatch.fnmatch(name,p) for p in forbidden):raise RuntimeError('Forbidden fallback/source: '+name)
    for name in native:
        if not any(fnmatch.fnmatch(name,p) for p in allowed):raise RuntimeError('Unclassified native source: '+name)
    return stage('passed',sources=sources,contract_sha256=fingerprint(contract),review=contract.get('review'))

class Runner:
    def __init__(self,args):
        self.args=args;self.root=args.root.resolve();self.out=args.output.resolve()
        for key in ['EPOK_RUNTIME_OPT','EPOK_SCRIPT_OPT']:
            if os.environ.get(key) not in [None,'','-O2']:raise RuntimeError('Final acceptance requires normal-O2 flags; experimental override present: '+key)
        sys.path.insert(0,str(self.root/'tools'))
        import epok_documents
        self.documents=epok_documents
        self.queue=read(args.queue);self.contracts=read(args.contracts) if args.contracts else {}
        self.adapters=read(args.adapters) if args.adapters else {}
        self.adapter_sources=adapter_source_inventory(self.root,self.adapters)
        self.editor=args.editor.resolve();self.snapshot=source_snapshot(self.root,args.source_snapshot)
        self.editor_sha256=digest(self.editor)
        # A runtime/compiler-only change invalidates build evidence, not authored
        # template projects or their reusable .epok caches. These are the source
        # builders and bundled authored content used by project creation.
        builders=set(self.root.glob('src/*template*.rs'))
        builders|={self.root/'src'/name for name in ['workspace.rs','game_helpers.rs','scene.rs']}
        self.generation_inputs={p.relative_to(self.root).as_posix():digest(p) for p in builders if p.is_file()}
        for p in files(self.root/'resources/templates'):self.generation_inputs[p.relative_to(self.root).as_posix()]=digest(p)
        for p in files(self.root/'resources/templates/dungeon-complete-flavors'):self.generation_inputs[p.relative_to(self.root).as_posix()]=digest(p)
        self.toolchain=toolchain_inventory(self.root/'.tools/mips')
        self.sdk=sdk_inventory(args.nugget)
        self.engine=fingerprint(dict(editor=digest(self.editor),sources=self.snapshot,runner=digest(__file__),toolchain=self.toolchain,sdk=self.sdk,
            contract=fingerprint(self.contracts),adapters=fingerprint(self.adapters),adapter_sources=self.adapter_sources,
            acceptance_contract=dict(matrix=[{k:r[k] for k in ('template','flavor','mode')} for r in self.queue['final_matrix']],
                scenarios=self.queue['matrix_contract']['scenarios'],gates=self.queue['fixed_gates']),
            build_environment={k:os.environ.get(k) for k in ('CPPFLAGS','EPOK_RUNTIME_OPT','EPOK_SCRIPT_OPT','CXXFLAGS')}))
        self.rows=[]
        seen=set()
        for row in self.queue['final_matrix']:
            case=identifier(row)
            if case in seen:raise RuntimeError('Duplicate matrix row: '+case)
            seen.add(case)
            if not args.case or any(fnmatch.fnmatch(case,pattern) for pattern in args.case):self.rows.append(row)
        if len(seen)!=18 or len({(r['template'],r['flavor']) for r in self.queue['final_matrix']})!=18 or any(r['mode']!='native_cpp' for r in self.queue['final_matrix']):
            raise RuntimeError('The final matrix must contain18 C++/Blueprint/LuaAOT cases, all native_cpp')
        templates={'basic','sample','third-person','platformer-2d','dungeon','dungeon-complete'}
        expected={(t,f,'native_cpp') for t in templates for f in ['cpp','blueprint','lua']}
        if {(r['template'],r['flavor'],r['mode']) for r in self.queue['final_matrix']}!=expected:raise RuntimeError('Final template/flavor coverage differs from the required6x3')
        if not self.rows:raise RuntimeError('No selected matrix cases')
        self.ledger_path=self.out/'matrix.json'
        self.ledger=read(self.ledger_path) if self.ledger_path.exists() else dict(schema=SCHEMA,rows={})
        for old_case in list(self.ledger['rows']):
            if old_case not in seen:
                previous=self.ledger['rows'].pop(old_case)
                self.ledger.setdefault('historical_rows',{}).setdefault(old_case,[]).append(previous)
        self.engine_record=self.out/'dependencies'/self.engine/'inputs.json'
        write(self.engine_record,dict(editor=str(self.editor),editor_sha256=digest(self.editor),sources=self.snapshot,adapter_sources=self.adapter_sources,runner_sha256=digest(__file__),toolchain=self.toolchain,sdk=self.sdk))

    def contract(self,row):return self.contracts.get(row['template'],{}).get(row['flavor'],{})
    def project(self,row):
        return self.out/'projects'/(row['template']+'--'+row['flavor'])
    def generation_key(self,row):
        return fingerprint(dict(inputs=self.generation_inputs,template=row['template'],flavor=row['flavor']))
    def directory(self,row):return self.out/'cases'/label(row)/self.engine[:16]
    def dependency(self,row):
        project=self.project(row)
        if not project.exists():return None
        values,_,_=project_inputs(project,self.documents)
        return fingerprint(dict(engine=self.engine,inputs=values,case=identifier(row)))
    def row(self,row):
        case=identifier(row);entry=self.ledger['rows'].setdefault(case,{})
        if entry.get('engine_sha256')!=self.engine:
            if entry:self.ledger.setdefault('historical_rows',{}).setdefault(case,[]).append(entry)
            entry=dict(case=case,template=row['template'],flavor=row['flavor'],mode=row['mode'],engine_sha256=self.engine,
                stages={s:stage('missing') for s in STAGES})
            self.ledger['rows'][case]=entry
        return entry
    def save(self):
        rows=self.ledger['rows']
        self.ledger.update(updated=utc(),engine_sha256=self.engine,required_generation_cases=18,required_execution_cases=18,
            passed_rows=sum(v.get('accepted') is True and v.get('engine_sha256')==self.engine for v in rows.values()),
            # This runner cannot close documentation, hardware-status or480i gates.
            initiative_complete=False,p10_additional_gates='See current-acceptance-queue.json; matrix-only success never closes P10')
        write(self.ledger_path,self.ledger)
    def ensure_frozen(self):
        if adapter_source_inventory(self.root,self.adapters)!=self.adapter_sources:
            raise RuntimeError('Frozen scenario adapter source changed during acceptance')
        if digest(self.editor)!=self.editor_sha256 or source_snapshot(self.root,self.args.source_snapshot)!=self.snapshot:
            raise RuntimeError('Frozen editor/source dependencies changed during acceptance')
        current=toolchain_inventory(self.root/'.tools/mips')
        sdk=sdk_inventory(self.args.nugget)
        if current!=self.toolchain or sdk!=self.sdk:raise RuntimeError('Pinned MIPS toolchain/SDK changed during acceptance')
    def command(self,row,name,argv,cwd=None,timeout=1200,env=None):
        self.ensure_frozen();before=self.dependency(row)
        directory=self.directory(row);directory.mkdir(parents=True,exist_ok=True)
        log=directory/(name+'-'+str(time.time_ns())+'.log');start=time.monotonic()
        with log.open('w',encoding='utf-8') as stream:
            process=subprocess.Popen([str(x) for x in argv],cwd=cwd or self.root,stdout=stream,stderr=subprocess.STDOUT,env=env,start_new_session=os.name!='nt')
            try:code=process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                if os.name=='nt':subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],capture_output=True,timeout=20)
                else:os.killpg(process.pid,signal.SIGKILL)
                process.wait(timeout=20);code='timeout'
        receipt=dict(argv=[str(x) for x in argv],cwd=str(cwd or self.root),exit_code=code,
            seconds=round(time.monotonic()-start,3),log=str(log),log_sha256=digest(log))
        if code!=0:raise CommandFailure(receipt)
        self.ensure_frozen()
        if name!='create' and before!=self.dependency(row):raise RuntimeError('Authored project inputs changed while '+name+' ran; result not accepted')
        return receipt,log.read_text(encoding='utf-8',errors='replace')
    def valid(self,row,name):
        entry=self.row(row)['stages'][name]
        if entry.get('status')!='passed' or entry.get('dependency_sha256')!=self.dependency(row):return False
        for path,expected in entry.get('files',{}).items():
            if not Path(path).is_file() or digest(path)!=expected:return False
        return True
    def record(self,row,name,result):
        result['dependency_sha256']=self.dependency(row);result['recorded']=utc()
        self.row(row)['stages'][name]=result;self.save()
    def mode(self,row):
        project=self.project(row);_,manifest,path=project_inputs(project,self.documents)
        if manifest.get('gameplay_flavor',manifest.get('gameplay',row['flavor']))!=row['flavor']:
            raise RuntimeError('Generated manifest selected flavor differs from row')
        if manifest.get('lua_execution')!=row['mode']:
            manifest['lua_execution']=row['mode'];self.documents.write_text(path,self.documents.dumps(manifest))
        return project,manifest
    def prepare(self,row):
        self.active_stage='generation'
        project=self.project(row);generation=project/'.p10-generation.json'
        if not project.exists():
            project.parent.mkdir(parents=True,exist_ok=True)
            receipt,_=self.command(row,'create',[self.editor,'--create-project',project,'--template',row['template'],'--gameplay',row['flavor'],'--name','P10 '+row['template']+' '+row['flavor']])
            values,_,_=project_inputs(project,self.documents)
            # The provenance file is external to authored inputs to avoid hashing itself.
            write(generation,dict(engine_sha256=self.engine,generation_sha256=self.generation_key(row),generation_inputs=self.generation_inputs,
                template=row['template'],flavor=row['flavor'],inputs=values,command=receipt))
        if not generation.is_file():raise RuntimeError('Unproven existing generated project: '+str(project))
        previous=read(generation);current,manifest,_=project_inputs(project,self.documents)
        current.pop('.p10-generation.json',None)
        if previous.get('generation_sha256')!=self.generation_key(row) or semantic_project_inputs(previous['inputs'])!=current:
            raise RuntimeError('Generated project changed; preserve its canonical cache and reconcile authored template inputs with a reviewed source receipt before resuming')
        self.record(row,'generation',stage('passed',project=str(project),primary_case=row['template']+'/'+row['flavor'],generation_proof=str(generation),
            generation_sha256=self.generation_key(row),reused_across_engine_change=previous['engine_sha256']!=self.engine))
        contract=self.contract(row)
        self.active_stage='source';result=ownership_check(project,contract);self.record(row,'source',result)
        if self.valid(row,'reflection'):return
        self.active_stage='reflection'
        # --reflect alone is intentionally the native Clang manifest. The full
        # catalog API compiles Lua before Blueprint and publishes every provider.
        # The legacy flag keeps old editors headless; its incomplete response is
        # rejected by the explicit marker instead of silently passing C++ only.
        receipt,text=self.command(row,'reflect',[self.editor,'--project',project,'--class-registry','--reflect'])
        registry=json.loads(text)
        if registry.get('schema_version')!=1 or registry.get('catalog_compiled') is not True:
            raise RuntimeError('Editor lacks --class-registry compiled catalog; native --reflect is not complete provider metadata')
        classes={c['id']:c for c in registry['classes']}
        scene=self.documents.loads((project/manifest['startup_scene']).read_text(encoding='utf-8'))
        wanted={a.get('class',{}).get('class_id') for a in scene.get('actors',[])}
        wanted|={c.get('class',{}).get('class_id') for a in scene.get('actors',[]) for c in a.get('components',[])}
        if None in wanted or not wanted<=classes.keys():raise RuntimeError('Unreflected scene class identity')
        output=self.directory(row)/'reflection.json';write(output,registry)
        self.record(row,'reflection',stage('passed',command=receipt,files={str(output):digest(output)},scene_class_ids=sorted(wanted)))

    def build(self,row):
        self.active_stage='build'
        project,manifest=self.mode(row)
        if self.valid(row,'build') and self.valid(row,'resources'):return
        receipt,_=self.command(row,'build',[self.editor,'--project',project,'--analyze-memory'])
        build=project/'.epok/build';output=self.directory(row)/'build';output.mkdir(parents=True,exist_ok=True)
        artifacts={}
        for path in build.rglob('*'):
            if path.is_file() and (path.name in {'epok.ps-exe','epok.elf','epok.map','memory-report.json','scene.hh','sources.mk'} or path.suffix.lower()=='.bin'):
                relative=path.relative_to(build);target=output/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,target);artifacts[str(target)]=digest(target)
        for name in ['epok.ps-exe','memory-report.json','sources.mk']:
            if not (output/name).is_file():raise RuntimeError('Build did not produce '+name)
        flags=(output/'sources.mk').read_text(encoding='utf-8');mode=re.search(r'-DEPOK_LUA_MODE=(\d+)',flags)
        if row['flavor']=='lua' and (not mode or int(mode[1])!=EXECUTION[row['mode']]):raise RuntimeError('Actual cooked execution mode differs from row')
        self.record(row,'build',stage('passed',command=receipt,files=artifacts,executable_sha256=digest(output/'epok.ps-exe'),actual_mode=row['mode']))
        self.active_stage='resources';memory=read(output/'memory-report.json');ram=memory['ram'];reserve=ram['capacity']-ram['used']
        if ram['capacity']!=2097152 or reserve<65536:raise RuntimeError('Unchanged2MiB RAM/64KiB runtime reserve gate failed')
        if memory.get('executable_hash') not in [None,digest(output/'epok.ps-exe')]:raise RuntimeError('Memory report is for another executable')
        scene=self.documents.loads((project/manifest['startup_scene']).read_text(encoding='utf-8'))
        actors=scene['actors'];maximum=max((len(a.get('components',[])) for a in actors),default=0)
        if len(actors)>64 or maximum>8:raise RuntimeError('Composition exceeds64actors/eightcomponents')
        shared,provenance=shared_resources(project,self.documents,self.contract(row).get('class_roles',{}))
        path=self.directory(row)/'resource-manifest.json';write(path,shared)
        provenance_path=self.directory(row)/'resource-provenance.json';write(provenance_path,provenance)
        self.record(row,'resources',stage('passed',files={str(path):digest(path),str(provenance_path):digest(provenance_path)},resource_schema=2,static_ram_bytes=ram['used'],runtime_reserve_bytes=reserve,
            actors=len(actors),max_components=maximum,normalized_shared_sha256=fingerprint(shared),runtime_peaks='Required in scenario receipt; static map alone does not establish peaks'))

    def play(self,row):
        project,_=self.mode(row)
        for name,flag in [('native_play','--play-native'),('psx_play','--play-psx')]:
            self.active_stage=name
            if self.valid(row,name):continue
            receipt,text=self.command(row,name,[self.editor,'--project',project,flag,'--stop-after',str(self.args.play_seconds)])
            expected='Native PC runtime running' if name=='native_play' else 'Emulator running'
            if expected not in text:raise RuntimeError('Play launch not observed')
            # Success of this stage is only launch/lifecycle, not feature parity.
            self.record(row,name,stage('passed',command=receipt,scope='Launch/normal Stop only; behavior requires scenario gate',
                runtime='native_pc' if name=='native_play' else 'play_station',mode='native_cpp' if name=='native_play' else row['mode']))

    def scenario(self,row):
        self.active_stage='scenario'
        if self.valid(row,'scenario'):return
        project,_=self.mode(row);adapter=self.adapters.get(identifier(row),self.adapters.get(row['template']))
        if not adapter:self.record(row,'scenario',stage('missing',reason='No current full-template scenario adapter/receipt'));return
        output=self.directory(row)/'scenario.json';dependency=self.dependency(row)
        export_manifest=self.directory(row)/'export-manifest.json'
        parameters=dict(root=str(self.root),editor=str(self.editor),project=str(project),case=identifier(row),mode=row['mode'],flavor=row['flavor'],output=str(output),dependency_sha256=dependency,
            export_manifest=str(export_manifest),export_root=read(export_manifest)['relocated'] if export_manifest.is_file() else '')
        if adapter.get('command'):
            argv=[argument.format_map(parameters) for argument in adapter['command']]
            command,_=self.command(row,'scenario',argv,timeout=adapter.get('timeout',1800))
        else:command=None;output=Path(adapter.get('receipt','')).resolve()
        proof=read(output)
        self.check_scenario(row,proof,dependency)
        result=stage('passed',command=command,files={str(output):digest(output)},scope='Current final generated template scenario',normalized_trace_sha256=proof['normalized_trace_sha256'])
        for path,expected in proof['evidence_files'].items():result['files'][str(Path(path).resolve())]=expected
        self.record(row,'scenario',result)

    def check_scenario(self,row,proof,dependency):
        if proof.get('schema')!=SCHEMA or proof.get('case')!=identifier(row) or proof.get('dependency_sha256')!=dependency:
            raise RuntimeError('Scenario case/dependency provenance mismatch')
        build=self.row(row)['stages']['build']
        # Complete's Play and export images read streamed geometry from disc, while the build
        # stage analyses memory with the Play profile over PCdrv. Either certified image counts.
        manifest=self.directory(row)/'export-manifest.json'
        exported=read(manifest).get('executable_sha256') if manifest.is_file() else None
        if proof.get('executable_sha256') not in {build.get('executable_sha256'),exported}:raise RuntimeError('Scenario executed a different image')
        if proof.get('provider')!=row['flavor'] or proof.get('mode')!=row['mode']:raise RuntimeError('Scenario actual provider/mode mismatch')
        required=self.queue['matrix_contract']['scenarios'][row['template']]
        for name in required:
            if proof.get('scenarios',{}).get(name,{}).get('passed') is not True:raise RuntimeError('Missing/failed required scenario: '+name)
        if not re.fullmatch('[0-9a-f]{64}',proof.get('normalized_trace_sha256','')):raise RuntimeError('No normalized semantic trace hash')
        metrics=proof.get('metrics',{})
        required_metrics=('slow_frames','max_frame_us','simulation_steps','cpu_work_units','dropped_work','rejected_work','missing_sounds','heap_peak_bytes','stack_peak_bytes','vram_peak_bytes','hud_drops','max_runtime_actors','max_components_per_actor')
        if any(type(metrics.get(name)) is not int or metrics[name]<0 for name in required_metrics):raise RuntimeError('Required runtime metrics missing or unmeasured')
        # Dungeon Complete's frame time is reported, not blocking. See the contract.
        timed=row['template']!='dungeon-complete'
        if any(metrics[n]!=0 for n in ('dropped_work','rejected_work','missing_sounds','hud_drops')) or (timed and (metrics['slow_frames']!=0 or metrics['max_frame_us']>25000)):raise RuntimeError('Unchanged timing/drop/sound gate failed')
        if metrics['max_runtime_actors']>64 or metrics['max_components_per_actor']>8:raise RuntimeError('Runtime composition exceeds bounds')
        if metrics['stack_peak_bytes']==0 or metrics['vram_peak_bytes']==0 or metrics['cpu_work_units']==0:raise RuntimeError('Measured stack/VRAM/CPU work cannot be replaced with zero placeholders')
        if metrics['vram_peak_bytes']>1024*512*2:raise RuntimeError('VRAM over capacity')
        reserve=self.row(row)['stages']['resources'].get('runtime_reserve_bytes')
        if type(reserve) is not int or metrics['heap_peak_bytes']+metrics['stack_peak_bytes']>reserve:raise RuntimeError('Measured heap/stack peaks exceed available runtime RAM')
        if metrics['simulation_steps']<=0:raise RuntimeError('Equal-step CPU measurement missing')
        if not all(proof.get('baseline_comparison',{}).get(key) is True for key in ['matched_route_settings','instrumentation_same','steps_equal']):raise RuntimeError('Baseline route/settings/instrumentation/equal-step CPU comparison missing')
        if row['template']=='dungeon-complete' and not all(proof.get('rule_edit_checks',{}).get(name) is True for name in ['guard','shop','run_flow']):raise RuntimeError('Chosen-language guard/shop/run-flow edits not observed')
        export_manifest=self.directory(row)/'export-manifest.json'
        if not export_manifest.is_file() or proof.get('export_manifest_sha256')!=digest(export_manifest):raise RuntimeError('Exported feature scenario lacks matching relocated export provenance')
        for name in ('streaming_passed','music_passed','peak_coverage_complete','capture_lifecycle_passed','ownership_edit_observed','export_scenario_passed'):
            if proof.get(name) is not True:raise RuntimeError('Required scenario/coverage assertion missing: '+name)
        if not proof.get('evidence_files'):raise RuntimeError('Scenario has no backing trace/measurement files')
        for path,expected in proof['evidence_files'].items():
            if not Path(path).is_file() or digest(path)!=expected:raise RuntimeError('Scenario evidence missing/changed: '+path)

    def export(self,row):
        self.active_stage='export'
        if not self.valid(row,'build'):raise RuntimeError('Export requires a current archived build receipt')
        if self.valid(row,'export'):return
        if not self.toolchain or not self.sdk:raise RuntimeError('Pinned MIPS toolchain/SDK provenance is missing')
        project,_=self.mode(row)
        receipt,text=self.command(row,'export',[self.editor,'--project',project,'--export-psx'])
        candidates=[Path(line.strip()) for line in text.splitlines() if line.strip()]
        exported=next((p for p in reversed(candidates) if p.is_dir() and (p/'build.ps1').exists()),None)
        if not exported:raise RuntimeError('Export directory not found in editor output')
        relocated=self.args.relocation_root.resolve()/label(row)
        if contained(relocated,self.root) or contained(relocated,project):raise RuntimeError('Standalone rebuild must be outside editor repository and source project')
        if relocated.exists():
            marker=relocated/'.p10-export.json'
            if not marker.is_file() or read(marker).get('dependency_sha256')!=self.dependency(row):raise RuntimeError('Existing relocated export needs reviewed source reconciliation at this same path; preserve its inputs and reusable build cache')
        else:
            shutil.copytree(exported,relocated)
            write(relocated/'.p10-export.json',dict(dependency_sha256=self.dependency(row),source_export=str(exported)))
        for path in files(relocated,exclude=set()):
            if path.suffix.lower() in {'.mk','.ps1','.cmake','.c','.cpp','.h','.hpp','.hh'} or path.name=='Makefile':
                data=path.read_text(encoding='utf-8',errors='replace')
                # Diagnostic #line paths do not load a source file during build.
                data='\n'.join(line for line in data.splitlines() if not re.match(r'\s*#\s*line\b',line))
                data=data.replace('\\','/').lower()
                for forbidden in [str(project).replace('\\','/').lower(),str(self.root/'resources/templates').replace('\\','/').lower()]:
                    if forbidden in data:raise RuntimeError('Standalone build input refers to original project/template: '+str(path))
                for include in re.findall(r'^\s*#\s*include\s*["<]([^">]+)',data,re.M):
                    target=path.parent/include
                    if (Path(include).is_absolute() or re.match(r'^[a-z]:/',include) or '..' in Path(include).parts) and not (contained(target,relocated) or contained(target,self.args.nugget)):
                        raise RuntimeError('Standalone include escapes exported sources/pinned SDK: '+include)
        env=dict(os.environ);env['PATH']=str(self.root/'.tools/mips/bin')+os.pathsep+env.get('PATH','')
        argv=['powershell.exe','-NoProfile','-ExecutionPolicy','Bypass','-File',str(relocated/'build.ps1'),'-Make',str(self.root/'.tools/mips/bin/make.exe'),'-Nugget',str(self.args.nugget.resolve())]
        rebuild,_=self.command(row,'relocated-rebuild',argv,cwd=relocated,env=env)
        # --export-psx selects its own transport. A streamed project's image differs from
        # the --analyze-memory build by design, so the rebuild must match the export image.
        original=self.directory(row)/'build';reference=self.directory(row)/'export-image.ps-exe'
        shutil.copyfile(project/'.epok/build/epok.ps-exe',reference)
        if digest(relocated/'epok.ps-exe')!=digest(reference):raise RuntimeError('Relocated executable differs from selected-mode image')
        assets={}
        for path in original.rglob('*'):
            # The disc image is an editor package; a standalone rebuild loads the streamed data.
            if path.is_file() and path.suffix.lower()=='.bin' and path.name.lower()!='epok.bin':
                rel=path.relative_to(original);other=relocated/rel
                if not other.is_file() or digest(path)!=digest(other):raise RuntimeError('Standalone external asset mismatch: '+str(rel))
                assets[rel.as_posix()]=digest(other)
        output=self.directory(row)/'export-manifest.json'
        write(output,dict(executable_sha256=digest(reference),external_assets=assets,relocated=str(relocated),inputs=inventory(relocated,exclude={'build','__pycache__'})))
        self.record(row,'export',stage('passed',command=receipt,rebuild=rebuild,files={str(output):digest(output),str(relocated/'epok.ps-exe'):digest(reference)},
            mode=row['mode'],byte_identical=True,external_assets=assets,scope='Outside original project/repository; explicit pinned SDK/toolchain; scenario receipt must additionally attest matching exported feature replay'))

    def verify(self):
        self.ensure_frozen()
        for row in self.queue['final_matrix']:
            entry=self.row(row)
            for name in STAGES:
                if entry['stages'][name].get('status')=='passed' and not self.valid(row,name):
                    entry['stages'][name]['status']='stale'
            scenario=entry['stages']['scenario']
            if self.valid(row,'scenario'):
                path=next(iter(scenario['files']))
                try:self.check_scenario(row,read(path),self.dependency(row))
                except Exception as error:scenario.update(status='failed',reason=str(error))
            entry['accepted']=all(self.valid(row,name) for name in STAGES)
        # Compare outcomes/shared content across exactly3 AOT providers per template.
        for template in {r['template'] for r in self.queue['final_matrix']}:
            entries=[self.row(r) for r in self.queue['final_matrix'] if r['template']==template]
            ready=all(e.get('accepted') for e in entries)
            if ready:
                outcomes={e['stages']['scenario']['normalized_trace_sha256'] for e in entries}
                shared={e['stages']['resources']['normalized_shared_sha256'] for e in entries}
                if len(outcomes)!=1 or len(shared)!=1:
                    for entry in entries:entry.update(accepted=False,parity='failed: normalized outcomes or shared resources differ')
            for entry in entries:entry['three_provider_parity']='passed' if ready and len(outcomes)==len(shared)==1 else 'missing_or_failed'
        self.save()
        all_rows=list(self.ledger['rows'].values())
        self.ledger['matrix_accepted']=len(all_rows)==18 and all(e.get('accepted') and e.get('three_provider_parity')=='passed' for e in all_rows)
        self.save()
        return self.ledger['matrix_accepted']

class CommandFailure(RuntimeError):
    def __init__(self,receipt):self.receipt=receipt;super().__init__('Command failed: '+str(receipt))

def parser():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1])
    p.add_argument('--editor',type=Path)
    p.add_argument('--queue',type=Path)
    p.add_argument('--output',type=Path)
    p.add_argument('--contracts',type=Path,help='Reviewed ownership contracts per template/flavor')
    p.add_argument('--adapters',type=Path,help='Actual scenario command/receipt per case or template')
    p.add_argument('--source-snapshot',type=Path,help='Frozen repository-relative source->SHA map; validated on every invocation')
    p.add_argument('--nugget',type=Path)
    p.add_argument('--relocation-root',type=Path,default=Path(tempfile.gettempdir())/'epok-gameplay-final18')
    p.add_argument('--console-lock',type=Path)
    p.add_argument('--case',action='append',help='Case glob, e.g. dungeon-complete/lua/native_cpp; may repeat')
    for option in ['prepare','build','play','scenario','export','verify-evidence','all']:
        p.add_argument('--'+option,action='store_true')
    p.add_argument('--play-seconds',type=int,default=8)
    p.add_argument('--continue-on-failure',action='store_true')
    return p

def main(argv=None):
    args=parser().parse_args(argv);args.root=args.root.resolve()
    args.editor=args.editor or args.root/'target/debug/epok-editor.exe'
    args.queue=args.queue or args.root/'artifacts/gameplay-helpers/p10/current-acceptance-queue.json'
    args.output=args.output or args.root/'artifacts/gameplay-helpers/p10/final18'
    args.nugget=args.nugget or args.root/'third_party/nugget'
    args.console_lock=args.console_lock or args.root/'artifacts/gameplay-helpers/console-queue.lock'
    if not any(getattr(args,n) for n in ['prepare','build','play','scenario','export','verify_evidence','all']):raise SystemExit('Choose a phase; no default execution')
    runner=Runner(args)
    execute=any(getattr(args,n) for n in ['prepare','build','play','scenario','export','all'])
    # Ledger writes are serialized too, so a read/verify cannot race a build.
    with console_lock(args.console_lock):
        for row in runner.rows:
            for option,name in [('prepare','prepare'),('build','build'),('play','play'),('export','export'),('scenario','scenario')]:
                if not (getattr(args,option) or args.all):continue
                try:getattr(runner,name)(row)
                except Exception as error:
                    target=getattr(runner,'active_stage',{'prepare':'generation','play':'psx_play'}.get(name,name))
                    runner.record(row,target,stage('failed',reason=str(error),command=error.receipt if isinstance(error,CommandFailure) else None))
                    print(identifier(row),name,'FAILED',str(error),flush=True)
                    if not args.continue_on_failure:raise
            runner.save()
        accepted=runner.verify()
    requested=[]
    for option,names in [('prepare',['generation','source','reflection']),('build',['build','resources']),('play',['native_play','psx_play']),('export',['export']),('scenario',['scenario'])]:
        if getattr(args,option) or args.all:requested+=names
    phase_completed=bool(requested) and all(runner.valid(row,name) for row in runner.rows for name in requested)
    print(json.dumps(dict(requested_phase_completed=phase_completed,matrix_accepted=accepted,initiative_complete=False,ledger=str(runner.ledger_path))))
    if args.verify_evidence or args.all:return 0 if accepted else 2
    return 0 if phase_completed else 2

if __name__=='__main__':raise SystemExit(main())
