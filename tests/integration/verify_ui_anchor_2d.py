"""Exercise the actual 2D camera/ScreenAnchor service in the console runtime."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
EDITOR = ROOT / "target/debug/epok-editor.exe"
NS = uuid.UUID("bdb514d9-bc71-5c05-b199-63270681f240")
identity = lambda name: str(uuid.uuid5(NS, name))

def run(out, name, *args):
    result = subprocess.run([str(EDITOR), *map(str,args)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
    (out / name).write_text(result.stdout+result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stdout+result.stderr
    return result.stdout

def component(name, cls, props=None, root=False):
    props = props or {}
    return {"id":identity(name), "name":name, "class":{"name":cls["cpp_name"],"class_id":cls["id"]}, "properties":props, "overrides":list(props), "root":root}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--width",type=int,choices=(320,640),default=320)
    args=parser.parse_args()
    out=args.output.resolve();out.mkdir(parents=True,exist_ok=True);project=out/"project"
    if not project.exists(): run(out,"create.log","--create-project",project,"--template","basic","--name","2D Anchor")
    source=ROOT/"tests/fixtures/native-scenes/Anchor2DProbe.hpp"
    (project/"assets/scripts/Anchor2DProbe.hpp").write_bytes(source.read_bytes())
    metadata=json.loads(run(out,"reflect.log","--project",project,"--reflect"))
    classes={c["cpp_name"]:c for c in metadata["classes"]}
    settings_path=next(project.glob("*.epokproject"));settings=documents.loads(settings_path.read_text(encoding="utf-8"))
    settings["rendering"].update(width=args.width,height=args.width*3//4)
    documents.write_text(settings_path,documents.dumps(settings))
    scene_path=project/settings["startup_scene"];scene=documents.loads(scene_path.read_text(encoding="utf-8"))
    def actor(name,cls,root_cls,props=None):
        props=props or {}
        return {"id":identity(name),"name":name,"class":{"name":cls,"class_id":classes[cls]["id"]},"properties":props,"overrides":list(props),"active":True,"components":[component(name+".root",classes[root_cls],root=True)]}
    camera=actor("camera2d","epok::Actor2D","epok::SceneComponent2D")
    camera["components"].append(component("camera2d.camera",classes["epok::Camera2DComponent"]))
    canvas=actor("canvas","epok::UIActor","epok::RectTransformComponent")
    canvas["components"].append(component("canvas.canvas",classes["epok::CanvasComponent"]))
    content=actor("content","epok::UIActor","epok::RectTransformComponent")
    content.update(logical_parent=canvas["id"],attach={"actor":canvas["id"],"component":identity("canvas.root")})
    content["components"][0].update(properties={"rect":{"anchor_min":[0,0],"anchor_max":[0,0],"pivot":[.5,0],"position":[0,0],"size":[100,40],"rotation":0}},overrides=["rect"])
    anchor=component("anchor",classes["epok::ScreenAnchorComponent"],{"content":identity("content.root"),"pixel_y":0})
    canvas["components"].append(anchor)
    target=actor("target","Anchor2DProbe","epok::SceneComponent2D",{"anchor":anchor["id"],"camera":identity("camera2d.camera")})
    scene["actors"]=scene["actors"][:1]+[camera,canvas,content,target]
    documents.write_text(scene_path,documents.dumps(scene))
    run(out,"build.log","--project",project,"--analyze-memory")
    with socket.socket() as port:
        if os.name=="nt":port.setsockopt(socket.SOL_SOCKET,socket.SO_EXCLUSIVEADDRUSE,1)
        port.bind(("127.0.0.1",8077))
    mapping=(project/".epok/build/epok.map").read_text()
    address=int(re.findall(r"0x([0-9a-fA-F]+)\s+screen_anchor_2d_probe\s*$",mapping,re.M)[-1],16)&0x1fffff
    values=None
    with (out/"play.log").open("w",encoding="utf-8") as log:
        process=subprocess.Popen([str(EDITOR),"--project",str(project),"--play-psx","--stop-after","6"],cwd=ROOT,stdout=log,stderr=log,creationflags=getattr(subprocess,"CREATE_NO_WINDOW",0))
        try:
            deadline=time.monotonic()+120
            while process.poll() is None and time.monotonic()<deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw",timeout=2) as response: values=list(struct.unpack_from("<2I",response.read(),address))
                    if values==[10,10]:break
                except (OSError,ValueError):pass
                time.sleep(.1)
            assert values==[10,10],values
            assert process.wait(timeout=30)==0
        finally:
            if process.poll() is None:
                if os.name=="nt":subprocess.run(["taskkill.exe","/PID",str(process.pid),"/T","/F"],capture_output=True,check=False)
                else:process.terminate()
                process.wait(timeout=15)
    result={"display":[args.width,args.width*3//4],"checks":values,"editor_sha256":hashlib.sha256(EDITOR.read_bytes()).hexdigest(),"probe_sha256":hashlib.sha256(source.read_bytes()).hexdigest()}
    (out/"result.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print("PASS console 2D anchor",args.width,values,flush=True)

if __name__=="__main__":main()
