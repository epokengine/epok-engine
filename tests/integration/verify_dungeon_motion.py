"""Replay the pre-helper rules and compare every motion/camera/key trace word."""
import argparse
import json
from pathlib import Path
import shutil
import uuid
from verify_camera_rigs import ROOT,documents,write
from gameplay_probe import run_modes
from verify_dungeon_foundation import SOURCE,DRIVER,LISTENER


def verify(out,foundation,before):
    out.mkdir(parents=True,exist_ok=True)
    project=out/"project"
    source=before/"cpp/project"
    assert "public epok::ActorComponent" in (source/"assets/scripts/DungeonKey.hpp").read_text()
    if not project.exists():
        shutil.copytree(source,project,ignore=shutil.ignore_patterns(".epok","exports"))
    magic=0x6c000000|(uuid.uuid4().int&0xffffff)
    header=SOURCE.replace("DRIVER_ID",DRIVER).replace("LISTENER_ID",LISTENER).replace("MAGIC",str(magic))
    write(project/"assets/scripts/DungeonFoundationRoute.hpp",header)
    frozen=json.loads((before/"cpp/result.json").read_text())[0]["sample"][1:]
    def check(values,expected):
        assert values==[expected,*frozen],"legacy replay changed: "+repr(values)
    run_modes(out,project,"legacy_cpp",("native_cpp",),"dungeon_foundation_probe",magic,check,provider_count=1,trace=("dungeon_foundation_trace",1080))
    legacy=json.loads((out/"native_cpp-dungeon_foundation_trace.json").read_text())
    def fnv(words):
        value=2166136261
        for word in words:value=((value^word)*16777619)&0xffffffff
        return value
    assert fnv(legacy)==frozen[-1]
    rows=[]
    for flavor in ("cpp","blueprint","lua"):
        for record in json.loads((foundation/flavor/"result.json").read_text()):
            actual=json.loads((foundation/flavor/(record["mode"]+"-dungeon_foundation_trace.json")).read_text())
            assert len(actual)==len(legacy)==1080
            assert fnv(actual)==record["sample"][-1]
            assert all(a==b for i,(a,b) in enumerate(zip(legacy,actual)) if i%6!=4),"motion/camera/key words changed: "+flavor+"/"+record["mode"]
            changed=[frame for frame in range(180) if legacy[frame*6+4]!=actual[frame*6+4]]
            # Shared doors commit at their Input tick boundary. The old rule
            # hid the actor in Gameplay after the collecting physics step.
            assert len(changed)<=1 and all(legacy[f*6+4]==0 and actual[f*6+4]==1 for f in changed),changed
            rows.append({"flavor":flavor,"mode":record["mode"],"identical_motion_camera_key_words":900,"door_visibility_boundary_frames":changed})
    assert len(rows)==5
    (out/"comparison.json").write_text(json.dumps({"legacy_record_matches_p7b":True,"cases":rows,"scope":"180 frames per case; positions, camera blockage and key visibility are exact. At most one later door-visibility frame is the explicit native Input-boundary transition."},indent=2)+"\n",encoding="utf-8")
    print("PASS five exact 180-frame motion/camera/key comparisons and defined door transition",flush=True)


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/dungeon-shared/motion-reference")
    parser.add_argument("--foundation",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/dungeon-shared/foundation")
    parser.add_argument("--before",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/player-routes/dungeon")
    args=parser.parse_args();verify(args.out,args.foundation,args.before)
