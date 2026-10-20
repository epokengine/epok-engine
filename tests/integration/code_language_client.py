"""Shared stdio client for real language service acceptance checks."""
import json
import os
import queue
import subprocess
import threading
import time


def file_uri(path):
    value = path.as_uri()
    if os.name == "nt" and value.startswith("file:///") and value[9:10] == ":":
        value = value[:8] + value[8].lower() + "%3A" + value[10:]
    return value


class Client:
    def __init__(self, executable, root, settings, arguments=()):
        self.process = subprocess.Popen([executable, *arguments], cwd=root, stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        self.settings, self.sequence, self.events = settings, 0, queue.Queue()
        self.notifications, self.stderr, self.documents = [], [], {}
        def read():
            try:
                while True:
                    headers = {}
                    while line := self.process.stdout.readline():
                        if line in (b"\r\n", b"\n"):
                            break
                        key, value = line.decode().split(":", 1)
                        headers[key.lower()] = value.strip()
                    if not headers:
                        return
                    data = self.process.stdout.read(int(headers["content-length"]))
                    self.events.put(json.loads(data))
            except Exception as error:
                self.events.put(error)
        threading.Thread(target=read, daemon=True).start()
        threading.Thread(target=lambda: [self.stderr.append(line.decode(errors="replace").strip())
                                         for line in self.process.stderr], daemon=True).start()
        self.initialize = self.request("initialize", {"processId": os.getpid(), "rootUri": file_uri(root),
            "workspaceFolders": [{"uri": file_uri(root), "name": "Epok Language QA"}],
            "clientInfo": {"name": "Epok acceptance", "version": "1"},
            "initializationOptions": {"editorInfo": {"name": "Epok", "version": "0.7.0"},
                                      "editorPluginInfo": {"name": "Epok Monaco", "version": "0.7.0"}},
            "capabilities": {"workspace": {"configuration": True, "workspaceFolders": True},
                             "textDocument": {"completion": {"completionItem": {"snippetSupport": True}}}}})
        self.send("initialized", {})
        self.send("workspace/didChangeConfiguration", {"settings": settings})

    def write(self, value):
        encoded = json.dumps({"jsonrpc": "2.0", **value}).encode()
        self.process.stdin.write(f"Content-Length: {len(encoded)}\r\n\r\n".encode() + encoded)
        self.process.stdin.flush()

    def send(self, method, params):
        self.write({"method": method, "params": params})

    def request(self, method, params=None, timeout=30):
        self.sequence += 1
        request_id = self.sequence
        self.write({"id": request_id, "method": method, "params": params or {}})
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            value = self.events.get(timeout=max(.1, deadline - time.monotonic()))
            if isinstance(value, Exception):
                raise value
            if "method" in value:
                if "id" in value:
                    if value["method"] == "workspace/configuration":
                        result = []
                        for item in value["params"]["items"]:
                            setting = self.settings
                            for key in item.get("section", "").split("."):
                                if key:
                                    setting = setting.get(key) if isinstance(setting, dict) else None
                            result.append(setting)
                    else:
                        result = None
                    self.write({"id": value["id"], "result": result})
                else:
                    self.notifications.append(value)
            elif value.get("id") == request_id:
                assert "error" not in value, value
                return value.get("result")
        raise TimeoutError(method)

    def complete(self, path, text, expected):
        uri = file_uri(path)
        self.send("textDocument/didOpen", {"textDocument": {"uri": uri,
                  "languageId": "cpp" if path.suffix == ".hpp" else "lua", "version": 1, "text": text}})
        lines = text.split("\n")
        deadline = time.monotonic() + 20
        labels = []
        while time.monotonic() < deadline:
            result = self.request("textDocument/completion", {"textDocument": {"uri": uri},
                "position": {"line": len(lines) - 1, "character": len(lines[-1])}, "context": {"triggerKind": 1}})
            items = result if isinstance(result, list) else (result or {}).get("items", [])
            labels = [item["label"] for item in items]
            if any(expected in label for label in labels):
                return labels
            time.sleep(.2)
        raise AssertionError({"expected": expected, "labels": labels, "stderr": self.stderr[-10:]})

    def diagnostics(self, path, text, timeout=20):
        uri = file_uri(path)
        self.notifications = [n for n in self.notifications if not (
            n["method"] == "textDocument/publishDiagnostics" and n["params"]["uri"] == uri)]
        version = self.documents.get(uri, 0) + 1
        self.documents[uri] = version
        if version == 1:
            self.send("textDocument/didOpen", {"textDocument": {
                "uri": uri, "languageId": "lua", "version": version, "text": text}})
        else:
            self.send("textDocument/didChange", {"textDocument": {"uri": uri, "version": version},
                                               "contentChanges": [{"text": text}]})
        previous = None
        stable_since = time.monotonic()
        deadline = stable_since + timeout
        while time.monotonic() < deadline:
            # Drain notifications through a normal request, answering the
            # server's configuration requests while background diagnostics run.
            self.request("textDocument/hover", {"textDocument": {"uri": uri},
                         "position": {"line": 0, "character": 0}})
            published = [n["params"]["diagnostics"] for n in self.notifications
                         if n["method"] == "textDocument/publishDiagnostics" and n["params"]["uri"] == uri
                         and n["params"].get("version", version) == version]
            current = published[-1] if published else None
            if current != previous:
                stable_since = time.monotonic()
                previous = current
            if current is not None and time.monotonic() - stable_since >= 2:
                return current
            time.sleep(.2)
        raise TimeoutError(f"diagnostics: {uri}")

    def close(self):
        try:
            self.request("shutdown", timeout=3)
            self.send("exit", {})
            self.process.wait(timeout=5)
        except Exception:
            self.process.terminate()
            self.process.wait(timeout=5)
