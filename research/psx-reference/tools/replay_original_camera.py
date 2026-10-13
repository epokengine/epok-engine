"""Replay observed SCUS_942.44 target-camera updates without running game code.

Models 80031e84 (look direction), 80025188 (rail/target blending, smoothing,
limits), and their angle helpers. Lookup tables are read from the identified
local executable and verified against both RAM captures. This does not replay
rail selection, gameplay, rasterization, or any unobserved camera mode.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

EXE_SHA = "1b93cc563c90966e1d79b750f7d19bc1662859cceeb991775e9e9a22656d71d1"
RANGES = ((0x80025188, 0x800255bc), (0x80031e10, 0x80031fb4),
          (0x80025c54, 0x80025ce8), (0x800247dc, 0x80024918),
          (0x8004f794, 0x8004f914), (0x8005c91c, 0x8005d11e),
          (0x8005f6b8, 0x8005feba))


def u32(value): return value & 0xffffffff
def i32(value): return ((value + 0x80000000) & 0xffffffff) - 0x80000000
def i16(value): return ((value + 0x8000) & 0xffff) - 0x8000
def delta(a, b): return ((b - a + 2048) & 4095) - 2048
def lerp(a, b, weight): return (a + (i32(i32(weight) * delta(a, b)) >> 8)) & 4095
def midpoint(a, b): return (a + (delta(a, b) >> 1)) & 4095
def smooth(a, b, weight, limit):
    difference = max(-limit, min(limit, delta(a, b)))
    return (a + (i32(i32(weight) * difference) >> 8)) & 4095


def clamp_angle(angle, base, positive, negative):
    # This branch-based wrap keeps +2048, unlike the sign-extended helpers.
    difference = (angle & 4095) - (base & 4095)
    if difference > 2048: difference -= 4096
    if difference < -2048: difference += 4096
    if difference > positive: difference = positive
    if difference < -negative: difference = -negative
    return (base + difference) & 4095


class Direction:
    def __init__(self, sine, atan):
        if len(sine) != 1025 or len(atan) != 1025:
            raise ValueError("Expected two 1025-entry tables")
        self.sine, self.atan = sine, atan

    def sin(self, angle):
        quadrant, index = divmod(angle & 4095, 1024)
        value = self.sine[index if quadrant % 2 == 0 else 1024-index]
        return -value if quadrant >= 2 else value

    def ratan2(self, y, x):
        x, y = i32(x), i32(y)
        if min(x, y) == -0x80000000:
            raise ValueError("INT_MIN ratan2 input is outside this replay's verified domain")
        negative_x, negative_y = x < 0, y < 0
        x, y = abs(x), abs(y)
        if x == y == 0: return 0
        numerator, denominator = (y, x) if x > y else (x, y)
        index = (numerator // (denominator >> 10) if numerator & 0x7fe00000
                 else (numerator << 10) // denominator)
        if not 0 <= index <= 1024: raise ValueError("Atan table index outside bounds")
        angle = self.atan[index]
        if x <= y: angle = 1024 - angle
        if negative_x: angle = 2048 - angle
        return -angle if negative_y else angle

    def look(self, target, camera, reference_yaw):
        x, y, z = [i32(a-b) for a, b in zip(target, camera)]
        yaw = self.ratan2(i32(-x), i32(-z)) & 4095
        offset = yaw - (reference_yaw & 4095)
        if offset > 2048: offset -= 4096
        if offset < -2048: offset += 4096
        pitch = -1024
        if -1024 <= offset <= 1024:
            z_term = i32((i32(-z) >> 8) * self.sin(-yaw+1024)) >> 4
            x_term = i32((i32(-x) >> 8) * self.sin(-yaw)) >> 4
            pitch = self.ratan2(y, i32(z_term-x_term)) & 4095
        return [u32(pitch), yaw, 0]


def property_values(row, identifier):
    expected = {0x183: 3, 0x119: 4, 0x16e: 2}[identifier]
    matches = [p for p in row["properties"] if p[0] == identifier and p[4] != 0]
    if len(matches) > 1: raise ValueError("Duplicate resolved camera property")
    if not matches: return None
    if len(matches[0]) != 5 + expected: raise ValueError("Invalid camera property length")
    return [i16(v) for v in matches[0][5:]]


def follow(row, look):
    base, previous, flags = row["base"], row["previous"], row["flags"]
    override = (flags[0] >> 8) & 255
    skip_smoothing = (flags[1] >> 8) & 255
    angles = [(look[0] + flags[3]) & 4095, look[1], look[2]]
    if override == 0:
        weights = property_values(row, 0x183)
        angles = ([midpoint(a, b) for a, b in zip(angles, base)] if weights is None else
                  [lerp(a, b, w) for a, b, w in zip(angles, base, weights)])
        if flags[4] != 0: angles[0] = lerp(angles[0], flags[5], i32(flags[4]) >> 2)
        pitch = property_values(row, 0x16e)
        bias = 0 if pitch is None else pitch[0]
        if pitch is not None and flags[10] != 0:
            bias = pitch[1] if flags[10] == 1024 else lerp(*pitch, i32(flags[10]) >> 2)
        angles[0] = (angles[0] + bias) & 4095
        elapsed = u32(flags[9] - flags[7])
        if flags[6] != 0 and elapsed < flags[8]:
            weight = u32(elapsed * 256) // flags[8]
            angles = [lerp(a, b, weight) for a, b in zip(row["transition"], angles)]
        if not skip_smoothing and previous[0] != 0x1001 and flags[2] != 2:
            angles = [smooth(a, b, 128, 45) for a, b in zip(previous, angles)]
    elif override == 1:
        angles = list(base)
    limits = property_values(row, 0x119)
    if limits is not None:
        angles[0] = clamp_angle(angles[0], base[0], limits[2], limits[3])
        angles[1] = clamp_angle(angles[1], base[1], limits[0], limits[1])
        angles[2] &= 4095
    return angles


def verify(exe, before, after, profile):
    if hashlib.sha256(exe).hexdigest() != EXE_SHA: raise ValueError("Wrong source executable")
    if len(before) != 2097152 or len(after) != 2097152: raise ValueError("Expected two 2 MiB RAM captures")
    base, size = struct.unpack_from("<II", exe, 0x18)
    def read(address, length):
        if not base <= address < address+length <= base+size: raise ValueError("Outside EXE payload")
        return exe[0x800+address-base:0x800+address-base+length]
    ranges = []
    for start, end in RANGES:
        data = read(start, end-start)
        for ram in (before, after):
            if ram[start-0x80000000:end-0x80000000] != data:
                raise ValueError(f"Observed code/table differs from EXE at {start:08x}")
        ranges.append(dict(start=f"{start:08x}", end=f"{end:08x}", bytes=len(data), sha256=hashlib.sha256(data).hexdigest()))
    if profile.get("callback_error") or profile.get("overflow"): raise ValueError("Invalid original observer capture")
    trace = profile["camera_trace"]
    if trace.get("version") != 1 or trace.get("overflow") or trace.get("incomplete") or not trace.get("rows"):
        raise ValueError("Incomplete/empty camera trace")
    direction = Direction(struct.unpack("<1025h", read(0x8005c91c, 2050)),
                          struct.unpack("<1025h", read(0x8005f6b8, 2050)))
    errors, calls, modes, maximum_delta, properties = [], 0, Counter(), [0, 0, 0], Counter()
    for index, row in enumerate(trace["rows"]):
        if len(row["flags"]) != 11: raise ValueError("Invalid flag layout")
        look = direction.look(row["target"], row["camera"], row["base"][1])
        if look != row["look"]: errors.append(dict(row=index, stage="look", expected=row["look"], actual=look))
        angles = follow(row, look)
        if angles != row["angles"]: errors.append(dict(row=index, stage="follow", expected=row["angles"], actual=angles))
        for op in row["operations"]:
            if len(op) != 7 or op[0] not in (1, 2, 3): raise ValueError("Invalid helper trace")
            actual = (lerp(*op[2:5]) if op[0] == 1 else midpoint(*op[2:4]) if op[0] == 2 else smooth(*op[2:6]))
            if actual != op[6]: errors.append(dict(row=index, stage="helper", operation=op, actual=actual))
            calls += 1
        modes[str(row["flags"][2])] += 1
        for axis in range(3): maximum_delta[axis] = max(maximum_delta[axis], abs(delta(row["base"][axis], angles[axis])))
        for prop in row["properties"]:
            properties[(prop[0], tuple(prop[5:]))] += 1
    return dict(scope=__doc__, updates=len(trace["rows"]), angle_helper_calls=calls,
                modes=dict(modes), exact=not errors, error_count=len(errors), errors=errors[:16],
                max_base_angle_difference_turn12=maximum_delta,
                max_base_angle_difference_degrees=[v * 360 / 4096 for v in maximum_delta],
                properties=[dict(id=f"0x{i:03x}", values=list(v), count=n) for (i, v), n in sorted(properties.items())],
                verified_ranges=ranges)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("exe", "before", "after", "profile", "output"):
        parser.add_argument("--"+name, type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists(): parser.error("Refusing to overwrite evidence")
    inputs = {name: getattr(args, name).read_bytes() for name in ("exe", "before", "after", "profile")}
    report = verify(inputs["exe"], inputs["before"], inputs["after"], json.loads(inputs["profile"]))
    report["input_sha256"] = {name: hashlib.sha256(data).hexdigest() for name, data in inputs.items()}
    with args.output.open("x", encoding="utf-8") as out: json.dump(report, out, indent=2)
    print(json.dumps({k: v for k, v in report.items() if k not in ("scope", "verified_ranges", "input_sha256")}))
    if not report["exact"]: raise SystemExit(1)


if __name__ == "__main__": main()
