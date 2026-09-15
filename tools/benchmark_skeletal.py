"""Repeated runtime and memory measurements for skeletal characters.

Builds a fixed set of skeletal workloads (storage mode, texturing, material
lighting, instance count, on-screen or culled), runs each one several times in
the configured emulator and records completed-frame `epok::performance_stats`
samples, the cooked `// skeletal budget:` accounting, the executable size, the
build memory report and the texture VRAM allocation.

Everything measured here is emulator timing for one fixed camera and scene.
It is not hardware timing and not host/editor FPS. See docs/performance.md for
counter semantics (overlapping scopes; the scanline unit is about 64 us).

Outputs go to artifacts/performance/skeletal/<timestamp>/ (results.json,
report.md, one log per build and run). Needs the local MIPS toolchain and the
emulator configured in Local.epokconfig, a debug editor build in target/debug
and a virtualenv with tools/requirements.txt.
"""
import argparse
import hashlib
import http.client
import json
import os
import re
import statistics
import struct
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tests/integration"))

import epok_documents as documents  # noqa: E402
import verify_skeletal_textured as textured  # noqa: E402
from profile_runtime import EXTRA_FIELDS, FIELDS, symbol_address  # noqa: E402

EXE = textured.EXE
FLAGS = textured.FLAGS
CHARACTER = textured.CHARACTER
SEQUENCES = textured.SEQUENCES
ATLAS = textured.ATLAS
MANNEQUIN = ROOT / "resources/models/EpokMannequin.fbx"

SKELETAL_FIELDS = ["skeletal_scanlines", "skeletal_bone_matrices",
                   "skeletal_cpu_vertices", "skeletal_decoded_vertices"]
# Frame metrics summarised per workload. Timing scopes overlap: render includes
# vertex and polygon time, and skeletal time is inside them. Never add them up.
REPORTED = ["frame_microseconds", "frame_scanlines", "render_scanlines",
            "vertex_scanlines", "polygon_scanlines", "setup_scanlines",
            "skeletal_scanlines", "skeletal_bone_matrices",
            "skeletal_cpu_vertices", "skeletal_decoded_vertices",
            "gte_vertices", "software_vertices", "triangles", "clipped",
            "backfaces", "dropped_triangles", "visible_chunks", "tested_chunks",
            "dropped_steps", "present_wait_estimate_us"]

# One-instance camera and placement, shared by every single-character workload.
SOLO_CAMERA = (0, 0.75, -1.8)
SOLO_PLACEMENT = [(0.0, 0.0, 0.0)]
# Four and eight instances: identical transforms for every storage mode.
QUAD_CAMERA = (0, 0.75, -3.6)
QUAD_PLACEMENT = [(-1.725, 0.0, 0.6), (-0.575, 0.0, 0.6),
                  (0.575, 0.0, 0.6), (1.725, 0.0, 0.6)]
OCTET_CAMERA = (0, 0.75, -5.4)
OCTET_PLACEMENT = QUAD_PLACEMENT + [(-1.725, 0.0, 2.1), (-0.575, 0.0, 2.1),
                                    (0.575, 0.0, 2.1), (1.725, 0.0, 2.1)]
# Fully behind the single-instance camera, so the envelope cull removes them.
CULLED_PLACEMENT = [(x, 0.0, -9.0) for x, _, _ in QUAD_PLACEMENT]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def set_material_lighting(folder, unlit, slots=("Body", "Trim")):
    """Rewrite the named Material packages' `unlit` flag in place. Lit materials
    make the cooker choose CpuRigid, which is the point of one workload."""
    changed = []
    for path, meta, payload in textured.assets_of(folder):
        if meta["kind"] != "Material" or path.name.split(".")[0] not in slots:
            continue
        document = json.loads(payload)
        document["data"]["unlit"] = unlit
        textured.write_package(path, meta, json.dumps(document).encode("utf-8"))
        changed.append(path.name.split(".")[0])
    return sorted(changed)


def material_slots(folder):
    return sorted(path.name.split(".")[0] for path, meta, _ in textured.assets_of(folder)
                  if meta["kind"] == "Material")


def budget_line(folder):
    """The `// skeletal budget:` first line of the generated skeletal header."""
    text = (folder / ".epok/build/scene.hh").read_text()
    match = re.search(r"// skeletal budget: (.+)", text)
    if not match:
        return None
    line = match[1]
    fields = {
        "host_track_bytes": r"host pose tracks (\d+) B",
        "rigid_pose_bytes": r"target bone poses (\d+) B",
        "rigid_descriptor_bytes": r"bone/track/clip descriptors (\d+) B",
        "baked_frame_bytes": r"baked frame payload (\d+) B",
        "baked_descriptor_bytes": r"baked frame descriptors (\d+) B",
        "geometry_bytes": r"geometry (\d+) B",
        "animator_bytes": r"animator (\d+) B per character",
        "scratch_bytes": r"shared scratch (\d+) B",
        "animation_total_bytes": r"animation total (\d+) B",
        "animation_limit_bytes": r"of (\d+) B",
    }
    parsed = {name: int(re.search(pattern, line)[1]) for name, pattern in fields.items()}
    parsed["line"] = line
    return parsed


def memory_facts(folder):
    """RAM/VRAM from the build's own memory report, when the build wrote one."""
    path = folder / ".epok/build/memory-report.json"
    if not path.is_file():
        return None
    report = json.loads(path.read_text())

    def node(root, name):
        return next((c for c in root.get("children", []) if c["name"] == name), None)

    facts = dict(ram_used=report["ram"]["used"], ram_capacity=report["ram"]["capacity"],
                 files_used=report["files"]["used"])
    for name in ("Textures", "Scene data / geometry", "Engine / SDK code",
                 "Pools / runtime state"):
        child = node(report["ram"]["root"], name)
        if child:
            facts[f"ram_{name.split(' /')[0].lower().replace(' ', '_')}"] = child["bytes"]
    scene = (report.get("scenes") or [None])[0]
    if scene:
        vram = scene["vram"]
        facts["vram_used"] = vram["used"]
        for group in ("Textures", "Palettes"):
            child = node(vram["root"], group)
            if child:
                facts[f"vram_{group.lower()}"] = child["bytes"]
                facts[f"vram_{group.lower()}_detail"] = {
                    entry["name"]: entry["bytes"] for entry in child.get("children", [])}
    return facts


def build_facts(folder):
    facts = textured.header_facts(folder)
    facts.pop("text")
    exe = folder / ".epok/build/epok.ps-exe"
    return dict(header=facts, budget=budget_line(folder),
                ps_exe_bytes=exe.stat().st_size if exe.is_file() else None,
                ps_exe_sha256=digest(exe), memory=memory_facts(folder))


def sample_run(harness, folder, label, seconds, warmup, index):
    """One emulator run. Polls `epok::performance_stats`, which holds the last
    completed native frame, and keeps one row per distinct frame number. Samples
    taken during the first `warmup` seconds of sampling are discarded."""
    symbol_text = textured.symbols(folder)
    stats = symbol_address(symbol_text, "epok::performance_stats")
    clock = symbol_address(symbol_text, "epok::time")
    lighting = symbol_address(symbol_text, "epok::lighting_stats")
    match = re.search(r"\.bss\._ZN4epok17performance_statsE\s+0x[0-9a-fA-F]+\s+0x([0-9a-fA-F]+)",
                      symbol_text)
    extra = max(0, min(len(EXTRA_FIELDS), int(match[1], 16) // 4 - len(FIELDS))) if match else 0
    assert extra >= len(EXTRA_FIELDS), ("PerformanceStats lacks the skeletal counters", extra)
    frames, first_sample, misses = {}, None, 0
    log_path = harness.out / f"run-{label}-{index}.log"
    with log_path.open("w") as log:
        process = subprocess.Popen(
            [str(EXE), "--project", str(folder), "--play-psx", "--stop-after", str(seconds)],
            stdout=log, stderr=log, creationflags=FLAGS)
        try:
            while process.poll() is None:
                try:
                    ram = harness.request("cpu/ram/raw")
                    values = struct.unpack_from(f"<{len(FIELDS) + extra}I", ram, stats)
                    if 5 < values[0] < 1000000:
                        now = time.monotonic()
                        if first_sample is None:
                            first_sample = now
                        row = dict(zip(FIELDS + EXTRA_FIELDS[:extra], values))
                        row["frame_microseconds"] = struct.unpack_from("<I", ram, clock + 24)[0]
                        row["dropped_steps"] = struct.unpack_from("<I", ram, clock + 20)[0]
                        row["dropped_triangles"] = struct.unpack_from("<6I", ram, lighting)[5]
                        row["present_wait_estimate_us"] = max(
                            0, row["frame_microseconds"] - row["frame_scanlines"] * 64)
                        row["warm"] = now - first_sample >= warmup
                        frames[values[0]] = row
                except (OSError, ValueError, http.client.HTTPException):
                    # A truncated debugger read is a sampling miss, not a result.
                    misses += 1
                time.sleep(0.05)
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=30)
    rows = [frames[key] for key in sorted(frames)]
    if misses:
        print(f"  {label} run {index}: {misses} dropped debugger reads", flush=True)
    warm = [row for row in rows if row["warm"]]
    assert len(warm) >= 10, f"Too few warm samples for {label} run {index}: {len(warm)}"
    return rows, warm


def summarise(rows, fields=REPORTED):
    """Median, 95th percentile, maximum and minimum per field, with the sample
    count. Distinct completed frames, not a continuous trace."""
    out = {"samples": len(rows)}
    for field in fields:
        values = sorted(row[field] for row in rows if field in row)
        if not values:
            continue
        out[field] = dict(median=statistics.median(values),
                          p95=values[(len(values) * 95 + 99) // 100 - 1],
                          max=values[-1], min=values[0])
    return out


def workload_definitions():
    """(key, title, project key, placement, camera, clip plan, expectations)."""
    def spread(names, count):
        return [names[i % len(names)] for i in range(count)]

    seam = ["Clip00", "Clip01", "Clip02"]
    return [
        dict(key="A", title="1 instance, untextured, rigid", project="seam_rigid_plain",
             camera=SOLO_CAMERA, placement=SOLO_PLACEMENT, clips=spread(seam, 1),
             storage="RigidGte", visible=True),
        dict(key="B", title="1 instance, textured, rigid", project="seam_rigid",
             camera=SOLO_CAMERA, placement=SOLO_PLACEMENT, clips=spread(seam, 1),
             storage="RigidGte", visible=True),
        dict(key="C", title="1 instance, textured, baked", project="seam_baked",
             camera=SOLO_CAMERA, placement=SOLO_PLACEMENT, clips=spread(seam, 1),
             storage="BakedVertices", visible=True),
        dict(key="D", title="1 instance, textured, lit materials (CPU rigid)",
             project="seam_lit", camera=SOLO_CAMERA, placement=SOLO_PLACEMENT,
             clips=spread(seam, 1), storage="CpuRigid", visible=True),
        dict(key="E", title="4 instances, textured, rigid", project="seam_rigid",
             camera=QUAD_CAMERA, placement=QUAD_PLACEMENT, clips=spread(seam, 4),
             storage="RigidGte", visible=True),
        dict(key="F", title="4 instances, textured, baked", project="seam_baked",
             camera=QUAD_CAMERA, placement=QUAD_PLACEMENT, clips=spread(seam, 4),
             storage="BakedVertices", visible=True),
        dict(key="G", title="8 instances, textured, rigid", project="seam_rigid",
             camera=OCTET_CAMERA, placement=OCTET_PLACEMENT, clips=spread(seam, 8),
             storage="RigidGte", visible=True),
        dict(key="H", title="8 instances, textured, baked", project="seam_baked",
             camera=OCTET_CAMERA, placement=OCTET_PLACEMENT, clips=spread(seam, 8),
             storage="BakedVertices", visible=True),
        dict(key="I", title="4 instances off-screen, rigid", project="seam_rigid",
             camera=SOLO_CAMERA, placement=CULLED_PLACEMENT, clips=spread(seam, 4),
             storage="RigidGte", visible=False),
        dict(key="J", title="4 instances off-screen, baked", project="seam_baked",
             camera=SOLO_CAMERA, placement=CULLED_PLACEMENT, clips=spread(seam, 4),
             storage="BakedVertices", visible=False),
        dict(key="K", title="30-clip model, 1 instance, rigid", project="many_rigid",
             camera=SOLO_CAMERA, placement=SOLO_PLACEMENT, clips=["Clip00"],
             storage="RigidGte", visible=True),
        dict(key="L", title="30-clip model, 1 instance, baked", project="many_baked",
             camera=SOLO_CAMERA, placement=SOLO_PLACEMENT, clips=["Clip00"],
             storage="BakedVertices", visible=True),
        dict(key="M", title="Original mannequin, 1 instance, rigid",
             project="mannequin_rigid", camera=SOLO_CAMERA, placement=SOLO_PLACEMENT,
             clips=[None], storage="RigidGte", visible=True),
        dict(key="N", title="Original mannequin, 1 instance, baked",
             project="mannequin_baked", camera=SOLO_CAMERA, placement=SOLO_PLACEMENT,
             clips=[None], storage="BakedVertices", visible=True),
    ]


PROJECTS = {
    "seam_rigid_plain": dict(model=CHARACTER, storage="rigid-gte", textured=False, lit=False),
    "seam_rigid": dict(model=CHARACTER, storage="rigid-gte", textured=True, lit=False),
    "seam_baked": dict(model=CHARACTER, storage="baked-vertices", textured=True, lit=False),
    "seam_lit": dict(model=CHARACTER, storage="rigid-gte", textured=True, lit=True),
    "many_rigid": dict(model=SEQUENCES, storage="rigid-gte", textured=True, lit=False),
    "many_baked": dict(model=SEQUENCES, storage="baked-vertices", textured=True, lit=False),
    "mannequin_rigid": dict(model=MANNEQUIN, storage="rigid-gte", textured=False, lit=False),
    "mannequin_baked": dict(model=MANNEQUIN, storage="baked-vertices", textured=False, lit=False),
}


def prepare_project(harness, key, stamp):
    spec = PROJECTS[key]
    folder = ROOT / ".epok" / f"benchmark-{key}-{stamp}"
    slots = ("Body", "Trim") if spec["textured"] else ()
    project = textured.create_project(harness, folder, spec["model"], spec["storage"],
                                      textured_slots=slots)
    project["slots"] = material_slots(folder)
    project["lit_slots"] = set_material_lighting(folder, False) if spec["lit"] else []
    project["spec"] = {k: (str(v) if isinstance(v, Path) else v) for k, v in spec.items()}
    return project


NTSC_FRAME_SCANLINES = 263  # One NTSC vblank budget, ~16.7 ms. See docs/performance.md.


def cell(value):
    """Markdown cell: whole medians without a decimal tail, pipes escaped."""
    if isinstance(value, float) and value.is_integer():
        value = int(value)
    return str(value).replace("|", r"\|")


def table(headers, rows):
    lines = ["| " + " | ".join(headers) + " |",
             "| " + " | ".join("---" for _ in headers) + " |"]
    lines += ["| " + " | ".join(cell(value) for value in row) + " |" for row in rows]
    return "\n".join(lines)


def median(entry, field):
    value = entry.get("combined", {}).get(field)
    return value["median"] if value else ""


def write_report(report, out):
    """The mechanical part of report.md: environment, method, per-workload tables
    and the measured resource headroom. Interpretation is added by the analyst."""
    measured = {k: w for k, w in report["workloads"].items() if w["status"] == "measured"}
    lines = ["# Skeletal character measurements", "",
             f"Revision `{report['engine_revision']}`"
             + (" with an uncommitted working tree" if report["worktree_dirty"] else "")
             + f", {report['display'][0]}x{report['display'][1]}, "
             + f"{report['runs_per_workload']} runs of {report['seconds']} s per workload, "
             + f"first {report['warmup_seconds']} s of samples discarded.", ""]
    lines += ["## What these numbers are", ""]
    lines += [f"* {note}" for note in report["notes"]]
    lines += ["* Scanline counters have a resolution of about "
              f"{report['timer_unit_us_approx']} us per unit, so single-unit differences "
              "are at the edge of the instrument.",
              f"* One NTSC vblank is about {NTSC_FRAME_SCANLINES} scanline units.", ""]
    lines += ["## Environment", "",
              table(["Item", "Value"],
                    [["Host", " ".join(str(v) for v in report["platform"].values())],
                     ["Emulator", report["config"].get("emulator")],
                     ["Editor", f"`{report['editor']}` sha256 "
                                f"`{(report['editor_sha256'] or '')[:16]}`"]]
                    + [[name, f"{fact['bytes']} B, sha256 `{fact['sha256'][:16]}`"]
                       for name, fact in report["fixtures"].items()]), ""]
    lines += ["## Workloads", "",
              table(["Key", "Workload", "Storage", "Instances", "Clips", "Camera"],
                    [[k, w["title"], w["storage"], len(w["placement"]),
                      ", ".join(w.get("clip_names", [])), tuple(w["camera"])]
                     for k, w in report["workloads"].items()]), ""]
    lines += ["## Frame timing (scanline units, median of all warm samples)", "",
              table(["Key", "n", "frame", "p95", "max", "render", "vertex", "polygon",
                     "skeletal", "frame us", "dropped steps"],
                    [[k, w["combined"]["samples"],
                      median(w, "frame_scanlines"),
                      w["combined"]["frame_scanlines"]["p95"],
                      w["combined"]["frame_scanlines"]["max"],
                      median(w, "render_scanlines"), median(w, "vertex_scanlines"),
                      median(w, "polygon_scanlines"), median(w, "skeletal_scanlines"),
                      median(w, "frame_microseconds"), median(w, "dropped_steps")]
                     for k, w in measured.items()]), "",
              "Per-run medians of `frame_scanlines`, to show run-to-run spread:", "",
              table(["Key"] + [f"run {i}" for i in range(report["runs_per_workload"])],
                    [[k] + [run["warm"]["frame_scanlines"]["median"] for run in w["runs"]]
                     for k, w in measured.items()]), ""]
    lines += ["## Work counters (median)", "",
              table(["Key", "gte_vertices", "software_vertices", "triangles", "clipped",
                     "backfaces", "dropped triangles", "bone matrices", "cpu vertices",
                     "decoded vertices", "visible chunks"],
                    [[k, median(w, "gte_vertices"), median(w, "software_vertices"),
                      median(w, "triangles"), median(w, "clipped"),
                      median(w, "backfaces"), median(w, "dropped_triangles"),
                      median(w, "skeletal_bone_matrices"),
                      median(w, "skeletal_cpu_vertices"),
                      median(w, "skeletal_decoded_vertices"),
                      median(w, "visible_chunks")]
                     for k, w in measured.items()]), ""]
    lines += ["## Memory and size (bytes)", "",
              table(["Key", "geometry", "rigid pose", "rigid desc.", "baked frames",
                     "baked desc.", "host tracks", "animation total", "animator/char",
                     "shared scratch", ".ps-exe", "static RAM", "VRAM texture",
                     "VRAM palette"],
                    [[k, w["build"]["budget"]["geometry_bytes"],
                      w["build"]["budget"]["rigid_pose_bytes"],
                      w["build"]["budget"]["rigid_descriptor_bytes"],
                      w["build"]["budget"]["baked_frame_bytes"],
                      w["build"]["budget"]["baked_descriptor_bytes"],
                      w["build"]["budget"]["host_track_bytes"],
                      w["build"]["budget"]["animation_total_bytes"],
                      w["build"]["budget"]["animator_bytes"],
                      w["build"]["budget"]["scratch_bytes"],
                      w["build"]["ps_exe_bytes"],
                      (w["build"]["memory"] or {}).get("ram_used"),
                      (w["build"]["memory"] or {}).get("vram_textures", 0),
                      (w["build"]["memory"] or {}).get("vram_palettes", 0)]
                     for k, w in measured.items()]), "",
              "The animation budget limit reported by the cooker is "
              f"{next(iter(measured.values()))['build']['budget']['animation_limit_bytes']}"
              " B per model.", ""]
    headroom = []
    for k, w in measured.items():
        memory = w["build"]["memory"] or {}
        combined = w["combined"]
        headroom.append([
            k,
            f"{cell(combined['frame_scanlines']['median'])} / {NTSC_FRAME_SCANLINES}"
            + (" (over)" if combined["frame_scanlines"]["median"] > NTSC_FRAME_SCANLINES else ""),
            f"{memory.get('ram_used')} / {memory.get('ram_capacity')}",
            f"{memory.get('vram_used')} / 1048576",
            w["build"]["budget"]["animation_total_bytes"],
            combined["clipped"]["median"], combined["dropped_triangles"]["median"]])
    lines += ["## Headroom against each resource", "",
              table(["Key", "frame scanlines", "static RAM", "VRAM", "animation bytes",
                     "clipped", "dropped triangles"], headroom), ""]
    if report.get("failures"):
        lines += ["## Failed workloads", ""]
        lines += [f"* `{key}`: {error}" for key, error in report["failures"]] + [""]
    documents.write_text(out / "report.md", "\n".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path,
                        help="New or empty directory (default: a timestamped one)")
    parser.add_argument("--report-from", type=Path,
                        help="Rewrite report.md from an existing results.json, measuring nothing")
    parser.add_argument("--seconds", type=int, default=14, help="Emulator run length")
    parser.add_argument("--warmup", type=float, default=3.0,
                        help="Seconds of samples discarded at the start of each run")
    parser.add_argument("--runs", type=int, default=3, help="Repeats per workload")
    parser.add_argument("--only", help="Comma-separated workload keys to measure")
    args = parser.parse_args()

    if args.report_from:
        source = args.report_from
        source = source / "results.json" if source.is_dir() else source
        write_report(json.loads(source.read_text()), source.parent)
        print("Report:", source.parent / "report.md")
        return

    stamp = str(time.time_ns())
    out = args.output or ROOT / "artifacts/performance/skeletal" / stamp
    if out.exists() and any(out.iterdir()):
        parser.error("Output directory must be new or empty; measurements are never overwritten")
    out.mkdir(parents=True, exist_ok=True)
    harness = textured.Harness(out)

    def git(*arguments):
        result = subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True, text=True)
        return result.stdout.strip() if result.returncode == 0 else None

    wanted = set(args.only.split(",")) if args.only else None
    workloads = [w for w in workload_definitions() if not wanted or w["key"] in wanted]
    report = dict(
        engine_revision=git("rev-parse", "HEAD"),
        worktree_dirty=bool(git("status", "--porcelain")),
        editor=str(EXE), editor_sha256=digest(EXE),
        platform=dict(system=os.uname().sysname, release=os.uname().release,
                      machine=os.uname().machine),
        config=documents.loads((ROOT / "Local.epokconfig").read_text()),
        display=[320, 240], timer_unit_us_approx=64,
        seconds=args.seconds, warmup_seconds=args.warmup, runs_per_workload=args.runs,
        fixtures={path.name: dict(sha256=digest(path), bytes=path.stat().st_size)
                  for path in (ATLAS, CHARACTER, SEQUENCES, MANNEQUIN) if path.is_file()},
        notes=["Emulator timing for one fixed camera and scene; not hardware timing "
               "and not host/editor FPS.",
               "Timing scopes overlap (render includes vertex and polygon; skeletal "
               "work happens inside them). Do not add them together.",
               "One sample per distinct completed frame, polled at 20 Hz; not a "
               "continuous per-frame trace."],
        projects={}, workloads={})
    documents.write_text(out / "results.json", json.dumps(report, indent=2))

    projects, failures = {}, []
    for workload in workloads:
        key, project_key = workload["key"], workload["project"]
        print(f"[{key}] {workload['title']}", flush=True)
        entry = dict(workload)
        entry["placement"] = [list(p) for p in workload["placement"]]
        entry["camera"] = list(workload["camera"])
        try:
            if project_key not in projects:
                projects[project_key] = prepare_project(harness, project_key, stamp)
                report["projects"][project_key] = dict(
                    folder=str(projects[project_key]["folder"]),
                    spec=projects[project_key]["spec"],
                    slots=projects[project_key]["slots"],
                    lit_slots=projects[project_key]["lit_slots"],
                    clips={name: dict(frames=frames)
                           for name, (_, frames) in projects[project_key]["clips"].items()})
            project = projects[project_key]
            available = project["clips"]
            names = [name if name else sorted(available)[0] for name in workload["clips"]]
            missing = [name for name in names if name not in available]
            assert not missing, (missing, sorted(available))
            entry["clip_names"] = names
            placements = [(f"Character{i}", available[name][0], list(position))
                          for i, (name, position) in enumerate(zip(names, workload["placement"]))]
            textured.compose_scene(project, placements, camera_position=workload["camera"])
            harness.run("--project", project["folder"], "--build-psx",
                        log=f"build-{key}.log")
            entry["build"] = build_facts(project["folder"])
            storage = entry["build"]["header"]["storage"]
            assert storage == [workload["storage"]], (storage, workload["storage"])
            runs = []
            for index in range(args.runs):
                rows, warm = sample_run(harness, project["folder"], key,
                                        args.seconds, args.warmup, index)
                runs.append(dict(total_samples=len(rows), warm=summarise(warm),
                                 frame_first=rows[0]["frame"], frame_last=rows[-1]["frame"]))
                if workload["visible"]:
                    assert any(row["skeletal_scanlines"] > 0 for row in warm), \
                        f"Visible workload {key} recorded no skeletal work"
                else:
                    offenders = [row for row in warm
                                 if any(row[field] for field in SKELETAL_FIELDS)]
                    assert not offenders, ("Off-screen characters still cost skeletal work",
                                           key, offenders[:2])
                    assert warm[-1]["frame"] > warm[0]["frame"], \
                        f"Frame counter did not advance while culled ({key})"
                entry.setdefault("warm_rows", []).extend(warm)
            entry["runs"] = runs
            entry["combined"] = summarise(entry.pop("warm_rows"))
            entry["run_medians"] = [run["warm"]["frame_microseconds"]["median"]
                                    for run in runs]
            entry["status"] = "measured"
        except Exception as error:  # Record the limiting resource, never shrink silently.
            entry["status"] = "failed"
            entry["error"] = f"{type(error).__name__}: {error}"[:4000]
            failures.append((key, entry["error"]))
            print(f"  FAILED {entry['error'][:300]}", flush=True)
        report["workloads"][key] = entry
        documents.write_text(out / "results.json", json.dumps(report, indent=2))

    report["failures"] = failures
    documents.write_text(out / "results.json", json.dumps(report, indent=2))
    write_report(report, out)
    print("Measurements:", out)
    if failures:
        print("Failed workloads:", ", ".join(key for key, _ in failures))


if __name__ == "__main__":
    main()
