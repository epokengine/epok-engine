"""Serial P7B shared-player movement, arena, Dungeon and export acceptance."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
from verify_camera_rigs import ROOT


def verify(out):
    out.mkdir(parents=True, exist_ok=True)

    def run(script, *arguments, label):
        result = subprocess.run([sys.executable, str(ROOT/"tests/integration"/script), *map(str, arguments)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=5400)
        (out/(label+".log")).write_text(result.stdout+result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout[-5000:]+result.stderr[-5000:]
        print("PASS", label, flush=True)

    run("verify_player_composition.py", "--out", out/"composition", label="composition")
    for script, directory in (("verify_third_person_foundation.py", "foundation"),
                              ("verify_third_person_routes.py", "arena"),
                              ("verify_dungeon_foundation.py", "dungeon")):
        run(script, "--out", out/directory, label=directory)
    for flavor in ("cpp", "blueprint", "lua"):
        run("verify_third_person_foundation.py", "--out", out/"foundation", "--flavor", flavor, "--standalone-only", label="export-"+flavor)
    rows = {directory: [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((out/directory/flavor/"result.json").read_text())] for directory in ("foundation", "arena", "dungeon")}
    assert all(len(cases) == 5 and len({tuple(row["sample"][1:]) for row in cases}) == 1 for cases in rows.values())
    exports = [json.loads((out/"foundation"/flavor/"standalone-result.json").read_text()) for flavor in ("cpp", "blueprint", "lua")]
    assert all(row["identical"] for row in exports)
    report = {"execution_cases": sum(map(len, rows.values())), "matrix": rows, "exports": exports,
              "composition": json.loads((out/"composition/result.json").read_text())}
    (out/"result.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print("PASS 15 shared-player execution cases, three standalone exports and original-world composition", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/player-routes")
    verify(parser.parse_args().out)
