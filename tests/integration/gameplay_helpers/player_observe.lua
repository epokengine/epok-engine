-- Composed with frozen observe.lua. Device input is the only writable target
-- state. All gameplay/physics/animation/pickup/camera values are read from RAM.
local input_trace={};local input_writes={count=0,ranges={}}
local last_injected_tick=0;local previous_buttons=0
local function u16(p)return tonumber(ffi.cast('uint16_t*',mem+physical(p))[0])end
local function vec(p,kind,n)
 local out={};local names={'x','y','z'}
 for i=1,n do out[i]=i32(p+offset(kind,names[i]))end
 return out
end
local function registry()
 local value=u32(config.symbols['epok::active_object_registry']);assert(value~=0,'Actual registry unavailable')
 return value,u32(value+offset('ObjectRegistry','slots')),u16(value+offset('ObjectRegistry','capacity'))
end
local function resolve_id(p)
 local _,slots,capacity=registry();local index=u16(p+offset('ObjectId','index'));local generation=u16(p+offset('ObjectId','generation'))
 assert(index<capacity and generation~=0,'Actual object reference invalid')
 local slot=slots+index*config.layouts.ObjectSlot.size
 assert(u16(slot+offset('ObjectSlot','generation'))==generation and u8(slot+offset('ObjectSlot','state'))~=0,'Actual object reference stale')
 local value=u32(slot+offset('ObjectSlot','instance'));assert(value~=0,'Actual object instance missing');return value
end
local function registry_actors()
 local _,slots,capacity=registry();assert(capacity>0 and capacity<65535,'Invalid exact registry capacity')
 local values={}
 for i=0,capacity-1 do
  local slot=slots+i*config.layouts.ObjectSlot.size;local value=u32(slot+offset('ObjectSlot','instance'));local kind=u32(slot+offset('ObjectSlot','type'))
  if value~=0 and kind~=0 and u8(slot+offset('ObjectSlot','state'))==3 and u8(kind+offset('ClassDescriptor','family'))==1 then values[#values+1]=value end
 end
 return values
end
local function actual_actor(name)
 local found=nil
 for _,value in ipairs(registry_actors())do
  local label=ffi.string(mem+physical(value+offset('Actor','m_name')),33):match('^[^%z]*')
  if label==name then assert(not found,'Ambiguous current actor label');found=value end
 end
 return assert(found,'Actual source actor missing: '..name)
end
local function actual_component(owner,name)
 local count=u8(owner+offset('Actor','m_component_count'));assert(count<=8,'Actual component cap exceeded')
 local found=nil
 for i=0,count-1 do
  local value=resolve_id(owner+offset('Actor','m_components')+i*config.layouts.ObjectId.size)
  local label=ffi.string(mem+physical(value+offset('ActorComponent','m_name')),33):match('^[^%z]*')
  if label==name then assert(not found,'Ambiguous current component label');found=value end
 end
 return assert(found,'Actual source component missing: '..name)
end
local function root2d(owner)
 return resolve_id(owner+offset('Actor','m_root'))
end
local function position2d(owner)
 local base=root2d(owner)+offset('SceneComponent2D','transform')+offset('Transform2D','position')
 return {i32(base),i32(base+4)}
end
local function same_id(a,b)return u32(a)==u32(b)end
local function active(owner)
 local flag=config.actor_active_bit;return bit.band(u8(owner+flag.byte),bit.lshift(1,flag.bit))~=0
end
local input_base=config.symbols['epok::input']+offset('Input','ports')
local edges_base=config.symbols['epok::gameplay_controls()::value']+offset('GameplayControls','m_edges')
input_writes.ranges={{address=input_base,bytes=config.layouts.State.size},{address=edges_base,bytes=config.layouts.Edges.size}}
local function device_write(address,value,width)
 local allowed=false
 for _,range in ipairs(input_writes.ranges)do if address>=range.address and address+width<=range.address+range.bytes then allowed=true end end
 assert(allowed,'Observer attempted a non-device target write')
 if width==1 then ffi.cast('uint8_t*',mem+physical(address))[0]=value
 elseif width==2 then ffi.cast('uint16_t*',mem+physical(address))[0]=value
 else error('Unreviewed input write width')end
 input_writes.count=input_writes.count+1
end
local function inject_input(tick)
 local sample=assert(config.timeline[tick],'Missing exact input step');assert(sample.step==tick,'Input timeline identity mismatch')
 local held=sample.buttons
 local pressed=bit.band(held,bit.bnot(previous_buttons));local released=bit.band(previous_buttons,bit.bnot(held))
 local raw={held=held,pending_pressed=0,pending_released=0,pressed=pressed,released=released,frame_pressed=pressed,frame_released=released}
 for name,value in pairs(raw)do device_write(input_base+offset('State',name),value,2)end
 device_write(input_base+offset('State','connected'),1,1);device_write(input_base+offset('State','analog'),1,1)
 for i=1,4 do
  local delta=sample.adc[i]-128;local value=delta*4096/(delta<0 and 128 or 127)
  value=value<0 and math.ceil(value) or math.floor(value);if i%2==0 then value=-value end
  device_write(input_base+offset('State','axes')+(i-1)*2,value%65536,2)
 end
 local edges={frame_pressed=pressed,frame_released=released,pending_pressed=0,pending_released=0,tick_pressed=pressed,tick_released=released}
 for name,value in pairs(edges)do device_write(edges_base+offset('Edges',name),value,2)end
 previous_buttons=held;clock_callbacks=clock_callbacks+1;input_trace[#input_trace+1]=sample
end
local function snapshot_step(tick,boundary)
 if finished or #trace>=config.steps or tick<=#trace then return end
 assert(tick==#trace+1,'Real player simulation boundary skipped')
 local owner=actual_actor(config.player_name);local motor=actual_component(owner,config.motor_name)
 local animation=actual_component(owner,config.animation_name);local camera=actual_actor(config.camera_name)
 local is3d=config.template=='third-person';local motion_type=is3d and 'CharacterMotionSample3D' or 'CharacterMotionSample2D'
 local motor_type=is3d and 'CharacterMotor3DComponent' or 'CharacterMotor2DComponent'
 local animation_type=is3d and 'SkeletalLocomotionComponent' or 'SpriteLocomotion2DComponent'
 local motion=motor+offset(motor_type,'sample_')
 local playback=animation+offset(animation_type,'runtime_')+offset('Runtime','sample')
 local row={step=tick,owner=true,motion_status=u8(motion+offset(motion_type,'status')),
  animation_status=u8(playback+offset('LocomotionSample','status')),animation_state=u8(playback+offset('LocomotionSample','state')),
  animation_clip=i32(playback+offset('LocomotionSample','clip')),animation_rate=i32(playback+offset('LocomotionSample','rate')),
  velocity=vec(motion+offset(motion_type,'velocity'),is3d and 'GameplayVector3' or 'GameplayVector2',is3d and 3 or 2),
  displacement=vec(motion+offset(motion_type,'displacement'),is3d and 'GameplayVector3' or 'GameplayVector2',is3d and 3 or 2),
  grounded=u8(motion+offset(motion_type,'grounded'))~=0,jumped=u8(motion+offset(motion_type,'jumped'))~=0,
  blocked=u8(motion+offset(motion_type,'blocked'))~=0}
 if is3d then
  local data=u32(owner+offset('Actor','m_data'));assert(data~=0,'3D player slot missing')
  local base=data+offset('ActorData','transform')
  row.position={};row.rotation={};row.scale={}
  for i=0,2 do row.position[i+1]=i32(base+offset('Transform','position')+i*4);row.rotation[i+1]=i32(base+offset('Transform','rotation')+i*4);row.scale[i+1]=i32(base+offset('Transform','scale')+i*4)end
  local view=camera+offset('CameraRig3D','sample_')
  row.camera_position=vec(view+offset('CameraRigSample','position'),'GameplayVector3',3)
  row.camera_yaw=i32(view+offset('CameraRigSample','yaw'));row.camera_pitch=i32(view+offset('CameraRigSample','pitch'))
  row.camera_valid=u8(view+offset('CameraRigSample','status'))==0;row.camera_blocked=u8(view+offset('CameraRigSample','blocked'))~=0
 else
  row.position=position2d(owner)
  local view=actual_component(camera,config.camera_component_name)
  row.camera_position={i32(view+offset('Camera2DComponent','position')),i32(view+offset('Camera2DComponent','position')+4)}
  row.camera_valid=u8(view+offset('Camera2DComponent','initialized'))~=0 and u8(view+offset('Camera2DComponent','current'))~=0 and
   same_id(root2d(camera)+offset('SceneComponent2D','attach_parent'),owner+offset('Actor','m_root'))
  row.coins_active={};for _,name in ipairs(config.coin_names)do row.coins_active[#row.coins_active+1]=active(actual_actor(name))end
  local lift=actual_actor(config.lift_name);row.lift_position=position2d(lift)
  local body=actual_component(owner,config.body_name)
  row.hit_tile=u8(body+offset('CharacterBody2DComponent','last_hit_tile'))~=0
  row.tile={i32(body+offset('CharacterBody2DComponent','last_tile')),i32(body+offset('CharacterBody2DComponent','last_tile')+4)}
  -- Normalize identity to a semantic relation across providers. Lift movement
  -- alone does not certify carry; the later route must observe grounded contact.
  row.on_lift=row.grounded and same_id(body+offset('CharacterBody2DComponent','last_collider'),lift+offset('Object','m_id'))
 end
 assert(frame_started,'Player observation outside actual frame')
 trace[#trace+1]=row
 local cpu=completed_cycles+tonumber(PCSX.getCPUCycles())-frame_started
 step_observations[#step_observations+1]={step=tick,boundary=boundary,frame=frame_ordinal,cpu_work_units=cpu,clock_read=u32(ticks_address)}
 if #trace==config.steps then exact_cpu=cpu end
end
