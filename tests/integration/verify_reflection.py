"""Native reflection acceptance; uses the editor's real creation, cache, and staging services."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import argparse
import contextlib
import re
import socket
import struct
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
def debug_binary(name):
    for candidate in (ROOT / "target/debug" / name, ROOT / "target/debug" / f"{name}.exe"):
        if candidate.is_file():
            return candidate
    return ROOT / "target/debug" / name


EDITOR = debug_binary("epok-editor")
TOOL = debug_binary("epok-header-tool")


def run(*arguments, ok=True):
    result = subprocess.run([str(EDITOR), *map(str, arguments)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120)
    if ok and result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    if not ok and not result.returncode:
        raise AssertionError("Expected failure: " + " ".join(map(str, arguments)))
    return result.stdout + result.stderr if not ok else result.stdout


def write(path, contents):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, contents, encoding="utf-8")


def verify_emulator(project):
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1",8077))
    log_path = project / "emulator-validation.log"
    with log_path.open("w",encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR),"--project",str(project),"--play-psx","--stop-after","8"],cwd=ROOT,stdout=log,stderr=log)
        values = None
        deadline = time.monotonic()+100
        try:
            while time.monotonic()<deadline and process.poll() is None:
                try:
                    symbols=(project / ".epok/build/epok.map").read_text()
                    found=re.search(r"0x([0-9a-fA-F]+)\s+blueprint_probe\b",symbols)
                    if found:
                        address=int(found.group(1),16)&0x1fffff
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw",timeout=2) as response:
                            values=struct.unpack_from("<16I",response.read(),address)
                        if values[15] == 1: break
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
            assert values and values[0] == 0x42505231 and values[15] == 1, (values,log_path.read_text(errors="replace"))
            assert values[1] == 6, values
            assert values[2:8] == (51200,125,1,152576,9,0), values
            assert values[12] == 0 and values[13]>0 and values[14]>0, values
            assert process.wait(timeout=30) == 0, log_path.read_text(errors="replace")
            report=ROOT / "artifacts/blueprint-foundation/native-validation.json"
            write(report,json.dumps({"passed":True,"probe":values,"expected_scene_starts":6},indent=2))
        finally:
            if process.poll() is None: process.wait(timeout=40)
    print("Verified inherited execution, independent defaults/overrides, and two scene-bank resets in PCSX-Redux RAM",flush=True)


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--emulator",action="store_true")
    parser.add_argument("--keep",action="store_true",help="Retain the isolated project for failure diagnosis")
    options=parser.parse_args()
    assert EDITOR.is_file() and TOOL.is_file(), "Run cargo build --locked --bins first"
    with (contextlib.nullcontext(tempfile.mkdtemp(prefix="epok-reflection-")) if options.keep else tempfile.TemporaryDirectory(prefix="epok-reflection-")) as temp:
        if options.keep: print(f"Retained test directory: {temp}",flush=True)
        root = Path(temp) / "Game with spaces ゲーム"
        run("--create-project", root, "--name", "Reflection Game")
        descriptor = next(root.glob("*.epokproject"))
        run("--project", descriptor, "--new-script", "Enemy")
        enemy = root / "assets/scripts/Enemy.hpp"
        original = enemy.read_text(encoding="utf-8")
        assert "final" not in original and "void update" in original
        assert not enemy.with_suffix(".epokscript").exists(), "New reflection must not duplicate signatures in JSON"
        identity = original.split('Id="')[1].split('"')[0]
        common = root / "assets/scripts/shared/Combat.hpp"
        write(common, "#pragma once\ninline constexpr int initial_health=100;\n")
        source = f'''#pragma once
#include "epok.hpp"
#include "shared/Combat.hpp"
enum class EnemyMode {{ Idle=0, Attack=3 }};
extern "C" {{ inline uint32_t blueprint_probe[16]={{}}; }}
class EPOK_CLASS(Blueprintable, Id="{identity}") Enemy : public epok::Behaviour {{
public:
    EPOK_PROPERTY(EditAnywhere) epok::Fixed health=100.0;
    EPOK_PROPERTY(EditAnywhere) int32_t lives=initial_health;
    EPOK_PROPERTY(EditAnywhere) bool aggressive=true;
    EPOK_PROPERTY(EditAnywhere) uint32_t mask=4294967295u;
    EPOK_PROPERTY(EditAnywhere) EnemyMode mode=EnemyMode::Attack;
    EPOK_PROPERTY(EditAnywhere) epok::Fixed patrol[3]={{1.0,2.0,3.0}};
    EPOK_FUNCTION(BlueprintCallable) void take_damage(epok::Fixed amount) {{health-=amount;}}
    EPOK_FUNCTION(BlueprintEvent) virtual void on_death() {{}}
    int ticks=0;
    void start(epok::Transform&) override {{
        unsigned slot=entity().name[0]=='B'?1:0;
        blueprint_probe[0]=0x42505231; ++blueprint_probe[1];
        blueprint_probe[2+slot*3]=health.raw(); blueprint_probe[3+slot*3]=lives; blueprint_probe[4+slot*3]=bool(aggressive);
        if(ticks) ++blueprint_probe[12];
    }}
    void update(epok::Transform& t,epok::Fixed dt) override {{
        t.rotation[1]+=dt; health-=dt; ++ticks;
        unsigned slot=entity().name[0]=='B'?1:0; ++blueprint_probe[13+slot];
        if(!slot && ticks==10 && blueprint_probe[1]<6) epok::request_scene(epok::current_scene()?size_t(0):size_t(1));
        if(ticks>=20 && blueprint_probe[1]==6) blueprint_probe[15]=1;
    }}
}};
'''
        write(enemy, source)
        run("--project", root, "--new-script", "Boss", "--parent", "Enemy", "--folder", "Enemies/Bosses")
        print("Created Enemy and Boss through the shared creation service", flush=True)
        boss = root / "assets/scripts/Enemies/Bosses/Boss.hpp"
        assert "override" not in boss.read_text(encoding="utf-8")
        data = json.loads(run("--project", descriptor, "--reflect"))
        classes = {c["cpp_name"]: c for c in data["classes"]}
        assert classes["Enemy"]["id"] == identity
        assert classes["Boss"]["parent"] == identity
        assert not classes["Boss"]["abstract_class"]
        assert classes["epok::Behaviour"]["abstract_class"]
        properties = {p["name"]: p for p in classes["Enemy"]["properties"]}
        assert properties["health"]["default"] == 100.0
        assert properties["aggressive"]["default"] is True
        assert properties["patrol"]["default"] == [1.0,2.0,3.0]
        damage = next(f for f in classes["Enemy"]["functions"] if f["name"] == "take_damage")
        assert damage["parameters"][0]["value_type"]["kind"] == "fixed"
        assert damage["parameters"][0]["name"] == "amount"
        update = next(f for f in classes["epok::Behaviour"]["functions"] if f["name"] == "update")
        assert update["parameters"][0]["value_type"]["cpp_name"] == "epok::Transform"
        assert update["parameters"][0]["direction"] == "mutable_reference"
        assert any(str(path).endswith("fixed-point.hh") for path in data["dependencies"])
        cache = root / ".epok/reflection/Reflection.epokcache"
        timestamp = cache.stat().st_mtime_ns
        run("--project", root, "--reflect")
        assert cache.stat().st_mtime_ns == timestamp, "Unchanged registry must remain cached"
        write(common, "#pragma once\ninline constexpr int initial_health=125;\n")
        changed = json.loads(run("--project", root, "--reflect"))
        changed_enemy = next(c for c in changed["classes"] if c["cpp_name"] == "Enemy")
        assert next(p for p in changed_enemy["properties"] if p["name"] == "lives")["default"] == 125
        assert cache.stat().st_mtime_ns != timestamp, "Transitive include edits must invalidate reflection"
        snapshot = cache.read_bytes()
        write(enemy, source.replace("bool aggressive=true", "void* aggressive=nullptr"))
        assert "Unsupported reflected type" in run("--project", root, "--reflect", ok=False)
        assert cache.read_bytes() == snapshot, "Failed extraction must retain the last valid cache"
        print("Verified semantic metadata and transitive cache/error retention", flush=True)
        write(enemy, source)
        write(enemy, source.replace("Enemy : public", "Enemy final : public"))
        assert "final" in run("--project", root, "--reflect", ok=False)
        write(enemy, source)
        run("--project", root, "--reflect")
        assert "already exists" in run("--project", root, "--new-script", "enemy", ok=False)
        scene_path = root / "assets/scenes/Main.epokmap"
        scene = documents.loads(scene_path.read_text(encoding="utf-8"))
        entity = dict(scene["entities"][0])
        entity["id"] = str(uuid.uuid4())
        entity.update(kind="Empty", name="EnemyInstance", script={"name":"Enemy", "properties":{"health":12.5}})
        other = dict(entity)
        other["id"] = str(uuid.uuid4())
        other.update(name="BossInstance", script={"name":"Boss", "properties":{"health":37.25,"aggressive":False,"lives":9,"mask":42,"mode":0,"patrol":[4.0,5.0,6.0]}})
        scene["entities"].extend([entity, other])
        write(scene_path, json.dumps(scene))
        bank = dict(scene, name="ReloadBank")
        write(root / "assets/scenes/Reload.epokmap", json.dumps(bank))
        write(root / "ProjectSettings/Maps.epoksettings", json.dumps({"scenes":["assets/scenes/Reload.epokmap"]}))
        run("--project", descriptor, "--build-psx")
        print("Built inherited typed instances and scene banks for MIPS from a Unicode project path", flush=True)
        generated = (root / ".epok/build/scene.hh").read_text()
        assert "behaviour_1.health = Fixed(51200, Fixed::RAW)" in generated
        assert "behaviour_2.health = Fixed(152576, Fixed::RAW)" in generated
        assert "behaviour_2.aggressive = false" in generated
        assert "scene_1::behaviour_2=Boss{}" in generated
        export = Path(run("--project", root, "--export-psx").strip())
        assert (export / "scripts/shared/Combat.hpp").is_file()
        assert (export / "scripts/Enemies/Bosses/Boss.hpp").is_file()
        assert documents.loads((export/"Scripts.epokmanifest").read_text())["runtime_capabilities"]==["native"]
        environment = os.environ.copy()
        environment["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + environment["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"), "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")], cwd=export, env=environment, capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        assert (export / "epok.ps-exe").is_file()
        moved = root.with_name("Relocated ゲーム")
        shutil.move(root, moved)
        relocated = json.loads(run("--project", next(moved.glob("*.epokproject")), "--reflect"))
        assert next(c for c in relocated["classes"] if c["cpp_name"] == "Enemy")["id"] == identity
        if options.emulator:
            emulator_project = moved.with_name("EmulatorGame")
            shutil.move(moved,emulator_project)
            verify_emulator(emulator_project)
        print("PASS: semantic types/events, native inheritance, independent typed overrides, transitive cache/error retention, MIPS banks, standalone rebuild, and relocation")


if __name__ == "__main__":
    main()
