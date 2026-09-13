"""Capture the actual file Inspector and its independent GPU model preview."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import wave

REPO = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", type=Path, default=REPO / "target/debug/epok-editor.exe")
    parser.add_argument("--output", type=Path, default=REPO / "artifacts/asset-inspector")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    project = Path(tempfile.mkdtemp(prefix="epok-inspector-visual-")) / "Inspector Showcase"
    env = dict(os.environ, EPOK_EDITOR_HOME=str(REPO), LOCALAPPDATA=str(output / "profile"))

    def run(*flags):
        subprocess.run([str(args.editor.resolve()), *map(str, flags)], cwd=REPO,
                       env=env, check=True, timeout=120)

    run("--create-project", project, "--name", "Inspector Showcase")
    assets = project / "assets"
    shutil.copyfile(REPO / "resources/branding/epok.png", assets / "Logo.png")
    run("--project", project, "--import-texture", "assets/Logo.png", "--asset", "assets/Logo.epokasset")
    shutil.copyfile(REPO / "resources/models/EpokMannequin.fbx", assets / "Mannequin.fbx")
    run("--project", project, "--import-fbx", "assets/Mannequin.fbx")
    with wave.open(str(assets / "Tone.wav"), "wb") as audio:
        audio.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        audio.writeframes(b"\0\0" * 22050)
    run("--project", project, "--import-audio", "assets/Tone.wav", "--asset", "assets/Tone.epokasset")
    model = next(assets.rglob("SkeletalMesh.epokasset")).relative_to(project).as_posix()
    for name, path in [("model", model), ("texture", "assets/Logo.epokasset"),
                       ("audio", "assets/Tone.epokasset"), ("raw-model", "assets/Mannequin.fbx")]:
        run("--project", project, "--inspect-asset", path, "--window-size", "1440x900",
            "--screenshot", output / (name + ".png"))
    (output / "project.txt").write_text(str(project), encoding="utf-8")
    print(f"Inspector fixture: {project}")


if __name__ == "__main__":
    main()
