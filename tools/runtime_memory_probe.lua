-- External debugger observer for the pinned PsyQo MIPS32 allocator.
-- The only guest write is a canary in unused RAM at executable entry.
local ffi=require('ffi')
local mem=PCSX.getMemPtr()
local regs=PCSX.getRegisters()
local watch={}
local callbacks={}
local returns={}
local started=false
local error_message=''
local stack_min=config.ram_top
local touched_min=config.ram_top
local initial_sp=0
local heap_high=config.heap_start
local heap_live_peak=0
local heap_live=0
local heap_footprint=0
local sdk_high=0
local stack_calls=0
local scratchpad_calls=0
local low_ram_stack_calls=0
local low_ram_stack_min=config.image_start
local low_ram_stack_max=0
local heap_samples=0
local scans=0
local minimum_gap=config.ram_top-config.heap_start
local register_names={'r0','at','v0','v1','a0','a1','a2','a3','t0','t1','t2','t3','t4','t5','t6','t7','s0','s1','s2','s3','s4','s5','s6','s7','t8','t9','k0','k1','gp','sp','s8','ra'}
local function physical(value)return bit.band(value,0x1fffffff)end
local function unsigned(value)return value%4294967296 end
local function word(address)
 assert(address>=0 and address+4<=config.ram_top,'Memory read outside main RAM')
 return tonumber(ffi.cast('uint32_t*',mem+address)[0])
end
local function register(index)
 if index==0 then return 0 end
 return tonumber(regs.GPR.n[register_names[index+1]])
end
local function observe_sp(value,pc)
 local address=physical(value)
 if address>=0x1f800000 and address<=0x1f800400 then scratchpad_calls=scratchpad_calls+1;return end
 -- BIOS callbacks can enter linked functions using the BIOS-owned low-RAM
 -- stack (observed near 0x83ec). It is outside the application heap/stack gap.
 if address>=4 and address<config.image_start then
  low_ram_stack_calls=low_ram_stack_calls+1
  low_ram_stack_min=math.min(low_ram_stack_min,address)
  low_ram_stack_max=math.max(low_ram_stack_max,address)
  return
 end
 assert(address>=config.heap_start and address<=config.ram_top,string.format('Stack outside unused main RAM: SP=%08x PC=%08x previous=%08x',value,pc or 0,register(29)))
 stack_min=math.min(stack_min,address)
 minimum_gap=math.min(minimum_gap,stack_min-heap_high)
 assert(minimum_gap>=0,'Observed heap/stack overlap')
end
local function snapshot_heap()
 if not started then return end
 local meta=config.metadata
 local head,bottom,top,maximum=word(meta),word(meta+4),word(meta+8),word(meta+12)
 heap_samples=heap_samples+1
 if head==0 then assert(bottom==0 and top==0 and maximum==0,'Inconsistent uninitialized allocator');return end
 bottom,top=physical(bottom),physical(top)
 assert(bottom>=config.heap_start and bottom%8==0 and top%8==0 and bottom<top and top<=config.ram_top,'Invalid allocator extent')
 local cursor,free,blocks,allocated_end=bottom,0,0,bottom
 local footprint_end=bottom+8
 local current=physical(head)
 while current~=meta+16 do
  blocks=blocks+1
  assert(blocks<=(top-bottom)/8,'Allocator free-list cycle')
  assert(current>=cursor and current%8==0 and current+8<=top,'Unordered or out-of-range allocator free block')
  local next_block,size=word(current),word(current+4)
  assert(size>=8 and size%8==0 and current+size<=top,'Invalid allocator free-block size')
  if cursor<current then allocated_end=current end
  footprint_end=math.max(footprint_end,current+8)
  free=free+size;cursor=current+size;current=physical(next_block)
 end
 if cursor<top then allocated_end=top end
 maximum=maximum~=0 and physical(maximum) or bottom
 assert(maximum>=bottom and maximum<=top,'Invalid allocator maximum')
 sdk_high=math.max(sdk_high,maximum-bottom)
 heap_high=math.max(heap_high,allocated_end,footprint_end)
 heap_live=top-bottom-free
 heap_footprint=math.max(allocated_end,footprint_end)-bottom
 heap_live_peak=math.max(heap_live_peak,heap_live)
 minimum_gap=math.min(minimum_gap,math.min(stack_min,touched_min)-heap_high)
 assert(minimum_gap>=0,'Observed heap/stack overlap')
end
local function guarded(callback)
 return function()
  if error_message~='' then return true end
  local ok,message=pcall(callback)
  if not ok then error_message=tostring(message);PCSX.pauseEmulator() end
  return true
 end
end
-- The pinned pcsxffi.lua wrapper casts a fresh C callback per breakpoint.
-- Its native FFI library exposes the same operation with a reusable invoker;
-- sharing it avoids exhausting LuaJIT's callback slots on a large executable.
local native=ffi.load('PCSX')
local shared_callback=ffi.cast('bool (*)(uint32_t, unsigned, const char*)',function(address)
 local list=callbacks[physical(tonumber(address))]
 if not list then error_message='Unknown memory observer breakpoint';PCSX.pauseEmulator();return true end
 for _,callback in ipairs(list) do callback() end
 return true
end)
local function add_watch(address,label,callback)
 local key=physical(address)
 if not callbacks[key] then
  callbacks[key]={}
  watch[#watch+1]=native.addBreakpoint(address,0,4,label,shared_callback,label)
 end
 callbacks[key][#callbacks[key]+1]=callback
 return key
end
local function scan_stack()
 if not started then return end
 snapshot_heap()
 -- Heap blocks and their free-list headers must never be classified as stack.
 local offset=math.ceil(heap_high/4)*4
 while offset+4<=touched_min do
  if word(offset)~=0xa5a5a5a5 then touched_min=offset;break end
  offset=offset+4
 end
 scans=scans+1
 minimum_gap=math.min(minimum_gap,math.min(stack_min,touched_min)-heap_high)
 assert(minimum_gap>=0,'Observed heap/stack overlap')
end
add_watch(config.start,'Initialize memory observer',guarded(function()
 assert(not started,'Executable restarted during memory capture')
 initial_sp=physical(register(29))
 assert(initial_sp>config.heap_start and initial_sp<=config.ram_top and initial_sp%4==0,'Invalid initial SP')
 ffi.fill(mem+config.heap_start,initial_sp-config.heap_start,0xa5)
 stack_min=initial_sp;touched_min=initial_sp;started=true
end))
for _,site in ipairs(config.writes) do
 local pc,kind,rs,rt,n=unpack(site)
 add_watch(pc,'Linked stack write',guarded(function()
  if not started then return end
  local a,b,result=register(rs),register(rt),0
  if kind=='add' then result=a+b
  elseif kind=='addi' then result=a+n
  elseif kind=='sub' then result=a-b
  elseif kind=='and' or kind=='andi' then result=bit.band(a,kind=='and' and b or n)
  elseif kind=='or' or kind=='ori' then result=bit.bor(a,kind=='or' and b or n)
  elseif kind=='xor' or kind=='xori' then result=bit.bxor(a,kind=='xor' and b or n)
  elseif kind=='nor' then result=bit.bnot(bit.bor(a,b))
  elseif kind=='lui' then result=n*65536
  elseif kind=='sll' then result=bit.lshift(b,n)
  elseif kind=='srl' then result=bit.rshift(b,n)
  elseif kind=='sra' then result=bit.arshift(b,n)
  elseif kind=='sllv' then result=bit.lshift(b,a%32)
  elseif kind=='srlv' then result=bit.rshift(b,a%32)
  elseif kind=='srav' then result=bit.arshift(b,a%32)
  else
   local size
   if kind=='lb' or kind=='lbu' then size=1 elseif kind=='lh' or kind=='lhu' then size=2 else size=4 end
   local address=physical(unsigned(a+n))
   assert(address+size<=config.ram_top,'SP restore outside main RAM')
   result=tonumber(ffi.cast(size==1 and 'uint8_t*' or size==2 and 'uint16_t*' or 'uint32_t*',mem+address)[0])
   if kind=='lb' and result>=128 then result=result-256 elseif kind=='lh' and result>=32768 then result=result-65536 end
  end
  observe_sp(unsigned(result),pc);stack_calls=stack_calls+1
 end))
end
for _,entry in ipairs(config.allocators) do
 add_watch(entry,'Allocator entry',guarded(function()
  if not started then return end
  snapshot_heap()
  local destination=register(31)
  if not returns[destination] then
   returns[destination]=add_watch(destination,'Allocator return',guarded(snapshot_heap))
  end
 end))
end
EPOK_MEMORY_OBSERVER={watch=watch,returns=returns,shared_callback=shared_callback}
EPOK_MEMORY_OBSERVER.sample=guarded(scan_stack)
local function json_quote(value)
 return '"'..value:gsub('[%z\1-\31\\"]',function(character)
  if character=='\\' or character=='"' then return '\\'..character end
  return string.format('\\u%04x',string.byte(character))
 end)..'"'
end
EPOK_MEMORY_OBSERVER.json=function()
 local arena='null'
 if config.lua_stats and config.lua_stats~=0 then
  local address=config.lua_stats
  arena=string.format('{"live":%d,"peak":%d,"allocations":%d,"frees":%d,"reallocs":%d,"failures":%d,"arena_bytes":%d}',word(address),word(address+4),word(address+8),word(address+12),word(address+16),word(address+20),word(address+24))
 end
 return string.format('{"started":%s,"error":%s,"initial_sp":%d,"heap_start":%d,"heap_live_bytes":%d,"heap_footprint_bytes":%d,"heap_peak_live_bytes":%d,"heap_peak_footprint_bytes":%d,"sdk_maximum_bytes":%d,"stack_min_sp":%d,"stack_touched_min":%d,"stack_peak_bytes":%d,"minimum_gap_bytes":%d,"stack_callbacks":%d,"scratchpad_callbacks":%d,"low_ram_stack_callbacks":%d,"low_ram_stack_min":%d,"low_ram_stack_max":%d,"heap_samples":%d,"canary_scans":%d,"lua_arena":%s}',
 tostring(started),json_quote(error_message),initial_sp,config.heap_start,heap_live,heap_footprint,heap_live_peak,heap_high-config.heap_start,sdk_high,stack_min,touched_min,config.ram_top-math.min(stack_min,touched_min),minimum_gap,stack_calls,scratchpad_calls,low_ram_stack_calls,low_ram_stack_min,low_ram_stack_max,heap_samples,scans,arena)
end
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
-- HTTP can interrupt the allocator mid-update. Only coherent entry/return and
-- completed-frame hooks may sample it; requests read the retained observation.
PCSX.WebServer.Handlers['memory-observer']=function()return EPOK_MEMORY_OBSERVER.json()end
