// Exercises the actual Monaco WebView hosted by the production Epok binary.
// No browser mock, CDN, user project or user editor preferences are used.
import { chromium } from "playwright-core";
import { spawn, spawnSync } from "node:child_process";
import {
    mkdir,
    readFile,
    writeFile,
    copyFile,
    access,
    open,
} from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import net from "node:net";
import assert from "node:assert/strict";

const root = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../..",
);
const executable =
    process.env.EPOK_TEST_EDITOR ??
    path.join(root, "target/debug/epok-editor.exe");
const evidence = path.join(root, "artifacts/code-editor/native");
const project = path.join(evidence, "QA Project");
const local = path.join(evidence, "local");
const userRoot = path.join(
    process.env.LOCALAPPDATA ?? process.env.XDG_DATA_HOME,
    "Epok",
);
const exists = async (file) =>
    access(file).then(
        () => true,
        () => false,
    );
const results = { checks: [], pageErrors: [] };
const embedded = process.argv.includes("--pocket");
const reportName = embedded ? "pocket-report.json" : "report.json";
const screenshot = (name) =>
    path.join(evidence, `${embedded ? "pocket-" : ""}${name}.png`);
const check = (name, value) => {
    assert.ok(value, name);
    results.checks.push(name);
    console.log(`PASS ${name}`);
};
await mkdir(evidence, { recursive: true });
await mkdir(path.join(local, "Epok"), { recursive: true });
function run(args, env = process.env) {
    const result = spawnSync(executable, args, {
        cwd: root,
        env,
        encoding: "utf8",
        windowsHide: true,
    });
    if (result.status !== 0)
        throw new Error(
            `${args.join(" ")}: ${result.stderr}\n${result.stdout}`,
        );
    return result.stdout;
}
if (process.argv.includes("--install"))
    for (const tool of ["lua_ls", "clangd", "copilot"]) {
        console.log(run(["--install-code-tool", tool]));
    }
if (await exists(path.join(userRoot, "CodeEditor.epokprefs")))
    await copyFile(
        path.join(userRoot, "CodeEditor.epokprefs"),
        path.join(local, "Epok/CodeEditor.epokprefs"),
    );
const env = {
    ...process.env,
    LOCALAPPDATA: local,
    XDG_DATA_HOME: local,
    EPOK_EDITOR_HOME: root,
};
if (!(await exists(path.join(project, "Code Editor QA.epokproject"))))
    run(
        [
            "--create-project",
            project,
            "--name",
            "Code Editor QA",
            "--template",
            "basic",
        ],
        env,
    );
await mkdir(path.join(project, "assets/scripts"), { recursive: true });
const luaFile = path.join(project, "assets/scripts/CodeProbe.lua");
const cppFile = path.join(project, "assets/scripts/CppProbe.hpp");
await writeFile(
    luaFile,
    "---@class CodeProbe : epok.Actor3D\nlocal CodeProbe = epok.Actor3D:extend()\nfunction CodeProbe:tick(delta_seconds)\nend\nreturn CodeProbe\n",
);
await writeFile(
    cppFile,
    '#pragma once\n#include "epok.hpp"\ninline void cpp_probe() {}\n',
);
const { readdir } = await import("node:fs/promises");
const pockets = await readdir(path.join(project, "assets/PocketStation")).catch(
    () => [],
);
if (!pockets.some((name) => name.endsWith(".epokasset")))
    run(
        [
            "--project",
            project,
            "--new-pocketstation-app",
            "QA Pocket",
            "--pocketstation-template",
            "blank",
        ],
        env,
    );
const listener = net.createServer();
await new Promise((resolve) => listener.listen(0, "127.0.0.1", resolve));
const port = listener.address().port;
await new Promise((resolve) => listener.close(resolve));
const log = await open(path.join(evidence, "editor.log"), "w");
const child = spawn(
    executable,
    [
        "--project",
        project,
        ...(embedded
            ? [
                  "--open-pocketstation",
                  "QAPocket",
                  "--pocketstation-tab",
                  "code",
              ]
            : ["--open-code", "assets/scripts/CodeProbe.lua"]),
        "--window-size",
        "1440x900",
    ],
    {
        cwd: root,
        env: {
            ...env,
            WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
            EPOK_CODE_VIEW_DEBUG: "1",
        },
        stdio: ["ignore", log.fd, log.fd],
        windowsHide: false,
    },
);
console.log(`Native Epok pid ${child.pid}, WebView CDP port ${port}`);
async function until(fn, label, timeout = 60000) {
    const end = Date.now() + timeout;
    let last;
    while (Date.now() < end) {
        if (child.exitCode !== null)
            throw new Error(
                `Editor exited (${child.exitCode}): ${await readFile(path.join(evidence, "editor.log"), "utf8")}`,
            );
        try {
            const value = await fn();
            if (value) return value;
        } catch (error) {
            last = error;
        }
        await new Promise((resolve) => setTimeout(resolve, 150));
    }
    throw new Error(`${label} timed out: ${last ?? ""}`);
}
let browser, page;
try {
    await until(async () => {
        const response = await fetch(`http://127.0.0.1:${port}/json/version`);
        return response.ok;
    }, "WebView2 debugging endpoint");
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
    page = await until(
        () =>
            browser
                .contexts()
                .flatMap((c) => c.pages())
                .find((p) => p.url().includes("epok-code")),
        "Monaco page",
    );
    page.on("pageerror", (error) => results.pageErrors.push(String(error)));
    page.setDefaultTimeout(20000);
    await page.locator(".tab").first().waitFor();
    await page.evaluate(() => {
        window.__qaLspTrace = [];
        window.__qaRuns = [];
        const trace = (direction, server, message) => {
            if (!["lua", "pocket", "cpp"].includes(server)) return;
            const summary = JSON.parse(JSON.stringify(message));
            if (summary.result?.items) {
                summary.result = {
                    isIncomplete: summary.result.isIncomplete,
                    count: summary.result.items.length,
                    items: summary.result.items.slice(0, 4),
                };
            }
            window.__qaLspTrace.push({ direction, server, message: summary });
            if (window.__qaLspTrace.length > 100) window.__qaLspTrace.shift();
        };
        const receive = window.epokReceive;
        window.epokReceive = (value) => {
            if (value.id && value.result?.message?.startsWith("Built "))
                window.__qaRuns.push(value);
            if (value.event === "lsp") trace("in", value.server, value.message);
            receive(value);
        };
    });
    check(
        "production WebView loaded bundled Monaco",
        (await page.locator(".monaco-editor").count()) === 1,
    );
    check(
        "source tree lists Lua and C++",
        (await page
            .locator(".tree-file")
            .filter({ hasText: "CodeProbe.lua" })
            .count()) === 1 &&
            (await page
                .locator(".tree-file")
                .filter({ hasText: "CppProbe.hpp" })
                .count()) === 1,
    );
    if (embedded) {
        await until(
            async () =>
                /main\.plua/.test(
                    await page.locator(".tab.active").textContent(),
                ),
            "PocketStation embedded source",
        );
        check(
            "PocketStation Code tab hosts the production Monaco workspace",
            true,
        );
        await page.screenshot({
            path: path.join(evidence, "pocket-embedded.png"),
            timeout: 5000,
        });
        await page.locator(".tab.active .close").click();
        await page
            .locator(".tree-file")
            .filter({ hasText: "CodeProbe.lua" })
            .click();
        await until(
            async () =>
                /CodeProbe\.lua/.test(
                    await page.locator(".tab.active").textContent(),
                ) && (await page.locator(".tab").count()) === 1,
            "first gameplay source tab",
        );
    }
    if (process.argv.includes("--pause"))
        await new Promise((resolve) => setTimeout(resolve, 30000));
    const focus = async () => {
        await page
            .locator(".monaco-editor")
            .click({ position: { x: 100, y: 80 } });
        await page.keyboard.press("Control+End");
    };
    await focus();
    await page.keyboard.insertText("\n-- saved through native Monaco QA\n");
    await page.locator(".tab.dirty").waitFor();
    await page.locator("#save").click();
    await until(
        async () =>
            (await readFile(luaFile, "utf8")).includes(
                "-- saved through native Monaco QA",
            ),
        "native source save",
    );
    check("Save persists the buffer through the native bridge", true);
    await page
        .locator(".tree-file")
        .filter({ hasText: "CppProbe.hpp" })
        .click();
    await until(
        async () => (await page.locator(".tab").count()) === 2,
        "second source tab",
    );
    check(
        "multiple files have independent tabs",
        (await page.locator(".tab").count()) === 2,
    );
    await focus();
    await page.keyboard.insertText("\n// discard me\n");
    await page.locator(".tab.active .close").click();
    await page.locator("#confirm-close[open]").waitFor();
    await page.locator("#close-cancel").click();
    check(
        "Cancel preserves an unsaved tab",
        (await page.locator(".tab").count()) === 2,
    );
    await page.locator(".tab.active .close").click();
    await page.locator("#close-discard").click();
    await until(
        async () => (await page.locator(".tab").count()) === 1,
        "discarded source tab",
    );
    check(
        "Discard leaves the C++ source unchanged",
        !(await readFile(cppFile, "utf8")).includes("discard me"),
    );
    async function completion(file, text, expected, name) {
        await page.locator(".tree-file").filter({ hasText: file }).click();
        await until(
            async () =>
                (await page.locator(".tab.active").textContent()).includes(
                    file,
                ),
            `${file} active tab`,
        );
        await focus();
        if (file === "CodeProbe.lua") {
            // Complete inside the callback, where Lua permits an expression;
            // Lua forbids statements following the chunk's final return.
            await page.keyboard.press("Control+Home");
            await page.keyboard.press("ArrowDown");
            await page.keyboard.press("ArrowDown");
            await page.keyboard.press("ArrowDown");
            await page.keyboard.press("Home");
            await page.keyboard.insertText(`    ${text.trimStart()}\n`);
            await page.keyboard.press("ArrowLeft");
        } else await page.keyboard.insertText(text);
        await page.keyboard.press("Control+Space");
        const rows = page.locator(".suggest-widget.visible .monaco-list-row");
        let retried = Date.now();
        await until(
            async () => {
                if (expected.test((await rows.allTextContents()).join("\n")))
                    return true;
                if (Date.now() - retried > 1000) {
                    retried = Date.now();
                    await page.keyboard.press("Escape");
                    await page.keyboard.press("Control+Space");
                }
                return false;
            },
            `${name} completion`,
            45000,
        );
        check(name, true);
        results[`${name}Suggestions`] = await rows.allTextContents();
        await page.screenshot({
            path: screenshot(name.replaceAll(" ", "-")),
            timeout: 5000,
        });
        await page.keyboard.press("Escape");
        for (
            let i = 0;
            i < 5 && (await page.locator(".tab.active.dirty").count());
            i++
        )
            await page.keyboard.press("Control+z");
        check(
            `${name} probe is completely undone`,
            (await page.locator(".tab.active.dirty").count()) === 0,
        );
    }
    await completion(
        "CodeProbe.lua",
        "\nepok.",
        /Actor3D|Vector3|time|input/,
        "Lua engine API",
    );
    const pocket = page
        .locator("#tree summary")
        .filter({ hasText: "QA Pocket" });
    await pocket.click();
    await completion(
        "main.plua",
        "\nscreen.",
        /pixel|clear|sprite/,
        "PocketLua API",
    );
    await page.locator("#new-source").selectOption("pocket");
    await until(
        async () =>
            /routines\d+\.s/.test(
                await page.locator(".tab.active").textContent(),
            ),
        "new embedded assembly source",
    );
    check("New Pocket Assembly adds and opens a source inside the app", true);
    const newAssembly = (await page.locator(".tab.active").textContent()).match(
        /routines\d+\.s/,
    )[0];
    await completion(
        "CppProbe.hpp",
        "\ninline void completion_probe() { epok::Act",
        /Actor3D/,
        "C++ engine API",
    );
    await page.locator("#save-all").click();
    await page.locator("#settings-open").click();
    check(
        "easy setup exposes all three portable tool paths",
        (await page.locator("#tool-lua_ls").inputValue()) !== "" &&
            (await page.locator("#tool-clangd").inputValue()) !== "" &&
            (await page.locator("#tool-copilot").inputValue()) !== "",
    );
    await page.locator("#copilot-enabled").check();
    await page.locator("#apply-settings").click();
    await until(
        async () =>
            /Copilot:/.test(
                await page.locator("#copilot-status").textContent(),
            ),
        "official Copilot server status",
        45000,
    );
    check(
        "official Copilot server initializes and reports its sign-in state",
        true,
    );
    results.copilotStatus = await page.locator("#copilot-status").textContent();
    await page.screenshot({
        path: screenshot("setup"),
        timeout: 5000,
    });
    await page.locator("#setup button").filter({ hasText: "Close" }).click();
    if (process.argv.includes("--inline")) {
        await page
            .locator(".tree-file")
            .filter({ hasText: "CodeProbe.lua" })
            .click();
        await focus();
        const original = await readFile(luaFile, "utf8");
        const prompt = "\n-- Fibonacci sequence: 0, 1, 1, 2, 3, 5,";
        await page.keyboard.insertText(prompt);
        const ghost = page.locator(".ghost-text, .ghost-text-decoration");
        await until(
            async () => (await ghost.allTextContents()).join("").trim(),
            "Copilot inline suggestion",
            60000,
        );
        const suggestion = (await ghost.allTextContents())
            .join("")
            .replaceAll("\u00a0", " ")
            .trim();
        await page.screenshot({
            path: screenshot("copilot-inline"),
            timeout: 5000,
        });
        await page.keyboard.press("Tab");
        await page.locator("#save").click();
        await until(
            async () => (await readFile(luaFile, "utf8")).includes(suggestion),
            "accepted Copilot suggestion saved",
        );
        check(
            "Copilot shows a real inline suggestion and Tab accepts it into the source",
            true,
        );
        results.inlineSuggestion = suggestion;
        await focus();
        await page.keyboard.press("Control+z");
        await page.keyboard.press("Control+s");
        await until(
            async () => (await readFile(luaFile, "utf8")) === original + prompt,
            "undo accepted Copilot suggestion",
        );
        check(
            "Undo removes the accepted inline suggestion and retains the typed prompt",
            true,
        );
        // Restore only this synthetic probe through the same editor/save flow.
        await page.keyboard.press("Control+a");
        await page.keyboard.insertText(original);
        await page.keyboard.press("Control+s");
        await until(
            async () => (await readFile(luaFile, "utf8")) === original,
            "restore inline probe source",
        );
    }
    if (process.argv.includes("--chat")) {
        await page
            .locator(".tree-file")
            .filter({ hasText: "CodeProbe.lua" })
            .click();
        await focus();
        await page.locator("#chat-toggle").click();
        await page.locator("#chat-connect").click();
        await until(
            async () =>
                (await page.locator("#chat").getAttribute("data-state")) ===
                "ready",
            "official Copilot ACP chat session",
            120000,
        );
        check("Copilot Chat connects to the official ACP server", true);
        await page.locator("#chat-context").uncheck();
        await page
            .locator("#chat-input")
            .fill(
                "Reply with exactly one fenced Lua code block containing '-- EPOK_CHAT_READY', without any prose. Do not use tools or inspect files.",
            );
        await page.locator("#chat-send").click();
        await until(
            async () =>
                /EPOK_CHAT_READY/.test(
                    await page
                        .locator(".chat-message.copilot code")
                        .textContent(),
                ) &&
                (await page.locator("#chat").getAttribute("data-state")) ===
                    "ready",
            "streamed Copilot chat reply",
            120000,
        );
        check(
            "Copilot Chat streams a real response to a synthetic prompt",
            true,
        );
        const chatFits = await page.evaluate(() => {
            const messages = document
                .getElementById("chat-messages")
                .getBoundingClientRect();
            const code = document
                .querySelector(".chat-message.copilot code")
                .getBoundingClientRect();
            return (
                messages.height >= 100 &&
                code.top >= messages.top &&
                code.bottom <= messages.bottom &&
                ["chat-input", "chat-send", "chat-close"].every((id) => {
                    const bounds = document
                        .getElementById(id)
                        .getBoundingClientRect();
                    return (
                        bounds.left >= 0 &&
                        bounds.top >= 0 &&
                        bounds.right <= innerWidth &&
                        bounds.bottom <= innerHeight
                    );
                })
            );
        });
        check(
            "Chat keeps a readable conversation and all compose controls within the native viewport",
            chatFits,
        );
        await page
            .locator(".chat-message.copilot button")
            .filter({ hasText: "Insert at cursor" })
            .click();
        await page.locator(".tab.active.dirty").waitFor();
        await page.keyboard.press("Control+z");
        await until(
            async () => (await page.locator(".tab.active.dirty").count()) === 0,
            "undo inserted Copilot code",
        );
        check(
            "Chat code inserts into Monaco with a working Undo boundary",
            true,
        );
        await page.screenshot({
            path: screenshot("copilot-chat"),
            timeout: 5000,
        });
        if (process.argv.includes("--pause-chat"))
            await new Promise((resolve) => setTimeout(resolve, 30000));
        await page.locator("#chat-new").click();
        check(
            "New chat clears the visible conversation",
            (await page.locator(".chat-message").count()) === 0,
        );
        await page.locator("#chat-close").click();
    }
    // Run last: opening the device also moves the workspace to its Code tab.
    // Copilot above must be exercised in the original native host for this mode.
    await page.locator(".tree-file").filter({ hasText: newAssembly }).click();
    await until(
        async () =>
            (await page.locator(".tab.active").textContent()).includes(
                newAssembly,
            ),
        "assembly active before F5",
    );
    await focus();
    await page.keyboard.press("F5");
    await until(
        () =>
            page.evaluate(() =>
                window.__qaRuns.some(
                    (value) =>
                        /Built QA Pocket:/.test(value.result?.message) &&
                        !value.result?.error,
                ),
            ),
        "PocketStation F5 Build and Run",
    );
    check(
        "F5 saves the source workspace and builds the current PocketStation app",
        true,
    );
    check(
        "Monaco has no uncaught page errors",
        results.pageErrors.length === 0,
    );
    await writeFile(
        path.join(evidence, reportName),
        JSON.stringify(results, null, 2),
    );
    await page.evaluate(() =>
        window.ipc.postMessage(
            JSON.stringify({ method: "close_application", params: {} }),
        ),
    );
    await new Promise((resolve, reject) => {
        const timer = setTimeout(
            () => reject(new Error("Native editor did not close cleanly")),
            15000,
        );
        child.once("exit", () => {
            clearTimeout(timer);
            resolve();
        });
    });
} catch (error) {
    results.error = String(error);
    if (page)
        results.lspTrace = await page
            .evaluate(() => window.__qaLspTrace)
            .catch(() => null);
    if (page)
        results.message = await page
            .locator("#message")
            .textContent()
            .catch(() => null);
    await writeFile(
        path.join(evidence, reportName),
        JSON.stringify(results, null, 2),
    );
    if (page)
        await page
            .screenshot({
                path: path.join(evidence, "failure.png"),
                timeout: 5000,
            })
            .catch(() => {});
    throw error;
} finally {
    if (child.exitCode === null) child.kill();
    if (browser) await browser.close().catch(() => {});
    await log.close();
}
console.log(`Native code editor evidence: ${evidence}`);
