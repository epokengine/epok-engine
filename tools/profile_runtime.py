"""Sample completed native PSX frames; optional explicit virtual-pad replay."""
import epok_documents as documents
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import struct
import subprocess
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
# Only Windows builds carry the .exe suffix; macOS and Linux use the bare name.
EDITOR = ROOT / ("target/debug/epok-editor.exe" if os.name == "nt" else "target/debug/epok-editor")
FIELDS = "frame frame_scanlines simulation_scanlines world_scanlines collision_scanlines render_scanlines vertex_scanlines polygon_scanlines steps world_syncs local_matrices world_matrices collider_bounds fog_scanlines shade_scanlines emit_scanlines gte_vertices software_vertices gte_validation_errors gte_max_delta".split()
# Optional extra fields appended to PerformanceStats after the original layout.
EXTRA_FIELDS = "tested_chunks visible_chunks backfaces clipped triangles gte_screen_max_delta prepare_scanlines finish_scanlines setup_scanlines camera_scanlines sprite_scanlines hud_scanlines retained_triangles retained_rebuilds visibility_skipped_chunks streamed_chunks stream_failed_chunks skeletal_scanlines skeletal_bone_matrices skeletal_cpu_vertices skeletal_decoded_vertices stream_budget_skipped_chunks gte_batch_vertices gte_batch_fallback_vertices".split()
STREAMING_FIELDS = "reads bytes stalls stall_us errors timeouts xa_interruptions".split()
WARMUP_FIELDS = "attempts pages reads stall_us rejected failures".split()
ORDERED_CAMERA_FIELDS = ([f"camera_origin_{c}" for c in range(3)]
                         + [f"camera_basis_{r}_{c}" for r in range(3) for c in range(4)]
                         + [f"row_scale_{r}" for r in range(2)])
ORDERED_DRAW_FIELDS = ("frame candidates packets quads cpu_direct clipped_candidates page_switches origin_changes anchored_vertices anchor_rejections sample frame_microseconds".split()
                       + ORDERED_CAMERA_FIELDS)
PLAYBACK_STATS = {
    "ordered_validation": ("ordered_validation_stats", "vertices triples range_fallback_vertices flagged_vertices camera_errors batch_errors screen_errors max_camera_delta max_screen_delta max_world_camera_delta first_error_kind first_error_sample first_error_token first_error_vertex".split(),
                           {"max_camera_delta", "max_screen_delta", "max_world_camera_delta", "first_error_kind", "first_error_sample", "first_error_token", "first_error_vertex"}),
    "streaming_verify": ("streaming_verify_stats", "batches bytes scanlines last_scanlines max_scanlines".split(),
                         {"last_scanlines", "max_scanlines"}),
    "streaming_overlap": ("stream_verify_overlap_bytes", ["bytes"], set()),
    "ordered_draw": ("ordered_draw_stats", ORDERED_DRAW_FIELDS, set(ORDERED_DRAW_FIELDS)),
    "visibility_track": ("visibility_track_stats", "sample advances waiting_frames loops".split(), {"sample"}),
    "sequence": ("sequence_stats",
                 "active peak dropped conflicts skipped_targets skipped_events properties events markers completed cancelled clamped_ticks diagnostics_dropped".split(),
                 {"active", "peak"}),
    "effect": ("effect_stats",
               "active peak spawned dropped completed cancelled active_layers peak_layers dropped_bursts clamped_ticks".split(),
               {"active", "peak", "active_layers", "peak_layers"}),
    "particle": ("particle_stats",
                 "alive spawned dropped peak dropped_emitters".split(),
                 {"alive", "peak"}),
    # Object/Actor/Component model (epok::actor_stats, runtime/actor_tables.hpp).
    # Absent in a build whose banks carry no actors: the summary reports available=False.
    "actor": ("actor_stats",
              "alive peak rejected spawned deferred actors components scene_scripts banks_loaded".split(),
              {"alive", "peak", "actors", "components"}),
}


OPTIONAL_COUNTERS = ("streamed_chunks", "vertex_scanlines", "polygon_scanlines")


def normalize_streamed_chunks(row):
    # Keep this public entry point for existing samplers. Old builds may have
    # collected the ordered sub-timers even with --detail off: honor each field,
    # not an inferred mode. New sentinels must never become enormous timings.
    for field in OPTIONAL_COUNTERS:
        value = row.get(field)
        available = value is not None and value != 0xffffffff
        row[field] = value if available else None
        row[field + "_available"] = available


def latch_ordered_frame(row):
    """Reject mid-publication snapshots; legacy/non-ordered rows stay supported.

    Raw signed camera values are decoded as u32 bit patterns, sufficient for
    exact identity comparisons. Never replace them with rounded world floats.
    """
    ordered = row.get("ordered_draw")
    if ordered is None or "sample" not in ordered:
        return True
    if ordered["frame"] == 0 or ordered["frame"] != row["frame"]:
        return False
    # sample=UINT32_MAX denotes a completed frame without an ordered pass.
    if ordered["sample"] != 0xffffffff:
        row["frame_microseconds"] = ordered["frame_microseconds"]
    return True


def frame_medians(rows):
    if not rows:
        return {field: None for field in OPTIONAL_COUNTERS}
    medians = {key: statistics.median(row[key] for row in rows) for key in rows[0]
               if key != "frame" and all(isinstance(row.get(key), (int, float))
                                         and not isinstance(row.get(key), bool) for row in rows)}
    for field in OPTIONAL_COUNTERS:
        if not all(row.get(field + "_available", False) for row in rows):
            medians[field] = None
    return medians


def validate_gte_rows(rows):
    # CD startup may legitimately complete frames before any mesh is resident.
    # Require actual projection coverage, but do not fail those empty frames.
    assert any(row["gte_vertices"] > 0 for row in rows), "No GTE projection coverage"
    assert all(row["gte_validation_errors"] == 0 for row in rows), "GTE/software differential failed"
    # Ordered counters are cumulative: retain an error even when its frame
    # fell between polls. Older/non-ordered builds may not link these counters.
    for row in rows:
        ordered = row.get("ordered_validation")
        if ordered is not None:
            assert all(ordered[key] == 0 for key in
                       ("camera_errors", "batch_errors", "screen_errors", "first_error_kind")), "Ordered GTE differential failed"


def symbol_address(symbols, name):
    match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
    return (int(match[1], 16) & 0x1fffff) if match else None


def read_counters(ram, address, fields):
    if address is None:
        return None
    if address + len(fields) * 4 > len(ram):
        raise ValueError("Incomplete RAM snapshot for native counters")
    return dict(zip(fields, struct.unpack_from(f"<{len(fields)}I", ram, address)))


def rail_state_layout(symbols):
    address = symbol_address(symbols, "epok::rail_navigation_state")
    if address is None:
        return None, False
    section = r"\.(?:bss|sbss|data)\._ZN4epok21rail_navigation_stateE\s+0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)"
    sizes = [int(size, 16) for start, size in re.findall(section, symbols)
             if int(start, 16) & 0x1fffff == address]
    if not sizes or max(sizes) not in (88, 100):
        raise ValueError("Unsupported rail navigation state layout")
    return address, max(sizes) == 100


def read_rail_state(ram, address, position_available):
    if address is None:
        return None
    values = struct.unpack_from("<4H3I4B", ram, address)
    result = dict(zip("route next_route next_sample pending_link fraction transitions reversals pending crossing forward next_forward".split(), values))
    if position_available:
        result["target_position"] = [v / 4096 for v in struct.unpack_from("<3i", ram, address + 88)]
    return result


def playback_addresses(symbols):
    addresses = {}
    for key, (symbol, fields, _) in PLAYBACK_STATS.items():
        address = symbol_address(symbols, f"epok::{symbol}")
        if address is not None:
            section = rf"\.(?:bss|sbss|data|sdata)\._ZN4epok{len(symbol)}{symbol}E\s+0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)"
            sizes = [int(size, 16) for start, size in re.findall(section, symbols)
                     if int(start, 16) & 0x1fffff == address]
            if not sizes or max(sizes) < 4 * len(fields):
                raise ValueError(f"Unsupported linked layout for epok::{symbol}")
        addresses[key] = address
    return addresses


def playback_summary(rows, key, fields, gauges):
    observed = [row[key] for row in sorted(rows, key=lambda row: row["frame"]) if key in row]
    if not observed:
        return {"available": False}
    first, last = observed[0], observed[-1]
    # These runtime counters saturate, rather than wrap. Do not turn a reset
    # into a huge unsigned delta or treat an active-instance gauge as a total.
    reset = any(current[field] < previous[field]
                for previous, current in zip(observed, observed[1:])
                for field in fields if field not in gauges)
    return {"available": True, "first": first, "last": last,
            "maximum_observed": {field: max(row[field] for row in observed)
                                 for field in fields},
            "counter_reset_observed": reset,
            "during_capture": {field: None if reset else last[field] - first[field]
                               for field in fields if field not in gauges},
            "saturated": [field for field in fields if field not in gauges
                          and any(row[field] == 0xffffffff for row in observed)]}


def counter_summary(rows, key, fields):
    observed = [row[key] for row in sorted(rows, key=lambda row: row["frame"]) if key in row]
    if not observed:
        return {"available": False}
    first, last = observed[0], observed[-1]
    return {"available": True, "first": first, "last": last,
            "during_capture": {field: (last[field] - first[field]) & 0xffffffff for field in fields}}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", type=Path, default=ROOT / "examples/sample-game",
                        help="Game project to profile (default: the included sample project)")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--editor", type=Path, default=EDITOR,
                        help="Editor snapshot used to export/build this capture")
    parser.add_argument("--seconds", type=int, default=20)
    parser.add_argument("--min-frames", type=int, default=5,
                        help="Minimum completed frames required for a capture (default: 5)")
    parser.add_argument("--include-startup-frames", action="store_true",
                        help="Capture completed frames 1–5 as well as steady-state frames")
    parser.add_argument("--capture-after-frames", type=int, default=0,
                        help="Capture VRAM after at least this many completed frames")
    parser.add_argument("--capture-track-samples", default="",
                        help="Comma-separated baked visibility sample indices to capture as additional VRAM snapshots")
    parser.add_argument("--use-play-profile", action="store_true", help="Measure the saved Play content/data profile instead of the full scene bank")
    parser.add_argument("--input-replay", help="Explicit virtual-pad schedule: milliseconds:mask,... since emulator launch; final mask must be 0")
    parser.add_argument("--validate-gte", action="store_true")
    parser.add_argument("--optimization", choices=["Os","O2"])
    parser.add_argument("--detail", action="store_true", help="Build with per-quad shade/fog/emit timers")
    args = parser.parse_args()
    if args.output.exists() and any(args.output.iterdir()):
        parser.error("Output directory must be new or empty; existing measurements are never overwritten")
    args.output.mkdir(parents=True, exist_ok=True)
    editor = args.editor.resolve()
    env=os.environ.copy()
    if args.optimization:
        env["EPOK_RUNTIME_OPT"]=f"-{args.optimization}"
    if args.validate_gte:
        env["EPOK_VALIDATE_GTE"]="1"
    if args.detail:
        env["EPOK_PROFILE_DETAIL"]="1"
    if args.validate_gte or args.optimization or args.detail:
        (args.project / ".epok/build/main.o").unlink(missing_ok=True)
    profile_args = ["--use-play-profile"] if args.use_play_profile else []
    build = subprocess.run([str(editor), "--project", str(args.project), "--build-psx", *profile_args], capture_output=True, text=True,env=env)
    documents.write_text(args.output / "build.log", build.stdout + build.stderr)
    assert build.returncode == 0, build.stdout + build.stderr
    symbols = (args.project / ".epok/build/epok.map").read_text()
    address = int(re.search(r"0x([0-9a-fA-F]+)\s+epok::performance_stats\b", symbols)[1], 16) & 0x1fffff
    clock_address = int(re.search(r"0x([0-9a-fA-F]+)\s+epok::time\b", symbols)[1], 16) & 0x1fffff
    lighting_address = int(re.search(r"0x([0-9a-fA-F]+)\s+epok::lighting_stats\b", symbols)[1], 16) & 0x1fffff
    streaming_address = symbol_address(symbols, "epok::streaming_stats")
    warmup_address = symbol_address(symbols, "epok::streaming_warmup_stats")
    rail_address, rail_position_available = rail_state_layout(symbols)
    playback = playback_addresses(symbols)
    # PerformanceStats grows by appending fields; read the extra ones only when
    # the linked object is large enough for them.
    extra_count = 0
    match = re.search(r"\.bss\._ZN4epok17performance_statsE\s+0x[0-9a-fA-F]+\s+0x([0-9a-fA-F]+)", symbols)
    if match:
        extra_count = max(0, min(len(EXTRA_FIELDS), int(match[1], 16) // 4 - len(FIELDS)))
    config_path = next(path for path in (
        args.project / "Local.epokconfig", ROOT / "Local.epokconfig", ROOT / "Editor.epokconfig"
    ) if path.is_file())
    config = documents.loads(config_path.read_text())
    base = f"http://127.0.0.1:{config['web_port']}/api/v1/"

    def request(path):
        with urllib.request.urlopen(base + path, timeout=2) as response:
            return response.read()

    frames = {}
    captured = False
    track_targets = sorted({int(value) for value in args.capture_track_samples.split(",") if value.strip()})
    track_captures = []
    with (args.output / "runtime.log").open("w") as log:
        replay_args = ["--input-replay", args.input_replay] if args.input_replay else []
        process = subprocess.Popen([str(editor), "--project", str(args.project), "--play-psx", *profile_args, *replay_args, "--stop-after", str(args.seconds)], stdout=log, stderr=log, env=env,creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            while process.poll() is None:
                try:
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from(f"<{len(FIELDS)}I", ram, address)
                    if (0 if args.include_startup_frames else 5) < values[0] < 1000000:
                        row = dict(zip(FIELDS, values))
                        # Time: previous, accumulated, remainder, two bools/pad,
                        # then ticks, dropped_steps, frame_microseconds.
                        row["frame_microseconds"] = struct.unpack_from("<I", ram, clock_address + 24)[0]
                        row["dropped_steps"] = struct.unpack_from("<I", ram, clock_address + 20)[0]
                        if extra_count:
                            extra = struct.unpack_from(f"<{extra_count}I", ram, address + 4 * len(FIELDS))
                            row.update(zip(EXTRA_FIELDS[:extra_count], extra))
                        normalize_streamed_chunks(row)
                        lighting = struct.unpack_from("<6I", ram, lighting_address)
                        row["dropped_triangles"] = lighting[5]
                        streaming = read_counters(ram, streaming_address, STREAMING_FIELDS)
                        warmup = read_counters(ram, warmup_address, WARMUP_FIELDS)
                        if streaming is not None:
                            row["streaming"] = streaming
                        if warmup is not None:
                            row["streaming_warmup"] = warmup
                        if rail_address is not None:
                            row["rail_navigation"] = read_rail_state(ram, rail_address, rail_position_available)
                        for key, (_, fields, _) in PLAYBACK_STATS.items():
                            counters = read_counters(ram, playback[key], fields)
                            if counters is not None:
                                row[key] = counters
                        if not latch_ordered_frame(row):
                            time.sleep(0.01)
                            continue
                        # Scanline units approximate 63.6 us (NTSC); the remainder of the
                        # measured frame interval is presentation/DMA wait, not CPU work.
                        row["present_wait_estimate_us"] = max(0, row["frame_microseconds"] - row["frame_scanlines"] * 64)
                        frames[values[0]] = row
                        track_sample = row.get("visibility_track", {}).get("sample")
                        if track_targets and track_sample is not None and track_sample >= track_targets[0] and row["gte_vertices"] > 0:
                            target = track_targets.pop(0)
                            filename = f"vram-track-{target:04d}.bin"
                            (args.output / filename).write_bytes(request("gpu/vram/raw"))
                            track_captures.append({"requested_sample": target, "observed_sample": track_sample,
                                                   "frame": row["frame"], "file": filename})
                        if not captured and len(frames) >= args.capture_after_frames:
                            (args.output / "vram.bin").write_bytes(request("gpu/vram/raw"))
                            captured = True
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=15)
            if args.validate_gte or args.optimization or args.detail:
                (args.project / ".epok/build/main.o").unlink(missing_ok=True)
    assert len(frames) >= args.min_frames, f"Insufficient completed frames: {len(frames)}"
    rows = list(frames.values())
    # Cumulative CD/loading counters have totals and deltas, not frame medians.
    # Keep the existing scalar frame metrics unchanged.
    medians = frame_medians(rows)
    streaming_report = counter_summary(rows, "streaming", STREAMING_FIELDS)
    warmup_report = counter_summary(rows, "streaming_warmup", WARMUP_FIELDS)
    playback_report = {key: playback_summary(rows, key, fields, gauges)
                       for key, (_, fields, gauges) in PLAYBACK_STATS.items()}
    def git(*arguments):
        result = subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True, text=True)
        return result.stdout.strip() if result.returncode == 0 else None
    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
    distributions = {}
    for field in ("frame_scanlines", "frame_microseconds", "present_wait_estimate_us"):
        values = sorted(row[field] for row in rows)
        distributions[field] = {"min": values[0], "p95": values[(len(values)*95+99)//100-1], "max": values[-1]}
    report = {"samples": len(rows), "input_replay": args.input_replay, "gte_validation":args.validate_gte,"optimization_override":args.optimization,"detail_timers":args.detail,"timer_unit_us_approx": 64,
              "build": {"commit": git("rev-parse", "HEAD"), "worktree": git("status", "--porcelain"),
                        "editor": str(editor), "editor_sha256": digest(editor),
                        "project": str(args.project.resolve()), "config": config,
                        "executable_sha256": digest(args.project / ".epok/build/epok.ps-exe"),
                        "scene_header_sha256": digest(args.project / ".epok/build/scene.hh"),
                        "display_header_sha256": digest(args.project / ".epok/build/display.hh")},
              "distributions": distributions, "median": medians, "frames": rows,
              "streaming": streaming_report, "streaming_warmup": warmup_report,
              "playback": playback_report, "track_captures": track_captures,
              "playback_counter_notes": "Playback counters are current values from each sampled RAM snapshot, not separately latched completed-frame timings. Active/alive fields are gauges; other counters include startup work. during_capture excludes startup before the first sample and is unavailable across observed resets. Saturated counters cannot measure further dropped work. Maximum sampled values may miss peaks between samples; runtime peak counters retain their own high-water marks.",
              "streamed_chunks_available": all(row["streamed_chunks_available"] for row in rows),
              "streaming_counter_notes": "Cumulative totals include startup loading. Warmup batches include startup and explicit route load cuts, and are a subset of total reads/stalls. during_capture excludes work before the first sampled frame; steady frame timings are reported independently."}
    documents.write_text(args.output / "profile.json", json.dumps(report, indent=2))
    print(json.dumps({"samples": len(rows), "median": medians,
                      "streaming": streaming_report, "streaming_warmup": warmup_report,
                      "playback": playback_report}, indent=2))
    if args.validate_gte:
        validate_gte_rows(rows)


if __name__ == "__main__":
    main()
