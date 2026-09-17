"""Generate Epok's original character animation fixtures (no third-party assets, no DCC tool).

Writes ASCII FBX 7400 directly plus a small PNG atlas, using only the Python standard
library. Run:

    python3 tests/fixtures/create_textured_character_fixture.py

Outputs (checked in; regenerating is deterministic and byte-stable):

    resources/models/EpokSeamCharacter.fbx   4 bones, 32 control points, 20 polygons,
                                             two materials, a deliberate per-corner
                                             texture-coordinate seam, 3 clips.
    resources/models/EpokManySequences.fbx   same rig and mesh, 30 clips.
    resources/models/EpokSeamAtlas.png       64x32 orientation atlas.

Optional flags:

    --sidecar <path>   also write a JSON description of the expected per-polygon
                       control points and texture coordinates (for importer tests).
    --clip-count <n>   override the clip count of the many-sequence file (used to
                       produce reduced variants while the clip cap is being raised).
    --out <dir>        write somewhere other than resources/models.

Conventions match the existing rigid fixture: meters, Y up, front along -Z (the axis
triple ufbx asks for), 30 fps baked keys, one bone per control point (rigid skinning),
clip names are the animation stack names.

Texture coordinate convention: FBX stores v with origin at the bottom-left, while PNG
row 0 is the top. The sidecar therefore reports both `uv_fbx` and `uv_epok`, where
`uv_epok = [u, 1 - v]`.
"""

import argparse
import json
import math
import struct
import zlib
from pathlib import Path

# FBX stores times in these ticks; 46186158000 is divisible by 30, so 30 fps sample
# times land on exact integers and baked keys coincide with the importer's samples.
TICKS_PER_SECOND = 46186158000
FPS = 30
TICKS_PER_FRAME = TICKS_PER_SECOND // FPS

# ---------------------------------------------------------------------------
# Rig
# ---------------------------------------------------------------------------
# (name, parent, local translation). Emission order below is deliberately not the
# hierarchy order, so importers cannot rely on declaration order.
BONES = [
    ("Root", None, (0.0, 0.0, 0.0)),
    ("Spine", "Root", (0.0, 0.75, 0.0)),
    ("ArmL", "Spine", (0.4, 0.05, 0.0)),
    ("ArmR", "Spine", (-0.4, 0.05, 0.0)),
]
BONE_EMIT_ORDER = ["Root", "ArmL", "Spine", "ArmR"]


def bone_world():
    out = {}
    for name, parent, local in BONES:
        base = out[parent] if parent else (0.0, 0.0, 0.0)
        out[name] = tuple(base[i] + local[i] for i in range(3))
    return out


WORLD = bone_world()

# ---------------------------------------------------------------------------
# Mesh
# ---------------------------------------------------------------------------
# Unit-square texture tiles. Corner order follows the polygon corner order.
FULL = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
SEAM = [(1.0, 0.0), (0.5, 0.0), (0.5, 0.5), (1.0, 0.5)]
TL = [(0.0, 0.5), (0.5, 0.5), (0.5, 1.0), (0.0, 1.0)]
TR = [(0.5, 0.5), (1.0, 0.5), (1.0, 1.0), (0.5, 1.0)]
BL = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)]
BR = [(0.5, 0.0), (1.0, 0.0), (1.0, 0.5), (0.5, 0.5)]

BOX_SIGNS = [
    (-1, -1, -1),
    (1, -1, -1),
    (1, 1, -1),
    (-1, 1, -1),
    (-1, -1, 1),
    (1, -1, 1),
    (1, 1, 1),
    (-1, 1, 1),
]
# front(-Z), back(+Z), bottom(-Y), top(+Y), left(-X), right(+X); wound
# counter-clockwise seen from outside.
BOX_FACES = [
    (0, 3, 2, 1),
    (4, 5, 6, 7),
    (0, 1, 5, 4),
    (3, 7, 6, 2),
    (0, 4, 7, 3),
    (1, 2, 6, 5),
]


class MeshBuilder:
    def __init__(self):
        self.points = []          # control point positions
        self.point_bone = []      # bone name per control point
        self.polygons = []        # list of dicts: corners, uvs, material, label
        self.materials = ["Body", "Trim"]

    def box(self, center, size, bone, uv_tiles, materials, label):
        base = len(self.points)
        for sign in BOX_SIGNS:
            self.points.append(
                tuple(center[i] + sign[i] * size[i] / 2.0 for i in range(3))
            )
            self.point_bone.append(bone)
        for face, tile, material in zip(BOX_FACES, uv_tiles, materials):
            self.polygons.append(
                {
                    "corners": [base + i for i in face],
                    "uvs": list(tile),
                    "material": material,
                    "label": f"{label}.{len(self.polygons)}",
                }
            )

    def ngon(self, positions, bone, uvs, material, label):
        base = len(self.points)
        for p in positions:
            self.points.append(p)
            self.point_bone.append(bone)
        self.polygons.append(
            {
                "corners": [base + i for i in range(len(positions))],
                "uvs": list(uvs),
                "material": material,
                "label": label,
            }
        )


def build_mesh():
    m = MeshBuilder()
    # Torso. The front face uses the whole atlas so every unit-square corner is used;
    # the left face re-uses control point 0 with a different corner, creating the seam.
    m.box(
        (0.0, 0.75, 0.0),
        (0.5, 0.6, 0.3),
        "Spine",
        [FULL, TR, BL, TL, SEAM, BR],
        [0, 0, 0, 0, 0, 0],
        "torso",
    )
    m.box(
        (0.4, 0.8, 0.0),
        (0.16, 0.5, 0.16),
        "ArmL",
        [TL, TR, BL, BR, TL, TR],
        [1] * 6,
        "armL",
    )
    m.box(
        (-0.4, 0.8, 0.0),
        (0.16, 0.5, 0.16),
        "ArmR",
        [TR, TL, BR, BL, TR, TL],
        [1] * 6,
        "armR",
    )
    # A 5-gon panel on the root bone, so triangulation of an n-gon is exercised.
    pent_pos = []
    pent_uv = []
    for k in range(5):
        a = math.radians(90.0 + 72.0 * k)
        pent_pos.append((round(0.22 * math.cos(a), 6), round(0.35 + 0.22 * math.sin(a), 6), 0.25))
        pent_uv.append(
            (round(0.5 + 0.45 * math.cos(a), 6), round(0.5 + 0.45 * math.sin(a), 6))
        )
    m.ngon(pent_pos, "Root", pent_uv, 1, "panel")
    # A lone triangle crest, so the already-triangular case is covered too.
    m.ngon(
        [(-0.12, 1.05, 0.0), (0.0, 1.3, 0.0), (0.12, 1.05, 0.0)],
        "Spine",
        [(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)],
        0,
        "crest",
    )
    return m


# ---------------------------------------------------------------------------
# Animation
# ---------------------------------------------------------------------------
def seam_clips():
    """Clip00 rotates the child bones, Clip01 translates the root, Clip02 is one sample."""
    clips = []

    keys = {}
    frames = FPS + 1  # ~1 second
    for bone, axis, amp, phase in [
        ("ArmL", "Z", 35.0, 0.0),
        ("ArmR", "Z", -35.0, 0.0),
        ("Spine", "Y", 18.0, math.pi / 2.0),
    ]:
        track = []
        for f in range(frames):
            t = 2.0 * math.pi * f / FPS
            track.append((f, round(amp * math.sin(t + phase), 6)))
        keys[(bone, "Lcl Rotation", axis)] = track
    clips.append({"name": "Clip00", "frames": frames - 1, "keys": keys})

    keys = {}
    frames = FPS + 1
    for axis, amp, freq in [("X", 0.9, 1.0), ("Y", 0.35, 2.0), ("Z", 0.6, 1.0)]:
        track = []
        for f in range(frames):
            t = 2.0 * math.pi * freq * f / FPS
            track.append((f, round(amp * math.sin(t), 6)))
        keys[("Root", "Lcl Translation", axis)] = track
    clips.append({"name": "Clip01", "frames": frames - 1, "keys": keys})

    # Single-sample clip: local start == local stop, one key per animated channel.
    # The importer accepts a zero duration (frames = ceil(0 * 30) + 1 = 1).
    clips.append(
        {
            "name": "Clip02",
            "frames": 0,
            "keys": {
                ("Spine", "Lcl Rotation", "Y"): [(0, 45.0)],
                ("ArmL", "Lcl Rotation", "Z"): [(0, -60.0)],
            },
        }
    )
    return clips


def sequence_clips(count):
    """`count` short clips; clip k drives the left arm to k * 10 degrees."""
    clips = []
    for k in range(count):
        frames = FPS // 2 + 1  # 0.5 seconds
        target = float(k * 10)
        constant = k in (0, 7, 13)
        if constant:
            arm = [(0, target)]
        else:
            arm = [
                (f, round(target * f / (frames - 1), 6)) for f in range(frames)
            ]
        clips.append(
            {
                "name": f"Clip{k:02}",
                "frames": 0 if constant and k == 0 else frames - 1,
                "keys": {
                    ("ArmL", "Lcl Rotation", "Z"): arm,
                    # A deliberately constant companion track on every clip.
                    ("Spine", "Lcl Rotation", "Y"): [(0, float(k * 3 % 45))],
                },
            }
        )
    return clips


# ---------------------------------------------------------------------------
# ASCII FBX writer
# ---------------------------------------------------------------------------
def num(v):
    if isinstance(v, int):
        return str(v)
    if v == int(v) and abs(v) < 1e15:
        return str(int(v))
    return repr(round(float(v), 6))


def array(name, values, indent):
    pad = "\t" * indent
    body = ",".join(num(v) for v in values)
    chunks = []
    line = ""
    for piece in body.split(","):
        if line and len(line) + len(piece) + 1 > 100:
            chunks.append(line)
            line = piece
        else:
            line = f"{line},{piece}" if line else piece
    if line:
        chunks.append(line)
    inner = (",\n" + pad + "\t\t").join(chunks) if chunks else ""
    return (
        f"{pad}{name}: *{len(values)} {{\n{pad}\ta: {inner}\n{pad}}}\n"
        if values
        else f"{pad}{name}: *0 {{\n{pad}\ta: \n{pad}}}\n"
    )


class Ids:
    def __init__(self):
        self.next = 1000000

    def take(self):
        self.next += 1
        return self.next


def write_fbx(path, mesh, clips):
    ids = Ids()
    out = []
    connections = []
    counts = {}

    def obj(kind, name, sub, body):
        counts[kind] = counts.get(kind, 0) + 1
        oid = ids.take()
        out.append(f'\t{kind}: {oid}, "{kind}::{name}", "{sub}" {{\n{body}\t}}\n')
        return oid

    # --- bone models -------------------------------------------------------
    bone_local = {n: t for n, _, t in BONES}
    bone_parent = {n: p for n, p, _ in BONES}
    bone_id = {}
    for name in BONE_EMIT_ORDER:
        t = bone_local[name]
        body = (
            "\t\tVersion: 232\n"
            "\t\tProperties70:  {\n"
            '\t\t\tP: "RotationOrder", "enum", "", "",0\n'
            '\t\t\tP: "InheritType", "enum", "", "",1\n'
            '\t\t\tP: "ScalingMax", "Vector3D", "Vector", "",0,0,0\n'
            '\t\t\tP: "DefaultAttributeIndex", "int", "Integer", "",0\n'
            f'\t\t\tP: "Lcl Translation", "Lcl Translation", "", "A",{num(t[0])},{num(t[1])},{num(t[2])}\n'
            '\t\t\tP: "Lcl Rotation", "Lcl Rotation", "", "A",0,0,0\n'
            '\t\t\tP: "Lcl Scaling", "Lcl Scaling", "", "A",1,1,1\n'
            "\t\t}\n"
        )
        bone_id[name] = obj("Model", name, "LimbNode", body)
    for name in BONE_EMIT_ORDER:
        parent = bone_parent[name]
        connections.append(("OO", bone_id[name], bone_id[parent] if parent else 0))

    # --- mesh node ---------------------------------------------------------
    mesh_node = obj(
        "Model",
        "CharacterMesh",
        "Mesh",
        "\t\tVersion: 232\n"
        "\t\tProperties70:  {\n"
        '\t\t\tP: "RotationOrder", "enum", "", "",0\n'
        '\t\t\tP: "InheritType", "enum", "", "",1\n'
        '\t\t\tP: "DefaultAttributeIndex", "int", "Integer", "",0\n'
        '\t\t\tP: "Lcl Translation", "Lcl Translation", "", "A",0,0,0\n'
        '\t\t\tP: "Lcl Rotation", "Lcl Rotation", "", "A",0,0,0\n'
        '\t\t\tP: "Lcl Scaling", "Lcl Scaling", "", "A",1,1,1\n'
        "\t\t}\n"
        ,
    )
    connections.append(("OO", mesh_node, 0))

    # --- geometry ----------------------------------------------------------
    vertices = [c for p in mesh.points for c in p]
    polygon_index = []
    for poly in mesh.polygons:
        corners = poly["corners"]
        polygon_index.extend(corners[:-1])
        polygon_index.append(-corners[-1] - 1)  # last corner of a polygon is encoded

    uv_table = []
    uv_lookup = {}
    uv_index = []
    for poly in mesh.polygons:
        for uv in poly["uvs"]:
            key = (round(uv[0], 6), round(uv[1], 6))
            if key not in uv_lookup:
                uv_lookup[key] = len(uv_table)
                uv_table.append(key)
            uv_index.append(uv_lookup[key])
    uv_values = [c for uv in uv_table for c in uv]
    face_material = [poly["material"] for poly in mesh.polygons]

    geometry_body = (
        array("Vertices", vertices, 2)
        + array("PolygonVertexIndex", polygon_index, 2)
        + "\t\tGeometryVersion: 124\n"
        + "\t\tLayerElementUV: 0 {\n"
        + "\t\t\tVersion: 101\n"
        + '\t\t\tName: "AtlasUV"\n'
        + '\t\t\tMappingInformationType: "ByPolygonVertex"\n'
        + '\t\t\tReferenceInformationType: "IndexToDirect"\n'
        + array("UV", uv_values, 3)
        + array("UVIndex", uv_index, 3)
        + "\t\t}\n"
        + "\t\tLayerElementMaterial: 0 {\n"
        + "\t\t\tVersion: 101\n"
        + '\t\t\tName: ""\n'
        + '\t\t\tMappingInformationType: "ByPolygon"\n'
        + '\t\t\tReferenceInformationType: "IndexToDirect"\n'
        + array("Materials", face_material, 3)
        + "\t\t}\n"
        + "\t\tLayer: 0 {\n"
        + "\t\t\tVersion: 100\n"
        + "\t\t\tLayerElement:  {\n"
        + '\t\t\t\tType: "LayerElementUV"\n'
        + "\t\t\t\tTypedIndex: 0\n"
        + "\t\t\t}\n"
        + "\t\t\tLayerElement:  {\n"
        + '\t\t\t\tType: "LayerElementMaterial"\n'
        + "\t\t\t\tTypedIndex: 0\n"
        + "\t\t\t}\n"
        + "\t\t}\n"
    )
    geometry = obj("Geometry", "CharacterMesh", "Mesh", geometry_body)
    connections.append(("OO", geometry, mesh_node))

    # --- materials ---------------------------------------------------------
    colors = {"Body": (0.26, 0.48, 0.82), "Trim": (0.85, 0.62, 0.18)}
    for name in mesh.materials:
        c = colors[name]
        body = (
            "\t\tVersion: 102\n"
            '\t\tShadingModel: "phong"\n'
            "\t\tMultiLayer: 0\n"
            "\t\tProperties70:  {\n"
            f'\t\t\tP: "DiffuseColor", "Color", "", "A",{num(c[0])},{num(c[1])},{num(c[2])}\n'
            '\t\t\tP: "SpecularColor", "Color", "", "A",0,0,0\n'
            "\t\t}\n"
        )
        connections.append(("OO", obj("Material", name, "", body), mesh_node))

    # --- skin --------------------------------------------------------------
    skin = obj(
        "Deformer",
        "CharacterSkin",
        "Skin",
        "\t\tVersion: 101\n\t\tLink_DeformAcuracy: 50\n\t\tSkinningType: \"Linear\"\n",
    )
    connections.append(("OO", skin, geometry))
    for name, _, _ in BONES:
        indexes = [i for i, b in enumerate(mesh.point_bone) if b == name]
        w = WORLD[name]
        # Column-major 4x4 with translation in slots 12..14.
        link = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, w[0], w[1], w[2], 1]
        inverse = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -w[0], -w[1], -w[2], 1]
        body = (
            "\t\tVersion: 100\n"
            '\t\tUserData: "", ""\n'
            + array("Indexes", indexes, 2)
            + array("Weights", [1.0] * len(indexes), 2)
            + array("Transform", inverse, 2)
            + array("TransformLink", link, 2)
        )
        cluster = obj("Deformer", f"Cluster_{name}", "Cluster", body)
        connections.append(("OO", cluster, skin))
        connections.append(("OO", bone_id[name], cluster))

    # --- animation ---------------------------------------------------------
    axis_index = {"X": 0, "Y": 1, "Z": 2}
    for clip in clips:
        stop = clip["frames"] * TICKS_PER_FRAME
        stack = obj(
            "AnimationStack",
            clip["name"],
            "",
            "\t\tProperties70:  {\n"
            '\t\t\tP: "LocalStart", "KTime", "Time", "",0\n'
            f'\t\t\tP: "LocalStop", "KTime", "Time", "",{stop}\n'
            '\t\t\tP: "ReferenceStart", "KTime", "Time", "",0\n'
            f'\t\t\tP: "ReferenceStop", "KTime", "Time", "",{stop}\n'
            "\t\t}\n",
        )
        layer = obj("AnimationLayer", "BaseLayer", "", "")
        connections.append(("OO", layer, stack))
        grouped = {}
        for (bone, prop, axis), track in clip["keys"].items():
            grouped.setdefault((bone, prop), {})[axis] = track
        for (bone, prop), axes in sorted(grouped.items()):
            default = (1.0, 1.0, 1.0) if prop == "Lcl Scaling" else (0.0, 0.0, 0.0)
            if prop == "Lcl Translation":
                default = bone_local[bone]
            values = list(default)
            for axis, track in axes.items():
                values[axis_index[axis]] = track[0][1] + (
                    default[axis_index[axis]] if prop == "Lcl Translation" else 0.0
                )
            body = (
                "\t\tProperties70:  {\n"
                f'\t\t\tP: "d|X", "Number", "", "A",{num(values[0])}\n'
                f'\t\t\tP: "d|Y", "Number", "", "A",{num(values[1])}\n'
                f'\t\t\tP: "d|Z", "Number", "", "A",{num(values[2])}\n'
                "\t\t}\n"
            )
            node = obj("AnimationCurveNode", prop.replace("Lcl ", ""), "", body)
            connections.append(("OO", node, layer))
            connections.append(("OP", node, bone_id[bone], prop))
            for axis, track in sorted(axes.items()):
                offset = (
                    default[axis_index[axis]] if prop == "Lcl Translation" else 0.0
                )
                times = [f * TICKS_PER_FRAME for f, _ in track]
                vals = [round(v + offset, 6) for _, v in track]
                curve_body = (
                    f"\t\tDefault: {num(vals[0])}\n"
                    "\t\tKeyVer: 4009\n"
                    + array("KeyTime", times, 2)
                    + array("KeyValueFloat", vals, 2)
                    # 0x2 selects linear interpolation; keys are baked at every
                    # 30 fps sample, so interpolation never changes a sample.
                    + array("KeyAttrFlags", [2], 2)
                    + array("KeyAttrDataFloat", [0, 0, 0, 0], 2)
                    + array("KeyAttrRefCount", [len(track)], 2)
                )
                curve = obj("AnimationCurve", "", "", curve_body)
                connections.append(("OP", curve, node, f"d|{axis}"))

    # --- assemble ----------------------------------------------------------
    definitions = "".join(
        f'\tObjectType: "{k}" {{\n\t\tCount: {v}\n\t}}\n' for k, v in sorted(counts.items())
    )
    text = (
        "; FBX 7.4.0 project file\n"
        "; Generated by tests/fixtures/create_textured_character_fixture.py\n"
        ";\n"
        "FBXHeaderExtension:  {\n"
        "\tFBXHeaderVersion: 1003\n"
        "\tFBXVersion: 7400\n"
        "\tCreationTimeStamp:  {\n"
        "\t\tVersion: 1000\n"
        "\t\tYear: 2026\n\t\tMonth: 1\n\t\tDay: 1\n"
        "\t\tHour: 0\n\t\tMinute: 0\n\t\tSecond: 0\n\t\tMillisecond: 0\n"
        "\t}\n"
        '\tCreator: "Epok fixture generator"\n'
        "}\n"
        'Creator: "Epok fixture generator"\n'
        "GlobalSettings:  {\n"
        "\tVersion: 1000\n"
        "\tProperties70:  {\n"
        '\t\tP: "UpAxis", "int", "Integer", "",1\n'
        '\t\tP: "UpAxisSign", "int", "Integer", "",1\n'
        '\t\tP: "FrontAxis", "int", "Integer", "",2\n'
        '\t\tP: "FrontAxisSign", "int", "Integer", "",-1\n'
        '\t\tP: "CoordAxis", "int", "Integer", "",0\n'
        '\t\tP: "CoordAxisSign", "int", "Integer", "",1\n'
        '\t\tP: "OriginalUpAxis", "int", "Integer", "",1\n'
        '\t\tP: "OriginalUpAxisSign", "int", "Integer", "",1\n'
        '\t\tP: "UnitScaleFactor", "double", "Number", "",100\n'
        '\t\tP: "OriginalUnitScaleFactor", "double", "Number", "",100\n'
        '\t\tP: "TimeMode", "enum", "", "",6\n'
        '\t\tP: "TimeSpanStart", "KTime", "Time", "",0\n'
        f'\t\tP: "TimeSpanStop", "KTime", "Time", "",{TICKS_PER_SECOND}\n'
        '\t\tP: "CustomFrameRate", "double", "Number", "",-1\n'
        "\t}\n"
        "}\n"
        "Definitions:  {\n"
        "\tVersion: 100\n"
        f"\tCount: {sum(counts.values())}\n"
        '\tObjectType: "GlobalSettings" {\n\t\tCount: 1\n\t}\n'
        + definitions
        + "}\n"
        "Objects:  {\n"
        + "".join(out)
        + "}\n"
        "Connections:  {\n"
        + "".join(
            f'\tC: "{c[0]}",{c[1]},{c[2]}'
            + (f',"{c[3]}"' if len(c) > 3 else "")
            + "\n"
            for c in connections
        )
        + "}\n"
        'Takes:  {\n\tCurrent: ""\n}\n'
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return text


# ---------------------------------------------------------------------------
# Atlas
# ---------------------------------------------------------------------------
ATLAS_W, ATLAS_H = 64, 32
# name -> rgb. Kept to 9 distinct colors, all separable after 5-bit quantization.
ATLAS_COLORS = {
    "quadrant_top_left": (216, 64, 48),
    "quadrant_top_right": (48, 160, 216),
    "quadrant_bottom_left": (96, 200, 72),
    "quadrant_bottom_right": (232, 200, 56),
    "top_row": (24, 24, 32),
    "corner_top_left": (255, 255, 255),
    "corner_top_right": (255, 0, 255),
    "corner_bottom_left": (0, 255, 255),
    "corner_bottom_right": (128, 0, 200),
}


def write_png(path):
    rows = []
    for y in range(ATLAS_H):
        row = []
        for x in range(ATLAS_W):
            top = y < ATLAS_H // 2
            left = x < ATLAS_W // 2
            key = "quadrant_" + ("top_" if top else "bottom_") + ("left" if left else "right")
            color = ATLAS_COLORS[key]
            if y == 0:
                color = ATLAS_COLORS["top_row"]
            if (x, y) == (0, 0):
                color = ATLAS_COLORS["corner_top_left"]
            elif (x, y) == (ATLAS_W - 1, 0):
                color = ATLAS_COLORS["corner_top_right"]
            elif (x, y) == (0, ATLAS_H - 1):
                color = ATLAS_COLORS["corner_bottom_left"]
            elif (x, y) == (ATLAS_W - 1, ATLAS_H - 1):
                color = ATLAS_COLORS["corner_bottom_right"]
            row.extend(color)
        rows.append(bytes([0]) + bytes(row))
    raw = b"".join(rows)

    def chunk(tag, data):
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", ATLAS_W, ATLAS_H, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


# ---------------------------------------------------------------------------
# Sidecar
# ---------------------------------------------------------------------------
def sidecar(mesh, clips):
    """Expected per-polygon control points and texture coordinates, in FBX order."""
    polygons = []
    fan = []
    seams = {}
    for i, poly in enumerate(mesh.polygons):
        corners = []
        for cp, uv in zip(poly["corners"], poly["uvs"]):
            corners.append(
                {
                    "control_point": cp,
                    "uv_fbx": [round(uv[0], 6), round(uv[1], 6)],
                    "uv_epok": [round(uv[0], 6), round(1.0 - uv[1], 6)],
                }
            )
            seams.setdefault(cp, set()).add((round(uv[0], 6), round(uv[1], 6)))
        polygons.append(
            {
                "polygon": i,
                "label": poly["label"],
                "material": poly["material"],
                "material_name": mesh.materials[poly["material"]],
                "corners": corners,
            }
        )
        # Fan hint only: the importer uses ufbx's triangulator, which may choose a
        # different diagonal for the 5-gon. Corner order after the importer's
        # [0, 2, 1] winding swap is given as `epok_corner_order`.
        for k in range(1, len(corners) - 1):
            fan.append(
                {
                    "polygon": i,
                    "corner_order": [0, k, k + 1],
                    "epok_corner_order": [0, k + 1, k],
                    "control_points": [
                        corners[0]["control_point"],
                        corners[k + 1]["control_point"],
                        corners[k]["control_point"],
                    ],
                    "uv_epok": [
                        corners[0]["uv_epok"],
                        corners[k + 1]["uv_epok"],
                        corners[k]["uv_epok"],
                    ],
                }
            )
    seam_points = {
        str(cp): sorted([list(v) for v in uvs])
        for cp, uvs in sorted(seams.items())
        if len(uvs) > 1
    }
    return {
        "note": (
            "Expected control points and texture coordinates for EpokSeamCharacter.fbx. "
            "uv_fbx is as stored (origin bottom-left); uv_epok = [u, 1 - v]. "
            "`polygons` is authoritative. `fan_triangles` is only a hint: the importer "
            "triangulates with ufbx and applies the [0, 2, 1] winding swap (already "
            "reflected in epok_corner_order). Quads match the fan up to a cyclic "
            "rotation, but ufbx ear-clips the 5-gon differently (it produced "
            "24/28/25, 25/27/26, 25/28/27 for polygon 18). Assert per-corner "
            "coordinates by control point, not by triangle order, for n-gons."
        ),
        "control_point_count": len(mesh.points),
        "polygon_count": len(mesh.polygons),
        "triangle_count": sum(len(p["corners"]) - 2 for p in mesh.polygons),
        "materials": mesh.materials,
        "bones": [b[0] for b in BONES],
        "control_point_bone": mesh.point_bone,
        "control_point_position": [list(p) for p in mesh.points],
        "seam_control_points": seam_points,
        "atlas": {"width": ATLAS_W, "height": ATLAS_H, "colors": ATLAS_COLORS},
        "clips": [{"name": c["name"], "frames": c["frames"] + 1} for c in clips],
        "polygons": polygons,
        "fan_triangles": fan,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None)
    ap.add_argument("--sidecar", default=None)
    ap.add_argument("--clip-count", type=int, default=30)
    args = ap.parse_args()

    out = Path(args.out) if args.out else Path(__file__).resolve().parents[2] / "resources/models"
    mesh = build_mesh()
    clips = seam_clips()
    write_fbx(out / "EpokSeamCharacter.fbx", mesh, clips)
    write_fbx(out / "EpokManySequences.fbx", mesh, sequence_clips(args.clip_count))
    write_png(out / "EpokSeamAtlas.png")
    if args.sidecar:
        Path(args.sidecar).write_text(
            json.dumps(sidecar(mesh, clips), indent=1), encoding="utf-8"
        )
    print(
        f"control points {len(mesh.points)}, polygons {len(mesh.polygons)}, "
        f"triangles {sum(len(p['corners']) - 2 for p in mesh.polygons)}, "
        f"clips {len(clips)} / {args.clip_count}"
    )
    print("Wrote", out)


if __name__ == "__main__":
    main()
