"""Place the diagnostic probe at an existing cut endpoint in an isolated QA copy.

The selected original route is split so its endpoint is a full checkpoint.
It does not change source geometry, target camera data or the original project.
"""
import argparse
import json
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument("source",type=Path)
parser.add_argument("qa_scene",type=Path)
parser.add_argument("--route",type=int,default=34)
args=parser.parse_args()
assert args.source.resolve()!=args.qa_scene.resolve()
assert "artifacts" in args.qa_scene.resolve().parts, "Only modify an isolated QA artifact"
lines=args.source.read_text().splitlines()
line=next(i for i,s in enumerate(lines) if s.startswith("visibility_track:"))
track=json.loads(lines[line].split(": ",1)[1]);nav=track["navigation"]
route=nav["routes"][args.route]
cut=next(l for l in route["links"] if l["load_cut"])
assert cut["from_end"] and not cut["enabled"]
endpoint=route["first"]+route["count"]-1
nav["routes"].append({"first":route["first"],"count":route["count"]-1,"automatic":False,"links":[]})
route["first"]=endpoint;route["count"]=1;route["links"]=[cut]
nav["start_route"]=args.route
parameter=nav["parameters"][endpoint];camera=track["samples"][endpoint]
position=list(camera["position"]);distance=sum(parameter["limits"])/2
position[0]+=parameter["axis"][0]*distance;position[2]+=parameter["axis"][1]*distance
probe=next(i for i,s in enumerate(lines) if s.startswith('- {') and '"Navigation Probe (no collision)"' in s)
actor=json.loads(lines[probe][2:]);next(c for c in actor["components"] if c.get("root"))["properties"]["position"]=position
lines[probe]='- '+json.dumps(actor);lines[line]='visibility_track: '+json.dumps(track)
args.qa_scene.write_text('\n'.join(lines)+'\n')
print(json.dumps({"start_route":args.route,"start_sample":endpoint,"destination_route":cut["route"],
                  "initially_enabled":cut["enabled"],"action":"Cross enables cut and moves the diagnostic actor; runtime loads and commits camera"}))
