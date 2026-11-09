"""Exercise changing reveal counts through the real two-parity PSX HUD cache."""
import argparse
import json
from pathlib import Path
import subprocess
from gameplay_probe import run_modes, export_probe
from verify_camera_rigs import ROOT, EDITOR, documents, write

DRIVER = "ae682b37-581c-514d-80d0-6e5b15a86c6e"
MAGIC = 0x48554452
SOURCE = r'''#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t hud_reveal_cache_probe[8]={};}
class EPOK_CLASS(Blueprintable,Id="ae682b37-581c-514d-80d0-6e5b15a86c6e") HudRevealCacheProbe:public epok::DialoguePresenter {
    uint32_t frames_=0,errors_=0,states_=0,sum_=0;
public:
    void begin_play()override{data()->text.set_text("\xc3\x81[color=#ff0000]BC[/color]\nD");data()->text.enabled=true;data()->text.visible_glyphs=0;}
    void on_frame(uint32_t)override{
        if(frames_>=48)return;
        constexpr uint16_t limits[]={0,1,3,2,4,0xffffu,0,4};
        const auto frame=frames_++,phase=frame/6;
        if(frame%6==0)data()->text.visible_glyphs=limits[phase];
        if(frame%6==5){
            const auto expected=limits[phase]==0xffffu?4:limits[phase];
            if(epok::hud_stats.glyphs!=expected||epok::hud_stats.dropped)errors_|=1u<<phase;
            states_|=1u<<phase;sum_+=epok::hud_stats.glyphs;
        }
        if(frames_==48){auto* p=hud_reveal_cache_probe;p[1]=frames_;p[2]=errors_;p[3]=states_;p[4]=sum_;p[5]=epok::hud_stats.glyphs;p[6]=epok::hud_stats.dropped;p[7]=1;p[0]=0x48554452u;}
    }
};
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/flow-performance/reveal-cache")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    counter = 0

    def run(*command, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, command)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if args.standalone_only:
        export_probe(out, project, run, ["hud.hpp", "hud_core.hpp"])
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Reveal Cache")
    write(project / "assets/scripts/HudRevealCacheProbe.hpp", SOURCE)
    descriptor = next(project.glob("*.epokproject"))
    scene_path = project / documents.loads(descriptor.read_text(encoding="utf-8"))["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))

    def component(name, class_id, properties=None, root=False, inherited=None):
        value = {"id": class_id, "name": name, "class": {"name": "epok::" + name, "class_id": class_id},
                 "properties": properties or {}, "overrides": list(properties or {}), "root": root}
        if inherited:
            value.update(inherited=True, default_id=inherited)
        return value

    scene["actors"] = [{"id": DRIVER, "name": "Reveal Cache", "class": {"name": "HudRevealCacheProbe", "class_id": DRIVER},
        "properties": {}, "overrides": [], "components": [
            component("RectTransformComponent", "dc805165-6c65-48dc-8ff8-4a638a5d21df", root=True),
            component("CanvasComponent", "f2cfb26b-af53-4a46-9d91-1debea72e01b", inherited="0762122d-cb6b-52f6-8fb1-3287494801fc"),
            component("AudioComponent", "7f0eb028-5301-4ac7-b93b-5665fab12b20", inherited="b0e900ca-43e6-5bdf-bdbd-a52f0416b36b"),
            component("TextComponent", "fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34")]}]
    scene.pop("bake", None)
    write(scene_path, scene)

    def check(values, expected):
        assert values == [expected, 48, 0, 255, 18, 4, 0, 1], values

    run_modes(out, project, "retained HUD reveal", ("native_cpp",), "hud_reveal_cache_probe", MAGIC,
              check, provider_count=1)
    sources = (project / ".epok/build/sources.mk").read_text()
    assert "EPOK_DIALOGUE" in sources
    print("PASS reveal, shrink and unchanged draws across both GPU parities", flush=True)


if __name__ == "__main__":
    main()
