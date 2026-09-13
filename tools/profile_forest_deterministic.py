"""Isolated, instrumented Forest replay: exact manual ticks per rendered frame.

This is NOT normal gameplay FPS. The engine clock stays paused while the original
controller update, collision movement and sprite advancement run on a fixed 1/2
tick schedule. Autonomous game-side telemetry avoids HTTP sampling/phase drift.
"""
import epok_documents as documents
from profile_runtime import EDITOR
import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import socket
import struct
import subprocess
import time
import urllib.request

from compare_runtime import compare
from project_layout import project_manifest
from profile_forest_route import PHASES, digest, profile, save, save_png
from profile_runtime import ROOT, FIELDS, STREAMING_FIELDS, WARMUP_FIELDS, read_counters, symbol_address

META = "index phase phase_ticks replay_steps player_x player_y player_z camera_x camera_y camera_z clip sprite_frame flip_x texture total_ticks".split()
TAIL = "frame_microseconds dropped_steps dropped_triangles visible_chunks transformed_vertices triangles".split()
STRIDE = len(META) + len(FIELDS) + len(TAIL)
CAPACITY = sum(p[2] for p in PHASES)
SCOPE = ("Instrumented deterministic replay, not normal gameplay FPS: engine time remains paused; "
         "the original ForestController.update and sprite advancement execute an exact autonomous "
         "tick schedule in frame_update. Completed-frame telemetry uses a fixed BSS buffer, with "
         "identical instrumentation in both builds. Natural gameplay captures remain required.")


def replace_function(source, signature, body):
    start = source.index(signature)
    opening = source.index("{", start)
    depth, closing = 1, opening + 1
    # Existing Forest functions contain balanced braces and no brace characters
    # inside strings/comments. Fail closed if the expected signature is absent.
    while depth:
        if closing >= len(source):
            raise ValueError("Unbalanced controller function")
        depth += (source[closing] == "{") - (source[closing] == "}")
        closing += 1
    return source[:start] + signature + "{\n" + body + "\n}" + source[closing:]


def instrument(source, steps):
    if "forest_replay_rows" in source:
        raise ValueError("Source is already instrumented")
    assert 'void ForestController::start(Transform& t) {' in source
    source = source.replace('void ForestController::start(Transform& t) {',
                            'void ForestController::start(Transform& t) {\n    time.set_paused(true);', 1)
    buffer_capacity = len(expected_schedule(steps))
    definitions = f'''
// Explicit isolated benchmark instrumentation; see provenance.json and patch.diff.
extern "C" {{
volatile uint32_t forest_replay_rows[{buffer_capacity}][{STRIDE}]={{}};
volatile uint32_t forest_replay_status[6]={{}};
}}
namespace {{
constexpr uint32_t replay_bits[]={{{','.join(str(p[1]) for p in PHASES)}}};
constexpr uint32_t replay_targets[]={{{','.join(str(p[2]) for p in PHASES)}}};
constexpr uint32_t replay_steps[]={{{','.join(map(str, steps))}}};
uint32_t replay_phase=0,replay_ticks=0,replay_count=0;
bool replay_pending=false;
}}
'''
    source = source.replace('using namespace epok;', 'using namespace epok;\n' + definitions, 1)
    assignments = '\n'.join(f'        row[{len(META)+i}]=performance_stats.{name};' for i, name in enumerate(FIELDS))
    tail = ['time.frame_microseconds', 'time.dropped_steps', 'lighting_stats.dropped_triangles',
            'mesh_stats.visible_chunks', 'mesh_stats.transformed_vertices', 'lighting_stats.triangles']
    assignments += '\n' + '\n'.join(f'        row[{len(META)+len(FIELDS)+i}]={value};' for i, value in enumerate(tail))
    body = f'''    time.set_paused(true);
    forest_replay_status[0]=0x4652504c;
    forest_replay_status[2]={buffer_capacity};forest_replay_status[3]={STRIDE};forest_replay_status[5]=1;
    if(performance_stats.frame<20 || forest_replay_status[4])return;
    // time.advance has just measured the preceding completed frame interval.
    if(replay_pending){{
        auto* row=forest_replay_rows[replay_count];
{assignments}
        ++replay_count;replay_pending=false;
        forest_replay_status[1]=replay_count; // Publish completed row last.
    }}
    if(replay_ticks==replay_targets[replay_phase]){{++replay_phase;replay_ticks=0;}}
    if(replay_phase=={len(PHASES)}){{forest_replay_status[4]=1;return;}}
    uint32_t steps=replay_steps[replay_count%{len(steps)}];
    if(steps>replay_targets[replay_phase]-replay_ticks)steps=replay_targets[replay_phase]-replay_ticks;
    forest_test_command[0]=replay_bits[replay_phase];
    forest_test_command[1]=0x54455354;forest_test_command[2]=replay_phase+1;
    for(uint32_t step=0;step<steps;++step){{
        time.begin_tick();
        const Fixed dt(time.delta_raw,Fixed::RAW);
        update(t,dt);
        entity().sprite_animator.advance(dt,entity().sprite);
        ++replay_ticks;
    }}
    auto* row=forest_replay_rows[replay_count];
    row[0]=replay_count;row[1]=replay_phase;row[2]=replay_ticks;row[3]=steps;
    for(int c=0;c<3;++c)row[4+c]=uint32_t(t.position[c].raw());
    if(auto* camera=find_entity("Forest Camera"))for(int c=0;c<3;++c)row[7+c]=uint32_t(camera->transform.position[c].raw());
    row[10]=uint32_t(current_clip);row[11]=entity().sprite_animator.frame;
    row[12]=entity().sprite.flip_x;row[13]=uint32_t(entity().sprite.texture);row[14]=time.ticks;
    replay_pending=true;'''
    return replace_function(source, 'void ForestController::frame_update(Transform& t,uint32_t) ', body) if 'void ForestController::frame_update(Transform& t,uint32_t) ' in source else replace_function(
        source.replace('void ForestController::frame_update(Transform&,uint32_t)', 'void ForestController::frame_update(Transform& t,uint32_t)', 1),
        'void ForestController::frame_update(Transform& t,uint32_t) ', body)


def expected_schedule(steps):
    result = []
    for phase, (_, _, target) in enumerate(PHASES):
        ticks = 0
        while ticks < target:
            count = min(steps[len(result) % len(steps)], target - ticks)
            ticks += count
            result.append((phase, ticks, count))
    return result


def decode_rows(raw, count):
    if len(raw) != count * STRIDE * 4:
        raise ValueError("Incomplete replay telemetry")
    rows = []
    for i in range(count):
        values = struct.unpack_from(f'<{STRIDE}I', raw, i * STRIDE * 4)
        row = dict(zip(FIELDS, values[len(META):len(META)+len(FIELDS)]))
        row.update(zip(TAIL, values[-len(TAIL):]))
        row['present_wait_estimate_us'] = max(0, row['frame_microseconds'] - row['frame_scanlines'] * 64)
        row['replay'] = dict(zip(META, values[:len(META)]))
        row['replay_steps'] = row['replay']['replay_steps']
        rows.append(row)
    return rows


def compare_replays(baseline, candidate, before_vram, after_vram):
    for key in ('instrumentation_sha256', 'original_controller_sha256', 'steps_pattern'):
        if baseline['build'][key] != candidate['build'][key]:
            raise ValueError(f'Incomparable replay provenance: {key}')
    if len(baseline['frames']) != len(candidate['frames']):
        raise ValueError('Replay frame counts differ')
    for before, after in zip(baseline['frames'], candidate['frames']):
        if before['replay'] != after['replay']:
            raise ValueError(f"Replay state/steps differ at index {before['replay']['index']}")
        for field in ('visible_chunks','transformed_vertices','triangles'):
            if before[field] != after[field]:
                raise ValueError(f"Replay geometry counter {field} differs at index {before['replay']['index']}")
    # Keep the existing strict gates. Group the explicitly executed manual
    # steps, while preserving the engine's actual zero steps in raw profiles.
    reports = [dict(report, frames=[dict(row, steps=row['replay_steps']) for row in report['frames']])
               for report in (baseline, candidate)]
    result = compare(*reports, baseline_vram=before_vram, candidate_vram=after_vram)
    result['scope'] = SCOPE
    result['exact_state_and_steps'] = True
    result['per_frame_cpu_delta'] = [b['frame_scanlines']-a['frame_scanlines']
                                     for a, b in zip(baseline['frames'], candidate['frames'])]
    result['per_phase'] = {}
    for phase, (name, _, _) in enumerate(PHASES):
        parts = [dict(report, frames=[row for row in report['frames'] if row['replay']['phase']==phase]) for report in reports]
        result['per_phase'][name] = compare(*parts, minimum_samples=5)
    result['passed'] &= all(part['passed'] for part in result['per_phase'].values())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--editor', type=Path, default=EDITOR)
    parser.add_argument('--steps', default='1,2', help='Deterministic pattern: 1, 2, or 1,2')
    parser.add_argument('--seconds', type=int, default=150)
    parser.add_argument('--port', type=int)
    parser.add_argument('--streaming', choices=['inherit','on','off'], default='inherit')
    parser.add_argument('--visibility', choices=['inherit','on','off'], default='inherit')
    parser.add_argument('--compare', type=Path, help='Completed deterministic baseline output directory')
    parser.add_argument('--prepare-only', action='store_true', help='Create isolated copy/patch without build or emulator')
    args = parser.parse_args()
    steps = [int(x) for x in args.steps.split(',')]
    if not steps or any(x not in (1,2) for x in steps):
        parser.error('Steps must be a comma-separated pattern of 1 and/or 2')
    if args.output.exists() and any(args.output.iterdir()):
        parser.error('Output must be new or empty')
    source, out, editor = args.project.resolve(), args.output.resolve(), args.editor.resolve()
    if out == source or source in out.parents:
        parser.error('Output must be outside the source project')
    out.mkdir(parents=True, exist_ok=True)
    project = out/'project'
    shutil.copytree(source, project, ignore=shutil.ignore_patterns('.epok','.git','__pycache__'))
    controller = project/'assets/scripts/ForestController.cpp'
    original = controller.read_text(encoding='utf-8-sig')
    patched = instrument(original, steps)
    documents.write_text(controller, patched, encoding='utf-8')
    documents.write_text(out/'ForestController.original.cpp', original, encoding='utf-8')
    documents.write_text(out/'patch.diff', ''.join(difflib.unified_diff(original.splitlines(True), patched.splitlines(True),
                                                          fromfile='original/ForestController.cpp', tofile='instrumented/ForestController.cpp')), encoding='utf-8')
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text(encoding='utf-8-sig'))
    for argument, key in ((args.streaming,'streaming_geometry'),(args.visibility,'precomputed_visibility')):
        if argument!='inherit': manifest.setdefault('rendering',{})[key] = argument=='on'
    save(manifest_path, manifest)
    config_path = next(p for p in (project/'Local.epokconfig', ROOT/'Local.epokconfig', ROOT/'Editor.epokconfig') if p.is_file())
    config = documents.loads(config_path.read_text(encoding='utf-8-sig'))
    if args.port: config['web_port'] = args.port
    save(project/'Local.epokconfig', config)
    build = dict(project=str(project), original_project=str(source), editor=str(editor), editor_sha256=digest(editor),
                 original_controller_sha256=hashlib.sha256(original.encode()).hexdigest(),
                 instrumentation_sha256=digest(controller), steps_pattern=steps, manifest=manifest,
                 optimization_override=os.environ.get('EPOK_RUNTIME_OPT','').removeprefix('-') or None,
                 detail_timers=os.environ.get('EPOK_PROFILE_DETAIL')=='1', gte_validation=os.environ.get('EPOK_VALIDATE_GTE')=='1')
    save(out/'provenance.json', dict(scope=SCOPE, build=build, schedule=expected_schedule(steps)))
    if args.prepare_only:
        print(f'Prepared isolated deterministic project: {project}')
        return
    with socket.socket() as available: available.bind(('127.0.0.1', int(config['web_port'])))
    flags = getattr(subprocess,'CREATE_NO_WINDOW',0)
    compiled = subprocess.run([str(editor),'--project',str(project),'--build-psx'], capture_output=True,text=True,creationflags=flags,timeout=240)
    documents.write_text(out/'build.log', compiled.stdout+compiled.stderr,encoding='utf-8')
    assert compiled.returncode==0, compiled.stdout+compiled.stderr
    folder = project/'.epok/build'
    symbols = (folder/'epok.map').read_text()
    addresses = {name:symbol_address(symbols,name) for name in ('forest_replay_status','forest_replay_rows','epok::streaming_stats','epok::streaming_warmup_stats')}
    assert addresses['forest_replay_status'] is not None and addresses['forest_replay_rows'] is not None
    build['executable_sha256'] = digest(folder/'epok.ps-exe')
    build['scene_header_sha256'] = digest(folder/'scene.hh')
    display = (folder/'display.hh').read_text()
    width,height = [int(re.search(name+r'\s*=\s*(\d+)',display)[1]) for name in ('display_width','display_height')]
    def request(path,data=None):
        req=urllib.request.Request(f"http://127.0.0.1:{config['web_port']}/api/v1/"+path,data=data)
        with urllib.request.urlopen(req,timeout=4) as response: return response.read()
    report = dict(passed=False,scope=SCOPE,build=build)
    with (out/'runtime.log').open('w') as log:
        process=subprocess.Popen([str(editor),'--project',str(project),'--play-psx','--stop-after',str(args.seconds)],stdout=log,stderr=log,creationflags=flags)
        try:
            deadline=time.monotonic()+args.seconds-5
            while True:
                assert process.poll() is None, 'Owned emulator stopped before replay completed'
                try:
                    ram=request('cpu/ram/raw')
                    status=struct.unpack_from('<6I',ram,addresses['forest_replay_status'])
                    if status[0]==0x4652504c and status[4]: break
                except OSError: pass
                assert time.monotonic()<deadline, 'Deterministic replay timeout'
                time.sleep(.1)
            # No route advancement follows DONE. Allow both display buffers to
            # finish the final pose before capturing terminal VRAM.
            time.sleep(.2)
            request('execution-flow?function=pause',b'')
            ram=request('cpu/ram/raw')
            (out/'ram.bin').write_bytes(ram)
            status=struct.unpack_from('<6I',ram,addresses['forest_replay_status'])
            assert status[2:4]==(len(expected_schedule(steps)),STRIDE) and status[5]==1
            count=status[1]
            assert count==len(expected_schedule(steps))
            offset=addresses['forest_replay_rows']
            raw=ram[offset:offset+count*STRIDE*4]
            (out/'frames.bin').write_bytes(raw)
            rows=decode_rows(raw,count)
            assert [(r['replay']['phase'],r['replay']['phase_ticks'],r['replay_steps']) for r in rows]==expected_schedule(steps)
            assert all(r['steps']==0 and r['dropped_steps']==0 and r['dropped_triangles']==0
                       and r['gte_validation_errors']==0 for r in rows)
            assert all(r['replay']['index']==i for i,r in enumerate(rows))
            vram=request('gpu/vram/raw');(out/'vram.bin').write_bytes(vram)
            save_png(vram,out/'screen.png',width,height)
            capture=profile(rows,build);capture['comparison_limitation']=SCOPE
            save(out/'profile.json',capture)
            for phase,(name,_,_) in enumerate(PHASES):
                part=profile([r for r in rows if r['replay']['phase']==phase],build)
                part['comparison_limitation']=SCOPE;save(out/f'{name}.json',part)
            report.update(passed=True,frames=count,status=list(status),vram_sha256=hashlib.sha256(vram).hexdigest())
            report['music_state'] = {}
            for field in ('prepared','booting','ready','lookup','data_owner','boot_failed','active','requested'):
                address = symbol_address(symbols, 'epok::music_'+field)
                report['music_state'][field] = (None if address is None else
                    struct.unpack_from('<I',ram,address)[0] if field in ('active','requested') else bool(ram[address]))
            for name,fields in (('streaming',STREAMING_FIELDS),('streaming_warmup',WARMUP_FIELDS)):
                report[name]=read_counters(ram,addresses['epok::streaming_stats' if name=='streaming' else 'epok::streaming_warmup_stats'],fields)
            if report['streaming'] is not None:
                assert report['streaming']['errors']==0 and report['streaming']['timeouts']==0, report['streaming']
            if args.compare:
                base=args.compare.resolve()
                result=compare_replays(documents.loads((base/'profile.json').read_text()),capture,(base/'vram.bin').read_bytes(),vram)
                save(out/'comparison.json',result);report['comparison_passed']=result['passed']
        except Exception as error:
            report['passed']=False
            report['error']=f'{type(error).__name__}: {error}'
            raise
        finally:
            save(out/'report.json',report)
            if process.poll() is None:
                if os.name=='nt': subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=flags,timeout=15)
                else: process.terminate()
            process.wait(timeout=15)
    assert hashlib.sha256((source/'assets/scripts/ForestController.cpp').read_text(encoding='utf-8-sig').encode()).hexdigest()==build['original_controller_sha256'], 'Original controller changed'
    print(f'PASS deterministic instrumented replay ({count} exact frames): {out}')
    if args.compare and not report['comparison_passed']: raise SystemExit(1)


if __name__=='__main__': main()
