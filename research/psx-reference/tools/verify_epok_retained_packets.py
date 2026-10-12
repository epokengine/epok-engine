"""Compare retained/dynamic PSX images via one existing Epok MCP editor.

Uses an unsaved scene: a visible cached cube, enough invisible cubes to fill
the cache, then five visible dynamic cubes. Restores settings and the original
scene, refusing to discard unexpected authored changes. Never launches an editor.
"""
import argparse
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import sys
import time
import urllib.request
from PIL import Image, ImageChops


def authored(scene):
    return {k: v for k, v in scene.items() if k != "bake"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, default=Path("D:/GitProjects/GameEngines/Epok/EpokEngine"))
    parser.add_argument("--session", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
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
                                  "clientInfo": {"name": "retained-packets-qa", "version": "1"}})
    client.request("notifications/initialized", notification=True)

    def wait(predicate, timeout=300):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            state = client.tool("editor_state")
            if predicate(state):
                return state
            if state["last_error"] or (state["game_error"] and not state["build_or_play_active"]):
                raise AssertionError(state["last_error"] or state["game_error"])
            time.sleep(1)
        raise AssertionError("MCP operation timed out")

    state = wait(lambda s: not s["scene_loading"])
    assert not state["dirty"] and not state["build_or_play_active"]
    original = client.tool("scene_read")
    assert original["scene"].get("visibility_track") is None
    original_path = state["scene_path"]
    assert original_path == "assets/scenes/Main.epokmap", "Fixture preserves Main's first streamed mesh"
    map_file = Path(record["project"]) / original_path
    source_hash = hashlib.sha256(map_file.read_bytes()).hexdigest()
    manifests = list(Path(record["project"]).glob("*.epokproject"))
    assert len(manifests) == 1
    manifest_hash = hashlib.sha256(manifests[0].read_bytes()).hexdigest()
    settings = client.tool("project_settings")
    original_settings = settings["settings"]
    assert original_settings["rendering"]["streaming_geometry"]
    assert original_settings["rendering"]["streaming_triangle_budget"] == 2048
    expected_settings = original_settings
    expected_scene = None
    args.output.mkdir(parents=True)
    captures = []
    report = {"editor_pid": record["pid"], "source": original_path, "source_sha256": source_hash, "cases": []}

    def cube(name, position, color, scale):
        return {"op": "create", "actor": {"name": name, "position": position,
                "rotation": [17, -12, 9], "scale": [scale] * 3,
                "material": {"color": color, "unlit": True}}}

    try:
        # Keep one source streamed mesh off camera; without any archived payload
        # the exporter intentionally uses a world-sized non-streaming budget.
        operations = [{"op": "delete", "index": i} for i in range(state["actors"] - 1, 1, -1)]
        operations.append({"op": "update", "index": 1, "patch": {
            "name": "Invisible streamed resource", "position": [0, 0, -10000]}})
        operations.append({"op": "update", "index": 0, "patch": {
            "name": "Packet QA Camera", "kind": "Camera", "position": [0, 0, -5],
            "rotation": [0, 0, 0], "scale": [1, 1, 1]}})
        operations.append(cube("Cached visible cube", [-3, 0, 8], [1, 0.1, 0.03], 2))
        # Two 499-face source copies + the visible six-face cube + three
        # invisible six-face cubes occupy 1,022 cache records, within 64 actors.
        operations.append({"op": "duplicate", "index": 1})
        for index in range(3):
            operations.append(cube(f"Invisible cache entry {index}", [0, 0, -50], [0.1, 0.1, 1], 1))
        for index in range(5):
            operations.append(cube(f"Dynamic visible cube {index}", [index - 1, 0, 8 + index * 1.5], [0.03, 1, 0.1], 1.4))
        for offset in range(0, len(operations), 128):
            current = client.tool("scene_read")
            assert authored(current["scene"]) == (expected_scene or authored(original["scene"]))
            client.tool("scene_apply", revision=current["revision"], operations=operations[offset:offset + 128])
            expected_scene = authored(client.tool("scene_read")["scene"])
        for enabled in (True, False):
            name = "retained" if enabled else "dynamic"
            current = client.tool("project_settings")
            assert current["settings"] == expected_settings, "Unexpected project settings edit"
            changed = client.tool("project_settings", revision=current["revision"], patch={"rendering": {"retained_geometry": enabled}})
            expected_settings = changed["settings"]
            assert authored(client.tool("scene_read")["scene"]) == expected_scene, "Unexpected authored edit"
            print(f"Building {name} packet fixture via MCP (editor PID {record['pid']})", flush=True)
            client.tool("editor_control", action="play")
            wait(lambda s: s["playing"] and s["game_frame"] and s["game_frame"]["sequence"] > 180)
            client.tool("editor_control", action="pause")
            wait(lambda s: s["paused"])
            size, data = client.screenshot("game", args.output / f"{name}.png")
            image = Image.open(io.BytesIO(data)).convert("RGB")
            pixels = list(image.getdata())
            red = sum(r > 160 and g < 90 and b < 90 for r, g, b in pixels)
            green = sum(g > 160 and r < 90 and b < 90 for r, g, b in pixels)
            assert red > 100 and green > 100, (name, red, green)
            captures.append(image)
            build = Path(record["project"]) / ".epok/build"
            header = (build / "scene.hh").read_text()
            assert "render_capacity=2048;" in header
            assert "retained_quad_capacity=1024;" in header
            sys.path.insert(0, str(args.engine / "tools"))
            import profile_runtime as profile
            from sample_epok_runtime import executable_sections, verify_running_code
            symbols = (build / "epok.map").read_text()
            address = profile.symbol_address(symbols, "epok::performance_stats")
            config = next(p for p in (Path(record["project"]) / "Local.epokconfig", args.engine / "Local.epokconfig") if p.exists())
            port = module.documents.loads(config.read_text())["web_port"]
            with urllib.request.urlopen(f"http://127.0.0.1:{port}/api/v1/cpu/ram/raw", timeout=5) as response:
                ram = response.read()
            verify_running_code(ram, executable_sections((build / "epok.elf").read_bytes()))
            counters = profile.read_counters(ram, address, profile.FIELDS + profile.EXTRA_FIELDS)
            if enabled:
                assert counters["retained_triangles"] > 0, "Fixture must exercise actual retained emission"
                assert counters["triangles"] > counters["retained_triangles"] + 8, "Fixture must borrow dormant slots"
            evidence = args.output / name
            evidence.mkdir()
            for filename in ("epok.ps-exe", "epok.elf", "epok.map", "scene.hh", "display.hh", "memory-report.json"):
                shutil.copy2(build / filename, evidence / filename)
            report["cases"].append({"name": name, "size": size, "red_pixels": red, "green_pixels": green,
                "triangles": counters["triangles"], "retained_triangles": counters["retained_triangles"],
                "executable_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest()})
            client.tool("editor_control", action="stop")
            wait(lambda s: not s["build_or_play_active"] and not s["stopping"])
        difference = ImageChops.difference(*captures)
        report["different_pixels"] = sum(pixel != (0, 0, 0) for pixel in difference.getdata())
        assert report["different_pixels"] == 0, report
        print(json.dumps(report, indent=2), flush=True)
    finally:
        state = client.tool("editor_state")
        if state["build_or_play_active"]:
            client.tool("editor_control", action="stop")
            wait(lambda s: not s["build_or_play_active"])
        current_settings = client.tool("project_settings")
        assert current_settings["settings"] == expected_settings, "Unexpected settings edit; preserve it"
        restored = client.tool("project_settings", revision=current_settings["revision"],
                              patch={"rendering": {"retained_geometry": original_settings["rendering"]["retained_geometry"]}})
        assert restored["settings"] == original_settings
        current = client.tool("scene_read")
        if expected_scene is not None:
            assert authored(current["scene"]) == expected_scene, "Unexpected authored edit; preserve it"
            client.tool("scene_open", revision=current["revision"], path=original_path, discard_unsaved=True)
            wait(lambda s: not s["scene_loading"] and s["scene_path"] == original_path)
        assert hashlib.sha256(map_file.read_bytes()).hexdigest() == source_hash
        assert hashlib.sha256(manifests[0].read_bytes()).hexdigest() == manifest_hash
        report["source_and_settings_restored"] = True
        (args.output / "report.json").write_text(json.dumps(report, indent=2))
        record.update(mcp_session=client.session, sequence=client.sequence)
        metadata.write_text(json.dumps(record, indent=2))


if __name__ == "__main__":
    main()
