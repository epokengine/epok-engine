"""Check generated residency tables against every cooked visibility mask.

Read-only with respect to authored maps/builds. Supports the single-bank study
and player QA builds; the optional report is a new verification artifact.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
from audit_renderer_work import array, apply_delta


def verify(build):
    header = (build / "scene.hh").read_text()
    manifests = json.loads((build / "scene-stream-bake.json").read_text())
    assert len(manifests) == 1, "This diagnostic expects one tracked bank"
    bake = manifests[0]
    ids = array(header, "track_page_ids")
    indices = array(header, "track_page_samples")
    body = re.search(r"track_page_sets\[\]=\{([^;]+)\};", header)[1]
    spans = [(int(a), int(b)) for a, b in re.findall(r"\{track_page_ids\+(\d+),(\d+)\}", body)]
    samples = array(header, "track_samples")
    changes = array(header, "track_changes")
    checkpoints = {0}
    if re.search(r"\brail_routes\[", header):
        checkpoints.update(r[0] for r in array(header, "rail_routes"))
    assert len(indices) == len(samples) == len(bake["samples"])
    assert len(spans) > 0
    sets, selected = [], set()
    for i, (sample, expected_bits) in enumerate(zip(samples, bake["samples"])):
        if i in checkpoints:
            selected.clear()
        # Compressed camera samples and flat samples have different prefixes.
        offset, length = sample[4:6] if len(sample) == 7 else sample[3:5]
        apply_delta(selected, changes[offset:offset + length])
        assert selected == set(expected_bits), ("mask", i)
        expected = {c["page"] for c in bake["chunks"]
                    if any(c["word"] * 32 + q in selected for q in range(len(c["quads"])))}
        begin, count = spans[indices[i]]
        assert begin + count <= len(ids)
        actual = ids[begin:begin + count]
        assert actual == sorted(expected), ("pages", i, actual, expected)
        sets.append(set(actual))
    peak = max(map(len, sets), default=0)
    transition = max([peak] + [len(sets[a] | sets[b]) for a, b in bake["edges"]])
    assert transition <= bake["pool_pages"]
    pages = len((build / "GEOMETRY.BIN").read_bytes()) // 65536
    assert all(0 <= p < pages for p in ids)
    return dict(samples=len(samples), chunks=len(bake["chunks"]),
                unique_sets=len(spans), dictionary_page_ids=len(ids),
                table_bytes=len(indices)*2+len(spans)*8+len(ids)*4,
                peak_pages=peak, transition_pages=transition,
                pool_pages=bake["pool_pages"], archive_pages=pages,
                continuous_edges=len(bake["edges"]),
                scene_header_sha256=hashlib.sha256(header.encode()).hexdigest())


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = json.dumps(verify(args.build), indent=2) + "\n"
    if args.output:
        args.output.write_text(report)
    print(report)
