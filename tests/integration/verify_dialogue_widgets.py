"""Create normal dialogue widget classes through the shipped CLI and capture the UI designer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p4/widget-presets")
    parser.add_argument("--screenshots", action="store_true")
    parser.add_argument("--sample", action="append", choices=("line", "prompt", "choices", "closed"),
                        help="Preview states to capture for each style and resolution")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    editor = ROOT / "target/debug/epok-editor.exe"

    def run(*command, log):
        result = subprocess.run([str(editor), *map(str, command)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=300)
        (out / log).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, (log, (result.stdout + result.stderr)[-3000:])
        return result.stdout.strip()

    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Dialogue Widget Presets", log="create.log")
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    manifest["rendering"].update(width=320, height=240)
    documents.write_text(descriptor, documents.dumps(manifest))
    existing = {c["cpp_name"]: c["id"] for c in json.loads(run("--project", project, "--reflect", log="reflect.log"))["classes"]}
    lua_registry = project / "ProjectSettings/LuaClasses.epoksettings"
    lua_classes = documents.loads(lua_registry.read_text(encoding="utf-8"))["classes"] if lua_registry.is_file() else {}
    rows = []
    for presentation in ("classic_box", "portrait_conversation", "subtitles", "speech_bubble"):
        for flavor in ("cpp", "blueprint", "lua"):
            name = "Widget" + flavor.title() + "".join(s.title() for s in presentation.split("_"))
            log = name + ".log"
            source = project / (f"assets/Blueprints/{name}.epokbp" if flavor == "blueprint" else f"assets/scripts/{name}." + ("hpp" if flavor == "cpp" else "lua"))
            # Reflection does not always publish an uncompiled Blueprint asset.
            # Its ordinary document is authoritative when reusing this fixture.
            blueprint = documents.loads(source.read_text(encoding="utf-8")) if flavor == "blueprint" and source.is_file() else None
            if blueprint is not None:
                assert blueprint["name"] == name, (source, blueprint["name"])
            lua = lua_classes.get(f"assets/scripts/{name}.lua") if flavor == "lua" else None
            identity = existing.get(name) or (blueprint["id"] if blueprint else None) or lua or run("--project", project, "--new-dialogue-widget", name, "--gameplay", flavor,
                                                "--presentation", presentation, log=log)
            assert len(identity) == 36, (name, identity)
            companion = source if flavor == "blueprint" else project / f"assets/Classes/{identity}.epokclass"
            assert source.is_file() and companion.is_file()
            before = companion.read_bytes()
            rejected = subprocess.run([str(editor), "--project", str(project), "--new-dialogue-widget", name,
                                       "--gameplay", flavor, "--presentation", presentation], cwd=ROOT,
                                      capture_output=True, text=True, timeout=300)
            assert rejected.returncode != 0 and companion.read_bytes() == before
            rows.append({"presentation": presentation, "provider": flavor, "class": name, "id": identity,
                         "source": str(source.relative_to(project)), "composition": str(companion.relative_to(project)),
                         "composition_sha256": hashlib.sha256(before).hexdigest(), "duplicate_rejected": True})
            print("PASS", presentation, flavor, "editable class and protected identity", flush=True)
    screenshots = []
    if args.screenshots:
        for n, presentation in enumerate(("classic_box", "portrait_conversation", "subtitles", "speech_bubble")):
            row = next(r for r in rows if r["presentation"] == presentation and r["provider"] == ("cpp", "blueprint", "lua", "cpp")[n])
            for width, height in ((320, 240), (640, 480)):
                manifest["rendering"].update(width=width, height=height)
                documents.write_text(descriptor, documents.dumps(manifest))
                samples = args.sample or ["choices" if presentation == "speech_bubble" else "line"]
                for sample in samples:
                    suffix = "-" + sample if args.sample else ""
                    path = out / f"{presentation}-{width}{suffix}.png"
                    run("--project", project, "--open-ui-class", row["class"], "--ui-preview", sample,
                        "--screenshot", path, log=path.stem + ".log")
                    assert path.is_file() and path.stat().st_size > 1000
                    screenshots.append({"file": path.name, "provider": row["provider"], "sample": sample,
                                        "display": [width, height], "inspected": False})
    for row in rows:
        assert hashlib.sha256((project / row["composition"]).read_bytes()).hexdigest() == row["composition_sha256"], row
        row["composition_preserved"] = True
    result = {"editor_sha256": hashlib.sha256(editor.read_bytes()).hexdigest(), "classes": rows,
              "screenshots": screenshots, "scope": "CLI creation, identity protection and generic UI class sample previews"}
    (out / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
