"""Exercise the real editor over MCP using only Python's standard library.

Run after cargo build: python tests/integration/mcp.py [--emulator]
All files and preferences belong to an isolated test project under .epok/.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse
import base64
import http.client
import io
import json
import os
from pathlib import Path
import queue
import socket
import struct
import subprocess
import threading
import time
import uuid
import wave

ROOT = Path(__file__).resolve().parents[2]


class Client:
    def __init__(self, port, token):
        self.port, self.token, self.session, self.sequence = port, token, None, 0

    def request(self, method, params=None, notification=False):
        self.sequence += 1
        payload = {"jsonrpc": "2.0", "method": method}
        if not notification:
            payload["id"] = self.sequence
        if params is not None:
            payload["params"] = params
        headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream",
                   "Authorization": f"Bearer {self.token}", "MCP-Protocol-Version": "2025-11-25"}
        if self.session:
            headers["Mcp-Session-Id"] = self.session
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=25)
        try:
            connection.request("POST", "/mcp", json.dumps(payload), headers)
            response = connection.getresponse()
            assert response.status in (200, 202), (response.status, response.read().decode())
            self.session = response.getheader("Mcp-Session-Id") or self.session
            if response.status == 202:
                return None
            if "text/event-stream" in response.getheader("Content-Type", ""):
                while line := response.readline():
                    if line.startswith(b"data:") and line[5:].strip():
                        value = json.loads(line[5:])
                        if value.get("id") == self.sequence:
                            break
                else:
                    raise AssertionError("SSE ended without a result")
            else:
                value = json.loads(response.read())
            assert "error" not in value, value
            return value["result"]
        finally:
            connection.close()

    def tool(self, name, **args):
        result = self.request("tools/call", {"name": name, "arguments": args})
        assert not result.get("isError"), result
        return json.loads(result["content"][0]["text"])

    def screenshot(self, target, path):
        result = self.request("tools/call", {"name": "viewer_screenshot", "arguments": {"target": target}})
        assert not result.get("isError"), result
        image = result["content"][0]
        assert image["type"] == "image" and image["mimeType"] == "image/png"
        data = base64.b64decode(image["data"])
        assert data[:8] == b"\x89PNG\r\n\x1a\n"
        path.write_bytes(data)
        return struct.unpack(">II", data[16:24]), data


def wait_until(function, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = function()
        if value:
            return value
        time.sleep(0.1)
    raise AssertionError("Timed out waiting for editor state")


def check_stdio(executable, env):
    child = subprocess.Popen([str(executable), "--mcp-stdio"], stdin=subprocess.PIPE,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
    lines = queue.Queue()
    threading.Thread(target=lambda: [lines.put(line) for line in child.stdout], daemon=True).start()
    def send(value):
        child.stdin.write(json.dumps(value) + "\n")
        child.stdin.flush()
    def receive(request_id):
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline:
            value = json.loads(lines.get(timeout=20))
            if value.get("id") == request_id:
                assert "error" not in value, value
                return value["result"]
        raise AssertionError("Stdio response timeout")
    try:
        send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "epok-stdio-test", "version": "1"}}})
        assert receive(1)["serverInfo"]["name"] == "epok-editor"
        send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        send({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "editor_state", "arguments": {}}})
        assert not receive(2).get("isError")
    finally:
        child.stdin.close()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.terminate()
            child.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    parser.add_argument("--editor", type=Path, default=ROOT / "target/debug/epok-editor.exe")
    args = parser.parse_args()
    home = ROOT / ".epok" / f"mcp-integration-{uuid.uuid4().hex}"
    project = home / "Game"
    env = dict(os.environ, LOCALAPPDATA=str(home / "local"), XDG_DATA_HOME=str(home / "local"))
    preferences = home / "local/Epok/Editor.epokprefs"
    preferences.parent.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    token = uuid.uuid4().hex + uuid.uuid4().hex
    documents.write_text(preferences, json.dumps({"mcp": {"enabled": True, "port": port, "token": token}}))
    subprocess.run([str(args.editor), "--create-project", str(project), "--name", "MCP Demo", "--template", "sample"],
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
        initialized = client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                                   "clientInfo": {"name": "epok-integration", "version": "1"}})
        assert initialized["serverInfo"]["name"] == "epok-editor"
        client.request("notifications/initialized", notification=True)
        assert len(client.request("tools/list")["tools"]) >= 19
        assert client.request("resources/read", {"uri": "epok://scene/current"})["contents"]
        check_stdio(args.editor, env)
        initial = client.tool("scene_read")
        edited = client.tool("scene_apply", revision=initial["revision"], operations=[
            {"op": "create", "entity": {"name": "AI Cube", "position": [0, 1, 0], "material": {"color": [0.2, 0.6, 1]}}}])
        client.tool("entity_select", revision=edited["revision"], index=edited["results"][0]["index"], frame=True)
        client.tool("editor_view", scene_2d=False)
        size, before = client.screenshot("scene", home / "scene-before.png")
        assert size == (960, 600), size
        client.tool("editor_view", view={"yaw": 1.3, "pitch": 0.4})
        _, after = client.screenshot("scene", home / "scene-after.png")
        assert before != after, "Camera changes must reach the captured viewport"
        assert client.screenshot("hud", home / "hud.png")[0] == (640, 480)
        client.tool("scene_history", action="undo", revision=edited["revision"])
        client.tool("scene_history", action="redo", revision=initial["revision"])
        client.tool("scene_save", revision=edited["revision"])
        sound = io.BytesIO()
        with wave.open(sound, "wb") as wav:
            wav.setnchannels(1); wav.setsampwidth(2); wav.setframerate(22050)
            wav.writeframes(b"\0\0" * 2205)
        client.tool("project_files", action="write", path="assets/audio/mcp.wav", revision="absent",
                    encoding="base64", content=base64.b64encode(sound.getvalue()).decode())
        client.tool("asset_import", source="assets/audio/mcp.wav", destination="assets/audio/mcp.epokasset")
        wait_until(lambda: not client.tool("editor_state")["import_active"])
        state = client.tool("editor_state")
        assert not state["import_error"], state
        assert any(a["path"] == "assets/audio/mcp.epokasset" for a in client.tool("asset_list")["assets"])
        if args.emulator:
            client.tool("editor_control", action="play")
            wait_until(lambda: client.tool("editor_state")["game_frame"], timeout=90)
            client.tool("game_input", buttons=16384, duration_ms=1000)
            wait_until(lambda: client.tool("editor_state")["game_frame"]["buttons"] & 16384)
            wait_until(lambda: not client.tool("editor_state")["game_frame"]["buttons"] & 16384)
            client.tool("editor_control", action="pause")
            wait_until(lambda: client.tool("editor_state")["paused"])
            client.tool("editor_control", action="step")
            assert client.screenshot("game", home / "game.png")[0] == (640, 480)
            client.tool("editor_control", action="stop")
            wait_until(lambda: not client.tool("editor_state")["build_or_play_active"])
        client.tool("editor_control", action="editor_preferences")
        client.screenshot("editor", home / "editor.png")
        documents.write_text(home / "result.json", json.dumps({"passed": True, "emulator": args.emulator, "project": str(project)}, indent=2))
        print(f"MCP integration passed: {home}")
    finally:
        try:
            client.tool("editor_control", action="stop")
            wait_until(lambda: not client.tool("editor_state")["build_or_play_active"], timeout=10)
        except Exception:
            pass
        finally:
            process.terminate()
            process.wait(timeout=10)


if __name__ == "__main__":
    main()
