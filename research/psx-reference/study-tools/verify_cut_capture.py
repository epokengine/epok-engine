"""Check a cold, explicit diagnostic load cut in a native PSX capture."""
import argparse
import json
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument("profile",type=Path)
parser.add_argument("--source",type=int,required=True)
parser.add_argument("--destination",type=int,required=True)
args=parser.parse_args()
report=json.loads(args.profile.read_text());rows=sorted(report["frames"],key=lambda r:r["frame"])
assert "16384" in report["input_replay"], "Cross must activate the diagnostic portal"
assert rows[0]["rail_navigation"]["route"]==args.source
edges=[(a["rail_navigation"]["route"],b["rail_navigation"]["route"])
       for a,b in zip(rows,rows[1:]) if a["rail_navigation"]["route"]!=b["rail_navigation"]["route"]]
assert edges and edges[0]==(args.source,args.destination)
source_rows=[r for r in rows if r["rail_navigation"]["route"]==args.source and r["triangles"]>0]
assert len(source_rows)>=10, "No visible, settled source view before the cut"
warmup=report["streaming_warmup"]["during_capture"]
assert warmup["attempts"]==1 and warmup["pages"]>0 and warmup["reads"]>0
assert warmup["failures"]==warmup["rejected"]==0
assert report["streaming"]["last"]["errors"]==report["streaming"]["last"]["timeouts"]==0
for field in ("stream_failed_chunks","stream_budget_skipped_chunks","dropped_triangles","dropped_steps"):
    assert max(r[field] for r in rows)==0, field
assert rows[-1]["visibility_track"]["loops"]==0
print(json.dumps({"source":args.source,"destination":args.destination,"observed_edges":edges,
    "destination_pages":warmup["pages"],"cold_reads":warmup["reads"],"load_wait_seconds":warmup["stall_us"]/1e6,
    "result":"PASS: source rendering, gated cut, complete destination warmup, no transport/budget failures"},indent=2))
