"""Serial P7C world audit, movement/surface/pickup routes and standalone exports."""
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

    run("verify_platformer_composition.py", "--out", out/"composition", label="composition")
    for script, directory in (("verify_platformer_dialogue.py", "dialogue"),
                              ("verify_platformer_foundation.py", "foundation"),
                              ("verify_platformer_routes.py", "surfaces"),
                              ("verify_platformer_pickups.py", "pickups")):
        run(script, "--out", out/directory, label=directory)
    for flavor in ("cpp", "blueprint", "lua"):
        run("verify_platformer_foundation.py", "--out", out/"foundation", "--flavor", flavor, "--standalone-only", label="export-"+flavor)
    rows = {directory: [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((out/directory/flavor/"result.json").read_text())] for directory in ("foundation", "surfaces", "pickups", "dialogue")}
    assert all(len(cases) == 5 and len({tuple(row["sample"][1:]) for row in cases}) == 1 for cases in rows.values())
    exports = [json.loads((out/"foundation"/flavor/"standalone-result.json").read_text()) for flavor in ("cpp", "blueprint", "lua")]
    assert all(row["identical"] for row in exports)
    report = {"execution_cases": sum(map(len, rows.values())), "matrix": rows, "exports": exports,
              "composition": json.loads((out/"composition/result.json").read_text()),
              "remaining_acceptance": ["final workspace regression and evidence audit"]}
    (out/"result.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print("PASS 20 Platformer execution cases, three exports, dialogue headroom and original-world comparison", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/platformer-routes")
    verify(parser.parse_args().out)
