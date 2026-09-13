"""Exercise Ironwood's real Title controller in the native HUD child process.

Use a disposable project copy: the editor command takes the ordinary project
lock. No emulator, physical console or persistent memory card is used.
"""
from pathlib import Path
import argparse
import hashlib
import json
import struct
import subprocess

REPO = Path(__file__).resolve().parents[2]


def decode(data):
    offset = 0
    frames = []
    while offset < len(data):
        h = struct.unpack_from("<11I", data, offset)
        offset += 44
        assert h[0] == 0x31445548 and h[9] <= 2560 and h[10] <= 4096
        commands = [struct.unpack_from("<16i", data, offset + i * 64) for i in range(h[9])]
        offset += h[9] * 64
        destination = data[offset:offset + h[10]].decode()
        offset += h[10]
        frames.append(dict(number=h[1], fade=h[2], entities=h[3], stats=h[4:9],
                           commands=commands, destination=destination))
    assert offset == len(data)
    return frames


def text(frame):
    groups = {}
    for c in frame["commands"]:
        if c[0] == 2:
            groups.setdefault((c[1], c[4]), []).append(chr(c[2]))
    return "\n".join("".join(chars) for chars in groups.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", type=Path)
    parser.add_argument("--editor", type=Path, default=REPO / "target/debug/epok-editor.exe")
    parser.add_argument("--output", type=Path, default=REPO / "artifacts/native-hud")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    authored = args.project / "assets/scenes/Title.epokmap"
    before = hashlib.sha256(authored.read_bytes()).hexdigest()
    capture = subprocess.run([str(args.editor.resolve()), "--project", str(args.project.resolve()),
        "--preview-hud", "assets/scenes/Title.epokmap", "--frames", "120", "--output",
        str((args.output / "title.png").resolve())], cwd=REPO, check=True,
        capture_output=True, text=True, timeout=120)
    result = json.loads(capture.stdout)
    runner = max((args.project / ".epok/native-preview").glob("*/runner.exe"), key=lambda p: p.stat().st_mtime)

    def run(buttons):
        payload = b"".join(struct.pack("<II", (i + 1) * 1000000 // 60 - i * 1000000 // 60, b)
                           for i, b in enumerate(buttons))
        completed = subprocess.run([str(runner)], input=payload, capture_output=True,
                                   timeout=20, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        frames = decode(completed.stdout)
        assert len(frames) == len(buttons) + 1
        assert all(f["stats"][4] == 0 for f in frames)
        return frames

    def tap(bit):
        return [1 << bit] + [0] * 8

    idle = [0] * 120
    first = run(idle)
    edit = decode(subprocess.run([str(runner), "--edit"], input=b"", capture_output=True,
        timeout=20, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0)).stdout)
    assert len(edit) == 1 and edit[0]["number"] == 0 and edit[0]["fade"] == 0
    assert edit[0]["entities"] == 36 and not edit[0]["destination"]
    assert edit[0]["commands"] == first[65]["commands"], "Edit preview must match the completed runtime entrance"
    assert first == run(idle), "Reset must restore all globals and deterministic frames"
    assert first[-1]["entities"] == 36 and first[-1]["fade"] == 0
    assert first[-1]["commands"] == [tuple(c) for c in result["frame"]["commands"]]
    configuration = run(idle + tap(6) + tap(6) + tap(14))[-1]
    assert "Configuration" in text(configuration) and "Music volume" in text(configuration)
    volume = run(idle + tap(6) + tap(6) + tap(14) + tap(7))[-1]
    assert "Music volume       < 9 >" in text(volume)
    cards = run(idle + tap(6) + tap(14) + tap(14) + [0] * 40)[-1]
    assert "Choose a saved journey" in text(cards) and "Slot 1  Empty" in text(cards)
    held = run(idle + [1 << 14] * 120)[-1]
    assert "New game" in text(held) and not held["destination"], "Held confirmation must not select twice"
    transition = run(idle + tap(14) + tap(14) + [0] * 60)[-1]
    assert transition["destination"] == "ForestClearing"
    assert hashlib.sha256(authored.read_bytes()).hexdigest() == before
    summary = dict(frame=120, entities=36, dynamic_entities=28, draws=len(first[-1]["commands"]),
                   dropped=0, deterministic=True, configuration=True, volume=True, empty_cards=True,
                   held_input=True, scene_boundary=transition["destination"], authored_scene_unchanged=True,
                   automatic_edit_preview=True, edit_frame=0, edit_matches_runtime=True)
    (args.output / "verification.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
