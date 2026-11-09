-- Read actual authored parent/derived rule outputs and a public helper actor.
local function named_data(name)
 local found=nil;local count=u32(config.symbols['epok::object_count'])
 assert(count<=config.slot_capacity,'Runtime ActorData array capacity invalid')
 for n=0,count-1 do
  local address=config.symbols['epok::objects']+n*actor_size
  local label=ffi.string(mem+physical(address+offset('ActorData','name')),129):match('^[^%z]*')
  if label==name then assert(not found,'Ambiguous observed actor');found=address end
 end
 return found
end
local function snapshot_step(boundary)
 if finished or #trace>=config.steps then return end
 local tick=u32(ticks_address);if tick==0 or tick<=#trace then return end
 assert(tick==#trace+1,'Edited rule observation skipped an actual Time tick')
 local primary=named_data(config.actor_name);local secondary=named_data(config.secondary_name);local helper=named_data(config.helper_name)
 if not primary or not secondary or not helper then return end
 local first=transform(primary);local second=transform(secondary)
 local total=68*tick+math.floor(16*tick/60)
 -- Wait until both real callbacks applied; never reconstruct either transform.
 if first.rotation[2]~=config.primary_speed*total or second.rotation[2]~=config.secondary_speed*total then return end
 first.step=tick;first.owner=u32(primary+offset('ActorData','owner'))~=0
 first.secondary_owner=u32(secondary+offset('ActorData','owner'))~=0
 first.secondary_rotation=second.rotation
 local collider=helper+offset('ActorData','collider')
 first.helper_enabled=u8(collider+offset('Collider','enabled'))~=0
 first.helper_trigger=u8(collider+offset('Collider','trigger'))~=0
 local extents=collider+offset('Collider','half_extents');first.helper_half_extents={i32(extents),i32(extents+4),i32(extents+8)}
 trace[#trace+1]=first
 assert(frame_started,'Authored edit observation outside actual frame')
 local cpu=completed_cycles+tonumber(PCSX.getCPUCycles())-frame_started
 step_observations[#step_observations+1]={step=tick,boundary=boundary,frame=frame_ordinal,cpu_work_units=cpu}
 if #trace==config.steps then exact_cpu=cpu end
end
