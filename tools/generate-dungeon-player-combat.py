"""Generate editable project attack and target policy Blueprint graphs."""
import importlib.util
import json
from pathlib import Path
import uuid
from dungeon_player_controls import extend_blueprint, refresh_lua

ROOT = Path(__file__).resolve().parents[1]
NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "DungeonPlayerCombat/" + name))
F, U, I, B, V = ({"kind": name} for name in ("fixed", "uint32", "int32", "bool", "void"))
record = lambda name, fields: {"kind": "record", "cpp_name": name,
    "fields": [{"name": key, "value_type": value} for key, value in fields]}
STATE = record("DungeonAttackState", [("phase", U), ("attacking", B), ("shot_sent", B)])
STEP = record("DungeonAttackStep", [("state", STATE), ("clip", I), ("fire", B), ("invalidate_locomotion", B), ("starting", B)])
CHECK = record("DungeonTargetCheck", [("phase", U), ("keep", B), ("query_sight", B)])
DISTANCE = record("DungeonTargetDistance", [("high", U), ("low", U)])
VECTOR = record("epok::GameplayVector3", [(axis, F) for axis in ("x", "y", "z")])
SPAWN = record("epok::ProjectileSpawn3D", [("origin", VECTOR), ("velocity", VECTOR), ("lifetime", F), ("tag", U)])

TRACE = record("DungeonImpactTrace", [("mask",U),("ignore_player",B)])
SHOT_VISUAL = record("DungeonPlayerShotVisual", [("size",F),("red",U),("green",U),("blue",U)])
EFFECT_STEP = record("DungeonCombatEffectStep", [("age",U),("frame",U),("size",F),("playing",B)])
FEEDBACK = record("DungeonDamageFeedback", [("remaining_us",U),("enabled",B),("border_pixels",U),("red",U),("green",U),("blue",U)])
RUMBLE = record("DungeonDamageRumble", [("large",U),("small",B),("duration_ms",U)])

EVENTS = {name: identity(name) for name in ("attack", "blocks_movement", "target_input", "target_check",
    "candidate", "after_selected", "prefer_candidate", "projectile", "trace_settings", "impact", "shot_visual", "effect_step", "damage_feedback", "damage_rumble")}
spec = importlib.util.spec_from_file_location("combat_graph_base", ROOT / "tools/generate-dungeon-run-flow.py")
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
literal, link, parameter = base.literal, base.link, base.parameter
b = lambda value: literal(value, B)
u = lambda value: literal(value, U)
i = lambda value: literal(value, I)
f = lambda value: literal(float(value), F)


class Graph(base.Graph):
    def node(self, kind, inputs=None):
        node = {"id": identity(f"{self.name}/node/{len(self.nodes)}"),
            "kind": kind if isinstance(kind, dict) else {"kind": kind}, "inputs": inputs or {}, "outputs": {}}
        self.nodes.append(node)
        return node

    def p(self, name, field=""):
        if field:
            self.split.append(identity(self.name + "/entry") + ":out:" + name)
        return {"kind": "link", "node": identity(self.name + "/entry"), "pin": name + ("." + field if field else "")}

    def t(self, field):
        return self.read(identity("tuning"), field)

    def neg(self, value):
        return link(self.node("not", {"value": value}))

    def call(self, name, inputs=None):
        return self.node({"kind": "call", "function": identity("graph/" + name) if name in ("bounded", "bounded_size") else identity(name)}, inputs)

    def operation(self, operation, *values):
        return link(self.node({"kind": "operation", "operation": operation},
            {operation + f":parameter:{index}": value for index, value in enumerate(values)}))

    def state(self, phase=None, active=None, sent=None):
        return link(self.call("make_attack_state", {"phase": self.p("state", "phase") if phase is None else phase,
            "attacking": self.p("state", "attacking") if active is None else active,
            "shot_sent": self.p("state", "shot_sent") if sent is None else sent}))

    def step(self, state=None, clip=-1, fire=False, invalid=False, starting=False):
        return link(self.call("make_attack_step", {"state": self.p("state") if state is None else state,
            "clip": i(clip), "fire": b(fire), "invalidate_locomotion": b(invalid), "starting": b(starting)}))

    def check(self, phase, keep, query):
        return link(self.call("make_target_check", {"phase": phase, "keep": b(keep), "query_sight": b(query)}))

    def finish(self, steps):
        graph = super().finish(steps)
        entry = self.nodes[-1]
        old = entry["id"]
        entry["id"] = identity(self.name + "/entry")
        graph["entry"] = entry["id"]
        for node in self.nodes:
            for value in node["inputs"].values():
                if value.get("kind") == "link" and value["node"] == old:
                    value["node"] = entry["id"]
        graph["id"] = identity("graph/" + self.name)
        if self.name in EVENTS:
            graph["override_id"] = EVENTS[self.name]
        else:
            graph.pop("override_id", None)
        return graph


def build():
    graphs, split = [], []

    def add(g, steps):
        graphs.append(g.finish(steps))
        split.extend(g.split)

    g = Graph("attack", STEP, (("state", STATE), ("grounded", B), ("jumped", B), ("held", B), ("clip_playing", B), ("clips_ready", B)))
    can_start = g.all(g.p("grounded"), g.neg(g.p("jumped")), g.p("held"), g.p("clips_ready"))

    def finish_attack(invalid):
        return g.condition(can_start,
            [g.ret(g.step(g.state(u(0), b(True), b(False)), 0, invalid=invalid, starting=True))],
            [g.ret(g.step(g.state(active=b(False)), invalid=invalid))])

    add(g, [g.condition(g.p("state", "attacking"), [
        g.condition(g.any(g.p("jumped"), g.neg(g.p("grounded"))),
            [g.ret(g.step(g.state(active=b(False)), invalid=True))]),
        g.condition(g.neg(g.p("clip_playing")), [
            g.condition(g.op("equal", g.p("state", "phase"), u(0)),
                [g.ret(g.step(g.state(u(1), b(True), b(True)), 1, fire=True))]),
            g.condition(g.op("equal", g.p("state", "phase"), u(1)), [
                g.condition(g.p("held"), [g.ret(g.step(g.state(phase=u(0)), 0))],
                    [g.ret(g.step(g.state(phase=u(2)), 2))])]),
            finish_attack(True)]),
        g.condition(g.all(g.op("equal", g.p("state", "phase"), u(1)), g.neg(g.p("state", "shot_sent"))),
            [g.ret(g.step(g.state(sent=b(True)), fire=True))]),
        g.ret(g.step())], [finish_attack(False)])])

    g = Graph("blocks_movement", B, (("state", STATE), ("grounded", B), ("held", B)))
    add(g, [g.ret(g.any(g.p("state", "attacking"), g.all(g.p("grounded"), g.p("held"))))])

    g = Graph("target_input", U, (("free_held", B), ("free_pressed", B), ("lock_pressed", B)))
    add(g, [g.condition(g.p("free_pressed"), [g.ret(u(1))]),
        g.condition(g.all(g.neg(g.p("free_held")), g.p("lock_pressed")), [g.ret(u(2))]), g.ret(u(0))])

    g = Graph("target_check", CHECK, (("phase", U), ("candidate", B), ("within_keep", B)))
    phase = g.op("add", g.p("phase"), u(1))
    add(g, [g.condition(g.any(g.neg(g.p("candidate")), g.neg(g.p("within_keep"))),
        [g.ret(g.check(g.p("phase"), False, False))]),
        g.condition(g.op("greater_equal", phase, g.t("visibility_period")), [g.ret(g.check(u(0), True, True))]),
        g.ret(g.check(phase, True, False))])

    g = Graph("candidate", B, (("active", B), ("has_mesh", B), ("collider_enabled", B), ("marked", B)))
    add(g, [g.ret(g.all(*(g.p(name) for name, _ in g.parameters)))])

    g = Graph("after_selected", B, (("distance", DISTANCE), ("index", U), ("selected_distance", DISTANCE), ("selected_index", U)))
    add(g, [g.ret(g.any(g.op("greater", g.p("distance", "high"), g.p("selected_distance", "high")),
        g.all(g.op("equal", g.p("distance", "high"), g.p("selected_distance", "high")),
            g.any(g.op("greater", g.p("distance", "low"), g.p("selected_distance", "low")),
                g.all(g.op("equal", g.p("distance", "low"), g.p("selected_distance", "low")),
                    g.op("greater", g.p("index"), g.p("selected_index")))))))])

    g = Graph("prefer_candidate", B, (("distance", DISTANCE), ("index", U), ("best_exists", B), ("best_distance", DISTANCE), ("best_index", U), ("cycling", B)))
    add(g, [g.ret(g.any(g.neg(g.p("best_exists")),
        g.op("less", g.p("distance", "high"), g.p("best_distance", "high")),
        g.all(g.op("equal", g.p("distance", "high"), g.p("best_distance", "high")),
            g.any(g.op("less", g.p("distance", "low"), g.p("best_distance", "low")),
                g.all(g.p("cycling"), g.op("equal", g.p("distance", "low"), g.p("best_distance", "low")),
                    g.op("less", g.p("index"), g.p("best_index")))))))])

    g = Graph("projectile", SPAWN, (("origin", VECTOR), ("direction", VECTOR)))
    velocity = g.operation("4fe0f030-091c-4d55-984d-769077622c33", *(g.op("multiply", g.p("direction", axis), g.t("projectile_speed")) for axis in ("x", "y", "z")))
    spawn_id = str(uuid.uuid5(NAMESPACE, "ProjectileLibrary/spawn"))
    add(g, [g.ret(g.operation(spawn_id, g.p("origin"), velocity, g.t("projectile_lifetime"), u(0)))])


    # Extra authored policy graphs; factories in the abstract base only copy.
    g = Graph("bounded", U, (("value", U), ("minimum", U), ("maximum", U)))
    add(g, [g.condition(g.op("less", g.p("value"), g.p("minimum")), [g.ret(g.p("minimum"))]),
        g.condition(g.op("greater", g.p("value"), g.p("maximum")), [g.ret(g.p("maximum"))]), g.ret(g.p("value"))])
    g = Graph("bounded_size", F, (("value", F),))
    add(g, [g.condition(g.op("less", g.p("value"), f(0)), [g.ret(f(0))]),
        g.condition(g.op("greater", g.p("value"), f(32)), [g.ret(f(32))]), g.ret(g.p("value"))])
    def bound(g, value, low, high):
        return link(g.call("bounded", {"value":value,"minimum":u(low) if isinstance(low,int) else low,"maximum":u(high) if isinstance(high,int) else high}))
    def size(g, value):return link(g.call("bounded_size", {"value":value}))
    def settings(g, member, field):return g.read(identity(member),field)
    def make(g, method, **fields):return link(g.call(method,fields))
    g = Graph("trace_settings", TRACE, (("enemy", B),))
    add(g,[g.condition(g.p("enemy"),[g.ret(make(g,"make_impact_trace",mask=settings(g,"impact_tuning","enemy_trace_mask"),ignore_player=b(False)))]),
        g.ret(make(g,"make_impact_trace",mask=settings(g,"impact_tuning","player_trace_mask"),ignore_player=b(True)))])
    g = Graph("impact", U, (("enemy",B),("target_exists",B),("target_is_player",B),("target_layers",U)))
    # Extract bit 1 with bounded integer arithmetic, since the common graph
    # language has no bitwise binary operator. Both products fit UInt32.
    bit = g.op("subtract",g.op("divide",g.p("target_layers"),u(2)),g.op("multiply",g.op("divide",g.p("target_layers"),u(4)),u(2)))
    add(g,[g.condition(g.p("enemy"),[g.condition(g.p("target_is_player"),[g.ret(u(2))]),g.ret(u(3))]),
        g.condition(g.all(g.p("target_exists"),g.op("not_equal",bit,u(0))),[g.ret(u(1))]),g.ret(u(0))])
    g = Graph("shot_visual", SHOT_VISUAL)
    add(g,[g.ret(make(g,"make_shot_visual",size=size(g,settings(g,"shot_style","size")),
        **{name:bound(g,settings(g,"shot_style",name),0,255) for name in ("red","green","blue")}))])
    g = Graph("effect_step", EFFECT_STEP, (("kind",U),("age",U),("advance",B)))
    def effect_path(advance):
        age = bound(g,g.p("age"),0,65534) if advance else bound(g,g.p("age"),0,65535)
        if advance:age=g.op("add",age,u(1))
        def branch(hit):
            prefix="hit" if hit else "muzzle"
            last=bound(g,settings(g,"effect_tuning",prefix+"_end_age"),0,65535)
            extent=size(g,settings(g,"effect_tuning",prefix+"_size"))
            ended=g.condition(g.op("greater_equal",age,last),[g.ret(make(g,"make_effect_step",age=age,frame=u(0),size=extent,playing=b(False)))])
            if hit:
                first=bound(g,settings(g,"effect_tuning","hit_first_frame"),0,2047)
                stride=bound(g,settings(g,"effect_tuning","hit_ticks_per_frame"),1,65535)
                frame=bound(g,g.op("add",first,g.op("divide",age,stride)),0,2047)
                return [ended,g.ret(make(g,"make_effect_step",age=age,frame=frame,size=extent,playing=b(True)))]
            def out(field):return g.ret(make(g,"make_effect_step",age=age,frame=bound(g,settings(g,"effect_tuning",field),0,2047),size=extent,playing=b(True)))
            return [ended,g.condition(g.op("less",age,bound(g,settings(g,"effect_tuning","muzzle_switch_age"),0,65535)),[out("muzzle_first_frame")]),out("muzzle_second_frame")]
        return [g.condition(g.op("not_equal",g.p("kind"),u(0)),branch(True)),*branch(False)]
    add(g,[g.condition(g.p("advance"),effect_path(True)),*effect_path(False)])
    g = Graph("damage_feedback", FEEDBACK, (("remaining_us",U),("elapsed_us",U),("restart",B)))
    duration=bound(g,settings(g,"feedback_tuning","duration_us"),0,10000000)
    def blank():return g.ret(make(g,"make_damage_feedback",remaining_us=u(0),enabled=b(False),border_pixels=u(0),red=u(0),green=u(0),blue=u(0)))
    def feedback_path(restart):
        remaining=duration if restart else bound(g,g.p("remaining_us"),0,duration)
        ended=g.condition(g.op("greater_equal",g.p("elapsed_us"),remaining),[blank()])
        remaining=g.op("subtract",remaining,g.p("elapsed_us"))
        width=bound(g,settings(g,"feedback_tuning","border_pixels"),1,64)
        red=bound(g,settings(g,"feedback_tuning","red_base"),0,255)
        span=bound(g,settings(g,"feedback_tuning","red_span"),0,g.op("subtract",u(255),red))
        pixels=g.op("divide",g.op("subtract",g.op("add",g.op("multiply",width,remaining),duration),u(1)),duration)
        color=g.op("add",red,g.op("divide",g.op("multiply",span,remaining),duration))
        return [ended,g.ret(make(g,"make_damage_feedback",remaining_us=remaining,enabled=b(True),border_pixels=pixels,red=color,
            green=bound(g,settings(g,"feedback_tuning","green"),0,255),blue=bound(g,settings(g,"feedback_tuning","blue"),0,255)))]
    add(g,[g.condition(g.op("equal",duration,u(0)),[blank()]),g.condition(g.p("restart"),feedback_path(True)),*feedback_path(False)])
    g = Graph("damage_rumble", RUMBLE)
    add(g,[g.ret(make(g,"make_damage_rumble",large=bound(g,settings(g,"feedback_tuning","rumble_large"),0,255),
        small=settings(g,"feedback_tuning","rumble_small"),duration_ms=bound(g,settings(g,"feedback_tuning","rumble_duration_ms"),0,60000)))])

    positions = {node["id"]: [float(index % 6 * 290), float(index // 6 * 180)]
        for graph in graphs for index, node in enumerate(graph["nodes"])}
    return extend_blueprint(Graph, {"version": 8, "id": identity("class"), "name": "DungeonPlayerCombat", "parent": identity("base"),
        "family": "component", "defaults": {}, "variables": [], "functions": graphs, "template": {"actors": []},
        "layout": {"positions": positions, "comments": {}, "split_pins": sorted(set(split))}})


if __name__ == "__main__":
    destination = ROOT / "resources/templates/dungeon-complete-flavors/blueprint/DungeonPlayerCombat.epokbp"
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(build(), indent=2) + "\n", encoding="utf-8")
    refresh_lua(ROOT / "resources/templates/dungeon-complete-flavors/lua/DungeonPlayerCombat.lua", "DungeonPlayerCombat")
    print(destination)
