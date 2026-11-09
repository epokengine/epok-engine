"""Compare project-owned C++/Blueprint/Lua run rules on the actual PSX runtime."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import uuid
from gameplay_probe import run_modes, export_probe
from verify_camera_rigs import ROOT, EDITOR, documents, write

BASE = "a0aa09c9-b6d3-5ae4-b6ab-dc99050463f4"
FLOW = "df3f7e3e-b99c-5530-9d86-a44f87e67c5f"
DRIVER = "2173a6f4-8f29-578c-9dca-053176d575c0"
MAGIC = 0x52554E46
STEPS, WORDS = 600, 10
NATIVE = ROOT / "resources/templates/dungeon-complete/assets/scripts/DungeonCombat"
FLAVORS = ROOT / "resources/templates/dungeon-complete-flavors"

SOURCE = r'''#pragma once
#include "DungeonRunFlowBase.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t dungeon_run_flow_probe[8]={};inline uint32_t dungeon_run_flow_trace[600][10]={};}
class EPOK_CLASS(Blueprintable,Id="2173a6f4-8f29-578c-9dca-053176d575c0") RunFlowProbe:public epok::Actor3D {
    uint32_t frame_=0,errors_=0,manual_=0,automatic_=0,gates_=0,invalid_=0,hash_=2166136261u;
    static bool same(const DungeonRunSnapshot& a,const DungeonRunSnapshot& b){
        return a.state==b.state&&a.phase==b.phase&&a.elapsed==b.elapsed&&a.fade==b.fade&&a.music_gain==b.music_gain
            &&a.playing==b.playing&&a.alive==b.alive&&a.restart==b.restart&&a.stop_music==b.stop_music&&a.valid==b.valid;
    }
    void check(bool condition,uint32_t bit){if(!condition)errors_|=bit;}
    void invalid_deltas(DungeonRunFlowBase& run){
        const auto before=run.status;
        for(auto dt:{epok::Fixed(0.0),epok::Fixed(-1.0),epok::Fixed(2.0)}){
            run.advance(dt,true);check(same(before,run.status),1u<<8);++invalid_;
        }
    }
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonRunFlowBase") epok::ObjectId flow{};
    void tick(epok::Fixed)override{
        if(frame_>=600)return;
        auto* run=epok::active_object_registry?epok::active_object_registry->resolve<DungeonRunFlowBase>(flow):nullptr;
        if(!run)return;
        const auto step=frame_%60;
        if(frame_<300){
            if(step==0){run->timing={1.0,1.0,2.0,2.0,1.0,2.0};run->reactor_entry_z=53.0;run->start();check(run->status.valid&&run->status.phase==0&&run->status.alive,1);}
            if(step==20)check(!run->try_enter_boss(false,54.0),2);
            if(step==21)check(!run->try_enter_boss(true,53.0),2);
            if(step==22){const bool entered=run->try_enter_boss(true,54.0);check(entered&&run->status.state==1,4);gates_+=entered;}
            if(step==23){run->win();check(run->status.state==2&&!run->status.alive,8);run->die();check(run->status.state==2,8);}
            if(step==24){check(!run->restart_requested(false)&&run->restart_requested(true),16);++manual_;run->start();}
            if(step==35){run->die();check(run->status.phase==5&&run->status.state==3,32);}
            if(step==36){const auto before=run->status;run->die();check(same(before,run->status),32);}
            if(step==45){run->timing.music_lead=0.0;run->timing.fade_in=0.0;run->timing.control_delay=0.0;run->start();}
            if(step==49)check(run->status.playing&&run->status.fade==0,64);
            if(step==50){run->die();invalid_deltas(*run);}
            if(step==51){run->reactor_entry_z=55.0;run->start();check(!run->try_enter_boss(true,54.0),128);const bool entered=run->try_enter_boss(true,56.0);check(entered,128);gates_+=entered;run->win();}
            if(step==55){run->timing.death_hold=-1.0;run->start();check(!run->status.valid,512);}
            if(step==56){run->timing.death_hold=2.0;run->start();check(run->status.valid,512);}
            if(step==58)invalid_deltas(*run);
        }else{
            if(frame_==300){run->timing={0.2,0.75,0.35,0.4,1.25,0.3};run->reactor_entry_z=55.0;run->start();}
            if(run->status.playing)run->die();
        }
        const epok::Fixed dt=frame_<300?epok::Fixed(1.0):epok::Fixed(frame_%3==0?68:frame_%3==1?69:137,epok::Fixed::RAW);
        run->advance(dt,frame_>=300||step>=10);
        const auto s=run->status;
        const uint32_t words[]={s.state,s.phase,uint32_t(s.elapsed.raw()),s.fade,uint32_t(s.music_gain.raw()),s.playing,s.alive,s.restart,s.stop_music,s.valid};
        for(unsigned i=0;i<10;++i){dungeon_run_flow_trace[frame_][i]=words[i];hash_=(hash_^words[i])*16777619u;}
        if(s.restart){++automatic_;run->start();}
        if(++frame_==600){auto* p=dungeon_run_flow_probe;p[1]=frame_;p[2]=errors_;p[3]=manual_;p[4]=automatic_;p[5]=gates_;p[6]=hash_;p[7]=invalid_;p[0]=0x52554E46u+EPOK_LUA_MODE;}
    }
};
'''


def verify(out, flavor, modes, prepare_only, standalone_only):
    out = out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    counter = 0

    def run(*args, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        export_probe(out, project, run, ["gameplay_api.hpp"])
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Run Flow " + flavor)
    scripts = project / "assets/scripts"
    scripts.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(NATIVE / "DungeonRunFlowBase.hpp", scripts / "DungeonRunFlowBase.hpp")
    shutil.copyfile(NATIVE / "DungeonRulesBinding.hpp", scripts / "DungeonRulesBinding.hpp")
    if flavor == "cpp":
        shutil.copyfile(NATIVE / "DungeonRunFlow.hpp", scripts / "DungeonRunFlow.hpp")
    elif flavor == "blueprint":
        write(project / "assets/Blueprints/DungeonRunFlow.epokbp",
              (FLAVORS / "blueprint/DungeonRunFlow.epokbp").read_text(encoding="utf-8"))
    else:
        write(scripts / "DungeonRunFlow.lua", (FLAVORS / "lua/DungeonRunFlow.lua").read_text(encoding="utf-8"))
        write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/DungeonRunFlow.lua": FLOW}})
    assert flavor == "cpp" or not (scripts / "DungeonRunFlow.hpp").exists()
    write(scripts / "RunFlowProbe.hpp", SOURCE)
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    root = copy.deepcopy(next(c for c in scene["actors"][0]["components"] if c.get("root")))
    root["id"] = str(uuid.uuid5(uuid.UUID(DRIVER), "root"))
    flow = str(uuid.uuid5(uuid.UUID(DRIVER), "flow"))
    scene["actors"] = [{"id": str(uuid.uuid5(uuid.UUID(DRIVER), "actor")), "name": "Run flow probe",
        "class": {"name": "RunFlowProbe", "class_id": DRIVER}, "properties": {"flow": flow},
        "overrides": ["flow"], "components": [root, {"id": flow, "name": "Run rules",
        "class": {"name": "DungeonRunFlow", "class_id": FLOW}, "properties": {}, "overrides": []}]}]
    write(scene_path, scene)
    run("--project", project, "--reflect", label="reflection.log")
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    # Lua is compiled by the real Play/Build pipeline below; there is no
    # standalone --compile-lua command in the editor CLI.
    if prepare_only:
        print("Prepared", flavor, flush=True)
        return

    def check(values, expected):
        assert values[0] == expected and values[1:4] == [STEPS, 0, 5], values
        assert values[4] >= 6 and values[5] == 10 and values[7] == 30, values
    selected_modes = modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",))
    run_modes(out, project, flavor, selected_modes, "dungeon_run_flow_probe", MAGIC, check,
              provider_count=1, trace=("dungeon_run_flow_trace", STEPS * WORDS))
    reference = None
    for mode in selected_modes:
        path = out / (mode + "-dungeon_run_flow_trace.json")
        words = json.loads(path.read_text())
        assert len(words) == STEPS * WORDS
        if reference is not None:
            assert words == reference, (flavor, mode, "execution modes disagree")
        reference = words
    (out / "source-inventory.json").write_text(json.dumps({
        str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(project.glob("assets/scripts/*")) if p.is_file()}, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/run-flow/providers")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.output.resolve(), flavor, args.mode, args.prepare_only, args.standalone_only)
    if not args.prepare_only and not args.standalone_only:
        traces = list(args.output.resolve().glob("*/native_cpp-dungeon_run_flow_trace.json"))
        if len(traces) == 3:
            expected = json.loads(traces[0].read_text())
            for path in traces[1:]:
                actual = json.loads(path.read_text())
                if actual != expected:
                    index = next(i for i, (a, b) in enumerate(zip(actual, expected)) if a != b)
                    raise AssertionError((path, "provider trace mismatch", index // WORDS, index % WORDS, actual[index], expected[index]))
            print("PASS all three project policies: 6,000 exact Q12/state words agree", flush=True)


if __name__ == "__main__":
    main()
