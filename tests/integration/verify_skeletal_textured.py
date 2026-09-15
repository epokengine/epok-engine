"""Textured skeletal characters: real FBX/PNG -> portable assets -> MIPS -> PCSX-Redux.

Companion to verify_skeletal.py. That script proves rigid-GTE and baked playback
animate at all; this one proves the texture survives the whole pipeline (per-corner
UVs, packed page coordinates, no position duplication for UV seams), that the
skeletal PerformanceStats counters describe the work that was really done, and that
an off-screen character costs nothing.

Requires the configured local toolchain/emulator. Leaves an isolated review project
and every measurement under artifacts/skeletal-textured/<timestamp>/.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import copy
import hashlib
import json
import os
import re
import shutil
import struct
import subprocess
import time
import urllib.request
import uuid

import epok_documents as documents
from project_layout import project_manifest
from PIL import Image
from profile_runtime import EXTRA_FIELDS, FIELDS, symbol_address
from verify_rpg import png
from verify_skeletal import expected_poses, package

# Only Windows builds carry the .exe suffix; macOS and Linux use the bare name.
EXE = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
SKELETAL_FIELDS = ["skeletal_scanlines", "skeletal_bone_matrices",
                   "skeletal_cpu_vertices", "skeletal_decoded_vertices"]
ATLAS = ROOT / "resources/models/EpokSeamAtlas.png"
CHARACTER = ROOT / "resources/models/EpokSeamCharacter.fbx"
SEQUENCES = ROOT / "resources/models/EpokManySequences.fbx"
# The atlas quadrant centres and the near-black orientation row, in source pixels.
PROBES = {"red": (16, 12), "blue": (48, 12), "green": (16, 26), "yellow": (48, 26),
          "top_row": (32, 0)}


def rgb555(rgb):
    """PSX framebuffer word for an 8-bit colour, ignoring the mask bit."""
    r, g, b = (channel >> 3 for channel in rgb[:3])
    return r | (g << 5) | (b << 10)


def unpack555(value):
    return ((value & 31) << 3, ((value >> 5) & 31) << 3, ((value >> 10) & 31) << 3)


def screen(raw):
    """The 320x240 15bpp framebuffer out of a 2048-byte-per-row VRAM dump."""
    return b"".join(raw[y * 2048:y * 2048 + 640] for y in range(240))


def save_png(path, frame):
    pixels = struct.unpack("<76800H", frame)
    rgba = bytearray()
    for value in pixels:
        rgba += bytes(unpack555(value)) + b"\xff"
    path.write_bytes(png(320, 240, bytes(rgba)))


def histogram(frame):
    counts = {}
    for value in struct.unpack("<76800H", frame):
        key = value & 0x7fff
        counts[key] = counts.get(key, 0) + 1
    return counts


def near(counts, target, tolerance=1):
    """Pixels within +/-tolerance steps per RGB555 channel of a quantized colour."""
    want = unpack555(target)
    total = 0
    for value, count in counts.items():
        have = unpack555(value)
        if all(abs(a - b) <= tolerance * 8 for a, b in zip(want, have)):
            total += count
    return total


def write_package(path, meta, payload):
    """Rewrite an .epokasset in place. source_hash is the SHA-256 of the payload,
    exactly as assets::hash computes it, so Package::load still validates."""
    meta = dict(meta)
    meta["source_hash"] = hashlib.sha256(payload).hexdigest()
    header = json.dumps(meta).encode("utf-8")
    path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(header), len(payload))
                     + header + payload)


class Harness:
    def __init__(self, out):
        self.out = out
        self.port = self.web_port()
        self.results = {}

    def web_port(self):
        path = next(p for p in (ROOT / "Local.epokconfig", ROOT / "Editor.epokconfig")
                    if p.is_file())
        return documents.loads(path.read_text())["web_port"]

    def run(self, *args, log="build.log"):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True,
                                text=True, timeout=600, creationflags=FLAGS)
        with (self.out / log).open("a", encoding="utf-8") as handle:
            handle.write(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    def request(self, path, post=False):
        url = f"http://127.0.0.1:{self.port}/api/v1/{path}"
        request = urllib.request.Request(url, data=b"" if post else None)
        with urllib.request.urlopen(request, timeout=4) as response:
            return response.read()


def assets_of(folder):
    return [(path, *package(path)) for path in folder.glob("assets/**/*.epokasset")]


def assign_texture(folder, texture_id, slots=("Body", "Trim")):
    """No editor CLI assigns a texture to a material slot, so the Material packages
    are edited directly. The colour is neutralised as well: modulate_color maps 255
    to the PSX neutral 128, so a white material reproduces the atlas unmodulated."""
    assigned = {}
    for path, meta, payload in assets_of(folder):
        if meta["kind"] != "Material" or path.name.split(".")[0] not in slots:
            continue
        document = json.loads(payload)
        document["data"]["texture"] = texture_id
        document["data"]["color"] = [1.0, 1.0, 1.0]
        write_package(path, meta, json.dumps(document).encode("utf-8"))
        assigned[path.name.split(".")[0]] = meta["id"]
    return assigned


def create_project(harness, folder, fbx, storage, textured_slots=("Body", "Trim")):
    harness.run("--create-project", folder, "--name", "Skeletal Texture",
                "--template", "basic")
    manifest = project_manifest(folder)
    video = documents.loads(manifest.read_text())
    video["rendering"] = dict(width=320, height=240)
    documents.write_text(manifest, json.dumps(video))  # Fixed pixel-reference fixture.
    shutil.copyfile(ATLAS, folder / "assets" / ATLAS.name)
    shutil.copyfile(fbx, folder / "assets" / fbx.name)
    harness.run("--project", folder, "--import-texture", f"assets/{ATLAS.name}")
    harness.run("--project", folder, "--import-fbx", f"assets/{fbx.name}",
                "--animation-storage", storage)
    texture = next(m["id"] for _, m, _ in assets_of(folder) if m["kind"] == "Texture")
    assign_texture(folder, texture, textured_slots)
    catalogue = assets_of(folder)
    mesh = next(m["id"] for _, m, _ in catalogue if m["kind"] == "SkeletalMesh")
    clips = {json.loads(s)["data"]["name"]: (m["id"], json.loads(s)["data"]["frames"])
             for _, m, s in catalogue if m["kind"] == "AnimationClip"}
    return dict(texture=texture, mesh=mesh, clips=clips, folder=folder)


def compose_scene(project, placements, camera_position=(0, 0.75, -1.8),
                  camera_rotation=(0, 0, 0)):
    """Rewrite Main.epokmap with the template camera plus one Mesh3D actor per
    placement, copied from the sample game so the component classes stay authentic."""
    folder = project["folder"]
    scene_path = folder / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text())

    def component(actor, suffix):
        return next(c for c in actor["components"]
                    if c["class"]["name"].endswith(suffix))

    camera = next(a for a in scene["actors"]
                  if any(c["class"]["name"].endswith("Camera3DComponent")
                         for c in a["components"]))
    component(camera, "SceneComponent3D")["properties"].update(
        position=list(camera_position), rotation=list(camera_rotation), scale=[1, 1, 1])
    sample = documents.loads(
        (ROOT / "examples/sample-game/assets/scenes/SampleScene.epokmap").read_text())
    prototype = copy.deepcopy(next(
        a for a in sample["actors"]
        if any(c["class"]["name"].endswith("Mesh3DComponent") for c in a["components"])))
    actors = [camera]
    for name, clip, position in placements:
        actor = copy.deepcopy(prototype)
        actor["id"] = str(uuid.uuid4())
        actor["name"] = name
        actor["components"] = [c for c in actor["components"]
                               if c["class"]["name"].endswith(("SceneComponent3D",
                                                               "Mesh3DComponent"))]
        for c in actor["components"]:
            c["id"] = str(uuid.uuid4())
        component(actor, "SceneComponent3D")["properties"].update(
            position=list(position), rotation=[0, 0, 0], scale=[1, 1, 1])
        renderer = component(actor, "Mesh3DComponent")
        renderer["properties"]["skeletal_mesh"] = dict(
            asset=project["mesh"], clip=clip, looping=True, play_on_start=True)
        if "skeletal_mesh" not in renderer["overrides"]:
            renderer["overrides"].append("skeletal_mesh")
        actors.append(actor)
    scene["actors"] = actors
    documents.write_text(scene_path, json.dumps(scene, indent=2))


def header_facts(folder):
    text = (folder / ".epok/build/scene.hh").read_text()
    rows = re.findall(r"skin_vertices_\d+\[(\d+)\]\[3\]", text)
    # MeshGeometry also ends in "true,{0,0,0}"; a packed face carries four values.
    packed = re.findall(r"true,\{\d+,\d+,\d+,\d+\}", text)
    return dict(text=text,
                storage=sorted(set(re.findall(r"SkeletalStorage::(\w+)", text))),
                skin_vertex_rows=[int(v) for v in rows],
                skin_geometry_count=len(set(re.findall(r"MeshGeometry (skin_geometry_\d+)", text))),
                packed_uv_faces=len(packed),
                unpacked_uv_faces=len(re.findall(r"false,\{0,0,0,0\}", text)),
                texture_ids=sorted(set(re.findall(r"texture_id_[0-9a-f]+", text))))


def symbols(folder):
    return (folder / ".epok/build/epok.map").read_text()


def animator_readings(ram, symbol_text):
    """Animator is {bool enabled; const SkeletalMesh* model; int clip; uint32_t ticks;
    bool playing, looping;} (20 bytes, asserted in runtime/skeletal.hpp). The cooked
    model symbol is unique, so scanning the objects array for that pointer finds the
    live animators without depending on the Actor field offsets."""
    models = {}
    for address, name in re.findall(r"0x([0-9a-fA-F]+)\s+epok::scene_\d+::(skin_\d+)\b",
                                    symbol_text):
        models[name] = (int(address, 16) & 0x1fffff) | 0x80000000
    match = re.search(r"\.bss\._ZN4epok7objectsE\s*\n?\s*0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)",
                      symbol_text)
    assert match, "epok::objects section missing from the link map"
    start, size = int(match[1], 16) & 0x1fffff, int(match[2], 16)
    found = []
    for offset in range(start, start + size - 16, 4):
        pointer = struct.unpack_from("<I", ram, offset)[0]
        name = next((n for n, a in models.items() if a == pointer), None)
        if name is None:
            continue
        enabled, clip, ticks, playing, looping = (
            ram[offset - 4], *struct.unpack_from("<iI", ram, offset + 4),
            ram[offset + 12], ram[offset + 13])
        if enabled != 1 or playing > 1 or looping > 1 or clip < 0:
            continue
        found.append(dict(model=name, clip=clip, ticks=ticks,
                          playing=bool(playing), looping=bool(looping)))
    return found


def play(harness, folder, label, seconds=14, delays=(1.2, 0.55, 0.5), scratch=False):
    """One serialized emulator run: capture the framebuffer and the completed-frame
    PerformanceStats at each pause point. Only the process started here is touched."""
    symbol_text = symbols(folder)
    stats = symbol_address(symbol_text, "epok::performance_stats")
    scratch_address = symbol_address(symbol_text, "epok::skeletal_detail::scratch")
    match = re.search(r"\.bss\._ZN4epok17performance_statsE\s+0x[0-9a-fA-F]+\s+0x([0-9a-fA-F]+)",
                      symbol_text)
    extra = max(0, min(len(EXTRA_FIELDS), int(match[1], 16) // 4 - len(FIELDS))) if match else 0
    assert extra >= len(EXTRA_FIELDS), (
        "PerformanceStats is too small for the skeletal counters", extra)
    captures = []
    with (harness.out / f"emulator-{label}.log").open("w") as log:
        process = subprocess.Popen(
            [str(EXE), "--project", str(folder), "--play-psx", "--stop-after", str(seconds)],
            stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 60
            while True:
                assert process.poll() is None, f"See emulator-{label}.log"
                try:
                    if json.loads(harness.request("execution-flow"))["running"]:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, f"Emulator never started ({label})"
                time.sleep(0.1)
            for delay in delays:
                time.sleep(delay)
                harness.request("execution-flow?function=pause", True)
                ram = harness.request("cpu/ram/raw")
                values = struct.unpack_from(f"<{len(FIELDS) + extra}I", ram, stats)
                row = dict(zip(FIELDS + EXTRA_FIELDS[:extra], values))
                capture = dict(frame=row["frame"],
                               counters={k: row[k] for k in SKELETAL_FIELDS},
                               animators=animator_readings(ram, symbol_text))
                if scratch and scratch_address is not None:
                    capture["scratch"] = ram[scratch_address + 64 * 48:
                                             scratch_address + 64 * 48 + 512 * 6]
                capture["frame_buffer"] = screen(harness.request("gpu/vram/raw"))
                captures.append(capture)
                harness.request("execution-flow?function=resume", True)
            assert process.wait(timeout=40) == 0
        finally:
            if process.poll() is None:
                process.wait(timeout=60)
    return captures


def centroids(frame, expected, tolerance=1):
    """Screen-space centre of every atlas region, used to prove the UV orientation
    reached the GPU: the atlas top-left quadrant must land above and left of the
    bottom-right one, and the near-black top row above every quadrant."""
    pixels = struct.unpack("<76800H", frame)
    sums = {name: [0, 0, 0] for name in expected}
    wanted = {name: unpack555(value) for name, value in expected.items()}
    for index, value in enumerate(pixels):
        have = unpack555(value & 0x7fff)
        for name, want in wanted.items():
            if all(abs(a - b) <= tolerance * 8 for a, b in zip(want, have)):
                sums[name][0] += index % 320
                sums[name][1] += index // 320
                sums[name][2] += 1
    return {name: (round(x / n, 1), round(y / n, 1), n) if n else None
            for name, (x, y, n) in sums.items()}


def orientation_problems(centre):
    """The atlas is red top-left, blue top-right, green bottom-left, yellow
    bottom-right, with a near-black top row. Only relations that survive the
    animation are checked: the bottom-row quadrants sit on the limbs, whose
    screen-space centres swap as they swing, so their left/right order is not
    a property of the UV mapping."""
    if any(centre[name] is None for name in ("red", "blue", "green", "yellow")):
        return ["A quadrant colour is absent from the capture"]
    problems = []
    if not centre["red"][0] < centre["blue"][0]:
        problems.append("red is not left of blue")
    if not centre["red"][1] < centre["green"][1]:
        problems.append("red is not above green")
    if not centre["blue"][1] < centre["yellow"][1]:
        problems.append("blue is not above yellow")
    if centre["top_row"] and not centre["top_row"][1] < min(
            centre[name][1] for name in ("red", "blue", "green", "yellow")):
        problems.append("the near-black top row is not above the quadrants")
    return problems


def texture_report(harness, captures, label, minimum=40):
    """Identifiable-region check: every atlas quadrant and the near-black orientation
    row must reach the framebuffer in meaningful counts."""
    expected = {}
    with Image.open(ATLAS) as image:
        atlas = image.convert("RGB")
        for name, (x, y) in PROBES.items():
            expected[name] = rgb555(atlas.getpixel((x, y)))
    report = []
    centres = []
    for index, capture in enumerate(captures):
        counts = histogram(capture["frame_buffer"])
        save_png(harness.out / f"{label}-{index}.png", capture["frame_buffer"])
        report.append({name: near(counts, value) for name, value in expected.items()})
        centres.append(centroids(capture["frame_buffer"], expected))
    best = {name: max(row[name] for row in report) for name in expected}
    missing = [name for name, count in best.items() if count < minimum]
    return dict(expected_rgb555=expected, per_capture=report, best=best,
                missing=missing, centroids=centres,
                orientation=[orientation_problems(c) for c in centres])


def half_counts(frame, targets, tolerance=1):
    """Atlas-coloured pixels in the left and right halves of the framebuffer."""
    pixels = struct.unpack("<76800H", frame)
    wanted = [unpack555(t) for t in targets]
    halves = [0, 0]
    for index, value in enumerate(pixels):
        have = unpack555(value & 0x7fff)
        if any(all(abs(a - b) <= tolerance * 8 for a, b in zip(want, have))
               for want in wanted):
            halves[0 if index % 320 < 160 else 1] += 1
    return halves


def frame_index(ticks, frames, looping=True):
    """Mirrors epok::skeletal_detail::Scratch::frame_index (60 Hz ticks, 30 Hz samples)."""
    frame = ticks // 2
    length = frames - 1 if frames > 1 else 1
    if looping:
        frame %= length
    elif frame >= frames:
        frame = frames - 1
    return frame


def main():
    out = ROOT / "artifacts/skeletal-textured" / str(time.time_ns())
    out.mkdir(parents=True)
    harness = Harness(out)
    revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                              capture_output=True, text=True).stdout.strip()
    report = dict(engine_revision=revision, display=[320, 240], scenarios={},
                  fixtures={path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                            for path in (ATLAS, CHARACTER, SEQUENCES) if path.is_file()})

    folder = ROOT / ".epok" / ("skeletal-textured-" + str(time.time_ns()))
    documents.write_text(out / "project.txt", str(folder))
    project = create_project(harness, folder, CHARACTER, "rigid-gte")
    clips = project["clips"]
    report["clips"] = {name: dict(id=cid, frames=frames)
                       for name, (cid, frames) in clips.items()}
    assert {"Clip00", "Clip01", "Clip02"} <= set(clips), clips

    # 1-3. Textured rigid GTE, one visible instance.
    compose_scene(project, [("Character", clips["Clip00"][0], [0, 0, 0])])
    harness.run("--project", folder, "--build-psx")
    facts = header_facts(folder)
    assert facts["storage"] == ["RigidGte"], facts["storage"]
    assert facts["skin_vertex_rows"] == [32], facts["skin_vertex_rows"]
    assert facts["packed_uv_faces"] == 40, facts["packed_uv_faces"]
    assert facts["unpacked_uv_faces"] == 0, facts["unpacked_uv_faces"]
    rigid_bytes = (folder / ".epok/build/epok.ps-exe").stat().st_size
    captures = play(harness, folder, "rigid-visible")
    assert captures[0]["frame_buffer"] != captures[1]["frame_buffer"], \
        "Textured rigid GTE playback did not change the framebuffer"
    texture = texture_report(harness, captures, "rigid-visible")
    assert not texture["missing"], ("Atlas regions missing on screen", texture["best"])
    assert not any(texture["orientation"]), ("Atlas orientation wrong on screen",
                                            texture["orientation"], texture["centroids"])
    counters = [c["counters"] for c in captures]
    assert all(c["skeletal_bone_matrices"] > 0 for c in counters), counters
    assert all(c["skeletal_cpu_vertices"] == 0 for c in counters), counters
    assert all(c["skeletal_decoded_vertices"] == 0 for c in counters), counters
    report["scenarios"]["rigid_visible"] = dict(
        header=dict(storage=facts["storage"], skin_vertex_rows=facts["skin_vertex_rows"],
                    packed_uv_faces=facts["packed_uv_faces"],
                    skin_geometry_count=facts["skin_geometry_count"],
                    texture_ids=facts["texture_ids"]),
        ps_exe_bytes=rigid_bytes, frames=[c["frame"] for c in captures],
        frame_buffers_differ=True, counters=counters,
        animators=[c["animators"] for c in captures], texture_regions=texture)

    # 5a. Culled: the same character placed fully behind the camera.
    compose_scene(project, [("Character", clips["Clip00"][0], [0, 0, -9.0])])
    harness.run("--project", folder, "--build-psx")
    culled = play(harness, folder, "rigid-culled", delays=(1.2, 0.5, 0.5))
    culled_counters = [c["counters"] for c in culled]
    assert all(all(value == 0 for value in row.values()) for row in culled_counters), \
        ("Off-screen character still cost skeletal work", culled_counters)
    culled_frames = [c["frame"] for c in culled]
    assert culled_frames == sorted(culled_frames) and culled_frames[-1] > culled_frames[0], \
        ("Frame counter did not advance while culled", culled_frames)
    for index, capture in enumerate(culled):
        save_png(out / f"rigid-culled-{index}.png", capture["frame_buffer"])
    report["scenarios"]["rigid_culled"] = dict(counters=culled_counters,
                                               frames=culled_frames)

    # 7. Two instances of the same model, different clips, one shared geometry.
    compose_scene(project, [("Left", clips["Clip00"][0], [-1.15, 0, 0.6]),
                            ("Right", clips["Clip01"][0], [1.15, 0, 0.6])],
                  camera_position=(0, 0.75, -3.6))
    harness.run("--project", folder, "--build-psx")
    pair_facts = header_facts(folder)
    assert pair_facts["skin_geometry_count"] == 1, pair_facts["skin_geometry_count"]
    assert pair_facts["skin_vertex_rows"] == [32], pair_facts["skin_vertex_rows"]
    pair = play(harness, folder, "rigid-pair")
    pair_texture = texture_report(harness, pair, "rigid-pair")
    assert not pair_texture["missing"], ("Pair scenario lost atlas regions",
                                         pair_texture["best"])
    pair_clips = sorted({a["clip"] for capture in pair for a in capture["animators"]})
    quadrants = [pair_texture["expected_rgb555"][name]
                 for name in ("red", "blue", "green", "yellow")]
    halves = [half_counts(c["frame_buffer"], quadrants) for c in pair]
    best_halves = [max(h[0] for h in halves), max(h[1] for h in halves)]
    assert len(pair_clips) == 2, ("Both instances should run different clips", pair_clips)
    assert min(best_halves) > 200, ("Only one instance reached the framebuffer",
                                    best_halves)
    report["scenarios"]["rigid_two_instances"] = dict(
        skin_geometry_count=pair_facts["skin_geometry_count"],
        distinct_clips=pair_clips, textured_pixels_per_half=halves,
        counters=[c["counters"] for c in pair],
        animators=[c["animators"] for c in pair], texture_regions=pair_texture,
        ps_exe_bytes=(folder / ".epok/build/epok.ps-exe").stat().st_size)

    # 6. Reimport as baked vertices and replay, with a tick-exact pose comparison.
    compose_scene(project, [("Character", clips["Clip00"][0], [0, 0, 0])])
    harness.run("--project", folder, "--reimport-asset",
                f"assets/{CHARACTER.stem}.imported/Model.epokasset",
                "--animation-storage", "baked-vertices")
    assign_texture(folder, project["texture"])
    harness.run("--project", folder, "--build-psx")
    baked_facts = header_facts(folder)
    assert baked_facts["storage"] == ["BakedVertices"], baked_facts["storage"]
    assert baked_facts["packed_uv_faces"] == 40, baked_facts["packed_uv_faces"]
    baked = play(harness, folder, "baked-visible", scratch=True)
    baked_texture = texture_report(harness, baked, "baked-visible")
    assert not baked_texture["missing"], ("Baked scenario lost atlas regions",
                                          baked_texture["best"])
    assert not any(baked_texture["orientation"]), (
        "Atlas orientation wrong on screen", baked_texture["orientation"])
    baked_counters = [c["counters"] for c in baked]
    assert all(c["skeletal_cpu_vertices"] == 0 for c in baked_counters), baked_counters
    assert all(c["skeletal_bone_matrices"] == 0 for c in baked_counters), baked_counters
    assert all(c["skeletal_decoded_vertices"] > 0 for c in baked_counters), baked_counters
    catalogue = assets_of(folder)
    mesh_id = next(m["id"] for _, m, _ in catalogue if m["kind"] == "SkeletalMesh")
    clip_id, clip_frames = next((m["id"], json.loads(s)["data"]["frames"])
                                for _, m, s in catalogue
                                if m["kind"] == "AnimationClip"
                                and json.loads(s)["data"]["name"] == "Clip00")
    # expected_poses() reads every package as JSON; the texture package is binary.
    poses = expected_poses([entry for entry in catalogue
                            if entry[1]["kind"] != "Texture"], mesh_id, clip_id)
    vertex_count = len(poses[0]) // 3
    pose_checks = []
    for capture in baked:
        animator = next((a for a in capture["animators"] if a["clip"] >= 0), None)
        assert animator, "No live animator found in the objects array"
        index = frame_index(animator["ticks"], clip_frames, animator["looping"])
        decoded = struct.unpack_from(f"<{vertex_count * 3}h", capture["scratch"], 0)
        expected = poses[index]
        error = max(abs(a - b) for a, b in zip(decoded, expected))
        nearest = min(range(len(poses)),
                      key=lambda f: max(abs(a - b) for a, b in zip(decoded, poses[f])))
        pose_checks.append(dict(ticks=animator["ticks"], expected_frame=index,
                                nearest_frame=nearest, max_error_raw=error,
                                max_error_meters=error / 4096))
        assert error < 48, ("Decoded pose does not match the frame for the recorded "
                            "tick", pose_checks[-1])
    report["scenarios"]["baked_visible"] = dict(
        header=dict(storage=baked_facts["storage"],
                    packed_uv_faces=baked_facts["packed_uv_faces"],
                    skin_vertex_rows=baked_facts["skin_vertex_rows"]),
        counters=baked_counters, texture_regions=baked_texture,
        pose_checks=pose_checks, clip_frames=clip_frames,
        ps_exe_bytes=(folder / ".epok/build/epok.ps-exe").stat().st_size)

    # 8. Expanded clip capacity.
    max_clips = int(re.search(r"pub const MAX_CLIPS: usize = (\d+);",
                              (ROOT / "src/skeletal.rs").read_text())[1])
    report["max_clips"] = max_clips
    if max_clips >= 32 and SEQUENCES.is_file():
        many = ROOT / ".epok" / ("skeletal-many-" + str(time.time_ns()))
        wanted = ["Clip00", "Clip15", "Clip16", "Clip28", "Clip29"]
        many_project = create_project(harness, many, SEQUENCES, "rigid-gte")
        available = many_project["clips"]
        chosen = [name for name in wanted if name in available]
        spread = [(-2.4, 0.0), (-1.2, 0.0), (0.0, 0.0), (1.2, 0.0), (2.4, 0.0)]
        compose_scene(many_project,
                      [(name, available[name][0], [spread[i][0], 0, spread[i][1]])
                       for i, name in enumerate(chosen)],
                      camera_position=(0, 0.75, -5.4))
        harness.run("--project", many, "--build-psx")
        many_facts = header_facts(many)
        many_captures = play(harness, many, "many-sequences")
        many_texture = texture_report(harness, many_captures, "many-sequences", minimum=25)
        changed = many_captures[0]["frame_buffer"] != many_captures[1]["frame_buffer"]
        observed = sorted({a["clip"] for c in many_captures for a in c["animators"]})
        report["scenarios"]["many_sequences"] = dict(
            imported_clips=len(available), requested=wanted, placed=chosen,
            distinct_clip_indices=observed, frame_buffers_differ=changed,
            skin_geometry_count=many_facts["skin_geometry_count"],
            packed_uv_faces=many_facts["packed_uv_faces"],
            texture_regions=many_texture,
            ps_exe_bytes=(many / ".epok/build/epok.ps-exe").stat().st_size,
            animators=[c["animators"] for c in many_captures])
        assert len(chosen) == len(wanted), (wanted, sorted(available))
        assert changed, "Expanded-clip scene did not change the framebuffer"
        assert len(observed) == len(wanted), observed
        assert not many_texture["missing"], many_texture["best"]
    else:
        report["scenarios"]["many_sequences"] = dict(
            skipped=f"MAX_CLIPS is {max_clips} and/or {SEQUENCES.name} is absent")

    documents.write_text(ROOT / "artifacts/skeletal-textured-verification.json",
                         json.dumps(report, indent=2))
    documents.write_text(out / "skeletal-textured-verification.json",
                         json.dumps(report, indent=2))
    print("PASS textured skeletal cooking, playback, culling and counters:", out)


if __name__ == "__main__":
    main()
