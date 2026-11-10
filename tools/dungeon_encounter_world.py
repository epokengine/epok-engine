"""Authored encounter/world rules for the ordinary project Blueprint generator."""
import importlib.util
import json
from pathlib import Path
import uuid

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("typed_world_rules",Path(__file__).with_name("dungeon_player_controls.py"))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
d.identity=lambda n:str(uuid.uuid5(d.NS,"DungeonEncounter/"+n))
d.RULES={};d.FACTORIES={}
F,U,I,B,V=d.F,d.U,d.I,d.B,d.V
VECTOR=d.VECTOR
COLOR=d.record("DungeonWorldColor",[(n,U) for n in ("red","green","blue")])
HIT=d.record("DungeonWorldHit",[(n,B) for n in ("defeated","disable_collider","death_clip","victory","reset_projectiles","explosion","hud_dirty")]+[("reward",U),("sound",U)])
DEATH=d.record("DungeonWorldDeath",[(n,B) for n in ("defeat","clear_aim","reset_projectiles")])
DOOR=d.record("DungeonWorldDoor",[("progress",F),("opening",B),("moved",B),("opened",B),("locked_tint",B),("offset",F),("color",COLOR)])
CLIP=d.record("DungeonWorldClip",[("clip",I),("play",B),("loop",B),("charge_cue",B)])
DIRECTION=d.record("DungeonWorldDirection",[("value",VECTOR),("valid",B)])
MOTION=d.record("DungeonWorldMotion",[("delta",VECTOR),("yaw",F),("patrol_yaw",F),("move",B),("turn",B),("patrol_turn",B),("valid",B)])
RECORDS=[COLOR,HIT,DEATH,DOOR,CLIP,DIRECTION,MOTION]
NEW_CONFIG={
 "world_damage":("DungeonWorldDamageTuning",[("guard_reward",U,1),("boss_reward",U,0),("player_defeat",B,True),("player_reset",B,True),("boss_victory",B,True),("boss_reset",B,True),("hide_victorious_boss",B,True)]),
 "guard_colors":("DungeonGuardColors",[(n,U,v) for n,v in zip(("normal_red","normal_green","normal_blue","windup_red","windup_green","windup_blue","hit_red","hit_green","hit_blue"),(100,210,255,255,180,85,255,110,110))]),
 "boss_colors":("DungeonBossColors",[(n,U,v) for n,v in zip(("normal_red","normal_green","normal_blue","fan_red","fan_green","fan_blue","windup_red","windup_green","windup_blue","hit_red","hit_green","hit_blue"),(255,255,255,255,190,150,255,150,55,255,90,90))]),
 "enemy_shot_color":("DungeonEnemyShotColor",[("red",U,255),("green",U,100),("blue",U,50)]),
 "world_doors":("DungeonWorldDoorTuning",[("open_radius",F,5.0),("safety_radius",F,2.0),("speed",F,1.4),("closed_offset",F,.75),("travel",F,1.7),("locked_red",U,255),("locked_green",U,130),("locked_blue",U,110)]),
 "world_pickups":("DungeonWorldPickupTuning",[("radius",F,1.0),("maximum_y",F,2.2),("heal",U,35),("bob_rate",F,120.0),("bob_extent",F,.12),("left_x",F,-8.0),("right_x",F,8.0),("minimum_z",F,32.0),("maximum_z",F,55.0)]),
 "world_animation":("DungeonWorldAnimationTuning",[("idle_clip",I,0),("move_clip",I,1),("death_clip",I,2),("loop",B,True)]),
 "world_motion":("DungeonWorldMotionTuning",[("ground_step",F,-.05),("boss_interval",F,1/30)]),
}
d.CONFIG={**NEW_CONFIG,"movement":("DungeonEncounterMovement",[(n,F,v) for n,v in (("activation_radius",14),("retention_radius",22),("sight_radius",11),("guard_range",5),("patrol_extent",1.25),("patrol_arrival",.2),("chase_speed",1.05),("patrol_speed",.6))])}

def make(ty,**fields):
 name="make_world_"+ty["cpp_name"];d.FACTORIES[name]=ty
 return d.E("make",ty,name,*[fields[p["name"]] for p in ty["fields"]])
d.make=make
p,prop,ret,branch=d.param,d.prop,d.ret,d.branch
f,u,i,b=d.f,d.u,d.i,d.b
eq,lt,le,gt,ge,And,Or,Not=d.eq,d.lt,d.le,d.gt,d.ge,d.And,d.Or,d.Not
call,sdk,clamp,vec=d.call,d.sdk,d.clamp,d.vec
rule=d.rule

def define():
 value=p("value",U)
 rule("world_channel",U,[("value",U)],[branch(gt(value,u(255)),[ret(u(255))]),ret(value)],False)
 def color(group,prefix=""):
  return make(COLOR,**{n:call("world_channel",prop(group,prefix+n)) for n in ("red","green","blue")})
 boss,alive,boss_fight,registered,defeated=[p(n,B) for n in ("boss","alive","boss_fight","registered","defeated")]
 health=p("health",U);dead=p("dead_time",F)
 rule("world_damage_eligible",B,[("boss",B),("alive",B),("boss_fight",B),("health",U),("registered",B),("dead_time",F),("defeated",B)],
  [branch(Not(alive),[ret(b(False))]),branch(boss,[ret(And(boss_fight,gt(health,u(0))))]),ret(And(And(registered,le(dead,f(0))),Not(defeated)))])
 base,bonus=p("base",U),p("bonus",U)
 rule("world_damage_base",U,[("base",U)],[branch(gt(base,u(0x7fffffff)),[ret(u(0x7fffffff))]),ret(base)],False)
 bounded=call("world_damage_base",base)
 rule("world_damage_total",U,[("base",U),("bonus",U)],
  [branch(gt(bonus,u(0x7fffffff)-bounded),[ret(u(0x7fffffff))]),ret(bounded+bonus)])
 amount=p("amount",U);count=p("count",U)
 rule("world_reward_count",U,[("count",U),("amount",U)],
  [branch(Or(gt(count,u(0x7fffffff)),gt(amount,u(0x7fffffff)-count)),[ret(u(0x7fffffff))]),ret(count+amount)])
 depleted=p("depleted",B)
 remaining=p("remaining",U)
 rule("world_hit_sound",U,[("boss",B),("remaining",U)],
  [branch(gt(remaining,u(0)),[ret(u(2))]),branch(boss,[ret(u(8))]),ret(u(4))],False)
 rule("world_hit_result",HIT,[("boss",B),("depleted",B),("remaining",U)],
  [branch(boss,[branch(depleted,[ret(make(HIT,defeated=b(False),disable_collider=b(True),death_clip=b(True),victory=prop("world_damage","boss_victory"),reset_projectiles=prop("world_damage","boss_reset"),explosion=b(True),hud_dirty=b(True),reward=prop("world_damage","boss_reward"),sound=u(8)))]),
    ret(make(HIT,defeated=b(False),disable_collider=b(False),death_clip=b(False),victory=b(False),reset_projectiles=b(False),explosion=b(False),hud_dirty=b(True),reward=u(0),sound=call("world_hit_sound",boss,remaining)))]),
   branch(depleted,[ret(make(HIT,defeated=b(True),disable_collider=b(True),death_clip=b(True),victory=b(False),reset_projectiles=b(False),explosion=b(False),hud_dirty=b(True),reward=prop("world_damage","guard_reward"),sound=u(4)))]),
   ret(make(HIT,defeated=b(False),disable_collider=b(False),death_clip=b(False),victory=b(False),reset_projectiles=b(False),explosion=b(False),hud_dirty=b(False),reward=u(0),sound=call("world_hit_sound",boss,remaining)))])
 rule("world_player_death",DEATH,[("depleted",B)],
  [branch(depleted,[ret(make(DEATH,defeat=prop("world_damage","player_defeat"),clear_aim=b(True),reset_projectiles=prop("world_damage","player_reset")))]),ret(make(DEATH,defeat=b(False),clear_aim=b(False),reset_projectiles=b(False)))])
 state=p("state",U);playing=p("playing",B)
 rule("world_hide_boss",B,[("state",U),("playing",B)],[ret(And(And(eq(state,u(2)),Not(playing)),prop("world_damage","hide_victorious_boss")))])
 flash,windup=p("flash",F),p("windup",F);fan=p("fan",B)
 rule("world_enemy_color",COLOR,[("boss",B),("flash",F),("windup",F),("fan",B)],
  [branch(boss,[branch(gt(flash,f(0)),[ret(color("boss_colors","hit_"))]),branch(gt(windup,f(0)),[ret(color("boss_colors","windup_"))]),branch(fan,[ret(color("boss_colors","fan_"))]),ret(color("boss_colors","normal_"))]),
   branch(gt(flash,f(0)),[ret(color("guard_colors","hit_"))]),branch(gt(windup,f(0)),[ret(color("guard_colors","windup_"))]),ret(color("guard_colors","normal_"))])
 rule("world_shot_color",COLOR,[],[ret(color("enemy_shot_color"))])
 clip=p("clip",I)
 rule("world_clip_index",I,[("clip",I)],[branch(lt(clip,i(0)),[ret(i(0))]),branch(gt(clip,i(32767)),[ret(i(32767))]),ret(clip)],False)
 rule("world_death_animation",CLIP,[],[ret(make(CLIP,clip=call("world_clip_index",prop("world_animation","death_clip")),play=b(True),loop=b(False),charge_cue=b(False)))])
 action,current=p("action",U),p("current",I)
 idle=call("world_clip_index",prop("world_animation","idle_clip"));move=call("world_clip_index",prop("world_animation","move_clip"))
 def clip_result(value,play,cue=b(False)):return make(CLIP,clip=value,play=play,loop=prop("world_animation","loop"),charge_cue=cue)
 rule("world_enemy_animation",CLIP,[("boss",B),("action",U),("current",I)],
  [branch(eq(action,u(4)),[ret(clip_result(idle,b(True),boss))]),branch(Or(eq(action,u(2)),And(Not(boss),eq(action,u(3)))),[ret(clip_result(move,d.cmp("not_equal",current,move)))]),
   branch(eq(action,u(1)),[ret(clip_result(idle,d.cmp("not_equal",current,idle)))]),ret(clip_result(i(-1),b(False)))])
 # Door decisions are project rules; native code only supplies proximity queries
 # and applies the returned progress/offset/color through scene mechanisms.
 index=p("index",U);locked,key,near_open,near_safety=[p(n,B) for n in ("locked","has_key","near_open","near_safety")]
 before=p("progress",F);dt=p("dt",F)
 rule("world_door_open",B,[("index",U),("locked",B),("has_key",B),("state",U),("progress",F),("near_open",B),("near_safety",B)],
  [branch(And(And(eq(index,u(3)),eq(state,u(1))),And(gt(before,f(0)),near_safety)),[ret(b(True))]),
   branch(And(eq(index,u(3)),eq(state,u(1))),[ret(b(False))]),
   branch(And(Or(Or(Not(locked),key),eq(state,u(2))),near_open),[ret(b(True))]),ret(And(gt(before,f(0)),near_safety))])
 radius_kind=p("safety",B)
 rule("world_door_radius",F,[("safety",B)],[branch(radius_kind,[ret(d.cprop("world_doors","safety_radius",0,64))]),ret(d.cprop("world_doors","open_radius",0,64))])
 opening=p("opening",B)
 rule("world_door_progress",F,[("progress",F),("opening",B),("dt",F)],
  [branch(opening,[ret(clamp(clamp(before,0,1)+clamp(dt,0,1)*d.cprop("world_doors","speed",0,16),0,1))]),ret(clamp(clamp(before,0,1)-clamp(dt,0,1)*d.cprop("world_doors","speed",0,16),0,1))],False)
 params=[("index",U),("locked",B),("has_key",B),("state",U),("progress",F),("near_open",B),("near_safety",B),("dt",F)]
 opening=call("world_door_open",index,locked,key,state,before,near_open,near_safety)
 progress=call("world_door_progress",before,opening,dt);eased=progress*progress*(f(3)-f(2)*progress)
 tint_flag=And(eq(index,u(3)),Not(key));offset=d.cprop("world_doors","closed_offset",0,8)+eased*d.cprop("world_doors","travel",0,8)
 common=dict(progress=progress,opening=opening,moved=d.cmp("not_equal",before,progress),opened=And(lt(before,f(1)),eq(progress,f(1))),locked_tint=tint_flag,offset=offset)
 rule("world_door_step",DOOR,params,[branch(tint_flag,[ret(make(DOOR,**common,color=make(COLOR,red=call("world_channel",prop("world_doors","locked_red")),green=call("world_channel",prop("world_doors","locked_green")),blue=call("world_channel",prop("world_doors","locked_blue")))))]),ret(make(DOOR,**common,color=make(COLOR,red=u(255),green=u(255),blue=u(255))))])
 kind=p("kind",U);position=p("position",VECTOR)
 rule("world_pickup_visible",B,[("kind",U),("position",VECTOR)],
  [branch(Or(eq(kind,u(0)),eq(kind,u(2))),[ret(lt(position.field("x"),d.cprop("world_pickups","left_x",-4096,4096)))]),
   branch(eq(kind,u(1)),[ret(gt(position.field("x"),d.cprop("world_pickups","right_x",-4096,4096)))]),
   ret(And(gt(position.field("z"),d.cprop("world_pickups","minimum_z",-4096,4096)),lt(position.field("z"),d.cprop("world_pickups","maximum_z",-4096,4096))))])
 can_collect,nearby=p("can_collect",B),p("nearby",B);height=p("height",F)
 rule("world_pickup_eligible",B,[("can_collect",B),("nearby",B),("height",F)],
  [ret(And(And(can_collect,nearby),lt(height,d.cprop("world_pickups","maximum_y",-32,32))))])
 rule("world_pickup_radius",F,[],[ret(d.cprop("world_pickups","radius",0,64))])
 rule("world_pickup_heal",U,[],[ret(call("world_damage_base",prop("world_pickups","heal")))])
 elapsed=p("elapsed",F)
 rule("world_pickup_bob",F,[("elapsed",F)],
  [ret(d.sin(sdk("wrap_degrees",elapsed)*sdk("floor",d.cprop("world_pickups","bob_rate",0,720)))*d.cprop("world_pickups","bob_extent",0,8))])
 # Quantized local direction remains a chosen-language rule, never a Base
 # implementation. Reject outside its legacy64-unit range before scaling.
 delta=p("delta",VECTOR);bad=b(False)
 for axis in ("x","y","z"):bad=Or(bad,Or(lt(delta.field(axis),f(-64)),gt(delta.field(axis),f(64))))
 length=sdk("length",delta);quantum=sdk("floor",length*f(1024))/f(1024)
 invalid=make(DIRECTION,value=vec(f(0),f(0),f(0)),valid=b(False))
 rule("world_direction",DIRECTION,[("delta",VECTOR)],
  [branch(bad,[ret(invalid)]),branch(ge(length,f(64)),[ret(invalid)]),branch(gt(quantum,f(0)),[ret(make(DIRECTION,value=vec(*[delta.field(a)/quantum for a in ("x","y","z")]),valid=b(True)))]),ret(make(DIRECTION,value=delta,valid=b(True)))],False)
 rule("world_aim_direction",DIRECTION,[("delta",VECTOR)],[ret(call("world_direction",delta))])
 origin,player,home=p("origin",VECTOR),p("player",VECTOR),p("home",VECTOR)
 side=p("patrol_side",B);close,seen=p("close",B),p("seen",B);boss_speed=p("boss_speed",F)
 extent=d.cprop("movement","patrol_extent",0,16)
 rule("world_patrol_x",F,[("home",F),("side",B)],[branch(p("side",B),[ret(p("home",F)+extent)]),ret(p("home",F)-extent)],False)
 goal=vec(call("world_patrol_x",home.field("x"),side),home.field("y"),home.field("z"))
 rule("world_patrol_goal",VECTOR,[("home",VECTOR),("patrol_side",B)],[ret(goal)])
 chase=vec(player.field("x"),origin.field("y"),player.field("z"))
 rule("world_motion_goal",VECTOR,[("boss",B),("action",U),("origin",VECTOR),("player",VECTOR),("home",VECTOR),("patrol_side",B)],
  [branch(And(Not(boss),eq(action,u(3))),[ret(goal)]),ret(chase)],False)
 rule("world_motion_speed",F,[("boss",B),("action",U),("boss_speed",F)],
  [branch(boss,[ret(clamp(boss_speed,0,16))]),branch(eq(action,u(3)),[ret(d.cprop("movement","patrol_speed",0,16))]),ret(d.cprop("movement","chase_speed",0,16))],False)
 dyaw=sdk("heading_degrees",clamp(player.field("x")-origin.field("x"),-64,64),clamp(player.field("z")-origin.field("z"),-64,64))
 turn=Or(boss,And(close,seen));goal=call("world_motion_goal",boss,action,origin,player,home,side)
 direction=call("world_direction",vec(*[goal.field(a)-origin.field(a) for a in ("x","y","z")]))
 moving=Or(eq(action,u(2)),And(Not(boss),eq(action,u(3))))
 speed=call("world_motion_speed",boss,action,boss_speed)
 motion_delta=vec(direction.field("value").field("x")*speed*clamp(dt,0,1),d.cprop("world_motion","ground_step",-1,0),direction.field("value").field("z")*speed*clamp(dt,0,1))
 def output(yaw,do_turn,move,valid,delta=vec(f(0),f(0),f(0)),patrol_yaw=f(0),patrol_turn=b(False)):
  return make(MOTION,delta=delta,yaw=yaw,patrol_yaw=patrol_yaw,move=move,turn=do_turn,patrol_turn=patrol_turn,valid=valid)
 rule("world_enemy_motion",MOTION,[("boss",B),("action",U),("patrol_side",B),("close",B),("seen",B),("origin",VECTOR),("player",VECTOR),("home",VECTOR),("dt",F),("boss_speed",F)],
  [branch(moving,[branch(Not(direction.field("valid")),[ret(output(dyaw,turn,b(False),b(False)))]),
    branch(And(Not(boss),eq(action,u(3))),[ret(output(dyaw,turn,b(True),b(True),motion_delta,sdk("heading_degrees",direction.field("value").field("x"),direction.field("value").field("z")),b(True)))]),
    ret(output(dyaw,turn,b(True),b(True),motion_delta))]),ret(output(dyaw,turn,b(False),b(True)))])
 rule("world_boss_due",B,[("elapsed",F)],[ret(ge(elapsed,d.cprop("world_motion","boss_interval",.000244140625,1)))])


def refresh_lua(path,class_name="DungeonEncounter"):
 define()
 d.refresh_lua(path,class_name)


def extend_blueprint(Graph,blueprint):
 define()
 RULES,FACTORIES,SDK,identity=d.RULES,d.FACTORIES,d.SDK,d.identity
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
   elif k=="sdk":out=link(g.node({"kind":"operation","operation":SDK[v[0]][0]}, {SDK[v[0]][0]+f":parameter:{i}":graph_expr(a) for i,a in enumerate(v[1:])}))
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
  graph["id"]=identity("graph/"+name)
  if r["virtual"]:graph["override_id"]=identity(name)
  blueprint["functions"].append(graph);split.extend(g.split)
 blueprint["layout"]["positions"]={node["id"]:[float(i%6*290),float(i//6*180)] for graph in blueprint["functions"] for i,node in enumerate(graph["nodes"])}
 blueprint["layout"]["split_pins"]=sorted(set(split))
 return blueprint
