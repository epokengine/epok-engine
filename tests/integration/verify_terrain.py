"""Terrain cooking: chunk arrays in the editable namespace, atlas UVs, the
heightfield collider, and the budget gates that keep a grid inside PSX limits."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents

from project_paths import project_manifest
import hashlib, json, pathlib, re, struct, subprocess, time, uuid

ROOT = pathlib.Path(__file__).resolve().parents[2]
EXE = next(
    (
        candidate
        for candidate in (
            ROOT / "target/debug/epok-editor",
            ROOT / "target/debug/epok-editor.exe",
        )
        if candidate.is_file()
    ),
    ROOT / "target/debug/epok-editor",
)
ART = ROOT / "artifacts"
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0

MAGIC = b"EPTR"
VERSION = 1
HEIGHT_UNIT = 1.0 / 256.0


def uid():
    return str(uuid.uuid4())


def grid(cells_x, cells_z, cell_size, height):
    """Build the binary terrain payload the editor writes.

    Deliberately assembled here byte by byte rather than through the editor, so
    a change to the on-disk layout fails this test instead of silently
    round-tripping through its own writer.
    """
    body = struct.pack(
        "<4sHHHiH",
        MAGIC,
        VERSION,
        cells_x,
        cells_z,
        int(round(cell_size * 4096)),
        0,
    )
    assert len(body) == 16, len(body)
    heights = bytearray()
    for j in range(cells_z + 1):
        for i in range(cells_x + 1):
            heights += struct.pack("<h", int(round(height(i, j) / HEIGHT_UNIT)))
    materials = bytes(cells_x * cells_z)
    return body + bytes(heights) + materials


def package(path, identity, source, raw, kind="Terrain"):
    # Terrain is authored, never imported. The settings tag is written out
    # explicitly because an absent one deserializes to the legacy audio
    # default, which the kind/settings pairing correctly refuses.
    meta = json.dumps(
        dict(
            version=1,
            id=identity,
            kind=kind,
            importer_version=1,
            source=source,
            source_hash=hashlib.sha256(raw).hexdigest(),
            settings=dict(type="Authored"),
        )
    ).encode()
    path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(meta), len(raw)) + meta + raw)


def main():
    ART.mkdir(exist_ok=True)
    folder = ROOT / ".epok" / ("terrain-verify-" + str(time.time_ns()))
    documents.write_text(ART / "terrain-project.txt", str(folder))

    def run(*args, success=True):
        p = subprocess.run(
            [str(EXE), *map(str, args)],
            capture_output=True,
            text=True,
            timeout=180,
            creationflags=FLAGS,
        )
        with (ART / "terrain-build.log").open("a", encoding="utf-8") as log:
            log.write(p.stdout + p.stderr)
        assert (p.returncode == 0) == success, p.stdout + p.stderr
        return p.stdout + p.stderr

    run("--create-project", folder, "--template", "sample")
    video_path = project_manifest(folder)
    video = documents.loads(video_path.read_text())
    video["rendering"] = dict(width=320, height=240)
    documents.write_text(video_path, json.dumps(video))

    identity = uid()
    source = "assets/Ground.epokasset"
    # A 16 x 16 grid of 4-unit cells: 64 x 64 units, a gentle ridge along X.
    cells = 16
    package(
        folder / "assets/Ground.epokasset",
        identity,
        source,
        grid(cells, cells, 4.0, lambda i, j: min(i, cells - i) * 0.5),
    )

    scene_path = folder / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())

    def component(actor, class_name):
        for c in actor["components"]:
            if c["class"]["name"] == class_name:
                return c
        raise AssertionError(f"{actor['name']} has no {class_name}")

    def override(actor, class_name, key, value):
        c = component(actor, class_name)
        c["properties"][key] = value
        if key not in c["overrides"]:
            c["overrides"].append(key)

    # The sample scene's Ground is a scaled slab. Terrain collision indexes the
    # grid from the collider box's world minimum, so it has to be unrotated and
    # unscaled; the cell size carries the size instead.
    ground = next(a for a in scene["actors"] if a["name"] == "Ground")
    ground_index = scene["actors"].index(ground)
    override(ground, "epok::SceneComponent3D", "position", [0.0, 0.0, 0.0])
    override(ground, "epok::SceneComponent3D", "rotation", [0.0, 0.0, 0.0])
    override(ground, "epok::SceneComponent3D", "scale", [1.0, 1.0, 1.0])
    override(ground, "epok::Mesh3DComponent", "editable_mesh", None)
    terrain = dict(asset=identity, atlas=[2, 2], collision=True, merge=0)
    override(ground, "epok::Mesh3DComponent", "terrain", dict(terrain))

    def write_scene():
        documents.write_text(scene_path, json.dumps(scene))

    write_scene()

    run("--project", folder, "--build-psx")
    assert scene_path.read_text() == documents.dumps(
        scene
    ), "Build rewrote authoring references"

    header = (folder / ".epok/build/scene.hh").read_text(encoding="utf-8")
    # Cooked symbols are named after the actor's index in the scene.
    chunk_symbol = "editable_" + str(ground_index)
    object_symbol = "objects[" + str(ground_index) + "]"
    heights_symbol = "terrain_heights_" + str(ground_index)

    def quads_of(text):
        """Quads the cooked chunks declare.

        Read from each MeshGeometry's own quad_count rather than by counting
        braces in the array literal: a MeshQuad contains nested braces of its
        own, so brace counting silently multiplies the answer.
        """
        declarations = re.findall(
            r"MeshGeometry "
            + chunk_symbol
            + r"_\d+=\{"
            + chunk_symbol
            + r"_\d+_vertices,\d+, "
            + chunk_symbol
            + r"_\d+_faces,(\d+),",
            text,
        )
        assert declarations, "no cooked chunk declarations found"
        return sum(int(v) for v in declarations)

    # Terrain reuses the editable-mesh symbol namespace, which is what lets
    # streaming, precomputed visibility and retained packets work unchanged.
    chunks = sorted(
        set(re.findall(chunk_symbol + r"_(\d+)_faces\[\]", header)), key=int
    )
    assert chunks, "terrain emitted no chunk arrays"
    assert object_symbol + ".geometry=&" + chunk_symbol + "_0;" in header
    # 16 cells of 4 units gives a block of 3 cells, so 6 chunks per axis.
    assert len(chunks) == 36, f"expected 36 chunks, got {len(chunks)}"

    quads = quads_of(header)
    assert quads == cells * cells, f"expected one quad per cell, got {quads}"

    # Chunk-local positions must fit i16 Q12 on every axis.
    for values in re.findall(chunk_symbol + r"_\d+_vertices\[\]\[3\]=\{(.*?)\};", header):
        for value in re.findall(r"-?\d+", values):
            assert -32768 <= int(value) <= 32767, value

    # The heightfield collider is cooked as function-scope storage, because the
    # setup block lives inside initialize_components().
    assert "static constexpr int16_t " + heights_symbol + "[]=" in header
    assert object_symbol + ".collider.heights=" + heights_symbol + ";" in header
    assert object_symbol + ".collider.height_cells[0]=16u" in header
    assert object_symbol + ".collider.height_step=Fixed(16384,Fixed::RAW)" in header
    stored = re.search(heights_symbol + r"\[\]=\{(.*?)\};", header).group(1)
    entries = [int(v) for v in stored.split(",")]
    assert len(entries) == (cells + 1) ** 2, len(entries)
    assert all(v >= 0 for v in entries), "collider heights must sit above the box bottom"

    # Merging collapses the flat rows and must not change the collider.
    terrain["merge"] = 2
    override(ground, "epok::Mesh3DComponent", "terrain", dict(terrain))
    write_scene()
    run("--project", folder, "--build-psx")
    merged_header = (folder / ".epok/build/scene.hh").read_text(encoding="utf-8")
    merged = quads_of(merged_header)
    assert merged < quads, f"merging did not reduce quads: {merged} vs {quads}"
    assert (
        re.search(heights_symbol + r"\[\]=\{(.*?)\};", merged_header).group(1) == stored
    ), "merging changed the collider"

    # A rotated actor cannot carry a heightfield: the grid is indexed from the
    # collider box's world minimum.
    override(ground, "epok::SceneComponent3D", "rotation", [0.0, 35.0, 0.0])
    write_scene()
    output = run("--project", folder, "--build-psx", success=False)
    assert "unrotated" in output, output
    # Turning collision off makes the same rotated terrain legal to draw.
    terrain["collision"] = False
    override(ground, "epok::Mesh3DComponent", "terrain", dict(terrain))
    write_scene()
    run("--project", folder, "--build-psx")

    # A grid past the resident budget is rejected with an actionable message.
    override(ground, "epok::SceneComponent3D", "rotation", [0.0, 0.0, 0.0])
    terrain["collision"] = True
    terrain["merge"] = 0
    override(ground, "epok::Mesh3DComponent", "terrain", dict(terrain))
    write_scene()
    package(
        folder / "assets/Ground.epokasset",
        identity,
        source,
        grid(64, 64, 2.0, lambda i, j: 0.0),
    )
    output = run("--project", folder, "--build-psx", success=False)
    # The grid is well formed, so it loads and the error names the actor that
    # uses it rather than claiming the file is missing.
    assert "Ground: terrain compiles" in output, output
    assert "resident budget" in output, output

    # A single cell taller than one chunk cannot be encoded.
    package(
        folder / "assets/Ground.epokasset",
        identity,
        source,
        grid(4, 4, 4.0, lambda i, j: 40.0 if (i, j) == (2, 2) else 0.0),
    )
    output = run("--project", folder, "--build-psx", success=False)
    # The raised corner is shared by four cells; the first one scanned is
    # named, and what matters is that the actor and the drop are reported.
    assert re.search(r"Ground: cell \d+,\d+ drops 40\.0 units across one cell", output), output

    print("Terrain cooking, collider and budget gates verified.")


main()
