"""Offline negative contracts only; synthetic data is never target evidence."""
from pathlib import Path
import copy
import tempfile
import unittest
import zipfile
import dungeon_routes as route
import adapter

class DungeonRouteTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='dungeon-route-offline-',dir=Path(__file__).resolve().parent)
        self.base=Path(self.temp.name);self.project=self.base/'actual';self.program=self.project/'.epok/build';self.program.mkdir(parents=True)
        self.export=self.base/'relocated';self.export.mkdir()
        for folder in [self.program,self.export]:
            for name,value in {'epok.ps-exe':b'EXACT-IMAGE','epok.elf':b'DEBUG','epok.map':b'MAP','scene.hh':b'EXACT-HEADER','GEOMETRY.BIN':b'GEOMETRY'}.items():(folder/name).write_bytes(value)
        roles={'player':{'class_id':'player-class'},'key':{'class_id':'chosen-key'},'door':{'class_id':'chosen-door'}}
        scene={'actors':[{'id':'hero','name':'Renamed hero','class':{'class_id':'player-class'},'components':[]},
          {'id':'key','name':'Renamed key','class':{'class_id':'actor'},'components':[{'id':'flag','class':{'class_id':'flag-class'}},{'id':'key-rule','class':{'class_id':'chosen-key'},'properties':{'accepted_initiator':'hero','auto_collect':True,'collected_flag':'flag'}}]},
          {'id':'gate','name':'Renamed gate','class':{'class_id':'actor'},'components':[{'id':'door-rule','class':{'class_id':'chosen-door'},'properties':{'requirement_flag':'flag'}}]}]}
        self.binding={'project':self.project,'executable':self.program/'epok.ps-exe','source_inventory':{'assets/Rules.lua':'c'*64},'scene':scene,'installation':{'roles':roles},'template':'dungeon'}
        self.exact=route.exact_program(self.binding,self.program)
    def tearDown(self):self.temp.cleanup()
    def test_roles_bind_selected_uuid_after_rename(self):
        self.assertEqual(route.actor_roles(self.binding),{'player':'Renamed hero','key':'Renamed key','door':'Renamed gate'})
    def test_wrong_initiator_and_foreign_requirement_rejected(self):
        bad=copy.deepcopy(self.binding);bad['scene']['actors'][1]['components'][1]['properties']['accepted_initiator']='foreign'
        with self.assertRaisesRegex(ValueError,'actual player'):route.actor_roles(bad)
        bad=copy.deepcopy(self.binding);bad['scene']['actors'][2]['components'][0]['properties']['requirement_flag']='foreign'
        with self.assertRaisesRegex(ValueError,'flag binding'):route.actor_roles(bad)
    def test_wrong_image_and_header_rejected(self):
        (self.export/'epok.ps-exe').write_bytes(b'FOREIGN')
        with self.assertRaisesRegex(ValueError,'Foreign program'):route.exact_program(self.binding,self.export)
        (self.export/'epok.ps-exe').write_bytes(b'EXACT-IMAGE');(self.export/'scene.hh').write_bytes(b'FOREIGN-HEADER')
        with self.assertRaisesRegex(ValueError,'header differs'):route.exact_program(self.binding,self.export)
    def test_package_uses_exact_export_not_a_fixture(self):
        target=self.base/'program.zip';route.package_current(self.binding,self.export,target)
        with zipfile.ZipFile(target)as package:
            self.assertEqual(package.read('epok.ps-exe'),b'EXACT-IMAGE');self.assertEqual(package.read('debug/epok.elf'),b'DEBUG')
            self.assertEqual(set(package.namelist()),{'epok.ps-exe','debug/epok.elf','debug/epok.map','GEOMETRY.BIN','epok-package.json'})
    def observation(self):
        rows=[{'frame':1,'step':1,'key':True,'door':True,'cpu_cycles':100},
              {'frame':32,'step':32,'key':False,'door':True,'cpu_cycles':100},
              {'frame':120,'step':120,'key':False,'door':False,'cpu_cycles':100}]
        return {'reason':'key-door-complete','failures':[],'rows':rows,'executable_sha256':self.exact['exe_sha256'],'header_sha256':self.exact['header_sha256'],'source_inventory_sha256':adapter.digest(self.binding['source_inventory'])}
    def test_real_observation_binding_checks_all_three_identities(self):
        for field in ['executable_sha256','header_sha256','source_inventory_sha256']:
            value=self.observation();value[field]='b'*64
            with self.assertRaisesRegex(ValueError,'Foreign source/image/header'):route.validate_dungeon(value,self.binding,self.exact)
    def test_default_reset_and_single_transitions_required(self):
        value=self.observation();route.validate_dungeon(value,self.binding,self.exact)
        value['rows'][0]['key']=False
        with self.assertRaisesRegex(ValueError,'lifecycle'):route.validate_dungeon(value,self.binding,self.exact)
        value=self.observation();value['rows'].insert(2,{'frame':50,'step':50,'key':True,'door':True,'cpu_cycles':100})
        with self.assertRaisesRegex(ValueError,'exactly once'):route.validate_dungeon(value,self.binding,self.exact)
    def test_missing_cpu_or_steps_never_pass(self):
        value=self.observation();value['rows'][0]['cpu_cycles']=None
        with self.assertRaisesRegex(ValueError,'CPU work missing'):route.validate_dungeon(value,self.binding,self.exact)
        value=self.observation();value['rows'][-1]['step']=119
        with self.assertRaisesRegex(ValueError,'simulation step'):route.validate_dungeon(value,self.binding,self.exact)
    def test_semantic_normalization_does_not_hash_addresses_or_timing(self):
        a=self.observation();b=copy.deepcopy(a)
        for row in b['rows']:row['cpu_cycles']*=3;row['frame']+=1
        self.assertEqual(route.normalize_dungeon(a),route.normalize_dungeon(b))
        b['rows'][-1]['door']=True
        self.assertNotEqual(route.normalize_dungeon(a),route.normalize_dungeon(b))
    def test_complete_drop_and_image_gates_hold_while_frame_time_is_reported(self):
        value={'executable_sha256':'a'*64,'accepted':True,'slow_frames':0,'max_frame_us':25000,'max_dropped_triangles':0,'max_stream_skipped_chunks':0,'max_stream_failed_chunks':0,'missing_sound_assets':0}
        route.validate_complete(value,'a'*64)
        for field in ['max_dropped_triangles','max_stream_skipped_chunks','max_stream_failed_chunks','missing_sound_assets']:
            bad=value.copy();bad[field]=1
            with self.assertRaisesRegex(ValueError,'runtime gate'):route.validate_complete(bad,'a'*64)
        slow=value.copy();slow.update(accepted=False,slow_frames=9,max_frame_us=26368)
        route.validate_complete(slow,'a'*64)
        with self.assertRaisesRegex(ValueError,'wrong image'):route.validate_complete(value,'b'*64)
    def test_disc_routes_ignore_only_frame_timed_observation_fields(self):
        row={'frame':1,'step':2,'restarts':1,'state':0,'health':100,'fade':0,'music_volume':10,'image_color':[1,2,3],'image_enabled':True,'borders':[0,0,0,0]}
        value={'result':{'route':'closure','states':[1],'maximum_guards':3,'shots':1,'boss_health':0},
               'replay':{'events':[],'pad_changes':[],'stopped_frame':12000,'work':[]},'counters':{'game':[1],'combat':[2],'flow':[4]},'observation':{'rows':[row]}}
        timed=copy.deepcopy(value);timed['observation']['rows'][0].update(frame=2,fade=1,music_volume=11,image_color=[4,5,6],image_enabled=False,borders=[1,1,1,1])
        self.assertTrue(route.routes_match(route.normalize_complete_route(value,disc=True),route.normalize_complete_route(timed,disc=True)))
        self.assertNotEqual(route.normalize_complete_route(value),route.normalize_complete_route(timed))
        state=copy.deepcopy(value);state['observation']['rows'][0]['health']=90
        self.assertFalse(route.routes_match(route.normalize_complete_route(value,disc=True),route.normalize_complete_route(state,disc=True)))
        shifted=copy.deepcopy(value);shifted['observation']['rows'][0]['step']=3
        self.assertFalse(route.routes_match(route.normalize_complete_route(value,disc=True),route.normalize_complete_route(shifted,disc=True)))
    def test_disc_routes_keep_restarted_runs_apart(self):
        # The step count starts again at a restart; one frame boundary moving must not
        # decide which run's row a shared step number compares.
        first={'frame':1,'step':5,'restarts':1,'health':64}
        second={'frame':9,'step':5,'restarts':3,'health':100}
        later={'frame':10,'step':5,'restarts':3,'health':90}
        keyed=route.step_keyed([first,second,later])
        self.assertEqual(keyed,{'1:5':[{'step':5,'restarts':1,'health':64}],'3:5':[{'step':5,'restarts':3,'health':100},{'step':5,'restarts':3,'health':90}]})
        self.assertNotEqual(keyed,route.step_keyed([first,later]))
    def test_complete_semantics_retain_exact_steps_and_gameplay_counters(self):
        value={'result':{'route':'guards','states':[1],'maximum_guards':3,'shots':20,'boss_health':280},
               'replay':{'events':[],'pad_changes':[],'stopped_frame':1800,'work':[{'phase':4,'steps':1,'frames':100,'cpu_scanlines':50}]},
               'counters':{'game':[1],'combat':[2],'flow':[4]},'observation':{'rows':[]}}
        a=route.normalize_complete_route(value);changed=copy.deepcopy(value);changed['replay']['work'][0]['cpu_scanlines']=70
        self.assertEqual(a,route.normalize_complete_route(changed))
        changed['replay']['work'][0]['steps']=2
        self.assertNotEqual(a,route.normalize_complete_route(changed))
    def test_fixture_creation_commands_never_accepted(self):
        with self.assertRaisesRegex(ValueError,'substitute a fixture'):route.command(['python','tool.py','--create-project','foreign'],self.base/'x.log',self.base)

if __name__=='__main__':unittest.main()
