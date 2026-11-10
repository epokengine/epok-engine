"""Offline contract tests; synthetic files never count as runtime acceptance."""
from pathlib import Path
import copy
import json
import tempfile
import unittest
import adapter
import layout

class AdapterTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='adapter-offline-',dir=Path(__file__).resolve().parent)
        self.base=Path(self.temp.name);self.root=self.base/'repository';self.project=self.root/'actual'
        self.export=self.base/'relocated';self.root.mkdir();self.project.mkdir();self.export.mkdir()
        self.dependency='a'*64
        self.case='basic/lua/native_cpp'
        adapter.write(self.project/'project.epokproject',{'startup_scene':'assets/scenes/Main.epokmap','lua_execution':'native_cpp','rendering':{'width':320,'height':240}})
        adapter.write(self.project/'ProjectSettings/Template.epoksettings',{'template':'basic','gameplay':'lua','roles':{'game_root':{'name':'GameRoot','class_id':'root-class'}}})
        self.scene={'actors':[{'id':'camera','name':'Camera','class':{'name':'epok::Actor3D','class_id':'engine-actor'},'components':[]},
                              {'id':'source-metadata','name':'Source Metadata3','class':{'name':'epok::Actor3D','class_id':'engine-actor'},'components':[]},
                              {'id':'root','name':'Renamed Empty Root','class':{'name':'GameRoot','class_id':'root-class'},'components':[{'root':True,'class':{'name':'epok::SceneComponent3D'}}]}]}
        adapter.write(self.project/'assets/scenes/Main.epokmap',self.scene)
        (self.project/'.epok/build').mkdir(parents=True);(self.project/'.epok/build/epok.ps-exe').write_bytes(b'EXACT-AOT-PROGRAM')
        (self.export/'epok.ps-exe').write_bytes(b'EXACT-AOT-PROGRAM')
        adapter.write(self.export/'.p10-export.json',{'dependency_sha256':self.dependency})
        self.export_manifest=self.base/'export-manifest.json'
        adapter.write(self.export_manifest,{'relocated':str(self.export),'executable_sha256':adapter.sha(self.export/'epok.ps-exe'),'external_assets':{},'inputs':{'.p10-export.json':adapter.sha(self.export/'.p10-export.json')}})
    def tearDown(self):self.temp.cleanup()
    def bind(self,**kwargs):
        args=dict(root=self.root,project=self.project,case=self.case,mode='native_cpp',dependency=self.dependency,export_root=self.export,export_manifest=self.export_manifest)
        args.update(kwargs);return adapter.validate_binding(**args)
    def test_current_basic_three_actors_inert_root_not_old_p7_count(self):
        binding=self.bind();self.assertEqual(len(binding['scene']['actors']),3)
        self.assertEqual(adapter.select_actor(binding)['id'],'root')
    def test_foreign_project_rejected_even_if_program_identical(self):
        data=adapter.documents(self.project/'ProjectSettings/Template.epoksettings');data['template']='sample'
        adapter.write(self.project/'ProjectSettings/Template.epoksettings',data)
        with self.assertRaisesRegex(ValueError,'Foreign project'):self.bind()
    def test_foreign_dependency_export_rejected(self):
        adapter.write(self.export/'.p10-export.json',{'dependency_sha256':'b'*64})
        with self.assertRaisesRegex(ValueError,'different dependency'):self.bind()
    def test_wrong_export_program_rejected(self):
        (self.export/'epok.ps-exe').write_bytes(b'OTHER')
        with self.assertRaisesRegex(ValueError,'executable identity mismatch'):self.bind()
    def test_export_original_repo_path_rejected(self):
        with self.assertRaisesRegex(ValueError,'relocated outside'):self.bind(export_root=self.project)
    def test_changed_export_source_rejected(self):
        source=self.export/'build.ps1';source.write_text('current')
        manifest=adapter.read(self.export_manifest);manifest['inputs']['build.ps1']=adapter.sha(source);adapter.write(self.export_manifest,manifest)
        source.write_text('foreign')
        with self.assertRaisesRegex(ValueError,'Export input changed'):self.bind()
    def observation(self,binding):
        trace=[{'step':n,'position':[0,0,0],'rotation':[0,0,0],'scale':[4096]*3,'owner':True} for n in range(1,3)]
        return {'executable_sha256':adapter.sha(binding['executable']),'source_inventory_sha256':adapter.digest(binding['source_inventory']),
                'reason':'exact-steps','errors':[],'diagnostic_memory':False,'trace':trace,
                'clock_observer':{'kind':'time-write-and-frame-return' if binding['template']=='basic' else 'selected-actor-tick-return','address':0x80002000,'write_callbacks':2,'frames':1},
                'step_observations':[{'step':n,'boundary':('time-write' if n==1 else 'frame-return') if binding['template']=='basic' else 'tick-return','frame':1,'cpu_work_units':n*1250} for n in range(1,3)],
                'frames':[{'cpu_cycles':2500,'observer_frame':1,'frame_sequence':1,'ticks':2,'start_ticks':0,'executed_steps':2,'requested_steps':2}],'cpu_work_units':2500}
    def test_observation_wrong_source_and_image_rejected(self):
        binding=self.bind();value=self.observation(binding);value['source_inventory_sha256']='b'*64
        with self.assertRaisesRegex(ValueError,'another source/image'):adapter.validate_observation(value,binding,2,False)
        value=self.observation(binding);value['executable_sha256']='b'*64
        with self.assertRaisesRegex(ValueError,'another source/image'):adapter.validate_observation(value,binding,2,False)
    def test_missing_steps_or_cpu_not_autogreen(self):
        binding=self.bind();value=self.observation(binding);value['trace'].pop()
        with self.assertRaisesRegex(ValueError,'equal-step'):adapter.validate_observation(value,binding,2,False)
        value=self.observation(binding);value['cpu_work_units']=0
        with self.assertRaisesRegex(ValueError,'CPU work unmeasured'):adapter.validate_observation(value,binding,2,False)
    def test_basic_missing_time_write_or_final_read_only_not_accepted(self):
        binding=self.bind();value=self.observation(binding);value['clock_observer']['write_callbacks']=0
        with self.assertRaisesRegex(ValueError,'Time write observation never triggered'):adapter.validate_observation(value,binding,2,False)
        value=self.observation(binding);value['step_observations']=value['step_observations'][-1:]
        with self.assertRaisesRegex(ValueError,'per-step clock observations'):adapter.validate_observation(value,binding,2,False)
    def test_basic_actual_movement_fails(self):
        binding=self.bind();value=self.observation(binding);value['trace'][1]['rotation'][1]=1
        with self.assertRaisesRegex(ValueError,'not inert'):adapter.validate_observation(value,binding,2,False)
    def test_sample_independent_fixed_timestep_oracle(self):
        binding=self.bind();binding['template']='sample';value=self.observation(binding)
        for row in value['trace']:row['rotation'][1]=45*(68*row['step']+16*row['step']//60)
        adapter.validate_observation(value,binding,2,False)
        value['trace'][1]['rotation'][1]+=1
        with self.assertRaisesRegex(ValueError,'Spinner differs'):adapter.validate_observation(value,binding,2,False)
    def test_diagnostic_memory_cannot_be_perf_control(self):
        binding=self.bind();value=self.observation(binding);value['diagnostic_memory']=True
        with self.assertRaisesRegex(ValueError,'wrong observation phase'):adapter.validate_observation(value,binding,2,False)
    def test_dwarf_constant_layout_supported_unknown_not_guessed(self):
        text=''' <1><1>: Abbrev Number:1 (DW_TAG_structure_type)
 <2> DW_AT_name : (strp) (offset:0x1): ActorData
 <3> DW_AT_byte_size : (data2) 12
 <2><4>: Abbrev Number:2 (DW_TAG_member)
 <5> DW_AT_name : (string) transform
 <6> DW_AT_data_member_location: (implicit_const) 8
'''
        structures=layout.dwarf_structures(text)
        self.assertEqual(layout.require_layout(structures,'ActorData',['transform'])['members']['transform'],8)
        structures['ActorData'][0]['members']['transform']=None
        with self.assertRaisesRegex(ValueError,'Missing/ambiguous'):layout.require_layout(structures,'ActorData',['transform'])

if __name__=='__main__':unittest.main()
