"""Verify a visibility-decoder A/B without equating camera rounding with lost faces.

Requires identical cooked scene/geometry/display and verifies both linked code
images in RAM. Compare every common frame; camera-dependent packet counts must
be identical whenever the published projection is identical. End masks are
compared at their independently linked addresses. Not a pixel/whole-route test.
"""
import argparse
import hashlib
import json
from pathlib import Path

from analyze_epok_cadence import analyze, unpack_rows
from analyze_epok_function_costs import costs
from capture_epok_cadence import elf_symbols, function_cost_hooks
from sample_epok_runtime import executable_sections, verify_running_code


def compare_rows(before, after):
    count=min(len(before),len(after))
    same_pose=0
    pose_fields=lambda r: {k:v for k,v in r['ordered'].items()
                           if k.startswith(('camera_', 'row_scale')) or k=='sample'}
    outputs=lambda r: {k:v for k,v in r['ordered'].items() if k not in ('frame','frame_microseconds')}
    for a,b in zip(before,after):
        if a['performance']['frame']!=b['performance']['frame']:
            raise ValueError('Unpaired frame numbers')
        if a['visibility']!=b['visibility'] or a['ordered']['candidates']!=b['ordered']['candidates']:
            raise ValueError('Visibility workload changed')
        if pose_fields(a)==pose_fields(b):
            if outputs(a)!=outputs(b):raise ValueError('Draw outputs changed at identical camera pose')
            same_pose+=1
    return dict(paired_frames=count,identical_projection_and_ordered_counters=same_pose,
                differing_camera_pose=count-same_pose,
                all_paired_visibility_counters_and_candidate_counts_identical=True)


def load(folder):
    raw={n:(folder/n).read_bytes() for n in ('profile.json','observer-config.json','epok.elf','before.bin','after.bin')}
    p,c=json.loads(raw['profile.json']),json.loads(raw['observer-config.json'])
    symbols=elf_symbols(raw['epok.elf']);sections=executable_sections(raw['epok.elf'])
    if c['function_costs']!=function_cost_hooks(symbols,sections):raise ValueError('Stale function hooks')
    for n in ('before.bin','after.bin'):verify_running_code(raw[n],sections)
    costs(p,c)  # Reject malformed/overlapping calls and incoherent publications.
    return raw,p,c,symbols


def compare(before, after):
    a,ap,ac,asyms=load(before);b,bp,bc,bsyms=load(after)
    same={}
    for n in ('scene.hh','GEOMETRY.BIN','display.hh'):
        data=(before/n).read_bytes()
        if data!=(after/n).read_bytes():raise ValueError('Cooked input changed: '+n)
        same[n]=hashlib.sha256(data).hexdigest()
    ar,br=unpack_rows(ap,ac),unpack_rows(bp,bc)
    result=compare_rows(ar,br)
    if ar[-1]['visibility']!=br[-1]['visibility']:raise ValueError('End navigation state differs')
    masks={}
    for suffix in ('13track_currentE','10track_nextE','11rail_anchorE'):
        names=[n for n in asyms if n.endswith(suffix)]
        if len(names)!=1 or names[0] not in bsyms:raise ValueError('Ambiguous/missing mask symbol')
        n=names[0];aa,az,_=asyms[n];ba,bz,_=bsyms[n]
        aa&=0x1fffffff;ba&=0x1fffffff
        if az!=bz or aa+az>len(a['after.bin']) or ba+bz>len(b['after.bin']):raise ValueError('Mask bounds mismatch')
        av,bv=a['after.bin'][aa:aa+az],b['after.bin'][ba:ba+bz]
        if av!=bv:raise ValueError('End visibility mask changed: '+n)
        masks[n]=dict(bytes=az,sha256=hashlib.sha256(av).hexdigest())
    summaries=[]
    for folder,p,c in ((before,ap,ac),(after,bp,bc)):
        r=analyze(p,c);memory=json.loads((folder/'memory-report.json').read_text())
        summaries.append(dict(capture=str(folder),cpu_ms=r['frame_cpu_work_ms'],
                              display_origin_ms=r['completed_ordered_display_origin_interval_ms'],
                              intervals_over_30hz=r['completed_ordered_intervals_over_30hz'],
                              counters=r['counters'],maximum_failures=r['maximum_failures'],
                              ram_used=memory['ram']['used'],vram_used=memory['scenes'][0]['vram']['used']))
    return dict(scope=__doc__,**result,identical_cooked_inputs=same,identical_end_masks=masks,
                summaries=summaries,
                input_sha256=[{n:hashlib.sha256(v).hexdigest() for n,v in x.items()} for x in (a,b)])


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('before',type=Path);p.add_argument('after',type=Path);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args()
    if args.output.exists():p.error('Refusing to overwrite evidence')
    result=compare(args.before,args.after)
    args.output.write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in result.items() if k not in ('summaries','input_sha256')},indent=2))


if __name__=='__main__':main()
