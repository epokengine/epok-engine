"""Generate ordinary editable Blueprint graphs for the project's player vitals."""
import importlib.util
import json
from pathlib import Path
import uuid

ROOT = Path(__file__).resolve().parents[1]
NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "DungeonPlayerVitals/" + name))
CLASS, BASE = identity("class"), identity("base")
EVENTS = {name: identity(name) for name in ("start", "advance", "damage", "heal", "can_heal")}
OPERATIONS = {
    "make_state": "43903550-2746-5f1c-af57-ac221db6681c",
    "valid": "df6acd25-a526-5605-86e3-1a73c31737b8",
    "damage": "564672a4-0a4a-51e6-ad32-4d8f447b2da7",
    "heal": "adc1907c-df26-56b3-8d13-25015e4792f2",
}
F, U, I, B, V = ({"kind": k} for k in ("fixed", "uint32", "int32", "bool", "void"))
HEALTH = {"kind": "record", "cpp_name": "epok::HealthState", "fields": [
    {"name": name, "value_type": U} for name in ("maximum", "current")]}
CHANGE = {"kind": "record", "cpp_name": "epok::HealthChange", "fields": [
    {"name": name, "value_type": ty} for name, ty in (
        ("state", HEALTH), ("applied", U), ("depleted", B), ("revived", B),
        ("result", {"kind": "enum", "cpp_name": "epok::HealthResult",
                    "variants": {"Unchanged": 0, "Applied": 1, "InvalidState": 2}}))]}
spec = importlib.util.spec_from_file_location("dungeon_run_graph", ROOT / "tools/generate-dungeon-run-flow.py")
run_graph = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run_graph)
literal = run_graph.literal
link = run_graph.link
parameter = run_graph.parameter
f = lambda value: literal(float(value), F)
u = lambda value: literal(value, U)
i = lambda value: literal(value, I)


class Graph(run_graph.Graph):
    def node(self, kind, inputs=None):
        node = {"id": identity(f"{self.name}/node/{len(self.nodes)}"),
                "kind": kind if isinstance(kind, dict) else {"kind": kind}, "inputs": inputs or {}, "outputs": {}}
        self.nodes.append(node)
        return node

    def r(self, name, field=""):
        return self.read(identity(name), field)

    def call(self, name, inputs=None):
        return self.node({"kind": "call", "function": EVENTS.get(name, identity(name))}, inputs)

    def operation(self, name, *values):
        operation = OPERATIONS[name]
        return link(self.node({"kind": "operation", "operation": operation},
                              {operation + f":parameter:{n}": value for n, value in enumerate(values)}))

    def assign(self, health=None, immunity=None, valid=None):
        return self.call("assign", {"value": self.r("health") if health is None else health,
                                    "cooldown": self.r("immunity") if immunity is None else immunity,
                                    "available": self.r("valid") if valid is None else valid})

    def field(self, node, name):
        self.split.append(node["id"] + ":out:value")
        return link(node, "value." + name)

    def finish(self, steps):
        graph = super().finish(steps)
        graph["id"] = identity(self.name)
        if self.name in EVENTS:
            graph["id"] = identity("graph/" + self.name)
            graph["override_id"] = EVENTS[self.name]
        else:
            graph.pop("override_id", None)
        return graph


def build():
    graphs, split = [], []

    def add(g, steps):
        graphs.append(g.finish(steps))
        split.extend(g.split)

    g = Graph("start")
    maximum, immunity = g.r("tuning", "maximum"), g.r("tuning", "hit_immunity")
    valid = g.all(g.op("greater", maximum, u(0)), g.op("greater_equal", immunity, f(0)),
                  g.op("less_equal", immunity, f(60)))
    add(g, [g.assign(g.operation("make_state", maximum, maximum), f(0), valid), g.ret()])

    g = Graph("advance", parameters=(("dt", F),))
    dt = parameter("dt")
    remaining = g.op("subtract", g.r("immunity"), dt)
    add(g, [g.condition(g.all(g.r("valid"), g.op("greater", dt, f(0)), g.op("less_equal", dt, f(1)),
                             g.op("greater", g.r("immunity"), f(0))), [
        g.condition(g.op("less", remaining, f(0)), [g.assign(immunity=f(0))],
                    [g.assign(immunity=remaining)])]), g.ret()])

    # Value-returning project functions capture results on their execution pins.
    # The later assignment cannot cause a pure node to evaluate a second hit.
    for name, operation in (("damage_change", "damage"), ("healing_change", "heal")):
        g = Graph(name, CHANGE, (("amount", I),))
        add(g, [g.ret(g.operation(operation, g.r("health"), parameter("amount")))])

    g = Graph("damage", CHANGE, (("amount", I), ("enabled", B)))
    accepted = g.all(g.r("valid"), parameter("enabled"), g.op("greater", parameter("amount"), i(0)),
                     g.op("less_equal", g.r("immunity"), f(0)))
    change = g.call("damage_change", {"amount": parameter("amount")})
    add(g, [g.condition(accepted, [change,
        g.condition(g.op("greater", g.field(change, "applied"), u(0)),
                    [g.assign(g.field(change, "state"), g.r("tuning", "hit_immunity"))]),
        g.ret(link(change))], [g.ret(g.operation("damage", g.r("health"), i(0)))])])

    g = Graph("heal", CHANGE, (("amount", I),))
    change = g.call("healing_change", {"amount": parameter("amount")})
    add(g, [g.condition(g.r("valid"), [change,
        g.condition(g.op("greater", g.field(change, "applied"), u(0)), [g.assign(g.field(change, "state"))]),
        g.ret(link(change))], [g.ret(g.operation("heal", g.r("health"), i(0)))])])

    g = Graph("can_heal", B)
    add(g, [g.ret(g.all(g.r("valid"), g.operation("valid", g.r("health")),
                       g.op("less", g.r("health", "current"), g.r("health", "maximum"))))])
    positions = {node["id"]: [float(n % 6 * 290), float(n // 6 * 180)]
                 for graph in graphs for n, node in enumerate(graph["nodes"])}
    return {"version": 8, "id": CLASS, "name": "DungeonPlayerVitals", "parent": BASE, "family": "component",
            "defaults": {}, "variables": [], "functions": graphs, "template": {"actors": []},
            "layout": {"positions": positions, "comments": {}, "split_pins": sorted(set(split))}}


if __name__ == "__main__":
    destination = ROOT / "resources/templates/dungeon-complete-flavors/blueprint/DungeonPlayerVitals.epokbp"
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(build(), indent=2) + "\n", encoding="utf-8")
    print(destination)
