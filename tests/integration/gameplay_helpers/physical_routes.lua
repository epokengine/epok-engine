-- Explicit initial position setup only. Game rules, motion/velocity, grounded
-- flags, camera state, captures and platform state are never written here.
local physical_trace={};local pose_setups={};local physical_setup_observations={}
local function current_stage(tick)
 for _,stage in ipairs(config.physical_route.stages)do
  if tick>=stage.start and tick<stage.start+stage.setup+stage.measure then return stage,tick-stage.start+1 end
 end
 return nil,nil
end
local function set_initial_position(tick,stage,position,support)
 assert(tick>=stage.start and tick<stage.start+stage.setup,'Position write outside explicit setup')
 assert(#position==(config.template=='third-person' and 3 or 2),'Unreviewed position write dimensions')
 local owner=actual_actor(config.player_name);local address
 if config.template=='third-person' then
  local data=u32(owner+offset('Actor','m_data'));assert(data~=0,'Setup actor data unavailable')
  address=data+offset('ActorData','transform')+offset('Transform','position')
 else address=root2d(owner)+offset('SceneComponent2D','transform')+offset('Transform2D','position')end
 for i,value in ipairs(position)do ffi.cast('int32_t*',mem+physical(address+(i-1)*4))[0]=value end
 pose_setups[#pose_setups+1]={tick=tick,phase=stage.name,kind='position-only',address=address,position=position,sampled_support_world=support}
end
local function lift_world_position()
 local at=position2d(actual_actor(config.lift_name));local parent=config.physical_route.lift_parent_offset
 return {at[1]+parent[1],at[2]+parent[2]}
end
local function physical_input(tick)
 if tick<=240 then return assert(config.timeline[tick],'Original240 input prefix missing')end
 if tick<=240+config.physical_route.prefix_padding then return {step=tick,buttons=0,adc={128,128,128,128}}end
 local stage,index=current_stage(tick);assert(stage,'Unreviewed physical phase tick')
 local sample={step=tick,buttons=index>stage.setup and stage.buttons or 0,adc={128,128,128,128}}
 if index==1 then
  if stage.pose then
   local raw={};for _,v in ipairs(stage.pose)do raw[#raw+1]=math.floor(v*4096)end
   set_initial_position(tick,stage,raw)
  elseif stage.lift_pose then
   local at=lift_world_position();set_initial_position(tick,stage,{at[1],at[2]-15*4096},at)
  end
 end
 if stage.second_pose_step==index then
  local raw={};for _,v in ipairs(stage.second_pose)do raw[#raw+1]=math.floor(v*4096)end
  set_initial_position(tick,stage,raw)
 end
 if index==stage.jump_setup_step or index-stage.setup==stage.jump_measure_step then sample.buttons=bit.bor(sample.buttons,16384)end
 if config.template=='third-person' and index<=stage.setup then
  -- Center the camera through its real look input, not a camera/angle write.
  local camera=actual_actor(config.camera_name);local view=camera+offset('CameraRig3D','sample_')
  local yaw=i32(view+offset('CameraRigSample','yaw'))/4096
  local difference=(yaw+180)%360-180
  local dt=(68+math.floor(16*tick/60)-math.floor(16*(tick-1)/60))/4096
  local adc=128-difference*127/(config.physical_route.look_rate*dt)
  sample.adc[3]=math.max(1,math.min(255,math.floor(adc+.5)))
 end
 return sample
end
local function physical_record(row,tick)
 local stage,index=current_stage(tick)
 if not stage then return end
 if index<=stage.setup then
  if index==stage.jump_setup_step then
   physical_setup_observations[#physical_setup_observations+1]={phase=stage.name,tick=tick,kind='actual-jump-seed',jumped=row.jumped,velocity=row.velocity}
  end
  return
 end
 local value={phase=stage.name,step=index-stage.setup,global_tick=tick,sample=row,input=input_trace[tick]}
 if config.template=='third-person' then
  local view=actual_actor(config.camera_name)+offset('CameraRig3D','sample_')
  value.camera_distance=i32(view+offset('CameraRigSample','distance'))
 else
  local body=actual_component(actual_actor(config.player_name),config.body_name)
  value.on_ceiling=u8(body+offset('CharacterBody2DComponent','on_ceiling'))~=0
  if stage.name=='lift-carry' then value.lift_world_position=lift_world_position()end
 end
 physical_trace[#physical_trace+1]=value
end
