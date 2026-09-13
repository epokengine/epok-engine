"""Rebuild the small original pixel-art acceptance fixture (standard library only)."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import copy
import hashlib
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[2] / "examples/rpg-2-5d-demo"
for directory in ["assets/scenes", "assets/scripts", "assets/textures", "ProjectSettings"]:
    (ROOT / directory).mkdir(parents=True, exist_ok=True)

def save(path, value):
    documents.write_text(ROOT / path, json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

def png(w, h, pixel):
    def chunk(name, data):
        return struct.pack(">I", len(data)) + name + data + struct.pack(">I", zlib.crc32(name + data) & 0xffffffff)
    data = b"".join(b"\0" + bytes(c for x in range(w) for c in pixel(x, y)) for y in range(h))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(data)) + chunk(b"IEND", b"")

ATLAS = "ff7585e7-44f2-44aa-a22d-022fa83fdb11"
STONE = "ed2240de-a2c8-44e5-86aa-6a33c7570791"
WATER = "6dc37680-4e0f-4827-80a6-047c84b5d449"
def atlas(x, y):
    f, x = divmod(x, 16)
    if y < 32:
        shift = [0, 1, 0, -1][f]
        if 4 <= x <= 11 and 3 <= y <= 10:
            return (37, 37, 57, 255) if y < 6 or x in [4, 11] else (235, 182, 130, 255)
        if 3 <= x <= 12 and 11 <= y <= 22:
            return (45, 152, 184, 255) if y < 20 else (224, 169, 53, 255)
        if ((4 <= x <= 6 and 23 <= y <= 28 + shift) or (9 <= x <= 11 and 23 <= y <= 28 - shift)):
            return (55, 58, 86, 255)
        return (0, 0, 0, 0)
    if y < 48:
        y -= 32
        radius = [6, 5, 4, 3][f]
        d = abs(x - 7) + abs(y - 7)
        if d < radius:
            return (255, 235 if d < 3 else 130, 60 if d < 3 else 26, 128)
        return (0, 0, 0, 0)
    if 2 <= x <= 13 and 50 <= y <= 61:
        return (88, 189, 108, 255) if f == 0 else (70, 108, 196, 255)
    return (0, 0, 0, 0)

def texture(name, ident, data):
    source = f"assets/textures/{name}.png"
    (ROOT / source).write_bytes(data)
    metadata = json.dumps(dict(version=2, id=ident, kind="Texture", importer_version=1, source=source,
                               source_hash=hashlib.sha256(data).hexdigest(), settings={"type": "Texture"}), separators=(",", ":")).encode()
    (ROOT / f"assets/textures/{name}.epokasset").write_bytes(b"EPOKAS01" + struct.pack("<II", len(metadata), len(data)) + metadata + data)

texture("characters-effects", ATLAS, png(64, 64, atlas))
texture("stone", STONE, png(32, 32, lambda x, y: (58, 73, 82, 255) if y % 8 == 0 or (x + (y // 8 % 2) * 8) % 16 == 0 else (110 + (x * 3 + y * 7) % 4 * 5, 127, 132, 255)))
texture("water", WATER, png(32, 32, lambda x,y: [(24,80,136,128),(40,120,184,128),(64,160,216,128),(112,192,232,128)][((x//4)+(y//3))%4]))

def entity(name, kind="Empty", pos=(0, 0, 0), scale=(1, 1, 1), **components):
    return dict(name=name, kind=kind, position=list(pos), rotation=[0, 0, 0], scale=list(scale), **components)

sprite = dict(texture=ATLAS, region=[0, 0, 16, 32], size=[0.8, 1.6], pivot=[0.5, 0], orientation="Upright")
emitter = dict(sprite=dict(texture=ATLAS, region=[0, 32, 16, 16], size=[1, 1], orientation="Spherical", unlit=True, blend="Add"),
               frames=4, frame_columns=4, frame_duration=0.1, lifetime=1.3, start_size=0.6, end_size=0.1,
               velocity=[0, 1.4, 0], spread=[0.5, 0.3, 0.5], gravity=[0, -0.2, 0], start_color=[1, 0.8, 0.3], end_color=[0.25, 0.05, 0], max_particles=48)
entities = [entity("Main Camera", "Camera", (0, 4, -7))]
entities[0]["rotation"] = [24, 0, 0]
entities += [entity("Ground", "Mesh", (0, -0.2, 1), (7, 0.4, 7), material=dict(texture=STONE, color=[0.6, 0.7, 0.75]), collider={}),
             entity("Occluder", "Mesh", (0, 0.65, 0.8), (1.6, 1.3, 0.65), material=dict(texture=STONE), collider={}),
             entity("Player", pos=(-1.7, 0, -0.6), sprite=sprite, sprite_animator=dict(sheet=dict(clips=[dict(name="Walk", looping=True, frames=[dict(region=[16 * i, 0, 16, 32], duration=0.15, event=i + 1) for i in range(4)])])),
                    collider=dict(center=[0, 0.45, 0], half_extents=[0.22, 0.45, 0.22]), blob_shadow=dict(radius=0.45, strength=0.25, distance=2), script=dict(name="RoadmapDemo")),
             entity("Behind occluder", pos=(0, 0, 1.8), sprite=dict(sprite, color=[0.8, 0.6, 1])),
             entity("Front of occluder", pos=(0, 0, -0.3), sprite=dict(sprite, flip_x=True, color=[1, 0.85, 0.4])),
             entity("Campfire", pos=(2, 0.2, 1), particle_emitter=dict(emitter, continuous=True, rate=16)),
             entity("Sparks", pos=(-1.7, 0.3, -0.6), particle_emitter=dict(emitter, continuous=False, play_on_start=False, burst=24, spread=[1.4, 1, 1.4], gravity=[0, -2, 0])),
             entity("Portal trigger", pos=(2.5, 0.5, -1.5), collider=dict(trigger=True, half_extents=[0.6, 0.5, 0.6]), sprite=dict(texture=ATLAS, region=[16, 48, 16, 16], orientation="Fixed", size=[1, 1], unlit=True)),
             entity("Sun", light=dict(kind="Directional", intensity=0.7, color=[1, 0.88, 0.75])),
             entity("HUD", canvas={})]
hud = len(entities)-1
entities += [entity("Portrait", parent=hud, rect=dict(anchor_min=[0, 1], anchor_max=[0, 1], pivot=[0, 1], position=[8, -8], size=[24, 40]), image=dict(texture=ATLAS, region=[0, 0, 16, 32], color=[1, 1, 1])),
             entity("Controls", parent=hud, rect=dict(anchor_min=[0, 1], anchor_max=[0, 1], pivot=[0, 1], position=[38, -8], size=[275, 35]), text=dict(text="D-pad: move  X: burst  O: pool\nSELECT: scene  START: pause", color=[1, 0.95, 0.8], wrap=True)),
             entity("Status", parent=hud, rect=dict(anchor_min=[0, 0], anchor_max=[0, 0], pivot=[0, 0], position=[8, 8], size=[305, 22]), text=dict(text="Explore! Walk around the wall.", wrap=True))]
imported = ROOT / "assets/models/ramp.epokasset"
if imported.exists():
    package=imported.read_bytes()
    metadata=json.loads(package[16:16+struct.unpack("<I",package[8:12])[0]])
    entities.append(entity("Imported stone ramp", "Mesh", (-2.4, 0.02, 2.6), editable_mesh=dict(asset=metadata["id"])))
entities.append(entity("Water", "Mesh", (1.6,0.04,2.7), (1.8,0.035,1.4),material=dict(texture=WATER,color=[0.6,0.8,1],blend="Average",uv_scroll=[0.12,0.04]),palette_animator=dict(enabled=True,texture=WATER,first=1,last=2,speed=4,reverse=False)))
save("2.5D Engine Acceptance.epokproject", dict(format_version=1, editor_version="0.1.0", name="2.5D Engine Acceptance", startup_scene="assets/scenes/Courtyard.epokmap", auto_build=True,rendering=dict(width=640,height=480)))
save("ProjectSettings/Maps.epoksettings", dict(scenes=["assets/scenes/Night.epokmap"]))
scene = dict(version=1, name="Courtyard", entities=entities, environment=dict(ambient=[0.45, 0.45, 0.5]))
save("assets/scenes/Courtyard.epokmap", scene)
night = copy.deepcopy(scene)
night["name"] = "Night"
night["environment"]["ambient"] = [0.15, 0.2, 0.4]
night["fog"]=dict(enabled=True,start=10,end=24,color=[0.1,0.15,0.28])
night["entities"][2]["position"][0] = 0.7
night["entities"][9]["light"]["color"] = [0.4, 0.55, 1]
night["entities"][6]["particle_emitter"]["start_color"] = [0.2, 0.5, 1]
save("assets/scenes/Night.epokmap", night)
save("assets/scripts/RoadmapDemo.epokscript", dict(name="RoadmapDemo", properties=[]))
documents.write_text(ROOT / ".gitignore", "/.epok/\n/UserSettings/\n/exports/\n/artifacts/\n/Local.epokconfig\n")
print(f"Created deterministic demo assets and scenes in {ROOT}")
