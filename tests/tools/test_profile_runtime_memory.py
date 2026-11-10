import importlib.util
import ctypes
import os
from pathlib import Path
import struct
import sys
import unittest

MODULE = Path(__file__).resolve().parents[2] / "tools/profile_runtime_memory.py"
spec = importlib.util.spec_from_file_location("profile_runtime_memory", MODULE)
memory = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = memory
spec.loader.exec_module(memory)


class HeapTests(unittest.TestCase):
    def setUp(self):
        self.ram = bytearray(0x200000)
        self.meta, self.bottom, self.top = 0x2000, 0x10000, 0x1ffff8
        self.marker = self.meta + 16

    def word(self, offset, value):
        struct.pack_into("<I", self.ram, offset, value)

    def setup_heap(self, live, old_max=None):
        for i, value in enumerate((self.bottom + live, self.bottom, self.top, self.bottom + (live if old_max is None else old_max))):
            self.word(self.meta + 4 * i, value | 0x80000000)
        self.word(self.bottom + live, self.marker | 0x80000000)
        self.word(self.bottom + live + 4, self.top - self.bottom - live)

    def inspect(self):
        return memory.heap_snapshot(self.ram, self.meta, self.bottom)

    def test_lazy_heap(self):
        self.assertFalse(self.inspect()["initialized"])

    def test_growth_and_free_header_are_measured_after_in_place_realloc(self):
        self.setup_heap(40, old_max=16)
        value = self.inspect()
        self.assertEqual((value["live_bytes"], value["footprint_bytes"], value["sdk_maximum_bytes"]), (40, 48, 16))

    def test_free_holes_do_not_count_as_live_allocations(self):
        self.setup_heap(56)
        self.word(self.meta, self.bottom | 0x80000000)
        self.word(self.bottom, (self.bottom + 56) | 0x80000000)
        self.word(self.bottom + 4, 24)
        value = self.inspect()
        self.assertEqual((value["live_bytes"], value["footprint_bytes"], value["free_blocks"]), (32, 64, 2))

    def test_fully_allocated_and_fully_free(self):
        self.setup_heap(0)
        self.assertEqual(self.inspect()["footprint_bytes"], 8)
        self.word(self.meta, self.marker | 0x80000000)
        self.assertEqual(self.inspect()["live_bytes"], self.top - self.bottom)

    def test_corrupt_links_sizes_and_addresses_fail_closed(self):
        for offset, value in ((self.bottom, self.bottom), (self.bottom + 4, 7), (self.meta, 0x80210000)):
            self.setUp()
            self.setup_heap(0)
            self.word(offset, value)
            with self.assertRaises(ValueError):
                self.inspect()


class StackTests(unittest.TestCase):
    def test_actual_mips_prologue_and_dynamic_frame(self):
        registers = [0] * 32
        registers[29], registers[2] = 0x801fff00, 4096
        for word, expected in ((0x27bdffe0, 0x801ffee0), (0x03a2e823, 0x801fef00)):
            write = memory.decode_stack_write(0x80010000, word)
            self.assertEqual(memory.predicted_stack_pointer(write, registers, None), expected)

    def test_restoring_stack_from_memory(self):
        registers = [0] * 32
        registers[2] = 0x80020000
        write = memory.decode_stack_write(0x80010000, 0x8c5d0000)
        def read(address, count):
            self.assertEqual((address, count), (0x80020000, 4))
            return struct.pack("<I", 0x801ff000)
        self.assertEqual(memory.predicted_stack_pointer(write, registers, read), 0x801ff000)

    def test_objdump_scan_and_unknown_writer(self):
        rows = memory.linked_stack_writes("80010000: 27bdffe0 addiu sp,sp,-32\n80010004: afbf001c sw ra,28(sp)\n80010008: 03a2e823 subu sp,sp,v0\n")
        self.assertEqual(len(rows), 2)
        with self.assertRaises(ValueError):
            memory.decode_stack_write(0x80010000, (29 << 11) | 0x2a)
        for op in (10, 11, 34, 38):
            with self.assertRaises(ValueError):
                memory.decode_stack_write(0x80010000, (op << 26) | (29 << 16))


@unittest.skipUnless(os.name == "nt" and (MODULE.parents[1] / ".tools/redux/lua51.dll").exists(),
                     "The pinned Windows LuaJIT DLL is required")
class LuaObserverTests(unittest.TestCase):
    """Exercise the actual observer in LuaJIT with a deterministic memory API."""
    def run_lua(self, body):
        directory = MODULE.parents[1] / ".tools/redux"
        with os.add_dll_directory(str(directory)):
            lua = ctypes.CDLL(str(directory / "lua51.dll"))
            lua.luaL_newstate.restype = ctypes.c_void_p
            lua.luaL_openlibs.argtypes = [ctypes.c_void_p]
            lua.luaL_loadstring.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
            lua.lua_pcall.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int]
            lua.lua_tolstring.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p]
            lua.lua_tolstring.restype = ctypes.c_char_p
            lua.lua_close.argtypes = [ctypes.c_void_p]
            state = lua.luaL_newstate()
            try:
                lua.luaL_openlibs(state)
                status = lua.luaL_loadstring(state, body.encode())
                if status == 0:
                    status = lua.lua_pcall(state, 0, 1, 0)
                result = lua.lua_tolstring(state, -1, None)
                self.assertEqual(status, 0, result.decode() if result else "Lua error")
                return memory.json.loads(result)
            finally:
                lua.lua_close(state)

    def setup_lua(self, writes=()):
        source = r'''
local ffi=require('ffi')
local ram=ffi.new('uint8_t[2097152]')
local words=ffi.cast('uint32_t*',ram)
local regs={GPR={n={sp=0x801fff00,ra=0x80018000,v0=4096}}}
local hooks={}
local paused=false
local shared_invoker=nil
PCSX={getMemPtr=function()return ram end,getRegisters=function()return regs end,
 pauseEmulator=function()paused=true end}
local native={addBreakpoint=function(address,kind,size,label,invoker)
 assert(not hooks[address],'Duplicate breakpoint in fixture')
 assert(type(invoker)=='cdata','The native adapter needs one shared FFI callback')
 assert(shared_invoker==nil or shared_invoker==invoker,'The observer must reuse the same callback')
 shared_invoker=invoker
 hooks[address]=function()return invoker(address,size,label)end
 return address
end}
local original_load=ffi.load
ffi.load=function(name,...)if name=='PCSX' then return native end;return original_load(name,...)end
local function hit(address)assert(hooks[address],'Missing hook')()end
local function heap(live,old_max)
 words[0x2000/4]=0x80010000+live;words[0x2004/4]=0x80010000
 words[0x2008/4]=0x801ffff8;words[0x200c/4]=0x80010000+(old_max or live)
 words[(0x10000+live)/4]=0x80002010;words[(0x10004+live)/4]=0x1efff8-live
end
'''
        config = {"start": 0x80010000, "heap_start": 0x10000, "ram_top": 0x200000,
                  "image_start": 0x10000, "metadata": 0x2000, "allocators": [0x80011000], "stack_write_sites": len(writes)}
        return source + memory.lua_probe(config, writes)

    def test_transient_heap_growth_and_stack_canary(self):
        write = memory.decode_stack_write(0x80012000, 0x27bdf000)  # addiu sp,sp,-4096
        result = self.run_lua(self.setup_lua([write]) + r'''
hit(0x80010000);hit(0x80011000);heap(24);hit(0x80018000)
hit(0x80011000);heap(64,24);hit(0x80018000)
hit(0x80011000);heap(0,24);hit(0x80018000)
hit(0x80012000)
words[(0x1fff00-8192)/4]=0x12345678
EPOK_MEMORY_OBSERVER.sample()
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertEqual(result["error"], "")
        self.assertEqual(result["heap_peak_live_bytes"], 64)
        self.assertEqual(result["heap_peak_footprint_bytes"], 72)
        self.assertEqual(result["sdk_maximum_bytes"], 24)
        self.assertEqual(result["stack_min_sp"], 0x1fff00 - 4096)
        self.assertEqual(result["stack_touched_min"], 0x1fff00 - 8192)
        self.assertEqual(result["stack_peak_bytes"], 8192 + 256)
        self.assertEqual(result["minimum_gap_bytes"], 0x1fff00 - 8192 - 0x10000 - 72)

    def test_corrupt_heap_is_reported_and_http_does_not_sample_mid_mutation(self):
        result = self.run_lua(self.setup_lua() + r'''
hit(0x80010000);heap(0)
EPOK_MEMORY_OBSERVER.sample()
words[0x10004/4]=7
local before=PCSX.WebServer.Handlers['memory-observer']()
assert(before:find('"error":""',1,true),'HTTP sampled an incoherent allocator')
hit(0x80011000)
assert(paused,'Corruption must pause the guest')
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertIn("Invalid allocator free-block size", result["error"])

    def test_dynamic_stack_write_uses_live_source_register(self):
        write = memory.decode_stack_write(0x80012000, 0x03a2e823)
        result = self.run_lua(self.setup_lua([write]) + '''
hit(0x80010000);hit(0x80012000)
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertEqual(result["error"], "")
        self.assertEqual(result["stack_peak_bytes"], 4352)

    def test_frame_pointer_name_matches_the_pinned_register_api(self):
        write = memory.decode_stack_write(0x80012000, (30 << 21) | (29 << 11) | 0x21)
        result = self.run_lua(self.setup_lua([write]) + '''
regs.GPR.n.s8=0x801ff000
hit(0x80010000);hit(0x80012000)
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertEqual(result["error"], "")
        self.assertEqual(result["stack_peak_bytes"], 4096)

    def test_large_executable_uses_one_callback_and_composes_same_pc_hooks(self):
        writes = [memory.decode_stack_write(0x80011000 + 4 * i, 0x27bdffe0) for i in range(1600)]
        result = self.run_lua(self.setup_lua(writes) + '''
hit(0x80010000);hit(0x80011000)
heap(24);hit(0x80018000)
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertEqual(result["error"], "")
        self.assertEqual(result["stack_callbacks"], 1)
        self.assertEqual(result["stack_peak_bytes"], 288)
        self.assertEqual(result["heap_peak_live_bytes"], 24)

    def test_bios_low_ram_stack_is_separate_from_application_stack(self):
        write = memory.decode_stack_write(0x80012000, 0x27bdffd0)
        result = self.run_lua(self.setup_lua([write]) + '''
hit(0x80010000);regs.GPR.n.sp=0x83ec;hit(0x80012000)
return EPOK_MEMORY_OBSERVER.json()
''')
        self.assertEqual(result["error"], "")
        self.assertEqual(result["low_ram_stack_callbacks"], 1)
        self.assertEqual(result["low_ram_stack_min"], 0x83bc)
        self.assertEqual(result["stack_peak_bytes"], 256)


if __name__ == "__main__":
    unittest.main()
