"""Real MIPS PS-EXE packages with several scenes, resident audio and a video.

Builds a fresh three-scene project whose camera carries a Lua `Pilot` component:
Cross requests scene 1, Circle scene 0 and Triangle scene 2. The startup scene
plays a resident MusicSequence and the second scene a resident sound effect.
Whole Game and Selected Scenes packages are extracted to clean folders and run
in PCSX-Redux with no disc while a Lua driver presses the buttons and samples
`epok::scene_stats` and the sound-effect and note voices in `epok::audio_voices`.
Finally a video in the third scene must stop a Whole Game export by name and
leave a Selected Scenes export without that scene unaffected.
Never connects to physical serial hardware.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
from verify_fmv import write_avi
import argparse
import json
import math
import re
import struct
import subprocess
import time
import uuid
import wave
import zipfile

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
# One note at tick zero, off at 96 PPQN / 0.5 seconds (as verify_psx_sequence.py).
MIDI = bytes.fromhex("4d546864000000060000000100604d54726b0000000c00903c6460803c0000ff2f00")
PILOT = """---@class Pilot : epok.ActorComponent
local Pilot = epok.ActorComponent:extend()

---@override
function Pilot:tick(delta_seconds)
    if epok.input.pressed(14, 0) then
        epok.request_scene(1)
    end
    if epok.input.pressed(13, 0) then
        epok.request_scene(0)
    end
    if epok.input.pressed(12, 0) then
        epok.request_scene(2)
    end
end

return Pilot
"""
INTRO = """---@class IntroPlayer : epok.ActorComponent
local IntroPlayer = epok.ActorComponent:extend()

IntroPlayer.video = epok.AssetRef("Video")

function IntroPlayer:begin_play()
    epok.play_video(self.video, true)
end

return IntroPlayer
"""
# Buttons are held for six frames; each phase then settles before sampling.
DRIVER = r"""local ffi = require('ffi')
local ram = ffi.cast('uint8_t*', PCSX.getMemPtr())
local function u32(address) return ffi.cast('uint32_t*', ram + address)[0] end
local function i8(address) return ffi.cast('int8_t*', ram + address)[0] end
local out = assert(io.open(%(log)s, 'w'))
local pad = PCSX.SIO0.slots[1].pads[1]
local B = PCSX.CONSTS.PAD.BUTTON
local presses = {[300] = B.CROSS, [600] = B.TRIANGLE, [900] = B.CIRCLE, [1200] = B.SELECT}
local function capture(name)
  local shot = PCSX.GPU.takeScreenShot()
  local f = assert(io.open(%(work)s .. '/' .. name .. '.raw', 'wb'))
  f:write(string.format('%%d %%d %%d\n', shot.width, shot.height, tonumber(shot.bpp)))
  f:write(tostring(shot.data))
  f:close()
end
local frames, effect_seen, effect_released = 0, false, false
-- Notes retrigger and release; keep each window's peak, not a single frame.
local peak_effects, peak_notes = 0, 0
local function voices()
  local effects, notes = 0, 0
  for i = 0, 23 do
    local base = %(voices)d + i * %(stride)d
    if u32(base) ~= 0 then
      if %(stride)d == 16 and i8(base + 12) >= 0 then notes = notes + 1 else effects = effects + 1 end
    end
  end
  return effects, notes
end
local function sample(label)
  local overlay = %(overlay)d ~= 0 and ram[%(overlay)d] or -1
  out:write(string.format('{"label":"%%s","frame":%%d,"transitions":%%d,"rejected":%%d,"active":%%d,'
    .. '"effect_voices":%%d,"note_voices":%%d,"effect_seen":%%s,"effect_released":%%s,"overlay":%%d}\n',
    label, frames, u32(%(scene)d), u32(%(scene)d + 4), u32(%(scene)d + 8), peak_effects, peak_notes,
    tostring(effect_seen), tostring(effect_released), overlay))
  out:flush()
  peak_effects, peak_notes = 0, 0
end
EPOK_PACKAGE_DRIVER = PCSX.Events.createEventListener('GPU::Vsync', function()
  frames = frames + 1
  local effects, notes = voices()
  if effects > peak_effects then peak_effects = effects end
  if notes > peak_notes then peak_notes = notes end
  if effects > 0 then effect_seen = true elseif effect_seen then effect_released = true end
  for start, button in pairs(presses) do
    if frames == start then pad.setOverride(button) end
    if frames == start + 6 then pad.clearOverride(button) end
  end
  if frames == 280 then sample('start') end
  if frames == 580 then sample('after_cross') end
  if frames == 880 then sample('after_triangle') end
  if frames == 1180 then sample('after_circle'); capture('before_select') end
  if frames == 1480 then sample('after_select'); capture('after_select'); out:close(); PCSX.quit(0) end
end)
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    out = args.output or ROOT / "artifacts/ps-exe-scenes" / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=False)
    project = out / "project"

    def run(*arguments, success=True):
        result = subprocess.run([str(EXE), *map(str, arguments)], capture_output=True, text=True,
                                creationflags=FLAGS, timeout=600)
        text = result.stdout + result.stderr
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(text)
        assert (result.returncode == 0) == success, text
        return text

    def component(class_id, name, overrides=(), properties=None):
        return {"class": dict(class_id=class_id, name=name), "id": str(uuid.uuid4()), "name": name,
                "overrides": list(overrides), "properties": properties or {}, "root": False}

    run("--create-project", project, "--name", "Scene Check", "--template", "basic")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    documents.write_text(project / "Local.epokconfig", json.dumps(config, indent=2))
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text())
    manifest["rendering"]["width"], manifest["rendering"]["height"] = 320, 240
    documents.write_text(manifest_path, documents.dumps(manifest))

    with wave.open(str(project / "assets/blip.wav"), "wb") as wav:
        wav.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        wav.writeframes(b"".join(struct.pack("<h", round(math.sin(i * 2 * math.pi * 880 / 22050) * 12000))
                                 for i in range(4410)))
    run("--project", project, "--import-audio", "assets/blip.wav", "--audio-usage", "sfx")
    run("--project", project, "--create-starter-bank", "assets/Retro.epokasset")
    ids = {r["path"]: r["id"] for r in json.loads(run("--project", project, "--scan-assets"))["assets"]}
    (project / "assets/song.mid").write_bytes(MIDI)
    run("--project", project, "--import-audio", "assets/song.mid", "--sound-bank", ids["assets/Retro.epokasset"],
        "--sequence-loop", "whole", "--voice-limit", "16")
    ids = {r["path"]: r["id"] for r in json.loads(run("--project", project, "--scan-assets"))["assets"]}
    run("--project", project, "--new-lua-class", "Pilot")
    (project / "assets/scripts/Pilot.lua").write_text(PILOT, encoding="utf-8")
    classes = (project / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8")
    pilot = re.search(r"Pilot\.lua: ([0-9a-f-]{36})", classes)[1]

    scenes = project / "assets/scenes"
    template = documents.loads((scenes / "Main.epokmap").read_text())
    audio_class = "7f0eb028-5301-4ac7-b93b-5665fab12b20"
    for name, clip in (("Main", ids["assets/song.epokasset"]), ("Second", ids["assets/blip.epokasset"]),
                       ("Third", None)):
        scene = json.loads(json.dumps(template))
        scene["name"] = name
        components = scene["actors"][0]["components"]
        components.append(component(pilot, "Pilot"))
        if clip:
            components.append(component(audio_class, "epok::AudioComponent", ["audio"],
                                        dict(audio=dict(clip=clip, play_on_start=True))))
        documents.write_text(scenes / f"{name}.epokmap", documents.dumps(scene))

    def package(name, *scope):
        archive = out / f"{name}-ps-exe.zip"
        run("--project", project, "--export-ps-exe", "--debug-symbols", "--output", archive, *scope)
        folder = out / f"extracted-{name}"
        with zipfile.ZipFile(archive) as package_zip:
            package_zip.extractall(folder)
        return archive, folder, json.loads((folder / "epok-package.json").read_text())

    def play(name, folder):
        symbols = (folder / "debug/epok.map").read_text()

        def address(symbol, required=True):
            match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(symbol) + r"\b", symbols)
            assert match or not required, symbol
            return int(match[1], 16) & 0x1fffff if match else 0
        music = address("epok::music_sequence_stats", required=False)
        work = out / f"redux-{name}"
        (work / "portable").mkdir(parents=True)
        (work / "portable/pcsx.json").write_text("{}")
        log = work / "samples.jsonl"
        driver = work / "driver.lua"
        driver.write_text(DRIVER % dict(log=json.dumps(str(log)), work=json.dumps(str(work)),
                                        voices=address("epok::audio_voices"), stride=16 if music else 12,
                                        scene=address("epok::scene_stats"),
                                        overlay=address("epok::debug_hud::visible", required=False)),
                          encoding="utf-8")
        emulator = Path(config["emulator"])
        if emulator.with_name("pcsx-redux.main").is_file():
            emulator = emulator.with_name("pcsx-redux.main")
        bios = next(b for b in (emulator.parent / "openbios-fastboot.bin", emulator.parent / "openbios.bin")
                    if b.is_file())
        with (work / "emulator.log").open("w", encoding="utf-8") as console:
            subprocess.run([str(emulator), "-portable", str(work / "portable"), "-run", "-stdout", "-fastboot",
                            "-noupdate", "-interpreter", "-softgpu", "-2mb", "-no-gdb", "-no-ui",
                            "-loadexe", str(folder / "epok.ps-exe"), "-pcdrv", "-pcdrvbase", str(folder),
                            "-bios", str(bios), "-dofile", str(driver)],
                           cwd=work / "portable", stdout=console, stderr=console, creationflags=FLAGS,
                           timeout=300)
        samples = {s["label"]: s for s in map(json.loads, log.read_text().splitlines())}
        assert "EPOK: runtime ready" in (work / "emulator.log").read_text(errors="replace")
        assert len(samples) == 5, samples
        return samples

    def lower_left_changed(name):
        """Pixels that differ in the overlay's corner across the Select press."""
        shots = []
        for label in ("before_select", "after_select"):
            header, pixels = (out / f"redux-{name}/{label}.raw").read_bytes().split(b"\n", 1)
            width, height, _ = map(int, header.split())
            values = struct.unpack(f"<{width * height}H", pixels[:width * height * 2])
            shots.append([values[y * width + x] for y in range(height // 2, height) for x in range(width // 2)])
        return sum(a != b for a, b in zip(*shots))

    # The package build must not depend on the project's own build folder.
    def hidden(callback):
        build = project / ".epok/build"
        build.rename(build.with_name("build.hidden"))
        try:
            return callback()
        finally:
            build.with_name("build.hidden").rename(build)

    results = {}
    _, folder, description = package("whole-game", "--debug-overlay")
    assert description["content"] == "whole_game"
    assert description["debug_overlay"] == dict(panels=["fps", "cpu", "gte", "gpu", "spu"],
                                                toggle_button="select", initially_visible=False)
    assert description["initial_scene"] == "assets/scenes/Main.epokmap"
    assert description["scenes"] == [f"assets/scenes/{n}.epokmap" for n in ("Main", "Second", "Third")]
    samples = results["whole-game"] = hidden(lambda: play("whole-game", folder))
    start, cross, triangle, circle = (samples[k] for k in ("start", "after_cross", "after_triangle",
                                                           "after_circle"))
    assert start["active"] == 0 and start["transitions"] == 0, start
    assert start["note_voices"] > 0 and start["effect_voices"] == 0, ("Startup MusicSequence only", start)
    assert cross["active"] == 1 and cross["transitions"] == 1, cross
    assert cross["effect_voices"] > 0 and cross["effect_released"], ("Resident sound effect played to its end", cross)
    assert triangle["active"] == 2 and triangle["transitions"] == 2, triangle
    assert circle["active"] == 0 and circle["transitions"] == 3 and circle["rejected"] == 0, circle
    assert circle["note_voices"] > 0, ("MusicSequence restarts with its scene", circle)
    print("PASS Whole Game: startup scene, three transitions, MusicSequence and sound effect", flush=True)
    assert all(samples[k]["overlay"] == 0 for k in ("start", "after_circle")), "The overlay starts hidden"
    assert samples["after_select"]["overlay"] == 1, samples["after_select"]
    assert samples["after_select"]["active"] == 0 and samples["after_select"]["transitions"] == 3
    assert lower_left_changed("whole-game") > 500, "Select draws the overlay"
    print("PASS Select shows the hidden debug overlay", flush=True)

    selected = ("--selected-scene", "assets/scenes/Main.epokmap", "--selected-scene",
                "assets/scenes/Second.epokmap", "--initial-scene", "assets/scenes/Second.epokmap")
    _, folder, description = package("selected", *selected)
    assert description["content"] == "selected_scenes"
    assert description["initial_scene"] == "assets/scenes/Second.epokmap"
    assert description["scenes"] == ["assets/scenes/Main.epokmap", "assets/scenes/Second.epokmap"]
    samples = results["selected"] = hidden(lambda: play("selected", folder))
    start, cross, triangle, circle = (samples[k] for k in ("start", "after_cross", "after_triangle",
                                                           "after_circle"))
    # Bank 0 is the initial scene (Second): its sound effect plays, no music.
    assert start["active"] == 0 and start["effect_voices"] > 0 and start["note_voices"] == 0, start
    assert cross["active"] == 1 and cross["transitions"] == 1 and cross["note_voices"] > 0, cross
    assert triangle["active"] == 1 and triangle["transitions"] == 1 and triangle["rejected"] == 1, (
        "An excluded scene must be rejected", triangle)
    assert circle["active"] == 0 and circle["transitions"] == 2, circle
    print("PASS Selected Scenes: initial scene, included transition, excluded scene rejected", flush=True)
    assert description["debug_overlay"] is None and samples["after_select"]["overlay"] == -1
    assert lower_left_changed("selected") == 0, "Without the option Select draws nothing"

    write_avi(project / "assets/intro.avi")
    video = re.search(r"Imported Video: ([0-9a-f-]{36})",
                      run("--project", project, "--import-video", "assets/intro.avi"))[1]
    run("--project", project, "--new-lua-class", "IntroPlayer")
    (project / "assets/scripts/IntroPlayer.lua").write_text(INTRO, encoding="utf-8")
    classes = (project / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8")
    intro = re.search(r"IntroPlayer\.lua: ([0-9a-f-]{36})", classes)[1]
    third = documents.loads((scenes / "Third.epokmap").read_text())
    third["actors"][0]["components"].append(component(intro, "IntroPlayer", ["video"], dict(video=video)))
    documents.write_text(scenes / "Third.epokmap", documents.dumps(third))
    whole = out / "whole-game-ps-exe.zip"
    previous = whole.read_bytes()
    log = run("--project", project, "--export-ps-exe", "--debug-symbols", "--output", whole, success=False)
    assert "PS-EXE export cannot include video: assets/intro.avi" in log, log
    assert "remove the videos" in log and "MusicSequence" not in log.split("cannot include", 1)[1], log
    assert whole.read_bytes() == previous, "A failed export replaced the previous package"
    _, _, description = package("selected-after-video", *selected)
    assert description["scenes"] == ["assets/scenes/Main.epokmap", "assets/scenes/Second.epokmap"]
    print("PASS a video stops Whole Game by name; Selected Scenes without it still exports", flush=True)
    documents.write_text(out / "report.json", json.dumps(results, indent=2))
    print(f"PS-EXE scene, audio and video checks passed: {out}")


if __name__ == "__main__":
    main()
