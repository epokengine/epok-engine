"""Matched native-vs-VM Lua feasibility harness for Epok's PSX runtime.

Builds four isolated PsyQo executables that run identical workloads with
identical numeric semantics, then measures guest CPU cycles between identical
markers under headless PCSX-Redux.

  native              handwritten C++ matching Epok's proposed AOT lowering
  vm-parser           PsyQo Lua + liblua.a, chunks loaded from source
  vm-noparser         PsyQo Lua + liblua-noparser.a, same chunks as bytecode
  vm-cached-dispatch  as vm-noparser plus two documented
                      integration choices (numeric-id handler cache + event
                      bitmask, and per-instance chunk re-load for isolation)

This is NOT the Epok renderer or editor: it links no engine code at all.

Flags:
  --emulator        also measure guest CPU cycles (otherwise build + size only)
  --output PATH     JSON report destination
  --build-dir PATH  where to stage the toolchain copy and the four builds
  --keep            keep the build directory

Reproduction and caveats: tests/integration/lua_feasibility/README.md
"""
import argparse
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HARNESS = ROOT / "tests/integration/lua_feasibility"
NUGGET = ROOT / "third_party/nugget"
PSXLUA = NUGGET / "third_party/psxlua"

PREFIX = "mipsel-none-elf"
WORKLOADS = ["absent", "empty", "arith", "native_call", "position", "alloc", "event_only"]
INSTANCE_COUNTS = [1, 16, 32, 64]
WARMUP_STEPS = 6
SAMPLE_STEPS = 60
PROBE_SLOTS = 512

# Mirrors tests/integration/lua_feasibility/harness.hpp.
PROBE = {
    "magic": 0, "variant": 1, "cases": 2, "done": 3, "header_size": 4, "header": 5,
    "blob_size": 23, "chunk_size": 24, "chunk_off": 32, "chunks": 40,
    "heap_retained": 41, "heap_peak": 42, "alloc_count": 43, "alloc_bytes": 44,
    "free_count": 45, "stack_high": 46, "load_ok": 47, "realloc_count": 48,
    "checksum": 64, "calls": 128, "case_retained": 192, "case_peak": 256,
    "case_allocs": 320, "case_alloc_bytes": 384,
}
MAGIC = 0x4C554146
DONE = 0x0000D09E
LUAC_HEADER_BYTES = 18

VARIANTS = [
    # name, variant id, main source, lua archive (None = no VM)
    ("native", 0, "native_main.cpp", None),
    ("vm-parser", 1, "vm_main.cpp", "liblua.a"),
    ("vm-noparser", 2, "vm_main.cpp", "liblua-noparser.a"),
    ("vm-cached-dispatch", 3, "vm_main.cpp", "liblua-noparser.a"),
]

FAILURES = []


def fail(message):
    FAILURES.append(message)
    print(f"FAIL: {message}", flush=True)


def sh(command, cwd=None, timeout=1800, check=True, env=None):
    result = subprocess.run([str(c) for c in command], cwd=cwd, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=timeout, env=env)
    if check and result.returncode != 0:
        raise RuntimeError(f"command failed: {command}\n{result.stdout}\n{result.stderr}")
    return result


def case_index(workload, slot):
    return workload * len(INSTANCE_COUNTS) + slot


# --------------------------------------------------------------------------
# Environment
# --------------------------------------------------------------------------
def resolve_environment():
    env = {"platform": sys.platform, "toolchain": {}, "emulator": None, "bios": None}
    for name in ("gcc", "g++", "size", "nm", "ar"):
        path = shutil.which(f"{PREFIX}-{name}")
        env["toolchain"][name] = path
        if path is None:
            fail(f"missing toolchain binary {PREFIX}-{name}")
    if env["toolchain"]["gcc"]:
        env["gcc_version"] = sh([env["toolchain"]["gcc"], "--version"]).stdout.splitlines()[0]
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
    env["revisions"] = {
        "nugget": sh(["git", "-C", str(NUGGET), "rev-parse", "HEAD"], check=False).stdout.strip() or None,
        "psxlua": sh(["git", "-C", str(PSXLUA), "rev-parse", "HEAD"], check=False).stdout.strip() or None,
        "epok": sh(["git", "-C", str(ROOT), "rev-parse", "HEAD"], check=False).stdout.strip() or None,
    }
    return env


# --------------------------------------------------------------------------
# Build staging. The pinned Nugget/psxlua tree is copied so that nothing in the
# repository is written to; every object file lands in the build directory.
# --------------------------------------------------------------------------
def stage(build_dir):
    copy = build_dir / "nugget"
    if not (copy / "psyqo/psyqo.mk").is_file():
        shutil.copytree(NUGGET, copy, ignore=shutil.ignore_patterns(".git", "*.o", "*.a", "*.dep", "*.elf"),
                        symlinks=True)
    return copy


def build_lua_archives(nugget_copy, lib_dir):
    """psx and psx-noparser share object files, so they must not be built
    concurrently or without an intervening clean."""
    lib_dir.mkdir(parents=True, exist_ok=True)
    src = nugget_copy / "third_party/psxlua/src"
    results = {}
    for target, archive in (("psx", "liblua.a"), ("psx-noparser", "liblua-noparser.a")):
        sh(["make", "clean"], cwd=nugget_copy / "third_party/psxlua")
        sh(["make", target, "-j4"], cwd=nugget_copy / "third_party/psxlua")
        produced = src / archive
        if not produced.is_file():
            fail(f"psxlua target {target} did not produce {archive}")
            continue
        shutil.copy2(produced, lib_dir / archive)
        results[archive] = (lib_dir / archive).stat().st_size
    sh(["make", "clean"], cwd=nugget_copy / "third_party/psxlua")
    return results


MAKEFILE = """# Generated by tests/integration/verify_lua_feasibility.py. Do not edit.
TARGET = {target}
TYPE = ps-exe
SRCS = harness_common.cpp {main}{extra}
CXXFLAGS = -std=c++20
CPPFLAGS += -DHARNESS_VARIANT={variant}
{lua}
include {psyqo_mk}
"""

LUA_FRAGMENT = """CPPFLAGS += -I{psxlua_src} -DLUA_TARGET_PSX
LIBRARIES += {archive}
"""


def build_variant(build_dir, nugget_copy, lib_dir, name, variant, main, archive, blob_header):
    directory = build_dir / name
    directory.mkdir(parents=True, exist_ok=True)
    extra = " harness_dump.c" if variant == 1 else ""
    sources = ["harness.hpp", "q12.hpp", "workloads_lua.hpp", "harness_common.cpp", main]
    if variant == 1:
        sources.append("harness_dump.c")
    for source in sources:
        shutil.copy2(HARNESS / source, directory / source)
    if blob_header is not None:
        (directory / "bytecode_blob.h").write_text(blob_header, encoding="utf-8")
    lua = ""
    if archive:
        lua = LUA_FRAGMENT.format(psxlua_src=nugget_copy / "third_party/psxlua/src",
                                  archive=lib_dir / archive)
    (directory / "Makefile").write_text(MAKEFILE.format(
        target=name.replace("-", "_"), main=main, extra=extra, variant=variant, lua=lua,
        psyqo_mk=nugget_copy / "psyqo/psyqo.mk"), encoding="utf-8")
    log = sh(["make", "-j4"], cwd=directory, check=False)
    (directory / "build.log").write_text(log.stdout + log.stderr, encoding="utf-8")
    elf = directory / f"{name.replace('-', '_')}.elf"
    exe = directory / f"{name.replace('-', '_')}.ps-exe"
    if log.returncode != 0 or not elf.is_file():
        return {"built": False, "error": (log.stdout + log.stderr)[-4000:]}
    table, sizes = symbols(elf)
    sections = section_sizes(elf)
    cooker = sizes.get("harness_bytecode", 0)
    if cooker:
        sections["bss_excluding_bytecode_cooker_buffer"] = sections["bss"] - cooker
        sections["bytecode_cooker_buffer_bytes"] = cooker
        sections["bss_note"] = ("This build stages dumped bytecode in a BSS buffer so the harness "
                                "can cook it on target. A shipping build would not carry it.")
    return {"built": True, "elf": str(elf), "ps_exe": str(exe),
            "ps_exe_bytes": exe.stat().st_size if exe.is_file() else None,
            "sections": sections, "symbols": table, "symbol_sizes": sizes}


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
    return {"all": sections, "text": text, "rodata": rodata, "data": data, "bss": bss,
            "linked_resident_bytes": text + rodata + data + bss}


def symbols(elf):
    out = sh([shutil.which(f"{PREFIX}-nm"), "-S", "--defined-only", str(elf)]).stdout
    table, sizes = {}, {}
    for line in out.splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+(?:([0-9a-fA-F]+)\s+)?\S\s+(\S+)$", line)
        if match:
            table[match[3]] = int(match[1], 16)
            if match[2]:
                sizes[match[3]] = int(match[2], 16)
    return table, sizes


# --------------------------------------------------------------------------
# Emulator
# --------------------------------------------------------------------------
DRIVER = r"""local ffi = require('ffi')
local mem = PCSX.getMemPtr()
local probe = ffi.cast('int32_t*', mem + %(probe)d)
local blob = ffi.cast('uint8_t*', mem + %(blob)d)
local blobsize = ffi.cast('int32_t*', mem + %(blobsize)d)
local markid = ffi.cast('int32_t*', mem + %(markid)d)
local started = nil
local buckets = {}
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
  local n = 0
  if %(hasblob)d == 1 then n = tonumber(blobsize[0]) end
  if n == nil or n < 0 or n > %(blobmax)d then n = 0 end
  local parts = {}
  for i = 0, n - 1 do parts[#parts + 1] = string.format('%%02x', blob[i]) end
  f:write(table.concat(parts))
  f:write('","cycles":{')
  local first = true
  for id, list in pairs(buckets) do
    if not first then f:write(',') end
    first = false
    f:write('"' .. id .. '":[')
    for i, v in ipairs(list) do
      if i > 1 then f:write(',') end
      f:write(tostring(v))
    end
    f:write(']')
  end
  f:write('}}')
  f:close()
  PCSX.quit(0)
end

harness_begin = PCSX.addBreakpoint(%(begin)d, 'Exec', 4, 'harness begin', function()
  if finished then return true end
  started = tonumber(PCSX.getCPUCycles())
  return true
end)

harness_end = PCSX.addBreakpoint(%(finish)d, 'Exec', 4, 'harness end', function()
  if finished then return true end
  if started == nil then return true end
  local elapsed = tonumber(PCSX.getCPUCycles()) - started
  started = nil
  local id = tonumber(markid[0])
  local key = tostring(id)
  local bucket = buckets[key]
  if bucket == nil then bucket = {} buckets[key] = bucket end
  bucket[#bucket + 1] = elapsed
  if %(dumponly)d == 1 and id == 3901 then finish('dump') end
  return true
end)

harness_finished = PCSX.addBreakpoint(%(done)d, 'Exec', 4, 'harness done', function()
  finish('done')
  return true
end)
"""


def run_emulator(env, build, workdir, dump_only, timeout):
    workdir.mkdir(parents=True, exist_ok=True)
    output = workdir / "measurements.json"
    if output.exists():
        output.unlink()
    table = build["symbols"]
    missing = [s for s in ("harness_mark_begin", "harness_mark_end", "harness_done",
                           "harness_mark_id", "harness_probe") if s not in table]
    if missing:
        return {"ok": False, "error": f"missing symbols: {missing}"}
    blob = table.get("harness_bytecode", table["harness_probe"])
    blobsize = table.get("harness_bytecode_size", table["harness_probe"])
    script = workdir / "driver.lua"
    script.write_text(DRIVER % {
        "probe": table["harness_probe"] & 0x1FFFFF,
        "blob": blob & 0x1FFFFF,
        "blobsize": blobsize & 0x1FFFFF,
        "markid": table["harness_mark_id"] & 0x1FFFFF,
        "output": json.dumps(str(output)),
        "slots": PROBE_SLOTS,
        "hasblob": 1 if "harness_bytecode" in table else 0,
        "blobmax": 24576,
        "begin": table["harness_mark_begin"],
        "finish": table["harness_mark_end"],
        "done": table["harness_done"],
        "dumponly": 1 if dump_only else 0,
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
            return {"ok": False, "error": f"emulator timed out after {timeout}s", "log": str(workdir / 'emulator.log')}
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    if not output.is_file():
        return {"ok": False, "error": f"emulator produced no measurements (exit {code})",
                "log": str(workdir / "emulator.log")}
    data = json.loads(output.read_text(encoding="utf-8"))
    return {"ok": True, "data": data}


def statistics_for(samples):
    if not samples:
        return None
    ordered = sorted(samples)
    return {"samples": len(ordered), "min": ordered[0], "median": statistics.median(ordered),
            "p95": ordered[min(len(ordered) - 1, (len(ordered) * 95 - 1) // 100)],
            "max": ordered[-1], "sum": sum(ordered)}


# --------------------------------------------------------------------------
# Bytecode ABI
# --------------------------------------------------------------------------
def blob_header_source(blob, offsets, sizes):
    rows = []
    for i in range(0, len(blob), 16):
        rows.append("    " + ", ".join(f"0x{b:02x}" for b in blob[i:i + 16]) + ",")
    return ("// Generated by tests/integration/verify_lua_feasibility.py from bytecode\n"
            "// produced on-target by the vm-parser build under PCSX-Redux.\n"
            "// Do not edit; regenerate by re-running the harness.\n"
            "#pragma once\n\n"
            "static const unsigned char BYTECODE_BLOB[] = {\n" + "\n".join(rows) + "\n};\n\n"
            f"static const int BYTECODE_OFFSET[{len(offsets)}] = {{{', '.join(map(str, offsets))}}};\n"
            f"static const int BYTECODE_SIZE[{len(sizes)}] = {{{', '.join(map(str, sizes))}}};\n")


def try_host_luac(nugget_copy, out_dir):
    """The docs warn a desktop luac may not match the PSX bytecode ABI. Record
    what actually happens on this machine instead of assuming."""
    result = sh(["make", "clean"], cwd=nugget_copy / "third_party/psxlua", check=False)
    build = sh(["make", "host32"], cwd=nugget_copy / "third_party/psxlua", check=False, timeout=600)
    sh(["make", "clean"], cwd=nugget_copy / "third_party/psxlua", check=False)
    ok = build.returncode == 0
    return {"attempted": True, "succeeded": ok,
            "reason": None if ok else (build.stdout + build.stderr)[-1200:].strip(),
            "note": "A host luac was not used for this report; bytecode was produced on-target."}


# --------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    parser.add_argument("--output", default=str(ROOT / "artifacts/lua_feasibility/report.json"))
    parser.add_argument("--build-dir", default=None)
    parser.add_argument("--keep", action="store_true")
    parser.add_argument("--timeout", type=int, default=1800)
    args = parser.parse_args()

    build_dir = Path(args.build_dir) if args.build_dir else Path(
        tempfile.mkdtemp(prefix="epok-lua-feasibility-"))
    build_dir.mkdir(parents=True, exist_ok=True)
    print(f"Build directory: {build_dir}", flush=True)

    env = resolve_environment()
    report = {
        "harness": "lua_feasibility",
        "what_this_is": ("Isolated PsyQo feasibility harness. It links no Epok engine code; it "
                         "is not the Epok renderer or editor and not a game."),
        "environment": env,
        "configuration": {
            "workloads": WORKLOADS, "instance_counts": INSTANCE_COUNTS,
            "warmup_steps": WARMUP_STEPS, "sample_steps": SAMPLE_STEPS,
            "step_rate_hz": 60,
            "delta_sequence": ("Q12 seconds, raw 68/69 alternating exactly as runtime/time.hpp "
                               "Time::begin_tick(); generated by identical native code in all "
                               "four variants."),
            "optimization": "BUILD=Release, -Os, identical for all four variants",
            "numeric_contract": ("q12.hpp mirrors runtime/blueprint_runtime.hpp epok::bp "
                                 "(saturating int64 intermediates, div-by-zero to zero, "
                                 "Q12 raw int32 with 4096 == 1.0)."),
        },
        "cpu_measured": bool(args.emulator),
        "hardware_validated": False,
        "variants": {},
        "bytecode_abi": {},
        "comparison": {},
        "limitations": [
            "Guest CPU cycles are PCSX-Redux interpreter cycles between identical markers; "
            "they are not host time, not frame time and not FPS.",
            "No result here was validated on physical PlayStation hardware.",
            "Linked section sizes are .text/.rodata/.data/.bss of the linked executable. "
            "They are not archive sizes and not total RAM consumption.",
            "Heap figures for the VM variants come from the harness's own lua_Alloc, so they "
            "cover Lua allocations only, not the psyqo allocator's own block headers or "
            "fragmentation.",
            "The measured window includes the tail of the begin marker and the mark-id store; "
            "that offset is identical in every variant.",
            "Lua chunks here use raw Q12 integers, not psyqo FixedPoint tables. A FixedPoint "
            "based binding would allocate substantially more; this harness measures the "
            "cheaper representation.",
            "No speedup multiplier, FPS figure or RAM saving is reported that was not measured "
            "by this run.",
        ],
    }

    if not all(env["toolchain"].values()):
        report["passed"] = False
        report["failures"] = FAILURES
        write_report(args.output, report)
        print("FAIL: toolchain unavailable", flush=True)
        return 1

    nugget_copy = stage(build_dir)
    lib_dir = build_dir / "lib"
    print("Building psxlua archives (parser and no-parser)...", flush=True)
    report["archives"] = build_lua_archives(nugget_copy, lib_dir)
    report["archives_note"] = ("Archive byte sizes on disk. These are NOT RAM consumption and "
                               "NOT linked sizes; see variants[*].sections.")

    builds = {}
    # 1. native and vm-parser first: the parser build is also the bytecode cooker.
    for name, variant, main, archive in VARIANTS[:2]:
        print(f"Building {name}...", flush=True)
        builds[name] = build_variant(build_dir, nugget_copy, lib_dir, name, variant, main, archive, None)
        if not builds[name]["built"]:
            fail(f"{name} failed to build")

    # 2. Produce bytecode on-target, on the target VM itself, then
    #    check the header the no-parser core will accept.
    abi = report["bytecode_abi"]
    abi["route"] = ("Bytecode is produced on-target: the vm-parser build runs under PCSX-Redux "
                    "and lua_dump()s each chunk (strip=1) into a guest buffer that the driver "
                    "reads back. No host luac is involved, so there is no host/target ABI gap "
                    "to bridge.")
    abi["host_luac"] = try_host_luac(nugget_copy, build_dir)
    blob = None
    if env["emulator"] is None or env["bios"] is None:
        fail("PCSX-Redux or openbios.bin not found; bytecode cannot be produced on-target "
             "and the no-parser variants cannot be built")
        abi["produced"] = False
    elif not builds["vm-parser"]["built"]:
        abi["produced"] = False
    else:
        print("Producing bytecode on-target (vm-parser under PCSX-Redux)...", flush=True)
        run = run_emulator(env, builds["vm-parser"], build_dir / "cook", dump_only=True, timeout=300)
        if not run["ok"]:
            fail(f"bytecode production failed: {run['error']}")
            abi["produced"] = False
        else:
            probe = run["data"]["probe"]
            blob = bytes.fromhex(run["data"]["bytecode"])
            sizes = probe[PROBE["chunk_size"]:PROBE["chunk_size"] + len(WORKLOADS)]
            offsets = probe[PROBE["chunk_off"]:PROBE["chunk_off"] + len(WORKLOADS)]
            parser_header = bytes(probe[PROBE["header"]:PROBE["header"] + LUAC_HEADER_BYTES])
            abi.update({
                "produced": True,
                "blob_bytes": len(blob),
                "chunk_bytes": dict(zip(WORKLOADS, sizes)),
                "chunk_offsets": dict(zip(WORKLOADS, offsets)),
                "parser_build_accepts_header": parser_header.hex(),
                "parser_build_header_fields": decode_header(parser_header),
            })
            (build_dir / "bytecode.bin").write_bytes(blob)
            abi["blob_path"] = str(build_dir / "bytecode.bin")
            # Every chunk must start with the exact header this core emits.
            mismatched = []
            for name, off, size in zip(WORKLOADS, offsets, sizes):
                if size <= 0:
                    mismatched.append(f"{name}: empty dump")
                    continue
                chunk_header = blob[off:off + LUAC_HEADER_BYTES]
                if chunk_header != parser_header:
                    mismatched.append(f"{name}: {chunk_header.hex()} != {parser_header.hex()}")
            abi["chunk_header_mismatches"] = mismatched
            if mismatched:
                fail(f"dumped bytecode headers disagree with luaU_header(): {mismatched}")
            if sum(sizes) != len(blob):
                fail(f"bytecode blob size {len(blob)} != sum of chunk sizes {sum(sizes)}")

    # 3. Build the two no-parser variants against that bytecode.
    header_source = None
    if blob:
        header_source = blob_header_source(blob, offsets, sizes)
        (build_dir / "bytecode_blob.h").write_text(header_source, encoding="utf-8")
    for name, variant, main, archive in VARIANTS[2:]:
        if header_source is None:
            builds[name] = {"built": False, "error": "no on-target bytecode available"}
            fail(f"{name} not built: no on-target bytecode available")
            continue
        print(f"Building {name}...", flush=True)
        builds[name] = build_variant(build_dir, nugget_copy, lib_dir, name, variant, main,
                                     archive, header_source)
        if not builds[name]["built"]:
            fail(f"{name} failed to build")

    # 4. Measure.
    for name, variant, main, archive in VARIANTS:
        build = builds[name]
        entry = {"built": build["built"], "variant_id": variant,
                 "lua_archive": archive, "main_source": main}
        if build["built"]:
            entry["sections"] = build["sections"]
            entry["ps_exe_bytes"] = build["ps_exe_bytes"]
        else:
            entry["build_error"] = build.get("error")
        report["variants"][name] = entry

    if args.emulator:
        if env["emulator"] is None or env["bios"] is None:
            fail("--emulator requested but PCSX-Redux or openbios.bin is unavailable")
        else:
            for name, variant, main, archive in VARIANTS:
                build = builds[name]
                if not build["built"]:
                    continue
                print(f"Measuring {name} under PCSX-Redux...", flush=True)
                run = run_emulator(env, build, build_dir / f"run-{name}", dump_only=False,
                                   timeout=args.timeout)
                if not run["ok"]:
                    fail(f"{name} measurement failed: {run['error']}")
                    report["variants"][name]["measurement_error"] = run["error"]
                    continue
                report["variants"][name].update(summarize(run["data"], name))

    build_comparison(report)
    passed = not FAILURES
    report["passed"] = passed
    report["failures"] = FAILURES
    write_report(args.output, report)

    if not args.keep and args.build_dir is None:
        print(f"Retaining build directory for inspection: {build_dir}", flush=True)

    if passed:
        print("PASS: matched native/VM Lua feasibility harness built"
              + (" and measured" if args.emulator else " (CPU not requested)"), flush=True)
        return 0
    print(f"FAIL: {len(FAILURES)} problem(s); see {args.output}", flush=True)
    return 1


def decode_header(header):
    if len(header) != LUAC_HEADER_BYTES:
        return {"error": f"unexpected header length {len(header)}"}
    return {
        "signature": header[0:4].hex(),
        "signature_ok": header[0:4] == b"\x1bLua",
        "version": header[4], "version_text": f"{header[4] >> 4}.{header[4] & 0xF}",
        "format": header[5],
        "endianness": header[6], "endianness_text": "little" if header[6] == 1 else "big",
        "sizeof_int": header[7], "sizeof_size_t": header[8],
        "sizeof_instruction": header[9], "sizeof_lua_Number": header[10],
        "lua_Number_is_integral": bool(header[11]),
        "tail": header[12:18].hex(),
        "tail_ok": header[12:18] == b"\x19\x93\r\n\x1a\n",
    }


def summarize(data, name):
    probe = data["probe"]
    out = {"emulator_reason": data.get("reason")}
    if probe[PROBE["magic"]] != MAGIC:
        fail(f"{name}: probe magic {probe[PROBE['magic']]:#x} != {MAGIC:#x} (program did not run)")
        out["probe_valid"] = False
        return out
    if probe[PROBE["done"]] != DONE:
        fail(f"{name}: run did not reach the done marker")
    if probe[PROBE["load_ok"]] != 1:
        fail(f"{name}: a chunk failed to load or a handler raised an error (see emulator.log)")
    out["probe_valid"] = True
    out["load_ok"] = probe[PROBE["load_ok"]] == 1
    header = bytes(probe[PROBE["header"]:PROBE["header"] + LUAC_HEADER_BYTES])
    if probe[PROBE["header_size"]] == LUAC_HEADER_BYTES:
        out["accepted_bytecode_header"] = header.hex()
        out["accepted_bytecode_header_fields"] = decode_header(header)
    out["heap"] = {
        "measured": name != "native",
        "retained_bytes_at_exit": probe[PROBE["heap_retained"]],
        "peak_live_bytes": probe[PROBE["heap_peak"]],
        "allocation_count": probe[PROBE["alloc_count"]],
        "allocated_bytes_total": probe[PROBE["alloc_bytes"]],
        "free_count": probe[PROBE["free_count"]],
        "reallocation_count": probe[PROBE["realloc_count"]],
        "note": ("Lua allocations only, from the harness lua_Alloc. The native variant performs "
                 "no heap allocation by construction." if name != "native" else
                 "The native variant performs no heap allocation by construction."),
    }
    stack = probe[PROBE["stack_high"]]
    out["stack_high_water_bytes"] = (None if stack == -1 else
                                     {"at_least": 32768} if stack == -2 else stack)
    cycles = data["cycles"]
    cases = {}
    for w, workload in enumerate(WORKLOADS):
        for s, count in enumerate(INSTANCE_COUNTS):
            index = case_index(w, s)
            key = f"{workload}@{count}"
            cases[key] = {
                "workload": workload, "instances": count,
                "checksum": probe[PROBE["checksum"] + index] & 0xFFFFFFFF,
                "handler_calls": probe[PROBE["calls"] + index],
                "steady_state_cycles_per_step": statistics_for(cycles.get(str(index), [])),
                "setup_cycles": first(cycles.get(str(1000 + index))),
                "teardown_cycles": first(cycles.get(str(2000 + index))),
                "lua_heap_retained_bytes": probe[PROBE["case_retained"] + index],
                "lua_heap_peak_bytes": probe[PROBE["case_peak"] + index],
                "steady_state_allocations": probe[PROBE["case_allocs"] + index],
                "steady_state_allocated_bytes": probe[PROBE["case_alloc_bytes"] + index],
            }
            samples = cases[key]["steady_state_cycles_per_step"]
            if samples and samples["samples"] != SAMPLE_STEPS:
                fail(f"{name}: {key} produced {samples['samples']} samples, expected {SAMPLE_STEPS}")
    out["cases"] = cases
    out["phases"] = {
        "vm_open_cycles": first(cycles.get("3900")),
        "bytecode_dump_cycles": first(cycles.get("3901")),
        "chunk_load_cycles": {workload: first(cycles.get(str(3000 + w)))
                              for w, workload in enumerate(WORKLOADS)},
    }
    return out


def first(values):
    return values[0] if values else None


def build_comparison(report):
    variants = report["variants"]
    measured = {n: v for n, v in variants.items() if v.get("cases")}
    comparison = {"checksums_match": None, "per_case": {}}
    if len(measured) < 2:
        comparison["note"] = "Fewer than two variants produced measurements; nothing to compare."
        report["comparison"] = comparison
        return
    ok = True
    for w, workload in enumerate(WORKLOADS):
        for count in INSTANCE_COUNTS:
            key = f"{workload}@{count}"
            checksums = {n: v["cases"][key]["checksum"] for n, v in measured.items()}
            calls = {n: v["cases"][key]["handler_calls"] for n, v in measured.items()}
            if len(set(checksums.values())) != 1:
                ok = False
                fail(f"checksum mismatch for {key}: {checksums} "
                     "(the four variants did not compute the same result)")
            if len(set(calls.values())) != 1:
                ok = False
                fail(f"handler call count mismatch for {key}: {calls}")
            comparison["per_case"][key] = {
                "checksum": checksums,
                "handler_calls": calls,
                "median_cycles_per_step": {
                    n: (v["cases"][key]["steady_state_cycles_per_step"] or {}).get("median")
                    for n, v in measured.items()},
                "p95_cycles_per_step": {
                    n: (v["cases"][key]["steady_state_cycles_per_step"] or {}).get("p95")
                    for n, v in measured.items()},
                "steady_state_allocated_bytes": {
                    n: v["cases"][key]["steady_state_allocated_bytes"] for n, v in measured.items()},
            }
    comparison["checksums_match"] = ok
    comparison["linked_sections"] = {
        n: v.get("sections", {}).get("all") and {
            "text": v["sections"]["text"], "rodata": v["sections"]["rodata"],
            "data": v["sections"]["data"], "bss": v["sections"]["bss"],
            "linked_resident_bytes": v["sections"]["linked_resident_bytes"]}
        for n, v in variants.items() if v.get("sections")}
    report["comparison"] = comparison


def write_report(path, report):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"Report: {path}", flush=True)


if __name__ == "__main__":
    sys.exit(main())
