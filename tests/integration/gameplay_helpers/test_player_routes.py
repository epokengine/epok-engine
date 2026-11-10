"""Host-only player observer contracts; no target runtime acceptance."""
from pathlib import Path
import copy
import json
import unittest
import adapter
import player_routes as routes
from test_observer import LuaHost, MOCK, model_config

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'Cargo.toml').exists())


def input_config(template):
    value=model_config(template,'host-not-written');value.update(template=template,timeline=routes.timeline(template),steps=240,
      observation_boundary='player-next-tick-and-frame-return',tick_entry=0x80003004)
    def record(size,**members):return {'size':size,'members':members}
    value['symbols'].update({'epok::input':0x80008000,'epok::gameplay_controls()::value':0x80009000})
    value['layouts'].update(Input=record(96,ports=0),State=record(24,held=0,pending_pressed=2,pending_released=4,pressed=6,released=8,
      frame_pressed=10,frame_released=12,connected=14,analog=15,axes=16),GameplayControls=record(12,m_edges=0),
      Edges=record(12,frame_pressed=0,frame_released=2,pending_pressed=4,pending_released=6,tick_pressed=8,tick_released=10))
    return value


def binding(template):
    return {'template':template,'executable':HERE/'player_observe.lua','source_inventory':{'scope':'synthetic-host-only'}}


def observation(template):
    bound=binding(template);trace=[];frames=[];steps=[]
    for n in range(1,241):
        row={'step':n,'position':[n,0,n] if template=='third-person' else [n,0],
          'motion_status':0,'animation_status':0,'animation_state':n%6 if template=='third-person' else n%2,
          'camera_valid':True,'camera_position':[n+1,2,3] if template=='third-person' else [n,1],
          'jumped':n in (26,126),'grounded':n not in range(26,80),'camera_yaw':n*10}
        if template=='platformer-2d':row.update(coins_active=[True]*7+[n<20],lift_position=[n,1],hit_tile=True)
        trace.append(row);frames.append({'cpu_cycles':100,'observer_frame':n,'frame_sequence':n,'start_ticks':n-1,'ticks':n,'executed_steps':1,'requested_steps':1})
        steps.append({'step':n,'boundary':'frame-return','frame':n,'cpu_work_units':100*n,'clock_read':n})
    return dict(reason='exact-steps',errors=[],executable_sha256=adapter.sha(bound['executable']),source_inventory_sha256=adapter.digest(bound['source_inventory']),
      trace=trace,input_trace=routes.timeline(template),frames=frames,step_observations=steps,cpu_work_units=24000,
      clock_observer={'kind':'player-next-tick-and-frame-return','write_callbacks':240,'frames':240},input_writes={'count':4560})


class PlayerRouteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.lua=LuaHost()

    def test_all_six_actual_project_role_bindings_are_valid_without_changes(self):
        for template in routes.TEMPLATES:
            for flavor in ['cpp','blueprint','lua']:
                project=ROOT/'artifacts/gameplay-helpers/p10/final18/projects'/(template+'--'+flavor)
                installed=adapter.documents(project/'ProjectSettings/Template.epoksettings');manifest=adapter.documents(next(project.glob('*.epokproject')))
                scene=adapter.documents(project/manifest['startup_scene']);before=adapter.inventory(project)
                value=routes.roles({'template':template,'installation':installed,'scene':scene})
                self.assertTrue(value['player_name']);self.assertEqual(before,adapter.inventory(project))
                if template=='platformer-2d':self.assertEqual(len(value['coin_names']),8)

    def test_exact_p7_button_and_orbit_boundaries_are_shared_by_providers(self):
        third=routes.timeline('third-person');platform=routes.timeline('platformer-2d')
        self.assertEqual(len(third),240);self.assertEqual(len(platform),240)
        self.assertEqual([r['step'] for r in third if r['buttons']&(1<<14)],[26,126])
        self.assertEqual([r['step'] for r in platform if r['buttons']&(1<<14)],[61,131])
        self.assertEqual([r['step'] for r in third if r['adc'][2]==200],list(range(41,61)))
        self.assertEqual(platform[9]['buttons'],0);self.assertEqual(platform[10]['buttons'],1<<7)
        self.assertEqual(platform[44]['buttons'],1<<7);self.assertEqual(platform[45]['buttons'],1<<5)

    def test_actual_composed_lua_injects_only_whitelisted_device_state(self):
        for template in routes.TEMPLATES:
            config=input_config(template);source=MOCK+'\n'+routes.observer_source(config)
            source+='''
local old=0
for step=1,240 do
 local value=config.timeline[step];inject_input(step)
 local held=tonumber(ffi.cast('uint16_t*',ram+0x8000)[0])
 local pressed=tonumber(ffi.cast('uint16_t*',ram+0x8006)[0])
 assert(held==value.buttons and pressed==bit.band(value.buttons,bit.bnot(old)),'Device edge replay differs')
 assert(tonumber(ffi.cast('uint16_t*',ram+0x9008)[0])==pressed,'Gameplay input tick edge differs')
 local axis=tonumber(ffi.cast('int16_t*',ram+0x8014)[0])
 assert(axis==(value.adc[3]==200 and 2322 or 0),'ADC conversion differs from target Input::sample')
 old=value.buttons
end
assert(#input_trace==240 and clock_callbacks==240 and input_writes.count==4560,'Input observations incomplete')
assert(not pcall(function()device_write(0x80001000,1,1)end),'World write was accepted')
assert(ffi.string(ram,0x8000)==string.rep(string.char(0),0x8000),'World RAM changed before input region')
assert(ffi.string(ram+0x8018,0x9000-0x8018)==string.rep(string.char(0),0x9000-0x8018),'World RAM changed between input regions')
assert(ffi.string(ram+0x900c,2097152-0x900c)==string.rep(string.char(0),2097152-0x900c),'World RAM changed after input region')
'''
            self.lua.run(source)

    def test_complete_host_models_pass_functional_contract(self):
        for template in routes.TEMPLATES:self.assertTrue(routes.validate_observation(observation(template),binding(template))['jump'])

    def test_missing_timeline_step_or_different_input_fails(self):
        value=observation('third-person');value['trace'].pop(50)
        with self.assertRaisesRegex(ValueError,'Missing actual steps'):routes.validate_observation(value,binding('third-person'))
        value=observation('third-person');value['input_trace'][20]['buttons']=0
        with self.assertRaisesRegex(ValueError,'different input timeline'):routes.validate_observation(value,binding('third-person'))

    def test_no_cpu_or_missing_clock_is_not_auto_green(self):
        value=observation('third-person');value['frames'][5]['cpu_cycles']=0
        with self.assertRaisesRegex(ValueError,'CPU work unmeasured'):routes.validate_observation(value,binding('third-person'))
        value=observation('third-person');value['step_observations'][5]['clock_read']=999
        with self.assertRaisesRegex(ValueError,'actual Time read'):routes.validate_observation(value,binding('third-person'))

    def test_frozen_camera_or_absent_jump_fails(self):
        value=observation('third-person')
        for row in value['trace']:row['camera_position']=[0,0,0]
        with self.assertRaisesRegex(ValueError,'camera did not move'):routes.validate_observation(value,binding('third-person'))
        value=observation('third-person')
        for row in value['trace']:row['jumped']=False
        with self.assertRaisesRegex(ValueError,'movement/jump'):routes.validate_observation(value,binding('third-person'))

    def test_missing_real_coin_or_static_platform_fails(self):
        value=observation('platformer-2d')
        for row in value['trace']:row['coins_active']=[True]*8
        with self.assertRaisesRegex(ValueError,'coin callback'):routes.validate_observation(value,binding('platformer-2d'))
        value=observation('platformer-2d')
        for row in value['trace']:row['lift_position']=[0,0]
        with self.assertRaisesRegex(ValueError,'platform/tile'):routes.validate_observation(value,binding('platformer-2d'))

    def test_bitfield_requires_real_compiled_position(self):
        debug=''' <1><1>: Abbrev Number:1 (DW_TAG_class_type)
 <2> DW_AT_name : (string) Actor
 <2><3>: Abbrev Number:2 (DW_TAG_member)
 <4> DW_AT_name : (string) m_active
 <5> DW_AT_bit_size : (data1) 1
 <6> DW_AT_data_bit_offset : (data2) 931
'''
        self.assertEqual(routes.actor_active_bit(debug),{'byte':116,'bit':3})
        with self.assertRaisesRegex(ValueError,'bitfield'):routes.actor_active_bit(debug.replace('data_bit_offset','unknown_offset'))


if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(PlayerRouteTests))
    adapter.write(HERE/'player-route-host-result.json',{'passed':result.wasSuccessful(),'tests_run':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),
      'scope':'Host contracts and actual LuaJIT input replay with PCSX mocks, plus actual six-project source binding reads; NOT current target runtime acceptance',
      'sources':{name:adapter.sha(HERE/name) for name in ['player_routes.py','player_observe.lua','test_player_routes.py']}})
    raise SystemExit(not result.wasSuccessful())
