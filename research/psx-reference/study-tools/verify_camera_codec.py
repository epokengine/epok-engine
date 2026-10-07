"""Compare an uncompressed and a compressed generated scene.hh exactly."""
import argparse
import json
import re
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument("before",type=Path)
parser.add_argument("after",type=Path)
args=parser.parse_args()
before=args.before.read_text();after=args.after.read_text()
def array(text,name):
    match=re.search(r"inline constexpr [^\n;]+\b"+name+r"(?:\[[^\]]*\])+=([^\n;]+);",text)
    assert match, name
    return json.loads(match[1].replace("{","[").replace("}","]"))
origins=array(after,"track_origins");scalars=array(after,"track_scalars")
samples=array(after,"track_samples");decoded=[]
for p,o,r,f,offset,length,cut in samples:
    decoded.append([[origins[o][i]+p[i] for i in range(3)],[scalars[v] for v in r],scalars[f],offset,length,cut])
assert decoded==array(before,"track_samples"), "Camera or XOR descriptor changed"
assert array(before,"track_changes")==array(after,"track_changes"), "Visibility delta changed"
axes=array(after,"rail_axes");settings=array(after,"rail_settings")
parameters=[[axes[a],settings[s][:2],settings[s][2]] for a,s in array(after,"rail_parameters")]
assert parameters==array(before,"rail_parameters"), "Navigation parameter changed"
print(json.dumps({"samples":len(samples),"parameters":len(parameters),"camera_origins":len(origins),
                  "camera_scalars":len(scalars),"rail_axes":len(axes),"rail_settings":len(settings),
                  "result":"exact integer equality; visibility bytes unchanged"},indent=2))
