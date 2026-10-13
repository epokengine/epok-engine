"""Check offline production-camera results against exact latched live PSX records.

Interpolated views need not equal authored samples. Only bit-identical origin,
basis and row scale records count; a nearby position is not accepted instead.
"""
import argparse
import hashlib
import json
from pathlib import Path
from analyze_original_profile import signed


def validate(probe, profile):
    rows=probe['rows']
    if any(len(row)!=29 or row[0]!=i for i,row in enumerate(rows)):
        raise ValueError('Invalid camera probe layout')
    fields=([f'camera_origin_{c}' for c in range(3)]
            +[f'camera_basis_{r}_{c}' for r in range(3) for c in range(4)]
            +[f'row_scale_{r}' for r in range(2)])
    by_key={}
    for row in rows:by_key.setdefault(tuple(row[1:18]),[]).append(row[0])
    matches=[];empty_frames=0
    for record in profile['rows']:
        draw=record['ordered_draw']
        if not draw['frame'] or draw['frame']!=record['frame']:
            raise ValueError('Unlatched live record')
        if draw['sample']==0xffffffff:
            empty_frames+=1;continue
        if draw['frame_microseconds']!=record['frame_microseconds']:
            raise ValueError('Unlatched live timing')
        key=tuple(signed(draw[name]) for name in fields)
        if key in by_key:matches.append(dict(frame=record['frame'],source_samples=by_key[key]))
    if not matches:raise ValueError('No exact live camera records matched the offline probe')
    return dict(scope=__doc__,live_records=len(profile['rows']),exact_live_records=len(matches),
                completed_frames_without_ordered_pass=empty_frames,
                unique_source_samples=len({i for row in matches for i in row['source_samples']}),matches=matches)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('probe',type=Path);parser.add_argument('profile',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.output.exists():parser.error('Refusing to overwrite evidence')
    inputs={p:p.read_bytes() for p in (args.probe,args.profile)}
    report=validate(*(json.loads(inputs[p]) for p in (args.probe,args.profile)))
    report['input_sha256']={str(p):hashlib.sha256(data).hexdigest() for p,data in inputs.items()}
    with args.output.open('x',encoding='utf-8') as target:json.dump(report,target,indent=2)
    print(json.dumps({k:v for k,v in report.items() if k!='matches'},indent=2))


if __name__=='__main__':main()
