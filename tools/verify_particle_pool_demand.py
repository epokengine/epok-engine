"""Console proof of optional particle storage and an ordinary actor emitter.

All projects are persistent fixtures under execution-reorg/particles/console.
Requires exclusive ownership of the shared MIPS/emulator/export queue.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import uuid

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools"))
sys.path.insert(0,str(ROOT/"tests/integration"))
import epok_documents as documents
from gameplay_probe import run_modes
from gameplay_helpers_baseline import linked_budget

EDITOR=ROOT/"target/debug/epok-editor.exe"
OUT=ROOT/"artifacts/gameplay-helpers/p8/execution-reorg/particles/console"
NAMESPACE=uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity=lambda name:str(uuid.uuid5(NAMESPACE,"particle-demand/"+name))
digest=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()


def write(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(value if isinstance(value,str) else json.dumps(value,indent=2)+"\n",encoding="utf-8")


def source():
    return r'''#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t particle_demand_probe[8]={};inline uint32_t particle_demand_trace[150][8]={};}
class EPOK_CLASS(Blueprintable,Id="CLASS") ParticleDemandProbe:public epok::Actor3D {
 uint32_t frame_=0,errors_=0,checkpoint_=0;epok::ObjectId original_{};epok::DataHandle stale_{},replacement_{};
 void check(bool okay,uint32_t bit){if(!okay)errors_|=uint32_t(1)<<bit;}
 static bool same(const char* a,const char* b){if(!a||!b)return false;while(*a&&*a==*b){++a;++b;}return *a==*b;}
 epok::Actor* named(const char* name){
  auto* registry=epok::active_object_registry;auto* level=registry?registry->resolve<epok::Level>(level_id()):nullptr;
  if(level)for(size_t i=0;i<level->actor_count();++i){auto* actor=registry->resolve<epok::Actor>(level->actor_at(i));if(actor&&same(actor->name(),name))return actor;}
  return nullptr;
 }
 void configure(epok::ActorData& data){
  auto& e=data.particle_emitter;e.enabled=true;e.continuous=true;e.rate=60.0;e.lifetime=0.25;e.max_particles=16;
  e.start_size=0.2;e.end_size=0.1;e.sprite.size[0]=0.2;e.sprite.size[1]=0.2;
  for(unsigned n=0;n<3;++n){e.velocity[n]=0.0;e.spread[n]=0.0;e.gravity[n]=0.0;}e.play();
 }
public:
 void on_frame(uint32_t)override{
  if(frame_>=150)return;++frame_;auto* actor=named("Particle owner");
  auto* data=actor?actor->data():nullptr;auto& stats=epok::particle_stats;
#ifdef EPOK_NO_PARTICLES
  check(false,30);
#endif
  if(frame_==1){check(data!=nullptr,0);if(data){original_=actor->id();stale_=epok::handle(data);configure(*data);}}
  if(frame_==20){check(stats.alive>0&&stats.spawned>0,1);checkpoint_=stats.spawned;if(data)data->particle_emitter.stop();}
  if(frame_==40){check(stats.alive==0&&stats.spawned==checkpoint_,2);if(data)data->particle_emitter.play();}
  if(frame_==60){check(stats.alive>0&&stats.spawned>checkpoint_,3);checkpoint_=stats.spawned;if(actor)actor->set_active(false);}
  if(frame_==80){check(stats.alive==0&&stats.spawned==checkpoint_,4);if(actor)actor->set_active(true);}
  if(frame_==100){check(stats.alive>0&&stats.spawned>checkpoint_,5);checkpoint_=stats.spawned;if(data)data->particle_emitter.enabled=false;}
  if(frame_==105){check(stats.alive==0&&stats.spawned==checkpoint_,6);if(data){auto& e=data->particle_emitter;e.enabled=true;e.continuous=false;e.stop();e.burst(8);}}
  if(frame_==110){check(stats.alive==8&&stats.spawned==checkpoint_+8,7);checkpoint_=stats.spawned;if(actor)actor->destroy();}
  if(frame_==112){
   auto* registry=epok::active_object_registry;check(!stale_.get()&&!registry->resolve<epok::Actor>(original_)&&stats.alive==0,8);
   auto* level=registry->resolve<epok::Level>(level_id());const auto* type=epok::find_object_class(epok::Actor3D::static_class_id);
   check(level&&type,9);if(level&&type)level->spawn_actor(*type,"Replacement emitter");
  }
  if(frame_==114){auto* next=named("Replacement emitter");check(next&&next->data(),10);if(next&&next->data()){
   replacement_=epok::handle(next->data());configure(*next->data());auto& e=next->data()->particle_emitter;e.continuous=false;e.stop();e.burst(8);
   check(!stale_.get()&&(replacement_.index!=stale_.index||replacement_.generation!=stale_.generation),11);
  }}
  if(frame_==120)check(stats.alive==8&&stats.spawned==checkpoint_+8,12);
  if(frame_==145)check(stats.alive==0&&stats.spawned==checkpoint_+8,13);
  const uint32_t row[]={frame_,stats.alive,stats.spawned,stats.dropped,stats.peak,stats.dropped_emitters,uint32_t(stale_.index),uint32_t(replacement_.index)};
  for(unsigned n=0;n<8;++n)particle_demand_trace[frame_-1][n]=row[n];
  if(frame_==150){check(!stats.dropped&&!stats.dropped_emitters&&!errors_,14);auto* p=particle_demand_probe;
   p[1]=frame_;p[2]=errors_;p[3]=stats.spawned;p[4]=stats.peak;p[5]=replacement_.get()&&!stale_.get();p[6]=replacement_.index==stale_.index;p[7]=stats.dropped+stats.dropped_emitters;p[0]=0x50415254u;
  }
 }
};
'''.replace('"CLASS"','"'+identity("class")+'"')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case",choices=("basic","complete","probe","control"),action="append")
    args=parser.parse_args()
    OUT.mkdir(parents=True,exist_ok=True)
    editor_hash=digest(EDITOR)
    snapshot={str(path.relative_to(ROOT)):digest(path) for folder in ("runtime","src","native","resources/templates/dungeon-complete","resources/templates/dungeon-complete-flavors")
              for path in (ROOT/folder).rglob("*") if path.is_file() and path.suffix in (".cpp",".hpp",".h",".rs",".lua",".epokbp",".epokmap")}
    previous=OUT/"source-snapshot.json"
    if previous.exists():assert json.loads(previous.read_text())==snapshot,"Integration source changed"
    write(previous,snapshot);write(OUT/"builder.json",{"editor_sha256":editor_hash})
    results=json.loads((OUT/"validation.json").read_text()) if (OUT/"validation.json").exists() else {"editor_sha256":editor_hash,"checks":{}}
    assert results["editor_sha256"]==editor_hash
    def run(label,*arguments):
        assert digest(EDITOR)==editor_hash
        start=time.monotonic();process=subprocess.run([str(EDITOR),*map(str,arguments)],cwd=ROOT,capture_output=True,text=True,encoding="utf-8",errors="replace")
        write(OUT/(label+".log"),process.stdout+process.stderr)
        assert process.returncode==0,(label,process.stdout[-4000:]+process.stderr[-4000:])
        assert digest(EDITOR)==editor_hash
        print(label,"PASS",round(time.monotonic()-start,2),flush=True)
        return process.stdout.strip()
    def capture(project,label,omitted):
        build=project/".epok/build";flags=(build/"sources.mk").read_text();assert ("-DEPOK_NO_PARTICLES" in flags)==omitted
        symbols=subprocess.check_output([str(ROOT/".tools/mips/bin/mipsel-none-elf-nm.exe"),"-S","-C","--defined-only",str(build/"epok.elf")],text=True)
        pool=re.findall(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+[bB]\s+\(anonymous namespace\)::particles$",symbols,re.M)
        assert bool(pool)!=omitted,(label,pool)
        for name in ("memory-report.json","sources.mk","scene.hh","epok.map"):
            write(OUT/(label+"-"+name),(build/name).read_text())
        write(OUT/(label+"-symbols.txt"),symbols)
        return {**linked_budget(project),"no_particles_flag":omitted,"pool_bytes":int(pool[0][1],16) if pool else 0,
                "staged_main_sha256":digest(build/"main.cpp"),"current_main_sha256":digest(ROOT/"runtime/main.cpp")}
    selected=args.case or ("basic","complete","probe")
    for case in selected:
        project=OUT/("complete" if case=="control" else case)/"project"
        if not project.exists():run(case+"-create","--create-project",project,"--template","dungeon-complete" if case=="complete" else "basic","--gameplay","cpp")
        if case=="basic":
            run("basic-build","--project",project,"--build-psx");budget=capture(project,"basic",True)
            budget.update(baseline_ram_bytes=983160,delta_static_ram_bytes=budget["static_ram_bytes"]-983160)
            results["checks"][case]=budget
            write(OUT/"validation.json",results)
            assert budget["delta_static_ram_bytes"]<=0,budget
        elif case=="complete":
            manifest=documents.loads(next(project.glob("*.epokproject")).read_text());scene=documents.loads((project/manifest["startup_scene"]).read_text())
            assert len(scene["actors"])==58 and sum(len(actor["components"]) for actor in scene["actors"])==156
            baseline=ROOT/"artifacts/gameplay-helpers/p8/player-vitals/complete/project"
            assets={str(path.relative_to(project)):digest(path) for path in (project/"assets").rglob("*.epokasset")}
            assert len(assets)==67 and all(digest(baseline/path)==value for path,value in assets.items())
            run("complete-embedded-build","--project",project,"--build-psx");embedded=capture(project,"complete-embedded",True)
            embedded.update(baseline_ram_bytes=2016808,saving_bytes=2016808-embedded["static_ram_bytes"])
            geometry=digest(project/".epok/build/GEOMETRY.BIN");assert geometry==digest(baseline/".epok/build/GEOMETRY.BIN")
            run("complete-pcdrv-package","--project",project,"--export-ps-exe","--debug-symbols");pcdrv=capture(project,"complete-pcdrv",True)
            pcdrv.update(baseline_ram_bytes=2017000,saving_bytes=2017000-pcdrv["static_ram_bytes"])
            result={"actors":58,"components":156,"assets_equal":67,"geometry_equal":True,"geometry_sha256":geometry,
                    "embedded":embedded,"pcdrv":pcdrv,"package":str(next((project/"exports").glob("*-ps-exe.zip")).relative_to(ROOT))}
            results["checks"][case]=result;write(OUT/"validation.json",results)
            assert embedded["saving_bytes"]>=32768 and pcdrv["saving_bytes"]>=32768,result
            assert min(embedded["remaining_after_runtime_headroom_bytes"],pcdrv["remaining_after_runtime_headroom_bytes"])>=0
        elif case=="probe":
            manifest=documents.loads(next(project.glob("*.epokproject")).read_text());scene_path=project/manifest["startup_scene"];scene=documents.loads(scene_path.read_text())
            camera=next(actor for actor in scene["actors"] if actor["name"]=="Main Camera")
            driver=next(actor for actor in scene["actors"] if actor["name"]=="GameRoot")
            driver["class"]={"name":"ParticleDemandProbe","class_id":identity("class")}
            owner=copy.deepcopy(camera);owner.update(id=identity("owner"),name="Particle owner",components=[copy.deepcopy(next(c for c in camera["components"] if c.get("root")))])
            owner["components"][0]["id"]=identity("owner-root");owner["components"][0]["properties"]["position"]=[0,1,4]
            scene["actors"]=[camera,driver,owner];write(scene_path,scene);write(project/"assets/scripts/ParticleDemandProbe.hpp",source())
            def check(values,expected):
                assert values[:3]==[expected,150,0] and values[3]>32 and values[4]>0 and values[5]==1 and values[7]==0,values
            run_modes(OUT/"probe",project,"ordinary actor emitter",("native_cpp",),"particle_demand_probe",0x50415254,check,
                      provider_count=1,trace=("particle_demand_trace",150*8))
            budget=capture(project,"active-probe",False)
            rows=json.loads((OUT/"probe/native_cpp-particle_demand_trace.json").read_text())
            assert len(rows)==1200
            results["checks"][case]={"budget":budget,"playback":json.loads((OUT/"probe/result.json").read_text()),"trace_words":len(rows),"ordinary_actor":True}
            write(OUT/"validation.json",results)
        else:
            control=OUT/"control";control.mkdir(exist_ok=True)
            authored=project/"assets/scripts/DungeonCombat/DungeonGame.cpp"
            original=authored.read_bytes();original_hash=digest(authored)
            (control/"source-original.cpp").write_bytes(original)
            package=next((project/"exports").glob("*-ps-exe.zip"))
            shutil.copyfile(package,control/"normal-package.zip")
            normal=capture(project,"control-treatment-before",True)
            requirement=b"\n// Console-only controlled demand; no gameplay operation or data.\nusing EpokConsoleParticleDemand = epok::ParticleEmitter;\n"
            try:
                authored.write_bytes(original+requirement)
                process=subprocess.run([str(EDITOR),"--project",str(project),"--export-ps-exe","--debug-symbols"],cwd=ROOT,capture_output=True,text=True,encoding="utf-8",errors="replace")
                write(control/"retained-build.log",process.stdout+process.stderr)
                assert process.returncode==0 or "at least 65536 bytes are required" in process.stdout+process.stderr,(process.returncode,process.stdout[-3000:]+process.stderr[-3000:])
                retained=capture(project,"control-retained",False)
                memory=json.loads((project/".epok/build/memory-report.json").read_text())
                assert memory["executable_hash"]==retained["executable_sha256"]
                report={"hypothesis":"Excluding unused particle storage and code saves at least32KiB in an otherwise identical current Complete scene",
                    "criterion_bytes":32768,"native_requirement":"using EpokConsoleParticleDemand = epok::ParticleEmitter;",
                    "requirement_contract":"src/script_backend.rs native_requirements; particle_pool_demand_preserves_native_dynamic_and_opaque_uses",
                    "control_cli_exit_code":process.returncode,"control":retained,"treatment":normal,
                    "saving_bytes":retained["static_ram_bytes"]-normal["static_ram_bytes"],
                    "coherent_net_target_vs_vitality_passed":False,"no_emulator_launched_for_control":True}
                write(control/"comparison.json",report)
            finally:
                authored.write_bytes(original)
            assert digest(authored)==original_hash
            run("control-normal-restore","--project",project,"--export-ps-exe","--debug-symbols")
            restored=capture(project,"control-treatment-restored",True)
            assert restored["executable_sha256"]==normal["executable_sha256"]
            shutil.copyfile(control/"normal-package.zip",package)
            report.update(restored=restored,source_restored_sha256=original_hash,normal_package_restored_sha256=digest(package),
                          controlled_32k_target_passed=report["saving_bytes"]>=32768)
            results["checks"][case]=report;write(control/"comparison.json",report);write(OUT/"validation.json",results)
            assert report["controlled_32k_target_passed"],report
    assert all(digest(ROOT/path)==value for path,value in snapshot.items()),"Product sources changed"
    net=results["checks"].get("complete",{})
    net_pass=bool(net) and min(net["embedded"]["saving_bytes"],net["pcdrv"]["saving_bytes"])>=32768
    results.update(status="Particle demand behavior validated; coherent net32KiB target remains failed" if net and not net_pass else "Selected particle demand checks completed; remaining checks are separate",
                   coherent_net_32k_target_passed=net_pass,source_snapshot="source-snapshot.json")
    write(OUT/"validation.json",results)


if __name__=="__main__":main()
