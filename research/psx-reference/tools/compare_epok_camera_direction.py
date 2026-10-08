"""Measure Epok's independent direction calculation against a verified original trace.

Uses original world-coordinate deltas and per-update parameters. Each blended
update starts from the observed previous angle, so this isolates direction and
one-step blending, not rail selection, frame cadence or accumulated drift.
No original lookup-table bytes are copied into Epok or the C++ probe.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from check_epok_camera_aim import inputs_for_row, negative_controls
from replay_original_camera import delta, i32, verify
from redux_host import default_engine

PROBE = r'''#include <cstdio>
#include "camera_direction.hpp"
int main(){
 unsigned n;
 while(std::scanf("%u",&n)==1){
  int v[27];for(int& x:v)if(std::scanf("%d",&x)!=1)return 2;
  uint16_t base[3],previous[3],look[3],result[3];
  int32_t camera[3],target[3];epok::CameraAimParameters p;
  for(unsigned i=0;i<3;++i){
   base[i]=uint16_t(v[i]);previous[i]=uint16_t(v[6+i]);
   p.base_weight[i]=uint16_t(v[9+i]);camera[i]=v[21+i];target[i]=v[24+i];
  }
  p.target_bias[0]=int16_t(v[12]);p.smoothing_weight=128;p.smoothing_limit=45;
  for(unsigned i=0;i<3;++i){p.positive_limit[i]=uint16_t(v[13+i*2]);p.negative_limit[i]=uint16_t(v[14+i*2]);}
  if(v[20]!=0)return 3;
  epok::camera_target_direction(camera,target,false,true,base[1],look);
  epok::blend_camera_aim(base,look,v[19]?previous:nullptr,p,result);
  std::printf("[%u,%u,%u,%u,%u,%u,%u]\n",n,look[0],look[1],look[2],result[0],result[1],result[2]);
 }
}
'''


def compare(rows, actual):
    if len(rows) != len(actual) or any(len(a) != 7 or a[0] != i for i,a in enumerate(actual)):
        raise ValueError('Incomplete, malformed or reordered C++ output')
    errors=[]
    for i,(row,a) in enumerate(zip(rows,actual)):
        errors.append(dict(row=i,look=[delta(x,y) for x,y in zip(row['look'],a[1:4])],
                           angles=[delta(x,y) for x,y in zip(row['angles'],a[4:7])]))
    return dict(updates=len(rows),
                exact_direction_updates=sum(not any(e['look']) for e in errors),
                exact_blended_updates=sum(not any(e['angles']) for e in errors),
                max_abs_direction_turn12=[max(abs(e['look'][c]) for e in errors) for c in range(3)],
                max_abs_blended_turn12=[max(abs(e['angles'][c]) for e in errors) for c in range(3)],
                errors=errors)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('exe','before','after','profile','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--engine',type=Path,default=default_engine())
    args=parser.parse_args()
    if args.output.exists():parser.error('Output must be new')
    data={n:getattr(args,n).read_bytes() for n in ('exe','before','after','profile')}
    profile=json.loads(data['profile'])
    if not verify(data['exe'],data['before'],data['after'],profile)['exact']:
        raise ValueError('Independent original replay failed')
    rejected=negative_controls(data,profile)
    rows=profile['camera_trace']['rows']
    inputs=[inputs_for_row(r,r['look'])+[i32(v) for v in r['camera']+r['target']] for r in rows]
    args.output.mkdir(parents=True)
    hashes={}
    for name in ('camera_direction.hpp','camera_aim.hpp','fixed_trigonometry.hpp'):
        payload=(args.engine/'runtime'/name).read_bytes()
        (args.output/name).write_bytes(payload)
        hashes[name]=hashlib.sha256(payload).hexdigest()
    cpp=args.output/'probe.cpp';cpp.write_text(PROBE)
    exe=args.output/'probe.exe'
    compiler='C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/Llvm/x64/bin/clang++.exe'
    command=[compiler,'-std=c++20','-O2','-Wall','-Wextra','-Werror','-D_CRT_SECURE_NO_WARNINGS',str(cpp),'-o',str(exe)]
    result=subprocess.run(command,capture_output=True,text=True)
    if result.returncode:raise RuntimeError(result.stdout+result.stderr)
    payload='\n'.join(' '.join(map(str,[i,*v])) for i,v in enumerate(inputs))
    result=subprocess.run([str(exe)],input=payload,check=True,capture_output=True,text=True)
    report=compare(rows,[json.loads(line) for line in result.stdout.splitlines()])
    report.update(scope=__doc__,input_sha256={n:hashlib.sha256(v).hexdigest() for n,v in data.items()},
                  header_sha256=hashes,probe_sha256=hashlib.sha256(cpp.read_bytes()).hexdigest(),
                  executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),compile_command=command,
                  negative_controls_rejected=rejected)
    (args.output/'comparison.json').write_text(json.dumps(report,indent=2))
    print(json.dumps({k:v for k,v in report.items() if k not in ('errors','compile_command','scope','input_sha256','header_sha256')},indent=2))


if __name__=='__main__':main()
