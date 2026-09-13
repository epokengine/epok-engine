"""ParticleEffect source acceptance using the existing Clang reflection catalog.

This checks the source/compiler extension; it is not Phase 3 runtime acceptance.
"""
import copy
import json
from pathlib import Path
import subprocess
import tempfile
from verify_blueprints import ROOT, documents, identity, run, write


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-effect-assets-")) / "Game"
    run("--create-project", root, "--name", "Effect Source Acceptance")
    # SDK layer metadata must coexist with user behaviour chains, rather than
    # being misclassified as another attachable/spawnable script.
    write(root / "assets/scripts/Probe.hpp", '#pragma once\n#include "epok.hpp"\nclass EPOK_CLASS(Blueprintable) Probe:public epok::Behaviour{public:void update(epok::Transform&,epok::Fixed)override{}};\n')
    path = Path(run("--project", root, "--new-particle-effect", "Fireball").strip())
    effect = json.loads(path.read_text())
    catalog = json.loads(run("--project", root, "--reflect"))
    cls = next(c for c in catalog["classes"] if c["cpp_name"] == "epok::EffectLayer")
    assert not cls["blueprintable"] and cls["parent"] is None
    size = next(p for p in cls["properties"] if p["name"] == "size")
    burst = next(f for f in cls["functions"] if f["name"] == "burst")
    assert size["timeline"] and burst["timeline"] == "crossing_event"
    layer = effect["layers"][0]
    slot = effect["timeline"]["slots"][0]
    assert slot["target"] == {"kind": "effect_layer_ref", "class": cls["id"]}
    effect["timeline"]["tracks"] = [{"id": identity(), "name": "Charge size", "slot": layer["slot"],
        "property": size["id"], "value_type": {"kind": "fixed"}, "priority": 0, "blend": "absolute",
        "restore": "leave_final", "interpolation": "smoothstep", "keys": [
            {"id": identity(), "tick": 0, "value": 0}, {"id": identity(), "tick": 4096, "value": 2}]}]
    effect["timeline"]["events"] = [{"id": identity(), "name": "Impact burst", "slot": layer["slot"],
        "function": burst["id"], "keys": [{"id": identity(), "tick": 2048, "arguments": {"count": {
            "kind": "literal", "value_type": {"kind": "uint32"}, "value": 8}}}]}]
    write(path, effect)
    result = json.loads(run("--project", root, "--validate-particle-effects"))[0]
    assert result["timeline"]["tracks"][0]["channels"][0][-1] == [4096, 8192]
    assert result["timeline"]["events"][0]["arguments"][0]["lanes"] == [8, 0, 0, 0]
    signature = result["signature"]
    effect["timeline"]["tracks"][0]["keys"].reverse()
    effect["name"] = "Renamed"
    write(path, effect)
    assert json.loads(run("--project", root, "--validate-particle-effects"))[0]["signature"] == signature
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["version"] = 4
    scene["entities"][-1]["particle_effect"] = {"version": 1, "asset": effect["id"],
        "bindings": {}, "enabled": True, "play_on_start": True, "seed": 123}
    write(scene_path, scene)
    run("--project", root, "--build-psx")
    staged_key = f"generated-playback:.epok/build/effects/{effect['id']}.hh"
    def staged_record():
        return json.loads((root / ".epok/ArtifactDependencies.json").read_text())["nodes"][staged_key]
    valid_record = staged_record()
    assert not valid_record["stale"]
    # Explicit instance overrides use layer/property UUIDs and fail closed when
    # orphaned or incompatible. Failed cooking never rewrites their source.
    valid_scene = copy.deepcopy(scene)
    component = scene["entities"][-1]["particle_effect"]
    component["version"] = 2
    component["layer_overrides"] = {identity(): {size["id"]: {
        "kind": "literal", "value_type": {"kind": "fixed"}, "value": 0.5}}}
    write(scene_path, scene)
    before = scene_path.read_bytes()
    assert "orphaned layer override" in run("--project", root, "--build-psx", ok=False)
    assert scene_path.read_bytes() == before
    assert staged_record()["stale"] and staged_record()["signature"] == valid_record["signature"]
    component["asset"] = effect["id"]
    component["layer_overrides"] = {effect["layers"][0]["id"]: {size["id"]: {
        "kind": "literal", "value_type": {"kind": "bool"}, "value": True}}}
    write(scene_path, scene)
    before = scene_path.read_bytes()
    assert "override type changed" in run("--project", root, "--build-psx", ok=False)
    assert scene_path.read_bytes() == before
    scene = valid_scene
    write(scene_path, scene)
    effect_header = root / f".epok/build/effects/{effect['id']}.hh"
    assert "inline constexpr LayerDefinition" in effect_header.read_text()
    assert "EPOK_EFFECTS" in (root / ".epok/build/sources.mk").read_text()
    probe = root / ".epok/timelines/probe.cpp"
    write(probe, '#include "scene.hh"\n')
    nugget = ROOT / "third_party/nugget"
    subprocess.run([str(ROOT / ".tools/mips/bin/mipsel-none-elf-g++.exe"), "-std=c++20", "-mips1", "-mabi=32", "-EL",
        "-msoft-float", "-ffreestanding", "-fno-rtti", "-fno-exceptions", "-DEPOK_BLUEPRINTS", "-fsyntax-only", str(probe),
        f"-I{root / '.epok/build'}", f"-I{nugget}", f"-I{nugget / 'third_party/EASTL/include'}",
        f"-I{nugget / 'third_party/EABase/include/Common'}"], check=True, timeout=60)
    effect["layers"][0]["content"]["emitter"]["max_particles"] = 129
    write(path, effect)
    before = path.read_bytes()
    assert "particle" in run("--project", root, "--validate-particle-effects", ok=False).lower()
    assert path.read_bytes() == before
    assert json.loads((root / f".epok/timelines/{effect['timeline']['id']}.json").read_text())["stale"]
    effect["layers"][0]["content"]["emitter"]["max_particles"] = 32
    effect["timeline"]["version"] = 1
    write(path, effect)
    before = path.read_bytes()
    migrated = json.loads(run("--project", root, "--validate-particle-effects"))[0]
    assert migrated["timeline"]["asset"] == effect["timeline"]["id"] and path.read_bytes() == before
    effect["layers"][0]["content"]["emitter"]["future_control"] = 0.5
    write(path, effect)
    before = path.read_bytes()
    assert "original source preserved" in run("--project", root, "--validate-particle-effects", ok=False)
    assert path.read_bytes() == before
    del effect["layers"][0]["content"]["emitter"]["future_control"]
    write(path, effect)
    # Every editor preset uses the same source schema, reflected target and
    # production effect cooker. No preset-specific runtime representation exists.
    for name in ["Fire", "Smoke", "Sparks", "Impact", "Projectile", "Aura", "Rune"]:
        preset_path = Path(run("--project", root, "--new-particle-effect", name, "--preset", name).strip())
        preset = json.loads(preset_path.read_text())
        assert len(preset["layers"]) == 1 and len(preset["timeline"]["tracks"]) == 1
        entity = copy.deepcopy(scene["entities"][-1])
        entity.update(id=identity(), name=name, kind="Empty", script=None)
        entity["particle_effect"]["asset"] = preset["id"]
        scene["entities"].append(entity)
    assert len(json.loads(run("--project", root, "--validate-particle-effects"))) == 8
    write(scene_path, scene)
    run("--project", root, "--build-psx")
    assert not staged_record()["stale"]
    print("PASS embedded TimelineAsset source, shared reflected layer targets, typed curves/events and unchanged particle limits:", root)


if __name__ == "__main__":
    main()
