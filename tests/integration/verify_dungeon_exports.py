"""Independently rebuild the three generated Dungeon foundation projects."""
import argparse
from pathlib import Path
import subprocess
from verify_camera_rigs import ROOT,EDITOR
from gameplay_probe import export_probe


def verify(out):
    for flavor in ("cpp","blueprint","lua"):
        directory=out/flavor
        project=directory/"project"
        def run(*args,label):
            result=subprocess.run([str(EDITOR),*map(str,args)],cwd=ROOT,capture_output=True,text=True,encoding="utf-8",timeout=900)
            (directory/label).write_text(result.stdout+result.stderr,encoding="utf-8")
            assert result.returncode==0,result.stdout+result.stderr
            return result.stdout.strip()
        export_probe(directory,project,run,("world_gameplay.hpp","object_storage.hpp","object_model.hpp","character3d.hpp","camera_rig.hpp"))


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    verify(parser.parse_args().out)
