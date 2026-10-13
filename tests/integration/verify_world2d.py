"""Play the Platformer 2D template in PCSX-Redux through the editor's MCP server.

Run after cargo build:
    python tests/integration/verify_world2d.py [--flavor cpp|lua|blueprint] [--editor PATH]

Creates an isolated project from the template, plays it in the emulator, holds
Right, jumps with Cross and captures the Game view at each step. The captures
must differ (the player and the following camera moved) and are kept under
.epok/world2d-integration-*/ for review. Requires the PSX SDK and emulator.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
_sys.path.insert(0, str(_Path(__file__).resolve().parent))
import epok_documents as documents
import argparse
import json
import os
import socket
import subprocess
import time
import uuid

from mcp import Client, wait_until

ROOT = _Path(__file__).resolve().parents[2]
RIGHT, LEFT, CROSS = 1 << 5, 1 << 7, 1 << 14


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--flavor", default="cpp", choices=["cpp", "lua", "blueprint"])
    default_editor = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
    parser.add_argument("--editor", type=_Path, default=default_editor)
    args = parser.parse_args()
    home = ROOT / ".epok" / f"world2d-integration-{uuid.uuid4().hex[:12]}"
    project = home / "Platformer"
    env = dict(os.environ, LOCALAPPDATA=str(home / "local"), XDG_DATA_HOME=str(home / "local"))
    preferences = home / "local/Epok/Editor.epokprefs"
    preferences.parent.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    token = uuid.uuid4().hex + uuid.uuid4().hex
    documents.write_text(preferences, json.dumps({"mcp": {"enabled": True, "port": port, "token": token}}))
    subprocess.run([str(args.editor), "--create-project", str(project), "--name", "Platformer",
                    "--template", "platformer-2d", "--gameplay", args.flavor],
                   check=True, env=env, stdout=subprocess.DEVNULL)
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
                                      "clientInfo": {"name": "epok-world2d", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        client.tool("editor_control", action="play")
        wait_until(lambda: client.tool("editor_state")["game_frame"], timeout=240)
        time.sleep(1.5)
        _, start = client.screenshot("game", home / "0-start.png")
        client.tool("game_input", buttons=RIGHT, duration_ms=1500)
        time.sleep(1.8)
        _, ran = client.screenshot("game", home / "1-ran-right.png")
        client.tool("game_input", buttons=RIGHT | CROSS, duration_ms=250)
        time.sleep(0.3)
        _, jumped = client.screenshot("game", home / "2-jumping.png")
        time.sleep(1.2)
        client.tool("game_input", buttons=LEFT, duration_ms=600)
        time.sleep(0.9)
        _, back = client.screenshot("game", home / "3-ran-left.png")
        assert start != ran, "Holding Right must move the player and the camera"
        assert ran != jumped, "Cross must make the player jump"
        assert jumped != back, "Holding Left must move the player back"
        client.tool("editor_control", action="stop")
        wait_until(lambda: not client.tool("editor_state")["build_or_play_active"], timeout=60)
        documents.write_text(home / "result.json", json.dumps({"passed": True, "flavor": args.flavor}, indent=2))
        print(f"World2D platformer ({args.flavor}) played in the emulator; captures in {home}")
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()


if __name__ == "__main__":
    main()
