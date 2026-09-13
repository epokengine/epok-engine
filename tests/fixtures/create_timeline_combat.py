"""Regenerate the authored Fireball combat Blueprint/scene with stable UUIDs.

Native input/target declarations stay in the example's assets/scripts directory.
This explicit authoring helper does not run in the editor or on the console.
"""
import copy
import json
from pathlib import Path
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

PROJECT = ROOT / "examples/timeline-spell"


def ident(name):
    return str(uuid.uuid5(uuid.NAMESPACE_URL, "epok.example.combat/" + name))


def main():
    effect = json.loads((PROJECT / "assets/Effects/Fireball.particle-effect.json").read_text())
    impact_id = next(m["id"] for m in effect["timeline"]["markers"] if m["name"] == "Impact")
    positions = {}
    def node(name, kind, inputs=None, position=(0, 0), **data):
        value = {"id": ident("node/" + name), "kind": {"kind": kind, **data}, "inputs": inputs or {}, "outputs": {}}
        positions[value["id"]] = list(position)
        return value
    def link(value):
        return {"kind": "link", "node": value["id"], "pin": "value"}
    def literal(kind, value):
        return {"kind": "literal", "value_type": {"kind": kind}, "value": value}
    def edge(source, pin, destination):
        source["outputs"][pin] = [destination["id"]]
    entry = node("cast", "entry")
    owner = node("owner", "builtin", position=(0, 220), operation={"kind": "self_entity"})
    transform = node("transform", "builtin", {"target": link(owner)}, (200, 240), operation={"kind": "get_transform"})
    play = node("play", "builtin", {"owner": link(owner), "transform": link(transform), "seed": literal("uint32", 87123)},
                (220, 0), operation={"kind": "spawn_particle_effect", "asset": effect["id"]})
    remember = node("remember", "call", {"value": link(play)}, (460, 0), function=ident("remember"))
    sequence = node("sequence", "builtin", {"playback": link(play)}, (460, 220), operation={"kind": "effect_sequence"})
    impact = node("impact", "wait_playback", {"playback": link(sequence)}, (700, 0),
                  condition={"kind": "marker", "timeline": effect["timeline"]["id"], "marker": impact_id})
    target = node("target", "builtin", {"target": {"kind": "parameter", "name": "victim"}}, (710, 230),
                  operation={"kind": "cast", "class": ident("target-class")})
    damage = node("damage", "call_on", {"__target": link(target), "amount": {"kind": "parameter", "name": "power"}},
                  (950, 0), **{"class": ident("target-class"), "function": ident("damage")})
    drain = node("drain", "wait_playback", {"playback": link(play)}, (1190, 0), condition={"kind": "effect_complete"})
    finished = node("finished", "call", position=(1190, 230), function=ident("finished"))
    cancelled = node("cancelled", "call", position=(950, 430), function=ident("cancelled"))
    for source, destination in [(entry, play), (play, remember), (remember, impact), (damage, drain)]:
        edge(source, "next", destination)
    edge(impact, "reached", damage)
    edge(impact, "completed", drain)
    edge(impact, "cancelled", cancelled)
    edge(drain, "completed", finished)
    edge(drain, "cancelled", cancelled)
    # Keep the complete cast graph visible in the ordinary editor canvas.
    for value, at in [(entry,(0,0)),(play,(180,0)),(remember,(360,0)),(impact,(540,0)),
                      (owner,(0,190)),(transform,(180,210)),(sequence,(360,190)),(target,(540,220)),
                      (damage,(540,390)),(drain,(360,390)),(finished,(180,450)),(cancelled,(0,390))]:
        positions[value["id"]] = list(at)
    cast = {"id": ident("graph/cast"), "name": "cast_spell", "override_id": ident("cast"), "returns": {"kind": "void"},
            "parameters": [{"name": "power", "value_type": {"kind": "uint32"}, "direction": "value"},
                           {"name": "victim", "value_type": {"kind": "record", "cpp_name": "epok::EntityHandle"}, "direction": "value"}],
            "entry": entry["id"], "nodes": [entry, owner, transform, play, remember, sequence, impact, target, damage, drain, finished, cancelled]}
    cancel_entry = node("cancel-entry", "entry")
    current = node("current", "call", position=(0, 180), function=ident("current"))
    stop = node("stop", "builtin", {"playback": link(current)}, (260, 0), operation={"kind": "stop_effect"})
    edge(cancel_entry, "next", stop)
    cancel = {"id": ident("graph/cancel"), "name": "cancel_spell", "override_id": ident("cancel"), "returns": {"kind": "void"},
              "parameters": [], "entry": cancel_entry["id"], "nodes": [cancel_entry, current, stop]}
    bp = {"version": 3, "id": ident("blueprint"), "name": "BP_Fireball", "parent": ident("combat-class"),
          "defaults": {}, "variables": [], "functions": [cast, cancel], "layout": {"positions": positions, "comments": {}},
          "template": {"construction": [], "entities": [], "overrides": {}, "references": []}}
    folder = PROJECT / "assets/blueprints"
    folder.mkdir(parents=True, exist_ok=True)
    documents.write_text(folder / "BP_Fireball.epokbp", documents.dumps(bp))

    original = documents.loads((PROJECT / "assets/scenes/Main.epokmap").read_text())
    scene = copy.deepcopy(original)
    camera = scene["entities"][0]
    camera.pop("script", None); camera.pop("timeline", None)
    camera.update(name="Combat Camera", id=ident("camera"))
    def entity(name, position=(0, 0, 0), **data):
        return {"id": ident("entity/" + name), "name": name, "kind": "Empty", "active": True,
                "position": list(position), "rotation": [0, 0, 0], "scale": [1, 1, 1], **data}
    def script(name, cls, properties, visual=False):
        return {"name": name, "class_id": cls, "properties": properties,
                "member_ids": {key: ident("combat-status" if visual and key=="status" else key) for key in properties},
                "provider": {"id": "blueprint" if visual else "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}
    controller = entity("Fireball Caster", script=script("BP_Fireball", bp["id"],
                        {"target": ident("entity/Target"), "status": ident("entity/Spell Status")}, True))
    sprite = {"enabled": True, "texture": effect["layers"][0]["content"]["sprite"]["texture"], "region": [0, 0, 16, 32],
              "size": [0.8, 1.6], "pivot": [0.5, 0], "orientation": "Upright", "unlit": True, "color": [1, 1, 1]}
    hero = entity("Mage", (-2, 0, 0), sprite=sprite)
    enemy_sprite = copy.deepcopy(sprite); enemy_sprite["flip_x"] = True; enemy_sprite["color"] = [1, 0.7, 0.5]
    target_entity = entity("Target", (2, 0, 0), sprite=enemy_sprite, script=script("SpellTarget", ident("target-class"),
                           {"status": ident("entity/Target Status")}))
    canvas = entity("Combat HUD", canvas={"enabled": True})
    def label(name, text, y):
        return entity(name, parent=4, rect={"anchor_min": [0, 1], "anchor_max": [0, 1], "pivot": [0, 1],
                      "position": [20, y], "size": [600, 28]}, text={"text": text, "enabled": True, "wrap": True})
    scene.update(name="Combat", entities=[camera, controller, hero, target_entity, canvas,
        label("Controls", "FIREBALL   X: cast   O: cancel   Square: reset   Start: pause", -20),
        label("Target Status", "Target HP: 100 / 100    Impact hits: 0", -55),
        label("Spell Status", "Ready. Fireball casts automatically once.", -420)])
    documents.write_text(PROJECT / "assets/scenes/Combat.epokmap", documents.dumps(scene))
    project_path = PROJECT / "Timeline Spell.epokproject"
    project = documents.loads(project_path.read_text())
    project["startup_scene"] = "assets/scenes/Combat.epokmap"
    documents.write_text(project_path, documents.dumps(project))
    print("Authored BP_Fireball combat example:", bp["id"])


if __name__ == "__main__":
    main()
