"""Compile the current runtime's camera math for an offline fixed-point audit.

Extracts production rotation_terms/inverse_local, FOV/row scale and ordered GTE
matrix setup verbatim. Only root cameras are supported by this study probe.
Uses production PsyQo Fixed/Trig and GTE register-upload code, with register I/O
stubbed. This is not a PSX rasterizer, timing measurement or hierarchical test.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import yaml
from redux_host import default_engine


def section(source, begin, end):
    if source.count(begin) != 1:
        raise ValueError(f"Non-unique production boundary: {begin}")
    start=source.index(begin)
    return source[start:source.index(end,start)]


def q12(value):
    f32=lambda x:struct.unpack("<f",struct.pack("<f",x))[0]
    scaled=f32(f32(value)*4096)
    return int(math.copysign(math.floor(abs(scaled)+.5),scaled))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine",type=Path,default=default_engine())
    parser.add_argument("--scene",type=Path,required=True)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    output=args.output.resolve()
    if output.exists() and any(output.iterdir()):parser.error("Output must be new or empty")
    output.mkdir(parents=True,exist_ok=True)
    engine=args.engine.resolve()
    inputs={p:p.read_bytes() for p in [args.scene,engine/"runtime/main.cpp",engine/"runtime/ordered_renderer.hpp",
        engine/"runtime/affine.hpp",engine/"runtime/gte_geometry.hpp",
        engine/"third_party/nugget/psyqo/fixed-point.hh",engine/"third_party/nugget/psyqo/trigonometry.hh",
        engine/"third_party/nugget/psyqo/src/trigonometry.cpp",engine/"runtime/fixed_trigonometry.hpp"]}
    scene=yaml.load(inputs[args.scene],Loader=yaml.CSafeLoader)
    track=scene["visibility_track"]
    actor=next(a for a in scene["actors"] if a["id"]==track["camera"])
    assert not actor.get("parent") and not actor.get("attach"),"Root camera required"
    transform=next(c["properties"] for c in actor["components"] if "position" in c.get("properties",{}))
    settings=next(c["properties"] for c in actor["components"] if "camera_fov" in c.get("properties",{}))
    source=inputs[engine/"runtime/main.cpp"].decode()
    ordered=inputs[engine/"runtime/ordered_renderer.hpp"].decode()
    rotation=section(source,"bool rotation_terms(","// Mesh vertices are projected")
    inverse=section(source,"Matrix inverse_local(","epok::ActorData* camera_entity()")
    focal=section(source,"  int32_t degrees=camera->camera_settings.field_of_view.raw();","  auto view=Matrix::identity();")
    scale=section(source,"  constexpr int32_t projection_focal_pixels =", "  PolygonEmitter emitter")
    setup=section(ordered,"  Matrix matrix=view.basis;","  bool anchored=false;")
    (output/"EASTL").mkdir();(output/"gte-matrix/psyqo").mkdir(parents=True)
    (output/"EASTL/functional.h").write_text("#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n")
    (output/"EASTL/array.h").write_text("#pragma once\n#include <array>\nnamespace eastl {using std::array;}\n")
    (output/"gte-matrix/gte_geometry.hpp").write_bytes(inputs[engine/"runtime/gte_geometry.hpp"])
    (output/"gte-matrix/psyqo/gte-registers.hh").write_bytes((engine/"tests/runtime/gte_register_stub.hpp").read_bytes())
    (output/"gte-matrix/psyqo/gte-kernels.hh").write_text("#pragma once\n")
    prefix='''#include <cstdio>
#include <cstdint>
#include "psyqo/trigonometry.hh"
#include "fixed_trigonometry.hpp"
#include "gte-matrix/gte_geometry.hpp"
using Fixed=psyqo::FixedPoint<12>;
using Matrix=epok::Affine<Fixed>;
psyqo::Trig<> trig;
namespace epok {
struct Transform {Fixed position[3],rotation[3],scale[3];};
inline Fixed projection_focal,projection_focal_x;
inline constexpr int display_width=512,display_height=240;
}
struct Camera {struct {Fixed field_of_view;bool mirror_x;} camera_settings;};
'''
    body='''
int main(){
 int index,raw[11];
 while(std::scanf("%d",&index)==1){
  for(int& v:raw)if(std::scanf("%d",&v)!=1)return 2;
  epok::Transform transform{};Camera data{},*camera=&data;
  for(int c=0;c<3;++c){transform.position[c]=Fixed(raw[c],Fixed::RAW);
    transform.rotation[c]=Fixed(raw[3+c],Fixed::RAW);transform.scale[c]=Fixed(raw[6+c],Fixed::RAW);}
  camera->camera_settings.field_of_view=Fixed(raw[9],Fixed::RAW);
  camera->camera_settings.mirror_x=raw[10]!=0;
  epok::CameraSpace<Fixed> view;view.basis=inverse_local(transform,false);
  for(int c=0;c<3;++c)view.origin[c]=transform.position[c];
'''
    tail='''
  if(!matrix_valid)return 3;
  std::printf("[%d",index);
  for(int c=0;c<3;++c)std::printf(",%d",view.origin[c].raw());
  for(int r=0;r<3;++r)for(int c=0;c<4;++c)std::printf(",%d",view.basis.values[r][c].raw());
  for(int r=0;r<2;++r)std::printf(",%d",row_scale[r].raw());
  using namespace psyqo::GTE;
  const uint32_t values[]={readRaw<Register::R11R12>(),readRaw<Register::R13R21>(),
   readRaw<Register::R22R23>(),readRaw<Register::R31R32>(),readRaw<Register::R33>(),
   readRaw<Register::TRX>(),readRaw<Register::TRY>(),readRaw<Register::TRZ>(),
   readRaw<Register::OFX>(),readRaw<Register::OFY>(),readRaw<Register::H>()};
  for(auto value:values)std::printf(",%u",value);
  std::puts("]");
 }
}
'''
    cpp=output/"probe.cpp";cpp.write_text(prefix+rotation+inverse+body+focal+scale+setup+tail)
    compiler=Path("C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/Llvm/x64/bin/clang++.exe")
    command=[str(compiler),"-std=c++20","-O2",str(cpp),str(engine/"third_party/nugget/psyqo/src/trigonometry.cpp"),
             "-o",str(output/"probe.exe"),"-I"+str(output),"-I"+str(engine/"runtime"),"-I"+str(engine/"third_party/nugget")]
    subprocess.run(command,check=True,capture_output=True,text=True)
    lines=[]
    for index,sample in enumerate(track["samples"]):
        values=[*sample["position"],*sample["rotation"],*transform["scale"],sample["fov"]]
        lines.append(" ".join(map(str,[index,*map(q12,values),int(settings.get("camera_mirror_x",False))])))
    result=subprocess.run([str(output/"probe.exe")],input="\n".join(lines),capture_output=True,text=True,check=True)
    rows=[json.loads(line) for line in result.stdout.splitlines()]
    if len(rows)!=len(track["samples"]) or any(len(row)!=29 or row[0]!=i for i,row in enumerate(rows)):
        raise ValueError("Incomplete camera probe")
    report=dict(scope=__doc__,rows=rows,compile_command=command,
                input_sha256={str(p):hashlib.sha256(v).hexdigest() for p,v in inputs.items()},
                generated_probe_sha256=hashlib.sha256(cpp.read_bytes()).hexdigest(),
                layout="index,origin[3],basis[12],row_scale[2],GTE[R0..R4,TR0..TR2,OFX,OFY,H]")
    (output/"cameras.json").write_text(json.dumps(report,indent=2))
    print(json.dumps(dict(samples=len(rows),output=str(output/"cameras.json"))))


if __name__=="__main__":main()
