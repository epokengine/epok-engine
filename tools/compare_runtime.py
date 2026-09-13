"""Strict no-regression comparison of sampled native profile_runtime.py captures.

Exit 0: sampled checks pass; 1: regression; 2: invalid/incomparable captures.
This compares measured workloads, not every possible path or physical console.
"""
import epok_documents as documents
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys


TIMING_FIELDS = ("frame_microseconds", "frame_scanlines")
REQUIRED = ("frame", *TIMING_FIELDS, "dropped_steps", "dropped_triangles")


def summarize(report, minimum_samples=20):
    rows = report.get("frames")
    if not isinstance(rows, list) or len(rows) < minimum_samples:
        raise ValueError(f"Need at least {minimum_samples} sampled native frames")
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("Frame sample must be an object")
        for key in REQUIRED:
            value = row.get(key)
            if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
                raise ValueError(f"Invalid or missing native frame field: {key}")
        if row["frame_microseconds"] <= 0 or int(row["frame"]) != row["frame"]:
            raise ValueError("Frame interval must be positive and frame ID must be an integer")
    rows = sorted(rows, key=lambda row: row["frame"])
    if len({row["frame"] for row in rows}) != len(rows):
        raise ValueError("Duplicate native frame IDs; deduplicate before comparison")
    if any(b["dropped_steps"] < a["dropped_steps"] for a, b in zip(rows, rows[1:])):
        raise ValueError("Dropped-step counter reset/wrapped inside capture; compare separate runs")
    summary = {"samples": len(rows)}
    for key in TIMING_FIELDS:
        values = sorted(row[key] for row in rows)
        summary[f"{key}.median"] = statistics.median(values)
        summary[f"{key}.p95"] = values[math.ceil(len(values) * .95) - 1]
        summary[f"{key}.max"] = values[-1]
    # Absolute cumulative count includes initial stalls before the first sample;
    # delta additionally exposes stalls introduced during the observed interval.
    summary["dropped_steps.total"] = rows[-1]["dropped_steps"]
    summary["dropped_steps.observed_delta"] = rows[-1]["dropped_steps"] - rows[0]["dropped_steps"]
    summary["dropped_triangles.max"] = max(row["dropped_triangles"] for row in rows)
    summary["dropped_triangles.mean"] = statistics.mean(row["dropped_triangles"] for row in rows)
    for key in ("gte_validation_errors", "stream_failed_chunks"):
        if any(key in row for row in rows):
            values = [row.get(key) for row in rows]
            if any(isinstance(value, bool) or not isinstance(value, int) or value < 0 for value in values):
                raise ValueError(f"Incomplete/invalid {key} samples")
            summary[f"{key}.max"] = max(values)
    return summary


def cpu_by_steps(baseline, candidate):
    """Compare CPU work at equal fixed-step counts, independent of their mix."""
    available = []
    for report in (baseline, candidate):
        rows = report["frames"]
        present = ["steps" in row for row in rows]
        if any(present):
            if not all(present) or any(isinstance(row["steps"], bool) or not isinstance(row["steps"], int)
                                       or row["steps"] < 0 for row in rows):
                raise ValueError("Incomplete/invalid fixed simulation steps samples")
        available.append(all(present))
    result = {"available": all(available), "minimum_samples_per_group": 5, "groups": {},
              "scope": "CPU comparisons hold the fixed simulation step count equal. Groups require at least five samples in each capture; missing/small groups are reported but not failures because optimizations change their frequency. Global frame-time and safety checks still apply."}
    if not result["available"]:
        result["reason"] = "Both captures must include steps for every sampled frame."
        return result, {}, []
    groups = sorted({row["steps"] for report in (baseline, candidate) for row in report["frames"]})
    comparisons, regressions = {}, []
    for steps in groups:
        values = [sorted(row["frame_scanlines"] for row in report["frames"] if row["steps"] == steps)
                  for report in (baseline, candidate)]
        counts = {"baseline": len(values[0]), "candidate": len(values[1])}
        comparable = min(counts.values()) >= result["minimum_samples_per_group"]
        result["groups"][str(steps)] = {"samples": counts,
                                       "status": "compared" if comparable else "insufficient_samples"}
        if not comparable:
            continue
        summaries = [{"median": statistics.median(v), "p95": v[math.ceil(len(v) * .95) - 1], "max": v[-1]}
                     for v in values]
        for statistic in ("median", "p95", "max"):
            key = f"frame_scanlines.by_steps.{steps}.{statistic}"
            a, b = summaries[0][statistic], summaries[1][statistic]
            comparisons[key] = {"baseline": a, "candidate": b, "delta": b - a}
            if b > a:
                regressions.append(key)
    return result, comparisons, regressions


def compare(baseline, candidate, minimum_samples=20, *, baseline_vram=None, candidate_vram=None):
    if minimum_samples < 5:
        raise ValueError("Minimum sample count must be at least 5")
    setting_keys = ("detail_timers", "gte_validation", "optimization_override", "timer_unit_us_approx")
    for key in setting_keys:
        if (key in baseline) != (key in candidate) or baseline.get(key) != candidate.get(key):
            raise ValueError(f"Incomparable capture setting: {key}")
    before = summarize(baseline, minimum_samples)
    after = summarize(candidate, minimum_samples)
    for key in ("gte_validation_errors.max", "stream_failed_chunks.max"):
        if key in before and key not in after:
            raise ValueError(f"Candidate no longer reports baseline safety counter: {key}")
    grouped_cpu, grouped_metrics, grouped_regressions = cpu_by_steps(baseline, candidate)
    full_step_coverage = grouped_cpu["available"] and all(
        group["status"] == "compared" for group in grouped_cpu["groups"].values())
    grouped_cpu["covers_all_samples"] = full_step_coverage
    regressions = []
    comparisons = {}
    for key in sorted((before.keys() | after.keys()) - {"samples"}):
        # Optional safety counters absent in an older baseline represent no
        # observed errors; any errors in the candidate still reject acceptance.
        a, b = before.get(key, 0), after.get(key, 0)
        comparisons[key] = {"baseline": a, "candidate": b, "delta": b - a}
        # The mixture of one/two/etc-step frames can move this median even
        # when CPU cost is identical at every step count. Replace only this
        # gate, and only when every sampled step count is compared on both
        # sides. Global CPU tails and all frame-time/safety gates stay strict.
        non_gating = key == "frame_scanlines.median" and full_step_coverage
        if non_gating:
            comparisons[key]["non_gating_reason"] = (
                "Every sampled fixed-step count has at least five samples in both captures. "
                "CPU medians are gated within each step count; this aggregate median is "
                "informative because changing their proportions can move it without a cost increase. "
                "Global CPU p95/max, frame times and safety counters remain gating.")
        if b > a and not non_gating:
            regressions.append(key)
    comparisons.update(grouped_metrics)
    regressions.extend(grouped_regressions)
    visual = {"status": "not_checked"}
    if baseline_vram is not None or candidate_vram is not None:
        if baseline_vram is None or candidate_vram is None:
            raise ValueError("VRAM validation requires both baseline and candidate snapshots")
        if len(baseline_vram) != 1024 * 512 * 2 or len(candidate_vram) != 1024 * 512 * 2:
            raise ValueError("VRAM snapshots must each contain the complete 1 MiB PSX VRAM")
        same = baseline_vram == candidate_vram
        visual = {"status": "equal" if same else "different",
                  "baseline_sha256": hashlib.sha256(baseline_vram).hexdigest(),
                  "candidate_sha256": hashlib.sha256(candidate_vram).hexdigest()}
        if not same:
            regressions.append("vram")
    return {"passed": not regressions, "regressions": regressions,
            "samples": {"baseline": before["samples"], "candidate": after["samples"]},
            "metrics": comparisons,
            "cpu_by_steps": grouped_cpu,
            "capture_settings": {key: baseline[key] for key in setting_keys if key in baseline},
            "visual_validation": visual,
            "scope": "Sampled native workloads only. Match scene, camera path, emulator, duration and build settings; also measure cold loading and warm traversal."}


def load(path):
    if path.is_dir():
        path = path / "profile.json"
    report = documents.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(report, dict):
        raise ValueError("Profile must contain a JSON object")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True, type=Path)
    parser.add_argument("--candidate", required=True, type=Path)
    parser.add_argument("--min-samples", type=int, default=20)
    parser.add_argument("--require-vram", action="store_true", help="Require exact matching vram.bin snapshots beside profiles")
    parser.add_argument("--baseline-vram", type=Path, help="Explicit deterministic baseline VRAM snapshot (requires candidate pair)")
    parser.add_argument("--candidate-vram", type=Path)
    parser.add_argument("--output", type=Path, help="Optional new comparison JSON file")
    args = parser.parse_args()
    try:
        baseline_vram = candidate_vram = None
        if bool(args.baseline_vram) != bool(args.candidate_vram):
            raise ValueError("Supply both --baseline-vram and --candidate-vram")
        if args.require_vram or args.baseline_vram:
            def snapshot(profile, explicit):
                return (explicit or ((profile if profile.is_dir() else profile.parent) / "vram.bin")).read_bytes()
            baseline_vram = snapshot(args.baseline, args.baseline_vram)
            candidate_vram = snapshot(args.candidate, args.candidate_vram)
        result = compare(load(args.baseline), load(args.candidate), args.min_samples,
                         baseline_vram=baseline_vram, candidate_vram=candidate_vram)
        text = json.dumps(result, indent=2)
        if args.output:
            with args.output.open("x", encoding="utf-8") as output:
                output.write(text + "\n")
        print(text)
        return 0 if result["passed"] else 1
    except (OSError, ValueError, TypeError) as error:
        print(f"Cannot compare native captures: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
