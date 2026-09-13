"""Exercise shipped MIDI/SoundBank CLI transactions without a console or synth service."""
from pathlib import Path
import hashlib
import json
import struct
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)


def main():
    project = ROOT / ".epok" / f"midi-authoring-{time.time_ns()}"
    assert project.resolve().is_relative_to((ROOT / ".epok").resolve())
    log = ROOT / "artifacts/audio-phase-c-cli-details.log"

    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True,
                                text=True, timeout=180, creationflags=FLAGS)
        with log.open("a", encoding="utf-8") as out:
            out.write(f"{args}\nexit={result.returncode}\n{result.stdout}{result.stderr}\n")
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        return result.stdout

    def package(path):
        data = path.read_bytes()
        assert data[:8] == b"EPOKAS01"
        length, source_size = struct.unpack_from("<II", data, 8)
        meta = json.loads(data[16:16 + length])
        source = data[16 + length:]
        assert len(source) == source_size
        assert hashlib.sha256(source).hexdigest() == meta["source_hash"]
        return meta, source

    run("--create-project", project, "--template", "basic")
    run("--project", project, "--create-starter-bank", "assets/Retro.epokasset")
    bank, _ = package(project / "assets/Retro.epokasset")
    assert bank["settings"]["options"]["provenance"].startswith("Epok Retro Triangle")
    run("--project", project, "--default-sound-bank", bank["id"])
    midi = bytes.fromhex("4d546864000000060000000100604d54726b0000000c00903c6460803c0000ff2f00")
    (project / "assets/song.mid").write_bytes(midi)
    run("--project", project, "--import-audio", "assets/song.mid", "--sound-bank", "default",
        "--voice-limit", "12", "--sequence-loop", "whole", "--audio-role", "ambience", "--load-mode", "resident")
    song = project / "assets/song.epokasset"
    meta, source = package(song)
    assert source == midi and meta["kind"] == "MusicSequence"
    assert meta["settings"]["options"]["role"] == "Ambience"
    assert meta["settings"]["options"]["load_mode"] == "Resident"
    assert meta["settings"]["options"]["voice_limit"] == 12
    identity = meta["id"]
    moved = project / "assets/moved.epokasset"
    song.rename(moved)
    (project / "assets/song.mid").unlink()
    run("--project", project, "--reimport-asset", "assets/moved.epokasset", "--snapshot", "--voice-limit", "auto")
    meta, source = package(moved)
    assert meta["id"] == identity and source == midi
    assert meta["settings"]["options"]["voice_limit"] is None
    before = moved.read_bytes()
    (project / "assets/broken.mid").write_bytes(midi[:-1])
    run("--project", project, "--reimport-asset", "assets/moved.epokasset", "--import-audio", "assets/broken.mid", success=False)
    assert moved.read_bytes() == before
    run("--project", project, "--create-starter-bank", "assets/Retro.epokasset", success=False)
    records = json.loads(run("--project", project, "--scan-assets"))["assets"]
    assert all(r["usable"] for r in records)
    assert not (project / ".epok/imported").exists(), "Authoring must not request a target cook"
    print(f"MIDI CLI: owned bank, default bank, role/residency, source snapshot, UUID move/reimport and failed transaction passed. Project: {project}")


if __name__ == "__main__":
    main()
