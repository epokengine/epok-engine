"""Compare independently authored player-vitality policies on the PSX runtime."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import uuid

from gameplay_probe import export_probe, run_modes
from verify_camera_rigs import ROOT, EDITOR, documents, write

BASE = "60930832-058d-5a4d-a0ef-9fde884c7396"
CLASS = "070a2ac6-e00f-5e9c-8bd2-560acb25ad2f"
DRIVER = str(uuid.uuid5(uuid.UUID(CLASS), "acceptance-driver"))
MAGIC, STEPS, WORDS = 0x56495441, 480, 14
NATIVE = ROOT / "resources/templates/dungeon-complete/assets/scripts/DungeonCombat"
FLAVORS = ROOT / "resources/templates/dungeon-complete-flavors"


def case(action, amount=0, enabled=True, delta=0, tuning=None, invalid=False):
    return (action, amount, enabled, delta, tuning is not None, *(tuning or (0, 0)), invalid)


CASES = [
    case(1, tuning=(100, 2867)), case(3, 12), case(3, 20), case(2, delta=4096),
    case(3, 12, enabled=False), case(4, 35), case(3, 2147483647), case(3, 20),
    case(4, 1), case(2, delta=2867), case(3, 1), case(4, -35), case(3, -20),
    case(2, delta=0), case(2, delta=-4096), case(2, delta=4506),
    case(1, tuning=(60, 2867)), case(3, 12), case(2, delta=2866), case(3, 20),
    case(2, delta=1), case(3, 20), case(4, 35), case(2, delta=3000),
    case(1, tuning=(4294967295, 2867)), case(3, 2147483647), case(2, delta=2867),
    case(3, 2147483647), case(2, delta=2867), case(3, 1), case(4, 2147483647),
    case(4, 2147483647), case(4, 1), case(1, tuning=(280, 0)), case(3, 20), case(3, 20),
    case(3, 20, invalid=True), case(4, 35), case(1, tuning=(0, 2867)), case(4, 35), case(3, 20),
    case(1, tuning=(100, -410)), case(3, 20), case(1, tuning=(100, 246170)), case(3, 20),
    case(1, tuning=(100, 2867)), case(3, 12),
]
CASES += [case(2, delta=(68, 69, 137)[n % 3]) for n in range(47, 60)]
assert len(CASES) == 60


def source():
    rows = ["{" + ",".join(str(int(value)) + ("u" if index == 5 else "")
                            for index, value in enumerate(row)) + "}" for row in CASES]
    return r'''#pragma once
#include "DungeonPlayerVitalsBase.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t dungeon_vitals_probe[8]={};inline uint32_t dungeon_vitals_trace[480][14]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER") DungeonVitalsProbe:public epok::Actor3D {
    uint32_t frame_=0,hash_=2166136261u;
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerVitalsBase") epok::ObjectId rules{};
    void on_frame(uint32_t)override{
        if(frame_>=480)return;
        auto* policy=epok::active_object_registry?epok::active_object_registry->resolve<DungeonPlayerVitalsBase>(rules):nullptr;
        if(!policy)return;
        if(!frame_&&(policy->tuning.maximum!=100||policy->tuning.hit_immunity.raw()!=2867||policy->health.maximum!=100||policy->health.current!=100||!policy->valid||policy->immunity.raw()))dungeon_vitals_probe[2]|=1;
        struct Case{uint32_t action;int32_t amount;bool enabled;int32_t delta;bool configure;uint32_t maximum;int32_t immunity;bool invalid;};
        static constexpr Case cases[]={CASES};
        const auto c=cases[frame_%60];
        if(c.configure)policy->tuning={c.maximum,epok::Fixed(c.immunity,epok::Fixed::RAW)};
        if(c.invalid)policy->assign({100,101},0.0,true);
        epok::HealthChange change;
        if(c.action==1)policy->start();
        else if(c.action==2){policy->advance(epok::Fixed(c.delta,epok::Fixed::RAW));++dungeon_vitals_probe[7];}
        else if(c.action==3){change=policy->damage(c.amount,c.enabled);++dungeon_vitals_probe[5];}
        else if(c.action==4){change=policy->heal(c.amount);++dungeon_vitals_probe[6];}
        const uint32_t words[]={frame_,policy->health.maximum,policy->health.current,uint32_t(policy->immunity.raw()),policy->valid,policy->can_heal(),
            change.state.maximum,change.state.current,change.applied,change.depleted,change.revived,uint32_t(change.result),c.action,c.enabled};
        for(unsigned n=0;n<14;++n){dungeon_vitals_trace[frame_][n]=words[n];hash_=(hash_^words[n])*16777619u;}
        ++dungeon_vitals_probe[3];
        if(++frame_==480){dungeon_vitals_probe[1]=frame_;dungeon_vitals_probe[4]=hash_;dungeon_vitals_probe[0]=0x56495441u+EPOK_LUA_MODE;}
    }
};
'''.replace('"DRIVER"', '"' + DRIVER + '"').replace("CASES", ",\n".join(rows))


def oracle():
    maximum, current, immunity, available, limit, duration = 100, 100, 0, True, 100, 2867
    words = []
    for frame in range(STEPS):
        action, amount, enabled, delta, configure, next_max, next_duration, invalid = CASES[frame % len(CASES)]
        if configure:
            limit, duration = next_max, next_duration
        if invalid:
            maximum, current, immunity, available = 100, 101, 0, True
        change_max, change_current, applied, depleted, revived, result = 0, 0, 0, False, False, 0
        if action == 1:
            maximum, current, immunity = limit, limit, 0
            available = limit > 0 and 0 <= duration <= 60 * 4096
        elif action == 2:
            if available and 0 < delta <= 4096 and immunity > 0:
                immunity = max(0, immunity - delta)
        else:
            positive = max(0, amount) if available and (action == 4 or enabled and immunity <= 0) else 0
            change_max, change_current = maximum, current
            if current > maximum:
                result = 2
            else:
                applied = min(positive, maximum - current if action == 4 else current)
                change_current = current + (applied if action == 4 else -applied)
                depleted = action == 3 and current > 0 and change_current == 0
                revived = action == 4 and current == 0 and change_current > 0
                result = int(applied > 0)
                if applied:
                    current = change_current
                    if action == 3:
                        immunity = duration
        can_heal = available and current <= maximum and current < maximum
        words += [frame, maximum, current, immunity & 0xffffffff, int(available), int(can_heal),
                  change_max, change_current, applied, int(depleted), int(revived), result, action, int(enabled)]
    return words


def verify(out, flavor, modes, prepare_only, standalone_only):
    out = out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project, counter = out / "project", 0

    def run(*args, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        manifest_path = next(project.glob("*.epokproject"))
        manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["lua_execution"] = "native_cpp"
        write(manifest_path, manifest)
        run("--project", project, "--build-psx", label="native-export-build.log")
        export_probe(out, project, run, ["health.hpp"])
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Player Vitals " + flavor)
    scripts = project / "assets/scripts"
    for name in ("DungeonRulesBinding.hpp", "DungeonPlayerVitalsBase.hpp"):
        shutil.copyfile(NATIVE / name, scripts / name)
    if flavor == "cpp":
        shutil.copyfile(NATIVE / "DungeonPlayerVitals.hpp", scripts / "DungeonPlayerVitals.hpp")
    elif flavor == "blueprint":
        write(project / "assets/Blueprints/DungeonPlayerVitals.epokbp",
              (FLAVORS / "blueprint/DungeonPlayerVitals.epokbp").read_text(encoding="utf-8"))
    else:
        write(scripts / "DungeonPlayerVitals.lua", (FLAVORS / "lua/DungeonPlayerVitals.lua").read_text(encoding="utf-8"))
        write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/DungeonPlayerVitals.lua": CLASS}})
    assert flavor == "cpp" or not (scripts / "DungeonPlayerVitals.hpp").exists()
    write(scripts / "DungeonVitalsProbe.hpp", source())
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    identity = lambda name: str(uuid.uuid5(uuid.UUID(DRIVER), name))
    root = copy.deepcopy(next(c for c in scene["actors"][0]["components"] if c.get("root")))
    root["id"] = identity("root")
    scene["actors"] = [{"id": identity("actor"), "name": "Vitals probe",
        "class": {"name": "DungeonVitalsProbe", "class_id": DRIVER}, "properties": {"rules": identity("rules")},
        "overrides": ["rules"], "components": [root, {"id": identity("rules"), "name": "Player vitals",
        "class": {"name": "DungeonPlayerVitals", "class_id": CLASS}, "properties": {}, "overrides": []}]}]
    write(scene_path, scene)
    run("--project", project, "--reflect", label="reflection.log")
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    if prepare_only:
        print("Prepared", flavor, flush=True)
        return
    expected_words = oracle()
    expected_hash = 2166136261
    for word in expected_words:
        expected_hash = ((expected_hash ^ word) * 16777619) & 0xffffffff
    counts = [sum(row[0] == action for row in CASES) * 8 for action in (3, 4, 2)]

    def check(values, expected):
        assert values == [expected, STEPS, 0, STEPS, expected_hash, *counts], values

    selected = modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",))
    run_modes(out, project, flavor, selected, "dungeon_vitals_probe", MAGIC, check,
              provider_count=1, trace=("dungeon_vitals_trace", STEPS * WORDS))
    for mode in selected:
        actual = json.loads((out / (mode + "-dungeon_vitals_trace.json")).read_text())
        if actual != expected_words:
            index = next(n for n, (a, b) in enumerate(zip(actual, expected_words)) if a != b)
            raise AssertionError((flavor, mode, index // WORDS, index % WORDS, actual[index], expected_words[index]))
    write(out / "source-inventory.json", {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
          for p in scripts.iterdir() if p.is_file()})
    print("PASS", flavor, "6,720 exact vitality/oracle words", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/player-vitals/providers")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.output.resolve(), flavor, args.mode, args.prepare_only, args.standalone_only)


if __name__ == "__main__":
    main()
