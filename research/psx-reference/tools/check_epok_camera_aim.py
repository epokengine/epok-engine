"""Compare Epok's integer camera-aim kernel with independently replayed source updates.

Scope: the normal, unbiased-by-effects follow mode, with all observed per-update
weights/limits. Does not integrate rail selection, direction calculation, update
cadence, PVS, rasterization or timing into Epok. Rejects unsupported source modes.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess

from replay_original_camera import Direction, i32, property_values, verify
from redux_host import default_engine

PROBE = r'''#include <cstdio>
#include <cstdint>
#include "camera_aim.hpp"
int main(){
 unsigned n;
 while(std::scanf("%u",&n)==1){
  int v[21];for(int& value:v)if(std::scanf("%d",&value)!=1)return 2;
  uint16_t base[3],target[3],previous[3],result[3];epok::CameraAimParameters p;
  for(unsigned i=0;i<3;++i){base[i]=uint16_t(v[i]);target[i]=uint16_t(v[3+i]);previous[i]=uint16_t(v[6+i]);p.base_weight[i]=uint16_t(v[9+i]);}
  p.target_bias[0]=int16_t(v[12]);p.smoothing_weight=128;p.smoothing_limit=45;
  p.positive_limit[0]=uint16_t(v[13]);p.negative_limit[0]=uint16_t(v[14]);
  p.positive_limit[1]=uint16_t(v[15]);p.negative_limit[1]=uint16_t(v[16]);
  p.positive_limit[2]=uint16_t(v[17]);p.negative_limit[2]=uint16_t(v[18]);
  if(v[20]!=0)return 3;
  epok::blend_camera_aim(base,target,v[19]?previous:nullptr,p,result);
  std::printf("[%u,%u,%u,%u]\n",n,result[0],result[1],result[2]);
 }
}
'''


def negative_controls(data, profile):
    rejected = []
    for field in ('look', 'angles', 'operations'):
        altered = copy.deepcopy(profile)
        target = altered['camera_trace']['rows'][0][field]
        if field == 'operations': target[0][6] ^= 1
        else: target[0] ^= 1
        if verify(data['exe'], data['before'], data['after'], altered)['exact']:
            raise ValueError('Verifier accepted corrupted '+field)
        rejected.append(field)
    for name, address in (('before', 0x5f6b8), ('after', 0x5c91c), ('exe', 0)):
        altered = dict(data)
        payload = bytearray(data[name]); payload[address] ^= 1
        altered[name] = bytes(payload)
        try: verify(altered['exe'], altered['before'], altered['after'], profile)
        except ValueError: rejected.append(name)
        else: raise ValueError('Verifier accepted corrupted '+name)
    return rejected


def inputs_for_row(row, look):
    flags = row['flags']
    if ((flags[0] >> 8) & 255 or flags[4] or flags[6]
            or property_values(row, 0x16e) is not None):
        raise ValueError('Unsupported source camera override/effect/transition')
    weights = property_values(row, 0x183) or [128]*3
    limits = property_values(row, 0x119) or [2048]*4
    if any(not 0 <= v <= 256 for v in weights) or any(not 0 <= v <= 2048 for v in limits):
        raise ValueError('Unsupported weights/limits')
    bias = i32(flags[3])
    if not -32768 <= bias <= 32767: raise ValueError('Pitch bias outside int16')
    smoothing = not ((flags[1] >> 8) & 255) and row['previous'][0] != 0x1001 and flags[2] != 2
    return [*row['base'], *[v & 4095 for v in look], *row['previous'], *weights, bias,
            limits[2], limits[3], limits[0], limits[1], 2048, 2048, int(smoothing), 0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('exe', 'before', 'after', 'profile', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--engine', type=Path, default=default_engine())
    args = parser.parse_args()
    if args.output.exists(): parser.error('Output must be new')
    data = {n: getattr(args, n).read_bytes() for n in ('exe', 'before', 'after', 'profile')}
    profile = json.loads(data['profile'])
    replay = verify(data['exe'], data['before'], data['after'], profile)
    if not replay['exact']: raise ValueError('Independent source replay failed')
    controls = negative_controls(data, profile)
    base = struct.unpack_from('<I', data['exe'], 0x18)[0]
    def table(address): return struct.unpack_from('<1025h', data['exe'], 0x800+address-base)
    direction = Direction(table(0x8005c91c), table(0x8005f6b8))
    rows = profile['camera_trace']['rows']
    inputs = [inputs_for_row(row, direction.look(row['target'], row['camera'], row['base'][1])) for row in rows]
    args.output.mkdir(parents=True)
    header = args.engine/'runtime/camera_aim.hpp'
    copied = args.output/'camera_aim.hpp'
    copied.write_bytes(header.read_bytes())
    cpp = args.output/'probe.cpp'; cpp.write_text(PROBE)
    exe = args.output/'probe.exe'
    compiler = Path('C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/Llvm/x64/bin/clang++.exe')
    command = [str(compiler), '-std=c++20', '-O2', '-Wall', '-Wextra', '-Werror',
               '-D_CRT_SECURE_NO_WARNINGS', str(cpp), '-o', str(exe)]
    compilation = subprocess.run(command, capture_output=True, text=True)
    if compilation.returncode: raise RuntimeError(compilation.stdout+compilation.stderr)
    payload = '\n'.join(' '.join(map(str, [i, *v])) for i, v in enumerate(inputs))
    result = subprocess.run([str(exe)], input=payload, check=True, capture_output=True, text=True)
    actual = [json.loads(line) for line in result.stdout.splitlines()]
    if len(actual) != len(rows) or any(len(v) != 4 or v[0] != i for i, v in enumerate(actual)):
        raise ValueError('Incomplete or reordered C++ output')
    errors = [dict(row=i, expected=row['angles'], actual=actual[i][1:])
              for i, row in enumerate(rows) if row['angles'] != actual[i][1:]]
    report = dict(scope=__doc__, updates=len(rows), exact=not errors, error_count=len(errors), errors=errors,
                  input_sha256={n: hashlib.sha256(b).hexdigest() for n, b in data.items()},
                  header_sha256=hashlib.sha256(copied.read_bytes()).hexdigest(),
                  probe_sha256=hashlib.sha256(cpp.read_bytes()).hexdigest(),
                  executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), compile_command=command,
                  negative_controls_rejected=controls)
    (args.output/'comparison.json').write_text(json.dumps(report, indent=2))
    print(json.dumps({k: report[k] for k in ('updates', 'exact', 'error_count')}))
    if errors: raise SystemExit(1)


if __name__ == '__main__': main()
