"""Capture default SDK cameras, retaining and checking both raw 480i fields."""
from __future__ import annotations
import argparse
import copy
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import time
import urllib.request
import uuid
from verify_camera_rigs import ROOT, EDITOR, PRESETS, documents, save_png, write


VRAM_BYTES = 1024 * 512 * 2
PROBE_WORDS = 16


def request(path, data=None):
    with urllib.request.urlopen(urllib.request.Request(
            "http://127.0.0.1:8077/api/v1/" + path, data=data), timeout=5) as response:
        return response.read()


@contextmanager
def paused_snapshot(read_probe, api=request):
    """Pause/acknowledge first; reject any torn RAM or VRAM instead of weaving it."""
    api("execution-flow?function=pause", b"")
    try:
        deadline = time.monotonic() + 3
        while json.loads(api("execution-flow"))["running"]:
            assert time.monotonic() < deadline, "Emulator did not acknowledge pause"
            time.sleep(.01)
        before = read_probe()
        first = api("gpu/vram/raw")
        second = api("gpu/vram/raw")
        after = read_probe()
        assert len(first) == len(second) == VRAM_BYTES, "Incomplete raw VRAM"
        assert before == after, "Probe advanced while emulator was paused"
        assert first == second, "VRAM changed during acknowledged pause"
        yield before, first
    finally:
        api("execution-flow?function=resume", b"")


def field_statistics(raw, width, height):
    """Inspect actual even/odd VRAM rows; no interpolation or field replacement."""
    assert len(raw) == VRAM_BYTES
    assert (width, height) in ((320, 240), (640, 480))
    words = struct.unpack("<524288H", raw)
    result = []
    for parity in range(2):
        selected = [words[y * 1024 + x] & 0x7fff
                    for y in range(parity, height, 2) for x in range(width)]
        colors = {"red": 0, "yellow": 0, "blue": 0}
        for value in selected:
            r, g, b = value & 31, (value >> 5) & 31, (value >> 10) & 31
            # The three existing unlit fixture meshes have distinct channels.
            if r >= 16 and r > 2 * g and r > 2 * b:
                colors["red"] += 1
            if r >= 16 and g >= 16 and b <= 8:
                colors["yellow"] += 1
            if b >= 16 and b > 2 * r and b > g:
                colors["blue"] += 1
        result.append({"parity": parity, "pixels": len(selected),
                       "black_pixels": selected.count(0), "mesh_pixels": colors,
                       "sha256": hashlib.sha256(struct.pack("<" + str(len(selected)) + "H", *selected)).hexdigest()})
    return result


def require_complete_fields(fields):
    for field in fields:
        assert all(n > 0 for n in field["mesh_pixels"].values()), (
            "480i field lacks one or more existing camera fixture meshes", field)


def save_field_png(raw, path, width, height, parity):
    # Half-height diagnostic only. The full image remains the acceptance image.
    pixels = b"".join(raw[y * 2048:(y + 1) * 2048]
                      for y in range(parity, height, 2))
    pixels += bytes(VRAM_BYTES - len(pixels))
    save_png(pixels, path, width, height // 2)


def verify(out: Path, prepare_only: bool, capture_only: bool, width: int = 320):
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"

    def run(*args):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        with (out / "prepare.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    config_path = out / "prepared.json"
    if not capture_only:
        if not project.exists():
            run("--create-project", project, "--template", "basic", "--name", "Camera Preset Views")
        magic = 0x46000000 | (uuid.uuid4().int & 0xffffff)
        source = r'''#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t camera_visual_probe[16]={};}
class EPOK_CLASS(Blueprintable,Id="be7dd6e8-1297-4692-8df7-e706df26a38a") CameraVisualProbe:public epok::ActorComponent {
 uint32_t phase_=0,frame_=0;
public:
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera0{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera1{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera2{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera3{};
 void on_frame(uint32_t)override{
  if(phase_>=4)return;
  const epok::ObjectId cameras[]={camera0,camera1,camera2,camera3};
  auto* camera=epok::active_object_registry->resolve<epok::CameraRig3D>(cameras[phase_]);if(!camera)return;
  if(frame_==0){camera_visual_probe[3]=camera->make_active();camera_visual_probe[13]=camera_visual_probe[14]=camera_visual_probe[15]=0;}
  // Read-only observations. Do not force field parity, change GP1, or stall GPU.
  const uint32_t status=*reinterpret_cast<volatile uint32_t*>(0x1f801814u);
  camera_visual_probe[12]=status;
  camera_visual_probe[13]|=1u<<((status>>13)&1u);
  camera_visual_probe[14]|=1u<<((status>>31)&1u);
  ++camera_visual_probe[15];
  const auto view=camera->sample();auto* p=camera_visual_probe;
  p[0]=MAGIC;p[1]=phase_;p[2]=++frame_;p[4]=uint32_t(view.yaw.raw());p[5]=uint32_t(view.pitch.raw());p[6]=uint32_t(view.distance.raw());
  p[7]=uint32_t(view.position.x.raw());p[8]=uint32_t(view.position.y.raw());p[9]=uint32_t(view.position.z.raw());
  p[10]=epok::active_camera().get()==camera->data();p[11]=uint32_t(view.status);
  if(frame_>=90){frame_=0;++phase_;}
 }
};
'''.replace("MAGIC", str(magic) + "u")
        write(project / "assets/scripts/CameraVisualProbe.hpp", source)
        classes = {c["cpp_name"]: c for c in json.loads(run("--project", project, "--reflect"))["classes"]}
        descriptor = next(project.glob("*.epokproject"))
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        # Preserve the requested display mode and inspect its actual fields.
        # Camera/mesh behavior is unchanged by the observational capture probe.
        manifest["rendering"].update(width=width, height=width * 3 // 4)
        write(descriptor, manifest)
        scene_path = project / manifest["startup_scene"]
        scene = documents.loads(scene_path.read_text(encoding="utf-8"))
        original = copy.deepcopy(scene["actors"][0])
        scene["actors"] = []

        def actor(name, position):
            value = copy.deepcopy(original)
            value.update(id=str(uuid.uuid4()), name=name, **{"class": {"name": "epok::Actor3D", "class_id": classes["epok::Actor3D"]["id"]}}, properties={}, overrides=[])
            value["components"] = [c for c in value["components"] if c.get("root")]
            value["components"][0]["id"] = str(uuid.uuid4())
            value["components"][0]["properties"].update(position=position, rotation=[0, 0, 0])
            return value

        def component(name, cls, props):
            return {"id": str(uuid.uuid4()), "name": name, "class": {"name": cls, "class_id": classes[cls]["id"]}, "properties": props, "overrides": list(props)}

        target = actor("Target", [0, 0, 0])
        scene["actors"].append(target)
        for name, position, scale, color in [
            ("Ground", [0, -.1, 0], [20, .1, 20], [.2, .3, .25]),
            ("Red", [-2, 1, 3], [1, 2, 1], [.9, .2, .15]),
            ("Yellow", [0, 1, 3], [1, 2, 1], [.95, .8, .15]),
            ("Blue", [2, 1, 3], [1, 2, 1], [.15, .4, .9]),
        ]:
            value = actor(name, position)
            value["components"][0]["properties"]["scale"] = scale
            value["components"].append(component("Mesh", "epok::Mesh3DComponent", {"material": {"color": color, "unlit": True}}))
            scene["actors"].append(value)
        write(scene_path, scene)
        refs = {}
        for index, name in enumerate(PRESETS.values()):
            run("--project", project, "--add-actor", classes["epok::" + name]["id"] + ":" + name)
            scene = documents.loads(scene_path.read_text(encoding="utf-8"))
            camera = scene["actors"][-1]
            camera["properties"]["target"] = target["id"]
            camera["overrides"] = list(camera["properties"])
            refs["camera" + str(index)] = camera["id"]
            write(scene_path, scene)
        scene["actors"][0]["components"].append(component("View Capture", "CameraVisualProbe", refs))
        write(scene_path, scene)
        run("--project", project, "--analyze-memory")
        config_path.write_text(json.dumps({"schema_version": 2, "probe_words": PROBE_WORDS, "magic": magic, "width": manifest["rendering"]["width"], "height": manifest["rendering"]["height"]}), encoding="utf-8")
        print("PASS four default camera presets prepared in one scene", flush=True)
    if prepare_only:
        return
    config = json.loads(config_path.read_text(encoding="utf-8"))
    assert config.get("schema_version") == 2 and config.get("probe_words") == PROBE_WORDS, "Prepare the current observational fixture before capture"
    with socket.socket() as probe:
        if os.name == "nt":
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        probe.bind(("127.0.0.1", 8077))
    build = project / ".epok/build"
    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+camera_visual_probe\s*$", (build / "epok.map").read_text(), re.M)[-1], 16) & 0x1fffff
    rows = []
    context = {"capture_schema": 2, "resolution": [config["width"], config["height"]],
               "display_mode": "interlaced" if config["height"] == 480 else "progressive",
               "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
               "exe_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
               "camera_sdk_sha256": hashlib.sha256((build / "camera_rig.hpp").read_bytes()).hexdigest(),
               "raw_fields_verified": False,
               "fixture_sha256": hashlib.sha256((project / "assets/scripts/CameraVisualProbe.hpp").read_bytes()).hexdigest(),
               "frame_clear_sdk_sha256": hashlib.sha256((build / "frame_clear.hpp").read_bytes()).hexdigest(),
               "psyqo_gpu_sha256": hashlib.sha256((ROOT / "third_party/nugget/psyqo/src/gpu.cpp").read_bytes()).hexdigest(),
               "acceptance": "pending raw field capture", "no_field_reconstruction": True}
    (out / "capture-context.json").write_text(json.dumps(context, indent=2) + "\n", encoding="utf-8")
    previous_result = out / "result.json"
    if previous_result.exists():
        previous_result.replace(out / f"previous-result-{time.time_ns()}.json")
    previous_result.write_text(json.dumps(context, indent=2) + "\n", encoding="utf-8")
    with (out / "play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "30"], cwd=ROOT, stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            deadline = time.monotonic() + 240
            for index, (rig, yaw, pitch, distance) in enumerate(zip(PRESETS, [0, 0, 0, 45], [0, 18, 90, 35.264], [0, 5.4, 10, 8])):
                def read_probe():
                    return list(struct.unpack_from("<16I", request("cpu/ram/raw"), address))

                minimum_frame = 10
                for capture_index in range(2):
                    values = None
                    while process.poll() is None and time.monotonic() < deadline:
                        try:
                            sample = read_probe()
                            if sample[0] == config["magic"] and sample[1] == index and minimum_frame <= sample[2] <= 75:
                                values = sample
                                break
                        except (OSError, ValueError):
                            pass
                        time.sleep(.01)
                    assert values is not None, (rig, "Camera phase was not observed")
                    with paused_snapshot(read_probe) as (values, pixels):
                        assert values is not None and values[0] == config["magic"] and values[1] == index and minimum_frame <= values[2] <= 75, (rig, values)
                        assert values[3] == values[10] == 1 and values[11] == 0, (rig, values)
                        assert all(abs(raw - expected * 4096) <= 1 for raw, expected in zip(values[4:7], (yaw, pitch, distance))), (rig, values)
                        stem = rig if capture_index == 0 else rig + "-later"
                        (out / f"{stem}-vram.bin").write_bytes(pixels)
                        save_png(pixels, out / f"{stem}.png", config["width"], config["height"])
                        fields = field_statistics(pixels, config["width"], config["height"])
                        if config["height"] == 480:
                            for parity in range(2):
                                save_field_png(pixels, out / f"{stem}-field{parity}.png", config["width"], config["height"], parity)
                        row = {"rig": rig, "capture": capture_index, "probe": values,
                               "framebuffer": stem + ".png", "raw_vram_sha256": hashlib.sha256(pixels).hexdigest(),
                               "paused_ram_vram_stable": True, "fields": fields,
                               "gpustat_observed_at_game_frame": {"raw": values[12], "display_field_bit13": (values[12] >> 13) & 1,
                                   "drawing_field_bit31": values[12] >> 31, "seen_bit13_mask": values[13], "seen_bit31_mask": values[14], "samples": values[15]},
                               "field_check": "pending" if config["height"] == 480 else "not_applicable"}
                        rows.append(row)
                        # Persist failed captures too; never turn a missing field into
                        # a passing view by dropping alternate rows or retrying it.
                        (out / "capture-observations.json").write_text(json.dumps(rows, indent=2) + "\n", encoding="utf-8")
                        if config["height"] == 480:
                            try:
                                require_complete_fields(fields)
                                row["field_check"] = "passed"
                            except AssertionError as error:
                                row["field_check"] = "failed"
                                row["field_error"] = str(error)
                        (out / "capture-observations.json").write_text(json.dumps(rows, indent=2) + "\n", encoding="utf-8")
                    minimum_frame = values[2] + 8
                    print("CAPTURE default view", rig, "capture", capture_index, "fields", row["field_check"], flush=True)
            assert process.wait(timeout=40) == 0
            context.update(acceptance="failed" if any(row["field_check"] == "failed" for row in rows) else "passed", views=rows,
                           raw_fields_verified=config["height"] == 480 and len(rows) == 8 and all(row["field_check"] == "passed" for row in rows))
            (out / "result.json").write_text(json.dumps(context, indent=2) + "\n", encoding="utf-8")
            assert all(row["field_check"] != "failed" for row in rows), "480i raw field validation failed; see capture-observations.json and retained raw captures"
        finally:
            if process.poll() is None:
                if os.name == "nt":
                    subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=15)
                else:
                    process.terminate()
                process.wait(timeout=15)
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/camera-views")
    parser.add_argument("--width", type=int, choices=(320, 640), default=320,
                        help="320 progressive or 640 raw interlaced acceptance with both fields checked")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--prepare-only", action="store_true")
    group.add_argument("--capture-only", action="store_true")
    args = parser.parse_args()
    verify(args.output.resolve(), args.prepare_only, args.capture_only, args.width)
