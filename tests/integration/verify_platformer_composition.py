"""Compare the authored 2D level/resources while accepting only pickup bindings."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
from verify_camera_rigs import ROOT, EDITOR, documents

CONTROLLERS = ("ca7559df-c335-44a1-ae8c-5268c5b68ace", "31feac4e-5004-460b-9de6-2648b95604d5", "4919923d-00cf-47ed-bcb8-393562762177")
PICKUPS = ("6755cf22-df21-4f8f-81b5-21c85f6872fa", "7d16f1c2-90f8-48db-88c4-de1f48183edf", "39caab97-d716-49fc-8029-a1fb3caf7380")


def snapshot(project):
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene = documents.loads((project/manifest["startup_scene"]).read_text(encoding="utf-8"))
    # The six old logical headings become one runtime world root. Compare every
    # retained actor unchanged and remap only their organizational parents.
    headings = {"Scenery", "Level", "Characters", "Props", "Coins", "Effects"}
    old_roots = [a for a in scene["actors"] if a["name"] in headings]
    if old_roots:
        assert len(old_roots) == 6
        for actor in old_roots:
            assert len(actor["components"]) == 1 and not actor.get("attach") and not actor.get("logical_parent")
            props = actor["components"][0]["properties"]
            assert props["position"] == [0, 0] and props["rotation"] == 0 and props["scale"] == [1, 1]
        world = old_roots[0]
        world["name"] = "World2D"
        ids = {a["id"] for a in old_roots}
        for actor in scene["actors"]:
            if actor.get("logical_parent") in ids:
                actor["logical_parent"] = world["id"]
        scene["actors"] = [a for a in scene["actors"] if a["id"] not in ids or a["id"] == world["id"]]
    aliases = {identity: role for identities, role in ((CONTROLLERS, "project-controller"), (PICKUPS, "project-pickup")) for identity in identities}
    for i, actor in enumerate(scene["actors"]):
        aliases[actor["id"]] = f"actor:{i}"
        for j, component in enumerate(actor["components"]):
            aliases[component["id"]] = f"component:{i}:{j}"
            if component["class"].get("class_id") in PICKUPS:
                # The only authored scene change allowed by this migration.
                for field in ("accepted_initiator", "auto_collect"):
                    component.get("properties", {}).pop(field, None)
                    if field in component.get("overrides", []):
                        component["overrides"].remove(field)
                for field in ("properties", "overrides"):
                    if not component.get(field):
                        component.pop(field, None)
    packages = {}
    resources = {}
    for path in sorted((project/"assets").rglob("*")):
        key = path.relative_to(project).as_posix()
        if path.is_file() and path.suffix == ".epokasset":
            blob = path.read_bytes()
            assert blob[:8] == b"EPOKAS01", key
            size = struct.unpack_from("<I", blob, 8)[0]
            meta = json.loads(blob[16:16+size])
            aliases[meta["id"]] = key
            packages[key] = (meta, blob[16+size:])
        elif path.is_file() and path.suffix == ".png":
            resources[key] = hashlib.sha256(path.read_bytes()).hexdigest()

    def visit(value):
        if isinstance(value, str):
            return aliases.get(value, value)
        if isinstance(value, list):
            return [visit(item) for item in value]
        if isinstance(value, dict):
            return {aliases.get(key, key): visit(item) for key, item in value.items()}
        return value

    for key, (meta, payload) in packages.items():
        # Texture pixels are byte exact. TileSet references are generated UUIDs.
        data = visit(json.loads(payload)) if meta["kind"] == "TileSet" else hashlib.sha256(payload).hexdigest()
        if meta["kind"] == "TileSet":
            assert meta["source_hash"] == hashlib.sha256(payload).hexdigest()
            meta["source_hash"] = hashlib.sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()
        resources[key] = {"meta": visit(meta), "payload": data}
    scene.pop("bake", None)
    scene.pop("navigation", None)
    return {"scene": visit(scene), "resources": resources}


def verify(out, before):
    out.mkdir(parents=True, exist_ok=True)
    baseline_record = json.loads((before/"composition.json").read_text())
    baseline_project = before/"project"
    for key, digest in baseline_record["assets"].items():
        assert hashlib.sha256((baseline_project/key).read_bytes()).hexdigest() == digest
    baseline = snapshot(baseline_project)
    rows = []
    for flavor in ("cpp", "blueprint", "lua"):
        project = out/flavor
        if not project.exists():
            result = subprocess.run([str(EDITOR), "--create-project", str(project), "--template", "platformer-2d", "--gameplay", flavor], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
            (out/(flavor+"-create.log")).write_text(result.stdout+result.stderr, encoding="utf-8")
            assert result.returncode == 0, result.stdout+result.stderr
        actual = snapshot(project)
        (out/(flavor+"-composition.json")).write_text(json.dumps(actual, indent=2)+"\n", encoding="utf-8")
        assert actual["resources"] == baseline["resources"], flavor+": original resource pixels/settings changed"
        assert actual["scene"] == baseline["scene"], flavor+": authored 2D world/tuning changed"
        installed = documents.loads((project/"ProjectSettings/Template.epoksettings").read_text())
        assert installed["template"] == "platformer-2d" and installed["gameplay"] == flavor
        assert set(installed["roles"]) == {"player", "controller", "locomotion", "pickup", "camera"}
        rows.append({"flavor": flavor, "actors": len(actual["scene"]["actors"]), "resources": len(actual["resources"]), "roles": installed["roles"]})
    report = {"editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "original_level_and_resources_match": True, "cases": rows}
    (out/"result.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print("PASS original 2D world/resources and semantic roles in three providers", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/platformer-composition")
    parser.add_argument("--before", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/platformer-2d-before")
    args = parser.parse_args()
    verify(args.out, args.before)
