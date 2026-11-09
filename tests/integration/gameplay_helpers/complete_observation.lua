-- External Redux observer: no linked code/BSS and no rule-state writes.
local C=COMPLETE_CONFIG
local L=C.abi.layouts
local function byte(address) return tonumber(ffi.cast('uint8_t*',mem+address)[0]) end
local function half(address) return tonumber(ffi.cast('uint16_t*',mem+address)[0]) end
local function word(address) return tonumber(ffi.cast('uint32_t*',mem+address)[0]) end
local function field(group,name)
 return word(C.symbols[group]+C.abi.word_offsets[group][name]*4)
end
local function object(role)
 return C.abi.object_address+C.scene.slots[role]*L.ActorData.size
end
local function active(role)
 return byte(object(role)+L.ActorData.members.alive)~=0 and byte(object(role)+L.ActorData.members.active)~=0
end
-- Event indices are the reviewed DungeonSound contract, not structure offsets.
local function cue(index) return word(C.symbols.audio+C.abi.word_offsets.audio.events*4+index*4) end
local controller=0;local observed={};local previous=-1;local stage=0;local entered=0;local mask=0
local previous_target=65535;local previous_generation=0
local first_target=nil;local initial_restarts=nil;local victory_restarts=nil
local actions={};local completed=false
local regs=PCSX.getRegisters()
EPOK_COMPLETE_CONTROLLER=PCSX.addBreakpoint(C.abi.controller_entry,'Exec',4,'Read current controller instance',function()
 controller=bit.band(tonumber(regs.GPR.n.a0),0x1fffff);return true
end)
local function place(role,n,z)
 local p=C.scene.positions_q12[role]
 local target=ffi.cast('int32_t*',mem+C.abi.player_position_address)
 target[0]=p[1];target[1]=0;target[2]=z or p[3]
 actions[#actions+1]=string.format('{"frame":%d,"pose_role":"%s","position_q12":[%d,0,%d]}',n,role,p[1],z or p[3])
end
local function guards(n)
 -- Same canonical encounter setup as guards route; actual spawn/policy/attacks run.
 local target=ffi.cast('int32_t*',mem+C.abi.player_position_address)
 target[0]=0;target[1]=0;target[2]=118784
 actions[#actions+1]=string.format('{"frame":%d,"pose_role":"guards","position_q12":[0,0,118784]}',n)
end
local function advance(next_stage,n) stage=next_stage;entered=n;mask=0 end
local function drive(r)
 local n=r.frame
 if stage==0 and r.phase==4 and not r.locked then
  initial_restarts=r.restarts;guards(n);advance(1,n)
 elseif stage==1 and n-entered>=60 then mask=2048;if n-entered>=62 then advance(2,n) end
 elseif stage==2 and r.target_valid then first_target=r.target_index;advance(3,n)
 elseif stage==3 and n-entered>=30 then mask=2048;if n-entered>=32 then advance(4,n) end
 elseif stage==4 and r.target_valid and r.target_index~=first_target then mask=1024;advance(5,n);mask=1024
 elseif stage==5 then
  mask=1024
  if n-entered>=30 and r.camera_mode==2 and not r.target_valid then advance(6,n) end
 elseif stage==6 and r.health>0 and r.damage>0 and r.image_enabled then place('pack0',n);advance(7,n)
 elseif stage==7 and r.heals>0 and r.pack_inactive then guards(n);advance(8,n)
 elseif stage==8 and r.state==3 and r.health==0 then advance(9,n)
 elseif stage==9 and r.restarts>initial_restarts and r.phase==4 and not r.locked then place('key',n);advance(10,n)
 elseif stage==10 and r.key~=0 then place('boss',n,C.scene.reactor_entry_q12+4096);advance(11,n)
 elseif stage==11 and r.state==1 then mask=34816;advance(12,n);mask=34816
 elseif stage==12 then
  mask=32768
  -- R1 edge is retried only at fixed planned intervals, ordinary lock cycling.
  if (n-entered)%120<2 then mask=34816 end
  if r.state==2 and r.boss_health==0 then victory_restarts=r.restarts;advance(13,n) end
 elseif stage==13 and n-entered>=30 then mask=8;if n-entered>=32 then advance(14,n) end
 elseif stage==14 and r.restarts>victory_restarts and r.phase==4 and not r.locked then completed=true;advance(15,n)
 end
end
local function sample()
 local n=tonumber(frame[0]);if n<0 or n>1000000 or n<=previous then return true end
 previous=n
 local target_index=65535;local target_generation=0;local valid=false;local mode=0
 if controller~=0 and controller+L.DungeonPlayerController.size<0x200000 then
  local handle=controller+L.DungeonPlayerController.members.target_handle
  target_index=half(handle+L.DataHandle.members.index)
  target_generation=word(handle+L.DataHandle.members.generation)
  mode=word(controller+L.DungeonPlayerController.members.camera_mode)
  if target_index<C.abi.slot_capacity then
   local data=C.abi.object_address+target_index*L.ActorData.size
   valid=word(data+L.ActorData.members.generation)==target_generation and byte(data+L.ActorData.members.alive)~=0 and byte(data+L.ActorData.members.active)~=0
  end
 end
 local image=object('hud')+L.ActorData.members.image
 local old_valid=false
 if previous_target<C.abi.slot_capacity then
  local data=C.abi.object_address+previous_target*L.ActorData.size
  old_valid=word(data+L.ActorData.members.generation)==previous_generation and byte(data+L.ActorData.members.alive)~=0 and byte(data+L.ActorData.members.active)~=0
 end
 local r={frame=n,step=field('game','ticks'),state=field('game','state'),health=field('game','health'),key=field('game','key'),kills=field('game','kills'),
  boss_health=field('game','boss_health'),damage=field('game','damage'),heals=field('game','heals'),restarts=field('game','restarts'),
  phase=field('flow','phase'),locked=field('flow','inputs_locked')~=0,fade=field('flow','fade'),music_playing=field('flow','music_playing')~=0,music_volume=field('flow','music_volume'),
  shots=field('combat','shots'),hits=field('combat','hits'),muzzle_errors=field('combat','muzzle_errors'),
  menu=field('economy','menu'),selected=field('economy','selection'),
  target_index=target_index,target_valid=valid,previous_target_valid=old_valid,camera_mode=mode,marker_active=active('marker'),pad_mask=last,
  image_enabled=byte(image+L.Image.members.enabled)~=0,pack_inactive=not active('pack0'),
  lock_audio=cue(9),damage_audio=cue(3),heal_audio=cue(16),hit_audio=cue(2),boss_destroyed_audio=cue(8),audio_missing=field('audio','missing')}
 local borders={};for i=0,3 do borders[#borders+1]=half(image+L.Image.members.borders+i*2) end
 local color={};for i=0,2 do color[#color+1]=byte(image+L.Image.members.color+i) end
 r.borders=borders;r.image_color=color
 local fields={}
 for _,key in ipairs({'frame','step','state','health','key','kills','boss_health','damage','heals','restarts','phase','locked','fade','music_playing','music_volume','shots','hits','muzzle_errors','menu','selected','target_index','target_valid','previous_target_valid','camera_mode','marker_active','pad_mask','image_enabled','pack_inactive','lock_audio','damage_audio','heal_audio','hit_audio','boss_destroyed_audio','audio_missing'}) do
  fields[#fields+1]='"'..key..'":'..tostring(r[key])
 end
 fields[#fields+1]='"borders":['..table.concat(borders,',')..']'
 fields[#fields+1]='"image_color":['..table.concat(color,',')..']'
 observed[#observed+1]='{'..table.concat(fields,',')..'}'
 previous_target=target_index;previous_generation=target_generation
 if C.closure then drive(r) end
 return true
end
EPOK_COMPLETE_OBSERVATION=PCSX.addBreakpoint(C.abi.frame_entry,'Exec',4,'Complete current policies and visual feedback',sample)
EPOK_COMPLETE_OBSERVER={pad=function()return mask end}
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
PCSX.WebServer.Handlers['complete-observation']=function()
 return string.format('{"schema":1,"completed":%s,"stage":%d,"rows":[%s],"pose_actions":[%s]}',tostring(completed),stage,table.concat(observed,','),table.concat(actions,','))
end
