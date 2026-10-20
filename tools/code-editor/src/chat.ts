type Json = any;
type Context = {
    root: string;
    source?: {
        uri: string;
        path: string;
        language: string;
        text: string;
        selection: boolean;
    };
};
type Host = {
    version(): string;
    enabled(): boolean;
    start(): Promise<{ generation: number }>;
    send(message: Json): void;
    context(): Context;
    insert(code: string): void;
    setup(): void;
    openUrl(uri: string): Promise<Json>;
    showMessage(params: Json): Promise<Json>;
};
type Message = { role: string; text: string };
const element = <T extends HTMLElement = HTMLElement>(id: string) =>
    document.getElementById(id) as T;

/** Official Copilot ACP client. Transport is supplied by Epok's local bridge. */
export class CopilotChat {
    generation = 0;
    private sequence = 0;
    private session?: string;
    private boot?: Promise<void>;
    private initializing?: Promise<void>;
    private authentication: Json[] = [];
    private capabilities: Json = {};
    private pending = new Map<
        number,
        {
            resolve(value: Json): void;
            reject(error: Error): void;
            timer: number;
        }
    >();
    private permissions = new Map<Json, () => void>();
    private messages: Message[] = [];
    private answer?: Message;
    private busy = false;
    private rendering = false;
    private epoch = 0;

    constructor(private host: Host) {
        element("chat-toggle").onclick = () => {
            const panel = element("chat");
            panel.hidden = !panel.hidden;
            document.body.classList.toggle("chat-visible", !panel.hidden);
            element("chat-toggle").setAttribute(
                "aria-expanded",
                String(!panel.hidden),
            );
            if (!panel.hidden)
                element<HTMLTextAreaElement>("chat-input").focus();
        };
        element("chat-close").onclick = () => element("chat-toggle").click();
        element("chat-setup").onclick = () => host.setup();
        element("chat-connect").onclick = () =>
            void this.ensure().catch((error) => this.error(error));
        element("chat-signin").onclick = () =>
            void this.signIn().catch((error) => this.error(error));
        element("chat-new").onclick = () => this.newConversation();
        element("chat-stop").onclick = () => this.cancel();
        element("chat-send").onclick = () =>
            void this.prompt().catch((error) => this.error(error));
        element("chat-input").onkeydown = (event) => {
            if (
                event.key === "Enter" &&
                !event.shiftKey &&
                !event.isComposing
            ) {
                event.preventDefault();
                void this.prompt().catch((error) => this.error(error));
            }
        };
        this.render();
    }
    private status(text: string, state = "idle") {
        element("chat-status").textContent = text;
        element("chat").dataset.state = state;
    }
    private send(message: Json) {
        this.host.send({ jsonrpc: "2.0", ...message });
    }
    private request(
        method: string,
        params: Json,
        timeout = 120000,
    ): Promise<Json> {
        const id = ++this.sequence;
        return new Promise((resolve, reject) => {
            const timer = window.setTimeout(() => {
                this.pending.delete(id);
                if (method === "session/prompt") this.cancel();
                reject(new Error(`Copilot chat: ${method} timed out`));
            }, timeout);
            this.pending.set(id, { resolve, reject, timer });
            this.send({ id, method, params });
        });
    }
    private initialize(): Promise<void> {
        if (!this.initializing)
            this.initializing = (async () => {
                if (!this.host.enabled())
                    throw new Error("Enable GitHub Copilot in Setup first.");
                this.status("Connecting to GitHub Copilot…", "connecting");
                const started = await this.host.start();
                this.generation = started.generation;
                const result = await this.request("initialize", {
                    protocolVersion: 1,
                    clientCapabilities: {
                        fs: { readTextFile: false, writeTextFile: false },
                        terminal: false,
                    },
                    clientInfo: {
                        name: "Epok",
                        title: "Epok Code",
                        version: this.host.version(),
                    },
                });
                if (result?.protocolVersion !== 1)
                    throw new Error(
                        "This Copilot server uses an unsupported chat protocol. Update it in Setup.",
                    );
                this.authentication = result.authMethods ?? [];
                this.capabilities = result.agentCapabilities ?? {};
            })().catch((error) => {
                this.initializing = undefined;
                throw error;
            });
        return this.initializing;
    }
    ensure(): Promise<void> {
        if (!this.boot)
            this.boot = (async () => {
                await this.initialize();
                const result = await this.request("session/new", {
                    cwd: this.host.context().root,
                    mcpServers: [],
                });
                this.session = result.sessionId;
                if (!this.session)
                    throw new Error("Copilot did not create a chat session.");
                this.status("Copilot chat ready", "ready");
            })().catch((error) => {
                this.boot = undefined;
                throw error;
            });
        return this.boot;
    }
    private async signIn() {
        await this.initialize();
        const method = this.authentication.find(
            (method) => !method.type || method.type === "agent",
        );
        if (!method)
            throw new Error(
                "Use Sign in to GitHub in Setup, then reconnect the chat.",
            );
        this.status(
            "Complete the GitHub sign-in in your browser.",
            "authenticating",
        );
        await this.request("authenticate", { methodId: method.id }, 600000);
        this.session = undefined;
        this.boot = undefined;
        await this.ensure();
    }
    private setBusy(value: boolean) {
        this.busy = value;
        element<HTMLButtonElement>("chat-send").disabled = value;
        element<HTMLButtonElement>("chat-new").disabled = value;
        element<HTMLButtonElement>("chat-connect").disabled = value;
        element<HTMLButtonElement>("chat-signin").disabled = value;
        element<HTMLButtonElement>("chat-stop").disabled = !value;
    }
    private async prompt() {
        const input = element<HTMLTextAreaElement>("chat-input");
        const text = input.value.trim();
        if (!text || this.busy) return;
        const context = this.host.context();
        const source = element<HTMLInputElement>("chat-context").checked
            ? context.source
            : undefined;
        if (
            source &&
            new TextEncoder().encode(source.text).length > 128 * 1024
        ) {
            this.error(
                "Select a smaller code range, or turn off Include current code. Attachments are limited to 128 KiB.",
            );
            return;
        }
        this.setBusy(true);
        const epoch = this.epoch;
        try {
            await this.ensure();
            if (epoch !== this.epoch) return;
            input.value = "";
            this.messages.push({
                role: "You",
                text:
                    text +
                    (source
                        ? `\n\n[${source.selection ? "Selection" : "File"}: ${source.path}]`
                        : ""),
            });
            this.answer = { role: "Copilot", text: "" };
            this.messages.push(this.answer);
            this.render();
            this.status("Copilot is responding…", "responding");
            const prompt: Json[] = [
                {
                    type: "text",
                    text: `You are assisting inside Epok Engine, a PlayStation game editor. The languages are C++20, Epok gameplay Lua and the separate PocketLua dialect compiled to ARM for PocketStation. Respect the selected language and existing engine APIs. Answer the user's question; use tools only when the user asks you to inspect or change files.\n\n${text}`,
                },
            ];
            if (source) {
                const label = `${source.language}; ${source.selection ? "selected code" : "current file"}`;
                if (this.capabilities.promptCapabilities?.embeddedContext)
                    prompt.push({
                        type: "resource",
                        resource: {
                            uri: source.uri,
                            mimeType: "text/plain",
                            text: source.text,
                        },
                        annotations: { audience: ["assistant"] },
                    });
                else
                    prompt.push({
                        type: "text",
                        text: `${source.path} (${label}):\n\n${source.text}`,
                    });
                prompt.push({
                    type: "text",
                    text: `The attached source uses ${label}. PocketLua APIs are screen, button, sound, clock, save, link, system and restricted math; gameplay Lua uses the epok namespace. Avoid mixing them.`,
                });
            }
            const result = await this.request(
                "session/prompt",
                { sessionId: this.session, prompt },
                600000,
            );
            if (epoch !== this.epoch) return;
            this.status(
                result?.stopReason === "cancelled"
                    ? "Response stopped"
                    : "Copilot chat ready",
                "ready",
            );
        } catch (error) {
            if (epoch === this.epoch) this.error(error);
        } finally {
            if (epoch === this.epoch) {
                this.setBusy(false);
                this.answer = undefined;
                this.render();
            }
        }
    }
    cancel() {
        if (this.session)
            this.send({
                method: "session/cancel",
                params: { sessionId: this.session },
            });
        for (const cancel of this.permissions.values()) cancel();
        this.permissions.clear();
        this.status("Stopping response…", "stopping");
    }
    newConversation() {
        if (this.busy) return;
        this.epoch++;
        this.session = undefined;
        this.boot = undefined;
        this.messages = [];
        this.answer = undefined;
        this.status("New conversation", "idle");
        this.render();
    }
    reset() {
        this.epoch++;
        this.generation = 0;
        for (const request of this.pending.values()) {
            clearTimeout(request.timer);
            request.reject(new Error("Copilot chat restarted"));
        }
        this.pending.clear();
        for (const cancel of this.permissions.values()) cancel();
        this.permissions.clear();
        this.initializing = undefined;
        this.boot = undefined;
        this.session = undefined;
        this.answer = undefined;
        this.setBusy(false);
        this.status("Reconnect to start a new conversation", "idle");
        this.render();
    }
    receiveNative(event: Json): boolean {
        if (event.server !== "copilot_chat") return false;
        if (
            event.generation !== undefined &&
            event.generation !== this.generation
        )
            return true;
        if (event.event === "server_error") {
            this.reset();
            this.error(event.message);
            return true;
        }
        if (event.event !== "lsp") return true;
        const value = event.message;
        if (value.method) {
            if (value.method === "session/update") {
                if (value.params.sessionId !== this.session) return true;
                const update = value.params.update;
                if (
                    update.sessionUpdate === "agent_message_chunk" &&
                    update.content?.type === "text" &&
                    this.answer
                ) {
                    this.answer.text += update.content.text;
                    if (this.answer.text.length > 2 * 1024 * 1024)
                        this.cancel();
                    this.scheduleRender();
                } else if (
                    update.sessionUpdate === "tool_call" ||
                    update.sessionUpdate === "tool_call_update"
                ) {
                    this.status(
                        `${update.title ?? "Copilot tool"}${update.status ? ` · ${update.status}` : ""}`,
                        "responding",
                    );
                } else if (update.sessionUpdate === "notice" && update.message)
                    this.status(String(update.message));
            } else if (value.id !== undefined) {
                const generation = this.generation;
                if (value.method === "session/request_permission")
                    this.permission(value.id, value.params);
                else if (value.method === "window/showDocument")
                    void this.host
                        .openUrl(value.params.uri)
                        .then(() => {
                            if (this.generation === generation)
                                this.send({
                                    id: value.id,
                                    result: { success: true },
                                });
                        })
                        .catch(() => {
                            if (this.generation === generation)
                                this.send({
                                    id: value.id,
                                    result: { success: false },
                                });
                        });
                else if (value.method === "window/showMessageRequest")
                    void this.host.showMessage(value.params).then((result) => {
                        if (this.generation === generation)
                            this.send({ id: value.id, result });
                    });
                else
                    this.send({
                        id: value.id,
                        error: {
                            code: -32601,
                            message:
                                "This editor does not provide that client capability",
                        },
                    });
            }
            return true;
        }
        const request = this.pending.get(value.id);
        if (request) {
            this.pending.delete(value.id);
            clearTimeout(request.timer);
            if (value.error)
                request.reject(
                    new Error(
                        value.error.message ?? "Copilot chat request failed",
                    ),
                );
            else request.resolve(value.result);
        }
        return true;
    }
    private permission(id: Json, params: Json) {
        const card = document.createElement("div");
        card.className = "chat-permission";
        const title = document.createElement("strong");
        title.textContent =
            params.toolCall?.title ?? "Copilot requests permission";
        const details = document.createElement("pre");
        details.textContent = JSON.stringify(params.toolCall, null, 2).slice(
            0,
            65536,
        );
        card.append(title, details);
        const generation = this.generation;
        const finish = (outcome: Json) => {
            card.remove();
            this.permissions.delete(id);
            if (generation === this.generation)
                this.send({ id, result: { outcome } });
        };
        this.permissions.set(id, () => finish({ outcome: "cancelled" }));
        for (const option of params.options ?? []) {
            if (option.kind !== "allow_once" && option.kind !== "reject_once")
                continue;
            const button = document.createElement("button");
            button.textContent = option.name;
            button.onclick = () =>
                finish({ outcome: "selected", optionId: option.optionId });
            card.append(button);
        }
        const cancel = document.createElement("button");
        cancel.textContent = "Cancel";
        cancel.onclick = () => finish({ outcome: "cancelled" });
        card.append(cancel);
        element("chat-permissions").append(card);
        this.status("Copilot is waiting for your decision", "permission");
    }
    private error(error: unknown) {
        const text = error instanceof Error ? error.message : String(error);
        this.status(text, "error");
        this.messages.push({ role: "Status", text });
        this.render();
    }
    private scheduleRender() {
        if (this.rendering) return;
        this.rendering = true;
        requestAnimationFrame(() => {
            this.rendering = false;
            this.render();
        });
    }
    private render() {
        const log = element("chat-messages");
        const nearBottom =
            log.scrollHeight - log.scrollTop - log.clientHeight < 80;
        log.replaceChildren();
        if (!this.messages.length) {
            const empty = document.createElement("p");
            empty.className = "chat-empty";
            empty.textContent =
                "Ask about Lua, PocketLua or C++. Connect and sign in with your GitHub Copilot account to start.";
            log.append(empty);
        }
        for (const message of this.messages) {
            const item = document.createElement("article");
            item.className = `chat-message ${message.role.toLowerCase()}`;
            const author = document.createElement("strong");
            author.textContent = message.role;
            item.append(author);
            const parts = message.text.split(/```[^\n]*\n([\s\S]*?)```/g);
            parts.forEach((text, index) => {
                if (index % 2 === 0) {
                    const prose = document.createElement("p");
                    prose.textContent = text;
                    item.append(prose);
                } else {
                    const pre = document.createElement("pre");
                    const code = document.createElement("code");
                    code.textContent = text;
                    pre.append(code);
                    item.append(pre);
                    const copy = document.createElement("button");
                    copy.textContent = "Copy code";
                    copy.onclick = () =>
                        void copyText(text).catch((error) => this.error(error));
                    const insert = document.createElement("button");
                    insert.textContent = "Insert at cursor";
                    insert.disabled = this.busy;
                    insert.onclick = () => {
                        try {
                            this.host.insert(text);
                        } catch (error) {
                            this.error(error);
                        }
                    };
                    item.append(copy, insert);
                }
            });
            log.append(item);
        }
        if (nearBottom) log.scrollTop = log.scrollHeight;
    }
}
async function copyText(text: string) {
    if (navigator.clipboard?.writeText)
        return navigator.clipboard.writeText(text);
    const field = document.createElement("textarea");
    field.value = text;
    document.body.append(field);
    field.select();
    const copied = document.execCommand("copy");
    field.remove();
    if (!copied) throw new Error("Select the code and use Ctrl+C to copy it.");
}
