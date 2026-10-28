"""Prepare the Dungeon Complete template from its authored, working game.

Follow asset UUIDs and project C++ includes, keep editable source counterparts,
and strip unrelated SoundFont presets without decoding or changing samples.
The source project is read-only. Output must be a new directory.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re
import shutil
import struct
import epok_documents as documents

UUID = re.compile(rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")


def asset(data):
    assert data[:8] == b"EPOKAS01"
    m, n = struct.unpack_from("<II", data, 8)
    assert len(data) == 16 + m + n
    return json.loads(data[16:16 + m]), data[16 + m:]


def package(meta, payload):
    raw = json.dumps(meta, separators=(",", ":"), ensure_ascii=False).encode()
    return b"EPOKAS01" + struct.pack("<II", len(raw), len(payload)) + raw + payload


def midi_programs(data):
    """Read actual note-on program/bank state across merged SMF tracks."""
    assert data[:4] == b"MThd"
    pos = 8 + int.from_bytes(data[4:8], "big")
    events = []
    while pos < len(data):
        tag, size = data[pos:pos + 4], int.from_bytes(data[pos + 4:pos + 8], "big")
        track = data[pos + 8:pos + 8 + size]
        pos += 8 + size
        if tag != b"MTrk":
            continue
        at = tick = running = 0
        def vlq():
            nonlocal at
            value = 0
            while True:
                byte = track[at]; at += 1
                value = (value << 7) | (byte & 127)
                if byte < 128:
                    return value
        while at < len(track):
            tick += vlq()
            status = track[at]
            if status >= 128:
                at += 1
                if status < 240:
                    running = status
            else:
                status = running
            assert status >= 128
            if status == 255:
                at += 1; length = vlq(); at += length
            elif status in (240, 247):
                length = vlq(); at += length
            else:
                size = 1 if status >> 4 in (12, 13) else 2
                message = track[at:at + size]; at += size
                events.append((tick, len(events), status, message))
    channels = [[0, 0, 0] for _ in range(16)]
    used = set()
    for _, _, status, msg in sorted(events):
        channel, kind = status & 15, status >> 4
        state = channels[channel]
        if kind == 12:
            state[2] = msg[0]
        elif kind == 11 and msg[0] in (0, 32):
            state[0 if msg[0] == 0 else 1] = msg[1]
        elif kind == 9 and msg[1]:
            used.add((128 if channel == 9 else state[0] * 128 + state[1], state[2]))
    return used


def chunks(data):
    at = 0
    while at + 8 <= len(data):
        name, size = struct.unpack_from("<4sI", data, at)
        yield name, data[at + 8:at + 8 + size]
        at += 8 + size
        # The source SF3 omits some optional odd-length padding.
        if size & 1 and data[at:at + 1] == b"\0":
            at += 1
    assert at == len(data)


def chunk(name, data):
    return name + struct.pack("<I", len(data)) + data + bytes(len(data) & 1)


def trim_soundfont(data, programs):
    assert data[:4] == b"RIFF" and data[8:12] == b"sfbk"
    groups = {body[:4]: list(chunks(body[4:])) for tag, body in chunks(data[12:]) if tag == b"LIST"}
    tables = dict(groups[b"pdta"])
    phdr = list(struct.iter_unpack("<20sHHHIII", tables[b"phdr"]))
    inst = list(struct.iter_unpack("<20sH", tables[b"inst"]))
    shdr = list(struct.iter_unpack("<20s5IBbHH", tables[b"shdr"]))
    bags = {prefix: list(struct.iter_unpack("<HH", tables[prefix + b"bag"])) for prefix in (b"p", b"i")}
    gens = {prefix: list(struct.iter_unpack("<HH", tables[prefix + b"gen"])) for prefix in (b"p", b"i")}
    keep_p = [i for i, h in enumerate(phdr[:-1]) if (h[2], h[1]) in programs]
    assert {(phdr[i][2], phdr[i][1]) for i in keep_p} == programs, "MIDI uses a missing SoundFont preset"
    def references(head, selected, prefix, op, field):
        return sorted({value for i in selected for zone in range(head[i][field], head[i + 1][field])
                       for code, value in gens[prefix][bags[prefix][zone][0]:bags[prefix][zone + 1][0]] if code == op})
    keep_i = references(phdr, keep_p, b"p", 41, 3)
    keep_s = set(references(inst, keep_i, b"i", 53, 1))
    pending = list(keep_s)
    while pending:
        sample = shdr[pending.pop()]
        if sample[9] & 15 in (2, 4, 8) and sample[8] not in keep_s:
            keep_s.add(sample[8]); pending.append(sample[8])
    keep_s = sorted(keep_s)
    i_map = {old: new for new, old in enumerate(keep_i)}
    s_map = {old: new for new, old in enumerate(keep_s)}
    def rebuild(head, selected, prefix, field, structural, remap, fmt):
        new_head, new_bag, new_gen, new_mod = [], [], [], bytearray()
        mods = tables[prefix + b"mod"]
        for i in selected:
            h = list(head[i]); h[field] = len(new_bag); new_head.append(h)
            for zone in range(head[i][field], head[i + 1][field]):
                a, b = bags[prefix][zone], bags[prefix][zone + 1]
                new_bag.append((len(new_gen), len(new_mod) // 10))
                new_gen += [(code, remap[value] if code == structural else value) for code, value in gens[prefix][a[0]:b[0]]]
                new_mod += mods[a[1] * 10:b[1] * 10]
        terminal = list(head[-1]); terminal[field] = len(new_bag); new_head.append(terminal)
        new_bag.append((len(new_gen), len(new_mod) // 10)); new_gen.append((0, 0)); new_mod += bytes(10)
        return {b"phdr" if prefix == b"p" else b"inst": b"".join(struct.pack(fmt, *h) for h in new_head),
                prefix + b"bag": b"".join(struct.pack("<HH", *h) for h in new_bag),
                prefix + b"gen": b"".join(struct.pack("<HH", *h) for h in new_gen), prefix + b"mod": bytes(new_mod)}
    tables.update(rebuild(phdr, keep_p, b"p", 3, 41, i_map, "<20sHHHIII"))
    tables.update(rebuild(inst, keep_i, b"i", 1, 53, s_map, "<20sH"))
    original = dict(groups[b"sdta"])[b"smpl"]
    samples, payload = [], bytearray()
    for old in keep_s:
        h = list(shdr[old]); assert not h[9] & 0x8000
        payload += bytes(len(payload) & 1)
        if h[9] & 16:
            source = original[h[1]:h[2]]
            h[1:3] = [len(payload), len(payload) + len(source)]
        else:
            source = original[h[1] * 2:h[2] * 2]
            delta = len(payload) // 2 - h[1]
            h[1:5] = [value + delta for value in h[1:5]]
        payload += source
        if not h[9] & 16:
            payload += bytes(92)
        if h[9] & 15 in (2, 4, 8):
            h[8] = s_map[h[8]]
        else:
            h[8] = 0
        samples.append(h)
    samples.append([b"EOS", 0, 0, 0, 0, 0, 0, 0, 0, 0])
    tables[b"shdr"] = b"".join(struct.pack("<20s5IBbHH", *h) for h in samples)
    groups[b"sdta"] = [(tag, bytes(payload) if tag == b"smpl" else body) for tag, body in groups[b"sdta"]]
    groups[b"pdta"] = [(tag, tables[tag]) for tag, _ in groups[b"pdta"]]
    result = chunk(b"RIFF", b"sfbk" + b"".join(chunk(b"LIST", group + b"".join(chunk(tag, body) for tag, body in rows)) for group, rows in groups.items()))
    return result, {"presets": len(keep_p), "instruments": len(keep_i), "samples": len(keep_s), "before": len(data), "after": len(result)}


def path_name(path):
    return (path.replace("assets/CombatDemo/resources/Character/Aqua/", "assets/Characters/Player/")
            .replace("assets/CombatDemo/resources/Character/Denise/Character.imported/", "assets/Characters/Player/Animation/")
            .replace("assets/CombatDemo/resources/Character/Brote/", "assets/Characters/Robot/")
            .replace("assets/CombatDemo/resources/Character/Servbot/ServbotEM00.imported/", "assets/Characters/Robot/Animation/")
            .replace("assets/CombatDemo/Textures/", "assets/UI/Combat/")
            .replace("assets/Audio/Dungeon/", "assets/Audio/SoundEffects/")
            .replace("assets/AudioLibraries/", "assets/Audio/Music/")
            .replace("assets/Sounds/BGM/", "assets/Audio/Music/"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    source, output = args.source.resolve(strict=True), args.output.resolve()
    assert not output.is_relative_to(source), "Output must stay outside the source project"
    assert not output.exists(), "Output must be new; existing projects are never overwritten"
    index = {}
    for file in (source / "assets").rglob("*.epokasset"):
        meta, payload = asset(file.read_bytes())
        assert meta["id"] not in index, f"Duplicate asset identity: {meta['id']}"
        index[meta["id"]] = (file, meta, payload)
    roots = [source / "assets/scenes/DungeonGame.epokmap", source / "assets/PocketStation/AsteroidCatcher.epokasset"]
    pending = [x.decode().lower() for f in roots for x in UUID.findall(f.read_bytes())]
    used = set()
    while pending:
        identity = pending.pop()
        if identity in used or identity not in index:
            continue
        used.add(identity)
        file, meta, payload = index[identity]
        pending += [x.decode().lower() for x in UUID.findall(file.read_bytes())]
    output.mkdir(parents=True)
    def write(path, data):
        target = output / path; target.parent.mkdir(parents=True, exist_ok=True); target.write_bytes(data)
    bank_report = None
    midi = (source / "assets/Sounds/BGM/main-gate-3-.mid").read_bytes()
    for identity in sorted(used):
        file, meta, payload = index[identity]
        rel = file.relative_to(source).as_posix()
        raw_path = meta["source"]
        raw = source / raw_path
        if meta["kind"] == "SoundBank":
            payload, bank_report = trim_soundfont(payload, midi_programs(midi))
            meta["source_hash"] = hashlib.sha256(payload).hexdigest()
            meta["settings"]["options"]["provenance"] = "FluidR3Mono_GM (MIT), Dungeon sustained-sample edits; only this song's reachable presets retained. See Credits/LICENSE-fluidr3mono.txt."
        if raw.is_file() and raw != file:
            write(path_name(raw_path), payload if meta["kind"] == "SoundBank" else raw.read_bytes())
        meta["source"] = path_name(raw_path)
        write(path_name(rel), package(meta, payload))
    # Native class roots plus every local include/translation unit they reach.
    scripts = source / "assets/scripts"
    scene_bytes = roots[0].read_bytes()
    scene_ids = {x.decode().lower() for x in UUID.findall(scene_bytes)}
    pending_files = []
    for file in scripts.rglob("*.hpp"):
        ids = re.findall(r'EPOK_CLASS\([^\n]*Id="([^"]+)"', file.read_text())
        if any(identity in scene_ids for identity in ids):
            pending_files.append(file)
    native = set()
    while pending_files:
        file = pending_files.pop()
        if file in native:
            continue
        native.add(file)
        cpp = file.with_suffix(".cpp")
        if cpp.is_file() and cpp not in native:
            pending_files.append(cpp)
        for name in re.findall(r'^\s*#include\s+"([^"]+)"', file.read_text(), re.M):
            candidate = file.parent / name
            if candidate.is_file() and candidate not in native:
                pending_files.append(candidate)
    for file in sorted(native):
        write(file.relative_to(source).as_posix(), file.read_bytes())
    write("assets/scenes/DungeonGame.epokmap", scene_bytes)
    write("ProjectSettings/Maps.epoksettings", b"scenes: []\n")
    for file in [source / "assets/AudioLibraries/LICENSE-fluidr3mono.txt", source / "assets/UI/Fonts/LICENSE-PressStart2P.txt"]:
        if file.is_file():
            write("Credits/" + file.name, file.read_bytes())
    for file in (source / "Tools/AudioSources/Kenney").rglob("*.txt"):
        write("Credits/Kenney/" + file.relative_to(source / "Tools/AudioSources/Kenney").as_posix(), file.read_bytes())
    manifest = documents.loads(next(source.glob("*.epokproject")).read_text())
    manifest["name"] = "Dungeon Complete"
    manifest["play"]["target"] = "embedded"
    manifest["play"]["data"] = "host"
    manifest["default_sound_bank"] = next(identity for identity in used if index[identity][1]["kind"] == "SoundBank")
    # Keep the proven resident scene / streamed geometry configuration.
    manifest["scene_storage"] = {"in_executable": ["assets/scenes/DungeonGame.epokmap"]}
    write("DungeonComplete.epokproject", documents.dumps(manifest).encode())
    report = {"source_asset_files": len(index), "retained_assets": len(used), "native_files": len(native),
              "bank": bank_report, "source_asset_bytes": sum(f.stat().st_size for f in (source / "assets").rglob("*") if f.is_file()),
              "template_bytes": sum(f.stat().st_size for f in output.rglob("*") if f.is_file()),
              "included": sorted(f.relative_to(output).as_posix() for f in output.rglob("*") if f.is_file()),
              "excluded_assets": sorted(f.relative_to(source).as_posix() for identity, (f, _, _) in index.items() if identity not in used)}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2))
    print(json.dumps({key: value for key, value in report.items() if key not in ("included", "excluded_assets")}, indent=2))


if __name__ == "__main__":
    main()
