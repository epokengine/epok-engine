"""Phase 1 source/cook acceptance through the production editor and real extractor."""
import json
from pathlib import Path
import subprocess
import tempfile
from verify_blueprints import ROOT, identity, run, write, node, documents


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-timeline-assets-")) / "Game"
    run("--create-project", root, "--name", "Timeline Asset Acceptance")
    class_id, property_id = identity(), identity()
    header = f'''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="{class_id}") Spell : public epok::Behaviour {{
public:
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="{property_id}") epok::Fixed progress=0.0;
    void update(epok::Transform&,epok::Fixed) override {{}}
}};
'''
    path = root / "assets/scripts/Spell.hpp"
    write(path, header)
    source = Path(run("--project", root, "--new-timeline", "Cast").strip())
    a = json.loads(source.read_text())
    slot = identity()
    a["slots"] = [{"id": slot, "name": "Caster", "target": {"kind": "entity_ref", "class": class_id}, "required": True}]
    a["tracks"] = [{"id": identity(), "name": "Charge", "slot": slot, "property": property_id,
        "value_type": {"kind": "fixed"}, "priority": 0, "blend": "absolute", "restore": "leave_final", "interpolation": "linear",
        "keys": [{"id": identity(), "tick": 0, "value": -10}, {"id": identity(), "tick": 4096, "value": 10}]}]
    a["markers"] = [{"id": identity(), "name": "Impact", "tick": 2048}]
    write(source, a)
    first = json.loads(run("--project", root, "--compile-timelines"))[0]
    cache = root / f".epok/timelines/{a['id']}.json"
    table = root / f".epok/timelines/{a['id']}.hh"
    write(path, header + "\n#error Broken reflection dependency\n")
    assert "reflection" in run("--project", root, "--compile-timelines", ok=False).lower()
    assert json.loads(cache.read_text())["stale"]
    write(path, header)
    run("--project", root, "--compile-timelines")
    stamp = table.stat().st_mtime_ns
    a["tracks"][0]["keys"].reverse()
    a["markers"][0]["name"] = "Hit"
    a["layout"] = {a["tracks"][0]["id"]: [25, 40]}
    write(source, a)
    second = json.loads(run("--project", root, "--compile-timelines"))[0]
    assert first["signature"] == second["signature"]
    assert table.stat().st_mtime_ns == stamp
    write(root / "assets/scripts/Other.hpp", '#pragma once\n#include "epok.hpp"\nclass EPOK_CLASS(Blueprintable) Other:public epok::Behaviour{public:void update(epok::Transform&,epok::Fixed)override{}};\n')
    assert json.loads(run("--project", root, "--compile-timelines"))[0]["signature"] == first["signature"]
    write(path, header.replace("progress=", "charge="))
    renamed = json.loads(run("--project", root, "--compile-timelines"))[0]
    assert renamed["signature"] != first["signature"] and renamed["tracks"][0]["field"] == "charge"
    assert renamed["tracks"][0]["property"] == property_id
    write(path, header.replace("TimelineAnimatable,", ""))
    assert "explicitly expose" in run("--project", root, "--compile-timelines", ok=False)
    stale = json.loads(cache.read_text())
    assert stale["stale"] and stale["compiled"]["signature"] == renamed["signature"]
    write(path, header)
    run("--project", root, "--compile-timelines")
    a["tracks"][0]["property"] = "progress"
    write(source, a)
    assert "no name fallback" in run("--project", root, "--compile-timelines", ok=False)
    assert json.loads(cache.read_text())["stale"]
    a["tracks"][0]["property"] = property_id
    write(source, a)
    run("--project", root, "--compile-timelines")
    assert not json.loads(cache.read_text())["stale"]
    enabled_id, vector_id, function_id = identity(), identity(), identity()
    typed_header = header.replace('    void update', f'''
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="{enabled_id}") bool enabled=false;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="{vector_id}") epok::Fixed offset[3]={{0.0,0.0,0.0}};
    EPOK_FUNCTION(TimelineCallable,Id="{function_id}") void impact(epok::Fixed power,epok::EntityHandle target) {{}}
    void update''')
    write(path, typed_header)
    for prop, name, kind, first_value, last_value, mode in [
        (enabled_id, "Enabled", {"kind": "bool"}, False, True, "step"),
        (vector_id, "Offset", {"kind": "vector", "length": 3}, [-1, 2, 4], [1, 4, 0], "smoothstep"),
    ]:
        a["tracks"].append({"id": identity(), "name": name, "slot": slot, "property": prop,
            "value_type": kind, "priority": 0, "blend": "absolute", "restore": "restore_initial", "interpolation": mode,
            "keys": [{"id": identity(), "tick": 0, "value": first_value}, {"id": identity(), "tick": 4096, "value": last_value}]})
    a["events"] = [{"id": identity(), "name": "Impact", "slot": slot, "function": function_id,
        "keys": [{"id": identity(), "tick": 2048, "arguments": {
            "power": {"kind": "literal", "value_type": {"kind": "fixed"}, "value": 1.5},
            "target": {"kind": "slot", "slot": slot}}}]}]
    write(source, a)
    typed = json.loads(run("--project", root, "--compile-timelines"))[0]
    assert typed["events"][0]["arguments"][0]["lanes"] == [6144, 0, 0, 0]
    assert len(next(t for t in typed["tracks"] if t["property"] == vector_id)["channels"]) == 3
    write(path, typed_header.replace("TimelineCallable", "BlueprintCallable"))
    assert "explicit timeline exposure" in run("--project", root, "--compile-timelines", ok=False)
    write(path, typed_header)
    run("--project", root, "--compile-timelines")
    bp_path = Path(run("--project", root, "--new-blueprint", "BP_Spell", "--parent", class_id).strip())
    bp = documents.loads(bp_path.read_text(encoding="utf-8"))
    variable_id = identity()
    bp["variables"] = [{"id": variable_id, "name": "charge", "value_type": {"kind": "fixed"}, "default": 0,
                        "editable": True, "timeline_animatable": True}]
    entry, finish = node("entry"), node("return")
    entry["outputs"]["next"] = [finish["id"]]
    bp_event = {"id": identity(), "name": "OnImpact", "timeline": "crossing_event", "parameters": [],
                "returns": {"kind": "void"}, "entry": entry["id"], "nodes": [entry, finish]}
    bp["functions"] = [bp_event]
    write(bp_path, bp)
    a["slots"][0]["target"]["class"] = bp["id"]
    a["tracks"][0]["property"] = variable_id
    a["events"].append({"id": identity(), "name": "Visual event", "slot": slot, "function": bp_event["id"],
                         "keys": [{"id": identity(), "tick": 2048, "arguments": {}}]})
    write(source, a)
    visual = json.loads(run("--project", root, "--compile-timelines"))[0]
    assert any(e["function"] == bp_event["id"] for e in visual["events"])
    bp["variables"][0]["timeline_animatable"] = False
    write(bp_path, bp)
    assert "explicitly expose" in run("--project", root, "--compile-timelines", ok=False)
    bp["variables"][0]["timeline_animatable"] = True
    write(bp_path, bp)
    run("--project", root, "--compile-timelines")
    compiler = ROOT / ".tools/mips/bin/mipsel-none-elf-g++.exe"
    translation_unit = root / ".epok/timelines/probe.cpp"
    write(translation_unit, '#include "' + table.name + '"\n')
    subprocess.run([str(compiler), "-std=c++20", "-mips1", "-mabi=32", "-EL", "-msoft-float", "-ffreestanding", "-fno-rtti", "-fno-exceptions",
        "-fsyntax-only", str(translation_unit), f"-I{ROOT / 'runtime'}"], check=True, timeout=60)
    # Compile the actual generated typed accessors against the staged native and
    # Blueprint classes, with real runtime ancestry and dynamic dispatch guards.
    run("--project", root, "--build-psx")
    write(translation_unit, '#include "scene.hh"\n#include "' + a["id"] + '.runtime.hh"\n')
    nugget = ROOT / "third_party/nugget"
    subprocess.run([str(compiler), "-std=c++20", "-mips1", "-mabi=32", "-EL", "-msoft-float", "-ffreestanding", "-fno-rtti", "-fno-exceptions",
        "-DEPOK_BLUEPRINTS", "-fsyntax-only", str(translation_unit), f"-I{root / '.epok/build'}", f"-I{nugget}",
        f"-I{nugget / 'third_party/EASTL/include'}", f"-I{nugget / 'third_party/EABase/include/Common'}"], check=True, timeout=60)
    write(source.with_name("Duplicate.timeline.json"), a)
    assert "Duplicate TimelineAsset UUID" in run("--project", root, "--compile-timelines", ok=False)
    report = {"passed": True, "fixture": str(root), "checks": ["real reflected ID opt-in", "source roundtrip", "layout/name/reorder stability",
        "unchanged table timestamps", "unrelated class isolation", "native rename invalidation", "stale cache retention", "ID-only recovery", "typed native and Blueprint properties/events", "MIPS table compilation", "duplicate UUID rejection"]}
    write(ROOT / "artifacts/timelines/phase1.json", report)
    print("PASS TimelineAsset source/cook/cache acceptance; retained fixture:", root)


if __name__ == "__main__":
    main()
