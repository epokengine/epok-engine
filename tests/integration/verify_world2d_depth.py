"""Play 2D canvases under the 3D world and Depth2D masks inside it, in PCSX-Redux and on Native PC.

Run after cargo build:
    python tests/integration/verify_world2d_depth.py [--editor PATH] [--skip-native]

Creates an isolated Basic project whose camera looks down +Z from z = -6, and adds a white
unlit cube in front of it, a green background on a screen canvas drawn Under 3D (covering
x 0-256) and a magenta 64 x 64 mask at the screen centre, attached to a node with a
Depth2D. Each run moves the cube or the mask's depth and captures Game:
  * near: the cube 3 units away, in front of the mask at 6 units, covers the mask and the
    background around it, while the background stays visible above it;
  * far: the cube 10 units away is hidden behind the mask;
  * anchor: the mask anchored to a 3D marker 14 units away lets the same far cube cover it.
Native PC runs the near and far cases through its own compositor. Captures are kept under
.epok/world2d-depth-*/ for review.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
_sys.path.insert(0, str(_Path(__file__).resolve().parent))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import copy
import io
import json
import os
import socket
import subprocess
import time
import uuid

from PIL import Image
from mcp import Client, wait_until

ROOT = _Path(__file__).resolve().parents[2]
ACTOR2D = {"class_id": "5308054e-0aaa-4d53-963b-440cf0c71916", "name": "epok::Actor2D"}
TRANSFORM2D = {"class_id": "27887770-a779-4a16-863c-5abd32786cfa", "name": "epok::SceneComponent2D"}
SPRITE = {"class_id": "892ac5c9-eae8-48b8-853a-db93d20a537b", "name": "epok::Sprite2DComponent"}
CANVAS = {"class_id": "bd384e4b-5903-485c-b114-5ffb6a330538", "name": "epok::CanvasLayer2DComponent"}
DEPTH = {"class_id": "a116bf9a-c98d-4d4b-b6df-0bb237ba81ee", "name": "epok::Depth2DComponent"}
ACTOR3D = {"class_id": "fc24ce9b-558c-49de-bc35-e040f350e486", "name": "epok::Actor3D"}
TRANSFORM3D = {"class_id": "ed73d249-b6cb-4a3c-a0e8-696de55e286f", "name": "epok::SceneComponent3D"}
MESH = {"class_id": "580f99b1-c905-4f96-b34f-807c51335ba0", "name": "epok::Mesh3DComponent"}
SKY = (24, 24, 96)
GREEN = (0, 200, 0)
MAGENTA = (248, 0, 248)
WHITE = (248, 248, 248)


def actor2d(name, position, components, attach=None):
    root = {"class": TRANSFORM2D, "id": str(uuid.uuid4()), "name": "Transform 2D", "root": True,
            "overrides": ["position", "rotation", "scale"],
            "properties": {"position": list(position), "rotation": 0.0, "scale": [1.0, 1.0]}}
    document = {"active": True, "class": ACTOR2D, "id": str(uuid.uuid4()), "name": name, "overrides": [],
                "properties": {}, "components": [root]}
    for cls, key, record in components:
        document["components"].append({"class": cls, "id": str(uuid.uuid4()), "name": cls["name"].split("::")[1],
                                       "overrides": [key], "properties": {key: record}})
    if attach:
        document["attach"] = {"actor": attach}
        document["logical_parent"] = attach
    return document


def actor3d(name, position, scale, mesh):
    components = [{"class": TRANSFORM3D, "id": str(uuid.uuid4()), "name": "Transform", "root": True,
                   "overrides": ["position", "rotation", "scale"],
                   "properties": {"position": list(position), "rotation": [0.0, 0.0, 0.0], "scale": [scale] * 3}}]
    if mesh:
        components.append({"class": MESH, "id": str(uuid.uuid4()), "name": "Mesh Renderer",
                           "overrides": ["editable_mesh", "lighting", "material", "skeletal_mesh", "terrain"],
                           "properties": {
                               "editable_mesh": None, "skeletal_mesh": None, "terrain": None,
                               "lighting": {"background_pass": False, "cast_shadows": False, "receive": "Realtime",
                                            "static_geometry": False, "subdivisions": 1},
                               "material": {"blend": "Cutout", "color": [1.0, 1.0, 1.0], "depth_bias": 0,
                                            "double_sided": False, "texture": None, "unlit": True,
                                            "uv_scroll": [0.0, 0.0]}}})
    return {"active": True, "class": ACTOR3D, "id": str(uuid.uuid4()), "name": name, "overrides": [],
            "properties": {}, "components": components}


def main():
    parser = argparse.ArgumentParser()
    default_editor = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
    parser.add_argument("--editor", type=_Path, default=default_editor)
    parser.add_argument("--skip-native", action="store_true")
    args = parser.parse_args()
    home = ROOT / ".epok" / f"world2d-depth-{uuid.uuid4().hex[:12]}"
    project = home / "Depth"
    env = dict(os.environ, LOCALAPPDATA=str(home / "local"), XDG_DATA_HOME=str(home / "local"))
    preferences = home / "local/Epok/Editor.epokprefs"
    preferences.parent.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    token = uuid.uuid4().hex + uuid.uuid4().hex
    documents.write_text(preferences, json.dumps({"mcp": {"enabled": True, "port": port, "token": token}}))

    def run(*arguments):
        result = subprocess.run([str(args.editor), *map(str, arguments)], env=env, capture_output=True, text=True,
                                timeout=300)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    run("--create-project", project, "--name", "Depth", "--template", "basic", "--gameplay", "cpp")
    textures = project / "assets/textures"
    textures.mkdir(parents=True, exist_ok=True)
    Image.new("RGBA", (256, 240), GREEN + (255,)).save(textures / "background.png")
    Image.new("RGBA", (64, 64), MAGENTA + (255,)).save(textures / "mask.png")
    for name in ("background", "mask"):
        run("--project", project, "--import-texture", f"assets/textures/{name}.png")
    ids = {a["path"]: a["id"] for a in json.loads(run("--project", project, "--scan-assets"))["assets"]}
    background_id = ids["assets/textures/background.epokasset"]
    mask_id = ids["assets/textures/mask.epokasset"]

    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["auto_build"] = False
    # Screen pixels below are 320 x 240 progressive ones.
    manifest["rendering"] = dict(manifest.get("rendering") or {}, width=320, height=240)
    scene_path = project / manifest["startup_scene"]
    base = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = next(a for a in base["actors"] if a["name"] == "Main Camera")
    camera["components"][0]["properties"]["position"] = [0.0, 0.0, -6.0]
    camera["components"][0]["properties"]["rotation"] = [0.0, 0.0, 0.0]
    for component in camera["components"]:
        if component["name"] == "Camera":
            component["properties"]["camera_sky_color"] = [c / 255 for c in SKY]

    def scene(cube_z, depth):
        document = copy.deepcopy(base)
        cube = actor3d("Cube", (0.0, 0.0, cube_z), 2.0, True)
        marker = actor3d("Marker", (0.0, 0.0, 8.0), 1.0, False)
        backdrop = actor2d("Backdrop", (0, 0), [(CANVAS, "canvas_layer2d", {"layer": 0, "under_3d": True})])
        background = actor2d("Background", (128, 120), [(SPRITE, "sprite2d", {"texture": background_id})],
                             attach=backdrop["id"])
        record = dict(depth)
        if record.pop("anchor", False):
            record["anchor"] = marker["id"]
        masks = actor2d("Masks", (0, 0), [(CANVAS, "canvas_layer2d", {"layer": 1}), (DEPTH, "depth2d", record)])
        mask = actor2d("Pillar", (160, 120), [(SPRITE, "sprite2d", {"texture": mask_id})], attach=masks["id"])
        document["actors"] += [cube, marker, backdrop, background, masks, mask]
        documents.write_text(scene_path, json.dumps(document, indent=2))

    def play(name, runtime):
        manifest["play"] = dict(manifest.get("play") or {}, runtime=runtime)
        documents.write_text(manifest_path, json.dumps(manifest, indent=2))
        process = subprocess.Popen([str(args.editor), "--project", str(project), "--window-size", "1280x800"], env=env,
                                   stdout=subprocess.DEVNULL, stderr=(home / f"{name}-editor.log").open("w"))
        client = Client(port, token)
        try:
            def listening():
                assert process.poll() is None, "Editor exited"
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                        return True
                except OSError:
                    return False
            wait_until(listening)
            client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                          "clientInfo": {"name": "epok-world2d-depth", "version": "1"}})
            client.request("notifications/initialized", notification=True)
            ready = "native_frame" if runtime == "native_pc" else "game_frame"
            for _ in range(3):
                client.tool("editor_control", action="play")
                state = wait_until(lambda: (lambda s: s if s[ready] or not s["build_or_play_active"] else None)(
                    client.tool("editor_state")), timeout=600)
                if state[ready]:
                    break
                time.sleep(2)
            else:
                raise AssertionError(f"Play did not start; see {project / '.epok/Build.log'}")
            # The loading fade ends when the sky beyond the background shows its colour.
            for _ in range(10):
                time.sleep(1.5)
                png = client.screenshot("game", home / f"{name}.png")[1]
                frame = Image.open(io.BytesIO(png)).convert("RGB")
                if near(sample(frame, 300, 30), SKY):
                    break
            client.tool("editor_control", action="stop")
            wait_until(lambda: not client.tool("editor_state")["build_or_play_active"], timeout=60)
            return frame
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()

    def near(c, target, tolerance=40):
        return all(abs(a - b) <= tolerance for a, b in zip(c, target))

    def sample(frame, x, y):
        w, h = frame.size
        return frame.getpixel((int(x * w / 320), int(y * h / 240)))

    def check(frame, label, centre, around):
        found = {"centre": sample(frame, 160, 120), "beside the mask": sample(frame, 115, 120),
                 "above the cube": sample(frame, 40, 30), "past the background": sample(frame, 300, 30)}
        expected = {"centre": centre, "beside the mask": around, "above the cube": GREEN, "past the background": SKY}
        wrong = {k: (found[k], expected[k]) for k in found if not near(found[k], expected[k])}
        assert not wrong, f"{label}: (found, expected) {wrong}"

    results = {}
    cases = [("near", -3.0, {"distance": 6.0}, WHITE, WHITE),
             ("far", 4.0, {"distance": 6.0}, MAGENTA, GREEN),
             ("anchor", 4.0, {"distance": 6.0, "anchor": True}, WHITE, GREEN)]
    runtimes = ["play_station"] + ([] if args.skip_native else ["native_pc"])
    for runtime in runtimes:
        for label, cube_z, depth, centre, around in cases:
            if runtime == "native_pc" and label == "anchor":
                continue
            scene(cube_z, depth)
            name = f"{runtime}-{label}"
            check(play(name, runtime), name, centre, around)
            results[name] = True
    documents.write_text(home / "result.json", json.dumps({"passed": True, "cases": results}, indent=2))
    print(f"Under 3D canvases and Depth2D masks composed with the 3D world ({', '.join(results)}); captures in {home}")


if __name__ == "__main__":
    main()
