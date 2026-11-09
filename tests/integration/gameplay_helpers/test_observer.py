"""Run the actual external Lua observer on host LuaJIT with read-only PCSX mocks.

These models exercise the capture contract, not a target runtime acceptance.
"""
from pathlib import Path
import ctypes
import hashlib
import json
import tempfile
import unittest
from unittest.mock import patch
import adapter
import layout

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'Cargo.toml').exists())


class LuaHost:
    def __init__(self):
        self.dll=ctypes.CDLL(str(ROOT/'.tools/redux/lua51.dll'))
        for name,args,result in [
            ('luaL_newstate',[],ctypes.c_void_p),('luaL_openlibs',[ctypes.c_void_p],None),
            ('luaL_loadstring',[ctypes.c_void_p,ctypes.c_char_p],ctypes.c_int),
            ('lua_pcall',[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int],ctypes.c_int),
            ('lua_tolstring',[ctypes.c_void_p,ctypes.c_int,ctypes.c_void_p],ctypes.c_char_p),
            ('lua_close',[ctypes.c_void_p],None)]:
            function=getattr(self.dll,name);function.argtypes=args;function.restype=result

    def run(self,source):
        state=self.dll.luaL_newstate()
        try:
            self.dll.luaL_openlibs(state)
            code=self.dll.luaL_loadstring(state,source.encode('utf-8'))
            if not code:code=self.dll.lua_pcall(state,0,0,0)
            if code:raise AssertionError(self.dll.lua_tolstring(state,-1,None).decode('utf-8'))
        finally:self.dll.lua_close(state)


MOCK=r'''
local ffi=require('ffi');local bit=require('bit')
local ram=ffi.new('uint8_t[2097152]');local cycles=0;local quit_code=nil;local hooks={}
local registers={GPR={n={ra=0}}}
local function physical(p)return bit.band(p,0x1fffff)end
local function put32(p,v)ffi.cast('uint32_t*',ram+physical(p))[0]=v end
local function get32(p)return tonumber(ffi.cast('uint32_t*',ram+physical(p))[0])end
local function put8(p,v)ram[physical(p)]=v end
PCSX={}
function PCSX.getMemPtr()return ram end
function PCSX.getRegisters()return registers end
function PCSX.getCPUCycles()return cycles end
function PCSX.quit(code)quit_code=code end
function PCSX.addBreakpoint(address,kind,width,cause,callback)
 local key=tostring(address)..':'..kind;assert(not hooks[key],'Duplicate host mock hook')
 local handle={callback=callback,width=width};hooks[key]=handle;return handle
end
local function invoke(address,kind,...)
 local hook=assert(hooks[tostring(address)..':'..kind],'Required host mock hook missing')
 local before=ffi.string(ram,0x20000)
 hook.callback(...)
 assert(ffi.string(ram,0x20000)==before,'External observer modified target model RAM')
end
'''

DRIVE=r'''
local c=TEST_CONFIG
put32(c.symbols['epok::object_count'],1)
local object=c.symbols['epok::objects'];ffi.copy(ram+physical(object),c.actor_name,#c.actor_name+1)
local owner=TEST_OPTIONS.owner_alias and 0xa0000700 or 0x80000700;put32(object+72,owner);put8(object+76,1);put8(owner,1)
local access=TEST_OPTIONS.watch_alias and 0xa0000200 or c.tick_counter_address
if TEST_OPTIONS.wrong_address then access=access+4 end
for n=0,2 do put32(object+32+24+n*4,4096)end
local batches=TEST_OPTIONS.single_step and {1} or {0,1,3,0,8,2}
local frame=0;local total=0;local raw_delta_sum=0;local delta_remainder=0
while total<c.steps and not quit_code do
 frame=frame+1;local count=math.min(batches[(frame-1)%#batches+1],c.steps-total)
 registers.GPR.n.ra=0x80002000;cycles=cycles+100
 invoke(c.frame_entry,'Exec')
 for n=1,count do
  if TEST_OPTIONS.skip_intermediate and n<count then
   total=total+1;put32(c.tick_counter_address,total);cycles=cycles+23
  else
   if c.template=='basic' and not TEST_OPTIONS.no_time_write and not TEST_OPTIONS.post_store then
    invoke(c.tick_counter_address,'Write',access,4)
   end
   total=total+1;put32(c.tick_counter_address,total);cycles=cycles+23
   if c.template=='basic' and not TEST_OPTIONS.no_time_write and TEST_OPTIONS.post_store then
    invoke(c.tick_counter_address,'Write',access,4)
   end
  end
  if c.template=='sample' then
   registers.GPR.n.ra=0x80002004;invoke(c.tick_entry,'Exec')
   -- Independent model of the unchanged selected Spinner's real fixed update.
   local delta=68;delta_remainder=delta_remainder+16
   if delta_remainder>=60 then delta=delta+1;delta_remainder=delta_remainder-60 end
   raw_delta_sum=raw_delta_sum+delta
   put32(object+32+12+4,45*raw_delta_sum)
   cycles=cycles+17;invoke(0x80002004,'Exec')
  end
 end
 put32(c.symbols['epok::performance_stats'],TEST_OPTIONS.repeat_frame and 1 or frame)
 put32(c.symbols['epok::performance_stats']+4,count)
 put32(c.symbols['epok::time']+8,count*16667)
 cycles=cycles+40;invoke(0x80002000,'Exec')
end
assert(quit_code~=nil,'Host model never completed observation')
'''


def model_config(template,output):
    symbols={'epok::objects':0x80001000,'epok::object_count':0x80000100,'epok::time':0x80000200,
             'epok::performance_stats':0x80000300,'epok::actor_stats':0x80000400,
             'epok::resource_usage':0x80000500,'epok::hud_stats':0x80000600}
    def record(size,**members):return {'size':size,'members':members}
    return dict(symbols=symbols,layouts={'ActorData':record(128,name=0,transform=32,owner=72,alive=76,audio=77),
        'Transform':record(36,position=0,rotation=12,scale=24),'Actor':record(4,m_component_count=0),
        'Time':record(16,ticks=0,dropped_steps=4,frame_microseconds=8),'ResourceUsage':record(12,resident_texture_bytes=0,active_texture_bytes=4,dropped_primitives=8),
        'HudStats':record(4,dropped=0),'AudioSource':record(1,enabled=0),'PerformanceStats':record(8,frame=0,steps=4)},
        slot_capacity=1,frame_entry=0x80003000,tick_entry=0x80003004 if template=='sample' else None,
        tick_counter_address=0x80000200,observation_boundary='time-write-and-frame-return' if template=='basic' else 'selected-actor-tick-return',
        template=template,actor_name='Actual Host Model Root',steps=120,output=output,diagnostic_memory=False,
        source_inventory_sha256='model-only',executable_sha256='model-only',performance_words=2,hud_words=1)


class ObserverTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.lua=LuaHost()

    def run_model(self,template='basic',**options):
        with tempfile.TemporaryDirectory(prefix='observer-host-',dir=HERE) as folder:
            output=Path(folder)/'observation.json';config=model_config(template,str(output).replace('\\','/'))
            source=MOCK+'\nTEST_CONFIG='+adapter.lua_literal(config)+'\nTEST_OPTIONS='+adapter.lua_literal(options)+'\n'
            source+='do\n'+(HERE/'observe.lua').read_text(encoding='utf-8').replace('@CONFIG@',adapter.lua_literal(config))+'\nend\n'+DRIVE
            self.lua.run(source)
            return json.loads(output.read_text(encoding='utf-8'))

    def assert_valid_model(self,value,template):
        binding={'template':template,'executable':HERE/'observe.lua','source_inventory':{'mock':'host-only'}}
        value['executable_sha256']=adapter.sha(binding['executable'])
        value['source_inventory_sha256']=adapter.digest(binding['source_inventory'])
        adapter.validate_observation(value,binding,120,False)

    def test_basic_pre_store_watch_has_all_120_actual_rows_including_catchup(self):
        value=self.run_model();self.assert_valid_model(value,'basic')
        self.assertEqual(value['clock_observer']['write_callbacks'],120)
        self.assertIn('time-write',{r['boundary'] for r in value['step_observations']})
        self.assertIn('frame-return',{r['boundary'] for r in value['step_observations']})

    def test_basic_post_store_watch_has_same_semantic_trace(self):
        before=self.run_model();after=self.run_model(post_store=True)
        self.assert_valid_model(after,'basic');self.assertEqual(before['trace'],after['trace'])

    def test_basic_one_step_frames_still_require_actual_time_watch(self):
        value=self.run_model(single_step=True);self.assert_valid_model(value,'basic')
        self.assertEqual(len(value['frames']),120)
        self.assertEqual(value['step_observations'][-1]['boundary'],'frame-return')

    def test_basic_no_time_write_cannot_auto_green_even_with_120_real_frames(self):
        value=self.run_model(no_time_write=True,single_step=True)
        self.assertEqual(len(value['trace']),120)
        with self.assertRaisesRegex(ValueError,'Time write observation never triggered'):self.assert_valid_model(value,'basic')

    def test_basic_only_frame_final_read_cannot_synthesize_catchup_rows(self):
        value=self.run_model(no_time_write=True)
        self.assertEqual(value['reason'],'observer-error')
        self.assertTrue(any('boundary skipped' in error for error in value['errors']))
        with self.assertRaisesRegex(ValueError,'Incomplete/error'):self.assert_valid_model(value,'basic')

    def test_uncached_owner_and_access_alias_read_the_same_actual_ram(self):
        normal=self.run_model();aliased=self.run_model(owner_alias=True,watch_alias=True)
        self.assert_valid_model(aliased,'basic');self.assertEqual(normal['trace'],aliased['trace'])

    def test_different_clock_member_cannot_trigger_a_valid_tick(self):
        value=self.run_model(wrong_address=True)
        self.assertEqual(value['reason'],'observer-error')
        self.assertTrue(any('Unexpected Time counter access' in error for error in value['errors']))

    def test_sample_observes_selected_spinner_and_retains_exact_trace(self):
        value=self.run_model('sample');self.assert_valid_model(value,'sample')
        self.assertEqual({r['boundary'] for r in value['step_observations']},{'tick-return'})
        self.assertEqual(value['trace'][-1]['rotation'],[0,45*8192,0])

    def test_repeated_published_frame_is_a_real_observer_error(self):
        value=self.run_model(repeat_frame=True)
        self.assertEqual(value['reason'],'observer-error')
        self.assertTrue(any('sequence missing or repeated' in error for error in value['errors']))

    def test_basic_layout_does_not_require_a_tick_actor_symbol(self):
        syms={name:{'address':address,'size':32} for name,address in model_config('basic','unused')['symbols'].items()}
        syms['epok::objects']['size']=128
        syms['GameScene::frame()']={'address':0x80003000,'size':256}
        structures={name:[record] for name,record in model_config('basic','unused')['layouts'].items()}
        with patch.object(layout,'symbols',return_value=syms),patch.object(layout,'tool',return_value=''),patch.object(layout,'dwarf_structures',return_value=structures):
            value=layout.inspect(ROOT,HERE/'not-an-executable','basic')
            self.assertIsNone(value['tick_entry']);self.assertEqual(value['tick_counter_address'],0x80000200)
            with self.assertRaisesRegex(ValueError,'Spinner simulation boundary'):layout.inspect(ROOT,HERE/'not-an-executable','sample')
            syms['epok::Level::tick_actor(one)']={'address':0x80004000,'size':256}
            syms['epok::Level::tick_actor(two)']={'address':0x80005000,'size':256}
            self.assertIsNone(layout.inspect(ROOT,HERE/'not-an-executable','basic')['tick_entry'])
            with self.assertRaisesRegex(ValueError,'Spinner simulation boundary'):layout.inspect(ROOT,HERE/'not-an-executable','sample')

    def test_layout_cannot_invent_missing_clock_or_frame(self):
        with patch.object(layout,'symbols',return_value={}):
            with self.assertRaisesRegex(ValueError,'GameScene frame boundary'):layout.inspect(ROOT,HERE/'missing','basic')


if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(ObserverTests))
    (HERE/'basic-sample-host-observer-result.json').write_text(json.dumps({
        'passed':result.wasSuccessful(),'tests_run':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),
        'scope':'Host LuaJIT executing actual observer with PCSX/RAM mocks; not actual emulator/runtime acceptance',
        'target_processes_started':0,'target_instructions_added':0,'target_bss_added':0,
        'sources':{name:adapter.sha(HERE/name) for name in ['adapter.py','layout.py','observe.lua','test_adapter.py','test_observer.py']}
    },indent=2)+'\n',encoding='utf-8')
    raise SystemExit(not result.wasSuccessful())
