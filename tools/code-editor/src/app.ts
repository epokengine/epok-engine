import * as monaco from "monaco-editor/esm/vs/editor/editor.api.js";
import "monaco-editor/esm/vs/editor/editor.all.js";
import "monaco-editor/esm/vs/basic-languages/lua/lua.contribution.js";
import "monaco-editor/esm/vs/basic-languages/cpp/cpp.contribution.js";
import { CopilotChat } from "./chat";

type Json = any;
type Config = {
    lua_ls: string;
    clangd: string;
    copilot: string;
    copilot_enabled: boolean;
};
type Source = {
    uri: string;
    path: string;
    language: string;
    text: string;
    savedText?: string;
    version: number;
    readonly: boolean;
};
type Buffer = Source & {
    model: monaco.editor.ITextModel;
    saved: string;
    view: monaco.editor.ICodeEditorViewState | null;
};
type Entry = Pick<Source, "uri" | "path" | "language">;
declare global {
    interface Window {
        ipc: { postMessage(message: string): void };
        epokReceive: (value: Json) => void;
        MonacoEnvironment: Json;
    }
}
window.MonacoEnvironment = {
    getWorker: () =>
        new Worker(new URL("./editor.worker.js", location.href), {
            type: "module",
        }),
};
const $ = <T extends HTMLElement = HTMLElement>(id: string) =>
    document.getElementById(id) as T;
const pending = new Map<
    string,
    {
        resolve: (value: Json) => void;
        reject: (error: Error) => void;
        timer: number;
    }
>();
let sequence = 0;
function notify(method: string, params: Json = {}) {
    window.ipc.postMessage(JSON.stringify({ method, params }));
}
function rpc(method: string, params: Json = {}): Promise<Json> {
    const id = `web-${++sequence}`;
    return new Promise((resolve, reject) => {
        const timer = window.setTimeout(() => {
            pending.delete(id);
            reject(new Error(`${method} timed out`));
        }, 60000);
        pending.set(id, { resolve, reject, timer });
        window.ipc.postMessage(JSON.stringify({ id, method, params }));
    });
}
function message(value: string, error = false) {
    $("message").textContent = value;
    $("message").style.color = error ? "#ffb5af" : "";
}
function report(error: unknown) {
    // Monaco cancels pending UI operations when models or panes change.
    if (
        error instanceof Error &&
        error.name === "Canceled" &&
        error.message === "Canceled"
    )
        return;
    message(error instanceof Error ? error.message : String(error), true);
}
let serverDialogQueue = Promise.resolve();
function showServerMessage(name: string, params: Json): Promise<Json> {
    const result = serverDialogQueue.then(
        () =>
            new Promise<Json>((resolve) => {
                const dialog = $<HTMLDialogElement>("server-message");
                $("server-message-title").textContent =
                    name === "copilot" ? "GitHub Copilot" : name;
                $("server-message-text").textContent = params?.message ?? "";
                const actions = $("server-message-actions");
                actions.replaceChildren();
                let selected: Json = null;
                for (const action of [...(params?.actions ?? []), null]) {
                    const button = document.createElement("button");
                    button.textContent = action?.title ?? "Dismiss";
                    button.onclick = () => {
                        selected = action;
                        dialog.close();
                    };
                    actions.append(button);
                }
                dialog.onclose = () => {
                    dialog.onclose = null;
                    resolve(selected);
                };
                dialog.showModal();
            }),
    );
    serverDialogQueue = result.then(
        () => undefined,
        () => undefined,
    );
    return result;
}
window.addEventListener("unhandledrejection", (event) => {
    event.preventDefault();
    report(event.reason);
});
window.addEventListener("error", (event) => report(event.message));

const editor = monaco.editor.create($("editor"), {
    theme: "vs-dark",
    automaticLayout: true,
    fontFamily: "'Cascadia Code', Consolas, monospace",
    fontSize: 14,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    tabSize: 4,
    insertSpaces: true,
    renderWhitespace: "selection",
    suggest: { showWords: false },
    quickSuggestions: { other: true, comments: false, strings: false },
    inlineSuggest: { enabled: true },
    padding: { top: 12 },
    bracketPairColorization: { enabled: true },
});
const buffers = new Map<string, Buffer>();
const models = new WeakMap<monaco.editor.ITextModel, Buffer>();
function findBuffer(uri: string) {
    return (
        buffers.get(uri) ??
        [...buffers.values()].find(
            (buffer) =>
                buffer.model.uri.toString() ===
                monaco.Uri.parse(uri).toString(),
        )
    );
}
let active: Buffer | undefined;
let entries: Entry[] = [];
let config: Config = {
    lua_ls: "",
    clangd: "",
    copilot: "",
    copilot_enabled: false,
};
let projectRoot = "",
    rootPath = "",
    pocketRoot = "",
    luaLibrary = "",
    appVersion = "";
const services = new Map<string, Client>();
const diagnostics = new Map<string, Json[]>();
const chat = new CopilotChat({
    version: () => appVersion,
    enabled: () => configured("copilot"),
    start: () => rpc("server_start", { server: "copilot_chat" }),
    send: (message) =>
        notify("server_send", { server: "copilot_chat", message }),
    context: () => {
        const selection =
            active && editor.getModel() === active.model
                ? editor.getSelection()
                : null;
        return {
            root: rootPath,
            source: active
                ? {
                      uri: active.uri,
                      path: active.path,
                      language: active.language,
                      text:
                          selection && !selection.isEmpty()
                              ? active.model.getValueInRange(selection)
                              : active.model.getValue(),
                      selection: !!selection && !selection.isEmpty(),
                  }
                : undefined,
        };
    },
    insert: (code) => {
        if (!active || active.readonly)
            throw new Error(
                "Select a writable source tab before inserting code.",
            );
        const selection = editor.getSelection();
        if (!selection) return;
        editor.pushUndoStop();
        editor.executeEdits("copilot-chat", [
            { range: selection, text: code, forceMoveMarkers: true },
        ]);
        editor.pushUndoStop();
        editor.focus();
    },
    setup: () => void showSetup().catch(report),
    openUrl: (uri) => rpc("open_url", { url: uri }),
    showMessage: (params) => showServerMessage("GitHub Copilot", params),
});
function serverFor(buffer: Source) {
    return buffer.language === "pocketlua"
        ? "pocket"
        : buffer.language === "lua"
          ? "lua"
          : buffer.language === "cpp"
            ? "cpp"
            : undefined;
}
function configured(name: string) {
    return !!(name === "lua" || name === "pocket"
        ? config.lua_ls
        : name === "cpp"
          ? config.clangd
          : config.copilot_enabled && config.copilot);
}
const lspPosition = (position: monaco.IPosition) => ({
    line: position.lineNumber - 1,
    character: position.column - 1,
});
const lspRange = (range: monaco.IRange) => ({
    start: {
        line: range.startLineNumber - 1,
        character: range.startColumn - 1,
    },
    end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
});
function range(value: Json): monaco.IRange {
    return {
        startLineNumber: value.start.line + 1,
        startColumn: value.start.character + 1,
        endLineNumber: value.end.line + 1,
        endColumn: value.end.character + 1,
    };
}
function markdown(value: Json): monaco.IMarkdownString {
    return {
        value: typeof value === "string" ? value : (value?.value ?? ""),
        isTrusted: false,
        supportHtml: false,
    };
}
function settings(name: string): Json {
    if (name === "lua")
        return {
            Lua: {
                runtime: { version: "Lua 5.2" },
                workspace: {
                    library: [luaLibrary],
                    checkThirdParty: false,
                    ignoreDir: [".epok/code"],
                },
                diagnostics: { globals: ["epok"] },
                telemetry: { enable: false },
            },
        };
    if (name === "pocket")
        return {
            Lua: {
                runtime: {
                    version: "Lua 5.4",
                    builtin: Object.fromEntries(
                        [
                            "basic",
                            "bit",
                            "bit32",
                            "coroutine",
                            "debug",
                            "ffi",
                            "io",
                            "jit",
                            "math",
                            "os",
                            "package",
                            "string",
                            "table",
                            "table.clear",
                            "table.new",
                            "utf8",
                        ].map((name) => [name, "disable"]),
                    ),
                },
                workspace: { library: ["api"], checkThirdParty: false },
                diagnostics: { disable: ["lowercase-global"] },
                telemetry: { enable: false },
            },
            files: { associations: { "*.plua": "lua" } },
        };
    if (name === "copilot")
        return {
            telemetry: { telemetryLevel: "off" },
            editor: { enableAutoCompletions: true },
        };
    return {};
}
class Client {
    seq = 0;
    generation = 0;
    ready: Promise<void> | undefined;
    opened = new Map<string, number>();
    openedText = new Map<string, string>();
    openedAt = new Map<string, number>();
    requests = new Map<
        number,
        {
            resolve: (v: Json) => void;
            reject: (e: Error) => void;
            timer: number;
            cancel?: monaco.IDisposable;
        }
    >();
    capabilities: Json = {};
    constructor(readonly name: string) {}
    send(message: Json) {
        notify("server_send", {
            server: this.name,
            message: { jsonrpc: "2.0", ...message },
        });
    }
    notification(method: string, params: Json) {
        this.send({ method, params });
    }
    request(
        method: string,
        params: Json = {},
        token?: monaco.CancellationToken,
    ): Promise<Json> {
        if (token?.isCancellationRequested) return Promise.resolve(null);
        const id = ++this.seq;
        return new Promise((resolve, reject) => {
            const timer = window.setTimeout(() => {
                const request = this.requests.get(id);
                this.requests.delete(id);
                request?.cancel?.dispose();
                this.notification("$/cancelRequest", { id });
                reject(new Error(`${this.name}: ${method} timed out`));
            }, 30000);
            const cancel = token?.onCancellationRequested(() => {
                this.notification("$/cancelRequest", { id });
                const request = this.requests.get(id);
                if (request) {
                    clearTimeout(request.timer);
                    this.requests.delete(id);
                    request.cancel?.dispose();
                    request.resolve(null);
                }
            });
            this.requests.set(id, { resolve, reject, timer, cancel });
            this.send({ id, method, params });
        });
    }
    ensure(): Promise<void> {
        if (!this.ready)
            this.ready = (async () => {
                if (!configured(this.name))
                    throw new Error(
                        `${this.name}: open Setup to install or select its language service`,
                    );
                const started = await rpc("server_start", {
                    server: this.name,
                });
                this.generation = started.generation;
                const root = this.name === "pocket" ? pocketRoot : projectRoot;
                const response = await this.request("initialize", {
                    processId: null,
                    rootUri: root,
                    workspaceFolders: [
                        {
                            uri: root,
                            name:
                                this.name === "pocket"
                                    ? "PocketStation"
                                    : $("project").textContent,
                        },
                    ],
                    clientInfo: { name: "Epok", version: appVersion },
                    capabilities: {
                        workspace: {
                            configuration: true,
                            workspaceFolders: true,
                            applyEdit: false,
                        },
                        window: {
                            showDocument: { support: true },
                            workDoneProgress: true,
                        },
                        general: { positionEncodings: ["utf-16"] },
                        textDocument: {
                            synchronization: {
                                dynamicRegistration: false,
                                didSave: true,
                            },
                            completion: {
                                completionItem: {
                                    snippetSupport: true,
                                    documentationFormat: [
                                        "markdown",
                                        "plaintext",
                                    ],
                                    resolveSupport: {
                                        properties: [
                                            "documentation",
                                            "detail",
                                            "additionalTextEdits",
                                        ],
                                    },
                                },
                            },
                            hover: { contentFormat: ["markdown", "plaintext"] },
                            signatureHelp: {
                                signatureInformation: {
                                    documentationFormat: [
                                        "markdown",
                                        "plaintext",
                                    ],
                                    parameterInformation: {
                                        labelOffsetSupport: true,
                                    },
                                },
                            },
                            definition: { linkSupport: true },
                            rename: { prepareSupport: true },
                            publishDiagnostics: { versionSupport: true },
                            inlineCompletion: {},
                        },
                    },
                    initializationOptions: {
                        editorInfo: { name: "Epok", version: appVersion },
                        editorPluginInfo: {
                            name: "Epok Monaco",
                            version: appVersion,
                        },
                    },
                });
                this.capabilities = response?.capabilities ?? {};
                this.notification("initialized", {});
                this.notification("workspace/didChangeConfiguration", {
                    settings: settings(this.name),
                });
                message(
                    `${this.name === "pocket" ? "PocketLua" : this.name} language service ready`,
                );
            })().catch((error) => {
                this.ready = undefined;
                throw error;
            });
        return this.ready;
    }
    async sync(buffer: Buffer) {
        await this.ensure();
        if (buffer.model.isDisposed() || buffers.get(buffer.uri) !== buffer)
            return;
        const version = this.opened.get(buffer.uri);
        if (version === undefined) {
            this.openedAt.set(buffer.uri, Date.now());
            this.notification("textDocument/didOpen", {
                textDocument: {
                    uri: buffer.uri,
                    languageId: buffer.language === "cpp" ? "cpp" : "lua",
                    version: buffer.version,
                    text: buffer.model.getValue(),
                },
            });
        } else if (version !== buffer.version) {
            const previous = (this.openedText.get(buffer.uri) ?? "").split(
                "\n",
            );
            this.notification("textDocument/didChange", {
                textDocument: { uri: buffer.uri, version: buffer.version },
                contentChanges: [
                    {
                        range: {
                            start: { line: 0, character: 0 },
                            end: {
                                line: previous.length - 1,
                                character: previous.at(-1)!.length,
                            },
                        },
                        text: buffer.model.getValue(),
                    },
                ],
            });
        }
        this.opened.set(buffer.uri, buffer.version);
        this.openedText.set(buffer.uri, buffer.model.getValue());
    }
    receive(value: Json) {
        if (value.method) {
            if (value.id !== undefined) {
                if (value.method === "workspace/configuration") {
                    const all = settings(this.name);
                    this.send({
                        id: value.id,
                        result: (value.params?.items ?? []).map((item: Json) =>
                            item.section
                                ? (item.section
                                      .split(".")
                                      .reduce(
                                          (v: Json, k: string) => v?.[k],
                                          all,
                                      ) ?? null)
                                : all,
                        ),
                    });
                } else if (value.method === "window/showDocument") {
                    const generation = this.generation;
                    const operation =
                        value.params.uri.startsWith("file:") &&
                        !value.params.external
                            ? open(
                                  value.params.uri,
                                  value.params.selection
                                      ? value.params.selection.start.line + 1
                                      : undefined,
                              )
                            : rpc("open_url", { url: value.params.uri });
                    void operation
                        .then(
                            () =>
                                generation === this.generation &&
                                this.send({
                                    id: value.id,
                                    result: { success: true },
                                }),
                        )
                        .catch(
                            () =>
                                generation === this.generation &&
                                this.send({
                                    id: value.id,
                                    result: { success: false },
                                }),
                        );
                } else if (value.method === "window/showMessageRequest") {
                    const generation = this.generation;
                    void showServerMessage(this.name, value.params)
                        .then((result) => {
                            if (generation === this.generation)
                                this.send({ id: value.id, result });
                        })
                        .catch(report);
                } else if (value.method === "workspace/applyEdit")
                    this.send({
                        id: value.id,
                        result: {
                            applied: false,
                            failureReason:
                                "Use an explicit editor refactoring action to apply workspace edits.",
                        },
                    });
                else this.send({ id: value.id, result: null });
            } else if (value.method === "textDocument/publishDiagnostics") {
                const uri = value.params.uri;
                const buffer = findBuffer(uri);
                if (
                    buffer &&
                    value.params.version !== undefined &&
                    value.params.version < buffer.version
                )
                    return;
                diagnostics.set(
                    `${this.name}:${uri}`,
                    value.params.diagnostics ?? [],
                );
                if (buffer)
                    monaco.editor.setModelMarkers(
                        buffer.model,
                        this.name,
                        (value.params.diagnostics ?? []).map((d: Json) => ({
                            ...range(d.range),
                            message: d.message,
                            source: d.source ?? this.name,
                            code:
                                d.code === undefined
                                    ? undefined
                                    : String(d.code),
                            severity:
                                d.severity === 1
                                    ? monaco.MarkerSeverity.Error
                                    : d.severity === 2
                                      ? monaco.MarkerSeverity.Warning
                                      : monaco.MarkerSeverity.Info,
                        })),
                    );
                renderProblems();
            } else if (value.method === "didChangeStatus") {
                const status = value.params;
                const text = `Copilot: ${status.message || status.kind || "ready"}`;
                $("copilot-status").textContent = text;
                $("auth-status").textContent = text;
            } else if (value.method === "window/showMessage")
                message(value.params?.message ?? "");
            return;
        }
        const request = this.requests.get(value.id);
        if (!request) return;
        this.requests.delete(value.id);
        clearTimeout(request.timer);
        request.cancel?.dispose();
        if (value.error && [-32800, -32801, -32802].includes(value.error.code))
            request.resolve(null);
        else if (value.error)
            request.reject(
                new Error(value.error.message ?? String(value.error)),
            );
        else request.resolve(value.result);
    }
    stop() {
        for (const request of this.requests.values()) {
            clearTimeout(request.timer);
            request.cancel?.dispose();
            request.reject(new Error("Language service restarted"));
        }
        this.requests.clear();
        this.ready = undefined;
        this.generation = 0;
        this.opened.clear();
        this.openedText.clear();
        this.openedAt.clear();
        for (const buffer of buffers.values())
            monaco.editor.setModelMarkers(buffer.model, this.name, []);
        for (const key of diagnostics.keys())
            if (key.startsWith(`${this.name}:`)) diagnostics.delete(key);
        renderProblems();
    }
}
function client(name: string) {
    let c = services.get(name);
    if (!c) {
        c = new Client(name);
        services.set(name, c);
    }
    return c;
}
async function requestFor(
    model: monaco.editor.ITextModel,
    method: string,
    params: Json,
    token?: monaco.CancellationToken,
) {
    const buffer = models.get(model);
    if (!buffer) return null;
    const name = serverFor(buffer);
    if (!name || !configured(name)) return null;
    const c = client(name);
    await c.sync(buffer);
    if (model.isDisposed() || token?.isCancellationRequested) return null;
    const query = () =>
        c.request(
            method,
            { textDocument: { uri: buffer.uri }, ...params },
            token,
        );
    const version = buffer.version;
    let result = await query();
    // LuaLS can answer before it has indexed a newly opened workspace's API
    // library. Keep the initial completion pending briefly instead of caching
    // an empty result in Monaco until the user types again.
    for (
        let attempt = 0;
        method === "textDocument/completion" &&
        attempt < 10 &&
        (!result ||
            (Array.isArray(result) ? !result.length : !result.items?.length)) &&
        Date.now() - (c.openedAt.get(buffer.uri) ?? 0) < 5000;
        attempt++
    ) {
        if (
            token?.isCancellationRequested ||
            model.isDisposed() ||
            buffer.version !== version
        )
            return null;
        await new Promise((resolve) => window.setTimeout(resolve, 200));
        if (
            token?.isCancellationRequested ||
            model.isDisposed() ||
            buffer.version !== version
        )
            return null;
        result = await query();
    }
    return model.isDisposed() ? null : result;
}
function syncBuffer(buffer: Buffer) {
    const failed = (error: unknown) => {
        if (!buffer.model.isDisposed()) report(error);
    };
    const name = serverFor(buffer);
    if (name && configured(name)) void client(name).sync(buffer).catch(failed);
    if (configured("copilot"))
        void client("copilot").sync(buffer).catch(failed);
}
function activate(buffer: Buffer, line?: number) {
    if (active) active.view = editor.saveViewState();
    active = buffer;
    editor.setModel(buffer.model);
    editor.updateOptions({ readOnly: buffer.readonly });
    if (buffer.view) editor.restoreViewState(buffer.view);
    $<HTMLButtonElement>("save").disabled = buffer.readonly;
    $<HTMLButtonElement>("external").disabled =
        buffer.path.startsWith("PocketStation/");
    if (line) {
        editor.setPosition({ lineNumber: line, column: 1 });
        editor.revealLineInCenter(line);
    }
    $("welcome").hidden = true;
    $("language").textContent =
        buffer.language === "pocketlua"
            ? "Pocket Lua"
            : buffer.language === "cpp"
              ? "C++"
              : buffer.language === "lua"
                ? "Epok Lua"
                : "ARM/Thumb";
    renderTabs();
    renderTree();
    editor.focus();
    syncBuffer(buffer);
    if (configured("copilot"))
        void client("copilot")
            .ensure()
            .then(() =>
                client("copilot").notification("textDocument/didFocus", {
                    textDocument: { uri: buffer.uri },
                }),
            )
            .catch(report);
}
function addBuffer(source: Source) {
    const existing = buffers.get(source.uri);
    if (existing) return existing;
    const model = monaco.editor.createModel(
        source.text,
        source.language === "pocketlua"
            ? "lua"
            : source.language === "arm"
              ? "plaintext"
              : source.language,
        monaco.Uri.parse(source.uri),
    );
    const buffer: Buffer = {
        ...source,
        model,
        saved: source.savedText ?? source.text,
        view: null,
    };
    buffers.set(source.uri, buffer);
    models.set(model, buffer);
    model.onDidChangeContent(() => {
        if (
            new TextEncoder().encode(model.getValue()).length >
            4 * 1024 * 1024
        ) {
            queueMicrotask(() => {
                void model.undo();
                report(
                    "Source buffers are limited to 4 MiB. The oversized edit was undone.",
                );
            });
            return;
        }
        buffer.version++;
        buffer.text = model.getValue();
        notify("change", {
            uri: buffer.uri,
            text: buffer.text,
            version: buffer.version,
        });
        renderTabs();
        syncBuffer(buffer);
    });
    return buffer;
}
async function open(uri: string, line?: number) {
    const buffer = findBuffer(uri) ?? addBuffer(await rpc("open", { uri }));
    activate(buffer, line);
}
async function save(buffer = active) {
    if (!buffer || buffer.readonly) return;
    await rpc("save", { uri: buffer.uri });
    message(`Saved ${buffer.path}`);
}
function saved(uri: string, text: string) {
    const buffer = buffers.get(uri);
    if (!buffer) return;
    buffer.saved = text;
    renderTabs();
    const name = serverFor(buffer);
    if (name && client(name).opened.has(uri))
        client(name).notification("textDocument/didSave", {
            textDocument: { uri },
        });
}
function renderTabs() {
    const nav = $("tabs");
    nav.replaceChildren();
    for (const buffer of buffers.values()) {
        const dirty = buffer.model.getValue() !== buffer.saved;
        const tab = document.createElement("div");
        tab.className = `tab${active === buffer ? " active" : ""}${dirty ? " dirty" : ""}`;
        tab.setAttribute("role", "tab");
        tab.setAttribute("aria-selected", String(active === buffer));
        tab.title = buffer.path;
        const label = document.createElement("span");
        label.textContent = buffer.path.split("/").pop() ?? buffer.path;
        tab.append(label);
        const close = document.createElement("button");
        close.className = "close";
        close.textContent = dirty ? "●" : "×";
        close.title = `Close ${label.textContent}`;
        close.onclick = (event) => {
            event.stopPropagation();
            void closeBuffer(buffer).catch(report);
        };
        tab.append(close);
        tab.onclick = () => activate(buffer);
        nav.append(tab);
    }
}
let closing: Buffer | undefined;
async function closeBuffer(buffer: Buffer, discard = false) {
    if (!discard && buffer.model.getValue() !== buffer.saved) {
        closing = buffer;
        $("close-message").textContent = `Save changes to ${buffer.path}?`;
        $<HTMLDialogElement>("confirm-close").showModal();
        return;
    }
    await rpc("close", { uri: buffer.uri, discard });
    for (const c of services.values()) {
        c.openedText.delete(buffer.uri);
        c.openedAt.delete(buffer.uri);
        if (c.opened.delete(buffer.uri))
            c.notification("textDocument/didClose", {
                textDocument: { uri: buffer.uri },
            });
    }
    for (const key of diagnostics.keys()) {
        if (findBuffer(key.slice(key.indexOf(":") + 1)) === buffer)
            diagnostics.delete(key);
    }
    renderProblems();
    buffers.delete(buffer.uri);
    if (active === buffer) {
        active = undefined;
        editor.setModel(null);
    }
    buffer.model.dispose();
    if (!active && buffers.size) activate([...buffers.values()].at(-1)!);
    else {
        renderTabs();
        $("welcome").hidden = !!active;
        renderTree();
    }
}
$("close-save").onclick = () => {
    const buffer = closing;
    $<HTMLDialogElement>("confirm-close").close();
    if (buffer)
        void save(buffer)
            .then(() => closeBuffer(buffer))
            .catch(report);
};
$("close-discard").onclick = () => {
    const buffer = closing;
    $<HTMLDialogElement>("confirm-close").close();
    if (buffer) void closeBuffer(buffer, true).catch(report);
};
$("close-cancel").onclick = () => $<HTMLDialogElement>("confirm-close").close();
const expanded = new Set<string>(["assets", "assets/scripts", "PocketStation"]);
function renderTree() {
    const tree = $("tree");
    tree.replaceChildren();
    const filter = $<HTMLInputElement>("filter").value.toLowerCase();
    const folders = new Map<string, HTMLElement>([["", tree]]);
    for (const entry of entries.filter((e) =>
        e.path.toLowerCase().includes(filter),
    )) {
        const parts = entry.path.split("/");
        let parent = tree;
        let path = "";
        for (const part of parts.slice(0, -1)) {
            path = path ? `${path}/${part}` : part;
            if (!folders.has(path)) {
                const details = document.createElement("details");
                details.open = !!filter || expanded.has(path);
                const key = path;
                details.ontoggle = () => {
                    if (details.open) expanded.add(key);
                    else expanded.delete(key);
                };
                const summary = document.createElement("summary");
                summary.textContent = part;
                details.append(summary);
                const inner = document.createElement("div");
                details.append(inner);
                parent.append(details);
                folders.set(path, inner);
            }
            parent = folders.get(path)!;
        }
        const button = document.createElement("button");
        button.className = `tree-file${active?.uri === entry.uri ? " active" : ""}`;
        button.dataset.language = entry.language;
        button.title = entry.path;
        button.setAttribute("role", "treeitem");
        const icon = document.createElement("span");
        icon.className = "tree-icon";
        icon.textContent =
            entry.language === "cpp"
                ? "C++"
                : entry.language === "pocketlua"
                  ? "▣"
                  : "◈";
        button.append(icon, document.createTextNode(parts.at(-1)!));
        button.onclick = () => void open(entry.uri).catch(report);
        parent.append(button);
    }
}
$("filter").oninput = renderTree;
$("refresh").onclick = () =>
    void rpc("tree")
        .then((value) => {
            entries = value;
            renderTree();
        })
        .catch(report);
$("save").onclick = () => void save().catch(report);
$("external").onclick = () => {
    if (active)
        void rpc("open_external", {
            uri: active.uri,
            line: editor.getPosition()?.lineNumber ?? 1,
        }).catch(report);
};
$<HTMLSelectElement>("new-source").onchange = () => {
    const select = $<HTMLSelectElement>("new-source");
    const kind = select.value;
    select.value = "";
    if (kind === "pocket") {
        if (!active) {
            report("Select a PocketStation source first");
            return;
        }
        void rpc("new_pocket_assembly", { uri: active.uri })
            .then((result) => {
                entries = result.tree;
                activate(addBuffer(result.document));
                renderTree();
            })
            .catch(report);
    } else if (kind) void rpc("new_class", { provider: kind }).catch(report);
};
$("save-all").onclick = () =>
    void rpc("save_all")
        .then(() => message("All source files saved"))
        .catch(report);
editor.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS,
    () => void save().catch(report),
);
editor.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyS,
    () => void rpc("save_all").catch(report),
);
async function runActive() {
    if (!active) return;
    const result = await rpc("run", { uri: active.uri });
    message(
        result.error ?? result.message ?? "Build & Run requested",
        !!result.error,
    );
}
editor.addCommand(monaco.KeyCode.F5, () => void runActive().catch(report));
document.addEventListener("keydown", (event) => {
    if (event.key === "F5" && !editor.hasTextFocus()) {
        event.preventDefault();
        void runActive().catch(report);
    }
    if (
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "s" &&
        !editor.hasTextFocus()
    ) {
        event.preventDefault();
        void (event.shiftKey ? rpc("save_all") : save()).catch(report);
    }
});
editor.onDidChangeCursorPosition((event) => {
    $("position").textContent =
        `Ln ${event.position.lineNumber}, Col ${event.position.column}`;
});
monaco.editor.registerEditorOpener({
    openCodeEditor: async (_source, resource, selection) => {
        const line = selection
            ? "startLineNumber" in selection
                ? selection.startLineNumber
                : selection.lineNumber
            : 1;
        await open(resource.toString(), line);
        return true;
    },
});

const kinds = [
    monaco.languages.CompletionItemKind.Text,
    monaco.languages.CompletionItemKind.Method,
    monaco.languages.CompletionItemKind.Function,
    monaco.languages.CompletionItemKind.Constructor,
    monaco.languages.CompletionItemKind.Field,
    monaco.languages.CompletionItemKind.Variable,
    monaco.languages.CompletionItemKind.Class,
    monaco.languages.CompletionItemKind.Interface,
    monaco.languages.CompletionItemKind.Module,
    monaco.languages.CompletionItemKind.Property,
    monaco.languages.CompletionItemKind.Unit,
    monaco.languages.CompletionItemKind.Value,
    monaco.languages.CompletionItemKind.Enum,
    monaco.languages.CompletionItemKind.Keyword,
    monaco.languages.CompletionItemKind.Snippet,
    monaco.languages.CompletionItemKind.Color,
    monaco.languages.CompletionItemKind.File,
    monaco.languages.CompletionItemKind.Reference,
    monaco.languages.CompletionItemKind.Folder,
    monaco.languages.CompletionItemKind.EnumMember,
    monaco.languages.CompletionItemKind.Constant,
    monaco.languages.CompletionItemKind.Struct,
    monaco.languages.CompletionItemKind.Event,
    monaco.languages.CompletionItemKind.Operator,
    monaco.languages.CompletionItemKind.TypeParameter,
];
for (const language of ["lua", "cpp"]) {
    monaco.languages.registerCompletionItemProvider(language, {
        triggerCharacters: [".", ":", '"', "<"],
        provideCompletionItems: async (model, position, context, token) => {
            const result = await requestFor(
                model,
                "textDocument/completion",
                {
                    position: lspPosition(position),
                    context: {
                        triggerKind: context.triggerKind + 1,
                        triggerCharacter: context.triggerCharacter,
                    },
                },
                token,
            );
            if (!result) return { suggestions: [] };
            const word = model.getWordUntilPosition(position);
            const fallback = {
                startLineNumber: position.lineNumber,
                endLineNumber: position.lineNumber,
                startColumn: word.startColumn,
                endColumn: word.endColumn,
            };
            return {
                incomplete: result.isIncomplete,
                suggestions: (Array.isArray(result)
                    ? result
                    : (result.items ?? [])
                ).map((item: Json) => ({
                    lspItem: item,
                    server: serverFor(models.get(model)!),
                    label: item.label,
                    kind: kinds[(item.kind ?? 1) - 1] ?? kinds[0],
                    detail: item.detail,
                    documentation: markdown(item.documentation),
                    sortText: item.sortText,
                    filterText: item.filterText,
                    insertText:
                        item.textEdit?.newText ?? item.insertText ?? item.label,
                    insertTextRules:
                        item.insertTextFormat === 2
                            ? monaco.languages.CompletionItemInsertTextRule
                                  .InsertAsSnippet
                            : undefined,
                    range: item.textEdit?.range
                        ? range(item.textEdit.range)
                        : item.textEdit?.insert
                          ? {
                                insert: range(item.textEdit.insert),
                                replace: range(item.textEdit.replace),
                            }
                          : fallback,
                    additionalTextEdits: item.additionalTextEdits?.map(
                        (e: Json) => ({
                            range: range(e.range),
                            text: e.newText,
                        }),
                    ),
                    command: item.command
                        ? {
                              id: "epok.lsp.command",
                              title: item.command.title,
                              arguments: [
                                  serverFor(models.get(model)!),
                                  item.command,
                              ],
                          }
                        : undefined,
                })),
            };
        },
        resolveCompletionItem: async (item, token) => {
            const original = (item as Json).lspItem;
            const name = (item as Json).server;
            if (
                !original ||
                !name ||
                !client(name).capabilities.completionProvider?.resolveProvider
            )
                return item;
            const resolved = await client(name).request(
                "completionItem/resolve",
                original,
                token,
            );
            if (!resolved) return item;
            return {
                ...item,
                detail: resolved.detail ?? item.detail,
                documentation: resolved.documentation
                    ? markdown(resolved.documentation)
                    : item.documentation,
                additionalTextEdits:
                    resolved.additionalTextEdits?.map((e: Json) => ({
                        range: range(e.range),
                        text: e.newText,
                    })) ?? item.additionalTextEdits,
            };
        },
    });
    monaco.languages.registerHoverProvider(language, {
        provideHover: async (model, position, token) => {
            const result = await requestFor(
                model,
                "textDocument/hover",
                { position: lspPosition(position) },
                token,
            );
            if (!result) return null;
            return {
                range: result.range ? range(result.range) : undefined,
                contents: (Array.isArray(result.contents)
                    ? result.contents
                    : [result.contents]
                ).map(markdown),
            };
        },
    });
    monaco.languages.registerDefinitionProvider(language, {
        provideDefinition: async (model, position, token) => {
            const result = await requestFor(
                model,
                "textDocument/definition",
                { position: lspPosition(position) },
                token,
            );
            if (!result) return null;
            return (Array.isArray(result) ? result : [result]).map(
                (location: Json) => ({
                    uri: monaco.Uri.parse(location.uri ?? location.targetUri),
                    range: range(
                        location.range ??
                            location.targetSelectionRange ??
                            location.targetRange,
                    ),
                }),
            );
        },
    });
    monaco.languages.registerReferenceProvider(language, {
        provideReferences: async (model, position, context, token) => {
            const result = await requestFor(
                model,
                "textDocument/references",
                { position: lspPosition(position), context },
                token,
            );
            return (result ?? []).map((location: Json) => ({
                uri: monaco.Uri.parse(location.uri),
                range: range(location.range),
            }));
        },
    });
    monaco.languages.registerSignatureHelpProvider(language, {
        signatureHelpTriggerCharacters: ["(", ","],
        signatureHelpRetriggerCharacters: [","],
        provideSignatureHelp: async (model, position, token) => {
            const result = await requestFor(
                model,
                "textDocument/signatureHelp",
                { position: lspPosition(position) },
                token,
            );
            if (!result) return null;
            return {
                value: {
                    activeSignature: result.activeSignature ?? 0,
                    activeParameter: result.activeParameter ?? 0,
                    signatures: result.signatures.map((s: Json) => ({
                        label: s.label,
                        documentation: markdown(s.documentation),
                        parameters: (s.parameters ?? []).map((p: Json) => ({
                            label: p.label,
                            documentation: markdown(p.documentation),
                        })),
                    })),
                },
                dispose() {},
            };
        },
    });
    monaco.languages.registerDocumentFormattingEditProvider(language, {
        provideDocumentFormattingEdits: async (model, options, token) => {
            const result = await requestFor(
                model,
                "textDocument/formatting",
                {
                    options: {
                        tabSize: options.tabSize,
                        insertSpaces: options.insertSpaces,
                    },
                },
                token,
            );
            return (result ?? []).map((e: Json) => ({
                range: range(e.range),
                text: e.newText,
            }));
        },
    });
    monaco.languages.registerRenameProvider(language, {
        provideRenameEdits: async (model, position, newName, token) => {
            const result = await requestFor(
                model,
                "textDocument/rename",
                { position: lspPosition(position), newName },
                token,
            );
            if (!result) return { edits: [] };
            const changes: Json[] = [
                ...Object.entries(result.changes ?? {}).flatMap(
                    ([uri, edits]) =>
                        (edits as Json[]).map((edit) => ({ uri, edit })),
                ),
                ...(result.documentChanges ?? [])
                    .filter((c: Json) => c.textDocument)
                    .flatMap((c: Json) =>
                        c.edits.map((edit: Json) => ({
                            uri: c.textDocument.uri,
                            edit,
                        })),
                    ),
            ];
            for (const change of changes) {
                if (!findBuffer(change.uri))
                    addBuffer(await rpc("open", { uri: change.uri }));
                if (findBuffer(change.uri)?.readonly)
                    throw new Error(
                        "A rename would modify read-only SDK source",
                    );
            }
            return {
                edits: changes.map((c) => ({
                    resource: monaco.Uri.parse(c.uri),
                    versionId: undefined,
                    textEdit: {
                        range: range(c.edit.range),
                        text: c.edit.newText,
                    },
                })),
            };
        },
    });
}
monaco.editor.registerCommand("epok.lsp.command", (_accessor, name, command) =>
    client(String(name)).request("workspace/executeCommand", command),
);
monaco.editor.registerCommand("epok.copilot.accept", (_accessor, command) =>
    client("copilot").request("workspace/executeCommand", command),
);
for (const language of ["lua", "cpp"])
    monaco.languages.registerInlineCompletionsProvider(language, {
        provideInlineCompletions: async (model, position, context, token) => {
            const buffer = models.get(model);
            if (!buffer || buffer.readonly || !configured("copilot"))
                return { items: [] };
            const c = client("copilot");
            await c.sync(buffer);
            if (model.isDisposed() || token.isCancellationRequested)
                return { items: [] };
            c.notification("textDocument/didFocus", {
                textDocument: { uri: buffer.uri },
            });
            const result = await c.request(
                "textDocument/inlineCompletion",
                {
                    textDocument: { uri: buffer.uri, version: buffer.version },
                    position: lspPosition(position),
                    context: { triggerKind: context.triggerKind === 0 ? 2 : 1 },
                    formattingOptions: {
                        tabSize: model.getOptions().tabSize,
                        insertSpaces: model.getOptions().insertSpaces,
                    },
                },
                token,
            );
            return {
                items: (result?.items ?? []).map((item: Json) => ({
                    insertText: item.insertText,
                    range: item.range ? range(item.range) : undefined,
                    command: item.command
                        ? {
                              id: "epok.copilot.accept",
                              title: "Accept Copilot suggestion",
                              arguments: [item.command],
                          }
                        : undefined,
                    lspItem: item,
                })),
            };
        },
        handleItemDidShow: (_list, item) =>
            client("copilot").notification("textDocument/didShowCompletion", {
                item: (item as Json).lspItem,
            }),
        handlePartialAccept: (_list, item, acceptedCharacters) =>
            client("copilot").notification(
                "textDocument/didPartiallyAcceptCompletion",
                {
                    item: (item as Json).lspItem,
                    acceptedLength: acceptedCharacters,
                },
            ),
        disposeInlineCompletions() {},
    });

function renderProblems() {
    const panel = $("problems");
    panel.replaceChildren();
    let count = 0;
    for (const [key, items] of diagnostics) {
        const uri = key.slice(key.indexOf(":") + 1);
        for (const diagnostic of items) {
            count++;
            const button = document.createElement("button");
            button.className = `problem ${diagnostic.severity === 1 ? "error" : "warning"}`;
            button.textContent = `${buffers.get(uri)?.path ?? uri.split("/").pop()}:${diagnostic.range.start.line + 1}  ${diagnostic.message}`;
            button.onclick = () =>
                void open(uri, diagnostic.range.start.line + 1).catch(report);
            panel.append(button);
        }
    }
    $("problems-toggle").textContent = `Problems (${count})`;
}
$("problems-toggle").onclick = () => {
    $("problems").hidden = !$("problems").hidden;
    editor.layout();
};
function renderSetup() {
    const target = $("tool-settings");
    target.replaceChildren();
    for (const [name, label, hint] of [
        [
            "lua_ls",
            "Lua Language Server",
            "Used for Epok Lua and a separate PocketLua workspace.",
        ],
        [
            "clangd",
            "clangd · C++",
            "Uses the engine’s generated compilation database and MIPS SDK headers.",
        ],
        [
            "copilot",
            "GitHub Copilot Language Server",
            "Official portable server. Sign in with your own GitHub account.",
        ],
    ]) {
        const group = document.createElement("div");
        group.className = "tool";
        const title = document.createElement("label");
        title.textContent = label;
        title.htmlFor = `tool-${name}`;
        const row = document.createElement("div");
        row.className = "tool-row";
        const input = document.createElement("input");
        input.id = `tool-${name}`;
        input.value = config[name as keyof Config] as string;
        input.placeholder = "Auto-detected path or full path to executable";
        const install = document.createElement("button");
        install.type = "button";
        install.textContent = "Install";
        install.onclick = () => {
            install.disabled = true;
            void rpc("install", { tool: name }).catch((error) => {
                install.disabled = false;
                report(error);
            });
        };
        row.append(input, install);
        const small = document.createElement("small");
        small.textContent = hint;
        group.append(title, row, small);
        target.append(group);
    }
    $<HTMLInputElement>("copilot-enabled").checked = config.copilot_enabled;
}
async function showSetup() {
    config = await rpc("config");
    renderSetup();
    $<HTMLDialogElement>("setup").showModal();
}
$("settings-open").onclick = () => void showSetup().catch(report);
$("welcome-setup").onclick = () => void showSetup().catch(report);
$("copilot-status").onclick = () => void showSetup().catch(report);
$("detect").onclick = () =>
    void rpc("detect")
        .then((value) => {
            config = value;
            renderSetup();
            $("setup-status").textContent = "Tool paths refreshed.";
        })
        .catch(report);
$("apply-settings").onclick = () =>
    void (async () => {
        const next: Config = {
            lua_ls: $<HTMLInputElement>("tool-lua_ls").value.trim(),
            clangd: $<HTMLInputElement>("tool-clangd").value.trim(),
            copilot: $<HTMLInputElement>("tool-copilot").value.trim(),
            copilot_enabled: $<HTMLInputElement>("copilot-enabled").checked,
        };
        await rpc("set_config", next);
        chat.reset();
        for (const service of services.values()) service.stop();
        services.clear();
        config = next;
        $("setup-status").textContent =
            "Settings saved. Language services restart for open files.";
        $("copilot-status").textContent = config.copilot_enabled
            ? "Copilot connecting…"
            : "Copilot off";
        if (active) syncBuffer(active);
    })().catch(report);
$("copilot-signin").onclick = () =>
    void (async () => {
        if (!config.copilot_enabled)
            throw new Error("Enable Copilot and Apply before signing in.");
        const c = client("copilot");
        await c.ensure();
        const result = await c.request("signIn");
        $("auth-status").textContent = result.userCode
            ? `Enter ${result.userCode} on GitHub to sign in.`
            : "Follow the GitHub sign-in page.";
        if (result.command)
            await c.request("workspace/executeCommand", result.command);
    })().catch(report);
$("copilot-signout").onclick = () =>
    void client("copilot")
        .ensure()
        .then(() => client("copilot").request("signOut"))
        .then(() => {
            $("auth-status").textContent = "Signed out of GitHub Copilot.";
        })
        .catch(report);

window.epokReceive = (value) => {
    if (chat.receiveNative(value)) return;
    if (value.id) {
        const request = pending.get(value.id);
        if (request) {
            pending.delete(value.id);
            clearTimeout(request.timer);
            if (value.error) request.reject(new Error(value.error));
            else request.resolve(value.result);
        }
        return;
    }
    if (value.event === "lsp") {
        const c = services.get(value.server);
        if (c && value.generation === c.generation) c.receive(value.message);
        return;
    }
    if (value.event === "open") {
        void open(value.uri, value.line).catch(report);
        return;
    }
    if (value.event === "tree_changed") {
        void rpc("tree")
            .then((tree) => {
                entries = tree;
                renderTree();
            })
            .catch(report);
        return;
    }
    if (value.event === "setup") {
        void showSetup().catch(report);
        return;
    }
    if (value.event === "saved") {
        saved(value.uri, value.text);
        return;
    }
    if (value.event === "error") {
        report(value.message);
        return;
    }
    if (value.event === "server_error") {
        const c = services.get(value.server);
        if (
            c &&
            (value.generation === undefined ||
                value.generation === c.generation)
        ) {
            c.stop();
            message(`${value.server}: ${value.message}`, true);
        }
        return;
    }
    if (value.event === "install_progress") {
        $("setup-status").textContent = value.message;
        return;
    }
    if (value.event === "installed") {
        if (value.error) {
            $("setup-status").textContent = value.error;
            report(value.error);
        } else {
            config[value.tool as "lua_ls" | "clangd" | "copilot"] = value.path;
            const input = $<HTMLInputElement>(`tool-${value.tool}`);
            if (input) input.value = value.path;
            $("setup-status").textContent =
                `${value.tool} installed and configured. Apply to restart language services.`;
        }
        for (const button of $(
            "tool-settings",
        ).querySelectorAll<HTMLButtonElement>("button"))
            button.disabled = false;
    }
};
void rpc("init")
    .then((snapshot) => {
        projectRoot = snapshot.root;
        rootPath = snapshot.rootPath;
        pocketRoot = snapshot.pocketRoot;
        luaLibrary = snapshot.luaLibrary;
        appVersion = snapshot.version;
        config = snapshot.config;
        entries = snapshot.tree;
        $("project").textContent = snapshot.project;
        for (const source of snapshot.documents) addBuffer(source);
        renderTree();
        if (buffers.size) activate([...buffers.values()][0]);
        $("copilot-status").textContent = config.copilot_enabled
            ? "Copilot connecting…"
            : "Copilot off";
        message("Monaco ready · Lua / C++ / PocketLua");
    })
    .catch(report);
