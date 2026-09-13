"""Real MIPS/CD acceptance: demand and whole-archive pools, camera changes,
eviction/reload, retained packets and exact native VRAM equivalence.
All generated projects and measurements use a new folder; port 8092 is reserved.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PORT = 8092


def uid():
    return str(uuid.uuid4())


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")


def package(path, identity, doc):
    payload = json.dumps(doc).encode()
    meta = json.dumps(dict(version=1, id=identity, kind="EditableMesh", importer_version=1,
                           source=f"assets/{path.name}", source_hash=hashlib.sha256(payload).hexdigest())).encode()
    path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(meta), len(payload)) + meta + payload)


def request(path, data=None):
    req = urllib.request.Request(f"http://127.0.0.1:{PORT}/api/v1/" + path, data=data)
    with urllib.request.urlopen(req, timeout=5) as response:
        return response.read()


def assert_stream_pool_in_bss(symbols, executable, pool_pages):
    """A nonzero metadata sentinel must not serialize the entire zero page pool."""
    matches = list(re.finditer(
        r"^\s*\.(bss|sbss|data|sdata)\._ZN5epok11stream_poolE\s+"
        r"0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)\b", symbols, re.MULTILINE))
    assert len(matches) == 1, "Expected one linked streaming pool in MIPS map"
    section, address, size = matches[0].groups()
    address, size = int(address, 16), int(size, 16)
    assert section in ("bss", "sbss"), f"Page pool serialized in .{section}: {size} bytes"
    assert size >= pool_pages * 65536, ("Page payload missing from pool", size, pool_pages)
    assert executable.startswith(b"PS-X EXE") and len(executable) >= 2048
    load, loaded_bytes = struct.unpack_from("<II", executable, 0x18)
    # PS-X EXE's load image includes .text/.rodata/.data; the pool must be
    # entirely outside this range, not merely filled with compressible zeroes.
    assert address >= load + loaded_bytes or address + size <= load, (
        "Page pool overlaps serialized PS-X EXE load image", hex(address), size,
        hex(load), loaded_bytes)
    return dict(section=section, address=address, bytes=size, page_bytes=pool_pages * 65536)


PROBE = r'''#include "StreamProbe.hpp"
extern "C" {volatile uint32_t stream_command=0,stream_probe[13]={};}
void StreamProbe::start(epok::Transform&){}
void StreamProbe::update(epok::Transform&,epok::Fixed){}
void StreamProbe::frame_update(epok::Transform&,uint32_t){
    using namespace epok;
    const uint32_t phase=stream_command;
    auto* camera=find_entity("Camera");auto* parent=find_entity("Parent");
    const int x=phase==0?-70:(phase==1?0:((phase==2||phase==8)?70:-70));
    camera->transform.position[0]=Fixed(x*4096,Fixed::RAW);camera->transform.position[1]=Fixed(1);camera->transform.position[2]=Fixed(0);
    camera->transform.rotation[1]=phase==5?Fixed(12):Fixed(0);
    camera->camera_settings.field_of_view=phase==4?Fixed(45):Fixed(90);
    parent->transform.position[1]=phase==3?Fixed(1):Fixed(0);
    parent->transform.rotation[2]=phase==3?Fixed(2):Fixed(0);
    // Alternating materials on successive rendered frames expose shared-record
    // parity bugs that a fixed color cannot. Fog consumes cached shaded colors.
    const bool alternating=phase>=7&&phase<=9;
    const uint8_t tint=(performance_stats.frame&1)?255:80;
    const char* names[]={"Panels0","Panels1","Panels2"};
    for(const char* name:names){
        auto* object=find_entity(name);
        object->material.color[0]=alternating?tint:255;
        object->material.color[1]=alternating?255-tint:255;
        object->material.color[2]=alternating?128:255;
    }
    fog_environment.enabled=phase==7||phase==8;
    fog_environment.start=2*4096;fog_environment.end=12*4096;
    fog_environment.color[0]=30;fog_environment.color[1]=80;fog_environment.color[2]=110;
    stream_probe[0]=0x5354524d;stream_probe[1]=stream_probe[1]+1;
    stream_probe[3]=stream_probe[2]==phase?stream_probe[3]+1:0;stream_probe[2]=phase;
    stream_probe[4]=performance_stats.triangles;stream_probe[5]=performance_stats.stream_failed_chunks;
    stream_probe[6]=performance_stats.visibility_skipped_chunks;
    stream_probe[7]=performance_stats.streamed_chunks;
    stream_probe[8]=performance_stats.retained_triangles;
    stream_probe[9]=performance_stats.retained_rebuilds;
    stream_probe[10]=performance_stats.frame;
    stream_probe[11]=lighting_stats.dropped_triangles;
    stream_probe[12]=performance_stats.visible_chunks;
}
'''


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    out = args.output or ROOT / "artifacts/streaming" / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=False)
    folder = out / "project"
    # All six modes use the same instrumentation. streamed_chunks is a
    # detail-only diagnostic; this suite checks correctness, not release FPS.
    environment = os.environ.copy()
    environment["EPOK_PROFILE_DETAIL"] = "1"
    with socket.socket() as available:
        available.bind(("127.0.0.1", PORT))

    def run(*arguments):
        result = subprocess.run([str(EXE), *map(str, arguments)], capture_output=True, text=True,
                                creationflags=FLAGS, timeout=240, env=environment)
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    run("--create-project", folder, "--template", "sample")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator"):
        config[key] = str((ROOT / config[key]).resolve())
    config["web_port"] = PORT
    save(folder / "Local.epokconfig", config)
    manifest_path = project_manifest(folder)
    manifest = documents.loads(manifest_path.read_text())
    manifest["rendering"] = dict(width=320, height=240, retained_geometry=False,
                                 streaming_pool_pages=2, streaming_geometry=False, precomputed_visibility=False,
                                 streaming_triangle_budget=6144)
    save(manifest_path, manifest)
    identities = []
    # Split by region so each object fits the bounded retained-record pool;
    # the sum of page payloads still exceeds the two-slot residency pool.
    for region, center in enumerate((-70, 0, 70)):
        group, slot, identity = uid(), uid(), uid()
        identities.append(identity)
        doc = dict(version=1, vertices=[], faces=[], groups=[dict(id=group, name="Panels", parent=None)],
                   materials=[dict(id=slot, name="Surface", material=dict(color=[0.25, 0.75, 1.0], unlit=True))])
        for row in range(25):
            for col in range(32):
                x, y = center - 1.6 + col * 0.1, -0.25 + row * 0.1
                base = len(doc["vertices"])
                doc["vertices"].extend([[x, y, 8], [x, y+0.09, 8], [x+0.09, y+0.09, 8], [x+0.09, y, 8]])
                doc["faces"].append(dict(id=uid(), vertices=list(range(base, base+4)), group=group, material=slot))
        package(folder / f"assets/Streaming{region}.epokasset", identity, doc)
    def entity(name, kind="Empty", **extra):
        return dict(name=name, kind=kind, position=[0, 0, 0], rotation=[0, 0, 0], scale=[1, 1, 1], **extra)
    scene = dict(version=1, name="SampleScene", entities=[entity("Camera", "Camera"), entity("Parent"),
                 *[entity(f"Panels{i}", "Mesh", parent=1, editable_mesh=dict(asset=identity), material=dict(unlit=True))
                   for i, identity in enumerate(identities)], entity("Probe", script=dict(name="StreamProbe"))])
    save(folder / "assets/scenes/SampleScene.epokmap", scene)
    documents.write_text(folder / "assets/scripts/StreamProbe.hpp", '#pragma once\n#include "epok.hpp"\nclass StreamProbe:public epok::Behaviour {public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override;void frame_update(epok::Transform&,uint32_t)override;};\n')
    documents.write_text(folder / "assets/scripts/StreamProbe.cpp", PROBE)
    save(folder / "assets/scripts/StreamProbe.epokscript", dict(name="StreamProbe", properties=[]))
    results = {}
    reference = {}
    for mode in ("resident", "visibility", "streaming", "resident_retained", "streaming_retained",
                 "streaming_fits_retained"):
        streaming = mode.startswith("streaming")
        retained = mode.endswith("_retained")
        archive_fits = mode == "streaming_fits_retained"
        manifest["rendering"]["precomputed_visibility"] = mode == "visibility" or streaming
        manifest["rendering"]["streaming_geometry"] = streaming
        manifest["rendering"]["retained_geometry"] = retained
        manifest["rendering"]["streaming_pool_pages"] = 4 if archive_fits else 2
        save(manifest_path, manifest)
        run("--project", folder, "--build-psx")
        build = folder / ".epok/build"
        symbols = (build / "epok.map").read_text()
        def address(name):
            match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
            assert match, name
            return int(match[1], 16) & 0x1fffff
        command = address("stream_command")
        probe = address("stream_probe")
        counters = address("epok::streaming_stats") if streaming else None
        page_count = (build / "GEOMETRY.BIN").stat().st_size // 65536 if streaming else 0
        if streaming:
            assert page_count == 4, "Fixture must exercise four archive pages"
            assert archive_fits == (page_count <= manifest["rendering"]["streaming_pool_pages"])
            assert (build / "epok.cue").is_file(), "Streaming must boot with an actual disc"
            header = (build / "scene.hh").read_text()
            assert "inline constexpr MeshQuad editable_2_" not in header, "Payload remains prelinked"
        report = dict(executable_bytes=(build / "epok.ps-exe").stat().st_size, pages=page_count,
                      pool_pages=manifest["rendering"]["streaming_pool_pages"] if streaming else 0,
                      archive_fits=archive_fits, detail_timers=True, phases=[])
        if streaming:
            report["pool_storage"] = assert_stream_pool_in_bss(
                symbols, (build / "epok.ps-exe").read_bytes(),
                manifest["rendering"]["streaming_pool_pages"])
        log_path = out / f"{mode}-runtime.log"
        with log_path.open("w") as log:
            process = subprocess.Popen([str(EXE), "--project", str(folder), "--play-psx", "--stop-after", "240"],
                                       stdout=log, stderr=log, creationflags=FLAGS, env=environment)
            try:
                deadline = time.monotonic() + 90
                while True:
                    assert process.poll() is None, log_path.read_text(errors="replace")
                    try:
                        # --play-psx starts execution. A bootstrap resume can
                        # race emulator initialization and crash the host before
                        # its first frame; wait for the native probe instead.
                        values = struct.unpack_from("<13I", request("cpu/ram/raw"), probe)
                        if values[0] == 0x5354524d and values[3] >= 5:
                            break
                    except (OSError, ValueError):
                        pass
                    assert time.monotonic() < deadline, "No completed native frames"
                    time.sleep(0.1)
                for phase in range(11):
                    request("execution-flow?function=pause", b"")
                    request(f"cpu/ram/raw?offset={command}&size=4", struct.pack("<I", phase))
                    request("execution-flow?function=resume", b"")
                    deadline = time.monotonic() + 30
                    while True:
                        ram = request("cpu/ram/raw")
                        values = struct.unpack_from("<13I", ram, probe)
                        if values[2] == phase and values[3] >= 8:
                            break
                        assert time.monotonic() < deadline, (mode, phase, values)
                        time.sleep(0.05)
                    request("execution-flow?function=pause", b"")
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<13I", ram, probe)
                    vram = request("gpu/vram/raw")
                    (out / f"{mode}-{phase}.vram").write_bytes(vram)
                    assert values[4] > 0 and values[5] == 0 and values[11] == 0, (mode, phase, values)
                    # Every visible mesh in this fixture is editable; streaming
                    # must account for all successful chunks, including after
                    # switching between resident and whole-archive render paths.
                    assert values[7] == (values[12] if streaming else 0), (mode, phase, values)
                    if retained:
                        assert values[8] > 0, ("Retention was not exercised", mode, phase, values)
                    else:
                        assert values[8] == 0, (mode, phase, values)
                    if mode == "resident":
                        reference[phase] = vram
                    else:
                        assert vram == reference[phase], f"Native VRAM differs: {mode} phase {phase}"
                    row = dict(phase=phase, probe=list(values), vram_sha256=hashlib.sha256(vram).hexdigest())
                    if counters is not None:
                        row["streaming"] = list(struct.unpack_from("<7I", ram, counters))
                        assert row["streaming"][4:6] == [0, 0], row
                        if archive_fits:
                            assert row["streaming"][0] == page_count, (
                                "Whole-archive warmup must load each page once without subsequent reads", row)
                    report["phases"].append(row)
                    print(f"PASS {mode} phase {phase}, {values[4]} triangles", flush=True)
                if streaming:
                    reads = report["phases"][-1]["streaming"][0]
                    if archive_fits:
                        assert reads == page_count, "Whole-archive pages must not be evicted or reloaded"
                    else:
                        assert reads > page_count, "Returning to an evicted region must reload pages"
                request("execution-flow?function=resume", b"")
            finally:
                # Close only the process tree this test launched. Leaving a
                # failed child emulator alive would steal the next test's port.
                if process.poll() is None:
                    subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                   capture_output=True, creationflags=FLAGS, check=False)
                process.wait(timeout=15)
        results[mode] = report
        save(out / "report.json", results)
    assert results["streaming"]["executable_bytes"] < results["resident"]["executable_bytes"], results
    print(f"PASS geometry CD streaming, visibility, eviction/reload and native VRAM equivalence: {out}", flush=True)


if __name__ == "__main__":
    main()
