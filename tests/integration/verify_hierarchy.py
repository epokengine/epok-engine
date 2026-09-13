"""Compile isolated scenes and compare actual PSX VRAM for parenting and materials.

Pass --exe if the main editor executable is open and an alternate target was built.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import copy
import json
import pathlib
import socket
import struct
import subprocess
import time
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / 'artifacts'
ARTIFACTS.mkdir(exist_ok=True)
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, 'CREATE_NO_WINDOW') else 0


def entity(name, kind='Mesh', position=(0, 0, 0), rotation=(0, 0, 0), scale=(1, 1, 1), parent=None):
    return dict(name=name, kind=kind, position=position, rotation=rotation, scale=scale,
                parent=parent, material=dict(color=[1, 0, 0], unlit=True))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--exe', type=pathlib.Path, default=ROOT / 'target/debug/epok-editor.exe')
    args = parser.parse_args()
    # Refuse to attach to or interrupt another emulator.
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 8077))
    folder = ROOT / '.epok' / ('hierarchy-verify-' + str(time.time_ns()))
    subprocess.run([str(args.exe.resolve()), '--create-project', str(folder), '--template', 'sample'],
                   check=True, creationflags=FLAGS)
    video_path=project_manifest(folder)
    video=documents.loads(video_path.read_text());video['rendering']=dict(width=320,height=240)
    documents.write_text(video_path, json.dumps(video))  # Fixed pixel-reference fixture.
    camera = entity('Main Camera', 'Camera', (0, 3, -6), (18, 0, 0))
    flat = [camera, entity('Red child', position=(0, 0.5, 0))]
    nested = [camera, entity('Red child', position=(0, 1, 0), rotation=(0, 0, -90), scale=(0.5,)*3, parent=2),
              entity('Rotated parent', 'Empty', (1, 0.5, 0), (0, 0, 90), (2,)*3, 3),
              entity('Root pivot', 'Empty', (1, 0, 0))]

    def write_scene(entities):
        documents.write_text(folder / 'assets/scenes/SampleScene.epokmap',
            json.dumps(dict(version=1, name='Hierarchy verification', entities=entities), indent=2))

    def request(path, post=False):
        req = urllib.request.Request('http://127.0.0.1:8077/api/v1/' + path, data=b'' if post else None)
        with urllib.request.urlopen(req, timeout=2) as response:
            return response.read()

    def red_pixels(raw):
        # Both framebuffer pages are in VRAM. Use the first page consistently after several frames.
        return {(x, y) for y in range(240) for x in range(320)
                if (pixel := struct.unpack_from('<H', raw, (y*1024+x)*2)[0]) & 31 > 24
                and (pixel >> 5) & 31 < 3 and (pixel >> 10) & 31 < 3}

    def capture(entities, label, animated=False):
        write_scene(entities)
        with (ARTIFACTS / f'hierarchy-{label}.log').open('w') as log:
            process = subprocess.Popen([str(args.exe.resolve()), '--project', str(folder), '--play-psx', '--stop-after', '7'],
                                       stdout=log, stderr=log, creationflags=FLAGS)
            try:
                deadline = time.monotonic()+30
                while True:
                    if process.poll() is not None:
                        raise AssertionError(f'Emulator exited; see hierarchy-{label}.log')
                    try:
                        if json.loads(request('execution-flow'))['running']:
                            break
                    except (OSError, ValueError):
                        pass
                    if time.monotonic()>deadline:
                        raise TimeoutError('Emulator did not start')
                    time.sleep(0.1)
                time.sleep(1.5)
                request('execution-flow?function=pause', True)
                raw = request('gpu/vram/raw')
                first = red_pixels(raw)
                assert len(first)>100, f'No red material rendered: {len(first)} pixels'
                if animated:
                    request('execution-flow?function=resume', True)
                    time.sleep(0.6)
                    request('execution-flow?function=pause', True)
                    second = red_pixels(request('gpu/vram/raw'))
                    assert len(first ^ second)>100, 'Parent C++ script did not move the child'
                assert process.wait(timeout=12)==0
                return first
            finally:
                if process.poll() is None:
                    process.wait(timeout=40)

    a = capture(flat, 'flat')
    b = capture(nested, 'nested')
    overlap = len(a & b) / len(a | b)
    assert overlap>0.95, f'Nested and equivalent world transforms differ: IoU {overlap:.3f}'
    camera_scene = copy.deepcopy(nested)
    camera_scene[0] = entity('Main Camera', 'Camera', (1.5, 1, -3), (18, 0, -90), (0.5,)*3, 4)
    camera_scene.append(entity('Camera rig', 'Empty', (2, 0, 0), (0, 0, 90), (2,)*3))
    c = capture(camera_scene, 'camera')
    camera_overlap = len(a & c) / len(a | c)
    assert camera_overlap>0.95, f'Parented camera inverse differs: IoU {camera_overlap:.3f}'
    nested[3]['script'] = dict(name='Spinner', properties=dict(speed=90))
    capture(nested, 'animated', True)
    # Keep a static, inspectable fixture for screenshots without changing the user's scene.
    nested[3].pop('script')
    write_scene(nested)
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 8077))
    report = (f'PASS native parenting (two levels, rotation, scale, translation): framebuffer IoU {overlap:.3f}\n'
              f'PASS parented camera: framebuffer IoU {camera_overlap:.3f}\n'
              'PASS unlit red material in actual PSX VRAM\n'
              'PASS C++ parent animation moves child\nPASS owned emulator cleanup\n'
              f'Fixture: {folder}\n')
    documents.write_text(ARTIFACTS / 'hierarchy-verification.txt', report)
    print(report)


if __name__ == '__main__':
    main()
