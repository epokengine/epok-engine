"""End-to-end check of full-motion video on the emulated console.

Creates a throwaway project, writes an original 16:9 test clip (an uncompressed
AVI generated here: four colour quadrants, a moving black bar and a stereo
tone), imports it as a Video asset, plays it from a Lua class at scene start,
builds the PS-X EXE and BIN/CUE and runs them in PCSX-Redux. A Lua driver reads
`epok::video_stats` and captures the screen, then checks that:

* the clip streams and plays to its end in real time without decode errors,
* the picture shows the expected colours inside a letterboxed 320x240 frame,
* the game's own display mode comes back afterwards, and
* START skips the clip when it is skippable.

Run from the repository root after `cargo build --locked`:
    python3 tests/integration/verify_fmv.py [--display 640x480|320x240] [--keep DIR]
Requires the MIPS toolchain, psxavenc, mkpsxiso and PCSX-Redux from setup.
Screenshots are written next to the report in the work directory.
"""
import argparse
import json
import math
import os
import platform
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import uuid
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WIDTH, HEIGHT, RATE, FRAMES = 256, 144, 15, 45
SAMPLE_RATE = 44100
QUADRANTS = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]


def editor():
    name = "epok-editor.exe" if platform.system() == "Windows" else "epok-editor"
    path = ROOT / "target/debug" / name
    assert path.is_file(), f"build the editor first: {path}"
    return path


def emulator_paths():
    for candidate in (ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/PCSX-Redux",
                      ROOT / ".tools/redux/pcsx-redux.exe",
                      ROOT / ".tools/redux/pcsx-redux",
                      ROOT / ".tools/linux/redux/pcsx-redux"):
        if candidate.is_file():
            bios = sorted((ROOT / ".tools").rglob("openbios.bin"))
            return candidate, bios[0] if bios else None
    return None, None


def run(*args, cwd=None, timeout=900):
    result = subprocess.run([str(a) for a in args], cwd=cwd, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=timeout)
    assert result.returncode == 0, f"{args[1:4]} failed:\n{result.stdout}\n{result.stderr}"
    return result.stdout


def frame_pixels(index):
    """Bottom-up BGR rows, as uncompressed AVI stores them."""
    bar = (index * 5) % (WIDTH - 16)
    rows = []
    for y in range(HEIGHT - 1, -1, -1):
        row = bytearray()
        for x in range(WIDTH):
            r, g, b = QUADRANTS[(y >= HEIGHT // 2) * 2 + (x >= WIDTH // 2)]
            if bar <= x < bar + 16 and HEIGHT // 2 - 12 <= y < HEIGHT // 2 + 12:
                r = g = b = 0
            row += bytes((b, g, r))
        rows.append(bytes(row))
    return b"".join(rows)


def chunk(fourcc, data):
    return fourcc + struct.pack("<I", len(data)) + data + (b"\0" if len(data) % 2 else b"")


def write_avi(path):
    frame_bytes = WIDTH * HEIGHT * 3
    samples = SAMPLE_RATE // RATE
    audio = []
    for i in range(FRAMES):
        block = bytearray()
        for n in range(samples):
            t = (i * samples + n) / SAMPLE_RATE
            block += struct.pack("<hh", int(8000 * math.sin(2 * math.pi * 440 * t)),
                                 int(6000 * math.sin(2 * math.pi * 660 * t)))
        audio.append(bytes(block))
    avih = struct.pack("<14I", 1000000 // RATE, frame_bytes * RATE, 0, 0x10, FRAMES, 0, 2,
                       frame_bytes, WIDTH, HEIGHT, 0, 0, 0, 0)
    video_header = struct.pack("<4s4sIHHIIIIIIIIhhhh", b"vids", b"DIB ", 0, 0, 0, 0, 1, RATE,
                               0, FRAMES, frame_bytes, 0xFFFFFFFF, 0, 0, 0, WIDTH, HEIGHT)
    bitmap = struct.pack("<IiiHHIIiiII", 40, WIDTH, HEIGHT, 1, 24, 0, frame_bytes, 0, 0, 0, 0)
    audio_header = struct.pack("<4s4sIHHIIIIIIIIhhhh", b"auds", b"\0\0\0\0", 0, 0, 0, 0, 4,
                               SAMPLE_RATE * 4, 0, samples * FRAMES, samples * 4, 0xFFFFFFFF, 4,
                               0, 0, 0, 0)
    wave = struct.pack("<HHIIHH", 1, 2, SAMPLE_RATE, SAMPLE_RATE * 4, 4, 16)
    header = chunk(b"LIST", b"hdrl" + chunk(b"avih", avih)
                   + chunk(b"LIST", b"strl" + chunk(b"strh", video_header) + chunk(b"strf", bitmap))
                   + chunk(b"LIST", b"strl" + chunk(b"strh", audio_header) + chunk(b"strf", wave)))
    movi = bytearray(b"movi")
    index = bytearray()
    for i in range(FRAMES):
        for fourcc, data, flags in ((b"00db", frame_pixels(i), 0x10), (b"01wb", audio[i], 0)):
            index += fourcc + struct.pack("<III", flags, len(movi), len(data))
            movi += chunk(fourcc, data)
    body = b"AVI " + header + chunk(b"LIST", bytes(movi)) + chunk(b"idx1", bytes(index))
    path.write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)


def symbols(elf):
    """Address of each named ELF32 symbol, without a cross-toolchain nm."""
    data = elf.read_bytes()
    shoff, = struct.unpack_from("<I", data, 0x20)
    shentsize, shnum = struct.unpack_from("<HH", data, 0x2E)
    sections = [struct.unpack_from("<IIIIIIIIII", data, shoff + i * shentsize) for i in range(shnum)]
    table = {}
    for section in sections:
        if section[1] != 2:  # SHT_SYMTAB
            continue
        strings = sections[section[6]]
        for offset in range(section[4], section[4] + section[5], 16):
            name, value = struct.unpack_from("<II", data, offset)
            end = data.index(b"\0", strings[4] + name)
            table[data[strings[4] + name:end].decode()] = value
    return table


DRIVER = r"""local ffi = require('ffi')
local out = %(out)s
local stats = ffi.cast('uint32_t*', PCSX.getMemPtr() + %(stats)d)
local pad = PCSX.SIO0.slots[1].pads[1]
local frames, captured = 0, 0
local log = assert(io.open(out .. '/stats.txt', 'w'))
local function capture(name)
  local ss = PCSX.GPU.takeScreenShot()
  local f = assert(io.open(out .. '/' .. name .. '.raw', 'wb'))
  f:write(string.format('%%d %%d %%d\n', ss.width, ss.height, tonumber(ss.bpp)))
  f:write(tostring(ss.data))
  f:close()
end
EPOK_FMV_DRIVER = PCSX.Events.createEventListener('GPU::Vsync', function()
  frames = frames + 1
  local line = tostring(frames)
  for i = 0, 11 do line = line .. ' ' .. tostring(tonumber(stats[i])) end
  log:write(line .. '\n')
  if %(skip)d > 0 and frames == %(skip)d then pad.setOverride(PCSX.CONSTS.PAD.BUTTON.START) end
  if %(skip)d > 0 and frames == %(skip)d + 4 then pad.clearOverride(PCSX.CONSTS.PAD.BUTTON.START) end
  if frames %% 15 == 0 then capture(string.format('f%%04d', frames)) end
  if frames >= %(burst)d and frames < %(burst)d + 24 then capture(string.format('b%%04d', frames)) end
  if frames >= %(frames)d then log:close(); PCSX.quit(0) end
end)
"""


def emulate(build, work, stats, frames, skip=0, burst=0):
    emulator, bios = emulator_paths()
    assert emulator and bios, "PCSX-Redux and openbios.bin must be installed under .tools"
    work.mkdir(parents=True, exist_ok=True)
    (work / "portable").mkdir(exist_ok=True)
    driver = work / "driver.lua"
    driver.write_text(DRIVER % {"out": json.dumps(str(work)), "stats": stats & 0x1FFFFF,
                                "frames": frames, "skip": skip, "burst": burst},
                       encoding="utf-8")
    command = [emulator, "-portable", work / "portable", "-run", "-stdout", "-fastboot",
               "-noupdate", "-interpreter", "-softgpu", "-2mb", "-no-gdb", "-no-ui",
               "-iso", build / "epok.cue", "-loadexe", build / "epok.ps-exe",
               "-bios", bios, "-dofile", driver]
    with (work / "emulator.log").open("w", encoding="utf-8") as log:
        subprocess.run([str(c) for c in command], cwd=work / "portable", stdout=log, stderr=log,
                       stdin=subprocess.DEVNULL, timeout=900)
    rows = []
    for line in (work / "stats.txt").read_text().splitlines():
        v = list(map(int, line.split()))
        rows.append({"vsync": v[0], "state": v[1] & 0xFF, "clip": v[2], "plays": v[3],
                     "shown": v[4], "dropped": v[5], "sectors": v[6], "errors": v[7],
                     "error_code": v[8], "decode_lines": v[9], "bitstream_lines": v[10],
                     "failed_frame": v[11], "failed_status": v[12]})
    return rows


def picture(raw):
    head, data = raw.read_bytes().split(b"\n", 1)
    width, height, bpp = map(int, head.split())
    assert bpp == 0, "the console shows video in 15-bit colour"
    pixels = []
    for i in range(width * height):
        v = data[2 * i] | data[2 * i + 1] << 8
        pixels.append(((v & 31) << 3, (v >> 5 & 31) << 3, (v >> 10 & 31) << 3))
    return width, height, pixels


def png(raw):
    width, height, pixels = picture(raw)
    rows = b"".join(b"\0" + bytes(c for p in pixels[y * width:(y + 1) * width] for c in p)
                    for y in range(height))

    def part(kind, body):
        return (struct.pack(">I", len(body)) + kind + body
                + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF))
    raw.with_suffix(".png").write_bytes(
        b"\x89PNG\r\n\x1a\n" + part(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + part(b"IDAT", zlib.compress(rows)) + part(b"IEND", b""))


def near(actual, expected, tolerance=40):
    return all(abs(a - e) <= tolerance for a, e in zip(actual, expected))


def make_project(work, display):
    project = work / "project"
    run(editor(), "--create-project", project, "--name", "FMV Check", "--template", "basic")
    descriptor = next(project.glob("*.epokproject"))
    text = descriptor.read_text(encoding="utf-8")
    width, height = display.split("x")
    text = text.replace("  data: executable\n", "  data: disc\n")
    text = re.sub(r"\n  height: \d+\n", f"\n  height: {height}\n", text)
    text = re.sub(r"\n  width: \d+\n", f"\n  width: {width}\n", text)
    descriptor.write_text(text, encoding="utf-8")
    write_avi(project / "assets/intro.avi")
    imported = run(editor(), "--project", project, "--import-video", "assets/intro.avi")
    video = re.search(r"Imported Video: ([0-9a-f-]{36})", imported)[1]
    report = json.loads(imported.strip().splitlines()[-1])
    run(editor(), "--project", project, "--new-lua-class", "IntroPlayer")
    (project / "assets/scripts/IntroPlayer.lua").write_text(
        "---@class IntroPlayer : epok.ActorComponent\n"
        "local IntroPlayer = epok.ActorComponent:extend()\n\n"
        "IntroPlayer.video = epok.AssetRef(\"Video\")\n"
        "IntroPlayer.skippable = true\n\n"
        "function IntroPlayer:begin_play()\n"
        "    epok.play_video(self.video, self.skippable)\n"
        "end\n\n"
        "return IntroPlayer\n", encoding="utf-8")
    classes = (project / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8")
    class_id = re.search(r"IntroPlayer\.lua: ([0-9a-f-]{36})", classes)[1]
    scene = project / "assets/scenes/Main.epokmap"
    text = scene.read_text(encoding="utf-8")
    marker = "  name: Main Camera"
    component = (f"  - class:\n      class_id: {class_id}\n      name: IntroPlayer\n"
                 f"    id: {uuid.uuid4()}\n    name: IntroPlayer\n    overrides:\n    - video\n"
                 f"    properties:\n      video: {video}\n")
    head, tail = text.split(marker, 1)
    head, identity = head.rsplit("  id: ", 1)
    scene.write_text(head + component + "  id: " + identity + marker + tail, encoding="utf-8")
    run(editor(), "--project", project, "--build-psx", timeout=1800)
    return project, report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--display", default="640x480", choices=["640x480", "320x240"])
    parser.add_argument("--keep", type=Path)
    args = parser.parse_args()
    work = args.keep or Path(tempfile.mkdtemp(prefix="epok-fmv-"))
    work.mkdir(parents=True, exist_ok=True)
    project, report = make_project(work, args.display)
    info = report["info"]
    # psxavenc keeps all but the last frame or two of the source.
    assert (info["width"], info["height"]) == (320, 176) and FRAMES - 3 <= info["frames"], info
    frames = info["frames"]
    assert info["audio"] == {"rate": 37800, "channels": 2}, info
    build = project / ".epok/build"
    stats = symbols(build / "epok.elf")["_ZN4epok11video_statsE"]

    rows = emulate(build, work / "play", stats, 420, burst=100)
    playing = [r for r in rows if r["state"] == 4]
    assert playing, "the video never reached the Playing state"
    end = rows[-1]
    assert end["state"] == 0, f"the player did not return to the game: {end}"
    assert end["errors"] == 0, f"playback errors: {end}"
    assert end["shown"] >= frames - 1, f"only {end['shown']} of {frames} frames shown: {end}"
    # A frame decodes (bitstream, MDEC, upload) within the time between frames.
    line_ms = 1000 / (60 * 262.5)
    decode_ms = max(r["decode_lines"] for r in playing) * line_ms
    bitstream_ms = max(r["bitstream_lines"] for r in playing) * line_ms
    assert 0 < bitstream_ms < decode_ms < 1000 / RATE, (bitstream_ms, decode_ms)
    seconds = (playing[-1]["vsync"] - playing[0]["vsync"]) / 60
    assert abs(seconds - frames / RATE) < 1.0, f"played for {seconds:.2f}s instead of {frames / RATE}s"
    middle = min((r for r in playing if r["shown"] >= frames // 2), key=lambda r: r["vsync"])
    capture = work / "play" / f"f{(middle['vsync'] // 15 + 1) * 15:04d}.raw"
    width, height, pixels = picture(capture)
    assert (width, height) == (320, 240), (width, height)
    at = lambda x, y: pixels[y * width + x]
    # 320x176 picture centred vertically: black bars of 32 lines.
    assert near(at(160, 8), (0, 0, 0)) and near(at(160, 231), (0, 0, 0))
    for (x, y), color in zip([(40, 60), (280, 60), (40, 180), (280, 180)], QUADRANTS):
        assert near(at(x, y), color), f"pixel {x},{y} is {at(x, y)}, expected {color}"
    # Every displayed frame, vsync by vsync: the bar only ever moves right, so
    # no older picture reappears when the double-buffered display flips.
    positions = []
    for raw in sorted((work / "play").glob("b*.raw")):
        width, _, pixels = picture(raw)
        row = pixels[119 * width:120 * width]
        positions.append(next((x for x, p in enumerate(row) if sum(p) < 120), None))
    assert None not in positions and positions == sorted(positions), positions
    assert positions[-1] > positions[0], positions
    after = sorted((work / "play").glob("f*.raw"))[-1]
    restored = picture(after)[:2]
    expected = tuple(map(int, args.display.split("x")))
    assert restored == expected, f"display {restored} after the video, expected {expected}"

    skip_at = playing[0]["vsync"] + 60
    skipped = emulate(build, work / "skip", stats, skip_at + 60, skip=skip_at)
    final = skipped[-1]
    assert final["state"] == 0 and final["shown"] < frames - 10, f"START did not skip: {final}"

    for raw in [capture, after]:
        png(raw)
    summary = {"display": args.display, "report": report, "playing_seconds": round(seconds, 2),
               "max_decode_ms": round(decode_ms, 1), "max_bitstream_ms": round(bitstream_ms, 1),
               "final": end, "skipped": final, "frame": str(capture.with_suffix(".png")),
               "work": str(work)}
    (work / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps(summary, indent=2))
    if not args.keep:
        print(f"Kept artefacts in {work}")


if __name__ == "__main__":
    main()
