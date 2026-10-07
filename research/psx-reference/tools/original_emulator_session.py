"""Owned, isolated original-game session controlled through JSON lines on stdin.

Uses an authenticated local Redux observer, not desktop input.
No game-memory writes, asset conversion or original-file modifications.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import time
import uuid
from PIL import Image


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


class Frames:
    def __init__(self, connection):
        self.connection = connection
        self.stream = connection.makefile("rb")

    def line(self):
        line = self.stream.readline(128)
        if not line.endswith(b"\n"):
            raise ValueError("Incomplete bridge header")
        return line.decode("ascii").split()

    def exact(self, count):
        data = self.stream.read(count)
        if len(data) != count:
            raise ValueError("Incomplete bridge payload")
        return data

    def payload(self, header):
        if len(header) != 2 or header[0] != "CSD1":
            raise ValueError(f"Unexpected data header: {header}")
        size = int(header[1])
        if not 0 <= size <= 32*1024*1024:
            raise ValueError("Invalid data payload size")
        result = self.exact(size)
        if result.startswith(b"ERROR: "):
            raise RuntimeError(result.decode("utf-8", errors="replace"))
        return result

    def frame(self, buttons=0):
        assert 0 <= buttons <= 65535
        self.connection.sendall(f"F {buttons}\n".encode())
        header = self.line()
        if header[0] == "CSD1":
            self.payload(header)  # Consume error replies without desynchronizing TCP.
            raise ValueError("Unexpected data response to frame request")
        if header[0] == "EPKD1":
            assert len(header) == 7
            flags = list(map(int, header[1:]))
            assert all(v in (0, 1) for v in flags[:3]) and 0 <= flags[4] <= 64 and flags[5] in (0, 476)
            self.exact(flags[4]*16 + flags[5])
            header = self.line()
        if header == ["EPKF0"]:
            return None
        assert len(header) == 10 and header[0] == "CSF1", header
        values = list(map(int, header[1:8])) + [float(header[8]),int(header[9])]
        width, height, depth = values[:3]
        assert 0 < width <= 1024 and 0 < height <= 512 and depth in (2, 3)
        data = self.exact(width*height*depth)
        metadata = dict(zip(("width", "height", "depth", "sequence", "buttons", "cycles", "vsyncs", "guest_fps", "pc"), values))
        return metadata, data

    def data(self, command):
        self.connection.sendall((command+"\n").encode())
        return self.payload(self.line())


def save_frame(folder, name, frame):
    target = child(folder, name, ".png")
    assert frame is not None, "No active display"
    metadata, data = frame
    if metadata["depth"] == 3:
        image = Image.frombytes("RGB", (metadata["width"], metadata["height"]), data)
    else:
        rgb = bytearray()
        for pixel, in struct.iter_unpack("<H", data):
            for shift in (0, 5, 10):
                value = (pixel >> shift) & 31
                rgb.append((value << 3) | (value >> 2))
        image = Image.frombytes("RGB", (metadata["width"], metadata["height"]), bytes(rgb))
    image.save(target)
    target.with_suffix(".json").write_text(json.dumps(metadata, indent=2), encoding="utf-8")
    return dict(path=str(target), **metadata)


def child(folder, name, suffix):
    if Path(name).name != name or not name.endswith(suffix):
        raise ValueError("Output must be a plain filename of the expected type")
    target = folder / name
    if target.exists() or (suffix == ".png" and target.with_suffix(".json").exists()):
        raise ValueError("Refusing to overwrite captured evidence")
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--engine", type=Path, default=Path("D:/GitProjects/GameEngines/Epok/EpokEngine"))
    parser.add_argument("--exe", type=Path, required=True, help="User-supplied reference executable; verified by hash")
    parser.add_argument("--cue", type=Path, required=True, help="User-supplied disc cue sheet")
    parser.add_argument("--disc", type=Path, required=True, help="User-supplied disc image for provenance")
    parser.add_argument("--camera-trace", action="store_true", help="Opt-in read-only target/rail camera trace; not a timing baseline")
    args = parser.parse_args()
    folder = args.output.resolve()
    if folder.exists() and any(folder.iterdir()):
        parser.error("Output/session must be new or empty")
    engine = args.engine.resolve()
    exe = args.exe.resolve()
    cue = args.cue.resolve()
    disc = args.disc.resolve()
    assert digest(exe) == "1b93cc563c90966e1d79b750f7d19bc1662859cceeb991775e9e9a22656d71d1"
    emulator = engine / ".tools/redux/pcsx-redux.main"
    bios = engine / ".tools/redux/openbios-fastboot.bin"
    bridge = Path(__file__).with_name("original_observer.lua")
    camera_bridge = Path(__file__).with_name("original_camera_observer.lua") if args.camera_trace else None
    folder.mkdir(parents=True, exist_ok=True)
    token = uuid.uuid4().hex
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(45)
        script = folder / "adapter.lua"
        prefix = f'ORIGINAL_PORT={listener.getsockname()[1]}\nORIGINAL_TOKEN="{token}"\nORIGINAL_FOLDER={json.dumps(folder.as_posix())}\n'
        if camera_bridge:
            # Snapshot the source so a later edit cannot change a live session.
            copy = folder / "camera-observer.lua"
            copy.write_bytes(camera_bridge.read_bytes())
            prefix += f'ORIGINAL_CAMERA_SCRIPT={json.dumps(copy.as_posix())}\n'
        script.write_text(prefix + bridge.read_text(encoding="utf-8"), encoding="utf-8")
        command = [str(emulator), "-portable", str(folder), "-logfile", str(folder / "runtime.log"),
                   "-run", "-stdout", "-fastboot", "-noupdate", "-interpreter", "-softgpu", "-2mb",
                   "-no-gdb", "-no-ui", "-no-pcdrv", "-no-webserver", "-debugger",
                   "-bios", str(bios), "-iso", str(cue), "-loadexe", str(exe), "-dofile", str(script)]
        startup = None
        if os.name == "nt":
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow = 0
        with (folder / "process.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen(command, cwd=folder, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=log, startupinfo=startup,
                                       creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            provenance = dict(scope=__doc__, parent_pid=os.getpid(), pid=process.pid,
                              command=command, inputs={str(p):digest(p) for p in (exe, cue, disc, emulator, bios, bridge)})
            if camera_bridge: provenance["inputs"][str(camera_bridge)] = digest(camera_bridge)
            (folder / "session.json").write_text(json.dumps(provenance, indent=2), encoding="utf-8")
            try:
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(10)
                    frames = Frames(connection)
                    assert frames.line() == ["EPOK", token]
                    print(json.dumps(dict(ready=True, pid=process.pid, folder=str(folder))), flush=True)
                    for line in sys.stdin:
                        try:
                            request = json.loads(line)
                            action = request["action"]
                            assert process.poll() is None, "Owned emulator exited"
                            if action == "stop":
                                try:
                                    frames.frame(0)
                                except Exception:
                                    pass  # Always terminate our owned child, even with a broken observer.
                                break
                            elif action in ("capture", "status"):
                                frame = frames.frame()
                                result = (save_frame(folder, request["name"], frame) if action == "capture"
                                          else None if frame is None else frame[0])
                            elif action == "hold":
                                mask, count = request["buttons"], request["vsyncs"]
                                assert isinstance(mask, int) and 0 <= mask <= 65535 and 1 <= count <= 900
                                first = frames.frame(mask)
                                assert first is not None
                                end = time.monotonic() + 45
                                frame = first
                                try:
                                    while frame[0]["vsyncs"] - first[0]["vsyncs"] < count:
                                        assert time.monotonic() < end, "Input hold timed out"
                                        time.sleep(.01)
                                        frame = frames.frame(mask)
                                        assert frame is not None
                                finally:
                                    frame = frames.frame(0)
                                result = dict(first=first[0], last=frame[0])
                                if "name" in request:
                                    result["capture"] = save_frame(folder, request["name"], frame)
                            elif action in ("ram", "save_state", "schema", "profile_stop"):
                                command,suffix={"ram":("M",".bin"),"save_state":("S",".state"),
                                                "schema":("T",".proto"),"profile_stop":("R",".json")}[action]
                                target = child(folder, request["name"], suffix)
                                data = frames.data(command)
                                if action == "ram": assert len(data) == 2097152
                                target.write_bytes(data)
                                result = dict(path=str(target), bytes=len(data), sha256=digest(target))
                            elif action in ("pause", "resume", "profile_start"):
                                result = frames.data({"pause":"H","resume":"C","profile_start":"P"}[action]).decode()
                            elif action == "load_state":
                                name=request["name"]
                                assert Path(name).name==name and ".." not in name and (folder/name).is_file()
                                result=dict(status=frames.data("L "+name).decode(),
                                            path=str(folder/name), sha256=digest(folder/name))
                            else:
                                raise ValueError("Unknown action")
                            print(json.dumps(dict(ok=True, result=result)), flush=True)
                        except Exception as error:
                            print(json.dumps(dict(ok=False, error=f"{type(error).__name__}: {error}")), flush=True)
            finally:
                if process.poll() is None:
                    process.terminate()
                process.wait(timeout=10)
                print(json.dumps(dict(stopped=True, pid=process.pid, exit_code=process.returncode)), flush=True)


if __name__ == "__main__":
    main()
