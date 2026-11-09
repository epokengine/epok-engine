"""Generate ordinary editable project encounter policy Blueprint graphs."""
import importlib.util
import json
from pathlib import Path
import uuid
from dungeon_encounter_world import extend_blueprint, refresh_lua

ROOT = Path(__file__).resolve().parents[1]
NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "DungeonEncounter/" + name))
F, U, I, B, V = ({"kind": k} for k in ("fixed", "uint32", "int32", "bool", "void"))
record = lambda name, fields: {"kind": "record", "cpp_name": name,
                              "fields": [{"name": key, "value_type": ty} for key, ty in fields]}
GUARD_FIELDS = [(key, F) for key in ("timer", "windup", "flash", "dead_time", "sight_timer", "ai_time")] + [("patrol", B), ("can_see", B)]
BOSS_FIELDS = [(key, F) for key in ("timer", "windup", "flash")]
GUARD = record("DungeonGuardState", GUARD_FIELDS)
BOSS = record("DungeonBossState", BOSS_FIELDS)
HEALTH = record("epok::HealthState", [("maximum", U), ("current", U)])
CHANGE = record("epok::HealthChange", [("state", HEALTH), ("applied", U), ("depleted", B), ("revived", B),
    ("result", {"kind": "enum", "cpp_name": "epok::HealthResult", "variants": {"Unchanged": 0, "Applied": 1, "InvalidState": 2}})])
GUARD_STEP = record("DungeonGuardStep", [("state", GUARD), ("dt", F), ("action", U)])
BOSS_STEP = record("DungeonBossStep", [("state", BOSS), ("action", U), ("fan", B), ("speed", F)])
EVENTS = {name: identity(name) for name in ("starting_health", "should_activate", "should_unload", "guard_start",
    "guard_schedule", "guard_decide", "boss_start", "boss_needs_sight", "boss_advance")}
EVENTS.update(guard_hit=identity("guard_hit_event"), boss_hit=identity("boss_hit_event"))
EVENTS.update({name: identity(name) for name in ("projectile", "shot_size", "shot_damage", "pellet_count", "pellet_index", "shot_target")})
SHOT_FIELDS = ("speed", "fan_speed", "lifetime", "fan_degrees", "size", "fan_size", "damage", "boss_damage", "target_height")
VECTOR = record("epok::GameplayVector3", [(name, F) for name in ("x", "y", "z")])
SPAWN = record("epok::ProjectileSpawn3D", [("origin", VECTOR), ("velocity", VECTOR), ("lifetime", F), ("tag", U)])
OPERATIONS = {"make_state": "43903550-2746-5f1c-af57-ac221db6681c", "damage": "564672a4-0a4a-51e6-ad32-4d8f447b2da7",
    "wrap_degrees": "9b882fef-4a7b-4463-81aa-65b2f7e0dcd0", "sine_degrees": "eeee7d31-a8c6-437a-95db-10e163771b24",
    "cosine_degrees": "565552a8-869a-4ca7-a1d8-a7ad6e1508a7", "vector3": "4fe0f030-091c-4d55-984d-769077622c33",
    "spawn": "f2e2aee3-6cad-5a01-8799-030df58f4dd4"}
spec = importlib.util.spec_from_file_location("encounter_graph_base", ROOT / "tools/generate-dungeon-run-flow.py")
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
literal, link, parameter = base.literal, base.link, base.parameter
f = lambda value: literal(float(value), F)
u = lambda value: literal(value, U)
b = lambda value: literal(value, B)
i = lambda value: literal(value, I)


class Graph(base.Graph):
    def node(self, kind, inputs=None):
        node = {"id": identity(f"{self.name}/node/{len(self.nodes)}"),
                "kind": kind if isinstance(kind, dict) else {"kind": kind}, "inputs": inputs or {}, "outputs": {}}
        self.nodes.append(node)
        return node

    def t(self, name):
        if name in SHOT_FIELDS:
            return self.read(identity("shot_tuning"), name)
        group = "health_tuning" if name in ("guard_health", "boss_health", "boss_phase_health") else (
            "movement" if name in ("activation_radius", "retention_radius", "sight_radius", "guard_range", "patrol_extent", "patrol_arrival", "chase_speed", "patrol_speed") else (
            "boss_timing" if name.startswith("boss_") else "guard_timing"))
        return self.read(identity(group), name)

    def p(self, name, field=""):
        if field:
            self.split.append(identity(self.name + "/entry") + ":out:" + name)
        return {"kind": "link", "node": identity(self.name + "/entry"), "pin": name + ("." + field if field else "")}

    def neg(self, value):
        return link(self.node("not", {"value": value}))

    def call(self, name, inputs=None):
        return self.node({"kind": "call", "function": identity(name)}, inputs)

    def operation(self, name, *values):
        operation = OPERATIONS[name]
        return link(self.node({"kind": "operation", "operation": operation},
                             {operation + f":parameter:{n}": value for n, value in enumerate(values)}))

    def state(self, boss=False, values=None, **changes):
        fields = BOSS_FIELDS if boss else GUARD_FIELDS
        values = values or {name: self.p("state", name) for name, _ in fields}
        return link(self.call("make_boss" if boss else "make_guard", {name: changes.get(name, values[name]) for name, _ in fields}))

    def step(self, state=None, dt=None, action=0, query=None):
        return link(self.call("make_guard_step", {"state": self.p("state") if state is None else state,
            "dt": f(0) if dt is None else dt, "action": u(action)}))

    def boss_step(self, state, action, fan, speed):
        return link(self.call("make_boss_step", {"state": state, "action": u(action), "fan": b(fan), "speed": speed}))

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
        if self.name in EVENTS:
            graph["id"] = identity("graph/" + self.name)
            graph["override_id"] = EVENTS[self.name]
        else:
            graph["id"] = identity(self.name)
            graph.pop("override_id", None)
        return graph


def build():
    graphs, split = [], []

    def add(g, steps):
        graphs.append(g.finish(steps))
        split.extend(g.split)

    g = Graph("starting_health", HEALTH, (("boss", B),))
    add(g, [g.condition(g.p("boss"), [g.ret(g.operation("make_state", g.t("boss_health"), g.t("boss_health")))],
                       [g.ret(g.operation("make_state", g.t("guard_health"), g.t("guard_health")))])])

    g = Graph("should_activate", B, (("defeated", B), ("exploring", B), ("near_home", B)))
    add(g, [g.ret(g.all(g.neg(g.p("defeated")), g.p("exploring"), g.p("near_home")))])
    g = Graph("should_unload", B, (("state", GUARD), ("boss_fight", B), ("near_home", B)))
    add(g, [g.ret(g.all(g.op("less_equal", g.p("state", "dead_time"), f(0)), g.any(g.p("boss_fight"), g.neg(g.p("near_home")))))])

    g = Graph("guard_start", GUARD, (("encounter", U),))
    encounter = g.p("encounter")
    modulo = lambda divisor: g.op("subtract", encounter, g.op("multiply", g.op("divide", encounter, u(divisor)), u(divisor)))
    states = []
    for mod in range(3):
        values = {name: (b(False) if ty == B else f(0)) for name, ty in GUARD_FIELDS}
        values.update(timer=g.t("guard_initial_delay"), sight_timer=f((mod + 1) * 137 / 4096))
        states.append(g.condition(g.op("equal", modulo(3), u(mod)), [
            g.condition(g.op("equal", modulo(2), u(1)), [g.ret(g.state(values=values, ai_time=f(-68 / 4096)))],
                        [g.ret(g.state(values=values))])]))
    add(g, states + [g.ret(g.state(values=values))])

    g = Graph("guard_schedule", GUARD_STEP, (("state", GUARD), ("dt", F)))
    dt = g.p("dt")
    dead = g.op("subtract", g.p("state", "dead_time"), dt)
    elapsed = g.op("add", g.p("state", "ai_time"), dt)
    changes = {"ai_time": f(0), "timer": g.op("subtract", g.p("state", "timer"), elapsed),
               "sight_timer": g.op("subtract", g.p("state", "sight_timer"), elapsed)}
    scheduled = []
    for flashed in (True, False):
        updated = g.state(**changes, **({"flash": g.op("subtract", g.p("state", "flash"), elapsed)} if flashed else {}))
        scheduled.append(g.ret(g.step(updated, elapsed, 7, g.op("less_equal", changes["sight_timer"], f(0)))))
    add(g, [g.condition(g.any(g.op("less_equal", dt, f(0)), g.op("greater", dt, f(1))), [g.ret(g.step())]),
        g.condition(g.op("greater", g.p("state", "dead_time"), f(0)), [
            g.condition(g.op("less_equal", dead, f(0)), [g.ret(g.step(g.state(dead_time=dead), action=6))],
                        [g.ret(g.step(g.state(dead_time=dead)))])]),
        g.condition(g.op("less", elapsed, g.t("ai_interval")), [g.ret(g.step(g.state(ai_time=elapsed)))]),
        g.condition(g.op("greater", g.p("state", "flash"), f(0)), [scheduled[0]], [scheduled[1]])])

    # Refresh sight once, capture that state on the execution pin, then decide.
    g = Graph("guard_refresh", GUARD, (("state", GUARD), ("close", B), ("observed", B)))
    add(g, [g.condition(g.neg(g.p("close")), [g.ret(g.state(can_see=b(False)))]),
        g.condition(g.op("less_equal", g.p("state", "sight_timer"), f(0)),
                    [g.ret(g.state(can_see=g.p("observed"), sight_timer=g.t("sight_interval")))]), g.ret(g.p("state"))])
    params = (("state", GUARD), ("dt", F), ("close", B), ("observed", B), ("within_range", B), ("at_patrol_goal", B))
    g = Graph("guard_decide", GUARD_STEP, params)
    refreshed = g.call("guard_refresh", {name: g.p(name) for name in ("state", "close", "observed")})
    action = g.call("guard_action", {name: link(refreshed) if name == "state" else g.p(name) for name, _ in params})
    add(g, [refreshed, action, g.ret(link(action))])
    g = Graph("guard_action", GUARD_STEP, params)
    dt = g.p("dt")
    windup = g.op("subtract", g.p("state", "windup"), dt)
    add(g, [g.condition(g.all(g.p("close"), g.p("state", "can_see")), [
        g.condition(g.op("greater", g.p("state", "windup"), f(0)), [
            g.condition(g.op("less_equal", windup, f(0)), [g.ret(g.step(g.state(windup=windup, timer=g.t("guard_cooldown")), dt, 5))],
                        [g.ret(g.step(g.state(windup=windup), dt))])]),
        g.condition(g.op("less_equal", g.p("state", "timer"), f(0)), [g.ret(g.step(g.state(windup=g.t("guard_windup")), dt, 4))]),
        g.condition(g.p("within_range"), [g.ret(g.step(dt=dt, action=1))], [g.ret(g.step(dt=dt, action=2))])]),
        g.condition(g.p("at_patrol_goal"), [g.ret(g.step(g.state(windup=f(0), patrol=g.neg(g.p("state", "patrol"))), dt, 3))],
                    [g.ret(g.step(g.state(windup=f(0)), dt, 3))])])

    g = Graph("guard_hit", GUARD, (("state", GUARD), ("depleted", B)))
    add(g, [g.condition(g.p("depleted"), [g.ret(g.state(flash=g.t("guard_flash"), dead_time=g.t("guard_death")))],
                       [g.ret(g.state(flash=g.t("guard_flash")))])])
    g = Graph("boss_hit", BOSS, (("state", BOSS),))
    add(g, [g.ret(g.state(boss=True, flash=g.t("boss_flash")))])

    g = Graph("boss_start", BOSS)
    add(g, [g.ret(link(g.call("make_boss", {"timer": g.t("boss_initial_delay"), "windup": f(0), "flash": f(0)})))])
    g = Graph("boss_needs_sight", B, (("state", BOSS), ("dt", F)))
    add(g, [g.ret(g.all(g.op("less_equal", g.p("state", "windup"), f(0)),
                       g.op("less_equal", g.op("subtract", g.p("state", "timer"), g.p("dt")), f(0))))])

    g = Graph("boss_advance", BOSS_STEP, (("state", BOSS), ("health", U), ("dt", F), ("observed", B), ("within_range", B)))
    dt = g.p("dt")
    fan_branches = []
    for fan in (True, False):
        speed = g.t("boss_fan_speed" if fan else "boss_speed")
        phase_branches = []
        for flashed in (True, False):
            changes = {"timer": g.op("subtract", g.p("state", "timer"), dt)}
            if flashed:
                changes["flash"] = g.op("subtract", g.p("state", "flash"), dt)
            windup = g.op("subtract", g.p("state", "windup"), dt)
            output = lambda action, **extra: g.ret(g.boss_step(g.state(boss=True, **(changes | extra)), action, fan, speed))
            phase_branches.append([
                g.condition(g.op("greater", g.p("state", "windup"), f(0)), [
                    g.condition(g.op("less_equal", windup, f(0)),
                        [output(5, windup=windup, timer=g.t("boss_fan_cooldown" if fan else "boss_cooldown"))],
                        [output(0, windup=windup)])]),
                g.condition(g.all(g.op("less_equal", changes["timer"], f(0)), g.p("observed")),
                            [output(4, windup=g.t("boss_fan_windup" if fan else "boss_windup"))]),
                g.condition(g.p("within_range"), [output(1)], [output(2)])])
        fan_branches.append([
            g.condition(g.any(g.op("less_equal", dt, f(0)), g.op("greater", dt, f(1))), [g.ret(g.boss_step(g.p("state"), 0, fan, speed))]),
            g.condition(g.op("greater", g.p("state", "flash"), f(0)), phase_branches[0], phase_branches[1])])
    add(g, [g.condition(g.op("less_equal", g.p("health"), g.t("boss_phase_health")), fan_branches[0], fan_branches[1])])

    g = Graph("projectile", SPAWN, (("origin", VECTOR), ("direction", VECTOR), ("pellet", I), ("fan", B)))
    def shot(angle, fan):
        cosine, sine = g.operation("cosine_degrees", angle), g.operation("sine_degrees", angle)
        speed = g.t("fan_speed" if fan else "speed")
        x, y, z = (g.p("direction", axis) for axis in ("x", "y", "z"))
        vx = g.op("add", g.op("multiply", x, cosine), g.op("multiply", z, sine))
        vz = g.op("subtract", g.op("multiply", z, cosine), g.op("multiply", x, sine))
        velocity = g.operation("vector3", g.op("multiply", vx, speed), g.op("multiply", y, speed), g.op("multiply", vz, speed))
        return g.ret(g.operation("spawn", g.p("origin"), velocity, g.t("lifetime"), u(1)))
    branches = []
    for fan in (True, False):
        angle = g.operation("wrap_degrees", g.t("fan_degrees"))
        branches.append([
            g.condition(g.op("less", g.p("pellet"), i(0)), [shot(g.op("subtract", f(0), angle), fan)]),
            g.condition(g.op("greater", g.p("pellet"), i(0)), [shot(angle, fan)]), shot(f(0), fan)])
    add(g, [g.condition(g.p("fan"), branches[0], branches[1])])

    for name, returns, parameter_name, yes, no in (
        ("shot_size", F, "fan", "fan_size", "size"),
        ("shot_damage", U, "boss_fight", "boss_damage", "damage")):
        g = Graph(name, returns, ((parameter_name, B),))
        add(g, [g.condition(g.p(parameter_name), [g.ret(g.t(yes))], [g.ret(g.t(no))])])
    g = Graph("pellet_count", U, (("fan", B),))
    add(g, [g.condition(g.p("fan"), [g.ret(u(3))], [g.ret(u(1))])])
    g = Graph("pellet_index", I, (("ordinal", U), ("fan", B)))
    add(g, [g.condition(g.neg(g.p("fan")), [g.ret(i(0))]),
        g.condition(g.op("equal", g.p("ordinal"), u(0)), [g.ret(i(-1))]),
        g.condition(g.op("equal", g.p("ordinal"), u(2)), [g.ret(i(1))]), g.ret(i(0))])
    g = Graph("shot_target", VECTOR, (("player_position", VECTOR),))
    add(g, [g.ret(g.operation("vector3", g.p("player_position", "x"),
        g.op("add", g.p("player_position", "y"), g.t("target_height")), g.p("player_position", "z")))])

    positions = {node["id"]: [float(n % 7 * 290), float(n // 7 * 180)] for graph in graphs for n, node in enumerate(graph["nodes"])}
    return extend_blueprint(Graph, {"version": 8, "id": identity("class"), "name": "DungeonEncounter", "parent": identity("base"), "family": "component",
            "defaults": {}, "variables": [], "functions": graphs, "template": {"actors": []},
            "layout": {"positions": positions, "comments": {}, "split_pins": sorted(set(split))}})


if __name__ == "__main__":
    destination = ROOT / "resources/templates/dungeon-complete-flavors/blueprint/DungeonEncounter.epokbp"
    destination.write_text(json.dumps(build(), indent=2) + "\n", encoding="utf-8")
    refresh_lua(ROOT / "resources/templates/dungeon-complete-flavors/lua/DungeonEncounter.lua")
    print(destination)
