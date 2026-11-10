"""Validate the exported EXE + external data, or a CD build, on a private PSX.

Only player position is patched to shorten encounter setup; controller input uses
Redux's ordinary virtual pad. Optional debugger hooks capture completed CPU work
or drive reproducible setup. No card changes or overclock.
Run in a terminal so the emulator inherits a console.
"""
from pathlib import Path
import argparse
import collections
import hashlib
import json
import os
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import zipfile
import yaml

E = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(E / 'Tools'))
import profile_runtime as profiler
import profile_runtime_memory as memory_profiler
sys.path.insert(0, str(E / 'tests/integration/gameplay_helpers'))
import complete_observation

parser = argparse.ArgumentParser()
parser.add_argument('--project', type=Path, required=True)
parser.add_argument('--route', choices=('guards', 'boss', 'closure'), default='guards')
parser.add_argument('--frames', type=int, default=1800)
parser.add_argument('--minimum-loops', type=int, default=None)
parser.add_argument('--package', type=Path, default=None)
parser.add_argument('--cd', type=Path)
parser.add_argument('--header', type=Path, help='Saved generated scene header matching the exported package')
parser.add_argument('--clock-site', type=lambda value: int(value, 0), help='Diagnostic breakpoint before previous-lines load; verify V1/S3 with objdump')
parser.add_argument('--capture-slow-frames', action='store_true', help='Observe completed CPU counters at the native on_frame entry for each interval above 25ms')
parser.add_argument('--deterministic-route', action='store_true', help='Apply the same virtual-pad timeline at pad polling, place the player before physics, and stop at the exact completed frame')
parser.add_argument('--capture-cpu-samples', action='store_true', help='Capture an EPOK_PROFILE_SAMPLE build during completed frames 301..1440; requires deterministic route')
parser.add_argument('--capture-memory', action='store_true', help='Observe the unchanged executable allocator, linked SP writes and unused-RAM canary; diagnostic host overhead is excluded from performance acceptance')
args = parser.parse_args()
if args.route == 'closure' and (not args.deterministic_route or args.frames < 12000):
    parser.error('closure requires deterministic current-project replay and at least12000 frames')
if args.capture_cpu_samples and (not args.deterministic_route or args.frames <= 301):
    parser.error('--capture-cpu-samples requires --deterministic-route and more than 301 frames')
if args.capture_memory and not args.deterministic_route:
    parser.error('--capture-memory requires --deterministic-route for a coherent final RAM snapshot')
P = args.project.resolve(strict=True)
args.package = args.package or P / 'exports/DungeonComplete-ps-exe.zip'
if args.cd:
    args.cd = args.cd.resolve(strict=True)
backend = 'cd' if args.cd else 'pcdrv'
OUT = P / '.epok/validation' / ('stream-' + backend + '-' + args.route + '-' + time.strftime('%Y%m%d-%H%M%S'))
portable = OUT / 'emulator'
portable.mkdir(parents=True)
with socket.socket() as probe:
    probe.bind(('127.0.0.1', 0))
    PORT = probe.getsockname()[1]

build = P / '.epok/build'
header = (args.header or build / 'scene.hh').read_text()
if args.cd:
    exe = build / 'epok.ps-exe'
    mapping = (build / 'epok.map').read_text()
    elf = build / 'epok.elf'
    geometry = (build / 'GEOMETRY.BIN').read_bytes()
else:
    package = OUT / 'package'
    package.mkdir()
    with zipfile.ZipFile(args.package) as archive:
        manifest = json.loads(archive.read('epok-package.json'))
        assert manifest['loader']['pcdrv_files'] == ['GEOMETRY.BIN']
        assert manifest['loader']['pcdrv_required'] and not manifest['loader']['cd_required']
        for record in manifest['files']:
            assert hashlib.sha256(archive.read(record['path'])).hexdigest() == record['sha256']
        for item in archive.infolist():
            target = (package / item.filename).resolve()
            assert target.is_relative_to(package.resolve()), item.filename
            archive.extract(item, package)
    exe = package / 'epok.ps-exe'
    mapping = (package / 'debug/epok.map').read_text()
    elf = package / 'debug/epok.elf'
    geometry = (package / 'GEOMETRY.BIN').read_bytes()
(OUT / 'scene.hh').write_bytes((args.header or build / 'scene.hh').read_bytes())

def address(name):
    found = re.findall(r'0x([0-9a-fA-F]+)\s+' + re.escape(name) + r'\s*$', mapping, re.M)
    assert found, name
    return int(found[-1], 16) & 0x1fffff

names = {
    'fps': ('dungeon_fps_telemetry', 7),
    'flow': ('dungeon_flow_telemetry', 8),
    'music': ('epok::music_sequence_stats', 16),
    'stream': ('epok::streaming_stats', 7),
    'warmup': ('epok::streaming_warmup_stats', 6),
    'texture': ('epok::texture_load_stats', 5),
    'combat': ('combat_telemetry', 9),
    'game': ('dungeon_telemetry', 11),
    'performance': ('epok::performance_stats', len(profiler.FIELDS + profiler.EXTRA_FIELDS)),
    'lighting': ('epok::lighting_stats', 9),
    'audio': ('dungeon_audio_telemetry', 18),
    'peak': ('dungeon_peak_profile', 103),
    'explore_peak': ('dungeon_explore_peak_profile', 103),
    'actors': ('epok::actor_stats', 9),
}
scene_contract = complete_observation.scene_contract(P, header)
compiled_contract = complete_observation.compiled_contract(E, elf, scene_contract)
# Actual common typed prefixes and full linked telemetry, independent of selected provider.
names.update(compiled_contract['names'])
symbols = {key: address(name) for key, (name, _) in names.items()}
(OUT/'current-program-contract.json').write_text(json.dumps({'scene':scene_contract,'abi':compiled_contract,'executable_sha256':complete_observation.sha(exe),'elf_sha256':complete_observation.sha(elf),'header_sha256':complete_observation.sha(OUT/'scene.hh')},indent=2))
# Detail profiling is opt-in and does not change the production acceptance gate.
for key, name in (('hud_detail', 'epok::hud_profile_detail'), ('hud_peak', 'epok::hud_profile_peak')):
    if re.search(r'0x[0-9a-fA-F]+\s+' + re.escape(name) + r'\s*$', mapping, re.M):
        names[key] = (name, 7)
        symbols[key] = address(name)
nm_tool = E / ('.tools/mips/bin/mipsel-none-elf-nm' + ('.exe' if os.name == 'nt' else ''))
nm = subprocess.check_output([str(nm_tool), '-S', '--defined-only', str(elf)], text=True)
count = int(re.search(r'std::array<ActorData,\s*(\d+)> objects', header)[1])
size = int(re.search(r'^\w+ (\w+) \w _ZN4epok7objectsE$', nm, re.M)[1], 16)
player = compiled_contract['player_position_address']
assert size // count == compiled_contract['layouts']['ActorData']['size'], 'ActorData header/ELF mismatch'
pages = collections.Counter()
for line in header.splitlines():
    if line.startswith('inline constexpr MeshGeometry editable_'):
        location = re.search(r',(\d+),(\d+),(\d+)\};$', line)
        assert location, line[:120]
        pages[int(location[1])] += 1
pool = int(re.search(r'stream_pool_pages=(\d+)', header)[1])
assert len(pages) <= pool, (pages, pool)

# Replay uses completed game frames, independent of host wall time and loading.
schedule = ([(0, 0), (480, 34816), (492, 32768), (690, 34816), (702, 32768),
             (900, 34816), (912, 32768), (1440, 0)] if args.route == 'guards' else
            [(0, 0), (390, 16), (510, 0), (540, 34816), (552, 32768), (1440, 0)])
if args.route == 'closure':schedule = [(0,0)]

lua = """
local ffi=require('ffi')
local mem=PCSX.getMemPtr()
local frame=ffi.cast('uint32_t*',mem+@PERF@)
local pad=PCSX.SIO0.slots[1].pads[1]
PCSX.settings.pads[1].DeviceType='Analog';PCSX.settings.pads[1].Connected=true
pad.map();pad.setAnalogMode(true)
local names={'SELECT','L3','R3','START','UP','RIGHT','DOWN','LEFT','L2','R2','L1','R1','TRIANGLE','CIRCLE','CROSS','SQUARE'}
local schedule={@SCHEDULE@}
local last=-1
local pad_changes={}
local function apply_pad_at(n)
 if n>1000000 then return end
 local mask=0;for _,step in ipairs(schedule) do if n>=step[1] then mask=step[2] end end
 if @CLOSURE@ and EPOK_COMPLETE_OBSERVER then mask=EPOK_COMPLETE_OBSERVER.pad() end
 if mask~=last then
  for i,name in ipairs(names) do local b=PCSX.CONSTS.PAD.BUTTON[name]
   if b then if bit.band(mask,bit.lshift(1,i-1))~=0 then pad.setOverride(b) else pad.clearOverride(b) end end
  end;last=mask;pad_changes[#pad_changes+1]=string.format('{"frame":%d,"mask":%d}',n,mask)
 end
end
local function replay_pad()apply_pad_at(tonumber(frame[0])) end
if @DETERMINISTIC@ then apply_pad_at(0) else EPOK_STREAM_PAD=PCSX.Events.createEventListener('GPU::Vsync',replay_pad) end
""".replace('@PERF@', str(symbols['performance'])).replace('@SCHEDULE@', ','.join('{%d,%d}' % x for x in schedule)).replace('@DETERMINISTIC@', str(args.deterministic_route).lower()).replace('@CLOSURE@', str(args.route == 'closure').lower())
fixture = OUT / 'pad.lua'
if args.deterministic_route:
    key_raw = scene_contract['positions_q12']['key']
    lua += """
local regs=PCSX.getRegisters()
local position=ffi.cast('int32_t*',mem+@PLAYER@)
local game=ffi.cast('uint32_t*',mem+@GAME@)
local route_events={};local route_stage=0;local stopped_frame=0;local stopping=false
local flow=ffi.cast('uint32_t*',mem+@FLOW@)
local work={};local work_frames=0;local first_work_frame=0;local last_work_frame=-1
@SAMPLER_SETUP@
local return_hooks={}
local function place(n,x,y,z)
 position[0]=x;position[1]=y;position[2]=z
 route_events[#route_events+1]=string.format('{"frame":%d,"position_q12":[%d,%d,%d]}',n,x,y,z)
end
EPOK_ROUTE_PAD=PCSX.addBreakpoint(@POLL@,'Exec',4,'Route pad sample',function()
 if tonumber(regs.GPR.n.a1)==0 then replay_pad() end
 return true
end)
EPOK_ROUTE_SETUP=PCSX.addBreakpoint(@FRAME@,'Exec',4,'Route setup before physics',function()
 local n=tonumber(frame[0]);if n>1000000 then return true end
 if n>0 and tonumber(regs.GPR.n.a1)>0 and n~=last_work_frame then
  local phase=tonumber(flow[0]);local steps=tonumber(frame[@STEPS@]);local key=phase..':'..steps
  local w=work[key] or {phase,steps,0,0,0,0,0,0};work[key]=w
  w[3]=w[3]+1;w[4]=w[4]+tonumber(frame[1]);w[5]=w[5]+tonumber(frame[@HUD@])
  w[6]=w[6]+tonumber(frame[@RENDER@]);w[7]=w[7]+tonumber(frame[@SCRIPTS@]);w[8]=w[8]+tonumber(frame[@SIMULATION@])
  if first_work_frame==0 then first_work_frame=n end
  last_work_frame=n;work_frames=work_frames+1
 end
 @SAMPLER_FRAME@
 if @CLOSURE@ then
  -- Closure setup/pad is driven by observed real transitions below.
 elseif @GUARDS@ then
  if route_stage==0 and n>300 then place(n,0,0,118784);route_stage=1 end
 else
  if route_stage==0 and n>240 then place(n,@KEY@);route_stage=1
  elseif route_stage==1 and tonumber(game[3])~=0 and n>300 then place(n,0,0,176128);route_stage=2 end
 end
 if n>=@STOP@ and not stopping then
  stopping=true
  local destination=tonumber(regs.GPR.n.ra)
  return_hooks[destination]=PCSX.addBreakpoint(destination,'Exec',4,'Route completed frame',function()
   if stopping then
    if EPOK_MEMORY_OBSERVER then EPOK_MEMORY_OBSERVER.sample() end
    stopped_frame=tonumber(frame[0]);PCSX.pauseEmulator();stopping=false
   end
   return true
  end)
 end
 return true
end)
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
PCSX.WebServer.Handlers['route-state']=function()
 local groups={};for _,w in pairs(work) do
  groups[#groups+1]=string.format('{"phase":%d,"steps":%d,"frames":%d,"cpu_scanlines":%d,"hud_scanlines":%d,"render_scanlines":%d,"scripts_scanlines":%d,"simulation_scanlines":%d}',unpack(w))
 end
 return string.format('{"stopped_frame":%d,"events":[%s],"pad_changes":[%s],"work_frames":%d,"first_work_frame":%d,"work":[%s]%s}',stopped_frame,table.concat(route_events,','),table.concat(pad_changes,','),work_frames,first_work_frame,table.concat(groups,','),@SAMPLER_JSON@)
end
""".replace('@PLAYER@', str(player)).replace('@GAME@', str(symbols['game'])).replace('@FLOW@', str(symbols['flow'])).replace('@POLL@', str(compiled_contract['poll_entry'] | 0x80000000)).replace('@FRAME@', str(compiled_contract['frame_entry'] | 0x80000000)).replace('@GUARDS@', str(args.route == 'guards').lower()).replace('@CLOSURE@', str(args.route == 'closure').lower()).replace('@KEY@', ','.join(map(str, key_raw))).replace('@STOP@', str(args.frames))
    for name in ('steps', 'hud_scanlines', 'render_scanlines', 'scripts_scanlines', 'simulation_scanlines'):
        token = {'steps': 'STEPS', 'hud_scanlines': 'HUD', 'render_scanlines': 'RENDER', 'scripts_scanlines': 'SCRIPTS', 'simulation_scanlines': 'SIMULATION'}[name]
        lua = lua.replace('@' + token + '@', str((profiler.FIELDS + profiler.EXTRA_FIELDS).index(name)))
    if args.capture_cpu_samples:
        sample_control = address('epok::cpu_sample_enabled')
        sample_bins = address('epok::cpu_sample_bins')
        sample_outside = address('epok::cpu_sample_outside')
        sample_layout = address('epok::cpu_sample_layout')
        sampler_setup = f"""
local sample_control=ffi.cast('uint32_t*',mem+{sample_control})
local sample_bins=ffi.cast('uint32_t*',mem+{sample_bins})
local sample_outside=ffi.cast('uint32_t*',mem+{sample_outside})
local sample_layout=ffi.cast('uint32_t*',mem+{sample_layout})
local sample_started=0;local sample_stopped=0
"""
        sampler_frame = f"""
if n==301 and sample_started==0 then
 local count=tonumber(sample_layout[2]);assert(count>0 and count<=8192,'Invalid CPU histogram size')
 sample_control[0]=0;for i=0,count-1 do sample_bins[i]=0 end;sample_outside[0]=0
 sample_started=n;sample_control[0]=1
elseif n>={min(1440, args.frames)} and sample_started~=0 and sample_stopped==0 then
 sample_control[0]=0;sample_stopped=n
end
"""
        sampler_json = "string.format(',\"sampler_window\":[%d,%d]',sample_started,sample_stopped)"
    else:
        sampler_setup, sampler_frame, sampler_json = '', '', "''"
    lua = lua.replace('@SAMPLER_SETUP@', sampler_setup).replace('@SAMPLER_FRAME@', sampler_frame).replace('@SAMPLER_JSON@', sampler_json)
if args.capture_slow_frames:
    # At this non-static member entry, the MIPS ABI supplies elapsed us in A1.
    # The previous PerformanceStats record is complete before frame_update runs.
    lua += """
local slow_records={};local slow_count=0
local perf=ffi.cast('uint32_t*',mem+@PERF@)
local flow=ffi.cast('uint32_t*',mem+@FLOW@)
local subsystems=ffi.cast('uint32_t*',mem+@SUBSYSTEMS@)
local regs=PCSX.getRegisters()
EPOK_SLOW_FRAME_PROBE=PCSX.addBreakpoint(@SITE@,'Exec',4,'Completed slow-frame CPU probe',function()
 local us=tonumber(regs.GPR.n.a1)
 if us>25000 then
  slow_count=slow_count+1
  if #slow_records<64 then
   local values={};for i=0,@LAST@ do values[#values+1]=tostring(tonumber(perf[i])) end
   local timers={};for i=1,12 do timers[#timers+1]=tostring(tonumber(subsystems[i])) end
   slow_records[#slow_records+1]=string.format('{"interval_us":%d,"flow_phase":%d,"fade":%d,"performance":[%s],"subsystem_scanlines":[%s]}',us,tonumber(flow[0]),tonumber(flow[2]),table.concat(values,','),table.concat(timers,','))
  end
 end
 return true
end)
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
PCSX.WebServer.Handlers['slow-frames']=function()return string.format('{"count":%d,"records":[%s]}',slow_count,table.concat(slow_records,',')) end
""".replace('@PERF@', str(symbols['performance'])).replace('@FLOW@', str(symbols['flow'])).replace('@SUBSYSTEMS@', str(address('dungeon_profile'))).replace('@LAST@', str(len(profiler.FIELDS + profiler.EXTRA_FIELDS) - 1)).replace('@SITE@', str(compiled_contract['frame_entry'] | 0x80000000))
if args.clock_site:
    lua += """
local clock_state='{}'
local lines=ffi.cast('uint16_t*',mem+@LINES@)
local ticks=ffi.cast('uint16_t*',mem+@TICKS@)
local regs=PCSX.getRegisters()
EPOK_CLOCK_PROBE=PCSX.addBreakpoint(@SITE@,'Exec',4,'Native clock gap probe',function()
 local current=tonumber(regs.GPR.n.v1)%65536
 local delta=(current-tonumber(lines[0]))%65536
 if delta>=225 then
  local now=tonumber(regs.GPR.n.s3)%65536
  clock_state=string.format('{"gap_lines":%d,"lines_before":%d,"lines_now":%d,"ticks_before":%d,"ticks_now":%d,"delta_ticks":%d}',delta,tonumber(lines[0]),current,tonumber(ticks[0]),now,(now-tonumber(ticks[0]))%65536)
  PCSX.pauseEmulator();return false
 end
 return true
end)
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
PCSX.WebServer.Handlers['clock-probe']=function()return clock_state end
""".replace('@LINES@', str(address('epok::sequence_previous_lines'))).replace('@TICKS@', str(address('epok::sequence_previous_ticks'))).replace('@SITE@', str(args.clock_site))
if args.capture_memory:
    memory_config, memory_writes = memory_profiler.elf_probe_config(elf, E / '.tools/mips/bin')
    lua += memory_profiler.lua_probe(memory_config, memory_writes)
    lua += """
EPOK_MEMORY_FRAME=PCSX.addBreakpoint(@FRAME@,'Exec',4,'Memory observer frame',function()
 EPOK_MEMORY_OBSERVER.sample();return true
end)
""".replace('@FRAME@', str(compiled_contract['frame_entry'] | 0x80000000))
    (OUT / 'memory-config.json').write_text(json.dumps(memory_config, indent=2))
lua += complete_observation.observer_lua(compiled_contract,scene_contract,symbols,args.route == 'closure')
fixture.write_text('-- Automated run: keep the SPU running and silence only its output.\npcall(function() PCSX.settings.spu.Mute=true end)\n' + lua)
bios = E / '.tools/redux/openbios-fastboot.bin'
if not bios.exists():
    bios = E / '.tools/redux/openbios.bin'
command = [str(E / '.tools/redux/pcsx-redux.main'), '-portable', str(portable),
           '-logfile', str(OUT / 'runtime.log'), '-run', '-stdout', '-fastboot',
           '-noupdate', '-interpreter', '-softgpu', '-2mb', '-no-gdb', '-no-ui',
           '-webserver', '-webserver-port', str(PORT), '-bios', str(bios),
           '-loadexe', str(exe), '-dofile', str(fixture)]
command += ['-iso', str(args.cd), '-no-pcdrv'] if args.cd else ['-pcdrv', '-pcdrvbase', str(package)]
if args.clock_site or args.capture_slow_frames or args.deterministic_route or args.capture_memory:
    command += ['-debugger']
log = (OUT / 'process.log').open('w')
process = subprocess.Popen(command, cwd=portable, stdout=log, stderr=log)

def request(path, data=None):
    return urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{PORT}/api/v1/' + path, data=data), timeout=3).read()

def teleport(position):
    request('execution-flow?function=pause', b'')
    try:
        data = struct.pack('<3i', *(round(v * 4096) for v in position))
        request(f'cpu/ram/raw?offset={player}&size={len(data)}', data)
    finally:
        request('execution-flow?function=resume', b'')

def verify_vram(vram):
    bank = re.search(r'inline constexpr Texture bank_textures_0\[\]=\{(.*?)\};', header)[1]
    checked = 0
    for descriptor in re.findall(r'\{([^{}]+)\}', bank):
        parts = descriptor.split(',')
        width, height, x, y, words, cx, cy = map(int, parts[:7])
        assert parts[7] == 'nullptr'
        bits = int(parts[9])
        page, offset = int(parts[10].rstrip('u')), int(parts[11])
        source = geometry[page * 65536 + offset:page * 65536 + offset + words * height * 2]
        for row in range(height):
            expected = source[row * words * 2:(row + 1) * words * 2]
            assert vram[(y + row) * 2048 + x * 2:(y + row) * 2048 + (x + words) * 2] == expected, (x, y, row)
        palette = re.search(r'\b' + re.escape(parts[8]) + r'\[\]=\{([^}]+)\}', header)[1]
        colors = 16 if bits == 4 else 256
        expected = struct.pack('<%dH' % colors, *list(map(int, palette.split(',')))[:colors])
        assert vram[cy * 2048 + cx * 2:cy * 2048 + cx * 2 + colors * 2] == expected
        checked += len(source) + len(expected)
    return checked

try:
    # Thousands of external breakpoint callbacks cost host time, not guest RAM.
    # This diagnostic never supplies a passing frame-time acceptance result.
    deadline = time.monotonic() + max(120, args.frames / 30 + 30) * (10 if args.capture_memory else 1)
    samples, events, stage, vram_bytes = [], [], 0, 0
    maximum_guards, maximum_dropped, maximum_skipped, maximum_failed = 0, 0, 0, 0
    states = set()
    counters = None
    stopped_snapshot = False
    while True:
        assert process.poll() is None, (OUT / 'process.log').read_text()[-2000:]
        assert time.monotonic() < deadline, ('Streaming playback timed out',
            {key: counters[key] for key in ('actors', 'flow', 'fps')} if counters else 'No telemetry received')
        try:
            ram = request('cpu/ram/raw')
        except OSError:
            time.sleep(.1)
            continue
        counters = {key: list(struct.unpack_from('<%dI' % count, ram, symbols[key])) for key, (_, count) in names.items()}
        if args.capture_memory:
            observation = json.loads(request('lua/memory-observer'))
            (OUT / 'memory-observer.json').write_text(json.dumps(observation, indent=2))
            assert not observation['error'], observation
        for key in ('hud_detail', 'hud_peak'):
            if key in counters and counters[key][0] == 0:
                del counters[key]  # Publication in progress; do not join partial phases.
        if args.clock_site:
            probe = json.loads(request('lua/clock-probe'))
            if probe:
                (OUT / 'clock-probe.json').write_text(json.dumps(probe, indent=2))
                print('Clock probe:', probe, flush=True)
                raise RuntimeError('Captured clock gap; evidence: ' + str(OUT))
        frame = counters['performance'][0]
        if not 0 < frame < 1000000 or counters['flow'][6] == 0:
            time.sleep(.1)
            continue
        perf = dict(zip(profiler.FIELDS + profiler.EXTRA_FIELDS, counters['performance']))
        maximum_dropped = max(maximum_dropped, counters['lighting'][5])
        maximum_skipped = max(maximum_skipped, perf['stream_budget_skipped_chunks'])
        maximum_failed = max(maximum_failed, perf['stream_failed_chunks'])
        maximum_guards = max(maximum_guards, counters['combat'][2])
        states.add(counters['game'][1])
        if counters['flow'][0] == 4 and not vram_bytes:
            vram = request('gpu/vram/raw')
            (OUT / 'vram.bin').write_bytes(vram)
            vram_bytes = verify_vram(vram)
        if not args.deterministic_route and args.route == 'guards' and stage == 0 and frame > 300:
            teleport([0, 0, 29]);stage = 1;events.append({'frame': frame, 'position': [0, 0, 29]})
        elif not args.deterministic_route and args.route == 'boss' and stage == 0 and frame > 240:
            pos = [value/4096 for value in scene_contract['positions_q12']['key']]
            teleport(pos);stage = 1;events.append({'frame': frame, 'position': pos})
        elif not args.deterministic_route and args.route == 'boss' and stage == 1 and counters['game'][3] and frame > 300:
            teleport([0, 0, 43]);stage = 2;events.append({'frame': frame, 'position': [0, 0, 43]})
        samples.append({'frame': frame, **{key: value for key, value in counters.items() if key not in ('performance', 'peak')},
                        'streamed_chunks': perf['streamed_chunks'], 'skipped': perf['stream_budget_skipped_chunks']})
        if frame >= args.frames:
            if not args.deterministic_route:
                break
            if json.loads(request('lua/route-state'))['stopped_frame'] == args.frames:
                if not stopped_snapshot:
                    # The CPU can reach its stop point between the RAM read and
                    # this control request. Read RAM again after observing the
                    # pause so counters and the route log share the same instant.
                    stopped_snapshot = True
                    continue
                break
        time.sleep(.1)
    fps, music, stream, texture = (counters[k] for k in ('fps', 'music', 'stream', 'texture'))
    (OUT / 'samples.json').write_text(json.dumps(samples, indent=2))
    (OUT / 'last-counters.json').write_text(json.dumps(counters, indent=2))
    if args.deterministic_route:
        replay = json.loads(request('lua/route-state'))
        replay['work'].sort(key=lambda group: (group['phase'], group['steps']))
        (OUT / 'replay.json').write_text(json.dumps(replay, indent=2))
        assert replay['stopped_frame'] == args.frames, replay
        assert [e['frame'] for e in replay['events']] == ([301] if args.route == 'guards' else [241, 301] if args.route == 'boss' else []), replay
        if args.route != 'closure':
            assert [(e['frame'], e['mask']) for e in replay['pad_changes']] == [entry for entry in schedule if entry[0] <= args.frames], replay
        assert replay['work_frames'] == counters['fps'][0], (replay['work_frames'], counters['fps'])
        events = [{'frame': e['frame'], 'position': [v / 4096 for v in e['position_q12']]} for e in replay['events']]
        if args.capture_cpu_samples:
            assert replay['sampler_window'] == [301, min(1440, args.frames)], replay
            base, shift, bins = struct.unpack_from('<3I', ram, sample_layout)
            assert 0x10000 <= base < 0x200000 and base % 4 == 0 and 2 <= shift <= 16
            assert 0 < bins <= 8192 and base + (bins << shift) <= 0x200000
            histogram_bytes = int(re.search(r'^\w+ (\w+) \w _ZN4epok15cpu_sample_binsE$', nm, re.M)[1], 16)
            assert histogram_bytes == bins * 4, (histogram_bytes, base, shift, bins)
            assert struct.unpack_from('<I', ram, sample_control)[0] == 0
            counts = list(struct.unpack_from(f'<{bins}I', ram, sample_bins))
            outside = struct.unpack_from('<I', ram, sample_outside)[0]
            assert sum(counts) > 0, 'CPU sampler collected no in-range samples'
            (OUT / 'cpu-samples.json').write_text(json.dumps({'base': base, 'shift': shift, 'bins': counts, 'outside': outside, 'window': replay['sampler_window']}))
    if args.capture_slow_frames:
        captured = json.loads(request('lua/slow-frames'))
        for record in captured['records']:
            record['performance'] = dict(zip(profiler.FIELDS + profiler.EXTRA_FIELDS, record['performance']))
            record['present_wait_estimate_us'] = max(0, record['interval_us'] - record['performance']['frame_scanlines'] * 64)
        (OUT / 'slow-frames.json').write_text(json.dumps(captured, indent=2))
        assert captured['count'] >= counters['fps'][1], (captured['count'], counters['fps'])
    if args.capture_memory:
        observation = json.loads(request('lua/memory-observer'))
        current_heap = memory_profiler.heap_snapshot(ram, memory_config['metadata'], memory_config['heap_start'])
        observation['final_heap_snapshot'] = current_heap
        observation['executable_sha256'] = hashlib.sha256(exe.read_bytes()).hexdigest()
        observation['coverage'] = ('Allocator entry/return snapshots include block headers and free-list footprint. '
            'Linked SP writes include startup and IRQ functions. The canary detects additional RAM writes, '
            'but does not prove reserved stack depth in unlinked BIOS or generated code. '
            'No target code or BSS is added. This diagnostic is not frame-time acceptance.')
        (OUT / 'memory-observer.json').write_text(json.dumps(observation, indent=2))
        assert observation['started'] and not observation['error'] and observation['canary_scans'] > 0, observation
        if args.deterministic_route:
            assert current_heap['live_bytes'] == observation['heap_live_bytes'], observation
            assert current_heap['footprint_bytes'] == observation['heap_footprint_bytes'], observation
    observed = json.loads(request('lua/complete-observation'))
    (OUT/'complete-observation.json').write_text(json.dumps(observed,indent=2))
    functional = complete_observation.validate_observation(observed['rows'],require_closure=args.route=='closure')
    if args.route=='closure':assert observed['completed'], observed['stage']
    assert music[11] == 0 and music[12] == 0 and music[10] == 1, music
    minimum_loops = args.minimum_loops if args.minimum_loops is not None else (1 if args.frames >= 4000 else 0)
    assert music[4] >= minimum_loops, f'Replay must cross {minimum_loops} MIDI loop boundaries: {music[4]}'
    assert stream[0] >= 4 and stream[1] >= len(geometry) and stream[4] == stream[5] == 0, stream
    assert vram_bytes and texture[3] == texture[4] == 0, texture
    assert maximum_dropped == maximum_skipped == maximum_failed == 0
    if args.route == 'guards':assert maximum_guards == 3 and counters['combat'][3] > 10
    elif args.route == 'boss':assert 1 in states and counters['game'][5] < 280
    else:assert all(functional.values())
    result = dict(accepted=fps[1] == 0 and not args.capture_memory, backend=backend, route=args.route, frames=fps[0],
                  slow_frames=fps[1], max_frame_us=fps[2], minimum_observed_fps=round(1e6 / fps[2], 2),
                  pool_pages=pool, geometry_pages=dict(pages), archive_bytes=len(geometry),
                  streaming=stream, texture=texture, verified_vram_bytes=vram_bytes, music=music,
                  maximum_guards=maximum_guards, shots=counters['combat'][3], boss_health=counters['game'][5],
                  states=sorted(states), max_dropped_triangles=maximum_dropped,
                  max_stream_skipped_chunks=maximum_skipped, max_stream_failed_chunks=maximum_failed,
                  missing_sound_assets=counters['audio'][-1], teleports=events,
                  deterministic_route=args.deterministic_route, functional_scenarios=functional,
                  current_contract_sha256=complete_observation.sha(OUT/'current-program-contract.json'),
                  complete_observation_sha256=complete_observation.sha(OUT/'complete-observation.json'),
                  diagnostic_memory_observer=args.capture_memory,
                  executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    (OUT / 'samples.json').write_text(json.dumps(samples, indent=2))
    (OUT / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2), flush=True)
    print('Evidence:', OUT, flush=True)
    # Frame time is reported in result['accepted'], not asserted: see the contract.
    assert not result['missing_sound_assets'], result
finally:
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill();process.wait(timeout=5)
    log.close()
