"""Regenerate the editable Dungeon Complete Blueprint run-flow class.

This emits ordinary graph nodes and split pins. It adds no graph runtime or
native gameplay policy. Projects edit the resulting class in the normal editor.
"""
from functools import reduce
import json
from pathlib import Path
import uuid

ROOT = Path(__file__).resolve().parents[1]
CLASS = "df3f7e3e-b99c-5530-9d86-a44f87e67c5f"
BASE = "a0aa09c9-b6d3-5ae4-b6ab-dc99050463f4"
TIMING = "202d836c-2271-52a2-8db0-8d36f4da6c3d"
STATUS = "68467e3b-f7f7-5a05-a54c-102cd208a22d"
ENTRY_Z = "8e8627cd-3e49-5557-94b6-3f60be28de73"
ASSIGN = "e3729b65-3558-5d55-aadb-b427a1e577b1"
TRUNCATE = "29249825-c121-54e6-a908-8cc041db8971"
F, U, B, V = ({"kind": kind} for kind in ("fixed", "uint32", "bool", "void"))
EVENTS = dict(zip(
    ("start", "die", "win", "advance", "try_enter_boss", "restart_requested"),
    ("9b6a7d68-efe4-56ef-880b-867786725a48", "6229c53c-6439-543a-8fe4-4f665d4ec86c",
     "28a88de5-ef38-5c50-9a66-24305ddc75e0", "b362cddc-ce9d-5a05-8e1c-748f10a9a701",
     "213c8b98-c8f2-500d-b590-dadac0609dd2", "0f57f68c-d789-528f-8b81-c691149a1dbb")))
FIELDS = [("state", U, 0), ("phase", U, 0), ("elapsed", F, 0.0), ("fade", U, 255),
          ("music_gain", F, 1.0), ("playing", B, False), ("alive", B, True),
          ("restart", B, False), ("stop_music", B, False), ("valid", B, True)]
SNAPSHOT = {"kind": "record", "cpp_name": "DungeonRunSnapshot",
            "fields": [{"name": name, "value_type": ty} for name, ty, _ in FIELDS]}


def identity(name):
    return str(uuid.uuid5(uuid.UUID(CLASS), name))


def literal(value, ty):
    return {"kind": "literal", "value_type": ty, "value": value}


def u(value):
    return literal(value, U)


def f(value):
    return literal(float(value), F)


def b(value):
    return literal(value, B)


def parameter(name):
    return {"kind": "parameter", "name": name}


def link(node, pin="value"):
    return {"kind": "link", "node": node["id"], "pin": pin}


class Graph:
    def __init__(self, name, returns=V, parameters=()):
        self.name, self.returns, self.parameters = name, returns, parameters
        self.nodes, self.reads, self.split = [], {}, []

    def node(self, kind, inputs=None):
        node = {"id": identity(f"{self.name}/node/{len(self.nodes)}"),
                "kind": kind if isinstance(kind, dict) else {"kind": kind},
                "inputs": inputs or {}, "outputs": {}}
        self.nodes.append(node)
        return node

    def read(self, member, field=""):
        if member not in self.reads:
            self.reads[member] = self.node({"kind": "get_variable", "member": member})
        node = self.reads[member]
        if field:
            self.split.append(node["id"] + ":out:value")
        return link(node, "value" + ("." + field if field else ""))

    def s(self, name):
        return self.read(STATUS, name)

    def t(self, name):
        return self.read(TIMING, name)

    def op(self, op, a, c):
        return link(self.node({"kind": "binary", "op": op}, {"a": a, "b": c}))

    def all(self, *values):
        return reduce(lambda a, c: self.op("and", a, c), values)

    def any(self, *values):
        return reduce(lambda a, c: self.op("or", a, c), values)

    def eq(self, field, value):
        return self.op("equal", self.s(field), u(value))

    def call(self, method, inputs=None):
        return self.node({"kind": "call", "function": EVENTS.get(method, identity("method/" + method))}, inputs)

    def write(self, **changes):
        inputs = {"value": literal({name: value for name, _, value in FIELDS}, SNAPSHOT)}
        inputs.update({"value." + name: changes.get(name, self.s(name)) for name, _, _ in FIELDS})
        node = self.node({"kind": "call", "function": ASSIGN}, inputs)
        self.split.append(node["id"] + ":in:value")
        return node

    def ret(self, value=None):
        return self.node("return", {} if value is None else {"value": value})

    def condition(self, condition, yes, no=()):
        return (condition, yes, no)

    def finish(self, steps):
        def sequence(steps, continuation=None):
            for step in reversed(steps):
                if isinstance(step, tuple):
                    condition, yes, no = step
                    node = self.node("branch", {"condition": condition})
                    for pin, branch in (("true", yes), ("false", no)):
                        target = sequence(branch, continuation)
                        if target:
                            node["outputs"][pin] = [target]
                else:
                    node = step
                    if node["kind"]["kind"] != "return" and continuation:
                        node["outputs"]["next"] = [continuation]
                continuation = node["id"]
            return continuation
        first = sequence(steps)
        entry = self.node("entry")
        entry["outputs"]["next"] = [first] if first else []
        graph = {"id": identity("method/" + self.name), "name": self.name,
                 "parameters": [{"name": name, "value_type": ty, "direction": "value"}
                                for name, ty in self.parameters],
                 "returns": self.returns, "entry": entry["id"], "nodes": self.nodes}
        if self.name in EVENTS:
            graph["override_id"] = EVENTS[self.name]
        return graph


def build():
    graphs, split = [], []

    def add(g, steps):
        graphs.append(g.finish(steps))
        split.extend(g.split)

    g = Graph("timing_valid", B)
    checks = [g.op(op, g.t(name), f(bound)) for name in
              ("music_lead", "fade_in", "control_delay", "death_hold", "fade_out", "restart_hold")
              for op, bound in (("greater_equal", 0), ("less_equal", 60))]
    checks += [g.op("greater_equal", g.read(ENTRY_Z), f(-4096)), g.op("less_equal", g.read(ENTRY_Z), f(4096))]
    add(g, [g.ret(g.all(*checks))])

    g = Graph("phase_duration", F)
    add(g, [g.condition(g.eq("phase", phase), [g.ret(g.t(name))]) for phase, name in
            ((2, "fade_in"), (3, "control_delay"), (5, "death_hold"), (6, "fade_out"), (7, "restart_hold"))]
           + [g.ret(g.t("music_lead"))])

    g = Graph("fade_progress", U, (("duration", F), ("elapsed", F)))
    duration, elapsed = parameter("duration"), parameter("elapsed")
    def progress(time):
        value = g.op("divide", g.op("multiply", time, f(255)), duration)
        return link(g.node({"kind": "operation", "operation": TRUNCATE}, {TRUNCATE + ":parameter:0": value}))
    add(g, [g.condition(g.op("less_equal", duration, f(0)), [g.ret(u(255))]),
            g.condition(g.op("greater", elapsed, duration), [g.ret(u(255))]),
            g.ret(progress(elapsed))])

    g = Graph("opacity", U)
    intro = g.any(g.eq("phase", 0), g.eq("phase", 1), g.eq("phase", 7))
    incoming = g.call("fade_progress", {"duration": g.t("fade_in"), "elapsed": g.s("elapsed")})
    outgoing = g.call("fade_progress", {"duration": g.t("fade_out"), "elapsed": g.s("elapsed")})
    add(g, [g.condition(intro, [g.ret(u(255))]),
            g.condition(g.eq("phase", 2), [incoming, g.ret(g.op("subtract", u(255), link(incoming)))]),
            g.condition(g.eq("phase", 6), [outgoing, g.ret(link(outgoing))]), g.ret(u(0))])

    g = Graph("gain", F)
    duration, elapsed = g.t("fade_out"), g.s("elapsed")
    add(g, [g.condition(g.eq("phase", 7), [g.ret(f(0))]),
            g.condition(g.eq("phase", 6), [
                g.condition(g.op("less_equal", duration, f(0)), [g.ret(f(0))]),
                g.condition(g.op("greater", elapsed, duration), [g.ret(f(0))]),
                g.ret(g.op("subtract", f(1), g.op("divide", elapsed, duration)))]), g.ret(f(1))])

    g = Graph("refresh")
    valid, opacity, gain = (g.call(name) for name in ("timing_valid", "opacity", "gain"))
    add(g, [valid, opacity, gain, g.write(
        valid=g.all(link(valid), g.op("less_equal", g.s("phase"), u(7)),
                    g.op("less_equal", g.s("state"), u(3)), g.op("greater_equal", g.s("elapsed"), f(0)),
                    g.op("less_equal", g.s("elapsed"), f(61))),
        playing=g.eq("phase", 4), alive=g.op("less", g.s("state"), u(2)), restart=b(False),
        stop_music=g.eq("phase", 7), fade=link(opacity), music_gain=link(gain)), g.ret()])

    g = Graph("start")
    add(g, [g.write(state=u(0), phase=u(0), elapsed=f(0)), g.call("refresh"), g.ret()])
    g = Graph("die")
    add(g, [g.condition(g.op("greater_equal", g.s("state"), u(2)), [g.ret()]),
            g.write(state=u(3), phase=u(5), elapsed=f(0)), g.call("refresh"), g.ret()])
    g = Graph("win")
    add(g, [g.condition(g.op("not_equal", g.s("state"), u(1)), [g.ret()]),
            g.write(state=u(2)), g.call("refresh"), g.ret()])

    g = Graph("try_enter_boss", B, (("has_key", B), ("player_z", F)))
    reject = g.any(g.op("not_equal", g.s("state"), u(0)),
                   link(g.node("not", {"value": parameter("has_key")})),
                   g.op("less_equal", parameter("player_z"), g.read(ENTRY_Z)))
    add(g, [g.condition(reject, [g.ret(b(False))]), g.write(state=u(1)), g.call("refresh"), g.ret(b(True))])
    g = Graph("restart_requested", B, (("pressed", B),))
    add(g, [g.ret(g.all(parameter("pressed"), g.s("playing"), g.op("greater_equal", g.s("state"), u(2))))])

    g = Graph("advance", V, (("dt", F), ("music_playing", B)))
    invalid_delta = g.any(g.op("less_equal", parameter("dt"), f(0)), g.op("greater", parameter("dt"), f(1)))
    duration = g.call("phase_duration")
    add(g, [g.condition(invalid_delta, [g.ret()]), g.call("refresh"),
            g.condition(link(g.node("not", {"value": g.s("valid")})), [g.ret()]),
            g.condition(g.eq("phase", 0), [
                g.condition(parameter("music_playing"), [g.write(phase=u(1), elapsed=f(0))]),
                g.call("refresh"), g.ret()]),
            g.condition(g.eq("phase", 4), [g.ret()]), duration,
            g.write(elapsed=g.op("add", g.s("elapsed"), parameter("dt"))),
            g.condition(g.op("less", g.s("elapsed"), link(duration)), [g.call("refresh"), g.ret()]),
            g.write(elapsed=g.op("subtract", g.s("elapsed"), link(duration))),
            g.condition(g.eq("phase", 7), [g.call("refresh"), g.write(restart=b(True)), g.ret()]),
            g.write(phase=g.op("add", g.s("phase"), u(1))),
            g.condition(g.eq("phase", 4), [g.write(elapsed=f(0))]), g.call("refresh"), g.ret()])

    positions, comments = {}, {}
    for row, graph in enumerate(graphs):
        for index, node in enumerate(graph["nodes"]):
            positions[node["id"]] = [float((index % 6) * 290), float((index // 6) * 180)]
        comments[identity("comment/" + graph["name"])] = graph["name"].replace("_", " ")
    return {"version": 8, "id": CLASS, "name": "DungeonRunFlow", "parent": BASE,
            "family": "component", "defaults": {}, "variables": [], "functions": graphs,
            "layout": {"positions": positions, "comments": comments, "split_pins": sorted(set(split))},
            "template": {"actors": []}}


if __name__ == "__main__":
    path = ROOT / "resources/templates/dungeon-complete-flavors/blueprint/DungeonRunFlow.epokbp"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(build(), indent=2) + "\n", encoding="utf-8")
    print(path)
