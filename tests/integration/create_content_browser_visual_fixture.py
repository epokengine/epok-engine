"""Create an isolated Content Browser visual fixture; never alter a user's project."""
import argparse
from pathlib import Path
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", type=Path, default=REPO / "target/debug/epok-editor.exe")
    parser.add_argument("--destination", type=Path)
    parser.add_argument("--screenshot", type=Path)
    parser.add_argument("--size", default="1258x342")
    args = parser.parse_args()
    destination = (args.destination or Path(tempfile.mkdtemp(prefix="epok-content-visual-")) / "QAGame").resolve()
    if destination.exists():
        raise SystemExit("Choose a new destination. Existing projects are never overwritten.")
    editor = args.editor.resolve()
    subprocess.run([str(editor), "--create-project", str(destination), "--name", "QAGame"], cwd=REPO, check=True)
    for name in ("AI", "Alembic", "Animation", "Blueprints", "Characters", "Cloth", "Cloth_Test", "Core", "Curves"):
        (destination / "assets" / name).mkdir()
    for name in ("AI", "Animation", "Blueprints", "Characters", "Cloth", "Cloth_Test", "Core"):
        (destination / "assets" / name / "Examples").mkdir()
    command = [str(editor), "--project", str(destination), "--window-size", args.size, "--screenshot-content-browser"]
    if args.screenshot:
        output = args.screenshot.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run([*command, "--screenshot", str(output)], cwd=REPO, check=True, timeout=120)
    else:
        print(subprocess.list2cmdline(command))


if __name__ == "__main__":
    main()
