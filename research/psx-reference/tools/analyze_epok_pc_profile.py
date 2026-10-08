"""Attribute a basic-block cycle profile to source lines of the linked ELF.

Reads a capture made with capture_epok_cadence.py --pc-profile and the ELF
snapshot stored in that capture. Cycles are exact per basic block (between
consecutive block leaders); within a block they are split evenly across its
instructions, which is an approximation. Instructions are grouped by their
outermost source line inside the profiled function and by their innermost
inlined function. A call's callee time stays in the calling block. Captures
from the earlier per-instruction mode are read unchanged. Read-only.
"""
import argparse
import collections
import json
from pathlib import Path
import shutil
import subprocess


def addr2line(tool, elf, addresses):
    """Inline chains, innermost first: [(function, file, line), ...] per address."""
    result = subprocess.run([tool, "-e", str(elf), "-f", "-i", "-C", "-a", *[hex(a) for a in addresses]],
                            check=True, capture_output=True, text=True).stdout.splitlines()
    chains, current = {}, None
    index = 0
    while index < len(result):
        line = result[index]
        if line.startswith("0x"):
            current = int(line, 16)
            chains[current] = []
            index += 1
            continue
        function, location = line, result[index + 1]
        path, _, number = location.rpartition(":")
        number = number.split()[0]
        chains[current].append((function, path, int(number) if number.isdigit() else 0))
        index += 2
    return chains


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--addr2line", default=shutil.which("mipsel-none-elf-addr2line"))
    args = parser.parse_args()
    if not args.addr2line:
        parser.error("mipsel-none-elf-addr2line not found; pass --addr2line")
    if args.output.exists():
        parser.error("Refusing to overwrite analysis output")
    profile = json.loads((args.capture / "profile.json").read_text())["pc_profile"]
    if profile["discarded"] or profile["open"] or not profile["frames"]:
        raise SystemExit("Incomplete profile: discarded/open interval or no profiled frames")
    measured = {pc: (hits, cycles) for pc, hits, cycles in profile["pcs"]}
    config = json.loads((args.capture / "observer-config.json").read_text())["pc_profile"]
    end = config["entry"] + config["size"]
    leaders = config.get("leaders") or list(range(config["entry"], end, 4))
    blocks, rows = [], {}
    for index, leader in enumerate(leaders):
        stop = leaders[index + 1] if index + 1 < len(leaders) else end
        hits, cycles = measured.get(leader, (0, 0))
        if not hits:
            continue
        length = (stop - leader) // 4
        blocks.append(dict(start=hex(leader), instructions=length, executions=hits / profile["frames"],
                           cycles_per_frame=cycles / profile["frames"]))
        for pc in range(leader, stop, 4):
            rows[pc] = (hits, cycles / length)
    chains = addr2line(args.addr2line, args.capture / "epok.elf", sorted(rows))
    frames = profile["frames"]
    total = sum(cycles for _, cycles in rows.values())
    by_line, by_function = collections.Counter(), collections.Counter()
    hits_by_line, pcs_by_line = collections.Counter(), collections.defaultdict(list)
    for pc, (hits, cycles) in rows.items():
        chain = chains[pc]
        outer = chain[-1]
        key = f"{Path(outer[1]).name}:{outer[2]}"
        by_line[key] += cycles
        hits_by_line[key] += hits
        pcs_by_line[key].append(pc)
        by_function[chain[0][0]] += cycles
    def table(counter):
        return [dict(key=k, cycles_per_frame=v / frames, percent=100 * v / total) for k, v in counter.most_common()]
    lines = table(by_line)
    for entry in lines:
        entry["instructions_per_frame"] = hits_by_line[entry["key"]] / frames
        entry["pcs"] = [hex(p) for p in sorted(pcs_by_line[entry["key"]])]
    report = dict(scope=__doc__, frames=frames, entry=hex(profile["entry"]), size=profile["size"],
                  cycles_per_frame=total / frames,
                  instructions_per_frame=sum(h for h, _ in rows.values()) / frames,
                  granularity="basic block" if config.get("leaders") else "instruction",
                  indirect_jumps=config.get("indirect_jumps"),
                  by_source_line=lines, by_inlined_function=table(by_function),
                  by_block=sorted(blocks, key=lambda b: -b["cycles_per_frame"]))
    args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(dict(cycles_per_frame=report["cycles_per_frame"],
                          instructions_per_frame=report["instructions_per_frame"]), indent=None))


if __name__ == "__main__":
    main()
