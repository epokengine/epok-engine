"""Compare a cooked-render implementation against an unchanged settled scene.

No header equality requirement: compact payloads intentionally change the ABI.
Pose, projection count, drawn geometry and both framebuffer parities must agree.
Timing is a measurement of Epok in the emulator, not reference hardware.
"""
import argparse
import json
from pathlib import Path
from audit_renderer_work import profile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    baseline = profile(args.baseline, 220)
    candidate = profile(args.candidate, 220)
    assert baseline['pose'] == candidate['pose'], 'Camera/target differs'
    assert baseline['build']['display_header_sha256'] == candidate['build']['display_header_sha256']
    for key in ('gte_vertices', 'software_vertices', 'triangles', 'visible_chunks'):
        assert baseline['median'][key] == candidate['median'][key], key
    left, right = [(p / 'vram.bin').read_bytes() for p in (args.baseline, args.candidate)]
    assert len(left) == len(right) == 1024*512*2
    differences = [sum(left[i:i+2] != right[i:i+2]
                       for y in range(parity*240, (parity+1)*240)
                       for i in range(y*2048, y*2048+1024, 2)) for parity in range(2)]
    assert differences == [0, 0], differences
    old, new = [p['median']['frame_microseconds'] for p in (baseline, candidate)]
    report = dict(baseline=baseline, candidate=candidate, differing_pixels= differences,
                  reduction_percent=(old-new)/old*100, fps_before=1e6/old, fps_after=1e6/new)
    args.output.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('baseline','candidate')}, indent=2))


if __name__ == '__main__':
    main()
