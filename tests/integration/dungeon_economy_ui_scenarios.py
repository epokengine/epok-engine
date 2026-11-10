"""Append UI to the existing economy circuit, three native providers only.

The 840x86 core prefix stays independently checked. UI conformance uses the
native C++ console capture plus the separate SDK/legacy host proof, explicitly.
No selected-language decisions are implemented by this fixture driver.
"""
from pathlib import Path
from contextlib import contextmanager
import copy
import hashlib
import json
import shutil
import os
import re
import subprocess

UI_LABELS='''reset hud inventory-open inventory-ready up-wrap release-a down-wrap release-b medkit-save busy-close failed-save retry-save disable-during-save reenable-durable heal-once inventory-reopen inventory-ready-again inventory-close release-c shop-focus shop-open shop-ready edit-price-heal shop-select release-d shop-save shop-durable shop-close release-e pocket-focus pocket-open pocket-ready slot-toggle release-f pocket-missing-image pocket-close release-g strict-xz-boundary strict-y-boundary inside-cylinder inventory-third inventory-capacity destroy-modal respawn-modal close-respawn cleanup deactivate-station reactivate-station result-open result-ready victory-publish victory-restore defeat-publish edited-hud damage-feedback final-cleanup'''.split()
assert len(UI_LABELS)==56
UI_LEAVES=['DungeonGameHUDWidget','DungeonTerminalWidget','DungeonShopWidget','DungeonInventoryWidget','DungeonEconomyStation']
UI_BASES=['DungeonEconomyPresenterEndpoint.hpp','DungeonEconomyWidgetBase.hpp','DungeonEconomyPresenterBase.hpp','DungeonEconomyStationBase.hpp']
ROOT=next(p for p in Path(__file__).resolve().parents if(p/'Cargo.toml').exists()and(p/'runtime').is_dir())
ARTIFACT=ROOT/'artifacts/gameplay-helpers/p8/economy/next-widgets'
NATIVE=ROOT/'resources/templates/dungeon-complete/assets/scripts/DungeonCombat'
FLAVORS=ROOT/'resources/templates/dungeon-complete-flavors'
ASSETS=ROOT/'resources/templates/dungeon-complete/assets'
BASE=None

def assert_transport_provenance(base,out,project):
    """Hard per-case isolation gate; inspect real target Make flags without compiling."""
    build=project/'.epok/build';map_path=build/'epok.map';sources_path=build/'sources.mk';log_path=out/'native_cpp-play.log'
    map_text=map_path.read_text();log=log_path.read_text(errors='replace');sources=sources_path.read_text();makefile=(build/'Makefile').read_text()
    def symbol(name):return re.findall(r'(?m)^\s*0x[0-9a-fA-F]+\s+'+re.escape(name)+r'\s*$',map_text)
    assert not symbol('epok_pocket_mailbox'),'Production mailbox exposed to external Play bridge'
    assert len(symbol('epok_economy_fixture_mailbox'))==1,'Fixture mailbox missing or duplicated'
    for pattern in(r'The game is talking to the PocketStation',r'PocketStation:\s*[1-9][0-9]* requests'):
        assert re.search(pattern,log)is None,'External Pocket bridge attached to fixture'
    macro='-Depok_pocket_mailbox=epok_economy_fixture_mailbox'
    flags={key:os.environ.get(key,'')for key in('EPOK_RUNTIME_OPT','EPOK_SCRIPT_OPT')}
    assert all(macro in value.split()for value in flags.values()),flags
    assert 'main.o main.dep: CPPFLAGS += $(EPOK_RUNTIME_OPT)'in makefile
    assert '$(SCRIPT_SRCS:.cpp=.o) $(SCRIPT_SRCS:.cpp=.dep): CPPFLAGS += $(EPOK_SCRIPT_OPT)'in makefile
    script_line=next(line for line in sources.splitlines()if line.startswith('SCRIPT_SRCS :='))
    objects=['main.o',*[name[:-4]+'.o'for name in script_line.split(':=',1)[1].split()if name.endswith('.cpp')]]
    # Override only these object recipes with info. No object/compiler recipe is
    # run; target-specific CPPFLAGS are expanded by the real existing Makefile.
    evidence=out/'compiler-command-inspection.mk'
    evidence.write_text('.PHONY: epok-fixture-inspect-force\nepok-fixture-inspect-force:\n'+''.join(name+': epok-fixture-inspect-force\n'+name+':\n\t$(info EPOK_FIXTURE_COMMAND|$@|$(COMPILE.cpp))\n'for name in objects),encoding='utf-8')
    environment=os.environ.copy();environment['PATH']=str(base.ROOT/'.tools/mips/bin')+os.pathsep+environment.get('PATH','')
    command=[str(base.ROOT/'.tools/mips/bin/make.exe'),'--no-print-directory','BUILD=Release','NUGGET_DIR='+str(base.ROOT/'third_party/nugget').replace('\\','/'),'-f','Makefile','-f',str(evidence.resolve()),*objects]
    result=subprocess.run(command,cwd=build,env=environment,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=60)
    (out/'compiler-command-inspection.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    assert result.returncode==0,result.stdout+result.stderr
    commands={}
    for line in result.stdout.splitlines():
        if line.startswith('EPOK_FIXTURE_COMMAND|'):
            _,target,value=line.split('|',2);commands[target]=value
    assert set(commands)==set(objects),(commands,objects)
    assert all(macro in value.split()for value in commands.values()),commands
    paths=[map_path,sources_path,build/'Makefile',log_path,evidence,out/'compiler-command-inspection.log']
    (out/'transport-isolation.json').write_text(json.dumps({'passed':True,'scope':'Exact map symbols, no external bridge log, actual Make target-specific compiler command expansion for every appTU; inspection executes no compiler/linker','flags':flags,'commands':commands,'files':{str(path.relative_to(project)if path.is_relative_to(project)else path.name).replace('\\','/'):hashlib.sha256(path.read_bytes()).hexdigest()for path in paths}},indent=2)+'\n',encoding='utf-8')

def stage_ui_assets(base,project,flavor):
    """Stage before the existing reflection call so typed UI records are known."""
    global BASE
    BASE=base
    from uuid import UUID,uuid5
    uiid=lambda name:str(uuid5(UUID('18263f9d-d972-530a-91ac-739c4ed39dc4'),'DungeonEconomyUI/'+name))
    scripts=project/'assets/scripts'
    for name in UI_BASES:shutil.copyfile(NATIVE/name,scripts/name)
    for name in UI_LEAVES:
        if flavor=='cpp':shutil.copyfile(NATIVE/(name+'.hpp'),scripts/(name+'.hpp'))
        else:
            assert not(scripts/(name+'.hpp')).exists(),'C++ UI policy leaked into selected flavor'
            extension='.lua'if flavor=='lua'else'.epokbp'
            target=(scripts if flavor=='lua'else project/'assets/Blueprints')/(name+extension)
            base.write(target,(FLAVORS/flavor/(name+extension)).read_text())
    if flavor=='lua':
        path=project/'ProjectSettings/LuaClasses.epoksettings';settings=base.documents.loads(path.read_text())
        settings['classes'].update({'assets/scripts/'+name+'.lua':uiid(name)for name in UI_LEAVES});base.write(path,settings)
    for source in(ASSETS/'Classes').glob('*.epokclass'):
        if source.stem not in{uiid(name)for name in UI_LEAVES}:continue
        target=project/'assets/Classes'/source.name
        if flavor=='blueprint':
            if target.exists():target.unlink() # fixture-owned generated companion only
        else:base.write(target,source.read_text())
    shutil.copytree(ASSETS/'Localization',project/'assets/Localization',dirs_exist_ok=True)
    shutil.copytree(base.ROOT/'resources/templates/dungeon-complete/assets/UI',project/'assets/UI',dirs_exist_ok=True)

def prepare_ui_project(base,project,flavor,scene,driver,first,second):
    """Reuse existing driver/device actors and the normal linked HUD instance."""
    from uuid import UUID,uuid5
    uiid=lambda name:str(uuid5(UUID('18263f9d-d972-530a-91ac-739c4ed39dc4'),'DungeonEconomyUI/'+name))
    original=base.documents.loads((ASSETS/'scenes/DungeonGame.epokmap').read_text())
    root=copy.deepcopy(next(a for a in original['actors']if a['name']=='Combat HUD'))
    controls=[copy.deepcopy(a)for a in original['actors']if a.get('logical_parent')==root['id']]
    policy=next(c for c in driver['components']if c['class']['name']=='DungeonEconomyPolicy')
    root['properties']['policy']=policy['id'];root['blueprint_instance']['overrides']['/properties/policy']=policy['id']
    policy['properties'].update(hud_owner=root['id'],pocket_station=first['id'],trading_station=second['id']);policy['overrides']=list(policy['properties'])
    for actor in(first,second):
        channel=next(c for c in actor['components']if c['class']['name']=='epok::PocketTransactionComponent')
        channel['properties']['automatic_poll']=False;channel['overrides']=list(channel['properties'])
    def component(name,key,class_id):return {'id':uiid('conformance/'+key),'name':key,'class':{'name':name,'class_id':class_id},'properties':{},'overrides':[]}
    interactor=component('epok::InteractorComponent','interactor','ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a')
    interactor['properties']={'range_3d':3.61,'auto_input':True,'proximity':True};interactor['overrides']=list(interactor['properties']);driver['components'].append(interactor)
    stations=[]
    for actor,menu in [(first,1),(second,2)]:
        station=component('DungeonEconomyStation','station-'+str(menu),uiid('DungeonEconomyStation'));station['properties']={'hud':root['id'],'menu':menu,'horizontal_range':2.5,'height_range':2.6,'range_3d':3.61};station['overrides']=list(station['properties']);actor['components'].append(station);stations.append(station)
    driver['properties'].update(ui_hud=root['id'],ui_interactor=interactor['id'],ui_pocket=stations[0]['id'],ui_shop=stations[1]['id']);driver['overrides']=list(driver['properties'])
    scene['actors']=[driver,first,second,root,*controls]
    while len(scene['actors'])<58:
        n=len(scene['actors']);identity=uiid('conformance/resident/'+str(n))
        scene['actors'].append({'id':identity,'name':'Fixture resident '+str(n),'class':{'name':'epok::Actor3D','class_id':'33586caf-af3c-5aa2-8989-695bab59f4b8'},'properties':{},'overrides':[],'components':[{'id':uiid('conformance/resident-root/'+str(n)),'name':'Root','class':{'name':'epok::SceneComponent3D','class_id':'ROOT_SCENE_COMPONENT'},'root':True,'properties':{},'overrides':[]}]})
    # Take actual native class UUIDs from the already reflected driver/root;
    # do not invent another class mapping or actor family in the fixture.
    native_actor=first['class'];native_root=first['components'][0]['class']
    for actor in scene['actors']:
        if actor['name'].startswith('Fixture resident '):actor['class']=copy.deepcopy(native_actor);actor['components'][0]['class']=copy.deepcopy(native_root)
    assert len(scene['actors'])==58 and max(len(a['components'])for a in scene['actors'])<=8

def install_ui_extension(base):
    """Installed only by the existing runner's CLI; imported host users stay core."""
    global BASE
    BASE=base
    core_commands=base.commands;core_source=base.source;core_steps=base.STEPS;core_host=base.host_oracle
    core_assert=base.assert_scenario_contracts;core_verify=base.verify;core_common=base.common_files
    core_run_modes=base.run_modes
    def isolated_run_modes(*args,**kwargs):
        kwargs['forbidden_runtime_logs']=[r'The game is talking to the PocketStation',r'PocketStation:\s*[1-9][0-9]* requests']
        return core_run_modes(*args,**kwargs)
    def commands():return [*core_commands(),*[{'label':'ui-'+name,'op':'ui','words':[25,n,0,0,0,0,0]}for n,name in enumerate(UI_LABELS)]]
    @contextmanager
    def core_context():
        previous=base.commands,base.source,base.STEPS,base.assert_scenario_contracts
        base.commands,base.source,base.STEPS,base.assert_scenario_contracts=core_commands,core_source,core_steps,core_assert
        try:yield
        finally:base.commands,base.source,base.STEPS,base.assert_scenario_contracts=previous
    def source():
        result=core_source()
        result=result.replace('#include "device_io.hpp"','#include "device_io.hpp"\n#include "DungeonEconomyPresenterBase.hpp"\n#include "DungeonEconomyStationBase.hpp"')
        marker='    void on_frame(uint32_t)override{'
        properties=''.join(f'    EPOK_PROPERTY(EditAnywhere,{kind}="{klass}") epok::ObjectId {name}{{}};\n'for name,kind,klass in [('ui_hud','ActorClass','DungeonEconomyPresenterBase'),('ui_interactor','ComponentClass','epok::InteractorComponent'),('ui_pocket','ComponentClass','DungeonEconomyStationBase'),('ui_shop','ComponentClass','DungeonEconomyStationBase')])
        result=result.replace(marker,properties+(Path(__file__).parent/'DungeonEconomyUIProbe.inl').read_text()+'\n'+marker)
        result=result.replace('if(!frame_){if(!memory_card.attach(card)','if(!frame_){if(a->automatic_poll||b->automatic_poll)fail_|=128;if(!memory_card.attach(card)')
        result=result.replace('        case 24:d=p->pocket_execute(q[1],0);economy_protocol_probe::fold(*p,d);break;','        case 24:d=p->pocket_execute(q[1],0);economy_protocol_probe::fold(*p,d);break;\n        case 25:ui_run(q[1],*p);d.notice=ui_output_.cue;d.amount=ui_output_.menu;d.heal=ui_output_.heal;break;')
        result=result.replace('if(q[0]>=22){','if(q[0]>=22&&q[0]<=24){')
        result=result.replace('        if(at!=86)fail_|=8;','        if(q[0]==25){ui_row(row);if(ui_fail_)fail_|=64;}\n        if(at!=86)fail_|=8;')
        return result
    def host(base_out,allow_build=True):
        with core_context():return core_host(base_out,allow_build)
    def assertions(trace):
        assert len(trace)==base.STEPS*base.WORDS
        with core_context():core_assert(trace[:core_steps*base.WORDS])
        rows=[trace[n*base.WORDS:(n+1)*base.WORDS]for n in range(core_steps,base.STEPS)]
        assert len(rows)==len(UI_LABELS)
        for stage,row in enumerate(rows):assert row[72]==stage and row[85]==0,('UI',UI_LABELS[stage],row[72:86])
        assert rows[13][77]==35 and rows[14][77]==0
        assert rows[26][13:15]==[883,2] and rows[41][84]==64 and rows[45][84]==58
        assert rows[51][82]==1 and rows[52][82]==1 and rows[53][81]==1024
    def validate(base_out,out,flavor,selected,fingerprint):
        assert selected==['native_cpp'],'UI acceptance covers the three native providers, not Complete VM'
        assert_transport_provenance(base,out,out/'project')
        reference=host(base_out,allow_build=False);path=out/'native_cpp-dungeon_economy_policy_trace.json';actual=json.loads(path.read_text());assertions(actual)
        assert actual[:len(reference)]==reference,'Current core840-word behavior changed'
        baseline=base_out/'cpp/native_cpp-dungeon_economy_policy_trace.json'
        if flavor!='cpp':
            expected=json.loads(baseline.read_text());assert len(expected)==len(actual)
            assert actual==expected,next((n,a,b)for n,(a,b)in enumerate(zip(actual,expected))if a!=b)
        observation=json.loads((out/'native_cpp-observation.json').read_text());values=observation['values'];assert values[:4]==[base.MAGIC,base.STEPS,0,base.STEPS]
        digest=2166136261
        for value in actual:digest=((digest^value)*16777619)&0xffffffff
        assert digest==values[4]and values[6:]==[base.POCKET_MASK,23]
        base.write(out/'conformance.json',{'provider':flavor,'modes':['native_cpp'],'core_prefix_rows':core_steps,'ui_rows':len(UI_LABELS),'words_per_case':len(actual),'native_policy_fallback':False,'common_contract_sha256':fingerprint,'ui_reference':'Current C++ PSX capture + separate independent legacy/SDK host proof; not an independent UI trace oracle','ui_baseline_sha256':hashlib.sha256(baseline.read_bytes()).hexdigest(),'host_core_reference_sha256':hashlib.sha256((base_out/'host-oracle/trace.json').read_bytes()).hexdigest()})
        print('PASS unified core+UI economy',flavor,base.STEPS,'rows',flush=True)
    def verify(base_out,flavor,modes=None,prepare_only=False,check_only=False):
        assert not modes or modes==['native_cpp'],'Complete VM is outside this acceptance matrix'
        # Rename only the fixture's actual SDK mailbox symbol in every
        # application TU. The unmodified Play bridge discovers the public
        # production symbol by exact name, so it cannot answer this mock bus.
        macro='-Depok_pocket_mailbox=epok_economy_fixture_mailbox'
        keys=('EPOK_RUNTIME_OPT','EPOK_SCRIPT_OPT')
        previous={key:os.environ.get(key)for key in keys}
        for key in keys:
            value=previous[key]or'-O2'
            os.environ[key]=value if macro in value.split()else value+' '+macro
        try:return core_verify(base_out,flavor,['native_cpp'],prepare_only,check_only)
        finally:
            for key,value in previous.items():
                if value is None:os.environ.pop(key,None)
                else:os.environ[key]=value
    def common_files(project):return sorted(set([*core_common(project),*[project/'assets/scripts'/name for name in UI_BASES]]))
    base.commands=commands;base.source=source;base.STEPS=core_steps+len(UI_LABELS);base.host_oracle=host;base.assert_scenario_contracts=assertions;base.validate_captures=validate;base.verify=verify;base.common_files=common_files;base.run_modes=isolated_run_modes
    base.write(ARTIFACT/'fixture/scenario-inventory.json',{'core_rows':core_steps,'ui_rows':len(UI_LABELS),'columns':86,'providers':['cpp/native_cpp','blueprint/native_cpp','lua/native_cpp'],'ui_labels':UI_LABELS,'scope':'Prepared, no console runs; current C++ baseline + independent host proofs, no native selected-rule fallback'})
