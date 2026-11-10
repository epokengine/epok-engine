"""Compare Dungeon's complete authored world/resources with the P7B snapshot."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
from verify_camera_rigs import ROOT,EDITOR,documents


def snapshot(project):
    manifest=documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene=documents.loads((project/manifest["startup_scene"]).read_text(encoding="utf-8"))
    aliases={}
    packages={}
    resources={}
    for path in sorted((project/"assets").rglob("*")):
        key=path.relative_to(project).as_posix()
        if path.is_file() and path.suffix==".epokasset":
            blob=path.read_bytes();assert blob[:8]==b"EPOKAS01"
            size=struct.unpack_from("<I",blob,8)[0]
            meta=json.loads(blob[16:16+size]);payload=blob[16+size:]
            aliases[meta["id"]]=key
            try: value=json.loads(payload)
            except (ValueError,UnicodeDecodeError): value=None
            packages[key]=(meta,payload,value)
        elif path.is_file() and path.suffix==".png":
            resources[key]=hashlib.sha256(path.read_bytes()).hexdigest()
    for key,(meta,_,value) in packages.items():
        if meta["kind"]=="EditableMesh":
            for collection in ("faces","groups","materials"):
                for i,item in enumerate(value[collection]):
                    aliases[item["id"]]=f"{key}:{collection}:{i}"
        if meta["kind"]=="DungeonRoom":
            for i,actor in enumerate(value["actors"]):
                aliases[actor["id"]]=f"{key}:actor:{i}"
                for j,component in enumerate(actor["components"]): aliases[component["id"]]=f"{key}:component:{i}:{j}"
    flag_owners={c["id"]:a["id"] for a in scene["actors"] for c in a["components"] if c["class"].get("class_id")=="397c4a8e-a044-59b5-ab51-a54d214f1789"}
    for i,actor in enumerate(scene["actors"]):
        aliases[actor["id"]]=f"actor:{i}"
        actor["components"]=[c for c in actor["components"] if c["id"] not in flag_owners]
        for j,component in enumerate(actor["components"]):
            aliases[component["id"]]=f"component:{i}:{j}"
            props=component.get("properties",{})
            overrides=component.get("overrides",[])
            if component["class"]["name"]=="DungeonKey" and "collected_flag" in props:
                assert flag_owners[props["collected_flag"]]==actor["id"]
                props["player"]=props.pop("accepted_initiator")
                props.pop("auto_collect");props.pop("collected_flag")
                props["lock"]=props.pop("lock_id")
                overrides=[{"accepted_initiator":"player","lock_id":"lock"}.get(k,k) for k in overrides if k not in ("auto_collect","collected_flag")]
            elif component["class"]["name"]=="DungeonDoor" and "requirement_flag" in props:
                props["key"]=flag_owners[props.pop("requirement_flag")]
                assert props.pop("initial_state")==2
                collider=props.pop("collider_3d")
                assert any(c["id"]==collider for c in actor["components"])
                props["lock"]=props.pop("lock_id")
                overrides=[{"requirement_flag":"key","lock_id":"lock"}.get(k,k) for k in overrides if k not in ("initial_state","collider_3d")]
            if "overrides" in component: component["overrides"]=sorted(overrides)
    for i,generator in enumerate(scene["dungeons"]):
        aliases[generator["id"]]=f"generator:{i}"
        for key in ("fingerprint","style","recipe"): generator["output"].pop(key,None)
    scene.pop("bake",None);scene.pop("navigation",None)
    def visit(value):
        if isinstance(value,str): return aliases.get(value,value)
        if isinstance(value,list): return [visit(item) for item in value]
        if isinstance(value,dict): return {aliases.get(key,key):visit(item) for key,item in value.items()}
        return value
    for key,(meta,payload,value) in packages.items():
        if value is None:
            content=hashlib.sha256(payload).hexdigest()
        else:
            content=visit(value)
            if meta["source_hash"]==hashlib.sha256(payload).hexdigest():
                meta["source_hash"]=hashlib.sha256(json.dumps(content,sort_keys=True).encode()).hexdigest()
        resources[key]={"meta":visit(meta),"payload":content}
    return {"scene":visit(scene),"resources":resources}


def difference(before,after,path=""):
    if type(before)!=type(after): return path+": types differ"
    if isinstance(before,dict):
        if before.keys()!=after.keys(): return path+": keys differ "+repr(before.keys()^after.keys())
        for key in before:
            result=difference(before[key],after[key],path+"/"+key)
            if result:return result
    elif isinstance(before,list):
        if len(before)!=len(after):return path+": lengths differ"
        for i,(a,b) in enumerate(zip(before,after)):
            result=difference(a,b,path+f"/{i}")
            if result:return result
    elif before!=after:return path+": "+repr(before)[:120]+" != "+repr(after)[:120]
    return None


def verify(out,before):
    out.mkdir(parents=True,exist_ok=True)
    rows=[]
    for flavor in ("cpp","blueprint","lua"):
        project=out/flavor
        if not project.exists():
            result=subprocess.run([str(EDITOR),"--create-project",str(project),"--template","dungeon","--gameplay",flavor],cwd=ROOT,capture_output=True,text=True,encoding="utf-8",timeout=900)
            (out/(flavor+"-create.log")).write_text(result.stdout+result.stderr,encoding="utf-8")
            assert result.returncode==0,result.stdout+result.stderr
        old=snapshot(before/("dungeon-"+flavor));actual=snapshot(project)
        for label,data in (("before",old),("after",actual)):
            (out/(flavor+"-"+label+".json")).write_text(json.dumps(data,indent=2)+"\n",encoding="utf-8")
        assert old==actual,difference(old,actual)
        rows.append({"flavor":flavor,"actors":len(actual["scene"]["actors"]),"resources":len(actual["resources"]),"world_and_resources_match":True})
        print("PASS",flavor,"original Dungeon world/resources",flush=True)
    (out/"result.json").write_text(json.dumps({"editor_sha256":hashlib.sha256(EDITOR.read_bytes()).hexdigest(),"cases":rows,"normalization":"Generated identities, ownership/rule properties and explicit flag component; derived bake/navigation caches and generator digests excluded."},indent=2)+"\n",encoding="utf-8")


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/dungeon-shared/composition")
    parser.add_argument("--before",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/player-routes/composition")
    args=parser.parse_args();verify(args.out,args.before)
