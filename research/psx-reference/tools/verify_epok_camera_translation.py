"""Two real Play PSX renders through one existing editor's MCP connection.

Uses an unsaved, temporary fixture and restores the original map. Refuses dirty
input or unexpected authored edits. Never launches an editor or saves a scene.
"""
import argparse
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import time
from PIL import Image, ImageChops, ImageOps


def authored(scene):
    return {k: v for k, v in scene.items() if k != "bake"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, default=Path("D:/GitProjects/GameEngines/Epok/EpokEngine"))
    parser.add_argument("--session", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mirror", action="store_true", help="Test horizontal projection reflection instead of world translation")
    args = parser.parse_args()
    assert not args.output.exists(), "Use a fresh output directory"
    spec = importlib.util.spec_from_file_location("epok_mcp", args.engine / "tests/integration/mcp.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    metadata = args.session / "session.json"
    record = json.loads(metadata.read_text())
    prefs = module.documents.loads(Path(record["preferences"]).read_text())
    client = module.Client(record["port"], prefs["mcp"]["token"])
    client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                  "clientInfo": {"name": "camera-translation-qa", "version": "1"}})
    client.request("notifications/initialized", notification=True)
    def wait(predicate, timeout=240):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            state = client.tool("editor_state")
            if predicate(state):
                return state
            if state["last_error"]:
                raise AssertionError(state["last_error"])
            if state["game_error"] and not state["build_or_play_active"]:
                raise AssertionError(state["game_error"])
            time.sleep(1)
        raise AssertionError("MCP operation timed out")
    state = wait(lambda s: not s["scene_loading"])
    assert not state["dirty"] and not state["build_or_play_active"]
    original_path = state["scene_path"]
    map_file = Path(record["project"]) / original_path
    original_hash = hashlib.sha256(map_file.read_bytes()).hexdigest()
    original = client.tool("scene_read")
    assert original["scene"].get("visibility_track") is None, "Fixture needs a map without rail playback"
    args.output.mkdir(parents=True)
    (args.output / "original-scene.json").write_text(json.dumps(original["scene"], indent=2))
    expected = None
    captures = []
    report = {"editor_pid": record["pid"], "source_scene": original_path, "source_sha256": original_hash, "cases": []}
    try:
        operations = [{"op": "delete", "index": i} for i in range(state["actors"] - 1, 0, -1)]
        operations += [
            {"op": "update", "index": 0, "patch": {"name": "Translation QA Camera", "kind": "Camera", "position": [0, 2, -5], "rotation": [8, 12, 2], "scale": [1, 1, 1]}},
            {"op": "create", "actor": {"name": "Translation QA Cube", "position": [0, 0, 8], "rotation": [17, -12, 9], "scale": [3, 3, 3], "material": {"color": [1, 0.125, 0.03], "unlit": True}}},
            {"op": "create", "actor": {"name": "Translation QA Parent", "kind": "Empty", "position": [0, 0, 0], "rotation": [7, 23, -11], "scale": [1.5, 0.75, 1.25]}},
            {"op": "reparent", "index": 0, "parent": 2, "keep_world": False},
            {"op": "reparent", "index": 1, "parent": 2, "keep_world": False},
        ]
        client.tool("scene_apply", revision=original["revision"], operations=operations)
        expected = authored(client.tool("scene_read")["scene"])
        cases = [("normal", [0, 0, 0]), ("mirrored", [0, 0, 0])] if args.mirror else [("near", [0, 0, 0]), ("far", [8000, -6000, 9000])]
        for name, offset in cases:
            current = client.tool("scene_read")
            assert authored(current["scene"]) == expected, "Unexpected authored edit; preserve it"
            if name == "far":
                client.tool("scene_apply", revision=current["revision"], operations=[{"op": "update", "index": 2, "patch": {"position": offset}}])
                expected = authored(client.tool("scene_read")["scene"])
            if args.mirror:
                changed = copy.deepcopy(current["scene"])
                props = next(c["properties"] for c in changed["actors"][0]["components"] if "camera_fov" in c.get("properties", {}))
                props["camera_mirror_x"] = name == "mirrored"
                client.tool("scene_apply", revision=current["revision"], operations=[{"op": "replace", "scene": changed}])
                expected = authored(client.tool("scene_read")["scene"])
            print(f"Building {name} hierarchical fixture via MCP (editor PID {record['pid']})", flush=True)
            client.tool("editor_control", action="play")
            state = wait(lambda s: s["playing"] and s["game_frame"] and s["game_frame"]["sequence"] > 180)
            client.tool("editor_control", action="pause")
            wait(lambda s: s["paused"])
            size, data = client.screenshot("game", args.output / f"{name}.png")
            image = Image.open(io.BytesIO(data)).convert("RGB")
            red = sum(1 for r, g, b in image.getdata() if r > 160 and g < 80 and b < 80)
            assert red > 200, f"Fixture cube missing: {red} red pixels"
            captures.append(image)
            report["cases"].append({"name": name, "parent_offset": offset, "size": size, "red_pixels": red,
                                    "executable_sha256": hashlib.sha256((Path(record['project']) / ".epok/build/epok.ps-exe").read_bytes()).hexdigest()})
            stopping = client.tool("editor_control", action="stop")["state"]
            assert stopping["stopping"], "Stop must expose its pending acknowledgement"
            stopped = wait(lambda s: not s["build_or_play_active"])
            assert not stopped["stopping"] and stopped["game_error"] is None and stopped["last_error"] is None, stopped
            report["cases"][-1]["stopped_without_error"] = True
        reference = ImageOps.mirror(captures[0]) if args.mirror else captures[0]
        difference = ImageChops.difference(reference, captures[1])
        report["different_pixels"] = sum(pixel != (0, 0, 0) for pixel in difference.getdata())
        if args.mirror:
            # Fixed-point projection rounds at pixel boundaries. Validate shape
            # and orientation with an explicit one-pixel tolerance, and report
            # exact differences instead of claiming bit-identical rasterization.
            def red_mask(image):
                return {(i % image.width, i // image.width) for i, (r, g, b) in enumerate(image.getdata()) if r > 160 and g < 80 and b < 80}
            expected_mask, actual_mask = red_mask(reference), red_mask(captures[1])
            def unmatched(a, b):
                return sum(not any((x + dx, y + dy) in b for dx in (-1, 0, 1) for dy in (-1, 0, 1)) for x, y in a)
            report["mirror_unmatched_pixels_beyond_one_pixel"] = unmatched(expected_mask, actual_mask) + unmatched(actual_mask, expected_mask)
            assert report["mirror_unmatched_pixels_beyond_one_pixel"] < 0.005 * len(expected_mask), report
            assert abs(len(expected_mask) - len(actual_mask)) < 0.02 * len(expected_mask), report
        else:
            assert report["different_pixels"] == 0, report
        print(json.dumps(report, indent=2), flush=True)
    finally:
        state = client.tool("editor_state")
        if state["build_or_play_active"]:
            client.tool("editor_control", action="stop")
            wait(lambda s: not s["build_or_play_active"])
        current = client.tool("scene_read")
        if expected is not None:
            assert authored(current["scene"]) == expected, "Unexpected authored edit; fixture left open, not discarded"
            client.tool("scene_open", revision=current["revision"], path=original_path, discard_unsaved=True)
            wait(lambda s: not s["scene_loading"] and s["scene_path"] == original_path)
        assert hashlib.sha256(map_file.read_bytes()).hexdigest() == original_hash, "Source map bytes changed"
        report["source_unchanged"] = True
        client.screenshot("scene", args.output / "restored-scene.png")
        client.screenshot("editor", args.output / "restored-editor.png")
        (args.output / "report.json").write_text(json.dumps(report, indent=2))
        record.update(mcp_session=client.session, sequence=client.sequence)
        metadata.write_text(json.dumps(record, indent=2))


if __name__ == "__main__":
    main()
