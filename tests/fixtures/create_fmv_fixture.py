"""Generate Epok's original FMV test streams (no third-party footage or audio).

Run from the repository root:
    python3 tests/fixtures/create_fmv_fixture.py <psxavenc> [ffmpeg]

Each 64x48 frame has red, green, blue and white quadrants and an 8x8 black
square that moves 12 pixels to the right per frame. psxavenc 0.3.1 keeps the
first four of six 30 fps frames in `fmv-pattern-v3.str`, with a 440/660 Hz
stereo tone, and the first three of four silent frames in `fmv-pattern-v2.str`.
The streams are checked in; neither tool is a test dependency.
"""
import math
import pathlib
import subprocess
import sys
import tempfile

WIDTH, HEIGHT, RATE = 64, 48, 30
QUADRANTS = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]


def frame(index):
    pixels = bytearray()
    for y in range(HEIGHT):
        for x in range(WIDTH):
            color = QUADRANTS[(y >= HEIGHT // 2) * 2 + (x >= WIDTH // 2)]
            square = 4 + 12 * index
            if square <= x < square + 8 and 20 <= y < 28:
                color = (0, 0, 0)
            pixels += bytes(color)
    return bytes(pixels)


def encode(psxavenc, ffmpeg, folder, name, frames, codec, audio):
    raw = folder / f"{name}.rgb"
    raw.write_bytes(b"".join(frame(i) for i in range(frames)))
    source = folder / f"{name}.mkv"
    command = [ffmpeg, "-hide_banner", "-loglevel", "error", "-y",
               "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{WIDTH}x{HEIGHT}",
               "-r", str(RATE), "-i", str(raw)]
    if audio:
        seconds = frames / RATE
        command += ["-f", "lavfi", "-i",
                    f"aevalsrc=0.25*sin(2*PI*440*t)|0.20*sin(2*PI*660*t):s=37800:d={seconds}",
                    "-c:a", "pcm_s16le"]
    command += ["-c:v", "ffv1", str(source)]
    subprocess.run(command, check=True)
    target = pathlib.Path("tests/fixtures") / f"{name}.str"
    subprocess.run([psxavenc, "-q", "-t", "str", "-v", codec, "-s", f"{WIDTH}x{HEIGHT}",
                    "-r", str(RATE), "-x", "2", str(source), str(target)], check=True)
    print(target, target.stat().st_size)


def main():
    psxavenc = sys.argv[1]
    ffmpeg = sys.argv[2] if len(sys.argv) > 2 else "ffmpeg"
    with tempfile.TemporaryDirectory() as temp:
        folder = pathlib.Path(temp)
        encode(psxavenc, ffmpeg, folder, "fmv-pattern-v3", 6, "v3", True)
        encode(psxavenc, ffmpeg, folder, "fmv-pattern-v2", 4, "v2", False)


if __name__ == "__main__":
    main()
