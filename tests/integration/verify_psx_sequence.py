"""Cook a portable MIDI/bank and measure the real PSX IRQ/SPU path in PCSX-Redux."""
from pathlib import Path
import json
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

EXE = ROOT / "target/debug/epok-editor.exe"
ART = ROOT / "artifacts"
PROFILE = sys.argv[1] if len(sys.argv) > 1 else "midi"
assert PROFILE in ("midi", "sony", "converted")
if PROFILE != "midi":
    ART = ART / f"audio-phase-e-emulator-{PROFILE}-{time.time_ns()}"
    ART.mkdir(parents=True)
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)


def main():
    project = ROOT / ".epok" / f"psx-sequence-{time.time_ns()}"
    print(f"Project: {project}", flush=True)
    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True, text=True,
                                timeout=240, creationflags=FLAGS)
        with (ART / "audio-phase-d-emulator-commands.log").open("a", encoding="utf-8") as log:
            log.write(f"{args}\nexit={result.returncode}\n{result.stdout}{result.stderr}\n")
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        return result.stdout

    run("--create-project", project, "--template", "sample")
    run("--project", project, "--create-starter-bank", "assets/Retro.epokasset")
    records = json.loads(run("--project", project, "--scan-assets"))["assets"]
    ids = {r["path"]: r["id"] for r in records}
    bank = ids["assets/Retro.epokasset"]
    # Owned fixture: one note at tick zero, off at 96 PPQN / 0.5 seconds.
    midi = bytes.fromhex("4d546864000000060000000100604d54726b0000000c00903c6460803c0000ff2f00")
    source = "assets/song.mid"
    selection = []
    if PROFILE == "sony":
        midi = bytes.fromhex("7051455300000001006007a120040200903c6460803c0000ff2f00")
        source = "assets/song.seq"
        selection = ["--sequence-profile", "sony-seq-v1", "--song-id", "0"]
    elif PROFILE == "converted":
        events = bytes.fromhex("903c6460803c0000ff2f00")
        size = (12 + len(events) + 3) & ~3
        midi = struct.pack("<IIHBB", size, 500000, 96, 4, 2) + events
        midi += bytes(size - len(midi))
        source = "assets/song.sep"
        selection = ["--sequence-profile", "converted-seq-le32-v1", "--song-index", "0"]
    (project / source).write_bytes(midi)
    run("--project", project, "--import-audio", source, "--sound-bank", bank,
        "--sequence-loop", "whole", "--voice-limit", "16", *selection)
    records = json.loads(run("--project", project, "--scan-assets"))["assets"]
    ids = {r["path"]: r["id"] for r in records}
    song = ids["assets/song.epokasset"]
    sample = ids["assets/Retro-triangle.epokasset"]
    run("--project", project, "--reimport-asset", "assets/Retro-triangle.epokasset", "--snapshot", "--loop")
    scene_path = project / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())
    spinner = next(i for i, e in enumerate(scene["entities"]) if (e.get("script") or {}).get("name") == "Spinner")
    scene["entities"][spinner]["audio"] = dict(clip=song, volume=0.5)
    scene["entities"][0]["audio"] = dict(clip=sample, volume=0.1, play_on_start=False)
    documents.write_text(scene_path, json.dumps(scene))
    documents.write_text(project / "assets/scripts/Spinner.cpp", r'''
#include "Spinner.hpp"
#include "common/hardware/spu.h"
#include "common/hardware/dma.h"
#include "common/hardware/counters.h"
uint32_t sequence_probe[12]={};
alignas(4) int16_t sequence_capture[20][512]={};
epok::AudioSource sequence_sfx[26];
void Spinner::start(epok::Transform&){
    entity().audio.stop();
    auto& silent=sequence_sfx[0];silent.enabled=true;silent.clip=DUMMY_CLIP;
    silent.volume=0.0;silent.priority=255;silent.play();
    entity().audio.play();
}
void Spinner::update(epok::Transform&,epok::Fixed){
    auto& source=entity().audio;
    const auto frame=++sequence_probe[0];
    if(frame==40){source.stop();sequence_probe[1]=!source.is_playing();}
    if(frame==45){source.play();sequence_probe[2]=source.is_playing();}
    if(frame>=70 && frame<90){
        sequence_probe[3]=source.is_playing();
        uint32_t active=0;for(int i=0;i<24;++i)active+=SPU_VOICES[i].currentVolume!=0;
        sequence_probe[4]=active;sequence_probe[5]=SPU_CTRL;
        sequence_probe[11]=SPU_VOICES[1].sampleRate;
        // Capture voice 1 (hardware capture region). Musical notes use the first
        // free voice; the dummy silent sample occupies voice 0 only for this probe.
        SPU_RAM_DTA=0x800>>3;SPU_CTRL=0xc030;
        DMA_CTRL[DMA_SPU].MADR=uint32_t(uintptr_t(sequence_capture[frame-70]));
        DMA_CTRL[DMA_SPU].BCR=(16<<16)|16;DMA_CTRL[DMA_SPU].CHCR=0x01000200;
        uint32_t timeout=1000000;while((DMA_CTRL[DMA_SPU].CHCR&0x01000000) && --timeout){}
        sequence_probe[6]=timeout!=0;SPU_CTRL=0xc000;
    }
    if(frame==95){source.enabled=false;}
    if(frame==97){sequence_probe[7]=!source.is_playing();source.enabled=true;source.play();}
    sequence_probe[8]=COUNTERS[0].mode;sequence_probe[9]=COUNTERS[1].mode;sequence_probe[10]=COUNTERS[2].mode;
}
'''.replace("DUMMY_CLIP", str(sorted([song, sample]).index(sample))))
    run("--project", project, "--build-psx")
    build = project / ".epok/build"
    report = json.loads((build / "audio/sequence-report.json").read_text())
    assert report[song]["spu_ram_bytes"] == 1344
    assert (build / f"audio/{song}.epsq").read_bytes()[:4] == b"EPSQ"
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))
    def request(path, post=False):
        req = urllib.request.Request("http://127.0.0.1:8077/api/v1/" + path, data=b"" if post else None)
        with urllib.request.urlopen(req, timeout=3) as response:
            return response.read()
    with (ART / "audio-phase-d-emulator.log").open("w") as log:
        process = subprocess.Popen([str(EXE), "--project", str(project), "--play-psx", "--stop-after", "9"],
                                   stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 80
            while True:
                assert process.poll() is None, "Build/boot failed: audio-phase-d-emulator.log"
                try:
                    if json.loads(request("execution-flow"))["running"]:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "Emulator did not start"
                time.sleep(0.1)
            time.sleep(3.5)
            request("execution-flow?function=pause", True)
            symbols = (build / "epok.map").read_text()
            ram = request("cpu/ram/raw")
            def address(name):
                found = re.search(r"0x([0-9a-f]+)\s+" + re.escape(name) + r"\b", symbols)
                assert found, name
                return int(found[1], 16) & 0x1fffff
            probe = struct.unpack_from("<12I", ram, address("sequence_probe"))
            stats = struct.unpack_from("<16I", ram, address("epok::music_sequence_stats"))
            decoded = struct.unpack_from("<10240h", ram, address("sequence_capture"))
            peak = max(map(abs, decoded))
            (ART / "audio-phase-d-sequence-capture.pcm").write_bytes(struct.pack("<10240h", *decoded))
            data = {"project": str(project), "source_profile": PROFILE, "probe": probe, "stats": stats, "cook": report,
                    "irq_max_us": stats[14] * 625 / 2646, "decoded_peak": peak}
            (ART / "audio-phase-d-emulator-evidence.json").write_text(json.dumps(data, indent=2))
            print(json.dumps(data, indent=2), flush=True)
            assert probe[0] > 100 and probe[1:4] == (1, 1, 1) and probe[7] == 1, probe
            assert stats[0] == 1 and stats[1] > 1000 and stats[2] >= 3 and stats[4] >= 3, stats
            assert stats[11] == 0 and stats[12] == 0, stats
            assert stats[14] > 0 and stats[13] < 14000, stats
            assert peak > 500 and min(decoded) < -500 and max(decoded) > 500, ("No sequenced decoded tone in SPU capture", peak)
            assert probe[11] == 1217, ("Capture must be the MIDI C4 instrument pitched from the A4 bank root, not an SFX", probe)
            assert process.wait(timeout=20) == 0
            print("PASS PSX sequence IRQ clock, loop, stop/restart, disable and resident bank playback", flush=True)
        finally:
            if process.poll() is None:
                process.wait(timeout=45)


if __name__ == "__main__":
    main()
