"""End-to-end check of the PocketStation link in Play.

Copies the sample project, adds the Star Catcher PocketStation app and gives
the sample's C++ component a PocketStation routine, then plays the game in
PCSX-Redux with `--play-psx`. The game talks to the editor's emulated
PocketStation through the emulator bridge. The check expects that the game:

* installs Star Catcher on the PocketStation's card and starts it,
* sends it a message (5 bonus stars, which Pocket Lua adds to the score),
* reads the score back through the shared link values (FUNC 82h), and
* switches the PocketStation's LED on once the score arrived.

Run from the repository root after `cargo build --locked`:
    python3 tests/integration/verify_pocketstation.py [--keep DIR]
Requires the MIPS toolchain and PCSX-Redux from setup.
"""
import argparse
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

BEHAVIOUR = """#pragma once
#include "epok.hpp"
#include "pocketstation_apps.hpp"
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="a997b0f2-b3ac-45c2-9d75-9ec11dc890af") Spinner : public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("a997b0f2-b3ac-45c2-9d75-9ec11dc890af");
    uint64_t class_id() const override {return static_class_id;}
    EPOK_PROPERTY(Editable) epok::Fixed speed=45.0;
    uint32_t frames=0;
    void tick(epok::Fixed dt) override {
        auto* owner=get_owner();
        auto* root=owner && epok::active_object_registry ? epok::active_object_registry->resolve<epok::SceneComponent3D>(owner->root_id()) : nullptr;
        if(root && root->transform) root->transform->rotation[1]+=speed*dt;
        ++frames;
        if(frames==60){
            if(const auto* app=epok::find_pocket_app("@CARD@"))
                epok::pocketstation.install_and_launch(*app);
        }
        if(frames==150){uint8_t bonus=5;epok::pocketstation.send(&bonus,1);}
        if(frames>=210&&frames%30==0){
            int32_t values[16]={};
            if(epok::pocketstation.values(values)==epok::PocketError::OK)
                epok::pocketstation.set_led(values[0]==5);
        }
    }
};
"""


def editor():
    name = "epok-editor.exe" if platform.system() == "Windows" else "epok-editor"
    path = ROOT / "target/debug" / name
    assert path.is_file(), f"build the editor first: {path}"
    return path


def run(*args, timeout=900):
    result = subprocess.run([str(a) for a in args], capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=timeout)
    if result.returncode != 0:
        sys.exit(f"{' '.join(map(str, args))} failed:\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--keep", type=Path, help="work in this folder and keep it")
    args = parser.parse_args()
    work = args.keep or Path(tempfile.mkdtemp(prefix="epok-pocketstation-"))
    project = work / "project"
    if project.exists():
        shutil.rmtree(project)
    shutil.copytree(ROOT / "examples/sample-game", project,
                    ignore=shutil.ignore_patterns(".epok", ".luarc.json"))
    created = run(editor(), "--project", project, "--new-pocketstation-app", "Star Catcher").split()
    card = created[-1]
    print(f"Created {created[0]} ({card})")
    (project / "assets/scripts/Spinner.hpp").write_text(BEHAVIOUR.replace("@CARD@", card))
    output = run(editor(), "--project", project, "--play-psx", "--stop-after", "25", timeout=1800)
    summary = next((line for line in output.splitlines() if line.startswith("PocketStation:")), None)
    print(summary or output[-2000:])
    failures = []
    if not summary:
        failures.append("the game never talked to the PocketStation")
    else:
        for expected in ("running Star Catcher", "LED on", f"card: {card}"):
            if expected not in summary:
                failures.append(f"expected `{expected}`")
    if not args.keep:
        shutil.rmtree(work, ignore_errors=True)
    if failures:
        sys.exit("FAILED: " + "; ".join(failures))
    print("PocketStation link: OK")


if __name__ == "__main__":
    main()
