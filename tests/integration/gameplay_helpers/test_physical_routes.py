"""Host contracts for explicit setup versus actual measured physical phases."""
from pathlib import Path
import copy
import importlib.util
import math
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
COMMON=HERE
ROOT=next(p for p in HERE.parents if (p / 'tools/epok_documents.py').is_file())
sys.path.insert(0,str(HERE));sys.path.append(str(COMMON))
import physical_support as physical
import capture_reuse
import adapter
spec=importlib.util.spec_from_file_location('physical_player_draft',HERE/'player_routes.py')
draft=importlib.util.module_from_spec(spec);spec.loader.exec_module(draft)
from test_observer import LuaHost
from test_player_routes import input_config,MOCK


def actual_binding(template,flavor='cpp'):
    project=ROOT/'artifacts/gameplay-helpers/p10/final18/projects'/(template+'--'+flavor)
    manifest=adapter.documents(next(project.glob('*.epokproject')))
    return {'template':template,'flavor':flavor,'project':project,'installation':adapter.documents(project/'ProjectSettings/Template.epoksettings'),
            'scene':adapter.documents(project/manifest['startup_scene'])}


def model(binding):
    route=physical.plan(binding);value={'physical_route':route,'physical_trace':[],'pose_setups':[],'physical_setup_observations':[],'trace':[{'step':i}for i in range(1,241)]}
    for stage in route['stages']:
        if stage.get('pose'):
            value['pose_setups'].append(dict(phase=stage['name'],tick=stage['start'],kind='position-only',position=[round(v*4096)for v in stage['pose']]))
        if stage.get('second_pose_step'):
            value['pose_setups'].append(dict(phase=stage['name'],tick=stage['start']+stage['second_pose_step']-1,kind='position-only',position=[round(v*4096)for v in stage['second_pose']]))
        if stage.get('jump_setup_step'):value['physical_setup_observations'].append(dict(phase=stage['name'],tick=stage['start']+stage['jump_setup_step']-1,kind='actual-jump-seed',jumped=True,velocity=[0,-252*4096]))
        if stage.get('lift_pose'):
            value['pose_setups'].append(dict(phase=stage['name'],tick=stage['start'],kind='position-only',position=[100*4096,-35*4096],sampled_support_world=[100*4096,-20*4096]))
        for n in range(1,stage['measure']+1):
            s=dict(step=stage['start']+stage['setup']+n-1,motion_status=0,animation_status=0,camera_valid=True,grounded=True,jumped=False,
              position=[0,0,0],velocity=[0,0,0],camera_pitch=18*4096,camera_position=[0,0,0],camera_blocked=False,blocked=False)
            row=dict(phase=stage['name'],step=n,global_tick=s['step'],sample=s,input={'step':s['step'],'buttons':stage['buttons'],'adc':[128]*4})
            if stage['name']=='ramp':s['position']=[round((-12+n*.065)*4096),round((.3 if 15<=n<=40 else 0)*4096),round(2.5*4096)]
            elif stage['name']=='wall':s['position']=[round(max(-15.22,-14.5-.08*n)*4096),0,-10*4096];s['blocked']=n>=12
            elif stage['name']=='boom-blocked':s.update(position=[-7*4096,0,round(-14.5*4096)],camera_position=[-7*4096,4096,round(-15.35*4096)],camera_blocked=True)
            elif stage['name']=='boom-clear':
                row['camera_distance']=round(5.4*4096);s['position']=[-7*4096,0,-6*4096]
                s['camera_position']=[-7*4096,4096,round(-6*4096-row['camera_distance']*math.cos(math.radians(18)))]
            elif stage['name']=='one-way':
                s.update(position=[-184*4096,(10 if n<10 else 20)*4096],velocity=[0,-4096 if n<10 else 0],grounded=n>=10,hit_tile=n>=10,tile=[-12,2]);row['on_ceiling']=False
            elif stage['name']=='plank-jump':s.update(position=[-184*4096,20*4096],jumped=n==1,hit_tile=True,tile=[-12,2]);row['on_ceiling']=False
            elif stage['name']=='lift-carry':
                at=[100*4096+n*100,-20*4096];s.update(position=[at[0],at[1]-15*4096],on_lift=True,hit_tile=False,tile=[0,0]);row.update(lift_world_position=at,on_ceiling=False)
            value['physical_trace'].append(row)
    return value


class PhysicalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.lua=LuaHost()
    def test_all_provider_plans_are_equal_and_prefix_frame_is_unmodified(self):
        for template in draft.TEMPLATES:
            plans=[physical.plan(actual_binding(template,flavor))for flavor in ['cpp','blueprint','lua']]
            self.assertEqual(plans[0],plans[1]);self.assertEqual(plans[0],plans[2])
            self.assertEqual(plans[0]['prefix_steps'],240);self.assertEqual(plans[0]['prefix_padding'],60)
            self.assertGreater(plans[0]['stages'][0]['start'],240+8)
    def test_valid_physical_models_preserve_original_thresholds(self):
        for template in draft.TEMPLATES:
            bound=actual_binding(template);result=physical.validate(model(bound),bound)
            self.assertTrue(result['setup_excluded_from_measured_phase_comparison'])
    def test_pose_during_measured_phase_is_rejected(self):
        bound=actual_binding('third-person');value=model(bound);value['pose_setups'][0]['tick']+=75
        with self.assertRaisesRegex(ValueError,'position setup write'):physical.validate(value,bound)
    def test_missing_actual_phase_or_wrong_setup_pose_is_rejected(self):
        bound=actual_binding('third-person');value=model(bound);value['physical_trace'].pop()
        with self.assertRaisesRegex(ValueError,'phase steps'):physical.validate(value,bound)
        value=model(bound);value['pose_setups'][0]['position'][0]+=1
        with self.assertRaisesRegex(ValueError,'reviewed setup'):physical.validate(value,bound)
    def test_wall_or_boom_failure_cannot_be_hidden_by_setup(self):
        bound=actual_binding('third-person');value=model(bound)
        for r in value['physical_trace']:
            if r['phase']=='wall':r['sample']['blocked']=False
        with self.assertRaisesRegex(ValueError,'wall response'):physical.validate(value,bound)
        value=model(bound)
        for r in value['physical_trace']:
            if r['phase']=='boom-blocked':r['sample']['camera_blocked']=False
        with self.assertRaisesRegex(ValueError,'occlusion'):physical.validate(value,bound)
    def test_empty_one_way_or_fake_lift_movement_cannot_pass(self):
        bound=actual_binding('platformer-2d');value=model(bound)
        for r in value['physical_trace']:
            if r['phase']=='one-way':r['sample']['velocity'][1]=0
        with self.assertRaisesRegex(ValueError,'upward one-way'):physical.validate(value,bound)
        value=model(bound)
        for r in value['physical_trace']:
            if r['phase']=='lift-carry':r['sample']['position'][0]-=1000
        with self.assertRaisesRegex(ValueError,'supported by the live lift'):physical.validate(value,bound)
        value=model(bound);value['physical_setup_observations'][0]['jumped']=False
        with self.assertRaisesRegex(ValueError,'actual input-driven jump'):physical.validate(value,bound)
    def test_normalized_comparison_excludes_setup_and_absolute_ticks(self):
        bound=actual_binding('third-person');value=model(bound);other=copy.deepcopy(value)
        for r in other['physical_trace']:r['global_tick']+=999;r['sample']['step']+=999;r['input']['step']+=999
        other['pose_setups'][0]['address']=0x80003000
        self.assertEqual(physical.normalized(value),physical.normalized(other))
        other['physical_trace'][0]['sample']['position'][0]+=1
        self.assertNotEqual(physical.normalized(value),physical.normalized(other))
    def test_actual_lua_keeps_240_input_prefix_and_has_only_explicit_setup_position_writes(self):
        config=input_config('third-person');config.update(player_name='Player',camera_name='Camera',physical_route=physical.plan(actual_binding('third-person')))
        config['steps']=config['physical_route']['steps']
        config['layouts']['Actor']={'size':16,'members':{'m_data':0}}
        config['layouts']['CameraRig3D']={'size':32,'members':{'sample_':0}}
        config['layouts']['CameraRigSample']={'size':16,'members':{'yaw':0}}
        source=MOCK+'\n'+draft.observer_source(config)+'''
for tick=1,300 do inject_input(tick)end
assert(#pose_setups==0,'Unmodified default prefix/padding wrote a pose')
actual_actor=function(name)return name=='Player' and 0x80001000 or 0x80002000 end
put32(0x80001000,0x80005000)
local first=config.physical_route.stages[1]
physical_input(first.start)
local address=0x80005000+config.layouts.ActorData.members.transform+config.layouts.Transform.members.position
assert(get32(address)==4294918144,'Real setup did not write reviewed x=-12')
assert(#pose_setups==1,'Initial setup not recorded')
put32(address,123)
for tick=first.start+1,first.start+first.setup+first.measure-1 do physical_input(tick)end
assert(get32(address)==123 and #pose_setups==1,'Observer kept forcing player position')
assert(not pcall(function()set_initial_position(first.start+first.setup,first,{0,0,0})end),'Measured phase accepted a pose write')
'''
        self.lua.run(source)
    def test_capture_dependency_parser_masks_braces_in_comments_and_literals(self):
        source='struct Example { const char* text="}"; /* { */ int value=0; };'
        self.assertEqual(capture_reuse.block(source,'struct Example {'),source[:-1])
    def measured_observation(self):
        frame=dict(start_ticks=0,ticks=240,cpu_cycles=338688,dropped_primitives=0,dropped_steps=0,
            actors=[0,0,0,0,0,5],resident_texture_bytes=10,active_texture_bytes=20,
            hud_dropped=0,max_components_per_actor=3)
        observed=dict(frames=[frame],step_observations=[dict(cpu_work_units=i*100)for i in range(1,241)])
        memory=dict(heap_peak_footprint_bytes=123456,stack_peak_bytes=65432)
        return observed,memory,dict(width=320,height=240)
    def test_setup_and_physical_tail_costs_count_without_changing_prefix_cpu(self):
        observed,memory,render=self.measured_observation()
        setup=copy.deepcopy(observed['frames'][0]);setup.update(start_ticks=240,ticks=300,cpu_cycles=1354752)
        tail=copy.deepcopy(setup);tail.update(start_ticks=300,ticks=900,cpu_cycles=1016064)
        observed['frames']+=[setup,tail]
        observed['step_observations'] += [dict(cpu_work_units=i*100)for i in range(241,901)]
        metrics,baseline=draft.measured_metrics(observed,memory,render)
        self.assertEqual(metrics['max_frame_us'],40000)
        self.assertEqual(metrics['slow_frames'],2)
        self.assertEqual(baseline['max_frame_us'],10000)
        self.assertEqual(baseline['slow_frames'],0)
        self.assertEqual(baseline['observed_frames'],1)
        self.assertEqual(metrics['simulation_steps'],240)
        self.assertEqual(metrics['cpu_work_units'],24000)
        self.assertEqual(baseline['cpu_work_units'],24000)
    def test_tail_resource_and_drop_peaks_cannot_be_hidden_by_prefix(self):
        observed,memory,render=self.measured_observation()
        tail=copy.deepcopy(observed['frames'][0]);tail.update(start_ticks=300,ticks=900,
            dropped_primitives=4,dropped_steps=2,actors=[0,0,7,0,0,63],
            resident_texture_bytes=900000,active_texture_bytes=950000,hud_dropped=9,max_components_per_actor=8)
        observed['frames'].append(tail)
        metrics,baseline=draft.measured_metrics(observed,memory,render)
        self.assertEqual(metrics['dropped_work'],6)
        self.assertEqual(metrics['rejected_work'],7)
        self.assertEqual(metrics['vram_peak_bytes'],320*240*4+950000)
        self.assertEqual(metrics['hud_drops'],9)
        self.assertEqual(metrics['max_runtime_actors'],63)
        self.assertEqual(metrics['max_components_per_actor'],8)
        self.assertEqual(metrics['heap_peak_bytes'],123456)
        self.assertEqual(metrics['stack_peak_bytes'],65432)
        self.assertEqual(baseline['observed_frames'],1)
    def test_missing_measured_frames_or_exact_step_cpu_cannot_auto_pass(self):
        observed,memory,render=self.measured_observation()
        observed['frames']=[]
        with self.assertRaisesRegex(ValueError,'Missing full measured'):draft.measured_metrics(observed,memory,render)
        observed,memory,render=self.measured_observation();observed['step_observations'].pop()
        with self.assertRaisesRegex(ValueError,'exact prefix CPU'):draft.measured_metrics(observed,memory,render)
    def test_default_route_metrics_remain_equal_and_interlace_vram_is_field_sized(self):
        observed,memory,render=self.measured_observation()
        metrics,baseline=draft.measured_metrics(observed,memory,render)
        for key in ['simulation_steps','cpu_work_units','slow_frames','max_frame_us']:
            self.assertEqual(metrics[key],baseline[key])
        render['height']=480
        metrics,_=draft.measured_metrics(observed,memory,render)
        self.assertEqual(metrics['vram_peak_bytes'],320*240*4+20)
    def test_changed_capture_dependency_or_selected_rule_invalidates_reuse(self):
        with tempfile.TemporaryDirectory(prefix='capture-contract-',dir=HERE) as directory:
            root=Path(directory);reference=root/'artifacts/gameplay-helpers/p7/player-routes/foundation/cpp'
            build=reference/'project/.epok/build';build.mkdir(parents=True);(root/'runtime').mkdir()
            current=root/'current';current_build=current/'.epok/build';current_build.mkdir(parents=True);(current_build/'epok.ps-exe').write_bytes(b'host-only-current-image')
            for name in ['input.hpp','gameplay_control.hpp','character_input.hpp','object_model.hpp']:
                data=(ROOT/'runtime'/name).read_bytes();(current_build/name).write_bytes(data);(build/name).write_bytes(data)
            old=reference/'project'
            for name in ['ThirdPersonController.hpp','ThirdPersonController.cpp']:
                for project in [old,current]:
                    file=project/'assets/scripts'/name;file.parent.mkdir(parents=True,exist_ok=True);file.write_text('host controller only',encoding='utf-8')
            adapter.write(reference/'result.json',[{'mode':'native_cpp','sample':[123,240,0,63,2,1,20,100],
              'editor_sha256':'a'*64,'exe_sha256':'b'*64}])
            binding={'template':'third-person','flavor':'cpp','project':current}
            self.assertEqual(capture_reuse.reconcile(root,binding)['status'],'reused-unaffected-capture-mechanism')
            object_source=(current_build/'object_model.hpp').read_text(encoding='utf-8')
            marker='inline const ClassDescriptor* find_object_class(uint64_t id) {'
            self.assertIn(marker,object_source)
            (current_build/'object_model.hpp').write_text(object_source.replace(marker,marker+'\n    (void)id; // changed metadata lookup implementation',1),encoding='utf-8')
            proof=capture_reuse.reconcile(root,binding)
            self.assertEqual(proof['status'],'missing')
            self.assertFalse(proof['source_identity']['object_model::inline const ClassDescriptor* find_object_class(']['identical'])
            self.assertTrue(any('find_object_class' in reason for reason in proof['missing']))
            (current_build/'object_model.hpp').write_bytes((build/'object_model.hpp').read_bytes())
            with (current_build/'gameplay_control.hpp').open('a',encoding='utf-8') as file:file.write('\n// changed dependency\n')
            self.assertEqual(capture_reuse.reconcile(root,binding)['status'],'missing')
            (current_build/'gameplay_control.hpp').write_bytes((build/'gameplay_control.hpp').read_bytes())
            (current/'assets/scripts/ThirdPersonController.cpp').write_text('changed selected rule',encoding='utf-8')
            self.assertEqual(capture_reuse.reconcile(root,binding)['status'],'missing')


if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(PhysicalTests))
    adapter.write(HERE/'host-physical-result.json',{'passed':result.wasSuccessful(),'tests_run':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),
      'scope':'Host models + actual composed Lua with PCSX mocks; no editor/MIPS/emulator acceptance',
      'sources':{n:adapter.sha(HERE/n) for n in ['player_routes.py','physical_routes.lua','physical_support.py','capture_reuse.py','test_physical_routes.py']}})
    raise SystemExit(not result.wasSuccessful())
