"""Measure an existing Epok PSX build through owned headless Redux breakpoints.

No compilation, scene mutation, input override or PSX-memory writes. The source
ELF locates every hook; unsupported GPU::flip code is rejected, not guessed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import time
import uuid

from original_emulator_session import Frames, digest
from sample_epok_runtime import executable_sections, verify_running_code


def elf_symbols(data):
    executable_sections(data)  # Validate ELF32 little-endian MIPS first.
    offset = struct.unpack_from("<I", data, 32)[0]
    stride, count = struct.unpack_from("<HH", data, 46)
    sections = [struct.unpack_from("<10I", data, offset + i * stride) for i in range(count)]
    symbols = {}
    for section in sections:
        _, kind, _, _, start, size, link, _, _, entry = section
        if kind != 2:
            continue
        if entry != 16 or size % entry or start + size > len(data) or link >= count:
            raise ValueError("Unsupported ELF symbol table")
        strings = sections[link]
        names = data[strings[4]:strings[4] + strings[5]]
        for index in range(start, start + size, entry):
            name, value, length, info, _, defined = struct.unpack_from("<IIIBBH", data, index)
            if not name or not defined:
                continue
            end = names.find(b"\0", name)
            if end < name:
                raise ValueError("Invalid ELF symbol name")
            name = names[name:end].decode()
            record = (value, length, info & 15)
            if name in symbols and symbols[name] != record:
                # Local compiler clones may share a spelling across TUs. Only
                # reject ambiguity if that particular symbol is requested.
                symbols[name] = None
            else:
                symbols[name] = record
    return symbols


def flip_hook(code, address):
    # Verified PsyQo non-interlaced GP1(05h) branch: lui v0,1f80; beq s1,s2;
    # delay-slot lui v1,0504; fallthrough lui v1,0500; sw v1,1814(v0).
    pattern = struct.pack("<5I", 0x3c021f80, 0x12320002, 0x3c030504, 0x3c030500, 0xac431814)
    matches = [i for i in range(0, len(code) - len(pattern) + 1, 4) if code[i:i + len(pattern)] == pattern]
    if len(matches) != 1:
        raise ValueError("Unsupported/ambiguous GPU::flip GP1 store sequence")
    return address + matches[0] + 16


def function_cost_hooks(symbols, sections):
    """Fail closed on ambiguous functions; return sites follow exact MIPS JALs."""
    selected = []
    for label, fragments in (('aim', ('advance_rail_aim',)),
                             ('rail', ('advance_rail_navigation',)),
                             ('ordered_prepare', ('OrderedDrawCursor', '7prepare')),
                             ('ordered_render', ('draw_ordered_world',))):
        matches = [(name, record) for name, record in symbols.items()
                   if record and record[2] == 2 and all(part in name for part in fragments)]
        if len(matches) != 1: raise ValueError('Missing/ambiguous cost hook: '+label)
        name, (entry, size, _) = matches[0]
        if not size or not any(start <= (entry & 0x1fffffff) and
            (entry & 0x1fffffff)+size <= start+len(code) for start,code in sections):
            raise ValueError('Cost function outside executable sections: '+label)
        returns = []
        for start, code in sections:
            for offset in range(0, len(code)-7, 4):
                instruction = struct.unpack_from('<I', code, offset)[0]
                pc = 0x80000000 + start + offset
                target = ((pc+4)&0xf0000000) | ((instruction&0x3ffffff)<<2)
                if instruction >> 26 == 3 and target == entry:
                    returns.append(pc+8)
        if not returns: raise ValueError('No verified direct callers for '+label)
        selected.append(dict(label=label, symbol=name, entry=entry, size=size, returns=returns))
    return selected


def observer_config(elf, profile):
    symbols = elf_symbols(elf)
    sections = executable_sections(elf)
    def unique(name):
        record = symbols.get(name)
        if record is None:
            raise ValueError(f"Missing/ambiguous linked symbol: {name}")
        return record
    def function(name):
        address, size, kind = unique(name)
        if kind != 2 or not size:
            raise ValueError(f"Not a linked function: {name}")
        for start, code in sections:
            offset = (address & 0x1fffffff) - start
            if 0 <= offset and offset + size <= len(code):
                return address, code[offset:offset + size]
        raise ValueError("Function outside executable ELF sections")
    flip, flip_code = function("_ZN5psyqo3GPU4flipEv")
    frame, _ = function("_ZN12_GLOBAL__N_19GameScene5frameEv")
    layouts = [
        ("performance", "performance_stats", profile.FIELDS + profile.EXTRA_FIELDS[:22]),
        ("time", "time", "previous accumulated remainder flags ticks dropped_steps frame_microseconds delta_raw".split()),
        ("lighting", "lighting_stats", "active_lights lit_objects gte_normals baked_vertices triangles dropped_triangles blob_triangles lighting_scanlines frame_scanlines".split()),
        ("streaming", "streaming_stats", profile.STREAMING_FIELDS + ["gameplay_requests", "gameplay_rejected"]),
        ("visibility", "visibility_track_stats", profile.PLAYBACK_STATS["visibility_track"][1]),
        ("ordered", "ordered_draw_stats", profile.ORDERED_DRAW_FIELDS),
        ("verify", "streaming_verify_stats", profile.PLAYBACK_STATS["streaming_verify"][1]),
        ("texture", "texture_load_stats", "attempts uploads bytes failures error".split()),
    ]
    # New transports expose overlap separately without changing old layouts.
    overlap_name = "stream_verify_overlap_bytes"
    if f"_ZN4epok{len(overlap_name)}{overlap_name}E" in symbols:
        layouts.append(("overlap", overlap_name, ["bytes"]))
    blocks = []
    offset = 3
    for key, name, fields in layouts:
        address, size, kind = unique(f"_ZN4epok{len(name)}{name}E")
        if kind != 1 or size != len(fields) * 4 or address % 4 or not 0x80000000 <= address <= 0x80200000 - size:
            raise ValueError(f"Unsupported linked counter layout: {name}")
        blocks.append(dict(name=key, address=address, offset=offset, fields=fields))
        offset += len(fields)
    return dict(frame_entry=frame, flip_entry=flip, gp1_store=flip_hook(flip_code, flip),
                blocks=blocks, row_words=offset)


def validate_executable(exe, sections):
    if exe[:8] != b"PS-X EXE" or len(exe) < 2048:
        raise ValueError("Expected PS-X EXE")
    base, size = struct.unpack_from("<II", exe, 0x18)
    base &= 0x1fffffff
    if len(exe) < 2048 + size:
        raise ValueError("Truncated PS-X EXE")
    for address, code in sections:
        offset = 2048 + address - base
        if address < base or address + len(code) > base + size or exe[offset:offset + len(code)] != code:
            raise ValueError("PS-X EXE instruction bytes do not match linked ELF")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=float, default=260)
    parser.add_argument("--function-costs", action="store_true",
                        help="Read-only inclusive cycle timings for aim, rail, ordered prepare/render; requires exact linked symbols")
    parser.add_argument("--engine", type=Path, default=Path("D:/GitProjects/GameEngines/Epok/EpokEngine"))
    args = parser.parse_args()
    if not 1 <= args.seconds <= 300:
        parser.error("seconds must be 1..300")
    folder, build, engine = args.output.resolve(), args.build.resolve(), args.engine.resolve()
    if folder.exists() and any(folder.iterdir()):
        parser.error("Output/session must be new or empty")
    sys.path.insert(0, str(engine / "tools"))
    import profile_runtime as profile
    snapshot = {name: (build / name).read_bytes() for name in (
        "epok.elf", "epok.ps-exe", "epok.cue", "epok.bin", "epok.map", "display.hh", "build-summary.json")}
    # Keep the tested implementation and cooked inputs, not only the binary.
    # Copies live in this capture; future builds must not erase the experiment.
    for name in ("main.cpp", "epok.hpp", "ordered_renderer.hpp", "ordered_projection.hpp", "baked_draw_types.hpp",
                 "gte_geometry.hpp", "fixed_trigonometry.hpp", "camera_aim.hpp", "camera_direction.hpp",
                 "rail_camera_aim.hpp", "track_camera_pose.hpp", "rail_navigation_types.hpp", "rail_navigation.hpp",
                 "visibility_track.hpp", "streaming_pool.hpp", "streaming.hpp", "streaming_read.hpp",
                 "texture_types.hpp", "texture.hpp", "scene.hh", "GEOMETRY.BIN", "memory-report.json"):
        if (build / name).is_file(): snapshot[name] = (build / name).read_bytes()
    if snapshot["epok.cue"].decode().replace("\r", "").strip() != 'FILE "epok.bin" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00':
        parser.error("Only the generated single-track epok.bin cue is supported")
    if "display_interlaced = false" not in snapshot["display.hh"].decode():
        parser.error("Observer requires a non-interlaced build")
    manifest = json.loads(snapshot["build-summary.json"])["inputs"]
    for name in ("epok.elf", "epok.ps-exe", "epok.bin", "epok.cue", "epok.map"):
        if hashlib.sha256(snapshot[name]).hexdigest() != manifest[name]:
            parser.error(f"Stale build manifest: {name}")
    sections = executable_sections(snapshot["epok.elf"])
    validate_executable(snapshot["epok.ps-exe"], sections)
    config = observer_config(snapshot["epok.elf"], profile)
    if args.function_costs:
        config['function_costs'] = function_cost_hooks(elf_symbols(snapshot['epok.elf']), sections)
    emulator, bios = engine / ".tools/redux/pcsx-redux.main", engine / ".tools/redux/openbios-fastboot.bin"
    bridge = Path(__file__).with_name("epok_cadence_observer.lua")
    cost_bridge = Path(__file__).with_name("epok_function_costs.lua")
    folder.mkdir(parents=True, exist_ok=True)
    for name, data in snapshot.items():
        (folder / name).write_bytes(data)
    if args.function_costs:
        (folder / cost_bridge.name).write_bytes(cost_bridge.read_bytes())
    (folder / "observer-config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")
    token = uuid.uuid4().hex
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0)); listener.listen(1); listener.settimeout(45)
        prefix = (f'OBSERVER_PORT={listener.getsockname()[1]}\nOBSERVER_TOKEN="{token}"\n'
                  f'FRAME_ENTRY={config["frame_entry"]}\nFLIP_ENTRY={config["flip_entry"]}\nGP1_STORE={config["gp1_store"]}\n'
                  + 'COUNTER_BLOCKS={' + ','.join('{'+str(b['address'])+','+str(len(b['fields']))+'}' for b in config['blocks']) + '}\n')
        if args.function_costs:
            prefix += 'FUNCTION_COSTS={' + ','.join('{'+str(f['entry'])+',{' + ','.join(map(str,f['returns']))+'}}'
                                                 for f in config['function_costs']) + '}\n'
        script = folder / "adapter.lua"
        script.write_text(prefix + bridge.read_text(encoding="utf-8"), encoding="utf-8")
        command = [str(emulator), "-portable", str(folder), "-logfile", str(folder / "runtime.log"),
                   "-run", "-stdout", "-fastboot", "-noupdate", "-interpreter", "-softgpu", "-2mb",
                   "-no-gdb", "-no-ui", "-no-pcdrv", "-no-webserver", "-debugger",
                   "-bios", str(bios), "-iso", str(folder / "epok.cue"),
                   "-loadexe", str(folder / "epok.ps-exe"), "-dofile", str(script)]
        startup = None
        if os.name == "nt":
            startup = subprocess.STARTUPINFO(); startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW; startup.wShowWindow = 0
        with (folder / "process.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen(command, cwd=folder, stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                       startupinfo=startup, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            provenance = dict(scope=__doc__, parent_pid=os.getpid(), pid=process.pid, command=command,
                              inputs={str(p): digest(p) for p in (emulator, bios, bridge, Path(__file__), *([cost_bridge] if args.function_costs else []))},
                              build_sha256={name: hashlib.sha256(data).hexdigest() for name, data in snapshot.items()})
            (folder / "session.json").write_text(json.dumps(provenance, indent=2), encoding="utf-8")
            try:
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(15)
                    frames = Frames(connection)
                    assert frames.line() == ["EPOK", token], "Unauthenticated observer"
                    deadline = time.monotonic() + 30
                    while True:
                        ram = frames.data("M")
                        try:
                            verify_running_code(ram, sections)
                            break
                        except AssertionError:
                            if time.monotonic() >= deadline:
                                raise
                            time.sleep(.25)
                    (folder / "before.bin").write_bytes(ram)
                    assert frames.data("P") == b"profiling"
                    print(json.dumps(dict(started=True, pid=process.pid, seconds=args.seconds, hooks={k:v for k,v in config.items() if k.endswith('entry') or k=='gp1_store'})), flush=True)
                    start, heartbeat = time.monotonic(), 0
                    while time.monotonic() - start < args.seconds:
                        if process.poll() is not None:
                            raise RuntimeError("Owned emulator exited during capture")
                        elapsed = time.monotonic() - start
                        if elapsed >= heartbeat + 30:
                            state = json.loads(frames.data("Q"))
                            print(json.dumps(dict(elapsed_seconds=round(elapsed), **state)), flush=True)
                            if state["overflow"] or state["callback_error"]:
                                raise RuntimeError("Observer stopped/overflowed")
                            heartbeat = elapsed
                        time.sleep(.25)
                    data = frames.data("R")
                    (folder / "profile.json").write_bytes(data)
                    ram = frames.data("M")
                    verify_running_code(ram, sections)
                    (folder / "after.bin").write_bytes(ram)
                    for name, expected in snapshot.items():
                        assert (build / name).read_bytes() == expected, f"Source build changed during capture: {name}"
                    result = json.loads(data)
                    assert not result["overflow"] and not result["callback_error"], "Incomplete observer capture"
                    print(json.dumps(dict(completed=True, rows=len(result["rows"]), presentations=len(result["presentations"]),
                                          running_code_verified_bytes=sum(len(c) for _, c in sections), folder=str(folder))), flush=True)
            finally:
                if process.poll() is None:
                    process.terminate()
                process.wait(timeout=10)
                print(json.dumps(dict(stopped=True, pid=process.pid, exit_code=process.returncode)), flush=True)


if __name__ == "__main__":
    main()
