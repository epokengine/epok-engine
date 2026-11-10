"""Shared PSX execution/export evidence for gameplay helper integration fixtures."""
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
from verify_camera_rigs import ROOT, EDITOR


def binary_reuse_matches(out, previous, current):
    """Reuse played native evidence only after an identical current-source rebuild."""
    path = out / "binary-equivalence-reuse.json"
    if not path.exists():
        return False
    proof = json.loads(path.read_text(encoding="utf-8"))
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    observation = json.loads((out / "native_cpp-observation.json").read_text(encoding="utf-8"))
    return (proof["previous_common_contract_sha256"] == previous
            and proof["current_common_contract_sha256"] == current
            and proof["current_editor_sha256"] == sha(EDITOR)
            and proof["build_exit_code"] == 0
            and proof["played_exe_sha256"] == observation["played_exe_sha256"]
            and proof["regenerated_exe_sha256"] == proof["played_exe_sha256"]
            and all(sha(out / name) == digest for name, digest in proof["evidence_sha256"].items()))


def run_modes(out, project, domain, modes, symbol, magic, check, provider_count=3, trace=None, trace_ring=None, stop_after_seconds=30, forbidden_runtime_logs=None):
    assert provider_count in (1, 3)
    assert not trace_ring or trace
    assert 30 <= stop_after_seconds <= 600
    results = []
    for mode in modes or ("native_cpp", "vm_bytecode", "vm_source"):
        import epok_documents as documents
        descriptor = next(project.glob("*.epokproject"))
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = mode
        documents.write_text(descriptor, documents.dumps(manifest))
        build = project / ".epok/build"
        expected_mode = {"native_cpp": 0, "vm_bytecode": 1, "vm_source": 2}[mode]
        expected = magic + expected_mode
        with socket.socket() as probe:
            if os.name == "nt":
                probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
            probe.bind(("127.0.0.1", 8077))
        values = None
        observed_hash = None
        address = None
        captured_trace = []
        with (out / (mode + "-play.log")).open("w", encoding="utf-8") as log:
            # Play already builds, certifies and validates the runtime budget.
            process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", str(stop_after_seconds), "--stop-on-stdin"],
                                       cwd=ROOT, stdin=subprocess.PIPE, stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            try:
                deadline = time.monotonic() + 600
                while process.poll() is None and time.monotonic() < deadline:
                    console_log = (out / (mode + "-play.log")).read_text(encoding="utf-8", errors="replace")
                    for pattern in forbidden_runtime_logs or ():
                        found = re.search(pattern, console_log)
                        assert found is None, (domain, mode, "Forbidden external runtime transport", found.group(0))
                    fatal = re.search(r"(?:^|\n)(?:Abort at [^\n]*|Lua arena exhausted:[^\n]*)", console_log)
                    assert fatal is None, (domain, mode, fatal.group(0).strip())
                    try:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=1) as response:
                            ram = response.read()
                        current_address = int(re.findall(r"0x([0-9a-fA-F]+)\s+" + re.escape(symbol) + r"\s*$", (build / "epok.map").read_text(), re.M)[-1], 16) & 0x1fffff
                        execution = int(re.search(r"-DEPOK_LUA_MODE=(\d+)", (build / "sources.mk").read_text())[1])
                        assert execution == expected_mode, (mode, execution)
                        current_hash = hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest()
                        if observed_hash is None:
                            observed_hash, address = current_hash, current_address
                        assert current_hash == observed_hash and current_address == address
                        values = list(struct.unpack_from(f"<{provider_count * 8}I", ram, address))
                        if trace_ring:
                            trace_symbol, total_words = trace
                            counter_symbol, capacity, row_words = trace_ring
                            assert total_words % row_words == 0
                            map_text = (build / "epok.map").read_text()
                            def trace_address(name):
                                return int(re.findall(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\s*$", map_text, re.M)[-1], 16) & 0x1fffff
                            completed = struct.unpack_from('<I', ram, trace_address(counter_symbol))[0]
                            first = len(captured_trace) // row_words
                            assert first <= completed <= total_words // row_words, (first, completed)
                            assert completed - first <= capacity, ('Trace reader lost frames', first, completed, capacity)
                            start = trace_address(trace_symbol)
                            for frame in range(first, completed):
                                row = list(struct.unpack_from(f'<{row_words}I', ram, start + (frame % capacity) * row_words * 4))
                                assert row[0] == frame, ('Trace slot overwritten before capture', frame, row[0])
                                captured_trace.extend(row)
                        if all(values[i * 8] == expected for i in range(provider_count)):
                            if trace_ring:
                                assert len(captured_trace) == trace[1], ('Incomplete trace', len(captured_trace), trace[1])
                            break
                    except (OSError, ValueError, IndexError):
                        pass
                    time.sleep(.05)
                assert values is not None, (domain, mode, process.poll(), (out / (mode + "-play.log")).read_text(encoding="utf-8")[-6000:])
                assert all(values[:8] == values[i*8:(i+1)*8] for i in range(provider_count)), (domain, mode, values)
                check(values[:8], expected)
                process.stdin.write(b"stop\n")
                process.stdin.flush()
                assert process.wait(timeout=60) == 0
                assert hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest() == observed_hash
            finally:
                if trace and values is not None:
                    trace_symbol, trace_words = trace
                    found = re.findall(r"0x([0-9a-fA-F]+)\s+"+re.escape(trace_symbol)+r"\s*$",(build/"epok.map").read_text(),re.M)
                    assert found, trace_symbol
                    trace_address = int(found[-1],16)&0x1fffff
                    words = captured_trace if trace_ring else list(struct.unpack_from(f"<{trace_words}I",ram,trace_address))
                    (out/(mode+"-"+trace_symbol+".json")).write_text(json.dumps(words)+"\n",encoding="utf-8")
                traces=re.findall(r"0x([0-9a-fA-F]+)\s+character_animation_trace\s*$",(build / "epok.map").read_text(),re.M) if (build / "epok.map").exists() else []
                if traces and values is not None:
                    trace_address=int(traces[-1],16)&0x1fffff
                    trace=list(struct.unpack_from("<1800I",ram,trace_address))
                    (out / (mode+"-trace.json")).write_text(json.dumps([trace[i*600:(i+1)*600] for i in range(3)])+"\n",encoding="utf-8")
                (out / (mode + "-observation.json")).write_text(json.dumps({"expected": expected, "probe_address": address,
                    "values": values, "played_exe_sha256": observed_hash}, indent=2) + "\n", encoding="utf-8")
                if process.poll() is None:
                    if os.name == "nt":
                        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=15)
                    else:
                        process.terminate()
                    process.wait(timeout=15)
        provider_values = {"cpp": values[:8], "blueprint": values[8:16], "lua": values[16:]} if provider_count == 3 else {"sample": values}
        results.append({"domain": domain, "mode": mode, **provider_values,
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "exe_sha256": observed_hash,
                        "ram_static_bytes": json.loads((build / "memory-report.json").read_text())["ram"]["used"]})
        (out / "result.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", domain, mode, "three provider records and actual playback agree" if provider_count == 3 else "template route and actual playback", flush=True)


def export_probe(out, project, run, headers):
    export = Path(run("--project", project, "--export-psx", label="standalone-export.log"))
    for header in headers:
        assert (export / header).read_bytes() == (ROOT / "runtime" / header).read_bytes()
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                             "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                            cwd=export, env=env, capture_output=True, text=True, timeout=900)
    (out / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stdout + result.stderr
    original = (project / ".epok/build/epok.ps-exe").read_bytes()
    assert (export / "epok.ps-exe").read_bytes() == original
    (out / "standalone-result.json").write_text(json.dumps({"identical": True, "export": str(export),
        "sha256": hashlib.sha256(original).hexdigest()}, indent=2) + "\n", encoding="utf-8")
    print("PASS standalone rebuild, byte-identical", flush=True)
