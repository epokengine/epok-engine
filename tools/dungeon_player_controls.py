"""Typed project controller rules used to regenerate editable Blueprint functions.

The matching C++ and Lua project classes contain ordinary authored methods.
This module performs no editor or runtime work at import.
"""
import importlib.util
import json
from pathlib import Path
import uuid

ROOT=Path(__file__).resolve().parents[1]
NATIVE=Path("resources/templates/dungeon-complete/assets/scripts/DungeonCombat")
NS=uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity=lambda n:str(uuid.uuid5(NS,"DungeonPlayerCombat/"+n))
F,U,I,B,V=({"kind":k} for k in ("fixed","uint32","int32","bool","void"))
CPP_TYPES={"fixed":"epok::Fixed","uint32":"uint32_t","int32":"int32_t","bool":"bool"}
record=lambda name,fields:{"kind":"record","cpp_name":name,"fields":[{"name":n,"value_type":t} for n,t in fields]}
VECTOR=record("epok::GameplayVector3",[(a,F) for a in ("x","y","z")])
INTENT=record("epok::MovementIntent",[(a,F) for a in ("x","y","strength")])
STATE=record("DungeonControlCamera",[(n,F) for n in ("yaw","pitch","goal_yaw","goal_pitch","hold","blend")]+[("mode",U)])
CAMINPUT=record("DungeonControlCameraInput",[("look",INTENT)]+[(n,F) for n in ("dt","facing","target_yaw","target_pitch")]+[(n,B) for n in ("free_aim","target","moving")])
MOVE=record("DungeonControlMove",[(n,F) for n in ("x","z","facing")])
PROPOSAL=record("DungeonControlView",[("origin",VECTOR),("offset",VECTOR)]+[(n,F) for n in ("distance","yaw","pitch")])
ANGLES=record("DungeonControlAngles",[("yaw",F),("pitch",F)])
MARKER=record("DungeonControlMarker",[("intro",F),("spin",F),("size",F),("tint",U),("average",B)])
RIG=record("DungeonControlRig",[(n,I) for n in ("bone","stop_a","stop_b")])
ANIM=record("DungeonControlAnimation",[("state",U)]+[(n,B) for n in ("loop","locomotion","switch_clip")]+[("rate",F)])
CLOCK=record("DungeonControlClock",[("phase",U),("ticks",U)])
DIRECTION=record("DungeonControlDirection",[("value",VECTOR),("valid",B)])
RECORDS=[STATE,CAMINPUT,MOVE,PROPOSAL,ANGLES,MARKER,RIG,ANIM,CLOCK,DIRECTION]
CONFIG={
 "control_movement":("DungeonControlMovementTuning",[("speed",F,6.0),("turn_rate",F,540.0)]),
 "control_camera":("DungeonControlCameraTuning",[(n,F,v) for n,v in (("initial_pitch",18),("orbit_rate",140),("pitch_rate",70),("pitch_min",-10),("pitch_max",55),("blend_rate",4),("normal_response",4),("manual_response",12),("normal_cap",90),("manual_cap",140),("pitch_response",12),("manual_time",1))]),
 "control_view":("DungeonControlViewTuning",[(n,F,v) for n,v in (("boom",5),("height",1.1),("aim_height",1.35),("shoulder_offset",.55),("shoulder_min",.5))]),
 "control_aim":("DungeonControlAimTuning",[(n,F,v) for n,v in (("pitch_min",-35),("pitch_max",45),("pose_rate",220),("focus_height",.8),("ray_distance",40))]+[("ray_mask",U,3),("muzzle_mask",U,1)]),
 "control_marker":("DungeonControlMarkerTuning",[(n,F,v) for n,v in (("size",.8),("intro_frames",8),("base_radius",96),("step_radius",64),("brightness_step",16),("brightness_end",128))]),
 "control_animation":("DungeonControlAnimationTuning",[(n,F,v) for n,v in (("walk_cycle",2.6),("run_cycle",6),("rate_min",.2),("rate_max",1.6),("idle_speed",491/4096),("run_speed",14746/4096),("keep_run_speed",3))]),
}

class E:
 def __init__(self,kind,ty,*values):self.kind,self.ty,self.values=kind,ty,values
 def __add__(self,v):return binary("add",self,v)
 def __sub__(self,v):return binary("subtract",self,v)
 def __mul__(self,v):return binary("multiply",self,v)
 def __truediv__(self,v):return binary("divide",self,v)
 def field(self,name):
  return E("field",next(v["value_type"] for v in self.ty["fields"] if v["name"]==name),self,name)
def literal(v,ty=F):return E("literal",ty,v)
def f(v):return literal(float(v),F)
def u(v):return literal(v,U)
def i(v):return literal(v,I)
def b(v):return literal(bool(v),B)
def param(n,t):return E("param",t,n)
def prop(n,field):return E("property",next(t for k,t,_ in CONFIG[n][1] if k==field),n,field)
def binary(op,a,c):
 assert isinstance(a,E) and isinstance(c,E)
 return E("binary",B if op in ("equal","not_equal","less","less_equal","greater","greater_equal","and","or") else a.ty,op,a,c)
def cmp(op,a,c):return binary(op,a,c)
def eq(a,c):return cmp("equal",a,c)
def lt(a,c):return cmp("less",a,c)
def le(a,c):return cmp("less_equal",a,c)
def gt(a,c):return cmp("greater",a,c)
def ge(a,c):return cmp("greater_equal",a,c)
def And(a,c):return binary("and",a,c)
def Or(a,c):return binary("or",a,c)
def Not(a):return E("not",B,a)
def ret(e):return ("return",e)
def branch(test,yes,no=()):return ("if",test,yes,no)

SDK={
 "clamp":("6b6ba8bd-8446-4fef-a147-e41b61f88a61",F),"vector3":("4fe0f030-091c-4d55-984d-769077622c33",VECTOR),
 "sine_degrees":("eeee7d31-a8c6-437a-95db-10e163771b24",F),"cosine_degrees":("565552a8-869a-4ca7-a1d8-a7ad6e1508a7",F),
 "wrap_degrees":("9b882fef-4a7b-4463-81aa-65b2f7e0dcd0",F),"delta_degrees":("c5b4efca-6c21-42fb-b9bf-f354902d7ee4",F),
 "move_toward":("3e369e64-cb38-4930-9842-02bbf054c5d9",F),"move_toward_degrees":("bacfcfc1-e38c-44d2-b47d-19ec3ae37f8d",F),
 "heading_degrees":("1057a007-0420-4b95-8f48-140be81f9ba5",F),"stick_intent":("5b0a28df-53d9-40e4-b7ca-c5381f473771",INTENT),
 "length2":("57b625c6-06d7-4635-b551-17a75ceffa3b",F),"length":("625d5493-89ff-401f-a535-308206d5f4d4",F),
 "truncate_unsigned":("29249825-c121-54e6-a908-8cc041db8971",U),
 "floor":(str(uuid.uuid5(NS,"MathLibrary/floor")),F),
}
def sdk(name,*args):return E("sdk",SDK[name][1],name,*args)
def clamp(v,lo,hi):return sdk("clamp",v,f(lo) if isinstance(lo,(int,float)) else lo,f(hi) if isinstance(hi,(int,float)) else hi)
def sin(v):return sdk("sine_degrees",v)
def cos(v):return sdk("cosine_degrees",v)
def vec(x,y,z):return sdk("vector3",x,y,z)
RULES={}
def rule(name,returns,parameters,body,virtual=True):
 RULES[name]={"name":name,"returns":returns,"parameters":parameters,"body":body,"virtual":virtual}
def call(name,*args):return E("call",RULES[name]["returns"],name,*args)
FACTORIES={}
def make(ty,**fields):
 name="make_controls_"+ty["cpp_name"].split("::")[-1]
 FACTORIES[name]=ty
 return E("make",ty,name,*[fields[p["name"]] for p in ty["fields"]])
def replace(ty,source,**fields):return make(ty,**{p["name"]:fields.get(p["name"],source.field(p["name"])) for p in ty["fields"]})
def safe_dt(dt):return clamp(dt,0,1)
def cprop(group,field,lo,hi):return clamp(prop(group,field),lo,hi)

def define():
 # All game decisions below generate ordinary functions in all three flavors.
 ready,blocked,controls,alive=[param(n,B) for n in ("ready","interface_block","controls","alive")]
 rule("control_gate",U,[("ready",B),("interface_block",B),("controls",B),("alive",B)],
  [branch(Or(Or(Not(ready),blocked),Not(controls)),[ret(u(1))]),branch(Not(alive),[ret(u(2))]),ret(u(0))])
 positive,negative=param("positive",B),param("negative",B)
 rule("control_button_axis",F,[("positive",B),("negative",B)],
  [branch(And(positive,Not(negative)),[ret(f(1))]),branch(And(negative,Not(positive)),[ret(f(-1))]),ret(f(0))],False)
 x,y=param("x",F),param("y",F);right,left,up,down,suppress=[param(n,B) for n in ("right","left","up","down","suppress")]
 dx,dy=call("control_button_axis",right,left),call("control_button_axis",up,down)
 rule("control_intent",INTENT,[("x",F),("y",F)]+[(n,B) for n in ("right","left","up","down","suppress")],
  [branch(suppress,[ret(sdk("stick_intent",f(0),f(0)))]),branch(Or(cmp("not_equal",dx,f(0)),cmp("not_equal",dy,f(0))),[ret(sdk("stick_intent",dx,dy))]),ret(sdk("stick_intent",x,y))])
 value=param("value",F)
 low=cprop("control_camera","pitch_min",-89,89);high=clamp(prop("control_camera","pitch_max"),low,89)
 rule("control_pitch",F,[("value",F)],[ret(clamp(value,low,high))],False)
 facing=param("facing",F)
 rule("control_camera_start",STATE,[("facing",F)],[ret(make(STATE,yaw=sdk("wrap_degrees",facing),pitch=call("control_pitch",prop("control_camera","initial_pitch")),
  goal_yaw=sdk("wrap_degrees",facing),goal_pitch=call("control_pitch",prop("control_camera","initial_pitch")),hold=f(0),blend=f(0),mode=u(0)))])
 free,target=param("free_aim",B),param("target",B)
 rule("control_camera_mode",U,[("free_aim",B),("target",B)],[branch(free,[ret(u(2))]),branch(target,[ret(u(1))]),ret(u(0))],False)
 s,a=param("state",STATE),param("input",CAMINPUT);dt=safe_dt(a.field("dt"));mode=call("control_camera_mode",a.field("free_aim"),a.field("target"))
 blend_target=param("active",B)
 rule("control_blend_target",F,[("active",B)],[branch(blend_target,[ret(f(1))]),ret(f(0))],False)
 blend=sdk("move_toward",clamp(s.field("blend"),0,1),call("control_blend_target",Or(a.field("free_aim"),a.field("target"))),cprop("control_camera","blend_rate",0,32)*dt)
 rule("control_camera_prepare",STATE,[("state",STATE),("input",CAMINPUT)],
  [ret(replace(STATE,s,yaw=sdk("wrap_degrees",s.field("yaw")),pitch=call("control_pitch",s.field("pitch")),
    goal_yaw=sdk("wrap_degrees",s.field("goal_yaw")),goal_pitch=call("control_pitch",s.field("goal_pitch")),
    hold=clamp(s.field("hold"),0,10),blend=blend,mode=mode))],False)
 mode=s.field("mode");look=a.field("look")
 hold=sdk("move_toward",clamp(s.field("hold"),0,10),f(0),dt)
 def manual_goal(start):
  yaw=s.field("yaw") if start else s.field("goal_yaw");pitch=s.field("pitch") if start else s.field("goal_pitch")
  return replace(STATE,s,hold=cprop("control_camera","manual_time",0,10),
   goal_yaw=sdk("wrap_degrees",yaw+look.field("x")*cprop("control_camera","orbit_rate",0,720)*dt),
   goal_pitch=call("control_pitch",pitch-look.field("y")*cprop("control_camera","pitch_rate",0,720)*dt))
 rule("control_camera_manual",STATE,[("state",STATE),("input",CAMINPUT)],
  [branch(And(gt(look.field("strength"),f(0)),cmp("not_equal",mode,u(1))),[
    branch(le(s.field("hold"),f(0)),[ret(manual_goal(True))]),ret(manual_goal(False))]),ret(replace(STATE,s,hold=hold))],False)
 rule("control_camera_goals",STATE,[("state",STATE),("input",CAMINPUT)],
  [branch(eq(mode,u(1)),[ret(replace(STATE,s,goal_yaw=a.field("target_yaw"),goal_pitch=call("control_pitch",a.field("target_pitch"))))]),
   branch(And(eq(mode,u(0)),le(s.field("hold"),f(0))),[
    branch(a.field("moving"),[ret(replace(STATE,s,goal_yaw=s.field("yaw")))]),ret(replace(STATE,s,goal_yaw=a.field("facing")))]),ret(s)],False)
 manual=param("manual",B)
 for name,normal,aim in (("control_response","normal_response","manual_response"),("control_yaw_cap","normal_cap","manual_cap")):
  bound=32 if name=="control_response" else 720
  rule(name,F,[("manual",B)],[branch(manual,[ret(cprop("control_camera",aim,0,bound))]),ret(cprop("control_camera",normal,0,bound))],False)
 dtp=param("dt",F);dt=safe_dt(dtp);manual=Or(cmp("not_equal",mode,u(0)),gt(s.field("hold"),f(0)))
 response=call("control_response",manual)*dt
 turn=sdk("delta_degrees",sdk("wrap_degrees",s.field("yaw")),sdk("wrap_degrees",s.field("goal_yaw")))*response/(f(1)+response)
 cap=call("control_yaw_cap",manual)*dt;yaw=s.field("yaw")+clamp(turn,f(0)-cap,cap)
 # The yaw snap belongs to the authored smoothing rule, not the shared rig.
 snap_from,snap_to=param("from",F),param("to",F);delta=sdk("delta_degrees",snap_from,snap_to)
 rule("control_yaw_snap",F,[("from",F),("to",F)],
  [branch(And(gt(delta,f(-.01)),lt(delta,f(.01))),[ret(snap_to)]),ret(snap_from)],False)
 yaw=sdk("wrap_degrees",call("control_yaw_snap",yaw,s.field("goal_yaw")))
 pitch_response=cprop("control_camera","pitch_response",0,32)*dt
 pitch=call("control_pitch",s.field("pitch")+(s.field("goal_pitch")-s.field("pitch"))*pitch_response/(f(1)+pitch_response))
 rule("control_camera_smooth",STATE,[("state",STATE),("dt",F)],[ret(replace(STATE,s,yaw=yaw,pitch=pitch))],False)
 rule("control_camera_step",STATE,[("state",STATE),("input",CAMINPUT)],
  [ret(call("control_camera_smooth",call("control_camera_goals",call("control_camera_manual",call("control_camera_prepare",s,a),a),a),a.field("dt")))])
 # Facing remains separate from camera yaw, preserving camera-relative input.
 intent=param("intent",INTENT);face=clamp(param("facing",F),-1440,1440);targetyaw=sdk("wrap_degrees",param("target_yaw",F));exists=param("target",B);owns=param("attack_owns",B);pressed=param("fire_pressed",B)
 angle=sdk("heading_degrees",intent.field("x"),intent.field("y"));step=cprop("control_movement","turn_rate",0,1440)*safe_dt(dtp)
 move_facing=sdk("move_toward_degrees",face,s.field("yaw")+angle,step)
 rule("control_facing",F,[("intent",INTENT),("state",STATE),("facing",F),("target_yaw",F),("target",B),("attack_owns",B),("fire_pressed",B),("dt",F)],
  [branch(And(eq(mode,u(0)),gt(intent.field("strength"),f(0))),[ret(move_facing)]),branch(And(eq(mode,u(1)),exists),[
    branch(And(gt(intent.field("strength"),f(0)),Not(owns)),[ret(move_facing)]),branch(pressed,[ret(targetyaw)]),ret(sdk("move_toward_degrees",face,targetyaw,step))]),
   branch(eq(mode,u(2)),[ret(sdk("move_toward_degrees",face,s.field("yaw"),step))]),ret(face)],False)
 speed=cprop("control_movement","speed",0,64);sy,cy=sin(s.field("yaw")),cos(s.field("yaw"))
 rule("control_movement_step",MOVE,[("intent",INTENT),("state",STATE),("facing",F),("target_yaw",F),("target",B),("attack_owns",B),("fire_pressed",B),("dt",F)],
  [ret(make(MOVE,x=(sy*intent.field("y")+cy*intent.field("x"))*speed,z=(cy*intent.field("y")-sy*intent.field("x"))*speed,
    facing=call("control_facing",intent,s,face,targetyaw,exists,owns,pressed,dtp)))])
 position,shoulder=param("position",VECTOR),param("shoulder_distance",F)
 blend=clamp(s.field("blend"),0,1);base=cprop("control_view","boom",.01,64)
 boom=base+(clamp(shoulder,cprop("control_view","shoulder_min",.01,64),64)-base)*blend
 height=cprop("control_view","height",-32,32)+(cprop("control_view","aim_height",-32,32)-cprop("control_view","height",-32,32))*blend
 side=cprop("control_view","shoulder_offset",-32,32)*blend;reach=boom*cos(s.field("pitch"));sy,cy=sin(s.field("yaw")),cos(s.field("yaw"))
 rule("control_view_proposal",PROPOSAL,[("position",VECTOR),("state",STATE),("shoulder_distance",F)],
  [ret(make(PROPOSAL,origin=vec(position.field("x"),position.field("y")+height,position.field("z")),
   offset=vec(cy*side-sy*reach,boom*sin(s.field("pitch")),f(0)-sy*side-cy*reach),distance=boom,yaw=s.field("yaw"),pitch=s.field("pitch")))])
 enabled=param("enabled",B);center,scale=param("center",F),param("scale",F)
 rule("control_focus_height",F,[("enabled",B),("center",F),("scale",F)],
  [branch(enabled,[ret(clamp(center,-32,32)*clamp(scale,-32,32))]),ret(cprop("control_aim","focus_height",-32,32))])
 origin,destination=param("origin",VECTOR),param("target",VECTOR);focus=param("focus",F)
 dx,dz=destination.field("x")-origin.field("x"),destination.field("z")-origin.field("z")
 horizontal=sdk("length2",clamp(dx,-32,32),clamp(dz,-32,32));vertical=origin.field("y")+cprop("control_view","height",-32,32)-(destination.field("y")+focus)
 yaw=sdk("heading_degrees",clamp(dx,-32,32),clamp(dz,-32,32))
 rule("control_target_angles",ANGLES,[("origin",VECTOR),("target",VECTOR),("focus",F)],
  [branch(eq(horizontal,f(0)),[branch(ge(destination.field("y"),origin.field("y")),[ret(make(ANGLES,yaw=yaw,pitch=call("control_pitch",f(89))))]),ret(make(ANGLES,yaw=yaw,pitch=call("control_pitch",f(-89))))]),
   ret(make(ANGLES,yaw=yaw,pitch=sdk("delta_degrees",f(0),sdk("heading_degrees",clamp(vertical,-128,128),horizontal))))])
 current,cam,aim=param("current",F),param("camera_pitch",F),param("target_pitch",F);free,target=param("free_aim",B),param("target",B)
 amin=cprop("control_aim","pitch_min",-89,89);amax=clamp(prop("control_aim","pitch_max"),amin,89)
 def toward(desired):return sdk("move_toward",current,clamp(desired,amin,amax),cprop("control_aim","pose_rate",0,1440)*safe_dt(dtp))
 rule("control_aim_step",F,[("current",F),("camera_pitch",F),("target_pitch",F),("dt",F),("free_aim",B),("target",B)],
  [branch(free,[ret(toward(cam))]),branch(target,[ret(toward(aim))]),ret(toward(f(0)))])
 intro,spin=param("intro",F),param("spin",F)
 initial=sdk("floor",cprop("control_marker","intro_frames",0,64));intro=sdk("move_toward",sdk("floor",clamp(intro,0,initial)),f(0),f(1))
 spin=sdk("floor",clamp(spin,0,63))+f(1);spin=spin-sdk("floor",spin/f(64))*f(64)
 radius=cprop("control_marker","base_radius",1,4096)+cprop("control_marker","step_radius",0,4096)*intro
 extent=cprop("control_marker","size",0,32)*clamp(radius/cprop("control_marker","base_radius",1,4096),0,64)
 tint=param("intro",F)
 rule("control_marker_brightness",F,[("intro",F)],
  [branch(gt(tint,f(0)),[ret(clamp((initial+f(1)-tint)*cprop("control_marker","brightness_step",0,255)*f(2),0,255))]),ret(clamp(cprop("control_marker","brightness_end",0,255)*f(2),0,255))],False)
 rule("control_marker_begin",U,[],[ret(sdk("truncate_unsigned",initial))])
 rule("control_marker_step",MARKER,[("intro",F),("spin",F)],
  [ret(make(MARKER,intro=intro,spin=spin,size=extent,tint=sdk("truncate_unsigned",call("control_marker_brightness",intro)),average=gt(intro,f(0))))])
 bones=param("bones",U)
 rule("control_rig",RIG,[("bones",U)],[branch(eq(bones,u(13)),[ret(make(RIG,bone=i(2),stop_a=i(6),stop_b=i(7)))]),ret(make(RIG,bone=i(-1),stop_a=i(-1),stop_b=i(-1)))])
 state,playing=param("state",U),param("playing",U);ground,finished,jumped=param("grounded",B),param("finished",B),param("jumped",B);vy,speed=param("vertical_speed",F),param("speed",F)
 def arate(which):
  denominator=cprop("control_animation","walk_cycle" if which==1 else "run_cycle",.001,64)
  low=cprop("control_animation","rate_min",0,2);high=clamp(prop("control_animation","rate_max"),low,2)
  return clamp(clamp(speed,0,64)/denominator,low,high)
 rule("control_animation_rate",F,[("state",U),("speed",F)],
  [branch(eq(state,u(1)),[ret(arate(1))]),branch(eq(state,u(2)),[ret(arate(2))]),ret(f(0))],False)
 def anim(nextstate):
  locom=Or(eq(nextstate,u(1)),eq(nextstate,u(2)))
  return make(ANIM,state=nextstate,loop=le(nextstate,u(2)),locomotion=locom,switch_clip=cmp("not_equal",nextstate,playing),rate=call("control_animation_rate",nextstate,speed))
 rule("control_animation_select",ANIM,[("state",U),("playing",U),("grounded",B),("vertical_speed",F),("speed",F),("finished",B),("jumped",B)],
  [branch(jumped,[ret(anim(u(3)))]),branch(Not(ground),[branch(gt(vy,f(0)),[ret(anim(u(3)))]),ret(anim(u(4)))]),
   branch(Or(eq(state,u(3)),eq(state,u(4))),[ret(anim(u(5)))]),
   branch(Or(cmp("not_equal",state,u(5)),finished),[
    branch(lt(speed,cprop("control_animation","idle_speed",0,64)),[ret(anim(u(0)))]),
    branch(Or(gt(speed,cprop("control_animation","run_speed",0,64)),And(eq(state,u(2)),gt(speed,cprop("control_animation","keep_run_speed",0,64)))),[ret(anim(u(2)))]),ret(anim(u(1)))]),ret(anim(state))])
 phase,frames=param("phase",U),param("frames",U);rate=param("rate",F);valid=param("model_valid",B)
 current_phase,increment=param("phase",U),param("increment",U)
 rule("control_phase_add",U,[("phase",U),("increment",U)],
  [branch(gt(current_phase,u(0xffffffff)-increment),[ret(u(0xffffffff))]),ret(current_phase+increment)],False)
 bounded_frames=param("frames",U)
 rule("control_frame_count",U,[("frames",U)],[branch(gt(bounded_frames,u(262144)),[ret(u(262144))]),branch(le(bounded_frames,u(1)),[ret(u(2))]),ret(bounded_frames)],False)
 period=(call("control_frame_count",frames)-u(1))*u(8192)
 advanced=call("control_phase_add",phase,sdk("truncate_unsigned",clamp(rate,0,2)*safe_dt(dtp)*f(60)*f(4096)));wrapped=advanced-(advanced/period)*period
 rule("control_animation_clock",CLOCK,[("phase",U),("frames",U),("rate",F),("dt",F),("model_valid",B)],
  [branch(valid,[ret(make(CLOCK,phase=wrapped,ticks=wrapped/u(4096)))]),ret(make(CLOCK,phase=advanced,ticks=u(0)))])
 yaw,pitch=param("yaw",F),param("pitch",F)
 direction=vec(sin(yaw)*cos(pitch),f(0)-sin(pitch),cos(yaw)*cos(pitch))
 rule("control_direction",VECTOR,[("yaw",F),("pitch",F)],[ret(direction)])
 ray=cprop("control_aim","ray_distance",0,128)
 rule("control_aim_ray",VECTOR,[("yaw",F),("pitch",F)],[ret(vec(direction.field("x")*ray,direction.field("y")*ray,direction.field("z")*ray))])
 muzzle=param("muzzle",B)
 rule("control_aim_mask",U,[("muzzle",B)],[branch(muzzle,[ret(prop("control_aim","muzzle_mask"))]),ret(prop("control_aim","ray_mask"))])
 target_flag,mode_flag=param("target",B),param("mode",U)
 rule("control_shot_aim",U,[("target",B),("mode",U)],
  [branch(target_flag,[ret(u(1))]),branch(eq(mode_flag,u(2)),[ret(u(2))]),ret(u(0))])
 delta=param("delta",VECTOR)
 too_large=Or(Or(Or(lt(delta.field("x"),f(-64)),gt(delta.field("x"),f(64))),Or(lt(delta.field("y"),f(-64)),gt(delta.field("y"),f(64)))),Or(lt(delta.field("z"),f(-64)),gt(delta.field("z"),f(64))))
 length=sdk("length",delta);quantized=sdk("floor",length*f(1024))/f(1024)
 invalid=make(DIRECTION,value=vec(f(0),f(0),f(0)),valid=b(False))
 rule("control_direction_toward",DIRECTION,[("delta",VECTOR)],
  [branch(too_large,[ret(invalid)]),branch(ge(length,f(64)),[ret(invalid)]),
   branch(gt(quantized,f(0)),[ret(make(DIRECTION,value=vec(*[delta.field(a)/quantized for a in ("x","y","z")]),valid=b(True)))]),ret(make(DIRECTION,value=delta,valid=b(True)))])
 displacement=param("displacement",VECTOR)
 rule("control_motion_speed",F,[("displacement",VECTOR),("dt",F)],
  [branch(le(dtp,f(0)),[ret(f(0))]),ret(sdk("length2",clamp(displacement.field("x"),-64,64)/safe_dt(dtp),clamp(displacement.field("z"),-64,64)/safe_dt(dtp)))])


def cpp_type(ty):return ty["cpp_name"] if ty["kind"]=="record" else CPP_TYPES[ty["kind"]]
def expression(e,language):
 k,v=e.kind,e.values
 if k=="literal":
  value=v[0]
  if e.ty==B:return ("true" if value else "false")
  if language=="cpp":return f"epok::Fixed({float(value)!r})" if e.ty==F else str(value)+("u" if e.ty==U else "")
  return repr(float(value)) if e.ty==F else str(value)
 if k in ("param","local"):return v[0]
 if k=="property":return ("self." if language=="lua" else "")+v[0]+"."+v[1]
 if k=="field":return "("+expression(v[0],language)+")."+v[1]
 if k=="not":return ("not " if language=="lua" else "!")+"("+expression(v[0],language)+")"
 if k=="binary":
  op={"add":"+","subtract":"-","multiply":"*","divide":"/","equal":"==","not_equal":"~=" if language=="lua" else "!=","less":"<","less_equal":"<=","greater":">","greater_equal":">=","and":"and" if language=="lua" else "&&","or":"or" if language=="lua" else "||"}[v[0]]
  return "("+expression(v[1],language)+" "+op+" "+expression(v[2],language)+")"
 if k in ("call","make","sdk"):
  name=v[0];arguments=", ".join(expression(x,language) for x in v[1:])
  if k=="sdk":prefix="epok.math." if language=="lua" else "epok::MathLibrary::"
  else:prefix="self:" if language=="lua" else ""
  return prefix+name+"("+arguments+")"
 raise AssertionError(k)
HEAVY_OPERATORS=("multiply","divide")
def heavy(e):
 """Work worth a local: rule and library calls, and fixed-point products."""
 if e.kind=="call":return True
 if e.kind=="sdk":return e.values[0]!="vector3"
 return e.kind=="binary" and e.values[0] in HEAVY_OPERATORS
def children(e):
 if e.kind=="field":return [e.values[0]]
 if e.kind=="not":return [e.values[0]]
 if e.kind=="binary":return [e.values[1],e.values[2]]
 if e.kind in ("call","make","sdk"):return list(e.values[1:])
 return []
def uses_expression(e,key):
 """(evaluated whenever e is, most evaluations) for the expression spelled key."""
 if expression(e,"cpp")==key:return True,1
 if e.kind=="binary" and e.values[0] in ("and","or"):
  # The right operand of a short circuit only runs on some paths.
  must,count=uses_expression(e.values[1],key);_,right=uses_expression(e.values[2],key)
  return must,count+right
 must,count=False,0
 for child in children(e):
  m,c=uses_expression(child,key);must|=m;count+=c
 return must,count
def uses_steps(steps,key):
 """(evaluated on every path through steps, most evaluations on one path)."""
 if not steps:return False,0
 step,rest=steps[0],list(steps[1:])
 if step[0]=="return":return uses_expression(step[1],key)
 if step[0]=="let":
  must,count=uses_expression(step[2],key);later_must,later=uses_steps(rest,key)
  return must or later_must,count+later
 must,count=uses_expression(step[1],key)
 yes_must,yes=uses_steps(list(step[2])+rest,key);no_must,no=uses_steps(list(step[3])+rest,key)
 return must or (yes_must and no_must),count+max(yes,no)
def substitute(e,key,local):
 if expression(e,"cpp")==key:return local
 if e.kind in ("literal","param","property","local"):return e
 if e.kind=="field":return E("field",e.ty,substitute(e.values[0],key,local),e.values[1])
 if e.kind=="not":return E("not",e.ty,substitute(e.values[0],key,local))
 if e.kind=="binary":return E("binary",e.ty,e.values[0],substitute(e.values[1],key,local),substitute(e.values[2],key,local))
 return E(e.kind,e.ty,e.values[0],*[substitute(x,key,local) for x in e.values[1:]])
def substitute_steps(steps,key,local):
 result=[]
 for step in steps:
  if step[0]=="return":result.append(("return",substitute(step[1],key,local)))
  elif step[0]=="let":result.append(("let",step[1],substitute(step[2],key,local)))
  else:result.append(("if",substitute(step[1],key,local),substitute_steps(step[2],key,local),substitute_steps(step[3],key,local)))
 return result
def heavy_expressions(steps,found):
 def visit(e):
  if heavy(e):found.setdefault(expression(e,"cpp"),e)
  for child in children(e):visit(child)
 for step in steps:
  if step[0]=="return":visit(step[1])
  elif step[0]=="let":visit(step[2])
  else:
   visit(step[1]);heavy_expressions(step[2],found);heavy_expressions(step[3],found)
 return found
def local_name(e,serial):
 base=e.values[0] if e.kind in ("call","sdk") else {"multiply":"product","divide":"quotient"}[e.values[0]]
 return base.replace("control_","").replace("world_","")+"_"+str(serial)
def hoist(steps,serial=None):
 """Name repeated pure work once, keeping every guard that protected it.

 Rules are pure trees that share nodes, so a value used several times is spelled
 out at each use. A local is placed only where its expression is very busy: every
 path onward evaluates it, and some path evaluates it at least twice. Nothing a
 branch or a short circuit protected, such as a division, can run earlier.
 """
 serial=serial if serial is not None else [0]
 steps=list(steps)
 changed=True
 while changed:
  changed=False
  found=heavy_expressions(steps,{})
  for key in sorted(found,key=lambda text:(-len(text),text)):
   for index in range(len(steps)):
    must,count=uses_steps(steps[index:],key)
    if must and count>=2:
     serial[0]+=1;e=found[key];name=local_name(e,serial[0])
     steps[index:]=[("let",name,e)]+substitute_steps(steps[index:],key,E("local",e.ty,name))
     changed=True;break
   if changed:break
 return [("if",s[1],hoist(s[2],serial),hoist(s[3],serial)) if s[0]=="if" else s for s in steps]
def body(steps,language,indent="    "):
 lines=[]
 for step in steps:
  if step[0]=="return":lines.append(indent+"return "+expression(step[1],language)+(";" if language=="cpp" else ""))
  elif step[0]=="let":
   lines.append(indent+("const auto "+step[1]+"="+expression(step[2],"cpp")+";" if language=="cpp" else "local "+step[1]+" = "+expression(step[2],"lua")))
  else:
   _,test,yes,no=step
   if language=="cpp":
    lines.append(indent+"if("+expression(test,language)+"){");lines+=body(yes,language,indent+"    ");lines.append(indent+"}")
    if no:lines[-1]+="else{";lines+=body(no,language,indent+"    ");lines.append(indent+"}")
   else:
    lines.append(indent+"if "+expression(test,language)+" then");lines+=body(yes,language,indent+"    ")
    if no:lines.append(indent+"else");lines+=body(no,language,indent+"    ")
    lines.append(indent+"end")
 return lines


def lua_methods(class_name):
 """Keep nominal record helpers inside their typed reflected overrides.

 Lua's authored annotation vocabulary intentionally has no project record
 names. These pure helper algorithms therefore expand into ordinary local
 Lua statements; their records still come from the inherited copy factories.
 C++ and Blueprint declarations and implementations remain unchanged.
 """
 # Authored Lua helper declarations have impure metadata. Expand every private
 # pure algorithm, including scalars, inside the inherited typed pure events.
 import re
 inline={name for name,r in RULES.items() if not r["virtual"]}
 def lua_type(t):return {"fixed":"Fixed","uint32":"UInt32","int32":"Int32","bool":"Bool"}[t["kind"]]
 def zero(ty):
  if ty==F:return "0.0"
  if ty==U:return "epok.math.truncate_unsigned(0.0)"
  if ty==I:return "epok.to_int(0.0)"
  if ty==B:return "false"
  if ty==VECTOR:return "epok.math.vector3(0.0, 0.0, 0.0)"
  factory=next(n for n,t in FACTORIES.items() if t==ty)
  return "self:"+factory+"("+", ".join(zero(p["value_type"]) for p in ty["fields"])+")"
 class Emitter:
  def __init__(self):self.serial=0
  def expr(self,e,env,lines,indent,cache):
   k,v=e.kind,e.values
   if k=="param":return env.get(v[0],v[0])
   if k=="local":return env.get(v[0],v[0])
   if k in ("literal","property"):return expression(e,"lua")
   if k=="field":return "("+self.expr(v[0],env,lines,indent,cache)+")."+v[1]
   if k=="not":return "not ("+self.expr(v[0],env,lines,indent,cache)+")"
   if k=="binary":
    if v[0] in ("equal","not_equal","less","less_equal","greater","greater_equal") and v[1].kind==v[2].kind=="literal":
     a,c=v[1].values[0],v[2].values[0]
     value={"equal":a==c,"not_equal":a!=c,"less":a<c,"less_equal":a<=c,"greater":a>c,"greater_equal":a>=c}[v[0]]
     return "true" if value else "false"
    op={"add":"+","subtract":"-","multiply":"*","divide":"/","equal":"==","not_equal":"~=","less":"<","less_equal":"<=","greater":">","greater_equal":">=","and":"and","or":"or"}[v[0]]
    left=self.expr(v[1],env,lines,indent,cache);right=self.expr(v[2],env,lines,indent,cache)
    if v[0] in ("equal","not_equal","less","less_equal","greater","greater_equal") and all(re.fullmatch(r'-?[0-9]+(?:\.[0-9]+)?',x) for x in [left,right]):
     a,c=map(float,[left,right]);value={"equal":a==c,"not_equal":a!=c,"less":a<c,"less_equal":a<=c,"greater":a>c,"greater_equal":a>=c}[v[0]]
     return "true" if value else "false"
    return "("+left+" "+op+" "+right+")"
   assert k in ("call","make","sdk"),k
   if k=="call" and v[0] in inline:
    if id(e) in cache:return cache[id(e)]
    args=[self.expr(x,env,lines,indent,cache) for x in v[1:]]
    self.serial+=1;local="rule"+str(self.serial)
    target=RULES[v[0]]
    arguments=dict(zip((n for n,_ in target["parameters"]),args))
    if len(target["body"])==1 and target["body"][0][0]=="return":
     value=self.expr(target["body"][0][1],arguments,lines,indent,{})
     if target["body"][0][1].kind=="literal" and e.ty==U:value="epok.math.truncate_unsigned("+str(float(target["body"][0][1].values[0]))+")"
     if target["body"][0][1].kind=="literal" and e.ty==I:value="epok.to_int("+str(float(target["body"][0][1].values[0]))+")"
     lines.append(indent+"local "+local+" = "+value)
    else:
     lines.append(indent+"local "+local+" = "+zero(e.ty))
     self.steps(target["body"],arguments,lines,indent,{},local)
    cache[id(e)]=local
    return local
   prefix="epok.math." if k=="sdk" else "self:"
   return prefix+v[0]+"("+", ".join(self.expr(x,env,lines,indent,cache) for x in v[1:])+")"
  def steps(self,steps,env,lines,indent,cache,destination=None):
   for index,step in enumerate(steps):
    if step[0]=="return":
     value=self.expr(step[1],env,lines,indent,cache)
     lines.append(indent+(destination+" = " if destination else "return ")+value)
     return
    if step[0]=="let":
     self.serial+=1;local=step[1]+"_"+str(self.serial)
     lines.append(indent+"local "+local+" = "+self.expr(step[2],env,lines,indent,cache))
     env={**env,step[1]:local}
     continue
    _,test,yes,no=step
    condition=self.expr(test,env,lines,indent,cache)
    lines.append(indent+"if "+condition+" then")
    if destination:
     # The authored helper returns early. Carry the remaining statements into
     # each branch instead of evaluating branches that did not run originally.
     tail=steps[index+1:]
     self.steps(list(yes)+list(tail),env,lines,indent+"    ",dict(cache),destination)
     lines.append(indent+"else")
     self.steps(list(no)+list(tail),env,lines,indent+"    ",dict(cache),destination)
     lines.append(indent+"end")
     return
    self.steps(yes,env,lines,indent+"    ",dict(cache))
    if no:
     lines.append(indent+"else")
     self.steps(no,env,lines,indent+"    ",dict(cache))
    lines.append(indent+"end")
 methods=[]
 for name,r in RULES.items():
  if name in inline:continue
  annotations="---@override\n" if r["virtual"] else "".join(f"---@param {p} {lua_type(t)}\n" for p,t in r["parameters"])+f'---@return {lua_type(r["returns"])}\n'
  lines=[];Emitter().steps(hoist(r["body"]),{},lines,"    ",{})
  methods.append(annotations+f'function {class_name}:{name}('+", ".join(p for p,_ in r["parameters"])+")\n"+"\n".join(lines)+"\nend\n")
 return "\n".join(methods)


def refresh_cpp(path):
 """Rewrite each rule method's body in place; signatures stay authored."""
 import re
 text=path.read_text(encoding="utf-8")
 for name,r in RULES.items():
  found=re.search(r"^    [\w:<>]+ "+name+r"\([^\n]*\{\n",text,re.M)
  assert found,name
  depth=1;index=found.end()
  while depth:
   depth+=text[index]=="{";depth-=text[index]=="}";index+=1
  text=text[:found.end()]+"\n".join(body(hoist(r["body"]),"cpp","        "))+"\n    "+text[index-1:]
 path.write_text(text,encoding="utf-8",newline="")


def refresh_lua(path,class_name):
 """Regenerate this module's authored Lua section without touching its prefix."""
 text=path.read_text(encoding="utf-8")
 first=next(iter(RULES))
 marker="-- Generated authored rules: "+class_name
 start=text.index(marker) if marker in text else text.rfind("\n\n",0,text.index(f"function {class_name}:{first}("))+2
 text=text[:start]+marker+"\n"+lua_methods(class_name)+f"\nreturn {class_name}\n"
 if class_name=="DungeonPlayerCombat":text=inline_feedback_helpers(text)
 path.write_text(text,encoding="utf-8")


def inline_feedback_helpers(text):
 """Expand the earlier UInt32 clamp without converting its32-bit arithmetic."""
 import re
 text=re.sub(r'self:bounded_size\(([^()]*)\)',r'epok.math.clamp(\1, 0.0, 32.0)',text)
 # Hoist generated locals before the whole statement, including authored
 # constructor calls whose argument list spans several physical source lines.
 statements=[];pending='';depth=0
 for line in text.splitlines():
  if pending:pending+=' '+line.strip()
  else:pending=line
  if not line.lstrip().startswith('--'):depth+=line.count('(')-line.count(')')
  if depth==0:statements.append(pending);pending=''
 assert not pending and depth==0
 result=[];serial=0
 for line in statements:
  prelude=[]
  while 'self:bounded(' in line:
   start=line.index('self:bounded(');cursor=start+len('self:bounded(');depth=1;begin=cursor;args=[]
   while depth:
    char=line[cursor]
    if char=='(':depth+=1
    elif char==')':
     depth-=1
     if not depth:args.append(line[begin:cursor].strip())
    elif char==',' and depth==1:args.append(line[begin:cursor].strip());begin=cursor+1
    cursor+=1
   assert len(args)==3,args
   value,minimum,maximum=args;serial+=1;local='bounded'+str(serial);indent=line[:len(line)-len(line.lstrip())]
   prelude+=[indent+'local '+local+' = '+value,indent+'if '+local+' < '+minimum+' then',indent+'    '+local+' = '+minimum,indent+'else',indent+'    if '+local+' > '+maximum+' then',indent+'        '+local+' = '+maximum,indent+'    end',indent+'end']
   line=line[:start]+local+line[cursor:]
  result+=prelude+[line]
 return '\n'.join(result)+'\n'


def extend_blueprint(Graph,blueprint):
 define()
 def link(node,pin="value"):return {"kind":"link","node":node["id"],"pin":pin}
 def literal_node(value,ty):return {"kind":"literal","value_type":ty,"value":value}
 split=blueprint["layout"]["split_pins"]
 for name,r in RULES.items():
  g=Graph(name,r["returns"],r["parameters"]);cache={}
  def graph_expr(e):
   if id(e) in cache:return cache[id(e)]
   k,v=e.kind,e.values
   if k=="literal":out=literal_node(v[0],e.ty)
   elif k=="param":out=g.p(v[0])
   elif k=="property":out=g.read(identity(v[0]),v[1])
   elif k=="field":
    base=graph_expr(v[0]);assert base["kind"]=="link";split.append(base["node"]+":out:"+base["pin"])
    out={**base,"pin":base["pin"]+"."+v[1]}
   elif k=="binary":out=g.op(v[0],graph_expr(v[1]),graph_expr(v[2]))
   elif k=="not":out=g.neg(graph_expr(v[0]))
   elif k=="sdk":out=g.operation(SDK[v[0]][0],*[graph_expr(a) for a in v[1:]])
   elif k in ("call","make"):
    target=v[0]
    if k=="call":parameters=RULES[target]["parameters"];fn=identity("graph/"+target)
    else:parameters=[(x["name"],x["value_type"]) for x in FACTORIES[target]["fields"]];fn=identity(target)
    out=link(g.node({"kind":"call","function":fn},{n:graph_expr(a) for (n,_),a in zip(parameters,v[1:])}))
   else:raise AssertionError(k)
   cache[id(e)]=out;return out
  def graph_steps(steps):
   result=[]
   for s in steps:
    if s[0]=="return":result.append(g.ret(graph_expr(s[1])))
    else:result.append(g.condition(graph_expr(s[1]),graph_steps(s[2]),graph_steps(s[3])))
   return result
  graph=g.finish(graph_steps(r["body"]))
  if r["virtual"]:graph["override_id"]=identity(name)
  blueprint["functions"].append(graph);split.extend(g.split)
 blueprint["layout"]["positions"]={node["id"]:[float(i%6*290),float(i//6*180)] for graph in blueprint["functions"] for i,node in enumerate(graph["nodes"])}
 blueprint["layout"]["split_pins"]=sorted(set(split))
 return blueprint
