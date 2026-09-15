#!/usr/bin/env python3
"""Verify the Lua VM bytecode ABI and the VM runtime's conformance on target.

Two independent claims are checked, both against the emulator rather than
against an assumption:

1.  ABI.  The host cooker (`native/lua/epok_ldump32.c`, the same code
    `src/lua_bytecode.rs` calls) and the pinned `luaU_dump` running ON TARGET
    are handed the same normalized chunk, and their output is compared byte for
    byte.  psxlua's `lua_Number` is `long` — 64-bit on this host, 32-bit on the
    PlayStation — so this is the check that the cooker writes what the console
    loads.

2.  Conformance.  `runtime/lua_runtime.hpp` is linked into a minimal PsyQo
    program twice: once with the chunk as source (parser archive) and once with
    the host-cooked bytecode (no-parser archive).  Both builds must produce
    identical probe values for the numeric edge cases of contract §3, a Bool
    round trip through the field accessors, a nested Lua -> C++ -> Lua call, and
    an absent slot that never enters the VM.

Nothing here was validated on physical hardware; linked section sizes are the
sizes of THIS smoke program, not of a game.

    python3 tests/integration/verify_lua_vm_abi.py --output report.json
    python3 tests/integration/verify_lua_vm_abi.py --no-emulator   # build only
"""
import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ROOT / "tests/integration/lua_vm_abi"
NUGGET = ROOT / "third_party/nugget"
PSXLUA = NUGGET / "third_party/psxlua"
PSXLUA_SRC = PSXLUA / "src"
NATIVE_LUA = ROOT / "native/lua"
PINNED_PSXLUA = "abed030e686b4e34987851bd0a028c93b1f73967"

PREFIX = "mipsel-none-elf"
PROBE_SLOTS = 64

# Mirrors tests/integration/lua_vm_abi/abi.hpp.
PROBE = {
    "magic": 0, "mode": 1, "done": 2, "load_ok": 3, "header_size": 4, "header": 5,
    "blob_size": 23, "iadd": 24, "idiv": 25, "fmul": 26, "ineg": 27, "ult": 28,
    "bool_before": 29, "bool_after": 30, "bool_field": 31, "nested": 32,
    "absent_bound": 33, "absent_entries": 34, "arena_peak": 35, "arena_live": 36,
    "arena_allocs": 37, "errors": 38,
}
MAGIC = 0x4C554142
DONE = 0x0000D09E

# `lua_bytecode::HEADER`. The target build reports the header it accepts and the
# driver decodes it; this constant is what the report is compared against.
HEADER = bytes([0x1B, 0x4C, 0x75, 0x61, 0x52, 0x00, 0x01, 0x04, 0x04, 0x04, 0x04, 0x01,
                0x19, 0x93, 0x0D, 0x0A, 0x1A, 0x0A])
# `lua_bytecode::KEEP_DEBUG_INFO` is true, so nothing is stripped by default.
STRIP = 0

# Expected conformance results. Each one is a consequence of contract §3 and is
# computed here rather than copied from a previous run.
EXPECTED = {
    "iadd": 0x7FFFFFFF,          # iadd(INT32_MAX, 1) saturates
    "idiv": 0,                   # idiv(7, 0) is zero, never a trap
    "fmul": (4097 * -1229) // 4096 + (1 if (4097 * -1229) % 4096 else 0),  # truncates toward zero
    "ineg": 0x7FFFFFFF,          # ineg(INT32_MIN) == INT32_MAX
    "ult": 1,                    # 0x80000000 > 1 as unsigned
    "bool_before": 0,
    "bool_after": 1,             # the chunk flipped a false field to true
    "bool_field": 1,
    "nested": 42,                # probe_call(41) -> __epok_call -> probe_iadd
    "absent_bound": 0,           # an unbound slot reports itself unbound
    "absent_entries": 0,         # and never enters Lua
    "errors": 0,                 # no arena allocation ever failed
}

FAILURES = []


def fail(message):
    FAILURES.append(message)
    print(f"FAIL: {message}", flush=True)


def sh(command, cwd=None, timeout=1800, check=True):
    result = subprocess.run([str(c) for c in command], cwd=cwd, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=timeout)
    if check and result.returncode != 0:
        raise RuntimeError(f"command failed: {command}\n{result.stdout}\n{result.stderr}")
    return result


# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------
def resolve_environment():
    env = {"platform": sys.platform, "toolchain": {}, "emulator": None, "bios": None}
    for name in ("gcc", "g++", "size", "nm", "ar"):
        path = shutil.which(f"{PREFIX}-{name}")
        env["toolchain"][name] = path
        if path is None:
            fail(f"missing toolchain binary {PREFIX}-{name}")
    for candidate in (
        ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/PCSX-Redux",
        ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/pcsx-redux",
        ROOT / ".tools/redux/pcsx-redux.exe",
        ROOT / ".tools/redux/pcsx-redux",
    ):
        if candidate.is_file():
            env["emulator"] = str(candidate)
            break
    if env["emulator"]:
        base = Path(env["emulator"]).parent
        for candidate in (base / "openbios.bin",
                          base.parent / "Resources/share/pcsx-redux/resources/openbios.bin"):
            if candidate.is_file():
                env["bios"] = str(candidate)
                break
        if env["bios"] is None:
            found = sorted((ROOT / ".tools").rglob("openbios.bin"))
            if found:
                env["bios"] = str(found[0])
    revision = sh(["git", "-C", str(PSXLUA), "rev-parse", "HEAD"], check=False).stdout.strip()
    env["revisions"] = {
        "psxlua": revision or None,
        "psxlua_pinned": PINNED_PSXLUA,
        "nugget": sh(["git", "-C", str(NUGGET), "rev-parse", "HEAD"], check=False).stdout.strip() or None,
    }
    if revision and revision != PINNED_PSXLUA:
        fail(f"psxlua is at {revision}, not the pinned {PINNED_PSXLUA}")
    if not PSXLUA_SRC.joinpath("lparser.c").is_file():
        fail("third_party/nugget/third_party/psxlua is not initialized; run the repository setup")
    return env


# ---------------------------------------------------------------------------
# Host cooker
# ---------------------------------------------------------------------------
# The same translation units build.rs compiles, plus the CLI front end. Keeping
# the list here would let it drift, so it is read back out of build.rs.
def host_units():
    text = (ROOT / "build.rs").read_text(encoding="utf-8")
    match = re.search(r"const UNITS: \[&str; \d+\] = \[(.*?)\];", text, re.S)
    if not match:
        raise RuntimeError("build.rs no longer declares the host cooker's UNITS list")
    return re.findall(r'"([a-z]+)"', match[1])


def build_host_cooker(build_dir):
    out = build_dir / "host_luac"
    command = ["cc", "-O1", "-w", "-DLUA_COMPAT_ALL", f"-I{PSXLUA_SRC}", f"-I{NATIVE_LUA}",
               str(SOURCES / "host_luac_main.c"), str(NATIVE_LUA / "epok_luac.c"),
               str(NATIVE_LUA / "epok_ldump32.c")]
    command += [str(PSXLUA_SRC / f"{unit}.c") for unit in host_units()]
    command += ["-o", str(out)]
    sh(command, timeout=900)
    return out


def cook_on_host(build_dir, chunk):
    tool = build_host_cooker(build_dir)
    source = build_dir / "abi_chunk.lua"
    source.write_text(chunk, encoding="utf-8")
    output = build_dir / "host.bin"
    result = sh([tool, "@abi_chunk.lua", source, output, STRIP], check=False)
    if result.returncode != 0:
        fail(f"host cooker rejected the chunk: {result.stderr.strip()}")
        return None
    return output.read_bytes()


# ---------------------------------------------------------------------------
# Target builds
# ---------------------------------------------------------------------------
LUA_UNITS_CORE = ["lapi", "lctype", "ldebug", "ldo", "lfunc", "lgc", "lmem", "lobject",
                  "lopcodes", "lstate", "lstring", "ltable", "ltm", "lundump", "lvm", "lzio",
                  "llibc", "lauxlib"]
LUA_UNITS_PARSER = ["lcode", "ldump", "llex", "lparser"]
LUA_UNITS_NOPARSER = ["lnoparser"]

ARCHFLAGS = ["-march=mips1", "-mabi=32", "-EL", "-fno-pic", "-mno-shared", "-mno-abicalls",
             "-mfp32", "-mno-llsc", "-fno-stack-protector", "-nostdlib", "-ffreestanding"]


def build_lua_archive(build_dir, variant):
    """The same recipe runtime/lua.mk uses, with per-variant object directories
    so the parser and no-parser builds can never share an object file."""
    obj_dir = build_dir / "lua" / variant
    obj_dir.mkdir(parents=True, exist_ok=True)
    units = LUA_UNITS_CORE + (LUA_UNITS_NOPARSER if variant == "noparser" else LUA_UNITS_PARSER)
    objects = []
    flags = ["-DLUA_TARGET_PSX", "-DLUA_COMPAT_ALL", "-Os", "-g", "-w", "-ffunction-sections",
             "-fdata-sections", "-mno-gpopt", "-fomit-frame-pointer", "-fno-builtin",
             "-fno-strict-aliasing", f"-I{PSXLUA_SRC}"] + ARCHFLAGS
    for unit in units:
        obj = obj_dir / f"{unit}.o"
        sh([f"{PREFIX}-gcc"] + flags + ["-c", "-o", str(obj), str(PSXLUA_SRC / f"{unit}.c")])
        objects.append(str(obj))
    archive = build_dir / "lua" / f"liblua-epok-{variant}.a"
    if archive.exists():
        archive.unlink()
    sh([f"{PREFIX}-gcc-ar", "rcs", str(archive)] + objects)
    return archive


MAKEFILE = """# Generated by tests/integration/verify_lua_vm_abi.py. Do not edit.
TARGET = {target}
TYPE = ps-exe
SRCS = abi_common.cpp {main}{extra}
CXXFLAGS = -std=c++20
CPPFLAGS += -I. -I{runtime} -I{psxlua} -DLUA_TARGET_PSX -Wno-attributes
LIBRARIES += {archive}
include {psyqo_mk}
# MIPS has no 64-bit divide; the saturating helpers use GCC's integer routines.
LIBRARIES += $(shell $(CC) -print-libgcc-file-name)
"""


def build_target(build_dir, name, main, archive, extra_sources, defines, generated):
    directory = build_dir / name
    directory.mkdir(parents=True, exist_ok=True)
    for source in ["abi.hpp", "abi_common.cpp", main] + extra_sources:
        shutil.copy2(SOURCES / source, directory / source)
    for filename, text in generated.items():
        (directory / filename).write_text(text, encoding="utf-8")
    (directory / "display.hh").write_text(
        "#pragma once\nnamespace epok {inline constexpr int display_width=320,display_height=240;}\n",
        encoding="utf-8")
    makefile = MAKEFILE.format(
        target=name.replace("-", "_"), main=main,
        extra="".join(f" {s}" for s in extra_sources if s.endswith((".c", ".cpp"))),
        runtime=ROOT / "runtime", psxlua=PSXLUA_SRC, archive=archive,
        psyqo_mk=NUGGET / "psyqo/psyqo.mk")
    makefile += "".join(f"CPPFLAGS += -D{d}\n" for d in defines)
    (directory / "Makefile").write_text(makefile, encoding="utf-8")
    log = sh(["make", "-j4", "BUILD=Release"], cwd=directory, check=False)
    (directory / "build.log").write_text(log.stdout + log.stderr, encoding="utf-8")
    target = name.replace("-", "_")
    elf, exe = directory / f"{target}.elf", directory / f"{target}.ps-exe"
    if log.returncode != 0 or not elf.is_file():
        fail(f"{name} did not build")
        return {"built": False, "error": (log.stdout + log.stderr)[-4000:]}
    return {"built": True, "elf": str(elf), "ps_exe": str(exe),
            "sections": section_sizes(elf), "symbols": symbols(elf)}


def section_sizes(elf):
    out = sh([shutil.which(f"{PREFIX}-size"), "-A", str(elf)]).stdout
    sections = {}
    for line in out.splitlines():
        match = re.match(r"^(\.\S+)\s+(\d+)\s+", line)
        if match:
            sections[match[1]] = int(match[2])
    text = sections.get(".text", 0)
    rodata = sections.get(".rodata", 0) + sections.get(".rodata1", 0)
    data = sections.get(".data", 0) + sections.get(".sdata", 0)
    bss = sections.get(".sbss", 0) + sections.get(".bss", 0)
    return {"text": text, "rodata": rodata, "data": data, "bss": bss,
            "linked_resident_bytes": text + rodata + data + bss}


def symbols(elf):
    out = sh([shutil.which(f"{PREFIX}-nm"), "--defined-only", str(elf)]).stdout
    table = {}
    for line in out.splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+\S\s+(\S+)$", line)
        if match:
            table[match[2]] = int(match[1], 16)
    return table


# ---------------------------------------------------------------------------
# Emulator
# ---------------------------------------------------------------------------
DRIVER = r"""local ffi = require('ffi')
local mem = PCSX.getMemPtr()
local probe = ffi.cast('int32_t*', mem + %(probe)d)
local blob = ffi.cast('uint8_t*', mem + %(blob)d)
local blobsize = ffi.cast('int32_t*', mem + %(blobsize)d)
local finished = false

local function finish(reason)
  if finished then return end
  finished = true
  local f = assert(io.open(%(output)s, 'w'))
  f:write('{"reason":"' .. reason .. '","probe":[')
  for i = 0, %(slots)d - 1 do
    if i > 0 then f:write(',') end
    f:write(tostring(tonumber(probe[i])))
  end
  f:write('],"bytecode":"')
  local n = tonumber(blobsize[0])
  if n == nil or n < 0 or n > %(blobmax)d then n = 0 end
  local parts = {}
  for i = 0, n - 1 do parts[#parts + 1] = string.format('%%02x', blob[i]) end
  f:write(table.concat(parts))
  f:write('"}')
  f:close()
  PCSX.quit(0)
end

abi_finished = PCSX.addBreakpoint(%(done)d, 'Exec', 4, 'abi done', function()
  finish('done')
  return true
end)
"""


def run_emulator(env, build, workdir, timeout):
    workdir.mkdir(parents=True, exist_ok=True)
    output = workdir / "probe.json"
    if output.exists():
        output.unlink()
    table = build["symbols"]
    missing = [s for s in ("abi_done", "abi_probe") if s not in table]
    if missing:
        return {"ok": False, "error": f"missing symbols: {missing}"}
    # --gc-sections drops the staging buffer from the smoke builds, which never
    # write it; point the reader at the probe so it reads a zero length.
    blob = table.get("abi_bytecode", table["abi_probe"])
    blob_size = table.get("abi_bytecode_size", table["abi_probe"] + 4 * PROBE["blob_size"])
    script = workdir / "driver.lua"
    script.write_text(DRIVER % {
        "probe": table["abi_probe"] & 0x1FFFFF,
        "blob": blob & 0x1FFFFF,
        "blobsize": blob_size & 0x1FFFFF,
        "output": json.dumps(str(output)),
        "slots": PROBE_SLOTS,
        "blobmax": 16384,
        "done": table["abi_done"],
    }, encoding="utf-8")
    portable = workdir / "portable"
    portable.mkdir(exist_ok=True)
    command = [env["emulator"], "-portable", str(portable), "-run", "-stdout", "-interpreter",
               "-softgpu", "-2mb", "-no-gdb", "-loadexe", build["ps_exe"],
               "-bios", env["bios"], "-dofile", str(script), "-debugger"]
    with (workdir / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=portable, stdout=log, stderr=log,
                                   stdin=subprocess.DEVNULL)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=30)
            return {"ok": False, "error": f"emulator timed out after {timeout}s",
                    "log": str(workdir / "emulator.log")}
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    if not output.is_file():
        return {"ok": False, "error": f"emulator produced no probe (exit {code})",
                "log": str(workdir / "emulator.log")}
    return {"ok": True, "data": json.loads(output.read_text(encoding="utf-8"))}


def as_bytes(blob):
    return bytes.fromhex(blob) if blob else b""


def decode_header(probe):
    size = probe[PROBE["header_size"]]
    if size <= 0 or size > 18:
        return None
    return bytes(probe[PROBE["header"] + i] & 0xFF for i in range(size))


def blob_header_source(name, data):
    rows = ["    " + ", ".join(f"0x{b:02x}" for b in data[i:i + 16]) + ","
            for i in range(0, len(data), 16)]
    return ("// Generated by tests/integration/verify_lua_vm_abi.py from the HOST cooker.\n"
            "// Do not edit; regenerate by re-running the verifier.\n"
            "#pragma once\n\n"
            f"static const unsigned char {name}[] = {{\n" + "\n".join(rows) + "\n};\n")


def lua_config(mode):
    """The header src/settings.rs generates into the staging directory."""
    return ("#pragma once\n#define EPOK_LUA_MODE_NATIVE 0\n#define EPOK_LUA_MODE_VM_BYTECODE 1\n"
            f"#define EPOK_LUA_MODE_VM_SOURCE 2\n#define EPOK_LUA_MODE {mode}\n")


def chunk_header_source(chunk):
    escaped = chunk.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return ("// Generated by tests/integration/verify_lua_vm_abi.py from\n"
            "// tests/integration/lua_vm_abi/abi_chunk.lua. Do not edit.\n"
            "#pragma once\n\n"
            f"#define ABI_DUMP_STRIP {STRIP}\n"
            f'static const char ABI_CHUNK[] = "{escaped}";\n')


# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", default=None)
    parser.add_argument("--output", default=None, help="write the JSON report here")
    parser.add_argument("--no-emulator", action="store_true",
                        help="build and cook only; skips both on-target checks")
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()

    build_dir = Path(args.build_dir) if args.build_dir else ROOT / ".epok/lua-vm-abi"
    build_dir.mkdir(parents=True, exist_ok=True)
    report = {"environment": resolve_environment(), "expected": EXPECTED}
    env = report["environment"]
    if FAILURES:
        finish(report, args)
        return 1

    chunk = (SOURCES / "abi_chunk.lua").read_text(encoding="utf-8")
    report["chunk_bytes"] = len(chunk)

    host = cook_on_host(build_dir, chunk)
    report["host_cook"] = {
        "bytes": len(host) if host else None,
        "header_matches_pinned": bool(host) and host[:18] == HEADER,
        "strip": STRIP,
    }
    if host and host[:18] != HEADER:
        fail("the host cooker did not emit the pinned psxlua header")
    if host:
        stripped = build_dir / "host_stripped.bin"
        tool = build_dir / "host_luac"
        sh([tool, "@abi_chunk.lua", build_dir / "abi_chunk.lua", stripped, 1], check=False)
        if stripped.is_file():
            report["host_cook"]["stripped_bytes"] = stripped.stat().st_size
            report["host_cook"]["debug_info_cost_bytes"] = len(host) - stripped.stat().st_size

    generated = {"abi_chunk.h": chunk_header_source(chunk)}
    parser_archive = build_lua_archive(build_dir, "parser")
    noparser_archive = build_lua_archive(build_dir, "noparser")
    report["archives"] = {"parser": parser_archive.stat().st_size,
                          "noparser": noparser_archive.stat().st_size}

    builds = {}
    builds["dump"] = build_target(build_dir, "dump", "dump_main.cpp", parser_archive,
                                  ["abi_dump.c"], [], generated)
    source_generated = dict(generated)
    source_generated["lua-config.hh"] = lua_config(2)
    builds["smoke-source"] = build_target(build_dir, "smoke-source", "smoke_main.cpp",
                                          parser_archive, [], [], source_generated)
    report["builds"] = {k: {kk: vv for kk, vv in v.items() if kk != "symbols"}
                        for k, v in builds.items()}

    if args.no_emulator or not env["emulator"] or not env["bios"]:
        if not args.no_emulator:
            fail("PCSX-Redux or its BIOS is unavailable; the on-target checks did not run")
        finish(report, args)
        return 1 if FAILURES else 0

    # --- 1. ABI ------------------------------------------------------------
    if builds["dump"]["built"]:
        run = run_emulator(env, builds["dump"], build_dir / "run-dump", args.timeout)
        if not run["ok"]:
            fail(f"dump build did not run: {run['error']}")
        else:
            probe = run["data"]["probe"]
            target = as_bytes(run["data"]["bytecode"])
            accepted = decode_header(probe)
            report["abi"] = {
                "target_bytes": len(target),
                "host_bytes": len(host) if host else None,
                "target_header": accepted.hex() if accepted else None,
                "pinned_header": HEADER.hex(),
                "header_matches": accepted == HEADER,
                "identical": bool(host) and target == host,
                "load_ok": probe[PROBE["load_ok"]] == 1,
            }
            if probe[PROBE["done"]] != DONE:
                fail("the dump build did not reach its completion marker")
            if accepted != HEADER:
                fail(f"target accepts header {accepted.hex() if accepted else None}, "
                     f"lua_bytecode::HEADER is {HEADER.hex()}")
            if not host:
                pass
            elif target != host:
                first = next((i for i, (a, b) in enumerate(zip(target, host)) if a != b),
                             min(len(target), len(host)))
                report["abi"]["first_difference"] = first
                fail(f"host-cooked bytecode differs from the target dump at byte {first} "
                     f"({len(host)} host bytes vs {len(target)} target bytes)")

    # --- 2. Conformance ----------------------------------------------------
    if host:
        generated_bytecode = dict(generated)
        generated_bytecode["lua-config.hh"] = lua_config(1)
        generated_bytecode["abi_bytecode.h"] = blob_header_source("ABI_BYTECODE", host)
        builds["smoke-bytecode"] = build_target(build_dir, "smoke-bytecode", "smoke_main.cpp",
                                                noparser_archive, [], [], generated_bytecode)
        report["builds"]["smoke-bytecode"] = {
            k: v for k, v in builds["smoke-bytecode"].items() if k != "symbols"}

    results = {}
    for name in ("smoke-source", "smoke-bytecode"):
        build = builds.get(name)
        if not build or not build.get("built"):
            continue
        run = run_emulator(env, build, build_dir / f"run-{name}", args.timeout)
        if not run["ok"]:
            fail(f"{name} did not run: {run['error']}")
            continue
        probe = run["data"]["probe"]
        if probe[PROBE["magic"]] != MAGIC or probe[PROBE["done"]] != DONE:
            fail(f"{name} did not reach its completion marker")
            continue
        measured = {key: probe[PROBE[key]] for key in EXPECTED}
        measured["arena_peak"] = probe[PROBE["arena_peak"]]
        measured["arena_live"] = probe[PROBE["arena_live"]]
        measured["arena_allocations"] = probe[PROBE["arena_allocs"]]
        results[name] = measured
        for key, want in EXPECTED.items():
            if measured[key] != want:
                fail(f"{name}: {key} is {measured[key]}, expected {want}")
    report["conformance"] = results
    if len(results) == 2:
        a, b = results["smoke-source"], results["smoke-bytecode"]
        differing = [k for k in EXPECTED if a[k] != b[k]]
        report["modes_agree"] = not differing
        if differing:
            fail(f"source and bytecode packagings disagree on {differing}")

    finish(report, args)
    return 1 if FAILURES else 0


def finish(report, args):
    report["failures"] = FAILURES
    report["ok"] = not FAILURES
    report["caveats"] = [
        "Nothing here was validated on physical PlayStation hardware.",
        "Linked section sizes are this smoke program's, not a game's.",
        "Arena figures cover Lua allocations inside EPOK_LUA_ARENA_BYTES only.",
    ]
    text = json.dumps(report, indent=2, sort_keys=True)
    if args.output:
        Path(args.output).write_text(text, encoding="utf-8")
    print(text)


if __name__ == "__main__":
    sys.exit(main())
