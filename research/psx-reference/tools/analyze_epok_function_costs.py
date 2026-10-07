"""Inclusive MIPS function-cycle costs from a read-only, ELF-verified observer.

Timings include nested calls/interrupts. Per-function totals must not be added
together when scopes nest (rail contains aim and some ordered preparation).
Only calls fully contained in complete CPU frames enter frame distributions.
"""
import argparse
import bisect
import hashlib
import json
from pathlib import Path

from analyze_epok_cadence import analyze, unpack_rows
from analyze_original_profile import distribution
from capture_epok_cadence import elf_symbols, function_cost_hooks
from sample_epok_runtime import executable_sections, verify_running_code


def costs(profile, config):
    analyze(profile, config)  # Full publication/counter validation first.
    trace=profile.get('function_costs', {})
    hooks=config.get('function_costs', [])
    if trace.get('version') != 1 or not hooks or not trace.get('calls'):
        raise ValueError('Missing function costs')
    if any(type(i) is not int or not 1<=i<=len(hooks) for i in trace.get('incomplete', [])):
        raise ValueError('Invalid incomplete function index')
    rows=unpack_rows(profile, config)
    ends=[r['cycles'] for r in rows]
    clock=profile['cpu_clock_hz']
    complete=[i for i,r in enumerate(rows) if r['cpu_cycles']>=0 and r['ordered']['frame']>0
              and r['ordered']['frame']==r['performance']['frame']]
    complete_set=set(complete)
    report=[]
    buckets=[[] for _ in hooks]
    last_end=[-1 for _ in hooks]
    previous_end=-1
    excluded=0
    for call in trace['calls']:
        if len(call)!=5 or any(type(v) is not int for v in call):raise ValueError('Invalid cost row')
        index,start,end,ra,sp=call
        if not 1<=index<=len(hooks):raise ValueError('Unknown cost index')
        h=hooks[index-1]
        if start<0 or end<=start or ra not in h['returns'] or not 0x80000000<=sp<=0x80200000 or sp%4:
            raise ValueError('Invalid cost bounds/return/stack')
        if end<previous_end or start<last_end[index-1]:raise ValueError('Overlapping/reordered cost row')
        previous_end=end;last_end[index-1]=end
        frame=bisect.bisect_left(ends,end)
        if frame not in complete_set or start<ends[frame]-rows[frame]['cpu_cycles']:
            excluded+=1;continue
        buckets[index-1].append((frame,start,end))
    ms=lambda cycles:distribution([c*1000/clock for c in cycles])
    for h,calls in zip(hooks,buckets):
        totals={i:0 for i in complete}
        counts={i:0 for i in complete}
        for frame,start,end in calls:
            totals[frame]+=end-start;counts[frame]+=1
        if any(total>rows[i]['cpu_cycles'] for i,total in totals.items()):
            raise ValueError('Function total exceeds its CPU frame')
        report.append(dict(label=h['label'],calls=len(calls),frames_with_calls=sum(v>0 for v in counts.values()),
                           inclusive_call_ms=ms([end-start for _,start,end in calls]),
                           inclusive_per_complete_frame_ms=ms(list(totals.values())),
                           slow_cpu_frames=[dict(frame=rows[i]['performance']['frame'],
                                                sample=rows[i]['visibility'].get('sample'),
                                                calls=counts[i],function_ms=totals[i]*1000/clock,
                                                cpu_ms=rows[i]['cpu_cycles']*1000/clock)
                                            for i in complete if rows[i]['cpu_cycles']>clock/30]))
    return dict(scope=__doc__,complete_frames=len(complete),excluded_boundary_calls=excluded,
                incomplete_at_stop=trace.get('incomplete',[]),functions=report)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('capture',type=Path);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args()
    if args.output.exists():p.error('Refusing to overwrite evidence')
    inputs={n:(args.capture/n).read_bytes() for n in ('profile.json','observer-config.json','epok.elf','before.bin','after.bin')}
    config=json.loads(inputs['observer-config.json'])
    sections=executable_sections(inputs['epok.elf'])
    if config.get('function_costs')!=function_cost_hooks(elf_symbols(inputs['epok.elf']),sections):
        raise ValueError('Cost hooks differ from linked ELF/direct callers')
    for name in ('before.bin','after.bin'):verify_running_code(inputs[name],sections)
    result=costs(json.loads(inputs['profile.json']),config)
    result['input_sha256']={n:hashlib.sha256(v).hexdigest() for n,v in inputs.items()}
    args.output.write_text(json.dumps(result,indent=2))
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
