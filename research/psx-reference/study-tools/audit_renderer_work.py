"""Read-only work census for the reference rail import and its cooked PSX geometry.

This counts actual records; it does not simulate the GTE, GPU, or instruction cache.
Original SLST counts are pre-screen-reject and exclude sky/disabled world slots.
"""
import argparse
import hashlib
import json
import re
import statistics
import struct
import subprocess
from pathlib import Path


def array(header, name):
    body = re.search(r"\b" + name + r"\[.*?\]\s*=\s*(\{[^;]+\});", header)[1]
    return json.loads(body.replace("{", "[").replace("}", "]"))


def apply_delta(selected, data):
    data = iter(data)
    def varint(first):
        value, shift, byte = 0, 0, first
        while True:
            value |= (byte & 127) << shift
            if not byte & 128:
                return value
            shift += 7
            byte = next(data)
    cursor = 0
    for first in data:
        code = varint(first)
        cursor += code >> 1
        count = varint(next(data)) + 2 if code & 1 else 1
        for face in range(cursor, cursor + count):
            selected.symmetric_difference_update((face,))
        cursor += count


def census(build, routes_path, renderer_path, sample_index):
    header = (build / "scene.hh").read_text()
    samples = array(header, "track_samples")
    changes = array(header, "track_changes")
    rails = array(header, "rail_routes")
    route_index, rail = next((i, r) for i, r in enumerate(rails) if r[0] <= sample_index < r[0] + r[1])
    selected = set()
    for sample in samples[rail[0]:sample_index + 1]:
        offset, length = sample[4:6]
        apply_delta(selected, changes[offset:offset + length])
    archive = (build / "GEOMETRY.BIN").read_bytes()
    chunks = []
    pattern = (r"MeshGeometry (editable_\d+_\d+)=\{nullptr,(\d+), nullptr,(\d+),(true|3),"
               r"\{[^}]+\},\{[^}]+\},\{[^}]+\},[^,]+,[^,]+,(\d+),(\d+),(\d+),track_current\+(\d+)(?:,\d+,\d+)?\};")
    for match in re.finditer(pattern, header):
        name = match[1]
        vertices, faces = map(int, match.groups()[1:3])
        stride = 32 if match[4] == '3' else 72
        page, voffset, qoffset, word = map(int, match.groups()[4:])
        active = [q for q in range(faces) if word * 32 + q in selected]
        used = set()
        for q in active:
            used.update(struct.unpack_from("<4H", archive, page * 65536 + qoffset + q * stride))
        assert all(v < vertices for v in used)
        chunks.append(dict(name=name, vertices=vertices, faces=faces, page=page,
                           selected_faces=len(active), selected_vertices=len(used),
                           scratchpad=vertices <= 51, face_stride=stride))
    assert len(chunks) == 482, "Unexpected cooker ABI/chunk census"
    assert sum(c["selected_faces"] for c in chunks) == len(selected)
    visible = [c for c in chunks if c["selected_faces"]]
    routes = json.loads(routes_path.read_text())["routes"]
    route = routes[route_index]
    source = route["Samples"][sample_index - rail[0]]
    skies = {w["eid"] for w in json.loads(renderer_path.read_text())["worlds"] if w["sky"]}
    polygons = [p for p in source["Polygons"] if source["WorldMask"] & (1 << (p >> 13))
                and route["Worlds"][p >> 13] not in skies]
    states = [sum(((p >> 11) & 3) == state for p in polygons) for state in range(4)]
    source_triangles = sum(states[:3]) + states[3] * 2
    assert source_triangles == len(selected), "Import selection differs from filtered original SLST"
    return dict(sample=sample_index, route=route_index, local_sample=source["Index"],
                original=dict(slst_records=len(polygons), triangle_records=states[0],
                              half_quad_records=states[1] + states[2], full_quad_records=states[3],
                              equivalent_triangles=source_triangles,
                              referenced_topology_record_bytes_excluding_shared_tables=states[0]*6 + sum(states[1:])*8,
                              geometry_projection_invocations=sum(states[:3])*3 + states[3]*4,
                              selection_id_bytes=len(polygons)*2),
                epok=dict(total_chunks=len(chunks), pvs_chunks=len(visible),
                          total_face_records=sum(c["faces"] for c in chunks),
                          pvs_chunk_face_records=sum(c["faces"] for c in visible),
                          selected_face_records=len(selected),
                          selected_face_record_bytes=sum(c['selected_faces']*c['face_stride'] for c in visible),
                          pvs_chunk_vertices=sum(c["vertices"] for c in visible),
                          selected_vertices=sum(c["selected_vertices"] for c in visible),
                          selected_vertices_in_scratchpad_chunks=sum(c["selected_vertices"] for c in visible if c["scratchpad"]),
                          pvs_pages=sorted({c["page"] for c in visible}),
                          face_scan_iterations_two_passes=(2*sum(c["faces"] for c in visible)
                              if all(c['face_stride']==72 for c in visible) else None),
                          uncached_face_scan_iterations_two_passes=2*sum(c['faces'] for c in visible)),
                chunks=visible)


def profile(path, capture):
    report = json.loads((path / "profile.json").read_text())
    rows = report["frames"][capture - 1:]
    assert len(rows) >= 30
    def pose(r):
        n = r["rail_navigation"]
        return (r["visibility_track"]["sample"], n["route"], n["next_sample"], n["fraction"], tuple(n["target_position"]))
    assert len({pose(r) for r in rows}) == 1 and pose(rows[0])[0] == 93
    for field in ("stream_failed_chunks", "stream_budget_skipped_chunks", "dropped_triangles", "dropped_steps"):
        assert max(r[field] for r in rows) == 0
    assert len({r["streaming"]["reads"] for r in rows}) == 1
    assert all(r["streaming"]["errors"] == 0 for r in rows)
    fields = [f for f in rows[0] if f.endswith("scanlines") or f in
              ("gte_vertices", "software_vertices", "triangles", "retained_triangles", "visible_chunks", "frame_microseconds")]
    median = {f: statistics.median(r[f] for r in rows) for f in fields}
    median["render_outside_vertex_polygon_scanlines"] = statistics.median(
        r["render_scanlines"]-r["vertex_scanlines"]-r["polygon_scanlines"] for r in rows)
    median["render_outside_setup_vertex_polygon_scanlines"] = statistics.median(
        r["render_scanlines"]-r["setup_scanlines"]-r["vertex_scanlines"]-r["polygon_scanlines"] for r in rows)
    median["polygon_outside_detail_scanlines"] = statistics.median(
        r["polygon_scanlines"]-r["shade_scanlines"]-r["fog_scanlines"]-r["emit_scanlines"] for r in rows)
    return dict(rows=len(rows), pose=pose(rows[0]), median=median, build=report["build"])


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--build", type=Path, required=True)
    p.add_argument("--routes", type=Path, required=True)
    p.add_argument("--renderer", type=Path, required=True)
    p.add_argument("--baseline", type=Path, required=True)
    p.add_argument("--detail", type=Path, required=True)
    p.add_argument("--toolchain", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    result = census(a.build, a.routes, a.renderer, 93)
    result["baseline"] = profile(a.baseline, 220)
    result["detail"] = profile(a.detail, 180)
    for key in ("scene_header_sha256", "display_header_sha256"):
        assert result["baseline"]["build"][key] == result["detail"]["build"][key]
    for key in ("gte_vertices", "triangles", "visible_chunks"):
        assert result["baseline"]["median"][key] == result["detail"]["median"][key]
    va, vb = [(d / "vram.bin").read_bytes() for d in (a.baseline, a.detail)]
    assert len(va) == len(vb) == 1048576
    result["differing_framebuffer_pixels"] = sum(va[y*2048+x:y*2048+x+2] != vb[y*2048+x:y*2048+x+2]
                                                 for y in range(480) for x in range(0, 1024, 2))
    assert result["differing_framebuffer_pixels"] == 0
    nm = subprocess.check_output([str(a.toolchain / "mipsel-none-elf-nm.exe"), "--size-sort", "--print-size", "--demangle", str(a.build / "epok.elf")], text=True)
    frame = next(line for line in nm.splitlines() if line.endswith("(anonymous namespace)::GameScene::frame()"))
    address, size = [int(v, 16) for v in frame.split()[:2]]
    assembly = subprocess.check_output([str(a.toolchain / "mipsel-none-elf-objdump.exe"), "-d", "-C",
                                       f"--start-address={address}", f"--stop-address={address+size}", str(a.build / "epok.elf")], text=True)
    executable_hash = hashlib.sha256((a.build / "epok.ps-exe").read_bytes()).hexdigest()
    variant = next(k for k in ("baseline", "detail") if result[k]["build"]["executable_sha256"] == executable_hash)
    result["compiled_frame"] = dict(variant=variant, address=f"0x{address:08x}", code_bytes=size,
                                    rtps_sites=len(re.findall(r":\s+4a180001\b", assembly)))
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.with_suffix(".asm.txt").write_text(assembly)
    a.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k:v for k,v in result.items() if k != "chunks"}, indent=2))


if __name__ == "__main__":
    main()
