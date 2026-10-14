"""Play SubViewport2D pictures and a SplitScreen2D in PCSX-Redux through the editor's MCP server.

Run after cargo build:
    python tests/integration/verify_world2d_views.py [--editor PATH]

Creates an isolated Platformer 2D project and adds, straight into its map:
  * a zoomed-out MapCamera, a SubViewport2D drawing through it, and a Sprite2D on the
    HUD layer showing that picture in the top-right corner (a minimap);
  * then a SplitScreen2D showing the player's camera and the MapCamera side by side.
Checks in the emulator that the minimap corner holds the picture (its background
colour, then the level), and that the split screen draws its black divider with two
different halves. Captures are kept under .epok/world2d-views-*/ for review.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
_sys.path.insert(0, str(_Path(__file__).resolve().parent))
import epok_documents as documents
import argparse
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
TRANSFORM = {"class_id": "27887770-a779-4a16-863c-5abd32786cfa", "name": "epok::SceneComponent2D"}
CAMERA = {"class_id": "7245e90e-d900-4976-b738-0f95caa1d15d", "name": "epok::Camera2DComponent"}
SPRITE = {"class_id": "892ac5c9-eae8-48b8-853a-db93d20a537b", "name": "epok::Sprite2DComponent"}
PICTURE = {"class_id": "f5dc7806-fd18-40aa-9984-3b475e48df5d", "name": "epok::SubViewport2DComponent"}
SPLIT = {"class_id": "54813814-1419-4a34-b8f1-6183acb48376", "name": "epok::SplitScreen2DComponent"}
MINIMAP_BACKGROUND = (24, 32, 96)


def actor(name, position, components, attach=None):
    root = {"class": TRANSFORM, "id": str(uuid.uuid4()), "name": "Transform 2D", "root": True,
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


def main():
    parser = argparse.ArgumentParser()
    default_editor = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
    parser.add_argument("--editor", type=_Path, default=default_editor)
    args = parser.parse_args()
    home = ROOT / ".epok" / f"world2d-views-{uuid.uuid4().hex[:12]}"
    project = home / "Views"
    env = dict(os.environ, LOCALAPPDATA=str(home / "local"), XDG_DATA_HOME=str(home / "local"))
    preferences = home / "local/Epok/Editor.epokprefs"
    preferences.parent.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    token = uuid.uuid4().hex + uuid.uuid4().hex
    documents.write_text(preferences, json.dumps({"mcp": {"enabled": True, "port": port, "token": token}}))
    subprocess.run([str(args.editor), "--create-project", str(project), "--name", "Views",
                    "--template", "platformer-2d", "--gameplay", "cpp"],
                   check=True, env=env, stdout=subprocess.DEVNULL)
    scene_path = project / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text())
    by_name = {a["name"]: a for a in scene["actors"]}
    map_camera = actor("MapCamera", (0, 0), [(CAMERA, "camera2d", {"current": False, "zoom": 0.25})])
    minimap = actor("Minimap", (0, 0), [(PICTURE, "subviewport2d", {
        "camera": map_camera["id"], "size": [96, 64], "background": list(MINIMAP_BACKGROUND), "interval": 2})])
    screen = actor("MinimapScreen", (260, 40), [(SPRITE, "sprite2d", {"viewport": minimap["id"]})],
                   attach=by_name["HUD"]["id"])
    scene["actors"] += [map_camera, minimap, screen]
    documents.write_text(scene_path, json.dumps(scene, indent=2))

    def play(name):
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
                                          "clientInfo": {"name": "epok-world2d-views", "version": "1"}})
            client.request("notifications/initialized", notification=True)
            for _ in range(3):
                client.tool("editor_control", action="play")
                state = wait_until(lambda: (lambda s: s if s["game_frame"] or not s["build_or_play_active"] else None)(
                    client.tool("editor_state")), timeout=600)
                if state["game_frame"]:
                    break
                time.sleep(2)
            else:
                raise AssertionError(f"Play did not start; see {project / '.epok/Build.log'}")
            time.sleep(1.5)
            png = client.screenshot("game", home / f"{name}.png")[1]
            client.tool("editor_control", action="stop")
            wait_until(lambda: not client.tool("editor_state")["build_or_play_active"], timeout=60)
            return Image.open(io.BytesIO(png)).convert("RGB")
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()

    def near(c, target, tolerance=24):
        return all(abs(a - b) <= tolerance for a, b in zip(c, target))

    frame = play("0-minimap")
    w, h = frame.size
    sx, sy = w / 320, h / 240
    box = [frame.getpixel((int((260 + dx) * sx), int((40 + dy) * sy))) for dx in range(-44, 45, 4) for dy in range(-28, 29, 4)]
    background = sum(near(c, MINIMAP_BACKGROUND) for c in box)
    assert background > len(box) // 8, f"The minimap's background shows in its corner ({background}/{len(box)})"
    assert len(set(box)) > 4, "The minimap draws the level, not a flat rectangle"

    scene = documents.loads(scene_path.read_text())
    scene["actors"].append(actor("Split", (0, 0), [(SPLIT, "split_screen2d", {
        "cameras": [by_name["Player"]["id"], map_camera["id"]], "divider": 4})]))
    documents.write_text(scene_path, json.dumps(scene, indent=2))
    frame = play("1-split")
    w, h = frame.size
    column = [frame.getpixel((w // 2, int(y * h / 240))) for y in range(20, 220, 10)]
    assert all(max(c) < 40 for c in column), f"The divider runs down the middle: {column[:4]}"
    left = frame.crop((0, 0, w // 2 - 8, h)).resize((40, 30))
    right = frame.crop((w // 2 + 8, 0, w, h)).resize((40, 30))
    assert list(left.get_flattened_data()) != list(right.get_flattened_data()), "The two halves show different cameras"
    documents.write_text(home / "result.json", json.dumps({"passed": True}, indent=2))
    print(f"SubViewport2D minimap and SplitScreen2D played in the emulator; captures in {home}")


if __name__ == "__main__":
    main()
