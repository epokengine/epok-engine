import { build } from "esbuild";
import { mkdir, copyFile } from "node:fs/promises";
const outdir = "../../resources/code-editor";
await mkdir(outdir, { recursive: true });
await build({
    entryPoints: {
        app: "src/app.ts",
        "editor.worker":
            "node_modules/monaco-editor/esm/vs/editor/editor.worker.js",
    },
    outdir,
    bundle: true,
    format: "esm",
    minify: true,
    target: "es2022",
    loader: { ".ttf": "file" },
    assetNames: "[name]",
    legalComments: "linked",
});
await copyFile("src/index.html", `${outdir}/index.html`);
await copyFile("src/style.css", `${outdir}/style.css`);
await copyFile(
    "node_modules/monaco-editor/LICENSE",
    `${outdir}/LICENSE-monaco.txt`,
);
await copyFile(
    "node_modules/monaco-editor/ThirdPartyNotices.txt",
    `${outdir}/ThirdPartyNotices-monaco.txt`,
);
