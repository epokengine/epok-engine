"""Create the authored fireball example with stable IDs and shared TimelineAsset.

Run explicitly to regenerate the example; this is authoring data generation,
never a runtime preset or alternate timeline interpreter.
"""
import copy
import json
from pathlib import Path
import shutil
import struct
import uuid

ROOT = Path(__file__).resolve().parents[2]
PROJECT = ROOT / "examples/timeline-spell"


def ident(name):
    return str(uuid.uuid5(uuid.NAMESPACE_URL, "epok.example.fireball/" + name))


def main():
    folder = PROJECT / "assets/Effects"
    folder.mkdir(parents=True, exist_ok=True)
    textures = PROJECT / "assets/textures"
    textures.mkdir(parents=True, exist_ok=True)
    atlas = ROOT / "examples/rpg-2-5d-demo/assets/textures/characters-effects.epokasset"
    raw = atlas.read_bytes()
    texture_id = json.loads(raw[16:16 + struct.unpack_from("<I", raw, 8)[0]])["id"]
    for suffix in ["png", "epokasset"]:
        shutil.copyfile(atlas.with_suffix("." + suffix), textures / ("characters-effects." + suffix))
    sprite = dict(enabled=True, texture=texture_id, region=[0, 32, 16, 16], size=[1, 1], pivot=[0.5, 0.5],
                  flip_x=False, flip_y=False, orientation="Spherical", color=[1, 1, 1], unlit=True, blend="Add", depth_bias=0)
    effect = dict(version=1, id=ident("asset"), name="Fireball", seed=87123, layers=[], timeline=dict(
        version=2, id=ident("timeline"), name="Fireball: cast, travel, impact, aftermath", duration_ticks=16384,
        timebase="q12_seconds", loop_mode="once", slots=[], tracks=[], events=[], markers=[], layout={}))
    properties = dict(opacity="775f36b0-2b5b-4920-aa9f-981b166507ef", size="dbd149e3-2164-4bd8-87bc-92f53bdc0f74",
                      position="a8ab3325-89dd-416f-a946-082292c297c6", playing="37e43711-dba0-4bca-8499-952a73450d44")

    def layer(name, position, emitter=None, **style):
        s = dict(copy.deepcopy(sprite), **style)
        if emitter is None:
            content = dict(kind="sprite", sprite=s, frames=1, columns=1, frame_ticks=410)
        else:
            p = dict(enabled=True, play_on_start=False, continuous=False, rate=24, burst=0, max_particles=64,
                     lifetime=0.75, velocity=[0, 0.4, 0], spread=[0.3, 0.3, 0.3], gravity=[0, -0.2, 0],
                     start_size=0.25, end_size=0.02, start_color=[1, 0.6, 0.15], end_color=[0.2, 0.03, 0],
                     local_space=False, seed=1, sprite=s, frames=4, frame_columns=4, frame_duration=0.1)
            p.update(emitter)
            content = dict(kind="emitter", emitter=p)
        effect["layers"].append(dict(id=ident(name), slot=ident(name + "/slot"), name=name, enabled=True,
                                     position=position, content=content))
        effect["timeline"]["slots"].append(dict(id=ident(name + "/slot"), name=name, required=True,
            target=dict(kind="effect_layer_ref", **{"class": "27550d6d-5bba-4618-9d9e-33b9605bfb6c"})))

    def track(layer_name, prop, keys, mode="linear"):
        name = layer_name + "/" + prop
        ty = {"kind": "vector", "length": 3} if prop == "position" else {"kind": "bool" if prop == "playing" else "fixed"}
        effect["timeline"]["tracks"].append(dict(id=ident(name), name=name, slot=ident(layer_name + "/slot"),
            property=properties[prop], value_type=ty, priority=0, blend="absolute", restore="leave_final", interpolation=mode,
            keys=[dict(id=ident(name + "/key/" + str(i)), tick=t, value=v) for i, (t, v) in enumerate(keys)]))

    def burst(name, count, tick):
        effect["timeline"]["events"].append(dict(id=ident(name + "/burst"), name=name + " burst", slot=ident(name + "/slot"),
            function="c2650304-c940-408c-a8a8-05641c2a0489", keys=[dict(id=ident(name + "/burst/key"), tick=tick,
                arguments={"count": dict(kind="literal", value_type={"kind": "uint32"}, value=count)})]))

    layer("Cast sigil", [-2, 0.5, 0], color=[1, 0.45, 0.08])
    track("Cast sigil", "opacity", [(0, 0), (256, 1), (1792, 1), (2048, 0)])
    track("Cast sigil", "size", [(0, 0.2), (2048, 1.8)], "smoothstep")
    layer("Projectile core", [-2, 1, 0], size=[0.7, 0.7], color=[1, 0.8, 0.3])
    track("Projectile core", "opacity", [(0, 0), (2048, 1), (6144, 0)], "step")
    travel = [(0, [-2, 1, 0]), (2048, [-2, 1, 0]), (6144, [2, 1, 0]), (16384, [2, 1, 0])]
    track("Projectile core", "position", travel)
    layer("Travel embers", [-2, 1, 0], dict(continuous=True, lifetime=0.6, velocity=[-0.5, 0.1, 0], rate=32, max_particles=48))
    track("Travel embers", "position", travel)
    track("Travel embers", "playing", [(0, False), (2048, True), (6144, False)], "step")
    layer("Impact flash", [2, 1, 0], color=[1, 0.75, 0.35])
    track("Impact flash", "opacity", [(0, 0), (6143, 0), (6144, 1), (7600, 0)])
    track("Impact flash", "size", [(6144, 0.3), (7600, 3)], "ease_out")
    layer("Impact sparks", [2, 1, 0], dict(velocity=[0, 1.5, 0], spread=[2, 2, 2], gravity=[0, -3, 0],
        lifetime=0.7, start_size=0.15, end_size=0.01, max_particles=64))
    burst("Impact sparks", 32, 6144)
    layer("Aftermath smoke", [2, 0.5, 0], dict(velocity=[0, 0.7, 0], spread=[0.4, 0.2, 0.4], gravity=[0, 0, 0],
        lifetime=1.5, start_size=0.3, end_size=1.2, max_particles=32, start_color=[0.4, 0.3, 0.25], end_color=[0.02, 0.02, 0.02]), blend="Average")
    burst("Aftermath smoke", 12, 8192)
    layer("Residual glow", [2, 0.1, 0], size=[1.8, 0.6], color=[1, 0.2, 0.04])
    track("Residual glow", "opacity", [(0, 0), (8191, 0), (8192, 0.7), (16384, 0)])
    for name, tick in [("Cast", 0), ("Launch", 2048), ("Impact", 6144), ("Aftermath", 8192)]:
        effect["timeline"]["markers"].append(dict(id=ident("marker/" + name), name=name, tick=tick))
    (folder / "Fireball.particle-effect.json").write_text(json.dumps(effect, indent=2) + "\n", encoding="utf-8")
    print("Authored Fireball:", effect["id"], "layers", len(effect["layers"]), "tracks", len(effect["timeline"]["tracks"]) + len(effect["timeline"]["events"]))


if __name__ == "__main__":
    main()
