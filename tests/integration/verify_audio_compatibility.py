"""Owned Sony/converted SEQ fixtures through real CLI import, selection and source recovery."""
from pathlib import Path
import json
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents


def sony(song_id, key):
    events = bytes([0, 0x90, key, 100, 96, 0x80, key, 0, 0, 0xff, 0x2f, 0])
    return struct.pack(">HH", song_id, 96) + bytes.fromhex("07a1200402") + struct.pack(">I", len(events)) + events


def converted_seq(key):
    events = bytes([0x90, key, 100, 96, 0x80, key, 0, 0, 0xff, 0x2f, 0])
    size = (12 + len(events) + 3) & ~3
    return (struct.pack("<IIHBB", size, 500000, 96, 4, 2) + events).ljust(size, b"\0")


def vab():
    data = bytearray(0xc30)
    data[:4] = b"pBAV"
    struct.pack_into("<I", data, 4, 7)
    struct.pack_into("<I", data, 12, len(data))
    struct.pack_into("<HHH", data, 18, 1, 2, 1)
    data[24:26] = bytes([127, 64])
    data[32:37] = bytes([2, 127, 0, 0, 64])
    for at in (0x820, 0x840):
        data[at+2:at+5] = bytes([127, 64, 60])
        data[at+7] = 127
        data[at+12:at+14] = bytes([2, 2])
        data[at+22] = 1
    data[0xa22] = 2
    data[0xc20:0xc23] = bytes([12, 1, 0x71])
    return bytes(data)


def package(path):
    data = path.read_bytes()
    assert data[:8] == b"EPOKAS01"
    meta_size, source_size = struct.unpack_from("<II", data, 8)
    meta = json.loads(data[16:16+meta_size])
    source = data[16+meta_size:]
    assert len(source) == source_size
    return meta, source


def main():
    project = ROOT / ".epok" / f"compatibility-{time.time_ns()}"
    evidence = ROOT / "artifacts" / f"audio-phase-e-cli-{time.time_ns()}"
    evidence.mkdir(parents=True)
    def run(*args, error=None):
        cmd = [str(ROOT / "target/debug/epok-editor.exe"), *map(str, args)]
        start = time.perf_counter()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=180,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        with (evidence / "commands.log").open("a", encoding="utf-8") as log:
            log.write(f"{cmd!r}\nexit={result.returncode}; seconds={time.perf_counter()-start:.6f}\n{result.stdout}{result.stderr}\n")
        output = result.stdout + result.stderr
        if error:
            assert result.returncode != 0 and error.lower() in output.lower(), output
        else:
            assert result.returncode == 0, output
        return result.stdout
    def cli(*args, **kwargs):
        return run("--project", project, *args, **kwargs)

    run("--create-project", project, "--template", "sample")
    cli("--create-starter-bank", "assets/Portable.epokasset")
    native = package(project / "assets/Portable.epokasset")[0]["id"]
    # Wrong extension intentionally: the profile comes from the full structure.
    sep = b"pQES\0\0" + sony(7, 60) + sony(19, 64)
    (project / "assets/score.vab").write_bytes(sep)
    catalog = json.loads(cli("--inspect-audio-source", "assets/score.vab"))
    assert catalog["profile"] == "sony-sep-v0" and [s["id"] for s in catalog["songs"]] == [7, 19]
    cli("--import-audio", "assets/score.vab", "--sound-bank", native, error="Choose an existing Sony song ID")
    assert not (project / "assets/score.epokasset").exists()
    cli("--import-audio", "assets/score.vab", "--sound-bank", native, "--song-id", "19")
    score_path = project / "assets/score.epokasset"
    meta, original = package(score_path)
    assert original == sep and meta["settings"]["options"]["source_selection"]["song_id"] == 19
    score_id = meta["id"]
    score_path.rename(project / "assets/moved.epokasset")
    cli("--reimport-asset", "assets/moved.epokasset", "--snapshot")
    assert package(project / "assets/moved.epokasset")[0]["id"] == score_id

    converted = converted_seq(60) + converted_seq(64)
    (project / "assets/converted.seq").write_bytes(converted)
    profile = "converted-seq-le32-v1"
    catalog = json.loads(cli("--inspect-audio-source", "assets/converted.seq", "--sequence-profile", profile))
    assert catalog["profile"] == profile and len(catalog["songs"]) == 2
    cli("--import-audio", "assets/converted.seq", "--sequence-profile", profile, "--song-index", "1", "--sound-bank", native)
    converted_path = project / "assets/converted.epokasset"
    canonical_meta, original = package(converted_path)
    assert original == converted
    assert canonical_meta["settings"]["options"]["source_selection"]["profile"] == profile
    # Exercise the initial experimental metadata spelling only as a migration fixture.
    legacy_meta = json.loads(json.dumps(canonical_meta))
    legacy_meta["settings"]["options"]["source_selection"]["profile"] = "legacy-seqcomb-sepcomb-v1"
    encoded = json.dumps(legacy_meta).encode("utf-8")
    converted_path.write_bytes(b"EPOKAS01" + struct.pack("<II", len(encoded), len(original)) + encoded + original)
    cli("--reimport-asset", "assets/converted.epokasset", "--snapshot")
    migrated, source = package(converted_path)
    assert source == original and migrated["id"] == canonical_meta["id"]
    assert migrated["settings"]["options"]["source_selection"] == canonical_meta["settings"]["options"]["source_selection"]
    old = (project / "assets/converted.epokasset").read_bytes()
    (project / "assets/converted.seq").write_bytes(converted_seq(64) + converted_seq(60))
    cli("--reimport-asset", "assets/converted.epokasset", error="changed or moved")
    assert (project / "assets/converted.epokasset").read_bytes() == old
    cli("--reimport-asset", "assets/converted.epokasset", "--song-index", "0")

    bank_bytes = vab()
    (project / "assets/bank.seq").write_bytes(bank_bytes)
    info = json.loads(cli("--inspect-audio-source", "assets/bank.seq"))
    assert info["profile"] == "sony-vab-v7" and len(info["programs"][0]["tones"]) == 2
    cli("--import-audio", "assets/bank.seq", "--provenance", "Original generated Epok test (MIT)")
    bank_meta, snapshot = package(project / "assets/bank.epokasset")
    assert snapshot == bank_bytes and bank_meta["kind"] == "SoundBank"
    (project / "assets/split.vh").write_bytes(bank_bytes[:0xc20])
    (project / "assets/split.vb").write_bytes(bank_bytes[0xc20:])
    cli("--import-sound-bank", "assets/split.vh", error="combined size")
    cli("--import-sound-bank", "assets/split.vh", "--vb", "assets/split.vb")
    pair_path = project / "assets/split.epokasset"
    pair_meta, pair_bytes = package(pair_path)
    assert pair_bytes == bank_bytes and len(pair_meta["settings"]["options"]["imported"]["parts"]) == 2
    old = pair_path.read_bytes()
    (project / "assets/split.vb").write_bytes(bytes(8))
    cli("--reimport-asset", "assets/split.epokasset", error="combined size")
    assert pair_path.read_bytes() == old
    (project / "assets/split.vb").write_bytes(bytes(16))
    # Zero-filled VB is structurally valid but cannot become a resolved bank.
    cli("--reimport-asset", "assets/split.epokasset")
    assert package(pair_path)[0]["id"] == pair_meta["id"]
    (project / "assets/split.vh").unlink()
    (project / "assets/split.vb").unlink()
    cli("--reimport-asset", "assets/split.epokasset", "--snapshot")

    # An imported Sony bank is authoring-valid, with an explicit playback error.
    cli("--reimport-asset", "assets/moved.epokasset", "--snapshot", "--sound-bank", bank_meta["id"], "--ignore-unsupported")
    scene_path = project / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())
    scene["entities"][0]["audio"] = dict(clip=score_id, volume=0.5)
    documents.write_text(scene_path, json.dumps(scene))
    cli("--build-psx", error="Sony SoundBank playback unavailable")
    (evidence / "report.json").write_text(json.dumps(dict(project=str(project), sony_song=19,
        converted_reselection=True, profile_migration=True, bank_tones=2, pair_snapshot=True, playback_blocker=True), indent=2))
    print(f"PASS structural profiles, explicit songs, guarded reimport, complete bank sources and hard playback blocker; evidence {evidence}")


if __name__ == "__main__":
    main()
