-- External actual-Dungeon route. Only player pose is injected; collection,
-- ownership flag, door state/collision and selected-language callbacks execute
-- in the unchanged target program. No fixture actor or rule is installed.
local ffi=require('ffi');local bit=require('bit');local c=@CONFIG@
local mem=PCSX.getMemPtr();local regs=PCSX.getRegisters()
local function physical(p)return bit.band(tonumber(p),0x1fffff)end
local function u32(p)return tonumber(ffi.cast('uint32_t*',mem+physical(p))[0])end
local function i32(p)return tonumber(ffi.cast('int32_t*',mem+physical(p))[0])end
local function u8(p)return tonumber(ffi.cast('uint8_t*',mem+physical(p))[0])end
local function off(t,m)return c.layouts[t].members[m]end
local function put(p,v)ffi.cast('int32_t*',mem+physical(p))[0]=v end
local function quote(s)return '"'..tostring(s):gsub('\\','\\\\'):gsub('"','\\"'):gsub('\n','\\n')..'"'end
local function json(v)
 if type(v)=='table' then local out={};if v[1]~=nil or next(v)==nil then for _,x in ipairs(v)do out[#out+1]=json(x)end;return '['..table.concat(out,',')..']'end
  for k,x in pairs(v)do out[#out+1]=quote(k)..':'..json(x)end;table.sort(out);return '{'..table.concat(out,',')..'}'
 elseif type(v)=='string'then return quote(v)elseif type(v)=='number'or type(v)=='boolean'then return tostring(v)end
 return 'null'
end
local objects=c.symbols['epok::objects'];local stride=c.layouts.ActorData.size
local addresses={};local rows={};local transitions={};local failures={};local finished=false
local started=nil;local previous=nil;local returns={};local last_frame=0
local function find()
 local count=u32(c.symbols['epok::object_count']);assert(count<=c.slot_capacity,'Object array overflow')
 for index=0,count-1 do
  local address=objects+index*stride;local name=ffi.string(mem+physical(address+off('ActorData','name')),129):match('^[^%z]*')
  for role,label in pairs(c.actor_names)do if label==name then assert(not addresses[role]or addresses[role]==address,'Ambiguous actual actor');addresses[role]=address end end
 end
 return addresses.player and addresses.key and addresses.door
end
local function transform(address,member,index)return address+off('ActorData','transform')+off('Transform',member)+index*4 end
local function position(address)
 local out={};for i=0,2 do out[i+1]=i32(transform(address,'position',i))end
 -- Current generated Dungeon groups have identity rotation/scale. Fail closed
 -- rather than guess a world-space pose for a differently authored hierarchy.
 local seen={};local parent=i32(address+off('ActorData','parent'))
 while parent>=0 do
  assert(parent<c.slot_capacity and not seen[parent],'Invalid/cyclic authored parent');seen[parent]=true
  local p=objects+parent*stride
  for i=0,2 do assert(i32(transform(p,'rotation',i))==0 and i32(transform(p,'scale',i))==4096,'Rotated/scaled parent requires an affine observer');out[i+1]=out[i+1]+i32(transform(p,'position',i))end
  parent=i32(p+off('ActorData','parent'))
 end
 return out
end
local function active(address)return u8(address+off('ActorData','alive'))~=0 and u8(address+off('ActorData','active'))~=0 end
local function finish(reason)
 if finished then return end;finished=true
 local output={reason=reason,rows=rows,transitions=transitions,failures=failures,executable_sha256=c.executable_sha256,
  header_sha256=c.header_sha256,source_inventory_sha256=c.source_inventory_sha256,target_instructions_added=0,target_bss_added=0,
  route={away_steps=30,finish_steps=c.steps,player_pose_only=true}}
 local f=assert(io.open(c.output,'w'));f:write(json(output));f:close();PCSX.quit(#failures==0 and 0 or 1)
end
local function guarded(fn)return function()local ok,message=pcall(fn);if not ok then failures[#failures+1]=tostring(message);finish('observer-error')end;return true end end
local function completed()
 if finished or not started then return end
 local cycles=tonumber(PCSX.getCPUCycles())-started;started=nil
 local frame=u32(c.symbols['epok::performance_stats']);if frame<=last_frame then return end;last_frame=frame
 assert(find(),'Current Dungeon roles missing from runtime')
 local ticks=u32(c.symbols['epok::time']+off('Time','ticks'))
 local key,door=active(addresses.key),active(addresses.door)
 if not previous then assert(key and door,'Cold boot must reset key and door')
 elseif previous.key~=key or previous.door~=door then transitions[#transitions+1]={step=ticks,key=key,door=door} end
 previous={key=key,door=door}
 local actor_values={};for i=0,8 do actor_values[#actor_values+1]=u32(c.symbols['epok::actor_stats']+i*4)end
 rows[#rows+1]={frame=frame,step=ticks,key=key,door=door,cpu_cycles=cycles,
  dropped_steps=u32(c.symbols['epok::time']+off('Time','dropped_steps')),actor_stats=actor_values,
  dropped_primitives=u32(c.symbols['epok::resource_usage']+off('ResourceUsage','dropped_primitives')),
  hud_drops=u32(c.symbols['epok::hud_stats']+off('HudStats','dropped'))}
 if ticks>=c.steps then assert(not key and not door,'Actual key/door policy did not complete');finish('key-door-complete')end
end
P10_DUNGEON_FRAME=PCSX.addBreakpoint(c.frame_entry,'Exec',4,'Actual Dungeon frame',guarded(function()
 assert(not started,'Nested frame');started=tonumber(PCSX.getCPUCycles())
 if find()then
  local ticks=u32(c.symbols['epok::time']+off('Time','ticks'));local at=position(addresses.key)
  local x=at[1]-(ticks<30 and 4*4096 or 0)
  put(transform(addresses.player,'position',0),x)
  put(transform(addresses.player,'position',1),at[2]-math.floor(.92*4096))
  put(transform(addresses.player,'position',2),at[3])
 end
 local destination=tonumber(regs.GPR.n.ra)
 if not returns[destination]then returns[destination]=PCSX.addBreakpoint(destination,'Exec',4,'Completed actual Dungeon frame',guarded(completed))end
end))
