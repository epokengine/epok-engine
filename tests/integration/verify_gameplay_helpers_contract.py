"""Prove helper predicate overrides and record values through the real compilers."""
from pathlib import Path
import json
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

EDITOR = ROOT / "target/debug/epok-editor.exe"
OUT = ROOT / "artifacts/gameplay-helpers/p0/compiler-contract"
PROJECT = OUT / "project"


def run(*arguments):
    result = subprocess.run([str(EDITOR), *map(str, arguments)], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8", timeout=180)
    if result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result.stdout


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    if not PROJECT.exists():
        run("--create-project", PROJECT, "--template", "basic")
    header = PROJECT / "assets/scripts/HelperPredicate.hpp"
    header.write_text('''#pragma once
#include "epok.hpp"
struct EPOK_VALUE(Id="06ae2c47-e28b-5e1c-8d88-d47e49e5e436") HelperPredicateResult {
    bool accepted=false;
    uint32_t reason=0;
};
class EPOK_CLASS(Blueprintable, Owners=World3D|World2D, Id="75e0912f-ec50-5472-950e-26ec666bba46") HelperPredicate : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t observed=0;
    EPOK_FUNCTION(BlueprintEvent, BlueprintPure, Id="88c65dfe-2331-53ec-83ec-9ce661112ed9")
    virtual bool can_interact(epok::ObjectId initiator) const { return initiator.valid(); }
    EPOK_FUNCTION(BlueprintPure, Id="1b1b0ebc-ed26-5282-bdb4-a4073b625e81")
    HelperPredicateResult inspect(epok::ObjectId initiator) const { return {can_interact(initiator),7}; }
};
''', encoding="utf-8")
    reflection = json.loads(run("--project", PROJECT, "--reflect"))
    base = next(c for c in reflection["classes"] if c["cpp_name"] == "HelperPredicate")
    predicate = next(f for f in base["functions"] if f["name"] == "can_interact")
    assert predicate["event"] and predicate["returns"] == {"kind": "bool"}
    path = PROJECT / "assets/Blueprints/BP_HelperPredicate.epokbp"
    if not path.exists():
        run("--project", PROJECT, "--new-blueprint", "BP_HelperPredicate", "--parent", base["id"])
    asset = documents.loads(path.read_text(encoding="utf-8"))
    entry, returned = str(uuid.uuid4()), str(uuid.uuid4())
    asset["functions"].append({
        "id": str(uuid.uuid4()), "name": "can_interact", "override_id": predicate["id"],
        "parameters": predicate["parameters"], "returns": predicate["returns"], "entry": entry,
        "nodes": [{"id": entry, "kind": {"kind": "entry"}, "inputs": {}, "outputs": {"next": [returned]}},
                  {"id": returned, "kind": {"kind": "return"},
                   "inputs": {"value": {"kind": "literal", "value_type": {"kind": "bool"}, "value": True}}, "outputs": {}}],
    })
    # Repeated checks replace the one override instead of appending another.
    asset["functions"] = [f for f in asset["functions"] if f.get("override_id") != predicate["id"]] + [asset["functions"][-1]]
    documents.write_text(path, documents.dumps(asset))
    run("--project", PROJECT, "--compile-blueprints")
    lua = PROJECT / "assets/scripts/LuaHelperPredicate.lua"
    lua.write_text('''---@class LuaHelperPredicate : HelperPredicate
local LuaHelperPredicate = HelperPredicate:extend()
---@override
function LuaHelperPredicate:can_interact(initiator)
    return true
end
function LuaHelperPredicate:begin_play()
    local sample = self:inspect(self:owner_id())
    if sample.accepted then
        self.observed = sample.reason
    end
end
return LuaHelperPredicate
''', encoding="utf-8")
    descriptor = next(PROJECT.glob("*.epokproject"))
    results = []
    for mode in ("native_cpp", "vm_bytecode", "vm_source"):
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = mode
        documents.write_text(descriptor, documents.dumps(manifest))
        run("--project", PROJECT, "--export-psx")
        results.append({"mode": mode, "compiled": True})
    report = {"bool_event_override": True, "blueprint": True, "lua_record_and_override": results,
              "runtime_execution": "not part of this compiler-only contract probe"}
    (OUT / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
