"""Portable asset recovery and actual SPU playback through the MIPS toolchain/PCSX-Redux."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import json, math, pathlib, re, shutil, socket, struct, subprocess, time, urllib.request, wave

ROOT = pathlib.Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
ART = ROOT / "artifacts"
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0


def main():
    ART.mkdir(exist_ok=True)
    folder = ROOT / ".epok" / ("audio-verify-" + str(time.time_ns()))

    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True, text=True,
                                timeout=120, creationflags=FLAGS)
        with (ART / "audio-assets.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        return result.stdout

    run("--create-project", folder, "--template", "sample")
    source = folder / "assets/tone.wav"
    with wave.open(str(source), "wb") as wav:
        wav.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        wav.writeframes(b"".join(struct.pack("<h", round(math.sin(i * 2 * math.pi * 440 / 22050) * 12000)) for i in range(2205)))
    run("--project", folder, "--import-audio", "assets/tone.wav", "--loop")
    run("--project", folder, "--import-audio", "assets/tone.wav", "--asset", "assets/once.epokasset")
    index = json.loads(run("--project", folder, "--scan-assets"))
    ids = {r["path"]: r["id"] for r in index["assets"]}
    clip = ids["assets/tone.epokasset"]
    once = ids["assets/once.epokasset"]
    scene_path = folder / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())
    scene["entities"][0]["audio"] = dict(clip=clip, volume=0.25)
    scene["entities"][1]["audio"] = dict(clip=clip)
    scene["entities"][2]["audio"] = dict(clip=once)
    documents.write_text(scene_path, json.dumps(scene))
    assert not (folder / ".epok/imported").exists(), "Import must not cook a target"
    run("--project", folder, "--build-psx")
    original_scene = scene_path.read_bytes()
    moved = folder / "assets/Sound/moved.epokasset"
    moved.parent.mkdir()
    (folder / "assets/tone.epokasset").rename(moved)
    source.unlink()
    cache = (folder / ".epok/imported").resolve()
    assert cache.is_relative_to(folder.resolve()) and folder.resolve().is_relative_to(ROOT.resolve())
    shutil.rmtree(cache)
    run("--project", folder, "--build-psx")
    assert scene_path.read_bytes() == original_scene
    assert any(r["id"] == clip for r in json.loads(run("--project", folder, "--scan-assets"))["assets"])
    print("PASS closed-project asset move, missing source, cache reconstruction and UUID references", flush=True)

    duplicate = folder / "assets/copy.epokasset"
    shutil.copyfile(moved, duplicate)
    run("--project", folder, "--build-psx", success=False)
    duplicate.unlink()
    run("--project", folder, "--reimport-asset", "assets/Sound/moved.epokasset", "--snapshot", "--rate", "22050")
    print("PASS duplicate UUID blocks build; snapshot reimport preserves identity", flush=True)

    # This is a normal separately compiled game script. It probes emulated SPU registers
    # and DMA-reads Voice 1's decoded capture buffer (PSX SPU memory 0x800..0xbff).
    documents.write_text(folder / "assets/scripts/Spinner.cpp", r'''
#include "Spinner.hpp"
#include "common/hardware/spu.h"
#include "common/hardware/dma.h"
uint32_t audio_probe[12]={};
alignas(4) int16_t audio_capture[20][512]={};
epok::AudioSource crowded[26];
void Spinner::start(epok::Transform&){}
void Spinner::update(epok::Transform&,epok::Fixed){
    auto& source=entity().audio;
    uint32_t frame=++audio_probe[0];
    if(frame==40){source.stop();audio_probe[1]=!source.is_playing();}
    if(frame==45){source.play();audio_probe[2]=source.is_playing();source.volume=0.5;source.pitch=0.5;}
    if(frame>=70 && frame<90){
        audio_probe[3]=source.is_playing();
        audio_probe[4]=SPU_VOICES[1].volumeLeft;
        audio_probe[5]=SPU_VOICES[1].sampleRate;
        audio_probe[6]=SPU_VOICES[1].currentVolume;
        audio_probe[7]=SPU_CTRL;
        audio_probe[8]=SPU_VOICES[2].currentVolume;
        SPU_RAM_DTA=0x800>>3;SPU_CTRL=0xc030;
        DMA_CTRL[DMA_SPU].MADR=(uint32_t)audio_capture[frame-70];
        DMA_CTRL[DMA_SPU].BCR=(16<<16)|16;
        DMA_CTRL[DMA_SPU].CHCR=0x01000200;
        uint32_t timeout=1000000;
        while((DMA_CTRL[DMA_SPU].CHCR&0x01000000) && --timeout){}
        audio_probe[9]=timeout!=0;SPU_CTRL=0xc000;
    }
    if(frame==90){
        for(int i=0;i<25;++i){crowded[i]=source;crowded[i].volume=0.0;crowded[i].priority=128;crowded[i].play();}
        for(int i=0;i<25;++i)if(crowded[i].is_playing())++audio_probe[10];
        crowded[25]=source;crowded[25].volume=0.0;crowded[25].priority=0;crowded[25].play();
        if(!crowded[25].is_playing())audio_probe[11]=1;
        crowded[25].priority=255;crowded[25].play();
        if(crowded[25].is_playing())audio_probe[11]=audio_probe[11]|2;
    }
}
''')
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))

    def request(path, post=False):
        req = urllib.request.Request("http://127.0.0.1:8077/api/v1/" + path, data=b"" if post else None)
        with urllib.request.urlopen(req, timeout=3) as response:
            return response.read()

    with (ART / "audio-emulator.log").open("w") as log:
        process = subprocess.Popen([str(EXE), "--project", str(folder), "--play-psx", "--stop-after", "8"],
                                   stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 45
            while True:
                if process.poll() is not None:
                    raise AssertionError("Build/boot failed: artifacts/audio-emulator.log")
                try:
                    if json.loads(request("execution-flow"))["running"]:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "Emulator did not start"
                time.sleep(0.1)
            time.sleep(2.5)
            request("execution-flow?function=pause", True)
            symbols = (folder / ".epok/build/epok.map").read_text()
            ram = request("cpu/ram/raw")

            def address(name):
                return int(re.search(r"0x([0-9a-f]+)\s+" + name + r"\b", symbols)[1], 16) & 0x1fffff

            values = struct.unpack_from("<12I", ram, address("audio_probe"))
            documents.write_text(ART / "audio-probe.json", json.dumps(values))
            assert values[0] >= 90, values
            assert values[1:4] == (1, 1, 1), ("stop/play/loop", values)
            assert values[4:6] == (8192, 1024), ("volume/pitch", values)
            assert values[6] > 0 and values[7] == 0xc000, ("SPU envelope/output", values)
            assert values[8] == 0, ("one-shot did not finish", values)
            assert values[9] == 1, ("SPU capture DMA failed", values)
            assert values[10:] == (24,3), ("24-voice budget and priority", values)
            # Redux's voice capture can contain only a short nonzero window.
            # Sample several frames so signed-tone detection does not depend on
            # the waveform phase of a single capture.
            decoded = struct.unpack_from("<10240h", ram, address("audio_capture"))
            peak = max(map(abs, decoded))
            (ART / "audio-capture.pcm").write_bytes(struct.pack("<10240h", *decoded))
            assert 1000 < peak < 20000 and min(decoded) < -1000 and max(decoded) > 1000, ("No decoded tone", peak)
            assert process.wait(timeout=15) == 0
            print(f"PASS real SPU decoded audio (peak {peak}), loop, one-shot, script stop/play, volume/pitch and 24-voice priority", flush=True)
        finally:
            if process.poll() is None:
                process.wait(timeout=45)


if __name__ == "__main__":
    main()
