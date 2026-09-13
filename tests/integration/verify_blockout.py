"""Editable source identity, compiled chunk visibility, material overrides and real PSX clipping."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents

from project_paths import project_manifest
import hashlib, json, pathlib, re, shutil, socket, struct, subprocess, time, urllib.request, uuid

ROOT = pathlib.Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
ART = ROOT / "artifacts"
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0


def uid():
    return str(uuid.uuid4())


def package(path, identity, doc):
    raw = json.dumps(doc).encode()
    meta = json.dumps(
        dict(
            version=1,
            id=identity,
            kind="EditableMesh",
            importer_version=1,
            source="assets/Room.epokasset",
            source_hash=hashlib.sha256(raw).hexdigest(),
        )
    ).encode()
    path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(meta), len(raw)) + meta + raw)


def main():
    ART.mkdir(exist_ok=True)
    folder = ROOT / ".epok" / ("mesh-verify-" + str(time.time_ns()))
    documents.write_text(ART / "blockout-project.txt", str(folder))

    def run(*args, success=True):
        p = subprocess.run(
            [str(EXE), *map(str, args)],
            capture_output=True,
            text=True,
            timeout=120,
            creationflags=FLAGS,
        )
        with (ART / "blockout-build.log").open("a", encoding="utf-8") as log:
            log.write(p.stdout + p.stderr)
        assert (p.returncode == 0) == success, p.stdout + p.stderr
        return p.stdout

    run("--create-project", folder, "--template", "sample")
    video_path=project_manifest(folder)
    video=documents.loads(video_path.read_text());video['rendering']=dict(width=320,height=240)
    documents.write_text(video_path, json.dumps(video))  # Fixed pixel-reference fixture.

    group = uid()
    floor = uid()
    walls = uid()
    slot_floor = uid()
    slot_wall = uid()
    identity = uid()
    doc = dict(
        version=1,
        vertices=[],
        faces=[],
        groups=[
            dict(id=group, name="Room", parent=None),
            dict(id=floor, name="Floor", parent=group),
            dict(id=walls, name="Walls", parent=group),
        ],
        materials=[
            dict(
                id=slot_floor, name="Floor", material=dict(color=[0, 1, 0], unlit=True)
            ),
            dict(
                id=slot_wall, name="Walls", material=dict(color=[1, 0, 0], unlit=True)
            ),
        ],
    )

    def face(points, g, s):
        base = len(doc["vertices"])
        doc["vertices"].extend(points)
        doc["faces"].append(
            dict(id=uid(), vertices=list(range(base, base + 4)), group=g, material=s)
        )

    face([[-5, -1, -5], [-5, -1, 7], [5, -1, 7], [5, -1, -5]], floor, slot_floor)
    # Three red wall pieces leave a real window opening; the camera looks through it.
    for x0, x1, y0, y1 in [(-5, -1, -1, 4), (1, 5, -1, 4), (-1, 1, 2, 4)]:
        face([[x0, y0, 5], [x0, y1, 5], [x1, y1, 5], [x1, y0, 5]], walls, slot_wall)
    # Off-camera surfaces must not consume projection work despite sharing the entity/material.
    for x in range(40, 81, 4):
        face([[x, -1, 5], [x, 3, 5], [x + 2, 3, 5], [x + 2, -1, 5]], walls, slot_wall)
    source = folder / "assets/Room.epokasset"
    package(source, identity, doc)
    scene_path = folder / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())
    camera = scene["entities"][0]
    camera.update(position=[0, 0, -3], rotation=[0, 0, 0])
    mesh = scene["entities"][1]
    mesh.update(
        position=[0, 0, 0],
        rotation=[0, 0, 0],
        scale=[1, 1, 1],
        editable_mesh=dict(
            asset=identity, materials={slot_wall: dict(color=[0, 0, 1], unlit=True)}
        ),
    )
    scene["entities"] = [camera, mesh]
    documents.write_text(scene_path, json.dumps(scene))
    documents.write_text(folder / "assets/scripts/Spinner.cpp", """#include "Spinner.hpp"
uint32_t mesh_probe[8]={};
void Spinner::start(epok::Transform&){}
void Spinner::update(epok::Transform&,epok::Fixed){++mesh_probe[0];auto s=epok::mesh_stats;mesh_probe[1]=s.tested_chunks;mesh_probe[2]=s.visible_chunks;mesh_probe[3]=s.transformed_vertices;mesh_probe[4]=s.backfaces;mesh_probe[5]=s.clipped;mesh_probe[6]=epok::lighting_stats.dropped_triangles;mesh_probe[7]=epok::lighting_stats.triangles;}
""")
    moved = folder / "assets/Renamed.epokasset"
    source.rename(moved)
    run("--project", folder, "--build-psx")
    assert scene_path.read_text() == documents.dumps(
        scene
    ), "Build rewrote authoring references"
    assert identity in run("--project", folder, "--scan-assets")
    duplicate = folder / "assets/Duplicate.epokasset"
    shutil.copyfile(moved, duplicate)
    run("--project", folder, "--build-psx", success=False)
    duplicate.unlink()
    with socket.socket() as port:
        port.bind(("127.0.0.1", 8077))

    def request(path, post=False):
        req = urllib.request.Request(
            "http://127.0.0.1:8077/api/v1/" + path, data=b"" if post else None
        )
        with urllib.request.urlopen(req, timeout=3) as response:
            return response.read()

    with (ART / "blockout-emulator.log").open("w") as log:
        p = subprocess.Popen(
            [str(EXE), "--project", str(folder), "--play-psx", "--stop-after", "12"],
            stdout=log,
            stderr=log,
            creationflags=FLAGS,
        )
        try:
            deadline = time.monotonic() + 45
            while True:
                assert p.poll() is None, "See blockout-emulator.log"
                try:
                    if json.loads(request("execution-flow"))["running"]:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline
                time.sleep(0.1)
            time.sleep(2)
            request("execution-flow?function=pause", True)
            raw = request("gpu/vram/raw")
            screen = b"".join(raw[y * 2048 : y * 2048 + 640] for y in range(240))
            (ART / "blockout.vram").write_bytes(screen)
            pixels = struct.unpack("<76800H", screen)
            rgb = lambda v: (v & 31, (v >> 5) & 31, (v >> 10) & 31)
            assert (
                sum(1 for v in pixels if rgb(v)[2] > 20 and rgb(v)[0] < 5) > 500
            ), "Blue material override missing"
            assert (
                sum(1 for v in pixels if rgb(v)[1] > 20 and rgb(v)[0] < 5) > 500
            ), "Green floor missing (near-plane clipping)"
            assert not any(
                rgb(v)[0] > 20 and rgb(v)[1] < 5 and rgb(v)[2] < 5 for v in pixels
            ), "Default red leaked through override"
            assert rgb(pixels[120 * 320 + 160]) == rgb(
                pixels[20 * 320 + 160]
            ), "Window opening is filled by phantom cube geometry"
            symbols = (folder / ".epok/build/epok.map").read_text()
            address = (
                int(re.search(r"0x([0-9a-f]+)\s+mesh_probe\b", symbols)[1], 16)
                & 0x1FFFFF
            )
            values = struct.unpack_from("<8I", request("cpu/ram/raw"), address)
            documents.write_text(ART / "blockout-probe.json", json.dumps(values))
            assert (
                values[1] > values[2] > 0
                and values[3] > 0
                and values[5] > 0
                and values[6] == 0
            ), values
            assert p.wait(timeout=20) == 0
            print(
                "PASS editable asset UUID move/conflict, material slots/override, real window opening, chunk culling and near-plane clipping",
                values,
                flush=True,
            )
        finally:
            if p.poll() is None:
                p.wait(timeout=45)

    # A sloped, nonuniformly scaled mesh must use its actual normal in both
    # offline lighting and the native GTE path. The reversed second face also
    # exercises backface rejection independently of the window test.
    doc["vertices"] = []
    doc["faces"] = []
    face([[-2, -1, 0], [-2, 1, 2], [2, 1, 2], [2, -1, 0]], floor, slot_floor)
    face([[-2, -1, 4], [2, -1, 4], [2, 1, 6], [-2, 1, 6]], walls, slot_floor)
    doc["materials"][0]["material"] = dict(color=[1, 1, 1], unlit=False)
    package(moved, identity, doc)
    camera.update(position=[0, 1, -5], rotation=[0, 0, 0])
    mesh["editable_mesh"]["materials"] = {}
    mesh.update(scale=[1.5, 0.75, 1.25], rotation=[12, 15, 5])
    mesh["lighting"] = dict(static_geometry=True, receive="Baked", cast_shadows=False)
    sun = dict(
        name="Sun",
        kind="Empty",
        position=[0, 0, 0],
        rotation=[20, 0, 0],
        scale=[1, 1, 1],
        light=dict(kind="Directional", mode="Mixed", shadows=False, intensity=0.7),
    )
    scene["entities"] = [camera, mesh, sun]
    scene["environment"] = dict(ambient=[0.1, 0.1, 0.1], ao_strength=0)
    captures = []
    for mode in ["Baked", "Realtime"]:
        mesh["lighting"]["receive"] = mode
        documents.write_text(scene_path, json.dumps(scene))
        run("--project", folder, "--build-psx")
        with (ART / ("blockout-" + mode + ".log")).open("w") as log:
            process = subprocess.Popen(
                [
                    str(EXE),
                    "--project",
                    str(folder),
                    "--play-psx",
                    "--stop-after",
                    "10",
                ],
                stdout=log,
                stderr=log,
                creationflags=FLAGS,
            )
            try:
                deadline = time.monotonic() + 45
                while True:
                    assert process.poll() is None, (
                        "Emulator exited; see blockout-" + mode + ".log"
                    )
                    try:
                        if json.loads(request("execution-flow"))["running"]:
                            break
                    except (OSError, ValueError):
                        pass
                    assert time.monotonic() < deadline
                    time.sleep(0.1)
                time.sleep(1.5)
                request("execution-flow?function=pause", True)
                raw = request("gpu/vram/raw")
                screen = b"".join(raw[y * 2048 : y * 2048 + 640] for y in range(240))
                (ART / ("blockout-" + mode + ".vram")).write_bytes(screen)
                captures.append(struct.unpack("<76800H", screen))
                symbols = (folder / ".epok/build/epok.map").read_text()
                address = (
                    int(re.search(r"0x([0-9a-f]+)\s+mesh_probe\b", symbols)[1], 16)
                    & 0x1FFFFF
                )
                stats = struct.unpack_from("<8I", request("cpu/ram/raw"), address)
                assert stats[4] > 0 and stats[6] == 0, stats
                assert process.wait(timeout=20) == 0
            finally:
                if process.poll() is None:
                    process.wait(timeout=45)
    lit = [
        i
        for i, v in enumerate(captures[0])
        if rgb(v)[0] > 8 and rgb(v)[0] == rgb(v)[1] == rgb(v)[2]
    ]
    assert len(lit) > 500, "Sloped mesh failed to receive light"
    difference = sum(
        abs(rgb(captures[0][i])[0] - rgb(captures[1][i])[0]) for i in lit
    ) / len(lit)
    assert difference <= 2, ("Baked/GTE normal mismatch", difference)
    print(
        "PASS nonuniform sloped normals, baked/GTE agreement and backface rejection",
        difference,
        flush=True,
    )


if __name__ == "__main__":
    main()
