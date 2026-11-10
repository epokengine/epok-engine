-- External observer only: no target instructions, target BSS, rule substitution
-- or fixed ActorData ABI offsets. Configuration comes from this exact ELF.
local ffi=require('ffi')
local bit=require('bit')
local config=@CONFIG@
local mem=PCSX.getMemPtr();local regs=PCSX.getRegisters()
local function physical(p)return bit.band(tonumber(p),0x1fffff)end
local function u32(p)return tonumber(ffi.cast('uint32_t*',mem+physical(p))[0])end
local function i32(p)return tonumber(ffi.cast('int32_t*',mem+physical(p))[0])end
local function u8(p)return tonumber(ffi.cast('uint8_t*',mem+physical(p))[0])end
local function quote(s)return '"'..tostring(s):gsub('\\','\\\\'):gsub('"','\\"'):gsub('\n','\\n')..'"'end
local function json(v)
 if type(v)=='table' then
  local out={};if v[1]~=nil or next(v)==nil then for _,x in ipairs(v)do out[#out+1]=json(x)end;return '['..table.concat(out,',')..']' end
  for k,x in pairs(v)do out[#out+1]=quote(k)..':'..json(x)end;table.sort(out);return '{'..table.concat(out,',')..'}'
 elseif type(v)=='string' then return quote(v) elseif type(v)=='number' or type(v)=='boolean' then return tostring(v)end
 return 'null'
end
local function offset(type_name,member)return config.layouts[type_name].members[member]end
local function field(symbol,type_name,member)return config.symbols[symbol]+offset(type_name,member)end
local ticks_address=field('epok::time','Time','ticks')
local frame_started=nil;local completed_cycles=0;local trace={};local frames={};local errors={}
local returns={};local finished=false;local exact_cpu=nil;local actor=nil
local step_observations={};local frame_ordinal=0;local frame_ticks=0;local clock_callbacks=0
assert(ticks_address==config.tick_counter_address,'Clock address differs from exact ELF/DWARF')
assert((config.template=='basic' and config.observation_boundary=='time-write-and-frame-return') or
 (config.template=='sample' and config.observation_boundary=='selected-actor-tick-return'),'Wrong template observation boundary')
local actor_size=config.layouts.ActorData.size
local function find_actor()
 local count=u32(config.symbols['epok::object_count']);assert(count<=config.slot_capacity,'Runtime object count exceeds exact linked array')
 local found=nil
 for n=0,count-1 do
  local address=config.symbols['epok::objects']+n*actor_size
  local name=ffi.string(mem+physical(address+offset('ActorData','name')),129):match('^[^%z]*')
  if name==config.actor_name then assert(not found,'Ambiguous source actor name');found=address end
 end
 return found
end
local function transform(address)
 local base=address+offset('ActorData','transform');local out={}
 for _,member in ipairs({'position','rotation','scale'})do local values={};for n=0,2 do values[#values+1]=i32(base+offset('Transform',member)+4*n)end;out[member]=values end
 return out
end
local function snapshot_step(boundary)
 if finished or #trace>=config.steps then return end
 local tick=u32(ticks_address);if tick==0 or tick<=#trace then return end
 actor=actor or find_actor();if not actor then return end
 assert(tick==#trace+1,'Simulation boundary skipped; equal-step trace would be incomplete')
 local value=transform(actor);local owner=u32(actor+offset('ActorData','owner'))
 if owner==0 or u8(actor+offset('ActorData','alive'))==0 then return end
 if config.template=='sample' then
  local expected=45*(68*tick+math.floor(16*tick/60))
  -- tick_actor runs for each actor/group. Observe only after the actual
  -- selected Spinner has applied this exact tick, never reconstruct a row.
  if value.rotation[2]~=expected then return end
  assert(value.rotation[1]==0 and value.rotation[3]==0,'Spinner changed another axis')
 else
  for i=1,3 do assert(value.position[i]==0 and value.rotation[i]==0 and value.scale[i]==4096,'Inert GameRoot changed')end
 end
 trace[#trace+1]={step=tick,position=value.position,rotation=value.rotation,scale=value.scale,owner=true}
 assert(frame_started,'Observed simulation state outside actual frame')
 local cpu=completed_cycles+tonumber(PCSX.getCPUCycles())-frame_started
 step_observations[#step_observations+1]={step=tick,boundary=boundary,frame=frame_ordinal,cpu_work_units=cpu}
 if #trace==config.steps then
  exact_cpu=cpu
 end
end
local function counters()
 local perf={};for n=0,config.performance_words-1 do perf[#perf+1]=u32(config.symbols['epok::performance_stats']+n*4)end
 local actors={};for n=0,8 do actors[#actors+1]=u32(config.symbols['epok::actor_stats']+n*4)end
 local hud={};for n=0,config.hud_words-1 do hud[#hud+1]=u32(config.symbols['epok::hud_stats']+n*4)end
 local maximum_components=0;local active_audio=0
 for n=0,u32(config.symbols['epok::object_count'])-1 do
  local p=config.symbols['epok::objects']+n*actor_size;local owner=u32(p+offset('ActorData','owner'))
  if owner~=0 and u8(p+offset('ActorData','alive'))~=0 then
   maximum_components=math.max(maximum_components,u8(owner+offset('Actor','m_component_count')))
   if u8(p+offset('ActorData','audio')+offset('AudioSource','enabled'))~=0 then active_audio=active_audio+1 end
  end
 end
 return {ticks=u32(ticks_address),dropped_steps=u32(field('epok::time','Time','dropped_steps')),
  frame_sequence=u32(field('epok::performance_stats','PerformanceStats','frame')),
  requested_steps=u32(field('epok::performance_stats','PerformanceStats','steps')),
  frame_us=u32(field('epok::time','Time','frame_microseconds')),performance=perf,actors=actors,hud=hud,
  resident_texture_bytes=u32(field('epok::resource_usage','ResourceUsage','resident_texture_bytes')),
  active_texture_bytes=u32(field('epok::resource_usage','ResourceUsage','active_texture_bytes')),
  dropped_primitives=u32(field('epok::resource_usage','ResourceUsage','dropped_primitives')),
  hud_dropped=u32(field('epok::hud_stats','HudStats','dropped')),active_audio=active_audio,
  max_components_per_actor=maximum_components}
end
local function finish(reason)
 if finished then return end;finished=true
 local observation={reason=reason,trace=trace,frames=frames,errors=errors,cpu_work_units=exact_cpu,
  step_observations=step_observations,clock_observer={kind=config.observation_boundary,
   address=ticks_address,write_callbacks=clock_callbacks,frames=frame_ordinal},
  target_instructions_added=0,target_bss_added=0,diagnostic_memory=config.diagnostic_memory,
  source_inventory_sha256=config.source_inventory_sha256,executable_sha256=config.executable_sha256}
 if EPOK_MEMORY_OBSERVER then observation.memory=EPOK_MEMORY_OBSERVER.json()end
 local f=assert(io.open(config.output,'w'));f:write(json(observation));f:close();PCSX.quit(#errors==0 and 0 or 1)
end
local function guarded(fn)return function(...)
 local ok,message=pcall(fn,...);if not ok then errors[#errors+1]=tostring(message);finish('observer-error')end
 return true
end end
local function at_return(kind,fn)
 local destination=tonumber(regs.GPR.n.ra)
 local key=kind..':'..tostring(destination)
 if not returns[key] then returns[key]=PCSX.addBreakpoint(destination,'Exec',4,'P10 '..kind..' return',guarded(fn))end
end
if config.template=='sample' then
 assert(config.tick_entry,'Selected Spinner requires an actual compiled actor tick boundary')
 P10_TICK=PCSX.addBreakpoint(config.tick_entry,'Exec',4,'P10 real simulation callback',guarded(function()
  at_return('tick',function()snapshot_step('tick-return')end)
 end))
else
 -- Basic has no gameplay tick callbacks. A read-only data watch catches the
 -- actual Time counter even when begin_tick is inlined and tick_actor removed.
 -- Read the committed RAM value, never the prospective store/register value.
 -- Whether Redux notifies before or after the store, frame return observes the
 -- final committed tick; subsequent writes expose intermediate catch-up ticks.
 P10_TIME_TICKS=PCSX.addBreakpoint(ticks_address,'Write',4,'P10 real Time ticks',guarded(function(address,width)
  if finished or not frame_started then return end
  assert(physical(address)==physical(ticks_address) and width==4,'Unexpected Time counter access')
  clock_callbacks=clock_callbacks+1;snapshot_step('time-write')
 end))
end
P10_FRAME=PCSX.addBreakpoint(config.frame_entry,'Exec',4,'P10 actual GameScene frame',guarded(function()
 assert(not frame_started,'Nested or lost frame boundary');frame_started=tonumber(PCSX.getCPUCycles())
 frame_ordinal=frame_ordinal+1;frame_ticks=u32(ticks_address)
 at_return('frame',function()
  if finished then return end
  if config.template=='basic' then snapshot_step('frame-return')end
  if EPOK_MEMORY_OBSERVER then EPOK_MEMORY_OBSERVER.sample()end
  local elapsed=tonumber(PCSX.getCPUCycles())-frame_started
  local row=counters();row.cpu_cycles=elapsed;row.observer_frame=frame_ordinal;row.start_ticks=frame_ticks
  row.executed_steps=row.ticks-frame_ticks
  assert(row.executed_steps>=0 and row.executed_steps<=8,'Time reset or skipped simulation frame')
  assert(row.requested_steps==row.executed_steps,'Published simulation steps differ from actual clock progress')
  if #frames>0 then assert(row.frame_sequence==frames[#frames].frame_sequence+1,'Published frame sequence missing or repeated')end
  frames[#frames+1]=row;completed_cycles=completed_cycles+elapsed;frame_started=nil
  if #trace==config.steps then finish('exact-steps')elseif #frames>config.steps*4 then error('No complete selected-rule simulation trace')end
 end)
end))
