"""Play HUD check boxes, sliders and a string table in PCSX-Redux through the editor's MCP server.

Run after cargo build:
    python tests/integration/verify_hud_widgets.py [--editor PATH]

Creates an isolated project with a two-language string table, a keyed label, a
check box and a slider, and a C++ component on the camera that switches to the
second language in begin_play. Checks in the emulator that:
  * the keyed label shows the second language, pixel-identical to a plain label
    authored with that text;
  * Cross flips the focused check box (its mark disappears);
  * Down moves the focus to the slider and Right raises its value by one step.
Captures are kept under .epok/hud-widgets-integration-*/ for review. Requires the
PSX SDK and emulator.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
_sys.path.insert(0, str(_Path(__file__).resolve().parent))
import epok_documents as documents
import argparse
import hashlib
import io
import json
import os
import socket
import struct
import subprocess
import time
import uuid

from PIL import Image
from mcp import Client, wait_until

ROOT = _Path(__file__).resolve().parents[2]
RIGHT, DOWN, CROSS = 1 << 5, 1 << 6, 1 << 14
SWITCH_ID = "3c1f7a52-8e0b-4d7e-9f53-2a6b1c4d8e91"


def string_table(path):
    """Writes an authored String Table package (magic, metadata, JSON source)."""
    source = json.dumps({"version": 1, "languages": ["English", "Spanish"],
                         "entries": [{"key": "menu.quit", "values": ["Quit", "Salir"]}]}).encode()
    meta = json.dumps({"version": 2, "id": str(uuid.uuid4()), "kind": "StringTable", "importer_version": 1,
                       "source": "assets/Localization/Strings.epokasset",
                       "source_hash": hashlib.sha256(source).hexdigest(),
                       "settings": {"type": "Authored"}}).encode()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(meta), len(source)) + meta + source)


SWITCH = f'''#pragma once
#include "epok.hpp"

// Switches the game to another language of the project's string table when play starts.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D|UI, Id="{SWITCH_ID}") LanguageSwitch : public epok::ActorComponent {{
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t language = 1;
    void begin_play() override {{ epok::LocalizationLibrary::set_language(language); }}
}};
'''


def rect(x, y, w, h):
    return {"anchor_min": [0, 1], "anchor_max": [0, 1], "pivot": [0, 1], "position": [x, y], "size": [w, h]}


def count(png, test):
    image = Image.open(io.BytesIO(png)).convert("RGB")
    return sum(1 for p in image.get_flattened_data() if test(*p))


def mark(r, g, b):
    return g > 170 and r < 110 and b < 120


def fill(r, g, b):
    return r > 200 and g < 70 and b < 70


def main():
    parser = argparse.ArgumentParser()
    default_editor = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
    parser.add_argument("--editor", type=_Path, default=default_editor)
    args = parser.parse_args()
    home = ROOT / ".epok" / f"hud-widgets-integration-{uuid.uuid4().hex[:12]}"
    project = home / "Widgets"
    env = dict(os.environ, LOCALAPPDATA=str(home / "local"), XDG_DATA_HOME=str(home / "local"))
    preferences = home / "local/Epok/Editor.epokprefs"
    preferences.parent.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    token = uuid.uuid4().hex + uuid.uuid4().hex
    documents.write_text(preferences, json.dumps({"mcp": {"enabled": True, "port": port, "token": token}}))
    subprocess.run([str(args.editor), "--create-project", str(project), "--name", "Widgets", "--gameplay", "cpp"],
                   check=True, env=env, stdout=subprocess.DEVNULL)
    string_table(project / "assets/Localization/Strings.epokasset")
    documents.write_text(project / "assets/scripts/LanguageSwitch.hpp", SWITCH)
    documents.write_text(project / "assets/scripts/LanguageSwitch.cpp", '#include "LanguageSwitch.hpp"\n')
    # The switch rides on the template's camera; the HUD elements are added through MCP below.
    scene_path = project / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text())
    camera = next(a for a in scene["actors"] if a["name"] == "Main Camera")
    camera["components"].append({"class": {"class_id": SWITCH_ID, "name": "LanguageSwitch"},
                                 "id": str(uuid.uuid4()), "name": "LanguageSwitch"})
    documents.write_text(scene_path, json.dumps(scene, indent=2))
    process = subprocess.Popen([str(args.editor), "--project", str(project), "--window-size", "1280x800"], env=env,
                               stdout=subprocess.DEVNULL, stderr=(home / "editor.log").open("w"))
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
                                      "clientInfo": {"name": "epok-hud-widgets", "version": "1"}})
        client.request("notifications/initialized", notification=True)

        def revision():
            return client.tool("scene_actors")["revision"]

        def settled(name, **arguments):
            """Calls a tool once the editor accepts edits (asset scans and reflection finish first)."""
            def attempt():
                try:
                    return client.tool(name, revision=revision(), **arguments) or True
                except AssertionError as error:
                    last[0] = error
                    return False
            last = [None]
            try:
                return wait_until(attempt, timeout=240)
            except AssertionError:
                raise AssertionError(f"{name} kept failing: {last[0]}") from None

        base = len(client.tool("scene_actors")["actors"])
        canvas, label, toggle, slider = base, base + 1, base + 2, base + 3
        created = settled("scene_apply", operations=[
            {"op": "create", "actor": {"name": "Canvas", "kind": "Empty", "canvas": {"enabled": True}}},
            {"op": "create", "actor": {"name": "Label", "kind": "Empty", "parent": canvas, "rect": rect(24, -24, 200, 16),
                                       "text": {"text": "Quit", "key": "menu.quit", "color": [1, 1, 1], "wrap": False}}},
            {"op": "create", "actor": {"name": "Toggle", "kind": "Empty", "parent": canvas, "rect": rect(24, -56, 140, 20),
                                       "text": {"text": "Option", "color": [1, 1, 1], "wrap": False},
                                       "focusable": {"enabled": True, "kind": "Toggle", "checked": True}}},
            {"op": "create", "actor": {"name": "Slider", "kind": "Empty", "parent": canvas, "rect": rect(24, -96, 160, 12),
                                       "progress": {"enabled": True, "value": 0.5, "step": 0.1,
                                                    "color": [1, 0, 0], "background": [0.12, 0.12, 0.12]},
                                       "focusable": {"enabled": True}}},
            # Links are validated against the actors that exist, so they come last.
            {"op": "update", "index": toggle, "patch": {"focusable": {"neighbors": [-1, -1, -1, slider]}}},
            {"op": "update", "index": slider, "patch": {"focusable": {"neighbors": [-1, -1, toggle, -1]}}},
            {"op": "update", "index": canvas, "patch": {"canvas": {"focused": toggle}}},
        ])
        assert [r["index"] for r in created["results"]][:4] == [canvas, label, toggle, slider], created
        client.tool("scene_save", revision=revision())

        def play(name):
            # A script refresh right after opening can cancel the first build; Play again.
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
            return client.screenshot("game", home / f"{name}.png")[1]

        def stop():
            client.tool("editor_control", action="stop")
            wait_until(lambda: not client.tool("editor_state")["build_or_play_active"], timeout=60)

        keyed = play("0-keyed-spanish")
        marked, filled = count(keyed, mark), count(keyed, fill)
        assert marked > 20, f"The checked box draws its mark ({marked} pixels)"
        assert filled > 100, f"The slider draws its fill ({filled} pixels)"
        client.tool("game_input", buttons=CROSS, duration_ms=200)
        time.sleep(0.6)
        flipped = client.screenshot("game", home / "1-unchecked.png")[1]
        assert count(flipped, mark) < marked // 4, "Cross must clear the focused check box"
        client.tool("game_input", buttons=DOWN, duration_ms=200)
        time.sleep(0.6)
        client.tool("game_input", buttons=RIGHT, duration_ms=200)
        time.sleep(0.6)
        raised = client.screenshot("game", home / "2-slider-raised.png")[1]
        grown = count(raised, fill)
        assert grown > filled * 1.1, f"Right must raise the focused slider ({filled} -> {grown} pixels)"
        stop()

        # The same label authored as plain text in the second language. The switch still
        # runs, but only keyed texts follow the language.
        client.tool("scene_apply", revision=revision(), operations=[
            {"op": "update", "index": label, "patch": {"text": {"text": "Salir", "key": ""}}},
        ])
        client.tool("scene_save", revision=revision())
        plain = play("3-plain-spanish")
        stop()
        assert keyed == plain, "The keyed label in the second language must equal the plain label"
        documents.write_text(home / "result.json", json.dumps({"passed": True}, indent=2))
        print(f"HUD check box, slider and string table played in the emulator; captures in {home}")
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()


if __name__ == "__main__":
    main()
