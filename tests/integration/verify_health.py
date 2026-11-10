"""Execute independently authored health policies against the SDK on PSX."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import uuid

from gameplay_probe import export_probe, run_modes
from verify_camera_rigs import EDITOR, ROOT, documents, write

NAMESPACE = uuid.UUID("a42d5289-8a70-510e-b702-f1021ac4027c")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "acceptance/" + name))
BASE, POLICY, DRIVER = (identity(name) for name in ("base", "policy", "driver"))
EXECUTE, PUBLISH = (identity(name) for name in ("execute_step", "publish"))
MAGIC, STEPS, WORDS = 0x4845414C, 18, 10
U, I, B, V = ({"kind": k} for k in ("uint32", "int32", "bool", "void"))
OPERATIONS = {
    "make_state": "43903550-2746-5f1c-af57-ac221db6681c",
    "valid": "df6acd25-a526-5605-86e3-1a73c31737b8",
    "is_depleted": "c9fd8130-59a5-5513-8864-2ad5ee70c1af",
    "damage": "564672a4-0a4a-51e6-ad32-4d8f447b2da7",
    "heal": "adc1907c-df26-56b3-8d13-25015e4792f2",
}
# (reset, maximum, initial, amount, healing, shielded, inject_invalid)
CASES = [
    (True, 100, 100, 0, False, False, False),
    (False, 100, 100, 20, False, False, False),
    (False, 100, 100, 20, False, True, False),
    (False, 100, 100, 35, True, False, False),
    (False, 100, 100, 13, False, False, False),
    (False, 100, 100, 2147483647, False, False, False),
    (False, 100, 100, 20, False, False, False),
    (False, 100, 100, 1, True, False, False),
    (False, 100, 100, -20, False, False, False),
    (False, 100, 100, -20, True, False, False),
    (True, 60, 100, 0, False, False, False),
    (True, 4294967295, 4294967294, 2147483647, True, False, False),
    (False, 100, 100, 2147483647, False, False, False),
    (False, 100, 100, 2147483647, False, False, False),
    (False, 100, 100, 1, False, False, False),
    (False, 100, 100, 20, False, False, True),
    (False, 100, 100, 35, True, False, False),
    (True, 0, 100, 0, False, False, False),
]

BASE_SOURCE = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t health_probe[8]={};inline uint32_t health_trace[18][10]={};}
class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="BASE") HealthPolicyBase:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("BASE");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) epok::HealthState health{100,100};
    EPOK_PROPERTY(EditAnywhere) bool shielded=false;
    EPOK_PROPERTY(EditAnywhere) bool healing=false;
    EPOK_PROPERTY(EditAnywhere) bool reset_input=false;
    EPOK_PROPERTY(EditAnywhere) uint32_t maximum=100;
    EPOK_PROPERTY(EditAnywhere) uint32_t initial=100;
    EPOK_PROPERTY(EditAnywhere) uint32_t step=0;
    EPOK_PROPERTY(EditAnywhere) int32_t amount=0;
    EPOK_FUNCTION(BlueprintEvent,Id="EXECUTE") virtual void execute_step()=0;
    // Observation/data assignment only: no policy, health arithmetic or filtering.
    EPOK_FUNCTION(BlueprintCallable,Id="PUBLISH") void publish(epok::HealthChange change,bool valid,bool empty){
        health=change.state;
        auto* p=health_probe;
        if(step>=18){p[2]|=1;return;}
        const uint32_t row[]={step,health.maximum,health.current,change.applied,change.depleted,change.revived,uint32_t(change.result),valid,empty,shielded};
        if(!p[3])p[4]=2166136261u;
        for(unsigned i=0;i<10;++i){health_trace[step][i]=row[i];p[4]=(p[4]^row[i])*16777619u;}
        ++p[3];p[5]+=!valid;p[6]+=empty;
    }
};
'''
CPP_SOURCE = r'''#pragma once
#include "HealthPolicyBase.hpp"
class EPOK_CLASS(Blueprintable,Id="POLICY") HealthPolicy:public HealthPolicyBase {
public:
    void execute_step()override{
        if(reset_input)health=epok::HealthLibrary::make_state(maximum,initial);
        const auto before=health;
        const auto actual=shielded||amount==13?0:amount;
        const auto change=healing?epok::HealthLibrary::heal(before,amount):epok::HealthLibrary::damage(before,actual);
        publish(change,epok::HealthLibrary::valid(before),epok::HealthLibrary::is_depleted(before));
    }
};
'''
LUA_SOURCE = '''---@class HealthPolicy : HealthPolicyBase
local HealthPolicy = HealthPolicyBase:extend()

---@override
function HealthPolicy:execute_step()
    if self.reset_input then
        self.health = epok.health.make_state(self.maximum, self.initial)
    end
    local before = self.health
    local actual = self.amount
    if self.shielded or self.amount == 13 then actual = 0 end
    local change = epok.health.damage(before, actual)
    if self.healing then change = epok.health.heal(before, self.amount) end
    self:publish(change, epok.health.valid(before), epok.health.is_depleted(before))
end

return HealthPolicy
'''


def driver_source():
    rows = ["{" + ",".join(str(int(v)) + ("u" if n in (1, 2) else "")
                           for n, v in enumerate(case)) + "}" for case in CASES]
    return r'''#pragma once
#include "HealthPolicyBase.hpp"
class EPOK_CLASS(Blueprintable,Id="DRIVER") HealthProbe:public epok::Actor3D {
    uint32_t frame_=0;
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="HealthPolicyBase") epok::ObjectId policy{};
    void on_frame(uint32_t)override{
        if(frame_>=18)return;
        auto* rules=epok::active_object_registry?epok::active_object_registry->resolve<HealthPolicyBase>(policy):nullptr;
        if(!rules)return;
        struct Case{bool reset;uint32_t maximum,initial;int32_t amount;bool healing,shielded,invalid;};
        static constexpr Case cases[]={CASES};
        const auto c=cases[frame_];
        rules->step=frame_;rules->reset_input=c.reset;rules->maximum=c.maximum;rules->initial=c.initial;
        rules->amount=c.amount;rules->healing=c.healing;rules->shielded=c.shielded;
        if(c.invalid)rules->health={100,101};
        const auto calls=health_probe[3];rules->execute_step();
        if(health_probe[3]!=calls+1)health_probe[2]|=2;
        if(++frame_==18){health_probe[1]=frame_;health_probe[7]=1;health_probe[0]=0x4845414Cu+EPOK_LUA_MODE;}
    }
};
'''.replace("CASES", ",\n".join(rows))


def source_ids(source):
    for name in ("BASE", "POLICY", "DRIVER", "EXECUTE", "PUBLISH"):
        source = source.replace('"' + name + '"', '"' + globals()[name] + '"')
    return source


def oracle():
    maximum, current, words = 100, 100, []
    for step, (reset, limit, initial, amount, healing, shielded, invalid) in enumerate(CASES):
        if invalid:
            maximum, current = 100, 101
        if reset:
            maximum, current = limit, min(initial, limit)
        valid, empty = current <= maximum, current <= maximum and current == 0
        applied, depleted, revived, result = 0, False, False, 0
        if not valid:
            result = 2
        else:
            eligible = amount if healing or not (shielded or amount == 13) else 0
            applied = min(max(eligible, 0), maximum - current if healing else current)
            revived = healing and current == 0 and applied > 0
            current += applied if healing else -applied
            depleted = not healing and current == 0 and applied > 0
            result = int(applied > 0)
        words += [step, maximum, current, applied, int(depleted), int(revived), result, int(valid), int(empty), int(shielded)]
    return words


def blueprint(metadata):
    base = next(c for c in metadata["classes"] if c["cpp_name"] == "HealthPolicyBase")
    properties = {p["name"]: p["id"] for p in base["properties"]}
    nodes, positions = [], {}

    def node(kind, inputs=None, **fields):
        value = {"id": identity("node/" + str(len(nodes))), "kind": {"kind": kind, **fields},
                 "inputs": inputs or {}, "outputs": {}}
        positions[value["id"]] = [float(len(nodes) % 6 * 290), float(len(nodes) // 6 * 180)]
        nodes.append(value)
        return value

    link = lambda n: {"kind": "link", "node": n["id"], "pin": "value"}
    literal = lambda value, ty: {"kind": "literal", "value_type": ty, "value": value}
    read = lambda name: link(node("get_variable", member=properties[name]))
    operations = {o["name"]: o for library in metadata["function_libraries"]
                  if library["cpp_name"] == "epok::HealthLibrary" for o in library["operations"]}

    def call(name, **inputs):
        if name == "publish":
            return node("call", inputs, function=PUBLISH)
        operation = operations[name]
        pins = {p["id"]: inputs[p["name"]] for p in operation["parameters"]}
        return node("operation", pins, operation=operation["id"])
    entry, reset = node("entry"), node("branch", {"condition": read("reset_input")})
    state = call("make_state", maximum=read("maximum"), initial=read("initial"))
    assign = node("set_variable", {"value": link(state)}, member=properties["health"])
    healing = node("branch", {"condition": read("healing")})
    rejected = node("binary", {"a": read("amount"), "b": literal(13, I)}, op="equal")
    blocked = node("binary", {"a": read("shielded"), "b": link(rejected)}, op="or")
    gate = node("branch", {"condition": link(blocked)})
    entry["outputs"]["next"] = [reset["id"]]
    reset["outputs"] = {"true": [assign["id"]], "false": [healing["id"]]}
    assign["outputs"]["next"] = [healing["id"]]
    healing["outputs"]["false"] = [gate["id"]]
    for branch, pin, operation, amount in ((healing, "true", "heal", read("amount")),
                                         (gate, "true", "damage", literal(0, I)),
                                         (gate, "false", "damage", read("amount"))):
        before = read("health")
        change = call(operation, state=before, amount=amount)
        publish = call("publish", change=link(change), valid=link(call("valid", state=before)),
                       empty=link(call("is_depleted", state=before)))
        end = node("return")
        branch["outputs"][pin] = [publish["id"]]
        publish["outputs"]["next"] = [end["id"]]
    return {"version": 8, "id": POLICY, "name": "HealthPolicy", "parent": BASE, "family": "component",
            "defaults": {}, "variables": [], "template": {"actors": []},
            "functions": [{"id": identity("execute-graph"), "name": "execute_step", "override_id": EXECUTE,
                           "parameters": [], "returns": V, "entry": entry["id"], "nodes": nodes}],
            "layout": {"positions": positions, "comments": {}}}


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
        # Native source is the primary Lua template route.
        manifest_path = next(project.glob("*.epokproject"))
        manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["lua_execution"] = "native_cpp"
        write(manifest_path, manifest)
        run("--project", project, "--build-psx", label="native-export-build.log")
        export_probe(out, project, run, ["health.hpp"])
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Health " + flavor)
    scripts = project / "assets/scripts"
    write(scripts / "HealthPolicyBase.hpp", source_ids(BASE_SOURCE))
    write(scripts / "HealthProbe.hpp", source_ids(driver_source()))
    if flavor == "cpp":
        write(scripts / "HealthPolicy.hpp", source_ids(CPP_SOURCE))
    metadata = json.loads(run("--project", project, "--reflect", label="reflection.log"))
    health = next(l for l in metadata["function_libraries"] if l["cpp_name"] == "epok::HealthLibrary")
    assert {o["id"] for o in health["operations"]} == set(OPERATIONS.values()), health
    if flavor == "blueprint":
        write(project / "assets/Blueprints/HealthPolicy.epokbp", blueprint(metadata))
    elif flavor == "lua":
        write(scripts / "HealthPolicy.lua", LUA_SOURCE)
        write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/HealthPolicy.lua": POLICY}})
    assert flavor == "cpp" or not (scripts / "HealthPolicy.hpp").exists()
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    root = copy.deepcopy(next(c for c in scene["actors"][0]["components"] if c.get("root")))
    root["id"] = identity("root")
    component = identity("component")
    scene["actors"] = [{"id": identity("actor"), "name": "Health probe",
        "class": {"name": "HealthProbe", "class_id": DRIVER}, "properties": {"policy": component},
        "overrides": ["policy"], "components": [root, {"id": component, "name": "Health policy",
        "class": {"name": "HealthPolicy", "class_id": POLICY}, "properties": {}, "overrides": []}]}]
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    if prepare_only:
        print("Prepared", flavor, flush=True)
        return

    expected_words = oracle()
    expected_hash = 2166136261
    for word in expected_words:
        expected_hash = ((expected_hash ^ word) * 16777619) & 0xffffffff

    def check(values, expected):
        assert values == [expected, STEPS, 0, STEPS, expected_hash, 2, 3, 1], values

    selected = modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",))
    run_modes(out, project, flavor, selected, "health_probe", MAGIC, check,
              provider_count=1, trace=("health_trace", STEPS * WORDS))
    for mode in selected:
        actual = json.loads((out / (mode + "-health_trace.json")).read_text())
        assert actual == expected_words, (flavor, mode, "wide integer oracle mismatch")
    write(out / "source-inventory.json", {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
          for p in sorted(scripts.iterdir()) if p.is_file()})
    print("PASS", flavor, "all 180 trace words match the independent oracle", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/health/providers")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.output.resolve(), flavor, args.mode, args.prepare_only, args.standalone_only)


if __name__ == "__main__":
    main()
