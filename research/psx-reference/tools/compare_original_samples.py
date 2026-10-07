"""Match captured original SLST words, in order, against the source route export.

This checks four original runtime snapshots, not rendered pixels, UVs, dynamic
actors, frame cost, or every sample of the level. Cameras are compared only after
matching entire token lists; proximity alone is never treated as equivalence.
"""
import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path

from analyze_original_profile import analyze, signed


def compare(profile, routes):
    analyze(profile)
    if profile["version"] != 3 or routes["coordinate_scale"] != .00025:
        raise ValueError("Expected corrected original observer and native /4000 source routes")
    captured = profile.get("world_samples", [])
    if not captured:
        raise ValueError("No original world snapshots")
    by_tokens = defaultdict(list)
    samples = []
    for route_index, route in enumerate(routes["routes"]):
        for sample_index, sample in enumerate(route["Samples"]):
            entry = dict(route=route_index, source_sample=sample_index,
                         zone=route["Zone"], camera=route["Camera"],
                         position_native=[v*4000 for v in sample["Position"]])
            by_tokens[tuple(sample["Polygons"])].append(entry)
            samples.append((entry, sample["Polygons"]))
    result = []
    for snapshot in captured:
        if len(snapshot)<42 or not 1<snapshot[2]<=4096 or len(snapshot)!=42+snapshot[2]:
            raise ValueError("Invalid original snapshot layout/count")
        tokens = snapshot[42:]
        if any(not isinstance(v,int) or not 0<=v<=65535 for v in tokens):
            raise ValueError("Invalid SLST token")
        position = [signed(v)/256 for v in snapshot[4:7]]
        distance = lambda entry: math.dist(position,entry["position_native"])
        matches = [dict(entry, camera_distance_native=distance(entry))
                   for entry in by_tokens[tuple(tokens)]]
        nearest, nearby_tokens = min(samples,key=lambda item:distance(item[0]))
        result.append(dict(cycles=snapshot[0], camera_native=position, slst_header_count=snapshot[2],
                           slst_aux_halfword=snapshot[3], ordered_tokens_sha256=hashlib.sha256(
                               b"".join(v.to_bytes(2,"little") for v in tokens)).hexdigest(),
                           exact_ordered_list_matches=matches,
                           nearest_source_camera=dict(nearest,camera_distance_native=distance(nearest),
                                                       token_count=len(nearby_tokens),
                                                       only_original=len(set(tokens)-set(nearby_tokens)),
                                                       only_export=len(set(nearby_tokens)-set(tokens))),
                           gte_cp2c=snapshot[10:42]))
    return dict(scope=__doc__, snapshots=len(result),
                exact_ordered_list_snapshots=sum(bool(r["exact_ordered_list_matches"]) for r in result),
                rows=result)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile",type=Path)
    parser.add_argument("routes",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    if args.output.exists():parser.error("Refusing to overwrite evidence")
    inputs={p:p.read_bytes() for p in (args.profile,args.routes)}
    result=compare(json.loads(inputs[args.profile]),json.loads(inputs[args.routes]))
    result["input_sha256"]={str(p):hashlib.sha256(v).hexdigest() for p,v in inputs.items()}
    with args.output.open("x",encoding="utf-8") as target:json.dump(result,target,indent=2)
    print(json.dumps({k:v for k,v in result.items() if k!="rows"},indent=2))
    for row in result["rows"]:
        print(json.dumps({k:v for k,v in row.items() if k!="gte_cp2c"}))


if __name__=="__main__":main()
