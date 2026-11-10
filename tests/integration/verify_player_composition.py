"""Audit shared player construction and compare the original arena and resources."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from verify_camera_rigs import ROOT, EDITOR, documents
from starter_bindings import player_roles


def normalize(scene, controller=None):
    scene = dict(scene)
    scene.pop("bake", None)
    scene.pop("navigation", None)
    identities = {controller: "project-controller"} if controller else {}
    for i, actor in enumerate(scene["actors"]):
        identities[actor["id"]] = f"actor:{i}"
        for j, component in enumerate(actor["components"]):
            identities[component["id"]] = f"component:{i}:{j}"

    def visit(value):
        if isinstance(value, str):
            return identities.get(value, value)
        if isinstance(value, list):
            return [visit(item) for item in value]
        if isinstance(value, dict):
            return {key: visit(item) for key, item in value.items()}
        return value
    return visit(scene)


def audit(out, before):
    out.mkdir(parents=True, exist_ok=True)
    baseline = json.loads(before.read_text(encoding="utf-8"))
    rows = []
    for template in ("third-person", "dungeon"):
        configurations = []
        for flavor in ("cpp", "blueprint", "lua"):
            project = out / f"{template}-{flavor}"
            if not project.exists():
                result = subprocess.run([str(EDITOR), "--create-project", str(project), "--template", template, "--gameplay", flavor], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
                (out/f"{template}-{flavor}-create.log").write_text(result.stdout+result.stderr, encoding="utf-8")
                assert result.returncode == 0, result.stdout+result.stderr
            manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
            scene = documents.loads((project/manifest["startup_scene"]).read_text(encoding="utf-8"))
            installed = documents.loads((project/"ProjectSettings/Template.epoksettings").read_text(encoding="utf-8"))
            player, camera = player_roles(project, scene, template, flavor)
            assert len(player["components"]) == 5 and len(camera["components"]) == 4
            visual = next(a for a in scene["actors"] if any(c["id"] == player["properties"]["visual"] for c in a["components"]))
            selected = {"actors": [player, camera, visual]}
            # Both actors share the generated gameplay grouping; compare its
            # identity as an external reference rather than its full world index.
            config = normalize(selected, installed["roles"]["controller"]["class_id"])
            parent = player.get("logical_parent")
            def external(value):
                if isinstance(value, str):
                    return "gameplay-group" if value == parent else value
                if isinstance(value, list):
                    return [external(item) for item in value]
                if isinstance(value, dict):
                    return {key: external(item) for key, item in value.items()}
                return value
            configurations.append(external(config))
            assets = {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((project/"assets").rglob("*")) if p.is_file() and p.suffix in (".epokasset", ".png")}
            if template == "third-person" and flavor == "cpp":
                assert normalize(scene) == baseline["scene"], "authored Third Person arena changed"
                assert assets == baseline["assets"], "Third Person resources changed"
            if template == "dungeon":
                assert not (project/"assets/Meshes/Optimized").exists(), "unused arena resources were installed"
                assert not any(a["name"] == "Environment" for a in scene["actors"])
                assert scene["dungeons"][0]["spawn_actor"] == player["id"]
            rows.append({"template": template, "flavor": flavor, "actors": len(scene["actors"]), "resources": len(assets), "roles": installed["roles"], "player_components": len(player["components"]), "camera_components": len(camera["components"])})
        assert all(config == configurations[0] for config in configurations), f"{template}: player/camera tuning differs across flavors"
    report = {"editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "arena_and_resource_baseline_match": True, "player_tuning_matches_across_flavors": True, "cases": rows}
    (out/"result.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print("PASS original arena/resources, shared player tuning, role bindings and no-arena Dungeon generation in six projects", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/player-composition")
    parser.add_argument("--before", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/third-person-before/composition.json")
    args = parser.parse_args()
    audit(args.out, args.before)
