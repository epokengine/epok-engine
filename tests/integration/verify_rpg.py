"""Actual MIPS/PCSX-Redux acceptance: textures, cutout/depth/blends, bank reuse,
bounded particles, input edges and measured time. Uses an isolated demo copy;
reads generated symbols from RAM and native VRAM rather than editor previews.
Run after cargo build; needs the configured MIPS toolchain and emulator.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse, copy, hashlib, json, pathlib, re, shutil, socket, struct
import subprocess, time, urllib.request, uuid, zlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
ART = ROOT / "artifacts"
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0

def request(path, post=False):
    req = urllib.request.Request("http://127.0.0.1:8077/api/v1/" + path, data=b"" if post else None)
    with urllib.request.urlopen(req, timeout=4) as response:
        return response.read()

def png(width, height, rgba):
    def chunk(name, data):
        return struct.pack(">I", len(data)) + name + data + struct.pack(">I", zlib.crc32(name + data) & 0xffffffff)
    rows = b"".join(b"\0" + rgba[y * width * 4:(y + 1) * width * 4] for y in range(height))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")

PROBE_HPP = r'''#pragma once
#include "epok.hpp"
class AcceptanceProbe final:public epok::Behaviour {
public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override;
void frame_update(epok::Transform&,uint32_t)override;
};
'''
PROBE_CPP = r'''#include "AcceptanceProbe.hpp"
using namespace epok;
extern "C" { volatile uint32_t rpg_probe[24]={}; }
static EntityHandle stale;
static void check(bool value,unsigned slot){if(!value)rpg_probe[slot]=rpg_probe[slot]+1;}
void AcceptanceProbe::start(Transform&){
    rpg_probe[0]=0x52504731;rpg_probe[3]=rpg_probe[3]+1;
    if(rpg_probe[3]>1)check(!stale.get(),4);
    stale=handle(&entity());
    // Exercise slot reuse repeatedly without allocating more than one runtime slot.
    for(int i=0;i<100;++i){auto* e=create_entity("temporary");check(e!=nullptr,5);if(!e)break;auto h=handle(e);check(destroy_entity(e),5);check(!h.get(),5);}
    Input input_test;input_test.sample(0,true,1u<<uint8_t(Button::Cross));input_test.begin_tick();
    check(input_test.held(Button::Cross)&&input_test.pressed(Button::Cross),6);
    input_test.end_tick();input_test.begin_tick();check(input_test.held(Button::Cross)&&!input_test.pressed(Button::Cross),6);
    input_test.sample(0,true,0);input_test.begin_tick();check(input_test.released(Button::Cross)&&!input_test.held(Button::Cross),6);
    Time clock;clock.reset(100);check(clock.advance(1000100)==Time::max_steps&&clock.dropped_steps==52,7);
    clock.set_paused(true);check(clock.advance(2000100)==0,7);clock.set_paused(false);check(clock.advance(2016767)==1,7);
    rpg_probe[8]=rpg_probe[8]+1;
    auto* main_camera=find_entity("Camera");check(set_active_camera(main_camera),17);check(active_camera().get()==main_camera,17);
    Fixed point[3]={1.0,0.0,5.0},screen[2];check(camera_project(point,screen),17);check(screen[0].raw()==384*4096&&screen[1].raw()==240*4096,17);
    auto* temporary_camera=create_entity("temporary camera");check(temporary_camera!=nullptr,17);
    if(temporary_camera){temporary_camera->add<CameraSettings>();check(set_active_camera(temporary_camera),17);check(active_camera().get()==temporary_camera,17);destroy_entity(temporary_camera);check(active_camera().get()==main_camera,17);}
}
void AcceptanceProbe::frame_update(Transform&,uint32_t){
    unsigned f=rpg_probe[1];rpg_probe[1]=f+1;
    unsigned phase=f/90;rpg_probe[2]=phase;
    if(auto* camera=find_entity("Camera"))camera->camera_settings.field_of_view=phase==4?Fixed(60):Fixed(90);
    if(auto* e=find_entity("Palette probe")){auto& a=e->palette_animator;a.enabled=phase==5;a.speed=Fixed(0);a.offset=1;}
    auto* sprite=find_entity("Depth sprite");if(sprite){
        sprite->transform.position[2]=(phase==1||phase==2)?Fixed(7):Fixed(3);
        sprite->sprite.depth_bias=phase==2?-24:0;
        sprite->sprite.blend=phase==3?BlendMode::Average:BlendMode::Cutout;
    }
    if(f==360){for(const char* name:{"Burst A","Burst B","Burst C"})if(auto* e=find_entity(name))e->particle_emitter.burst(256);}
    if(phase==4){rpg_probe[9]=particle_stats.peak;rpg_probe[10]=particle_stats.dropped;rpg_probe[11]=particle_stats.spawned;}
    if(phase==4){Fixed point[3]={1.0,0.0,5.0},screen[2];check(camera_project(point,screen)&&screen[0].raw()>425*4096&&screen[0].raw()<435*4096,17);rpg_probe[18]=resource_usage.sprite_estimated_pixels;rpg_probe[19]=resource_usage.vram_texture_words;rpg_probe[20]=resource_usage.vram_palette_words;rpg_probe[21]=resource_usage.resident_texture_bytes;}
    if(f>=450&&f<750&&(f-450)%30==0){check(request_scene(size_t(current_scene()==0?1:0)),12);rpg_probe[13]=rpg_probe[13]+1;}
    if(f>=780){rpg_probe[2]=99;rpg_probe[14]=current_scene();rpg_probe[15]=particle_stats.alive;}
}
void AcceptanceProbe::update(Transform&,Fixed){rpg_probe[16]=rpg_probe[16]+1;}
'''

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=pathlib.Path, default=ROOT / "target/debug/epok-editor.exe")
    args = parser.parse_args()
    with socket.socket() as available:
        available.bind(("127.0.0.1", 8077))
    ART.mkdir(exist_ok=True)
    folder = ROOT / ".epok" / ("rpg-verify-" + str(time.time_ns()))
    shutil.copytree(ROOT / "examples/rpg-2-5d-demo", folder,
                    ignore=shutil.ignore_patterns(".epok", "UserSettings", "exports", "artifacts"))
    documents.write_text(ART / "rpg-verification-project.txt", str(folder))
    def save(path, value):
        documents.write_text(folder / path, json.dumps(value, indent=2), encoding="utf-8")
    ident = str(uuid.uuid4())
    rgba = bytes(c for y in range(8) for x in range(8)
                 for c in ((255, 0, 0, 255) if 2 <= x < 6 and 2 <= y < 6 else (0, 0, 0, 0)))
    image = png(8, 8, rgba)
    source = "assets/textures/probe.png"
    (folder / source).write_bytes(image)
    meta = json.dumps(dict(version=2, id=ident, kind="Texture", importer_version=1, source=source,
                           source_hash=hashlib.sha256(image).hexdigest(), settings={"type": "Texture"})).encode()
    (folder / "assets/textures/probe.epokasset").write_bytes(b"EPOKAS01" + struct.pack("<II", len(meta), len(image)) + meta + image)
    palette_id=str(uuid.uuid4());palette_png=png(2,1,bytes([255,0,0,255,0,0,255,255]))
    palette_source="assets/textures/palette-probe.png";(folder/palette_source).write_bytes(palette_png)
    palette_meta=json.dumps(dict(version=2,id=palette_id,kind="Texture",importer_version=1,source=palette_source,
                                source_hash=hashlib.sha256(palette_png).hexdigest(),settings={"type":"Texture"})).encode()
    (folder/"assets/textures/palette-probe.epokasset").write_bytes(b"EPOKAS01"+struct.pack("<II",len(palette_meta),len(palette_png))+palette_meta+palette_png)
    scripts = folder / "assets/scripts"
    documents.write_text(scripts / "AcceptanceProbe.hpp", PROBE_HPP)
    documents.write_text(scripts / "AcceptanceProbe.cpp", PROBE_CPP)
    save("assets/scripts/AcceptanceProbe.epokscript", dict(name="AcceptanceProbe", properties=[]))
    def e(name, kind="Empty", position=(0, 0, 0), **extra):
        return dict(name=name, kind=kind, position=list(position), rotation=[0, 0, 0], scale=[1, 1, 1], **extra)
    camera = e("Camera", "Camera")
    obstacle = e("Occluder", "Mesh", (0, 0, 5), material=dict(color=[0, 1, 0], unlit=True))
    obstacle["scale"] = [3, 3, .5]
    sprite = e("Depth sprite", position=(0, 0, 3), sprite=dict(texture=ident, size=[2, 2], pivot=[.5, .5], orientation="Fixed", unlit=True))
    control = e("Probe", script=dict(name="AcceptanceProbe"))
    entities = [camera, obstacle, sprite, control]
    for name in ["Burst A", "Burst B", "Burst C"]:
        entities.append(e(name, position=(8, 8, 5), particle_emitter=dict(continuous=False, play_on_start=False, max_particles=128, lifetime=.2, burst=1, sprite=dict(texture=ident, unlit=True))))
    canvas = len(entities)
    entities += [e("Canvas", canvas={}), e("Extended glyphs", parent=canvas, rect=dict(anchor_min=[0, 1], anchor_max=[0, 1], pivot=[0, 1], position=[8, -8], size=[400, 32]), text=dict(text="Extended glyphs:\náéíóúüñÁÉÍÓÚÜÑ¿¡", wrap=True))]
    entities.append(e("Palette probe",palette_animator=dict(enabled=False,texture=palette_id,first=1,last=2,speed=4)))
    scene = dict(version=1, name="Courtyard", entities=entities)
    save("assets/scenes/Courtyard.epokmap", scene)
    second = copy.deepcopy(scene);second["name"] = "Night"
    save("assets/scenes/Night.epokmap", second)
    captures = {};values = None
    log_path = ART / "rpg-verification.log"
    with log_path.open("w") as log:
        process = subprocess.Popen([str(args.exe.resolve()), "--project", str(folder), "--play-psx", "--stop-after", "40"], stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 70
            address = None
            while time.monotonic() < deadline:
                assert process.poll() is None, log_path.read_text(errors="replace")
                try:
                    state = json.loads(request("execution-flow"))
                    if not state["running"]:request("execution-flow?function=resume", True)
                    if address is None:
                        symbols = (folder / ".epok/build/epok.map").read_text()
                        found = re.search(r"0x([0-9a-fA-F]+)\s+rpg_probe\b", symbols)
                        assert found, "Probe symbol absent from exported MIPS image"
                        address = int(found.group(1), 16) & 0x1fffff
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<24I", ram, address)
                    if values[0] != 0x52504731:time.sleep(.05);continue
                    phase = values[2]
                    # Capture a stable rendered frame, not the first frame after a phase change.
                    if phase in range(7) and phase not in captures and values[1] % 90 > 12:
                        request("execution-flow?function=pause", True)
                        vram = request("gpu/vram/raw")
                        captures[phase] = vram
                        (ART / f"rpg-depth-{phase}.vram").write_bytes(vram)
                        pixels = bytearray()
                        for y in range(480):
                            for x in range(640):
                                p = struct.unpack_from("<H", vram, (y * 1024 + x) * 2)[0]
                                pixels.extend(((p & 31) * 255 // 31, ((p >> 5) & 31) * 255 // 31, ((p >> 10) & 31) * 255 // 31, 255))
                        (ART / f"rpg-depth-{phase}.png").write_bytes(png(640, 480, pixels))
                        request("execution-flow?function=resume", True)
                    if phase == 99:
                        request("execution-flow?function=pause", True)
                        values = struct.unpack_from("<24I", request("cpu/ram/raw"), address)
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(.06)
            assert values and values[2] == 99, ("Probe did not finish", values)
            assert set(captures) == set(range(7)), ("Missing phase captures", captures.keys())
            def pixel(phase, x=320, y=240):
                return struct.unpack_from("<H", captures[phase], (y * 1024 + x) * 2)[0] & 0x7fff
            assert pixel(0) == 31, ("Sprite in front", pixel(0))
            assert pixel(1) == 31 << 5, ("Sprite behind mesh", pixel(1))
            assert pixel(2) == 31, ("Depth bias", pixel(2))
            mixed = pixel(3)
            assert 12 <= (mixed & 31) <= 17 and 12 <= ((mixed >> 5) & 31) <= 17 and mixed >> 10 == 0, ("Average blend", mixed)
            assert pixel(0, 230, 160) == 31 << 5, "Transparent sprite border covered background"
            assert all(values[i] == 0 for i in [4, 5, 6, 7, 12, 17]), ("Runtime checks failed", values)
            def red_width(phase):
                xs=[x for x in range(640) if pixel(phase,x,240)==31]
                return max(xs)-min(xs)+1
            assert 1.65 < red_width(4)/red_width(0) < 1.82, ("Camera60degree FOV",red_width(0),red_width(4))
            assert values[18]>10000 and values[19]==34 and values[20]==512 and values[21]==1092,("Resource profiler",values)
            clut_offset=((480+sorted([ident,palette_id]).index(palette_id))*1024+640)*2
            palettes={phase:struct.unpack_from("<3H",captures[phase],clut_offset) for phase in [0,5,6]}
            assert palettes[0][0]==palettes[5][0]==palettes[6][0]==0,("Palette transparency",palettes)
            assert palettes[5][1:]==palettes[0][1:][::-1] and palettes[6]==palettes[0],("Palette rotation/disable restore",palettes)
            assert values[3] == 11 and values[13] == 10 and values[8] == 11, ("Bank transitions", values)
            assert values[9] == 256 and values[10] >= 512 and values[11] == 256, ("Particle global/per-emitter limits", values)
            assert values[15] == 0 and values[16] > 0, ("Finite particles/simulation", values)
            report = dict(passed=True, native_resolution=[640, 480], probe=list(values),
                          center_pixels={str(p):pixel(p) for p in captures},
                          camera=dict(fov_90_sprite_width=red_width(0), fov_60_sprite_width=red_width(4),
                                      measured_focal_ratio=red_width(4)/red_width(0), api_errors=values[17]),
                          resources=dict(sprite_estimated_pixels=values[18], texture_vram_words=values[19],
                                         palette_vram_words=values[20], resident_texture_bytes=values[21]),
                          lifecycle=dict(scene_starts=values[3], scene_changes=values[13],
                                         slot_cycles=values[8]*100, handle_errors=values[4]+values[5]),
                          particles=dict(peak=values[9], dropped=values[10], spawned=values[11], final_alive=values[15]),
                          palettes=palettes,
                          fixture=str(folder))
            documents.write_text(ART / "rpg-verification.json", json.dumps(report, indent=2))
            print("PASS native cutout/depth/blend; camera60degree FOV/active camera/projection; resource counters; CLUT rotation/disable restore;10scene switches/1100slot cycles/stale handles;256particle cap/expiry/drop;target input/time probes", flush=True)
            assert process.wait(timeout=50) == 0
        finally:
            if process.poll() is None:
                process.wait(timeout=55)

if __name__ == "__main__":
    main()
