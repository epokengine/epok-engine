"""Compile actual Blueprint inheritance/events/Delay; verify MIPS, export and RAM."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse
import contextlib
import json
import os
from pathlib import Path
import re
import shutil
import socket
import struct
import subprocess
import tempfile
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "target/debug/epok-editor.exe"


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, data if isinstance(data, str) else json.dumps(data, indent=2), encoding="utf-8")


def run(*args, ok=True):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                            text=True, encoding="utf-8", errors="replace", timeout=180)
    if (result.returncode == 0) != ok:
        raise AssertionError(result.stdout + result.stderr)
    return result.stdout if ok else result.stdout + result.stderr


def identity():
    return str(uuid.uuid4())


def literal(kind, value):
    return {"kind": "literal", "value_type": {"kind": kind}, "value": value}


def node(kind, inputs=None, **settings):
    return {"id": identity(), "kind": {"kind": kind, **settings}, "inputs": inputs or {}, "outputs": {}}


def graph(function, nodes, own_id=None):
    entry = node("entry")
    nodes = [entry, *nodes, node("return")]
    for previous, following in zip(nodes, nodes[1:]):
        previous["outputs"]["next"] = [following["id"]]
    return {"id": own_id or identity(), "name": function["name"], "override_id": function["id"],
            "parameters": function["parameters"], "returns": function["returns"], "entry": entry["id"], "nodes": nodes}


def emulator(project):
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 8077))
    log_path = project / "blueprint-emulator.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "8"], cwd=ROOT, stdout=log, stderr=log)
        values = None
        try:
            deadline = time.monotonic() + 120
            while time.monotonic() < deadline and process.poll() is None:
                try:
                    symbols = (project / ".epok/build/epok.map").read_text()
                    found = re.search(r"0x([0-9a-fA-F]+)\s+blueprint_graph_probe\b", symbols)
                    if found:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                            values = struct.unpack_from("<16i", response.read(), int(found.group(1), 16) & 0x1fffff)
                        if values[15] == 1:
                            break
                except (OSError, ValueError):
                    pass
                time.sleep(0.1)
            assert values and values[0] == 0x42504731 and values[15] == 1, (values, log_path.read_text(errors="replace"))
            assert values[1] == 6, values
            assert values[2:4] == (80 * 4096, 150 * 4096), values
            assert values[4:6] == (5, 9), values
            assert values[6:8] == (-925 * 4096, -855 * 4096), values
            assert values[8:11] == (6, 6, 6), values
            assert values[12] == 0 and values[13] > 0 and values[14] > 0, values
            assert process.wait(timeout=40) == 0, log_path.read_text(errors="replace")
        finally:
            if process.poll() is None:
                process.wait(timeout=45)
    return values


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    parser.add_argument("--keep", action="store_true")
    args = parser.parse_args()
    with (contextlib.nullcontext(tempfile.mkdtemp(prefix="epok-blueprints-")) if args.keep else tempfile.TemporaryDirectory(prefix="epok-blueprints-")) as directory:
        root = Path(directory) / "Blueprint Game"
        if args.keep:
            print(f"Retained project: {root}", flush=True)
        run("--create-project", root, "--name", "Blueprint Acceptance")
        enemy_id, health_id, reward_id = identity(), identity(), identity()
        source = '''#pragma once
#include "epok.hpp"
extern "C" { inline int32_t blueprint_graph_probe[16]={}; }
class EPOK_CLASS(Blueprintable, Id="CLASS") Enemy : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere, Id="HEALTH") epok::Fixed health=100.0;
    EPOK_PROPERTY(EditAnywhere, Id="REWARD") int32_t reward=0;
    EPOK_FUNCTION(BlueprintCallable) void take_damage(epok::Fixed amount) {health-=amount; if(health<=0.0) on_death();}
    EPOK_FUNCTION(BlueprintCallable) void record() {
        unsigned slot=entity().name[0]=='B'?1:0;
        blueprint_graph_probe[4+slot]=reward; blueprint_graph_probe[6+slot]=health.raw(); ++blueprint_graph_probe[10];
    }
    EPOK_FUNCTION(BlueprintEvent) virtual void on_ready() {++blueprint_graph_probe[8];}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_death() {++blueprint_graph_probe[9];}
    int ticks=0;
    void start(epok::Transform&) override {
        unsigned slot=entity().name[0]=='B'?1:0;
        blueprint_graph_probe[0]=0x42504731; ++blueprint_graph_probe[1];
        blueprint_graph_probe[2+slot]=health.raw(); if(ticks)++blueprint_graph_probe[12]; on_ready();
    }
    void update(epok::Transform& transform,epok::Fixed dt) override {
        transform.rotation[1]+=dt; ++ticks;
        unsigned slot=entity().name[0]=='B'?1:0; ++blueprint_graph_probe[13+slot];
        if(ticks==2)take_damage(1000.0);
        if(!slot&&ticks==10&&blueprint_graph_probe[1]<6)epok::request_scene(epok::current_scene()?size_t(0):size_t(1));
        if(ticks>=20&&blueprint_graph_probe[1]==6)blueprint_graph_probe[15]=1;
    }
};
'''.replace('"CLASS"', f'"{enemy_id}"').replace('"HEALTH"', f'"{health_id}"').replace('"REWARD"', f'"{reward_id}"')
        # Delayed damage must not fire on_death a second time; only crossing zero emits it.
        source = source.replace("health-=amount; if(health<=0.0)", "bool was_alive=health>0.0; health-=amount; if(was_alive&&health<=0.0)")
        write(root / "assets/scripts/Enemy.hpp", source)
        write(root / "assets/scripts/Enemy.cpp", '#include "Enemy.hpp"\n')
        reflected = json.loads(run("--project", root, "--reflect"))
        enemy = next(c for c in reflected["classes"] if c["id"] == enemy_id)
        functions = {f["name"]: f for f in enemy["functions"]}
        bp_path = Path(run("--project", root, "--new-blueprint", "BP_Enemy", "--parent", enemy_id).strip())
        bp = documents.loads(bp_path.read_text(encoding="utf-8"))
        bp["defaults"] = {health_id: 80.0}
        death = graph(functions["on_death"], [node("call_parent"), node("set_variable", {"value": literal("int32", 5)}, member=reward_id)])
        ready = graph(functions["on_ready"], [node("call_parent"), node("delay", {"seconds": literal("fixed", 0.05)}),
                      node("call", {"amount": literal("fixed", 5.0)}, function=functions["take_damage"]["id"]),
                      node("call", function=functions["record"]["id"])])
        bp["functions"] = [death, ready]
        write(bp_path, bp)
        run("--project", root, "--compile-blueprints")
        boss_path = Path(run("--project", root, "--new-blueprint", "BP_Boss", "--parent", bp["id"], "--folder", "Bosses").strip())
        boss = documents.loads(boss_path.read_text(encoding="utf-8"))
        boss["defaults"] = {health_id: 120.0}
        boss["functions"] = [graph({**functions["on_death"], "id": death["id"]}, [node("call_parent"), node("set_variable", {"value": literal("int32", 9)}, member=reward_id)])]
        write(boss_path, boss)
        compiled = json.loads(run("--project", root, "--compile-blueprints"))
        classes = {c["cpp_name"]: c for c in compiled["classes"]}
        assert classes["BP_Boss"]["parent"] == bp["id"]
        assert classes["BP_Enemy"]["provider"] == {"id": "blueprint", "version": 1}
        print("Compiled native -> visual -> visual defaults, event overrides, Call Parent and Delay", flush=True)
        path = root / "assets/scenes/Main.epokmap"
        scene = documents.loads(path.read_text(encoding="utf-8"))
        for name, visual, overrides in [("EnemyInstance", bp, {}), ("BossInstance", boss, {"health": 150.0})]:
            entity = json.loads(json.dumps(scene["entities"][0]))
            entity.update(id=identity(), name=name, kind="Empty", script={"name": visual["name"], "class_id": visual["id"],
                          "provider": {"id": "blueprint", "version": 1}, "backend": {"id": "native", "version": 1}, "properties": overrides})
            scene["entities"].append(entity)
        write(path, scene)
        write(root / "assets/scenes/Reload.epokmap", {**scene, "name": "ReloadBank"})
        write(root / "ProjectSettings/Maps.epoksettings", {"scenes": ["assets/scenes/Reload.epokmap"]})
        run("--project", root, "--build-psx")
        headers = list((root / ".epok/build/scripts/generated").glob("*.hpp"))
        assert len(headers) == 2 and all("#line" in h.read_text() for h in headers)
        before = {h.name: h.stat().st_mtime_ns for h in headers}
        bp.setdefault("layout", {})["positions"] = {ready["entry"]: [850.0, 300.0]}
        write(bp_path, bp)
        run("--project", root, "--build-psx")
        assert before == {h.name: h.stat().st_mtime_ns for h in headers}, "Layout edits rebuilt semantic C++"
        export = Path(run("--project", root, "--export-psx").strip())
        environment=os.environ.copy()
        environment["PATH"]=str(ROOT / ".tools/mips/bin")+os.pathsep+environment["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                 "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget"),
                                 "-ToolchainBin", str(ROOT / ".tools/mips/bin")],
                                cwd=export, env=environment, capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        assert (export / "epok.ps-exe").is_file()
        assert not list(export.rglob("epok-header-tool*"))
        moved = root.with_name("Relocated Blueprint Game")
        shutil.move(root, moved)
        run("--project", next(moved.glob("*.epokproject")), "--build-psx")
        values = emulator(moved) if args.emulator else None
        report = {"passed": True, "probe": values, "classes": [bp["id"], boss["id"]],
                  "ps_exe_bytes": (moved / ".epok/build/epok.ps-exe").stat().st_size,
                  "standalone_rebuilt": True, "layout_preserved_generated_timestamps": True}
        write(ROOT / "artifacts/blueprints/native-validation.json", report)
        print("PASS: Blueprint inheritance, typed graph native compile, independent state, bank reset, Delay, export and relocation", flush=True)


if __name__ == "__main__":
    main()
