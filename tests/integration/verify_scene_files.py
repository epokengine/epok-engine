"""External scene files: scenes kept out of the executable load from SCENEnn.BIN.

Builds a fresh three-scene project whose camera carries a Lua `Pilot` component
(Cross requests scene 1, Triangle scene 2, Circle scene 0) and gives every scene
its own sky colour. The project's scene table decides which scenes stay in the
EXE. Each run boots in PCSX-Redux (never physical hardware) while a Lua driver
presses buttons and samples `epok::scene_stats`, `epok::scene_file_stats` and
the header of the shared scene region:

1. CD on demand, Main in the EXE: Second and Third load from the disc image.
2. PS-EXE export (PCDrv), every scene outside: the startup scene itself loads
   behind the boot loading screen.
3. PCDrv failures: a corrupt SCENE02.BIN is refused (checksum), the file is
   restored while the loading screen waits, and START loads it; a missing
   SCENE01.BIN is reported as missing.
4. A corrupt sector inside the CD image is refused the same way.
5. The executable and static RAM: the same project with every scene in the EXE.
6. Two maps of the Dungeon template: with both in the EXE the link overflows
   RAM; with both outside, static RAM stays within 32 KiB of a one-map game and
   the loaded map renders the same frame, pixel for pixel, as that game.
Measured loading times come from the runtime counters and are emulator times.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import json
import re
import shutil
import struct
import subprocess
import time
import uuid
import zipfile

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PILOT = """---@class Pilot : epok.ActorComponent
local Pilot = epok.ActorComponent:extend()

---@override
function Pilot:tick(delta_seconds)
    if epok.input.pressed(14, 0) then
        epok.request_scene(1)
    end
    if epok.input.pressed(12, 0) then
        epok.request_scene(2)
    end
    if epok.input.pressed(13, 0) then
        epok.request_scene(0)
    end
end

return Pilot
"""
SKY = {"Main": (0.75, 0.1, 0.1), "Second": (0.1, 0.75, 0.1), "Third": (0.1, 0.1, 0.75)}
STATES = ["idle", "waiting", "reading", "ready", "failed"]
ERRORS = ["none", "record", "drive", "missing", "size", "read", "checksum"]
# Events run on the frame they name. Buttons are held for six frames.
DRIVER = r"""local ffi = require('ffi')
local ram = ffi.cast('uint8_t*', PCSX.getMemPtr())
local function u32(address) return ffi.cast('uint32_t*', ram + address)[0] end
local function i8(address) return ffi.cast('int8_t*', ram + address)[0] end
local out = assert(io.open(%(log)s, 'w'))
local pad = PCSX.SIO0.slots[1].pads[1]
local B = PCSX.CONSTS.PAD.BUTTON
local events = %(events)s
local function centre()
  local shot = PCSX.GPU.takeScreenShot()
  local data = tostring(shot.data)
  local i = (math.floor(shot.height / 2) * shot.width + math.floor(shot.width / 2)) * 2
  local v = data:byte(i + 1) + data:byte(i + 2) * 256
  return v %% 32 * 8, math.floor(v / 32) %% 32 * 8, math.floor(v / 1024) %% 32 * 8
end
local function sample(label)
  local s, f, r = %(scene)d, %(files)d, %(region)d
  local red, green, blue = centre()
  out:write(string.format('{"label":"%%s","frame":%%d,"transitions":%%d,"rejected":%%d,"active":%%d,'
    .. '"loads":%%d,"failures":%%d,"bytes":%%d,"last_us":%%d,"loaded":%%d,"total":%%d,"resident":%%d,'
    .. '"loading":%%d,"state":%%d,"error":%%d,"magic":%%d,"bank":%%d,"rgb":[%%d,%%d,%%d]}\n',
    label, frames, u32(s), u32(s + 4), u32(s + 8), u32(f), u32(f + 4), u32(f + 8), u32(f + 12),
    u32(f + 20), u32(f + 24), i8(f + 28), i8(f + 29), ram[f + 30], ram[f + 31],
    r ~= 0 and u32(r) or 0, r ~= 0 and u32(r + 4) or -1, red, green, blue))
  out:flush()
end
frames = 0
EPOK_SCENE_DRIVER = PCSX.Events.createEventListener('GPU::Vsync', function()
  frames = frames + 1
  for _, e in ipairs(events) do
    if e[2] == 'press' then
      if frames == e[1] then pad.setOverride(B[e[3]]) end
      if frames == e[1] + 6 then pad.clearOverride(B[e[3]]) end
    elseif frames == e[1] then
      if e[2] == 'sample' then sample(e[3])
      elseif e[2] == 'copy' then
        local source = assert(io.open(e[3], 'rb')); local bytes = source:read('*a'); source:close()
        local target = assert(io.open(e[4], 'wb')); target:write(bytes); target:close()
      elseif e[2] == 'capture' then
        local shot = PCSX.GPU.takeScreenShot()
        local f = assert(io.open(e[3], 'wb'))
        f:write(string.format('%%d %%d %%d\n', shot.width, shot.height, tonumber(shot.bpp)))
        f:write(tostring(shot.data)); f:close()
      elseif e[2] == 'quit' then out:close(); PCSX.quit(0) end
    end
  end
end)
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    out = args.output or ROOT / "artifacts/scene-files" / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=False)
    project = out / "project"
    results = {}

    def run(*arguments, success=True):
        result = subprocess.run([str(EXE), *map(str, arguments)], capture_output=True, text=True,
                                creationflags=FLAGS, timeout=900)
        text = result.stdout + result.stderr
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(text)
        assert (result.returncode == 0) == success, text
        return text

    run("--create-project", project, "--name", "Scene Files", "--template", "basic")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    documents.write_text(project / "Local.epokconfig", json.dumps(config, indent=2))
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text())
    assert manifest.get("scene_storage") == {"in_executable": []}, (
        "New projects start with an empty scene table", manifest.get("scene_storage"))
    manifest["rendering"]["width"], manifest["rendering"]["height"] = 320, 240
    documents.write_text(manifest_path, documents.dumps(manifest))
    run("--project", project, "--new-lua-class", "Pilot")
    (project / "assets/scripts/Pilot.lua").write_text(PILOT, encoding="utf-8")
    classes = (project / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8")
    pilot = re.search(r"Pilot\.lua: ([0-9a-f-]{36})", classes)[1]
    scenes = project / "assets/scenes"
    template = documents.loads((scenes / "Main.epokmap").read_text())
    for name, sky in SKY.items():
        scene = json.loads(json.dumps(template))
        scene["name"] = name
        camera = scene["actors"][0]["components"]
        next(c for c in camera if c["name"] == "Camera")["properties"]["camera_sky_color"] = list(sky)
        camera.append({"class": dict(class_id=pilot, name="Pilot"), "id": str(uuid.uuid4()),
                       "name": "Pilot", "overrides": [], "properties": {}, "root": False})
        documents.write_text(scenes / f"{name}.epokmap", documents.dumps(scene))

    def configure(data, in_executable):
        value = documents.loads(manifest_path.read_text())
        value["play"]["data"], value["play"]["content"] = data, "whole_game"
        value["scene_storage"] = {"in_executable": [f"assets/scenes/{n}.epokmap" for n in in_executable]}
        documents.write_text(manifest_path, documents.dumps(value))

    def build(label):
        log = run("--project", project, "--build-psx", "--use-play-profile")
        build = project / ".epok/build"
        report = json.loads((build / "memory-report.json").read_text())
        snapshot = out / f"build-{label}"
        shutil.copytree(build, snapshot, ignore=shutil.ignore_patterns("*.o", "*.dep", "sdk"))
        return snapshot, report, log

    def address(symbols, name, required=True):
        match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
        assert match or not required, name
        return int(match[1], 16) & 0x1fffff if match else 0

    def play(name, symbols, events, media):
        work = out / f"redux-{name}"
        (work / "portable").mkdir(parents=True)
        (work / "portable/pcsx.json").write_text("{}")
        log = work / "samples.jsonl"
        lua_events = "{" + ",".join(
            "{" + ",".join(json.dumps(str(v)) if not isinstance(v, int) else str(v) for v in e) + "}"
            for e in events) + "}"
        driver = work / "driver.lua"
        driver.write_text(DRIVER % dict(log=json.dumps(str(log)), events=lua_events,
                                        scene=address(symbols, "epok::scene_stats"),
                                        files=address(symbols, "epok::scene_file_stats"),
                                        region=address(symbols, "__epok_scene_region_start", False)),
                          encoding="utf-8")
        emulator = Path(config["emulator"])
        if emulator.with_name("pcsx-redux.main").is_file():
            emulator = emulator.with_name("pcsx-redux.main")
        bios = next(b for b in (emulator.parent / "openbios-fastboot.bin", emulator.parent / "openbios.bin")
                    if b.is_file())
        command = [str(emulator), "-portable", str(work / "portable"), "-run", "-stdout", "-fastboot",
                   "-noupdate", "-interpreter", "-softgpu", "-2mb", "-no-gdb", "-no-ui", *media,
                   "-bios", str(bios), "-dofile", str(driver)]
        with (work / "emulator.log").open("w", encoding="utf-8") as console:
            subprocess.run(command, cwd=work / "portable", stdout=console, stderr=console,
                           creationflags=FLAGS, timeout=600)
        assert "EPOK: runtime ready" in (work / "emulator.log").read_text(errors="replace")
        samples = {s["label"]: s for s in map(json.loads, log.read_text().splitlines())}
        for s in samples.values():
            s["state"], s["error"] = STATES[s["state"]], ERRORS[s["error"]]
        results[name] = samples
        return samples

    def dominant(sample):
        red, green, blue = sample["rgb"]
        return max(("Main", red), ("Second", green), ("Third", blue), key=lambda p: p[1])[0]

    def ready(sample, active, bank, file):
        assert sample["active"] == active and sample["state"] == "ready", sample
        assert sample["resident"] == file and sample["magic"] == 0x454E4353 and sample["bank"] == bank, sample
        assert dominant(sample) == list(SKY)[active], ("Scene picture", sample)

    tour = [(300, "press", "CROSS"), (580, "sample", "second"), (600, "press", "TRIANGLE"),
            (880, "sample", "third"), (900, "press", "CIRCLE"), (1180, "sample", "main"),
            (1200, "press", "CROSS"), (1480, "sample", "second_again"), (1490, "quit")]

    # 1. CD on demand, Main resident.
    configure("disc", ["Main"])
    build_cd, report_cd, log = build("cd")
    assert "CD assets:" in log, log
    files = json.loads((build_cd / "scene-files.json").read_text())["files"]
    assert [(f["bank"], f["scene"], f["name"]) for f in files] == [
        (1, "Second", "SCENE01.BIN"), (2, "Third", "SCENE02.BIN")], files
    disc = (build_cd / "disc.xml").read_text()
    assert 'name="SCENE01.BIN"' in disc and 'name="SCENE02.BIN"' in disc and "SCENE00" not in disc
    symbols = (build_cd / "epok.map").read_text()
    samples = play("cd", symbols, [(250, "sample", "start")] + tour,
                   ["-iso", str(build_cd / "epok.cue"), "-loadexe", str(build_cd / "epok.ps-exe")])
    start = samples["start"]
    assert start["active"] == 0 and start["loads"] == 0 and start["resident"] == -1, start
    assert dominant(start) == "Main", start
    ready(samples["second"], 1, 1, 0)
    ready(samples["third"], 2, 2, 1)
    main_scene = samples["main"]
    assert main_scene["active"] == 0 and main_scene["loads"] == 2 and dominant(main_scene) == "Main", main_scene
    again = samples["second_again"]
    ready(again, 1, 1, 0)
    assert again["loads"] == 3 and again["failures"] == 0 and again["transitions"] == 4, again
    print(f"PASS CD: resident Main, Second and Third from the disc ({again['last_us']} us last load, emulated)",
          flush=True)

    # 2. PS-EXE package, every scene outside the EXE: the startup scene too.
    configure("disc", [])
    archive = out / "all-outside-ps-exe.zip"
    run("--project", project, "--export-ps-exe", "--debug-symbols", "--output", archive)
    package = out / "extracted-all-outside"
    with zipfile.ZipFile(archive) as zip_file:
        zip_file.extractall(package)
        names = set(zip_file.namelist())
    assert {"epok.ps-exe", "SCENE00.BIN", "SCENE01.BIN", "SCENE02.BIN"} <= names, names
    description = json.loads((package / "epok-package.json").read_text())
    assert description["loader"]["pcdrv_files"] == ["SCENE00.BIN", "SCENE01.BIN", "SCENE02.BIN"], description
    assert "Scene data, read through PCDrv when that scene starts" in (package / "README.txt").read_text()
    symbols = (package / "debug/epok.map").read_text()
    media = ["-loadexe", str(package / "epok.ps-exe"), "-pcdrv", "-pcdrvbase", str(package)]
    samples = play("pcdrv", symbols, [(250, "sample", "start")] + tour, media)
    start = samples["start"]
    assert start["active"] == 0 and start["loads"] == 1 and start["transitions"] == 0, start
    ready(start, 0, 0, 0)
    ready(samples["second"], 1, 1, 1)
    ready(samples["third"], 2, 2, 2)
    ready(samples["main"], 0, 0, 0)
    assert samples["second_again"]["loads"] == 5 and samples["second_again"]["failures"] == 0
    print("PASS PCDrv: the startup scene and every transition load from SCENEnn.BIN", flush=True)

    # 3. PCDrv failures: corrupt, then restore while the loading screen waits; missing.
    failing = out / "extracted-failures"
    shutil.copytree(package, failing)
    good = failing / "SCENE02.good"
    shutil.copyfile(failing / "SCENE02.BIN", good)
    data = bytearray((failing / "SCENE02.BIN").read_bytes())
    data[100] ^= 0x5A
    (failing / "SCENE02.BIN").write_bytes(data)
    (failing / "SCENE01.BIN").unlink()
    media = ["-loadexe", str(failing / "epok.ps-exe"), "-pcdrv", "-pcdrvbase", str(failing)]
    samples = play("pcdrv-failures", symbols, [
        (300, "press", "TRIANGLE"), (580, "sample", "corrupt"),
        (600, "copy", str(good), str(failing / "SCENE02.BIN")), (620, "press", "START"),
        (900, "sample", "restored"), (920, "press", "CROSS"), (1200, "sample", "missing"),
        (1210, "quit")], media)
    corrupt = samples["corrupt"]
    assert corrupt["state"] == "failed" and corrupt["error"] == "checksum", corrupt
    assert corrupt["resident"] == -1 and corrupt["failures"] == 1, ("Nothing of Third ran", corrupt)
    ready(samples["restored"], 2, 2, 2)
    missing = samples["missing"]
    assert missing["state"] == "failed" and missing["error"] == "missing" and missing["failures"] == 2, missing
    print("PASS PCDrv: corrupt file refused, restored and retried with START; missing file reported", flush=True)

    # 4. A damaged sector in the CD image.
    image = out / "corrupt-disc"
    shutil.copytree(build_cd, image)
    raw = bytearray((image / "epok.bin").read_bytes())
    marker = struct.pack("<II", 0x454E4353, 2)
    at = raw.find(marker)
    assert at > 0 and raw.find(marker, at + 1) < 0, "SCENE02.BIN must appear once in the image"
    raw[at + 64] ^= 0x5A
    (image / "epok.bin").write_bytes(raw)
    samples = play("cd-corrupt", (image / "epok.map").read_text(), [
        (300, "press", "TRIANGLE"), (580, "sample", "corrupt"), (590, "quit")],
        ["-iso", str(image / "epok.cue"), "-loadexe", str(image / "epok.ps-exe")])
    assert samples["corrupt"]["state"] == "failed" and samples["corrupt"]["error"] == "checksum", samples
    print("PASS CD: a damaged scene sector is refused", flush=True)

    # 5. Executable and RAM compared with every scene in the EXE.
    configure("disc", ["Main", "Second", "Third"])
    build_resident, report_resident, _ = build("resident")
    assert not (build_resident / "scene-overlays.ld").exists()
    assert json.loads((build_resident / "scene-files.json").read_text())["files"] == []

    def region(report):
        nodes = [c for c in report["ram"]["root"]["children"] if c["name"] == "External scenes"]
        return nodes[0]["bytes"] if nodes else 0
    sizes = {f: (build_cd / f).stat().st_size for f in ("SCENE01.BIN", "SCENE02.BIN")}
    comparison = {
        "exe_bytes": {"resident": (build_resident / "epok.ps-exe").stat().st_size,
                      "cd": (build_cd / "epok.ps-exe").stat().st_size},
        "static_ram": {"resident": report_resident["ram"]["used"], "cd": report_cd["ram"]["used"]},
        "scene_region": region(report_cd),
        "scene_files": sizes,
    }
    results["comparison"] = comparison
    assert comparison["scene_region"] == max(sizes.values()), comparison
    # These scenes hold only a camera (2 KiB each), so the loader's code costs
    # more than the data it moves out: tiny scenes are a net loss. Bound that
    # cost; the savings appear when scene data outweighs it.
    overhead = comparison["exe_bytes"]["cd"] - comparison["exe_bytes"]["resident"]
    comparison["loader_overhead_bytes"] = overhead
    assert overhead < 8192, comparison
    print(f"PASS sizes: {json.dumps(comparison)}", flush=True)

    # 6. Two Dungeon maps: too large for one executable, both fit outside it.
    dungeon = out / "dungeon"
    run("--create-project", dungeon, "--name", "Dungeon Files", "--template", "dungeon")
    documents.write_text(dungeon / "Local.epokconfig", json.dumps(config, indent=2))
    maps = dungeon / "assets/scenes"
    level = documents.loads((maps / "Main.epokmap").read_text())
    level["name"] = "Level2"
    documents.write_text(maps / "Level2.epokmap", documents.dumps(level))
    dungeon_manifest = project_manifest(dungeon)

    def dungeon_build(label, content, data, inside, success=True):
        value = documents.loads(dungeon_manifest.read_text())
        value["play"]["data"], value["play"]["content"] = data, content
        value["scene_storage"] = {"in_executable": [f"assets/scenes/{n}.epokmap" for n in inside]}
        documents.write_text(dungeon_manifest, documents.dumps(value))
        log = run("--project", dungeon, "--build-psx", "--use-play-profile", success=success)
        if not success:
            return None, None, log
        snapshot = out / f"dungeon-{label}"
        shutil.copytree(dungeon / ".epok/build", snapshot, ignore=shutil.ignore_patterns("*.o", "*.dep", "sdk"))
        return snapshot, json.loads((snapshot / "memory-report.json").read_text()), log

    _, _, log = dungeon_build("both-in", "whole_game", "disc", ["Main", "Level2"], success=False)
    assert "will not fit in region `ram'" in log or "PSX main RAM leaves only" in log, log[-3000:]
    single, single_report, _ = dungeon_build("single", "current_scene", "executable", [])
    outside, outside_report, _ = dungeon_build("outside", "whole_game", "disc", [])
    files = {p.name: p.stat().st_size for p in sorted(outside.glob("SCENE*.BIN"))}
    dungeon_sizes = {
        "static_ram": {"one_map_in_exe": single_report["ram"]["used"],
                       "two_maps_outside": outside_report["ram"]["used"]},
        "exe_bytes": {"one_map_in_exe": (single / "epok.ps-exe").stat().st_size,
                      "two_maps_outside": (outside / "epok.ps-exe").stat().st_size},
        "scene_region": region(outside_report),
        "scene_files": files,
    }
    results["dungeon"] = dungeon_sizes
    assert set(files) == {"SCENE00.BIN", "SCENE01.BIN"} and dungeon_sizes["scene_region"] == max(files.values())
    assert dungeon_sizes["static_ram"]["two_maps_outside"] <= dungeon_sizes["static_ram"]["one_map_in_exe"] + 32768, (
        dungeon_sizes)
    frames = {}
    for label, build_dir, media in [
            ("one-map-in-exe", single, ["-loadexe", str(single / "epok.ps-exe")]),
            ("two-maps-outside", outside, ["-iso", str(outside / "epok.cue"), "-loadexe",
                                           str(outside / "epok.ps-exe")])]:
        shot = out / f"dungeon-{label}.raw"
        samples = play(f"dungeon-{label}", (build_dir / "epok.map").read_text(),
                       [(200, "capture", str(shot)), (201, "sample", "frame"), (205, "quit")], media)
        frames[label] = shot.read_bytes()
        results["dungeon"][f"{label}_sample"] = samples["frame"]
    loaded = results["dungeon"]["two-maps-outside_sample"]
    assert loaded["state"] == "ready" and loaded["loads"] == 1 and loaded["resident"] == 0, loaded
    assert frames["one-map-in-exe"] == frames["two-maps-outside"], "The loaded map must render identically"
    print(f"PASS Dungeon: two maps outside the EXE fit and render like one map inside: {json.dumps(dungeon_sizes)}",
          flush=True)
    documents.write_text(out / "report.json", json.dumps(results, indent=2))
    print(f"Scene file checks passed: {out}")


if __name__ == "__main__":
    main()
