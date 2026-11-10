"""Offline observer-contract tests; synthetic rows are never acceptance evidence."""
import copy
import ctypes
import importlib.util
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile
import unittest
import yaml

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p / 'tools/epok_documents.py').is_file())
sys.path.insert(0,str(ROOT/'tests/integration/gameplay_helpers'))

import complete_observation as subject

def row(frame, **kw):
    result=dict(frame=frame,step=frame,state=0,health=100,key=0,kills=0,boss_health=280,
        damage=0,heals=0,restarts=1,phase=0,locked=True,fade=255,music_playing=True,music_volume=1000,
        shots=0,hits=0,muzzle_errors=0,target_index=65535,target_valid=False,camera_mode=0,
        marker_active=False,previous_target_valid=False,pad_mask=0,image_enabled=False,pack_inactive=False,lock_audio=0,damage_audio=0,
        heal_audio=0,hit_audio=0,boss_destroyed_audio=0,audio_missing=0,borders=[0]*4,image_color=[255,85,65])
    result.update(kw);return result

def complete_rows():
    rows=[row(i+1,phase=i,locked=i!=4,fade=0 if i>=3 else 255) for i in range(5)]
    rows += [row(6,phase=4,locked=False,camera_mode=1,target_index=3,target_valid=True,marker_active=True,lock_audio=1),
             row(7,phase=4,locked=False,camera_mode=1,target_index=4,target_valid=True,previous_target_valid=True,pad_mask=2048,marker_active=True,lock_audio=2),
             row(8,phase=4,locked=False,camera_mode=2,pad_mask=1024),
             row(9,phase=4,locked=False,health=85,damage=1,damage_audio=1,image_enabled=True,borders=[4]*4),
             row(10,phase=4,locked=False,health=100,damage=1,damage_audio=1,heals=1,heal_audio=1,pack_inactive=True),
             row(11,phase=5,state=3,health=0,damage=8,damage_audio=8),
             row(12,phase=6,state=3,health=0,fade=128),row(13,phase=7,state=3,health=0,music_volume=0,music_playing=False),
             row(14,phase=4,locked=False,restarts=2),
             row(15,phase=4,locked=False,restarts=2,state=1,key=1,boss_health=260,shots=1,hits=1,hit_audio=1),
             row(16,phase=4,restarts=2,state=2,key=1,boss_health=0,shots=14,hits=14,hit_audio=14,boss_destroyed_audio=1),
             row(17,phase=4,locked=False,restarts=3)]
    return rows

def fake_abi():
    structures={};syms={}
    sizes={'fps':28,'flow':32,'game':44,'combat':60,'audio':72,'economy':56}
    for i,(key,(symbol,typename,fields)) in enumerate(subject.TELEMETRY.items()):
        members={field:j*4 for j,field in enumerate(fields)}
        if key=='audio':members={'events':0,'missing':68}
        if key=='combat':members['states']=48
        structures[typename]=[{'size':sizes[key],'members':members}]
        syms[symbol]={'address':0x80020000+i*128,'size':sizes[key]}
    for name,size,members in [('ActorData',1400,{'transform':8,'image':120,'active':1300,'alive':1299,'generation':1350}),
                               ('Transform',36,{'position':0}),('Image',28,{'enabled':0,'color':1,'borders':16}),
                               ('DungeonPlayerController',320,{'camera_mode':84,'target_handle':120}),
                               ('DataHandle',8,{'index':0,'generation':4})]:
        structures[name]=[{'size':size,'members':members}]
    syms['epok::objects']={'address':0x80040000,'size':1400*90}
    for i,name in enumerate(['DungeonGame::on_frame(unsigned long)','DungeonPlayerController::update(epok::Transform&, epok::Fixed)','epok::ControllerPad::poll(unsigned int)']):
        syms[name]={'address':0x80011000+i*128,'size':100}
    return syms,structures

class Contracts(unittest.TestCase):
    def lua_parse(self,source):
        """Compile only with Redux's actual LuaJIT DLL; no PCSX/game execution."""
        dll=ctypes.CDLL(str(ROOT/'.tools/redux/lua51.dll'))
        dll.luaL_newstate.restype=ctypes.c_void_p
        dll.luaL_loadbuffer.argtypes=[ctypes.c_void_p,ctypes.c_char_p,ctypes.c_size_t,ctypes.c_char_p]
        dll.luaL_loadbuffer.restype=ctypes.c_int
        dll.lua_tolstring.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.POINTER(ctypes.c_size_t)]
        dll.lua_tolstring.restype=ctypes.c_char_p
        dll.lua_close.argtypes=[ctypes.c_void_p]
        state=dll.luaL_newstate()
        self.assertTrue(state,'Real host LuaJIT state allocation failed')
        try:
            code=dll.luaL_loadbuffer(state,source,len(source),b'@complete-observer-host-proof.lua')
            error=dll.lua_tolstring(state,-1,None).decode('utf-8',errors='replace') if code else None
            return code,error
        finally:dll.lua_close(state)

    def retained_failed_pad(self):
        path=ROOT/'artifacts/gameplay-helpers/p10/final18/projects/dungeon-complete--cpp/.epok/validation/stream-pcdrv-guards-20261107-003433/pad.lua'
        self.assertTrue(path.is_file(),'Preserve the actual failed composed Lua')
        self.assertEqual(subject.sha(path),'e4da0f82a69b170ea4318e4985a6f18785d54d94d9398a97a0bf43f367d564f6')
        return path

    def test_retained_composed_pad_is_rejected_by_actual_luajit(self):
        code,error=self.lua_parse(self.retained_failed_pad().read_bytes())
        self.assertEqual(code,3)
        self.assertIn("unexpected symbol near '['",error)

    def test_current_tuple_configuration_and_full_composition_parse_in_actual_luajit(self):
        failed=self.retained_failed_pad()
        program=json.loads(failed.with_name('current-program-contract.json').read_text(encoding='utf-8'))
        abi=program['abi']
        # Restore real compiled_contract Python types, rather than testing JSON
        # lists alone and accidentally hiding the tuple serialization regression.
        abi['names']={key:tuple(value) for key,value in abi['names'].items()}
        linked=subject.layout.symbols(ROOT,ROOT/'artifacts/gameplay-helpers/p10/complete-size-failure/current-Os-cpp/exported/epok.elf')
        symbols={key:linked[value[0]]['address']&0x1fffff for key,value in abi['names'].items()}
        observer=subject.observer_lua(abi,program['scene'],symbols)
        code,error=self.lua_parse(observer.encode('utf-8'))
        self.assertEqual(code,0,error)
        prior=failed.read_text(encoding='utf-8')
        self.assertEqual(prior.count('\nlocal COMPLETE_CONFIG='),1)
        composed=prior.split('\nlocal COMPLETE_CONFIG=',1)[0]+observer
        code,error=self.lua_parse(composed.encode('utf-8'))
        self.assertEqual(code,0,error)
        # Parsing proves syntax; PCSX hook API and real runtime remain console
        # gates and are never certified with mock host implementations here.

    def retained_header(self):
        path=ROOT/'artifacts/gameplay-helpers/p10/complete-size-failure/current-Os-cpp/exported/scene.hh'
        self.assertTrue(path.is_file(),'The real retained preflight failure header must be preserved')
        self.assertEqual(subject.sha(path),'12e0c3f74aa5390a5180b1f4739a2d077d715991a1cbc978444fa870e11066cf')
        return path.read_text(encoding='utf-8')

    def test_real_retained_header_uses_resident_scene_not_widget_prototype(self):
        header=self.retained_header()
        self.assertEqual(header.count('"Combat HUD",true,'),2)
        self.assertEqual(subject.header_slot(header,'Combat HUD'),45)
        self.assertEqual(subject.header_slot(header,'Player Aqua'),21)
        self.assertEqual(subject.header_slot(header,'Access Key'),41)
        self.assertEqual(subject.header_slot(header,'Lock-on Marker'),23)
        # The reusable Shop prototype is available to spawn, not a resident slot.
        with self.assertRaisesRegex(ValueError,'compiled actor label'):
            subject.header_slot(header,'DungeonShopWidget')

    def test_real_retained_scope_rejects_duplicate_actor_or_component(self):
        header=self.retained_header()
        duplicate=header.replace('"Combat HUD",true,-1,-1,-1,actor_components_1,','"Combat HUD",true,-1,-1,-1,actor_components_1,3,nullptr},{UINT64_C(1),"Combat HUD",true,-1,-1,-1,actor_components_1,')
        with self.assertRaisesRegex(ValueError,'compiled actor label'):
            subject.header_slot(duplicate,'Combat HUD')
        scope=subject.startup_scene_scope(header)
        component=next(line for line in scope.splitlines() if line.startswith('inline constexpr ActorComponentRecord actor_components_1[]'))
        duplicate=header.replace(component,component+'\n'+component)
        with self.assertRaisesRegex(ValueError,'compiled resident root component'):
            subject.header_slot(duplicate,'Combat HUD')

    def test_real_retained_scope_rejects_wrong_loader_and_negative_slot(self):
        header=self.retained_header()
        with self.assertRaisesRegex(ValueError,'compiled startup actor table'):
            subject.header_slot(header.replace('load_actor_bank(scene_0::actor_table','load_actor_bank(foreign::actor_table'),'Combat HUD')
        component=next(line for line in subject.startup_scene_scope(header).splitlines() if line.startswith('inline constexpr ActorComponentRecord actor_components_1[]'))
        negative=component.replace('true,-1,45,0','true,-1,-1,0')
        self.assertNotEqual(component,negative)
        with self.assertRaisesRegex(ValueError,'no resident ActorData'):
            subject.header_slot(header.replace(component,negative),'Combat HUD')

    def scene_project(self,flavor='cpp'):
        # Copy current curated installed authoring; artificial header tests only
        # host source-to-header binding, never a claim about played target code.
        project=ROOT/'artifacts/gameplay-helpers/p10/final18/projects'/('dungeon-complete--'+flavor)
        installation=yaml.safe_load((project/'ProjectSettings/Template.epoksettings').read_text(encoding='utf-8'))
        manifest=yaml.safe_load(next(project.glob('*.epokproject')).read_text(encoding='utf-8'))
        scene=yaml.safe_load((project/manifest['startup_scene']).read_text(encoding='utf-8'))
        return installation,manifest,scene

    def call_scene(self,installation,manifest,scene):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory);(p/'ProjectSettings').mkdir()
            (p/'ProjectSettings/Template.epoksettings').write_text(yaml.safe_dump(installation),encoding='utf-8')
            (p/'test.epokproject').write_text(yaml.safe_dump(manifest),encoding='utf-8')
            target=p/manifest['startup_scene'];target.parent.mkdir(parents=True,exist_ok=True);target.write_text(yaml.safe_dump(scene),encoding='utf-8')
            original=ROOT/'artifacts/gameplay-helpers/p10/final18/projects'/('dungeon-complete--'+installation['gameplay'])
            extension={'cpp':'.hpp','blueprint':'.epokbp','lua':'.lua'}[installation['gameplay']]
            for role in subject.LEAVES:
                source=next((original/'assets').rglob(installation['roles'][role]['name']+extension))
                target=p/source.relative_to(original);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
            if installation['gameplay']=='lua':shutil.copy2(original/'ProjectSettings/LuaClasses.epoksettings',p/'ProjectSettings/LuaClasses.epoksettings')
            components='\n'.join('inline constexpr ActorComponentRecord actor_components_'+str(i)+'[]={{UINT64_C(1),"Transform",true,-1,'+str(i)+',-1}};' for i,a in enumerate(scene['actors']))
            records=','.join('{UINT64_C(1),'+json.dumps(a['name'])+',true,-1,-1,-1,actor_components_'+str(i)+',8,nullptr}' for i,a in enumerate(scene['actors']))
            header='namespace epok::scene_0 {\n'+components+'\ninline constexpr ActorRecord actor_records[]={'+records+'};\n}\nnamespace epok {inline void load_bank_0(){load_actor_bank(scene_0::actor_table,objects.data(),object_count);} inline void initialize_scripts(){load_bank_0();}}'
            return subject.scene_contract(p,header)

    def test_all_three_current_authorings_resolve_ten_selected_roles(self):
        for flavor in ['cpp','blueprint','lua']:
            with self.subTest(flavor=flavor):
                value=self.call_scene(*self.scene_project(flavor))
                self.assertEqual(set(value['selected_roles']),set(subject.LEAVES))
                self.assertEqual(len(value['slots']),11)

    def test_explicit_player_binding_survives_moving_game_and_vitals(self):
        install,manifest,scene=self.scene_project()
        player=next(a for a in scene['actors'] if a['class']['class_id']==install['roles']['player']['class_id'])
        moved=[c for a in scene['actors'] for c in a['components'] if c['class']['class_id'] in [install['roles'][r]['class_id']for r in ['game_root','player_vitals']]]
        game=next(c for c in moved if c['class']['class_id']==install['roles']['game_root']['class_id'])
        game['properties']['player_actor']=player['id']
        owner=next(a for a in scene['actors'] if game in a['components'])
        other=next(a for a in scene['actors'] if a['id'] not in [player['id'],owner['id']] and len(a['components'])<=6)
        for actor in scene['actors']:actor['components']=[c for c in actor['components'] if c not in moved]
        other['components']+=moved
        self.assertTrue(self.call_scene(install,manifest,scene)['slots']['player']>=0)
        game['properties']['player_actor']=other['id']
        with self.assertRaisesRegex(ValueError,'selected current player'):self.call_scene(install,manifest,scene)

    def test_wrong_controller_and_duplicate_role_rejected(self):
        install,manifest,scene=self.scene_project()
        controller=next(c for a in scene['actors'] for c in a['components'] if c['class']['class_id']==install['roles']['controller']['class_id'])
        controller['properties']['combat_policy']='wrong'
        with self.assertRaisesRegex(ValueError,'selected game/combat'):self.call_scene(install,manifest,scene)
        install,manifest,scene=self.scene_project()
        actor=next(a for a in scene['actors'] if a['class']['class_id']==install['roles']['game_hud']['class_id'])
        duplicate=copy.deepcopy(actor);duplicate['id']='new';duplicate['components']=[];scene['actors'].append(duplicate)
        with self.assertRaisesRegex(ValueError,'selected role game_hud'):self.call_scene(install,manifest,scene)

    def test_compiled_telemetry_reads_full_fifteen_words_and_current_stride(self):
        syms,types=fake_abi();value=subject.compiled_contract(None,None,{'slots':{'player':4}},syms,types)
        self.assertEqual(value['names']['combat'][1],15)
        self.assertEqual(value['player_position_address'],0x40000+4*1400+8)
        self.assertEqual(value['frame_entry'],syms['DungeonGame::on_frame(unsigned long)']['address'])

    def test_changed_telemetry_prefix_and_linked_size_rejected(self):
        syms,types=fake_abi();types['DungeonTelemetry'][0]['members']['health']=32
        with self.assertRaisesRegex(ValueError,'prefix'):subject.compiled_contract(None,None,{'slots':{'player':4}},syms,types)
        syms,types=fake_abi();syms['combat_telemetry']['size']=36
        with self.assertRaisesRegex(ValueError,'mismatch'):subject.compiled_contract(None,None,{'slots':{'player':4}},syms,types)

    def test_cloned_or_missing_frame_boundary_is_not_guessed(self):
        syms,types=fake_abi();syms['DungeonGame::on_frame(unsigned int)']=syms['DungeonGame::on_frame(unsigned long)']
        with self.assertRaisesRegex(ValueError,'ambiguous'):subject.compiled_contract(None,None,{'slots':{'player':4}},syms,types)

    def test_partial_guards_boss_never_infers_death_heal_or_cycle(self):
        result=subject.validate_observation([row(1),row(2,phase=4,state=1,health=85,shots=10)])
        self.assertFalse(any(result.values()))

    def test_required_real_transitions_are_independently_checked(self):
        self.assertTrue(all(subject.validate_observation(complete_rows(),require_closure=True).values()))
        for field,value in [('damage_audio',0),('heal_audio',0),('hit_audio',0),('image_enabled',False),('boss_destroyed_audio',0),('pack_inactive',False)]:
            rows=complete_rows()
            for r in rows:r[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):subject.validate_observation(rows,require_closure=True)

    def test_counter_reset_cannot_look_like_healing(self):
        rows=complete_rows();rows[9]['restarts']=2
        self.assertFalse(subject.validate_observation(rows)['attack/feedback/heal'])

    def test_stale_target_and_freeaim_marker_fail(self):
        for mutate in ['target','marker']:
            rows=complete_rows()
            if mutate=='target':rows[6]['target_valid']=False
            else:rows[7]['marker_active']=True
            self.assertFalse(subject.validate_observation(rows)['lock/freeaim/targetcycle'])

    def test_killed_target_reacquisition_is_not_target_cycling(self):
        rows=complete_rows();rows[6]['previous_target_valid']=False
        self.assertFalse(subject.validate_observation(rows)['lock/freeaim/targetcycle'])
        rows=complete_rows();rows[6]['pad_mask']=0
        self.assertFalse(subject.validate_observation(rows)['lock/freeaim/targetcycle'])

    def test_death_requires_fade_and_music_stop_before_restart(self):
        for field,value in [('fade',0),('music_volume',1000),('music_playing',True)]:
            rows=complete_rows();rows[12][field]=value
            self.assertFalse(subject.validate_observation(rows)['death/retry/victory'])

    def test_missing_audio_and_nonmonotonic_frames_fail(self):
        rows=complete_rows();rows[-1]['audio_missing']=1
        with self.assertRaises(ValueError):subject.validate_observation(rows)
        with self.assertRaises(ValueError):subject.validate_observation([row(1),row(1)])

    def test_rule_edit_requires_selected_language_executed_changed_image(self):
        for flavor,extension in [('cpp','.hpp'),('blueprint','.epokbp'),('lua','.lua')]:
            path='assets/rules/Guard'+extension
            control=dict(mode='native_cpp',flavor=flavor,image_sha256='a'*64,execution_accepted=True,
                         selected_rule_files={'encounter_policy':path},source_inventory={path:'1','assets/scene':'same'},route_settings_sha256='c'*64)
            control['source_at_execution']=dict(control['source_inventory'])
            edited=copy.deepcopy(control);edited['image_sha256']='b'*64;edited['source_inventory'][path]='2';edited['source_at_execution']=dict(edited['source_inventory'])
            before=[dict(step=1,health=90)];after=[dict(step=1,health=80)]
            self.assertTrue(subject.validate_rule_edit(control,edited,'encounter_policy',before,after)['passed'])
            for broken in ['unchanged_image','foreign_edit','stale','same_behavior','wrong_steps','different_input']:
                invalid=copy.deepcopy(edited);b=copy.deepcopy(after)
                if broken=='unchanged_image':invalid['image_sha256']=control['image_sha256']
                if broken=='foreign_edit':invalid['source_inventory']['assets/scene']='other';invalid['source_at_execution']=dict(invalid['source_inventory'])
                if broken=='stale':invalid['source_at_execution']=control['source_inventory']
                if broken=='same_behavior':b=before
                if broken=='wrong_steps':b[0]['step']=2
                if broken=='different_input':invalid['route_settings_sha256']='d'*64
                with self.subTest(flavor=flavor,broken=broken),self.assertRaises(ValueError):subject.validate_rule_edit(control,invalid,'encounter_policy',before,b)

    def test_rule_receipt_rejects_foreign_current_source_before_reading_capture(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory);exe=p/'actual.ps-exe';exe.write_bytes(b'current')
            receipt=p/'foreign.json';receipt.write_text(json.dumps({'role':'run_flow','control':{'source_inventory':{'foreign':'x'},'image_sha256':subject.sha(exe)},'variant':{}}))
            with self.assertRaisesRegex(ValueError,'not the current final18'):
                subject.load_rule_edit_receipt(receipt,{'source_inventory':{'actual':'x'},'executable':exe})

    def test_driver_preserves_gates_and_no_fixture_substitution(self):
        source=(ROOT/'tests/integration/verify_dungeon_complete.py').read_text(encoding='utf-8')
        compile(source,'verify_dungeon_complete.py','exec')
        self.assertNotIn('"Player Aqua"',source);self.assertNotIn("a['name'] == 'Access Key'",source)
        self.assertIn('if us>25000',source)
        self.assertIn('fps[1] == 0',source)
        self.assertIn("maximum_dropped == maximum_skipped == maximum_failed == 0",source)
        lua=(HERE/'complete_observation.lua').read_text(encoding='utf-8')
        self.assertNotRegex(lua,r"(?:target_handle|image|health|phase|generation)\s*\[[^]]*\]\s*=")
        self.assertNotIn('writeMemory',lua)

if __name__=='__main__':unittest.main(verbosity=2)
