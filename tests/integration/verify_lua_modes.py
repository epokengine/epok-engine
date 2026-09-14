#!/usr/bin/env python3
"""M5 acceptance: one project, three LuaExecution modes, identical results.

ONE real project is created through the production CLI, the SAME `.lua` files
are compiled by every `LuaExecution` mode (`native_cpp`, `vm_bytecode`,
`vm_source`, written into the `<Name>.epokproject` manifest field
`lua_execution`), each mode is built with `--build-psx` and run in PCSX-Redux,
and a 96-slot probe array is read out of guest RAM. The probe values must be
IDENTICAL across the three modes and equal to constants computed here by
reimplementing the numeric contract (`knowledge/initiatives/lua-scripting/
contract.md` §3) in Python.

Everything measured here is measured in PCSX-Redux; nothing was validated on
physical hardware.

    python3 tests/integration/verify_lua_modes.py
    python3 tests/integration/verify_lua_modes.py --no-emulator   # build only
    python3 tests/integration/verify_lua_modes.py --keep          # retain project
"""
import sys as _sys
from pathlib import Path as _Path

_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents  # noqa: E402

import argparse  # noqa: E402
import contextlib  # noqa: E402
import hashlib  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import shutil  # noqa: E402
import statistics  # noqa: E402
import subprocess  # noqa: E402
import tempfile  # noqa: E402
import uuid  # noqa: E402

ROOT = _Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "tests/integration/lua_conformance"
NUGGET = ROOT / "third_party/nugget"
PREFIX = "mipsel-none-elf"
MODES = ("native_cpp", "vm_bytecode", "vm_source")
PROBE_SLOTS = 96
# `EnemyBase::slot`: the probe base each actor writes through.
GUARD_BASE, PATROL_BASE = 0, 32

# Guard/Patrol class identities, mirroring tests/integration/lua_conformance.
ENEMY_BASE_ID = "6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d10"
DIRECTOR_ID = "6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d20"
GUARD_ID = "1f9a6b30-2c4d-4e58-9a71-3b5c7d9e0f11"
PATROL_ID = "2a8b5c41-3d5e-4f69-8b82-4c6d8e0f1a22"
SENTINEL_ID = "3b9c6d52-4e6f-4a7b-9c93-5d7e9f0a1b33"
SCENE_ROOT_ID = "ed73d249-b6cb-4a3c-a0e8-696de55e286f"

RESULTS = []


def record(passed, line):
    RESULTS.append({"passed": bool(passed), "line": line})
    print(f"{'PASS' if passed else 'FAIL'}: {line}", flush=True)
    return bool(passed)


# ---------------------------------------------------------------------------
# The numeric contract (§3), reimplemented rather than copied from a run.
# ---------------------------------------------------------------------------
Q12 = 4096
I32_MIN, I32_MAX, U32_MAX = -(2**31), 2**31 - 1, 2**32 - 1


def saturate(value):
    return I32_MAX if value > I32_MAX else I32_MIN if value < I32_MIN else value


def usaturate(value):
    return U32_MAX if value > U32_MAX else 0 if value < 0 else value


def trunc(a, b):
    """C integer division: truncation toward zero, never floor."""
    quotient = abs(a) // abs(b)
    return quotient if (a < 0) == (b < 0) else -quotient


def iadd(a, b):
    return saturate(a + b)


def isub(a, b):
    return saturate(a - b)


def imul(a, b):
    return saturate(a * b)


def idiv(a, b):
    return saturate(trunc(a, b)) if b else 0


def imod(a, b):
    return a - trunc(a, b) * b if b else 0


def ineg(a):
    return saturate(-a)


def uadd(a, b):
    return U32_MAX if U32_MAX - a < b else a + b


def usub(a, b):
    return 0 if a < b else a - b


def umul(a, b):
    return usaturate(a * b)


def udiv(a, b):
    return a // b if b else 0


def umod(a, b):
    return a % b if b else 0


def fmul(a, b):
    return saturate(trunc(a * b, Q12))


def fdiv(a, b):
    return saturate(trunc(a * Q12, b)) if b else 0


def to_int(raw):
    return trunc(raw, Q12)


def from_int(value):
    return saturate(value * Q12)


def fixed(value):
    """Q12 raw of an exactly representable decimal literal."""
    raw = value * Q12
    assert abs(raw - round(raw)) < 1e-9, f"{value} is not exact in Q12"
    return int(round(raw))


def as_int32(value):
    """A UInt32 bit pattern as the int32 the probe array stores."""
    return value - 2**32 if value > I32_MAX else value


# ---------------------------------------------------------------------------
def expected_probe():
    """Every probe slot, derived from the fixture sources and the helpers above.

    Slots 58..61 (arena statistics) and 62 (the mode id) are mode-dependent by
    construction and are excluded from the cross-mode comparison.
    """
    probe = [0] * PROBE_SLOTS

    # EnemyBase::begin_play, through the BASE type, on both actors:
    #   damage(0.5) then on_alert(). Guard does not override damage, so the
    #   native body runs directly; Patrol overrides it and calls super first.
    health = fixed(100.0)
    health = isub(health, fixed(0.5))          # EnemyBase::damage
    hits = iadd(0, 1)
    guard_count = 1                            # Guard:on_alert
    patrol_count = 1 + 1                       # Patrol:damage, then on_alert

    for base, count in ((GUARD_BASE, guard_count), (PATROL_BASE, patrol_count)):
        probe[base + 0] = 1                    # Guard:begin_play ran
        probe[base + 1] = hits
        probe[base + 2] = health
        probe[base + 3] = fixed(1.5)           # speed
        probe[base + 4] = count
        probe[base + 5] = 7                    # stamina
        probe[base + 6] = 1                    # armed

    # First tick: self:damage(0.25), then an inherited property written in Lua.
    tick_health = isub(health, fixed(0.25))
    tick_hits = iadd(iadd(hits, 1), 10)
    guard_tick_count = guard_count
    patrol_tick_count = patrol_count + 1       # Patrol:damage increments count
    for base, count in ((GUARD_BASE, guard_tick_count), (PATROL_BASE, patrol_tick_count)):
        probe[base + 8] = count
        probe[base + 9] = tick_hits
        probe[base + 10] = tick_health
        probe[base + 11] = iadd(tick_health, fixed(1.5))   # Guard:report()

    # The Director deactivates Guard1 before tick 141 and destroys it before
    # tick 151; Patrol1 keeps ticking to the final tick.
    probe[GUARD_BASE + 7] = 140
    probe[PATROL_BASE + 7] = 160
    probe[GUARD_BASE + 12] = 1                 # on_disable, once
    probe[GUARD_BASE + 13] = 1                 # end_play, once
    probe[PATROL_BASE + 12] = 0
    probe[PATROL_BASE + 13] = 0

    # The numeric edge battery (Guard1 only; see Guard.lua).
    probe[16] = iadd(I32_MAX, 1)
    probe[17] = isub(-2147483647, 2)
    probe[18] = imul(65536, 65536)
    probe[19] = idiv(7, 0)
    probe[20] = imod(7, 0)
    probe[21] = ineg(isub(-2147483647, 2))
    probe[22] = as_int32(uadd(U32_MAX, 1))
    probe[23] = as_int32(usub(0, 1))
    probe[24] = as_int32(umul(65536, 65536))
    probe[25] = as_int32(udiv(9, 0))
    probe[26] = as_int32(umod(9, 0))
    probe[27] = fmul(fixed(1.000244140625), fixed(-0.300048828125))
    probe[28] = fdiv(fixed(5.0), fixed(0.0))
    probe[29] = to_int(fixed(-2.5))
    probe[30] = from_int(1000000)
    probe[31] = 1                              # 0x80000000 > 1 as unsigned
    probe[14] = 1                              # the one real bump() returned true
    for_sum = sum(range(10, 0, -2))
    probe[15] = 2                              # the elseif branch ran
    probe[53] = iadd(imul(for_sum, 10), 2)

    # Intrinsic transform access, driven from Lua on every tick with constant
    # steps: the final value is the tick count times the step, and the native
    # read-back proves the writes landed in the actor's root component.
    guard_rotation, guard_position = fixed(0.0), fixed(0.0)
    for _ in range(140):
        guard_rotation = saturate(guard_rotation + fixed(0.25))
        guard_position = saturate(guard_position + fixed(0.5))
    patrol_rotation, patrol_position = guard_rotation, guard_position
    for _ in range(160 - 140):
        patrol_rotation = saturate(patrol_rotation + fixed(0.25))
        patrol_position = saturate(patrol_position + fixed(0.5))
    probe[64] = guard_rotation
    probe[65] = guard_position
    probe[66] = patrol_rotation
    probe[67] = patrol_position
    probe[68] = patrol_rotation
    probe[69] = patrol_position

    # The builtin surface, driven from Lua with no C++ helper and no Blueprint.
    probe[70] = 1                              # is_valid(self.ref)
    probe[71] = 1                              # is_a(self.ref, "Guard") on a Guard
    probe[72] = 0                              # is_a(self.ref, "Patrol") on a Guard
    probe[73] = 0                              # cast to an unrelated class is null
    probe[74] = 1                              # cast to the shared base succeeds
    probe[75] = 0                              # input.held, no pad connected
    probe[76] = 0                              # input.pressed
    probe[77] = 0                              # input.released
    probe[78] = 0                              # request_scene(9): one scene only
    probe[79] = 1                              # is_valid(self.ref) at the spawn site
    probe[80] = 1                              # the spawned Lua actor began play
    probe[81] = 20                             # its own ticks before self-destruct
    probe[82] = 0                              # the stored reference is stale now
    probe[83] = 40                             # the tick the check ran on
    probe[84] = 0                              # the spawn is queued, not immediate
    probe[85] = 1                              # a Patrol is a Guard
    probe[86] = 1                              # and is a Patrol
    probe[87] = 1                              # so it casts to itself
    probe[88] = 1                              # the spawned actor is a Sentinel
    probe[89] = 1                              # and its own reference is valid

    probe[48] = 0x4C4D4F31                     # magic
    probe[49] = 2                              # EnemyBase::begin_play, twice
    probe[50] = 4                              # EnemyBase::damage body, 4 times
    probe[51] = 1                              # bump(): short circuit skipped two
    probe[52] = 12                             # next_id() * 10 + next_id()
    probe[54] = 0                              # no tick advanced while paused
    probe[55] = 0                              # no tick after set_active(false)
    probe[56] = 0                              # no tick after destroy
    probe[57] = 161                            # Director ticks
    probe[63] = 0x0000D09E                     # done sentinel
    return probe


# Slots whose value legitimately depends on the selected mode.
MODE_DEPENDENT = {58, 59, 60, 61, 62}
SLOT_NAMES = {
    **{GUARD_BASE + k: f"guard.{n}" for k, n in enumerate(
        ["begin_play", "hits@begin", "health@begin", "speed", "count@begin", "stamina",
         "armed", "ticks", "count@tick", "hits@tick", "health@tick", "report",
         "on_disable", "end_play", "bump_result", "definite_local"])},
    **{PATROL_BASE + k: f"patrol.{n}" for k, n in enumerate(
        ["begin_play", "hits@begin", "health@begin", "speed", "count@begin", "stamina",
         "armed", "ticks", "count@tick", "hits@tick", "health@tick", "report",
         "on_disable", "end_play", "unused", "unused"])},
    16: "iadd.saturate", 17: "isub.saturate", 18: "imul.saturate", 19: "idiv.zero",
    20: "imod.zero", 21: "ineg.int32_min", 22: "uadd.saturate", 23: "usub.saturate",
    24: "umul.saturate", 25: "udiv.zero", 26: "umod.zero", 27: "fmul.truncate",
    28: "fdiv.zero", 29: "to_int.truncate", 30: "from_int.saturate", 31: "unsigned.order",
    48: "magic", 49: "native.begin_play", 50: "native.damage", 51: "short_circuit.bumps",
    52: "evaluation.order", 53: "for.negative_step", 54: "pause.tick_delta",
    55: "deactivate.tick_delta", 56: "destroy.tick_delta", 57: "director.ticks",
    58: "lua.arena_peak", 59: "lua.arena_live", 60: "lua.arena_allocations",
    61: "lua.arena_failures", 62: "lua.mode", 63: "done",
    64: "guard.rotation.y", 65: "guard.position.x",
    66: "patrol.rotation.y", 67: "patrol.position.x",
    68: "native.rotation.y", 69: "native.position.x",
    70: "builtin.spawn.valid", 71: "builtin.is_a.match", 72: "builtin.is_a.mismatch",
    73: "builtin.cast.incompatible", 74: "builtin.cast.base", 75: "builtin.input.held",
    76: "builtin.input.pressed", 77: "builtin.input.released", 78: "builtin.request_scene",
    79: "builtin.self.ref", 80: "spawned.begin_play", 81: "spawned.ticks",
    82: "spawned.destroyed", 83: "builtin.checkpoint", 84: "builtin.spawn.deferred",
    85: "patrol.is_a.base", 86: "patrol.is_a.self", 87: "patrol.cast.self",
    88: "spawned.is_a.self", 89: "spawned.ref.valid",
}


# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------
def editor():
    for candidate in ("target/release/epok-editor", "target/release/epok-editor.exe",
                      "target/debug/epok-editor", "target/debug/epok-editor.exe"):
        path = ROOT / candidate
        if path.is_file():
            return path
    raise SystemExit("Build the editor first: cargo build --release")


def emulator_paths():
    emulator = None
    for candidate in (ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/PCSX-Redux",
                      ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/pcsx-redux",
                      ROOT / ".tools/redux/pcsx-redux.exe",
                      ROOT / ".tools/redux/pcsx-redux"):
        if candidate.is_file():
            emulator = candidate
            break
    bios = None
    if emulator:
        found = sorted((ROOT / ".tools").rglob("openbios.bin"))
        bios = found[0] if found else None
    return emulator, bios


EDITOR = editor()


def cli(*args, ok=True, timeout=2400):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                            text=True, encoding="utf-8", errors="replace", timeout=timeout)
    if (result.returncode == 0) != ok:
        raise AssertionError(" ".join(map(str, args)) + "\n" + result.stdout + result.stderr)
    return result


def tool(name, *args):
    binary = shutil.which(f"{PREFIX}-{name}")
    assert binary, f"missing {PREFIX}-{name} on PATH"
    result = subprocess.run([binary, *map(str, args)], capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=300)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


def digest(path):
    return hashlib.sha256(_Path(path).read_bytes()).hexdigest()


# ---------------------------------------------------------------------------
# Project construction
# ---------------------------------------------------------------------------
def create_project(directory):
    root = directory / "LuaModes"
    cli("--create-project", root, "--name", "Lua Modes")
    scripts = root / "assets/scripts"
    for name in ("EnemyBase.hpp", "Probe.cpp", "Guard.lua", "Patrol.lua", "Sentinel.lua"):
        shutil.copy2(FIXTURES / name, scripts / name)

    path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(path.read_text(encoding="utf-8"))
    # The Director is a component: the level calls `frame_update` once per
    # rendered frame even while `epok::time` is paused, which is the only way a
    # pause window can end.
    scene["actors"][0]["components"].append({
        "class": {"class_id": DIRECTOR_ID, "name": "Director"},
        "id": str(uuid.uuid5(uuid.NAMESPACE_DNS, "epok.lua.modes.director")),
        "name": "Director", "overrides": [], "properties": {}})

    def actor(name, class_id, class_name, slot):
        return {
            "active": True,
            "class": {"class_id": class_id, "name": class_name},
            "components": [{
                "class": {"class_id": SCENE_ROOT_ID, "name": "epok::SceneComponent3D"},
                "id": str(uuid.uuid5(uuid.NAMESPACE_DNS, f"epok.lua.modes.{name}.root")),
                "name": "Transform", "overrides": ["position"],
                "properties": {"position": [0.0, 0.0, 0.0]}, "root": True}],
            "id": str(uuid.uuid5(uuid.NAMESPACE_DNS, f"epok.lua.modes.{name}")),
            "name": name, "overrides": ["slot"], "properties": {"slot": slot},
        }

    scene["actors"].append(actor("Guard1", GUARD_ID, "Guard", GUARD_BASE))
    scene["actors"].append(actor("Patrol1", PATROL_ID, "Patrol", PATROL_BASE))
    documents.write_text(path, documents.dumps(scene))
    return root


def select(root, mode):
    descriptor = next(p for p in root.iterdir() if p.suffix == ".epokproject")
    text = descriptor.read_text(encoding="utf-8")
    updated = re.sub(r"^lua_execution: .*$", f"lua_execution: {mode}", text, flags=re.M)
    assert updated != text or f"lua_execution: {mode}" in text, "manifest has no lua_execution"
    documents.write_text(descriptor, updated)


# ---------------------------------------------------------------------------
# Build, measure, run
# ---------------------------------------------------------------------------
def section_sizes(elf):
    out = tool("size", "-A", str(elf))
    sections = {}
    for line in out.splitlines():
        match = re.match(r"^(\.\S+)\s+(\d+)\s+", line)
        if match:
            sections[match[1]] = int(match[2])
    text = sections.get(".text", 0)
    rodata = sections.get(".rodata", 0) + sections.get(".rodata1", 0)
    data = sections.get(".data", 0) + sections.get(".sdata", 0)
    bss = sections.get(".bss", 0) + sections.get(".sbss", 0)
    return {"text": text, "rodata": rodata, "data": data, "bss": bss,
            "linked_resident_bytes": text + rodata + data + bss}


def symbols(elf):
    table = {}
    for line in tool("nm", "--defined-only", str(elf)).splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+\S\s+(\S+)$", line)
        if match:
            table[match[2]] = int(match[1], 16)
    return table


DRIVER = r"""local ffi = require('ffi')
local probe = ffi.cast('int32_t*', PCSX.getMemPtr() + %(probe)d)
local started, finished = nil, false
local samples = {}

local function finish(reason)
  if finished then return end
  finished = true
  local f = assert(io.open(%(output)s, 'w'))
  f:write('{"reason":"' .. reason .. '","probe":[')
  for i = 0, %(slots)d - 1 do
    if i > 0 then f:write(',') end
    f:write(tostring(tonumber(probe[i])))
  end
  f:write('],"cycles":[')
  for i, v in ipairs(samples) do
    if i > 1 then f:write(',') end
    f:write(tostring(v))
  end
  f:write(']}')
  f:close()
  PCSX.quit(0)
end

lua_modes_begin = PCSX.addBreakpoint(%(begin)d, 'Exec', 4, 'tick begin', function()
  if finished then return true end
  started = tonumber(PCSX.getCPUCycles())
  return true
end)
lua_modes_end = PCSX.addBreakpoint(%(finish)d, 'Exec', 4, 'tick end', function()
  if finished then return true end
  if started ~= nil then
    samples[#samples + 1] = tonumber(PCSX.getCPUCycles()) - started
    started = nil
  end
  return true
end)
lua_modes_done = PCSX.addBreakpoint(%(done)d, 'Exec', 4, 'done', function()
  finish('done')
  return true
end)
"""


def run_emulator(project, workdir, timeout):
    emulator, bios = emulator_paths()
    if not emulator or not bios:
        return {"ok": False, "error": "PCSX-Redux or openbios.bin is not installed under .tools"}
    build = project / ".epok/build"
    table = symbols(build / "epok.elf")
    missing = [s for s in ("epok_lua_probe", "epok_lua_probe_begin", "epok_lua_probe_end",
                           "epok_lua_probe_done") if s not in table]
    if missing:
        return {"ok": False, "error": f"missing probe symbols: {missing}"}
    workdir.mkdir(parents=True, exist_ok=True)
    output = workdir / "probe.json"
    if output.exists():
        output.unlink()
    script = workdir / "driver.lua"
    script.write_text(DRIVER % {
        "probe": table["epok_lua_probe"] & 0x1FFFFF,
        "output": json.dumps(str(output)),
        "slots": PROBE_SLOTS,
        "begin": table["epok_lua_probe_begin"],
        "finish": table["epok_lua_probe_end"],
        "done": table["epok_lua_probe_done"],
    }, encoding="utf-8")
    portable = workdir / "portable"
    portable.mkdir(exist_ok=True)
    command = [str(emulator), "-portable", str(portable), "-run", "-stdout", "-interpreter",
               "-softgpu", "-2mb", "-no-gdb", "-loadexe", str(build / "epok.ps-exe"),
               "-bios", str(bios), "-dofile", str(script), "-debugger"]
    with (workdir / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=portable, stdout=log, stderr=log,
                                   stdin=subprocess.DEVNULL)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=30)
            return {"ok": False, "error": f"emulator timed out after {timeout}s"}
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    if not output.is_file():
        return {"ok": False, "error": f"emulator produced no probe (exit {code})",
                "log": str(workdir / "emulator.log")}
    return {"ok": True, **json.loads(output.read_text(encoding="utf-8"))}


def cycle_summary(cycles):
    # The first tick carries the one-shot numeric battery; it is warmup here.
    samples = sorted(cycles[20:260]) or sorted(cycles)
    if not samples:
        return None
    return {"samples": len(samples), "min": samples[0],
            "median": statistics.median(samples), "max": samples[-1],
            "p95": samples[(len(samples) * 95 - 1) // 100]}


def generated_inventory(build):
    directory = build / "scripts/generated/lua"
    if not directory.is_dir():
        return {}
    return {str(p.relative_to(directory)): digest(p)
            for p in sorted(directory.rglob("*")) if p.is_file() and p.suffix != ".o"}


def staged_documents(build):
    """Mode-independent staged documents: scene tables and actor identities."""
    result = {}
    for name in ("scene.epokmap", "scene.hh"):
        path = build / name
        if path.is_file():
            result[name] = digest(path)
    return result


def build_mode(project, mode, args, report):
    select(project, mode)
    cli("--project", project, "--build-psx")
    build = project / ".epok/build"
    entry = {
        "mode": mode,
        "ps_exe_bytes": (build / "epok.ps-exe").stat().st_size,
        "ps_exe_sha256": digest(build / "epok.ps-exe"),
        "elf_sections": section_sizes(build / "epok.elf"),
        "generated": generated_inventory(build),
        "staged_documents": staged_documents(build),
        "scripts_manifest": documents.loads(
            (build / "Scripts.epokmanifest").read_text(encoding="utf-8")),
        "interpreter_symbols": sorted(
            name for name in symbols(build / "epok.elf")
            if name.startswith(("lua_", "luaL_", "luaU_", "luaD_", "luaV_"))),
    }
    if not args.no_emulator:
        run = run_emulator(project, project / f".epok/lua-modes/{mode}", args.timeout)
        entry["emulator"] = {k: v for k, v in run.items() if k != "cycles"}
        entry["cycles"] = cycle_summary(run.get("cycles", [])) if run.get("ok") else None
    report["modes"].append(entry)
    return entry


# ---------------------------------------------------------------------------
# Acceptance checks
# ---------------------------------------------------------------------------
UNSUPPORTED = {
    "unbounded_loop": "    while true do end\n",
    "table_constructor": "    local t = {}\n",
    "string_concat": '    local s = "a" .. "b"\n',
}


def check_unsupported_diagnostics(project, report):
    """One source diagnostic, identical in every mode (contract §2)."""
    path = project / "assets/scripts/Broken.lua"
    template = (
        'local Broken = epok.class {\n'
        '    profile = 1,\n'
        '    id = "3b7d6e52-4e6f-4a7b-9c93-5d7e9f0a2b33",\n'
        '    name = "Broken",\n'
        '    extends = "EnemyBase",\n'
        '    properties = {},\n'
        '    functions = {}\n'
        '}\n\n'
        'function Broken:begin_play()\n@BODY@end\n\nreturn Broken\n'
    )
    diagnostics = {}
    for label, body in UNSUPPORTED.items():
        documents.write_text(path, template.replace('@BODY@', body))
        per_mode = {}
        for mode in MODES:
            select(project, mode)
            result = cli("--project", project, "--build-psx", ok=False)
            # Compare the diagnostic itself, not the surrounding build log: the
            # progress lines carry per-run timings.
            match = re.search(r'Error: "(.*)"\s*$', result.stdout + result.stderr, re.S)
            per_mode[mode] = match[1] if match else (result.stdout + result.stderr).strip()
        diagnostics[label] = per_mode
        texts = set(per_mode.values())
        record(len(texts) == 1 and "Broken.lua" in per_mode[MODES[0]]
               and "\n" not in per_mode[MODES[0]],
               f"unsupported `{label}` fails with one identical diagnostic in all three modes: "
               f"{per_mode[MODES[0]].split(': ', 1)[-1] if len(texts) == 1 else texts}")
    path.unlink()
    report["unsupported_diagnostics"] = diagnostics


def check_mode_switch(project, report, native_first):
    """A mode change replaces every execution artifact, in both directions."""
    build = project / ".epok/build"

    select(project, "vm_bytecode")
    cli("--project", project, "--build-psx")
    vm_generated = generated_inventory(build)
    vm_ok = ("lua_bindings.cpp" in vm_generated and "lua_chunks.cpp" in vm_generated)
    headers = [p for p in (build / "scripts/generated/lua").glob("*.hpp")]
    lowered = [p.name for p in headers if "epok::bp::" in p.read_text(encoding="utf-8")]
    record(vm_ok and not lowered,
           "switching to vm_bytecode emits lua_bindings.cpp + lua_chunks.cpp and leaves "
           f"no lowered epok::bp:: in a generated header (found {lowered})")

    select(project, "native_cpp")
    cli("--project", project, "--build-psx")
    back = generated_inventory(build)
    gone = "lua_bindings.cpp" not in back and "lua_chunks.cpp" not in back
    identical = {k: v for k, v in back.items() if k.endswith(".hpp")} == \
                {k: v for k, v in native_first["generated"].items() if k.endswith(".hpp")}
    record(gone and identical,
           "switching back to native_cpp removes lua_bindings.cpp and reproduces the "
           "first native build's header bytes exactly")

    hashes = {entry["mode"]: entry["ps_exe_sha256"] for entry in report["modes"]}
    record(len(set(hashes.values())) == len(hashes),
           f"the play-cache receipt is not reused across modes; epok.ps-exe hashes differ "
           f"({ {m: h[:12] for m, h in hashes.items()} })")
    report["mode_switch"] = {"vm_generated": sorted(vm_generated),
                             "native_generated": sorted(back),
                             "ps_exe_sha256": hashes}


def check_identity(report):
    """Class ids and member ids do not depend on the mode (contract §4)."""
    documents_per_mode = {e["mode"]: e["staged_documents"] for e in report["modes"]}
    reference = documents_per_mode[MODES[0]]
    same = all(value == reference for value in documents_per_mode.values())
    record(same and reference,
           "the staged scene documents (scene.epokmap, scene.hh) are byte-identical in "
           "all three modes")
    deps = {e["mode"]: sorted(e["scripts_manifest"].get("dependencies", []))
            for e in report["modes"]}
    # Only the AUTHORED sources: `vm_source` additionally depends on the
    # generated normalized chunks, which are that mode's own build inputs.
    lua_deps = {m: [d for d in v if d.endswith(".lua") and d.startswith("assets/")]
                for m, v in deps.items()}
    record(all(v == lua_deps[MODES[0]] for v in lua_deps.values()) and lua_deps[MODES[0]],
           f"Scripts.epokmanifest lists the same .lua dependencies in all three modes "
           f"({lua_deps})")
    report["identity"] = {"staged_documents": documents_per_mode, "lua_dependencies": lua_deps}


def check_probes(report):
    expected = expected_probe()
    per_mode = {}
    for entry in report["modes"]:
        run = entry.get("emulator")
        if not run or not run.get("ok"):
            record(False, f"{entry['mode']}: the emulator produced no probe "
                          f"({(run or {}).get('error')})")
            return
        per_mode[entry["mode"]] = run["probe"]

    for mode, probe in per_mode.items():
        wrong = [(slot, SLOT_NAMES.get(slot, str(slot)), expected[slot], probe[slot])
                 for slot in range(PROBE_SLOTS)
                 if slot not in MODE_DEPENDENT and probe[slot] != expected[slot]]
        record(not wrong, f"{mode}: every probe slot equals the value computed from the "
                          f"numeric contract{'' if not wrong else f'; wrong: {wrong}'}")

    reference = per_mode[MODES[0]]
    differing = [(slot, SLOT_NAMES.get(slot, str(slot)),
                  {m: p[slot] for m, p in per_mode.items()})
                 for slot in range(PROBE_SLOTS)
                 if slot not in MODE_DEPENDENT
                 and any(p[slot] != reference[slot] for p in per_mode.values())]
    record(not differing,
           f"the probe array is identical across native_cpp, vm_bytecode and vm_source"
           f"{'' if not differing else f'; differing: {differing}'}")

    report["probe"] = {
        "expected": expected,
        "per_mode": per_mode,
        "mode_dependent_slots": sorted(MODE_DEPENDENT),
        "table": [{"slot": slot, "name": SLOT_NAMES.get(slot, ""),
                   "expected": None if slot in MODE_DEPENDENT else expected[slot],
                   **{mode: per_mode[mode][slot] for mode in MODES}}
                  for slot in range(PROBE_SLOTS)],
    }


def check_export(project, report):
    """Every mode exports a standalone tree that builds with no editor."""
    results = {}
    for mode in MODES:
        select(project, mode)
        destination = _Path(cli("--project", project, "--export-psx").stdout.strip())
        make = subprocess.run(["make", "-j4", "BUILD=Release", f"NUGGET_DIR={NUGGET}"],
                              cwd=destination, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", timeout=2400)
        (destination / "export-build.log").write_text(make.stdout + make.stderr,
                                                      encoding="utf-8")
        executable = destination / "epok.ps-exe"
        files = {str(p.relative_to(destination)) for p in destination.rglob("*") if p.is_file()}
        manifest_path = destination / "Scripts.epokmanifest"
        manifest = documents.loads(manifest_path.read_text(encoding="utf-8")) \
            if manifest_path.is_file() else {}
        lua_deps = sorted(d for d in manifest.get("dependencies", [])
                          if d.endswith(".lua") and d.startswith("assets/"))
        entry = {
            "directory": str(destination),
            "built": make.returncode == 0 and executable.is_file(),
            "ps_exe_bytes": executable.stat().st_size if executable.is_file() else None,
            "has_lua_mk": "lua.mk" in files,
            "has_lua_runtime": "lua_runtime.hpp" in files,
            "has_bindings": any(f.endswith("lua_bindings.cpp") for f in files),
            "has_chunks": any(f.endswith("lua_chunks.cpp") for f in files),
            "lua_dependencies": lua_deps,
            "error": None if make.returncode == 0 else (make.stdout + make.stderr)[-3000:],
        }
        results[mode] = entry
        record(entry["built"],
               f"{mode}: `make BUILD=Release NUGGET_DIR=<nugget>` inside the export produces "
               f"epok.ps-exe with no editor")
        if mode == "native_cpp":
            record(not entry["has_bindings"] and not entry["has_chunks"],
                   "the native_cpp export contains no lua_bindings.cpp and no chunk payload")
        else:
            record(entry["has_lua_mk"] and entry["has_lua_runtime"] and entry["has_chunks"]
                   and entry["has_bindings"] and bool(lua_deps),
                   f"{mode}: the export carries lua.mk, lua_runtime.hpp, the chunk payload and "
                   f"a Scripts.epokmanifest listing {lua_deps}")
    report["exports"] = results


# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-emulator", action="store_true",
                        help="build every mode but skip the on-target run")
    parser.add_argument("--skip-export", action="store_true")
    parser.add_argument("--keep", action="store_true", help="retain the generated project")
    parser.add_argument("--timeout", type=int, default=900)
    parser.add_argument("--output", default=str(ROOT / "artifacts/lua_modes/report.json"))
    args = parser.parse_args()

    # The native scripts translation units and the generated main unit share one
    # optimization level, so the measured sizes compare like for like.
    os.environ["EPOK_RUNTIME_OPT"] = "-Os"
    report = {
        "modes": [],
        "environment": {
            "editor": str(EDITOR),
            "emulator": str(emulator_paths()[0]) if emulator_paths()[0] else None,
            "nugget": NUGGET.name,
        },
        "limitations": [
            "Measured in PCSX-Redux; not validated on physical hardware.",
            "Guest cycles are interpreter cycles between epok_lua_probe_begin and "
            "epok_lua_probe_end inside Guard:tick, including probe writes and any interrupt "
            "taken in the window; they are not host time and not a frame rate.",
            "Linked sizes are text+rodata+data+bss of this fixture project, not peak "
            "allocator, VRAM or SPU usage.",
            "A Blueprint child of a Lua class is covered by the Rust test "
            "src/lua_compile.rs::lua_classes_are_blueprint_parents_but_blueprint_classes_"
            "are_not_lua_parents: the CLI --new-blueprint resolves parents from the "
            "reflected C++ registry only, so a .epokbp with a Lua parent cannot be authored "
            "programmatically here.",
            "Corrupt-bytecode ABI rejection is covered by the Rust test "
            "src/lua_bytecode.rs::lua_bytecode_rejects_a_payload_whose_header_byte_drifted.",
        ],
    }

    keeper = contextlib.nullcontext(tempfile.mkdtemp(prefix="epok-lua-modes-")) if args.keep \
        else tempfile.TemporaryDirectory(prefix="epok-lua-modes-")
    with keeper as directory:
        directory = _Path(directory)
        if args.keep:
            print(f"Retained project directory: {directory}", flush=True)
        project = create_project(directory)
        report["project"] = str(project)

        entries = {}
        for mode in MODES:
            print(f"--- building {mode}", flush=True)
            entries[mode] = build_mode(project, mode, args, report)

        record(not entries["native_cpp"]["interpreter_symbols"],
               "native_cpp links no interpreter at all "
               f"({len(entries['native_cpp']['interpreter_symbols'])} lua_* symbols)")
        for mode in ("vm_bytecode", "vm_source"):
            record(bool(entries[mode]["interpreter_symbols"]),
                   f"{mode} links the PsyQo Lua core "
                   f"({len(entries[mode]['interpreter_symbols'])} lua_* symbols)")

        if not args.no_emulator:
            check_probes(report)
        check_identity(report)
        check_unsupported_diagnostics(project, report)
        check_mode_switch(project, report, entries["native_cpp"])
        if not args.skip_export:
            check_export(project, report)

    report["results"] = RESULTS
    report["passed"] = all(r["passed"] for r in RESULTS)
    output = _Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print("\n=== probe table (slot: expected | native_cpp | vm_bytecode | vm_source) ===")
    for row in report.get("probe", {}).get("table", []):
        expected = "mode-dependent" if row["expected"] is None else row["expected"]
        print(f"{row['slot']:>3} {row['name']:<22} {expected!s:>16} | "
              f"{row['native_cpp']:>12} | {row['vm_bytecode']:>12} | {row['vm_source']:>12}")
    print("\n=== measured in PCSX-Redux; not validated on physical hardware ===")
    for entry in report["modes"]:
        sections = entry["elf_sections"]
        cycles = entry.get("cycles") or {}
        print(f"{entry['mode']:<12} text={sections['text']:<8} rodata={sections['rodata']:<7} "
              f"data={sections['data']:<6} bss={sections['bss']:<7} "
              f"ps-exe={entry['ps_exe_bytes']:<8} "
              f"tick_cycles median={cycles.get('median', '-')!s:<10} p95={cycles.get('p95', '-')}")
    if report.get("probe"):
        for entry in report["modes"]:
            probe = report["probe"]["per_mode"][entry["mode"]]
            print(f"{entry['mode']:<12} lua arena peak={probe[58]} live={probe[59]} "
                  f"allocations={probe[60]} failures={probe[61]}")

    failed = [r["line"] for r in RESULTS if not r["passed"]]
    print(f"\n{len(RESULTS) - len(failed)}/{len(RESULTS)} checks passed. Report: {output}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
