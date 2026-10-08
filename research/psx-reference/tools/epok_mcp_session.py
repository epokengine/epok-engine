"""Keep one Epok editor alive and drive its existing MCP API for study QA.

No screen scraping or simulated keyboard input. Credentials/preferences are local
to a new session directory; the user's normal editor preferences are untouched.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import time
import uuid
from redux_host import default_engine


def frame_sequence(state):
    native = state.get('play_profile', {}).get('saved', {}).get('runtime') == 'native_pc'
    return (state.get('native_frame' if native else 'game_frame') or {}).get('sequence', 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, default=default_engine())
    parser.add_argument("--session", type=Path, required=True)
    parser.add_argument("--reconnect", action="store_true", help="Negotiate a fresh MCP session with the same editor; never replays an uncertain request")
    sub = parser.add_subparsers(dest="command", required=True)
    start = sub.add_parser("start")
    start.add_argument("--project", type=Path, required=True)
    start.add_argument("--editor", type=Path)
    call = sub.add_parser("call")
    call.add_argument("tool")
    call.add_argument("arguments", nargs="?", default="{}", help="JSON tool arguments")
    capture = sub.add_parser("capture")
    capture.add_argument("target", choices=["scene", "editor", "game", "hud"])
    capture.add_argument("output", type=Path)
    wait = sub.add_parser("wait", help="Read-only wait for asynchronous editor work; never relaunches or replays a mutation")
    wait.add_argument("state", choices=["idle", "running", "stopped"])
    wait.add_argument("--timeout", type=float, default=300)
    wait.add_argument("--frames", type=int, default=5)
    args = parser.parse_args()
    if args.command == "wait" and not (0 < args.timeout <= 600 and args.frames >= 0):
        parser.error("wait timeout must be 0..600 seconds (exclusive zero), frames must be nonnegative")
    wait_deadline = time.monotonic() + args.timeout if args.command == "wait" else None
    engine = args.engine.resolve()
    spec = importlib.util.spec_from_file_location("epok_mcp_client", engine / "tests/integration/mcp.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    folder = args.session.resolve()
    metadata = folder / "session.json"
    if args.command == "start":
        if metadata.exists():
            raise SystemExit("Session already exists; inspect its PID/server instead of launching another editor.")
        editor = (args.editor or engine / ("target/debug/epok-editor.exe" if os.name == "nt" else "target/debug/epok-editor")).resolve()
        project = args.project.resolve(strict=True)
        ordinary = Path(os.environ.get("LOCALAPPDATA", str(Path.home() / ".local/share"))) / "Epok/Editor.epokprefs"
        prefs = module.documents.loads(ordinary.read_text(encoding="utf-8")) if ordinary.exists() else {}
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        prefs["mcp"] = {"enabled": True, "port": port, "token": uuid.uuid4().hex + uuid.uuid4().hex}
        preferences = folder / "local/Epok/Editor.epokprefs"
        preferences.parent.mkdir(parents=True, exist_ok=True)
        module.documents.write_text(preferences, json.dumps(prefs))
        env = dict(os.environ, LOCALAPPDATA=str(folder / "local"), XDG_DATA_HOME=str(folder / "local"))
        startup = None
        if os.name == "nt":
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow = 0
        with (folder / "editor.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(editor), "--project", str(project), "--window-size", "1280x800"],
                                       cwd=engine, env=env, stdout=log, stderr=log, startupinfo=startup)
        record = {"pid": process.pid, "editor": str(editor), "project": str(project), "preferences": str(preferences), "port": port}
        metadata.write_text(json.dumps(record, indent=2), encoding="utf-8")
        print(json.dumps({"pid": process.pid, "session": str(metadata), "port": port}))
        return
    record = json.loads(metadata.read_text(encoding="utf-8"))
    prefs = module.documents.loads(Path(record["preferences"]).read_text(encoding="utf-8"))
    client = module.Client(record["port"], prefs["mcp"]["token"])
    client.session = None if args.reconnect else record.get("mcp_session")
    client.sequence = record.get("sequence", 0)
    if not client.session:
        # A newly owned editor may still be opening the project before it binds
        # MCP. Only the read-only wait command retries connection refusal; never
        # replay a mutation or restart the editor after an uncertain response.
        connect_deadline = wait_deadline or 0
        while True:
            try:
                reply = client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                                      "clientInfo": {"name": "epok-study-qa", "version": "1"}})
                break
            except ConnectionRefusedError:
                if time.monotonic() >= connect_deadline:
                    raise
                time.sleep(0.2)
        assert reply["serverInfo"]["name"] == "epok-editor"
        client.request("notifications/initialized", notification=True)
    try:
        if args.command == "call":
            print(json.dumps(client.tool(args.tool, **json.loads(args.arguments)), ensure_ascii=False, indent=2))
        elif args.command == "wait":
            deadline, phase = wait_deadline, object()
            while True:
                state = client.tool("editor_state")
                if state["last_error"] or state["game_error"]:
                    raise RuntimeError(state["last_error"] or state["game_error"])
                ready = {
                    "idle": not state["editing_locked"] and not state["build_or_play_active"],
                    "stopped": not state["build_or_play_active"] and not state["scene_loading"],
                    "running": state["playing"] and frame_sequence(state) > args.frames,
                }[args.state]
                if ready:
                    print(json.dumps(state, ensure_ascii=False, indent=2), flush=True)
                    break
                if state["operation"] != phase:
                    phase = state["operation"]
                    print(json.dumps({"waiting_for": args.state, "operation": phase}), flush=True)
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"Editor did not reach {args.state}")
                time.sleep(1)
        else:
            size, _ = client.screenshot(args.target, args.output.resolve())
            print(json.dumps({"path": str(args.output.resolve()), "size": size}))
    finally:
        record.update(mcp_session=client.session, sequence=client.sequence)
        metadata.write_text(json.dumps(record, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
