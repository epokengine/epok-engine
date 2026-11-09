"""Attribute the opt-in console PC histogram without inventing sub-bin precision."""
import argparse
import collections
import json
from pathlib import Path
import re
import subprocess


def summarize(samples, symbols):
    """Only attribute a bin to a function when that function covers its full range."""
    totals = collections.Counter()
    ambiguous = []
    unmapped = []
    hot_bins = []
    base, width = samples['base'], 1 << samples['shift']
    for index, count in enumerate(samples['bins']):
        if not count:
            continue
        start, end = base + index * width, base + (index + 1) * width
        candidates = [(lo, hi, name) for lo, hi, name in symbols if lo < end and hi > start]
        covering = [(lo, hi, name) for lo, hi, name in candidates if lo <= start and hi >= end]
        hot_bins.append({'start': start, 'end': end, 'samples': count,
                         'candidates': sorted({name for _, _, name in candidates})})
        if len(covering) == 1 and len(candidates) == 1:
            totals[covering[0][2]] += count
        else:
            row = {'start': start, 'end': end, 'samples': count,
                   'candidates': sorted({name for _, _, name in candidates})}
            (ambiguous if candidates else unmapped).append(row)
    inside = sum(samples['bins'])
    outside = samples['outside']
    return {
        'sample_count': inside + outside, 'inside_samples': inside,
        'outside_samples': outside, 'window': samples.get('window'),
        'bin_bytes': width,
        'attributed_samples': sum(totals.values()),
        'functions': [{'name': name, 'samples': count}
                      for name, count in totals.most_common()],
        'ambiguous_samples': sum(row['samples'] for row in ambiguous),
        'ambiguous_bins': sorted(ambiguous, key=lambda row: -row['samples']),
        'unmapped_samples': sum(row['samples'] for row in unmapped),
        'unmapped_bins': sorted(unmapped, key=lambda row: -row['samples']),
        'hot_bins': sorted(hot_bins, key=lambda row: -row['samples'])[:100],
        'limits': [
            'Counts describe the instrumented executable and selected window only.',
            'Timer interrupts can be deferred while interrupts are masked; ISR work may be underrepresented.',
            'A bin overlapping multiple functions is not divided or assigned to one candidate.',
            'Inlining is attributed to the containing linked function; counts are not inclusive call costs.',
        ],
    }


def read_symbols(text):
    symbols = []
    for line in text.splitlines():
        match = re.match(r'^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+[TtWw]\s+(.+)$', line)
        if match:
            address, size = (int(match[i], 16) for i in (1, 2))
            if size:
                start = address & 0x1fffff
                symbols.append((start, start + size, match[3]))
    # Aliases for an identical linked range must not multiply attribution.
    ranges = collections.defaultdict(set)
    for start, end, name in symbols:
        ranges[start, end].add(name)
    return [(start, end, ' | '.join(sorted(names)))
            for (start, end), names in sorted(ranges.items())]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--samples', type=Path, required=True)
    parser.add_argument('--elf', type=Path, required=True)
    parser.add_argument('--nm', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    symbols = subprocess.check_output([str(args.nm), '-n', '-S', '-C', '--defined-only',
                                       str(args.elf)], text=True)
    report = summarize(json.loads(args.samples.read_text()), read_symbols(symbols))
    report['elf'] = str(args.elf)
    report['samples_file'] = str(args.samples)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: report[key] for key in ('sample_count', 'attributed_samples',
          'ambiguous_samples', 'unmapped_samples', 'outside_samples')}, indent=2))
    for row in report['functions'][:20]:
        print(f"{row['samples']:7d} {row['name']}")


if __name__ == '__main__':
    main()
