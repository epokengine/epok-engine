"""Offline experiment, not the production packer. No game data is modified."""
import argparse
import json
from pathlib import Path
import subprocess
import time

parser=argparse.ArgumentParser()
parser.add_argument("audit",type=Path)
parser.add_argument("--solver",type=Path,required=True)
parser.add_argument("--target",type=int,default=7)
parser.add_argument("--output",type=Path,required=True)
args=parser.parse_args()
if args.output.exists():parser.error("Use a new output path")
audit=json.loads(args.audit.read_text())
jobs=audit["jobs"]
lines=[f'{len(jobs)} {audit["page_bytes"]} {args.target}']
lines.extend(f'{j["bytes"]} {j["page"]} {len(j["views"])} '+" ".join(map(str,j["views"])) for j in jobs)
started=time.monotonic()
result=subprocess.run([str(args.solver.resolve())],input="\n".join(lines),text=True,capture_output=True,check=True)
print(result.stderr.strip())
pages=list(map(int,result.stdout.split()))
assert len(pages)==len(jobs)
loads={};views=[set() for _ in range(audit["view_count"])]
for job,page in zip(jobs,pages):
    assert page>=0
    loads[page]=loads.get(page,0)+job["bytes"]
    for view in job["views"]:views[view].add(page)
assert max(loads.values())<=audit["page_bytes"]
peak=max(map(len,views))
report={"peak":peak,"archive_pages":len(loads),"seconds":time.monotonic()-started,
        "worst_views":[i for i,v in enumerate(views) if len(v)==peak],
        "pages":dict(zip((j["key"] for j in jobs),pages))}
args.output.write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k not in ("pages","worst_views")}))
