"""Authored workflows: source contracts, real Lua bodies and fail-closed observations."""
from pathlib import Path
import copy
import json
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE));sys.path.insert(0,str(HERE.parent))
import authored_edits as edits
from test_observer import LuaHost, MOCK, model_config

ROOT=edits.ROOT
GRAPH=ROOT/'artifacts/gameplay-helpers/p10/final18/projects/sample--blueprint/assets/Blueprints/Spinner.epokbp'
REGISTRY={'classes':[{'id':'actor-id','cpp_name':'epok::Actor','functions':[{'id':'actual-compiled-Actor-tick','name':'tick','event':True}]}]}


class EditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.lua=LuaHost()
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='authored-host-',dir=HERE);self.work=Path(self.temp.name)
    def tearDown(self):self.temp.cleanup()
    def copied(self,template,flavor):
        original=ROOT/'artifacts/gameplay-helpers/p10/final18/projects'/(template+'--'+flavor)
        project=self.work/(template+'-'+flavor);inputs=edits.clone_sources(original,project)
        self.assertEqual(edits.adapter.inventory(original),inputs);self.assertFalse((project/'.epok').exists())
        return original,project,inputs
    def test_actual_six_selected_language_sources_are_edited_without_original_changes(self):
        for template in ['basic','sample']:
            for flavor in ['cpp','blueprint','lua']:
                original,project,inputs=self.copied(template,flavor)
                source=edits.edit_selected_source(project,template,flavor,REGISTRY,GRAPH)
                self.assertEqual(edits.adapter.inventory(original),inputs)
                self.assertNotEqual(edits.adapter.sha(source),inputs[source.relative_to(project).as_posix()])
                changed=[name for name in inputs if edits.adapter.sha(project/name)!=inputs[name]]
                self.assertEqual(changed,[source.relative_to(project).as_posix()])
                if flavor=='blueprint':
                    graph=edits.documents.loads(source.read_text(encoding='utf-8'))
                    self.assertEqual(next(v for v in graph['variables'] if v['name']=='speed')['default'],45 if template=='basic' else 90)
                    tick=next(f for f in graph['functions'] if f['name']=='tick')
                    if template=='basic':
                        self.assertEqual(tick['override_id'],'actual-compiled-Actor-tick')
                        kinds=[n.get('kind',{}).get('operation',{}).get('kind') for n in tick['nodes']]
                        self.assertIn('self_object',kinds);self.assertNotIn('get_owner',kinds)

    def test_actual_edited_lua_rules_and_inherited_speed_run_in_host_luajit(self):
        for template,name,speed in [('basic','GameRoot',45),('sample','Spinner',90)]:
            _,project,_=self.copied(template,'lua')
            path=edits.edit_selected_source(project,template,'lua',REGISTRY,GRAPH)
            source=path.read_text(encoding='utf-8')
            program='''
epok={Actor3D={},ActorComponent={}}
local function extend(parent)
 local child={};child.__index=child;child.extend=extend
 return setmetatable(child,{__index=parent})
end
epok.Actor3D.extend=extend;epok.ActorComponent.extend=extend
'''
            program+='\nlocal selected=(function()\n'+source+'\nend)()\n'
            program+='''local derived=selected:extend()
local primary=setmetatable({rotation={x=0,y=0,z=0}},selected)
local secondary=setmetatable({rotation={x=0,y=0,z=0},speed=SPEED*2},derived)
local remainder=0
for step=1,120 do
 remainder=remainder+16;local delta=68
 if remainder>=60 then delta=69;remainder=remainder-60 end
 primary:tick(delta/4096);secondary:tick(delta/4096)
 local expected=SPEED*(68*step+math.floor(16*step/60))
 assert(primary.rotation.y*4096==expected and secondary.rotation.y*4096==2*expected,'Real authored Lua inheritance arithmetic differs')
end
'''.replace('SPEED',str(speed))
            self.lua.run(program)

    def test_real_blueprint_graph_is_an_algorithm_with_inherited_variable(self):
        _,project,_=self.copied('basic','blueprint')
        path=edits.edit_selected_source(project,'basic','blueprint',REGISTRY,GRAPH)
        asset=edits.documents.loads(path.read_text(encoding='utf-8'));function=asset['functions'][0];nodes={n['id']:n for n in function['nodes']}
        variable=asset['variables'][0]['id'];write=next(n for n in nodes.values() if n.get('kind',{}).get('operation',{}).get('kind')=='set_rotation')
        def run(speed):
            rotation=[0,0,0]
            for step in range(1,121):
                delta=68+(16*step//60-16*(step-1)//60)
                def value(input):
                    if input['kind']=='parameter':return delta
                    node=nodes[input['node']];kind=node['kind'];tag=kind['kind']
                    if tag=='get_variable':self.assertEqual(kind['member'],variable);return speed*4096
                    if tag=='builtin':
                        op=kind['operation']['kind']
                        if op=='self_object':return 'self'
                        if op=='get_rotation':self.assertEqual(value(node['inputs']['target']),'self');return list(rotation)
                    if tag=='vector_component':return value(node['inputs']['value'])[kind['index']]
                    if tag=='binary':
                        a=value(node['inputs']['a']);b=value(node['inputs']['b'])
                        return a*b//4096 if kind['op']=='multiply' else a+b
                    if tag=='make_vector':return [value(node['inputs'][axis]) for axis in ['x','y','z']]
                    raise AssertionError('Unexpected graph contract '+tag)
                rotation=value(write['inputs']['value'])
                self.assertEqual(rotation,[0,speed*(68*step+16*step//60),0])
            return rotation
        self.assertEqual(run(90)[1],2*run(45)[1])

    def valid_observation(self,speed=45):
        executable=self.work/'edited.ps-exe';executable.write_bytes(b'host-model-only')
        receipt={'primary_speed':speed,'secondary_speed':speed*2,'helper_half_extents':[8192,6144,4096],'edited_inventory':{'only':'host-model'}}
        rows=[dict(step=n,owner=True,secondary_owner=True,rotation=[0,speed*(68*n+16*n//60),0],secondary_rotation=[0,speed*2*(68*n+16*n//60),0],helper_enabled=True,helper_trigger=True,helper_half_extents=[8192,6144,4096])for n in range(1,121)]
        value=dict(reason='exact-steps',errors=[],executable_sha256=edits.adapter.sha(executable),source_inventory_sha256=edits.adapter.digest(receipt['edited_inventory']),trace=rows,frames=[{'cpu_cycles':100}],cpu_work_units=100)
        return value,receipt,executable
    def test_actual_observation_requires_both_angular_rules_and_helper_tuning(self):
        value,receipt,exe=self.valid_observation();edits.validate_observation(value,receipt,exe)
        value['trace'][70]['secondary_rotation'][1]+=1
        with self.assertRaisesRegex(ValueError,'inherited runtime rule'):edits.validate_observation(value,receipt,exe)
        value,receipt,exe=self.valid_observation();value['trace'][10]['helper_half_extents'][0]=4096
        with self.assertRaisesRegex(ValueError,'helper did not apply'):edits.validate_observation(value,receipt,exe)
    def test_missing_trace_or_foreign_source_cannot_pass(self):
        value,receipt,exe=self.valid_observation();value['trace'].pop()
        with self.assertRaisesRegex(ValueError,'every observed step'):edits.validate_observation(value,receipt,exe)
        value,receipt,exe=self.valid_observation();value['source_inventory_sha256']='wrong'
        with self.assertRaisesRegex(ValueError,'provenance mismatch'):edits.validate_observation(value,receipt,exe)
    def test_workflow_refuses_original_project_or_existing_fixture_overwrite(self):
        original=ROOT/'artifacts/gameplay-helpers/p10/final18/projects/basic--cpp'
        with self.assertRaisesRegex(ValueError,'outside the original'):edits.Workflow(ROOT,ROOT/'target/debug/epok-editor.exe',original,original/'unsafe',GRAPH)
        target=self.work/'existing';target.mkdir()
        with self.assertRaisesRegex(ValueError,'Preserve existing'):edits.clone_sources(original,target)
    def test_composed_observer_uses_real_tick_return_and_has_no_target_writer(self):
        config=model_config('basic','host-not-written');config.update(secondary_name='Derived',helper_name='Helper',primary_speed=45,secondary_speed=90,observation_boundary='selected-actor-tick-return')
        source=MOCK+'\n'+edits.observer_source(config)
        self.lua.run(source)
        self.assertNotIn("P10_TIME_TICKS=",source);self.assertIn('snapshot_step(\'tick-return\')',source)
        self.assertNotIn('device_write',source)


if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(EditTests))
    edits.adapter.write(HERE/'host-result.json',dict(passed=result.wasSuccessful(),tests_run=result.testsRun,failures=len(result.failures),errors=len(result.errors),
      scope='Host source copies, actual edited Lua execution, Blueprint graph model and composed Lua observer load; editor/catalog/MIPS/runtime execution NOT run',
      sources={n:edits.adapter.sha(HERE/n) for n in ['authored_edits.py','angular_observer.lua','test_authored_edits.py']}))
    raise SystemExit(not result.wasSuccessful())
