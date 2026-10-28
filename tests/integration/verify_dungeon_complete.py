"""Validate the exported EXE + external data, or a CD build, on a private PSX.

No game breakpoints, card changes or overclock. Only player position is patched
to shorten encounter setup; controller input uses Redux's ordinary virtual pad.
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

parser = argparse.ArgumentParser()
parser.add_argument('--project', type=Path, required=True)
parser.add_argument('--route', choices=('guards', 'boss'), default='guards')
parser.add_argument('--frames', type=int, default=1800)
parser.add_argument('--minimum-loops', type=int, default=None)
parser.add_argument('--package', type=Path, default=None)
parser.add_argument('--cd', type=Path)
parser.add_argument('--header', type=Path, help='Saved generated scene header matching the exported package')
parser.add_argument('--clock-site', type=lambda value: int(value, 0), help='Diagnostic breakpoint before previous-lines load; verify V1/S3 with objdump')
args = parser.parse_args()
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
(OUT / 'scene.hh').write_text(header)

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
}
symbols = {key: address(name) for key, (name, _) in names.items()}
nm_tool = E / ('.tools/mips/bin/mipsel-none-elf-nm' + ('.exe' if os.name == 'nt' else ''))
nm = subprocess.check_output([str(nm_tool), '-S', '--defined-only', str(elf)], text=True)
count = int(re.search(r'std::array<ActorData,\s*(\d+)> objects', header)[1])
size = int(re.search(r'^\w+ (\w+) \w _ZN4epok7objectsE$', nm, re.M)[1], 16)
record = re.search(r'"Player Aqua",(?:true|false),-?\d+,-?\d+,-?\d+,actor_components_(\d+)', header)[1]
slot = int(re.search(r'actor_components_' + record + r'\[\]=\{\{UINT64_C\(\d+\),"[^"]+",(?:true|false),-?\d+,(-?\d+),', header)[1])
player = address('epok::objects') + slot * (size // count) + 8
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
EPOK_STREAM_PAD=PCSX.Events.createEventListener('GPU::Vsync',function()
 local n=tonumber(frame[0]);if n>1000000 then return end
 local mask=0;for _,step in ipairs(schedule) do if n>=step[1] then mask=step[2] end end
 if mask~=last then
  for i,name in ipairs(names) do local b=PCSX.CONSTS.PAD.BUTTON[name]
   if b then if bit.band(mask,bit.lshift(1,i-1))~=0 then pad.setOverride(b) else pad.clearOverride(b) end end
  end;last=mask
 end
end)
""".replace('@PERF@', str(symbols['performance'])).replace('@SCHEDULE@', ','.join('{%d,%d}' % x for x in schedule))
fixture = OUT / 'pad.lua'
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
fixture.write_text(lua)
bios = E / '.tools/redux/openbios-fastboot.bin'
if not bios.exists():
    bios = E / '.tools/redux/openbios.bin'
command = [str(E / '.tools/redux/pcsx-redux.main'), '-portable', str(portable),
           '-logfile', str(OUT / 'runtime.log'), '-run', '-stdout', '-fastboot',
           '-noupdate', '-interpreter', '-softgpu', '-2mb', '-no-gdb', '-no-ui',
           '-webserver', '-webserver-port', str(PORT), '-bios', str(bios),
           '-loadexe', str(exe), '-dofile', str(fixture)]
command += ['-iso', str(args.cd), '-no-pcdrv'] if args.cd else ['-pcdrv', '-pcdrvbase', str(package)]
if args.clock_site:
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
    deadline = time.monotonic() + max(120, args.frames / 30 + 30)
    samples, events, stage, vram_bytes = [], [], 0, 0
    maximum_guards, maximum_dropped, maximum_skipped, maximum_failed = 0, 0, 0, 0
    states = set()
    while True:
        assert process.poll() is None, (OUT / 'process.log').read_text()[-2000:]
        assert time.monotonic() < deadline, 'Streaming playback timed out'
        try:
            ram = request('cpu/ram/raw')
        except OSError:
            time.sleep(.1)
            continue
        counters = {key: list(struct.unpack_from('<%dI' % count, ram, symbols[key])) for key, (_, count) in names.items()}
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
        if args.route == 'guards' and stage == 0 and frame > 300:
            teleport([0, 0, 29]);stage = 1;events.append({'frame': frame, 'position': [0, 0, 29]})
        elif args.route == 'boss' and stage == 0 and frame > 240:
            scene = yaml.safe_load((P / 'assets/scenes/DungeonGame.epokmap').read_text())
            actor = next(a for a in scene['actors'] if a['name'] == 'Access Key')
            pos = next(c['properties']['position'] for c in actor['components'] if 'position' in c.get('properties', {}))
            teleport(pos);stage = 1;events.append({'frame': frame, 'position': pos})
        elif args.route == 'boss' and stage == 1 and counters['game'][3] and frame > 300:
            teleport([0, 0, 43]);stage = 2;events.append({'frame': frame, 'position': [0, 0, 43]})
        samples.append({'frame': frame, **{key: value for key, value in counters.items() if key not in ('performance', 'peak')},
                        'streamed_chunks': perf['streamed_chunks'], 'skipped': perf['stream_budget_skipped_chunks']})
        if frame >= args.frames:
            break
        time.sleep(.1)
    fps, music, stream, texture = (counters[k] for k in ('fps', 'music', 'stream', 'texture'))
    (OUT / 'samples.json').write_text(json.dumps(samples, indent=2))
    (OUT / 'last-counters.json').write_text(json.dumps(counters, indent=2))
    assert music[11] == 0 and music[12] == 0 and music[10] == 1, music
    minimum_loops = args.minimum_loops if args.minimum_loops is not None else (1 if args.frames >= 4000 else 0)
    assert music[4] >= minimum_loops, f'Replay must cross {minimum_loops} MIDI loop boundaries: {music[4]}'
    assert stream[0] >= 4 and stream[1] >= len(geometry) and stream[4] == stream[5] == 0, stream
    assert vram_bytes and texture[3] == texture[4] == 0, texture
    assert maximum_dropped == maximum_skipped == maximum_failed == 0
    assert (maximum_guards == 3 and counters['combat'][3] > 10) if args.route == 'guards' else (1 in states and counters['game'][5] < 280)
    result = dict(accepted=fps[1] == 0, backend=backend, route=args.route, frames=fps[0],
                  slow_frames=fps[1], max_frame_us=fps[2], minimum_observed_fps=round(1e6 / fps[2], 2),
                  pool_pages=pool, geometry_pages=dict(pages), archive_bytes=len(geometry),
                  streaming=stream, texture=texture, verified_vram_bytes=vram_bytes, music=music,
                  maximum_guards=maximum_guards, shots=counters['combat'][3], boss_health=counters['game'][5],
                  states=sorted(states), max_dropped_triangles=maximum_dropped,
                  max_stream_skipped_chunks=maximum_skipped, max_stream_failed_chunks=maximum_failed,
                  missing_sound_assets=counters['audio'][-1], teleports=events,
                  executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    (OUT / 'samples.json').write_text(json.dumps(samples, indent=2))
    (OUT / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2), flush=True)
    print('Evidence:', OUT, flush=True)
    assert result['accepted'] and not result['missing_sound_assets'], result
finally:
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill();process.wait(timeout=5)
    log.close()
