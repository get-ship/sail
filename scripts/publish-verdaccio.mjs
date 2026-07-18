#!/usr/bin/env bun
// Cut a version and publish the @ship/sail-* packages to a registry, in
// dependency order. Defaults to --dry-run (no network write). A real publish
// requires --publish AND --yes-i-mean-it on top of that — this script alone
// never decides to go live.
//
// Usage:
//   bun scripts/publish-verdaccio.mjs --dry-run [--version 0.3.0] [--registry http://localhost:4873]
//   bun scripts/publish-verdaccio.mjs --publish --yes-i-mean-it --version 0.3.0 --registry http://localhost:4873
//
// No version bump is hardcoded: pass --version explicitly, or omit it to
// dry-run against whatever version is currently in each package.json.
//
// Packing/publishing uses `bun pm pack` / `bun publish` (npm-registry
// compatible) because this environment has no `npm` binary. `npm publish
// --dry-run` requires registry auth even for a dry run in some npm
// versions; `bun pm pack --dry-run` performs the equivalent no-network
// tarball-contents check. If `npm` is present on PATH, prefer it.

import { execSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, copyFileSync, unlinkSync, mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

const args = process.argv.slice(2);
const flag = (name) => args.includes(`--${name}`);
const opt = (name) => {
  const i = args.indexOf(`--${name}`);
  return i === -1 ? undefined : args[i + 1];
};

const dryRun = !flag("publish");
const confirmed = flag("yes-i-mean-it");
const version = opt("version");
const registry = opt("registry") ?? "http://localhost:4873";

if (!dryRun && !confirmed) {
  console.error(
    "Refusing to publish for real: --publish requires --yes-i-mean-it too."
  );
  process.exit(1);
}

const hasNpm = spawnSync("npm", ["--version"], { stdio: "ignore" }).status === 0;
const packManager = hasNpm ? "npm" : "bun";

// Dependency-ordered @ship/sail-* packages (tree-sitter-sail excluded: private).
// deps lists only in-scope @ship/sail-* / @ship/tokens workspace deps (the
// ones that need workspace:* -> real range rewriting before publish).
const PACKAGES = [
  { name: "@ship/tokens", dir: "packages/sail/tokens", deps: [] },
  { name: "@ship/sail-compiler-wasm", dir: "packages/sail/compiler-wasm", deps: [], build: buildWasm },
  { name: "@ship/sail-runtime", dir: "packages/sail/runtime", deps: ["@ship/tokens"] },
  { name: "@ship/sail-lsp-browser", dir: "packages/deck/lsp-browser", deps: ["@ship/sail-compiler-wasm"] },
  { name: "@ship/sail-dom", dir: "packages/sail/dom", deps: ["@ship/sail-runtime"] },
  { name: "@ship/sail-tutorial", dir: "packages/sail/tutorial", deps: ["@ship/sail-runtime", "@ship/tokens"] },
  { name: "@ship/sail-primitives", dir: "packages/components/sail/primitives", deps: ["@ship/sail-dom", "@ship/sail-runtime", "@ship/tokens"], build: buildCems },
  { name: "@ship/sail-canvas", dir: "packages/components/sail/canvas", deps: ["@ship/sail-dom", "@ship/sail-runtime"], build: buildCems },
  { name: "@ship/sail-charts", dir: "packages/components/sail/charts", deps: ["@ship/sail-dom", "@ship/sail-runtime"], build: buildCems },
  { name: "@ship/sail-mermaid", dir: "packages/components/sail/mermaid", deps: ["@ship/sail-runtime", "@ship/sail-dom"], build: buildCems },
  { name: "@ship/sail-eval", dir: "packages/deck/eval", deps: ["@ship/sail-dom", "@ship/sail-runtime"] },
  { name: "@ship/sail-chat-elements", dir: "packages/components/sail/chat-elements", deps: ["@ship/sail-runtime", "@ship/sail-dom", "@ship/sail-primitives"], build: buildCems },
  { name: "@ship/sail-media", dir: "packages/components/sail/media", deps: ["@ship/sail-dom", "@ship/sail-primitives", "@ship/sail-runtime"], build: buildCems },
  { name: "@ship/sail-code-editor", dir: "packages/components/sail/code-editor", deps: ["@ship/sail-runtime", "@ship/sail-dom", "@ship/sail-lsp-browser", "@ship/sail-primitives"], build: buildCems },
  { name: "@ship/sail-viewer", dir: "packages/components/sail/viewer", deps: ["@ship/sail-compiler-wasm", "@ship/sail-dom", "@ship/sail-primitives", "@ship/sail-runtime"], build: buildCems },
];

const versionOf = new Map();
for (const pkg of PACKAGES) {
  const pj = JSON.parse(readFileSync(path.join(repoRoot, pkg.dir, "package.json"), "utf8"));
  versionOf.set(pkg.name, version ?? pj.version);
}

function buildWasm() {
  console.log("  building wasm (bun run wasm)...");
  execSync("bun run wasm", { cwd: repoRoot, stdio: "inherit" });
}

let cemsBuilt = false;
function buildCems() {
  if (cemsBuilt) return;
  console.log("  building component-pack dist/custom-elements.json (cargo run -p deck-cli -- build-assets)...");
  execSync("cargo run -p deck-cli --quiet -- build-assets", { cwd: repoRoot, stdio: "inherit" });
  cemsBuilt = true;
}

function rewriteWorkspaceDeps(pkgJsonPath, deps) {
  const raw = readFileSync(pkgJsonPath, "utf8");
  const pj = JSON.parse(raw);
  for (const dep of deps) {
    if (pj.dependencies?.[dep] === "workspace:*") {
      pj.dependencies[dep] = `^${versionOf.get(dep)}`;
    }
  }
  if (version) pj.version = version;
  writeFileSync(pkgJsonPath, JSON.stringify(pj, null, 2) + "\n");
}

const results = [];
// Backups live OUTSIDE any package dir: packages without a "files"
// allowlist (e.g. sail-canvas, sail-mermaid, sail-code-editor,
// sail-lsp-browser) publish everything not gitignored, so a same-dir
// backup file would leak into their tarball.
const tmpRoot = mkdtempSync(path.join(os.tmpdir(), "publish-verdaccio-"));

for (const pkg of PACKAGES) {
  console.log(`\n=== ${pkg.name} (${pkg.dir}) ===`);
  const dir = path.join(repoRoot, pkg.dir);
  const pkgJsonPath = path.join(dir, "package.json");
  const backupPath = path.join(tmpRoot, `${pkg.name.replace(/[/@]/g, "_")}.package.json`);

  if (pkg.build) {
    try {
      pkg.build();
    } catch (err) {
      results.push({ name: pkg.name, ok: false, reason: `build step failed: ${err.message}` });
      continue;
    }
  }

  copyFileSync(pkgJsonPath, backupPath);
  try {
    rewriteWorkspaceDeps(pkgJsonPath, pkg.deps);

    let cmd;
    if (dryRun) {
      cmd =
        packManager === "npm"
          ? ["npm", ["publish", "--dry-run", "--registry", registry]]
          : ["bun", ["pm", "pack", "--dry-run"]];
    } else {
      cmd =
        packManager === "npm"
          ? ["npm", ["publish", "--registry", registry]]
          : ["bun", ["publish", "--registry", registry]];
    }

    const res = spawnSync(cmd[0], cmd[1], { cwd: dir, encoding: "utf8" });
    const output = (res.stdout ?? "") + (res.stderr ?? "");
    console.log(output.trim());
    results.push({ name: pkg.name, ok: res.status === 0, output });
  } finally {
    copyFileSync(backupPath, pkgJsonPath);
    unlinkSync(backupPath);
  }
}

rmSync(tmpRoot, { recursive: true, force: true });

console.log("\n=== publish-verdaccio summary ===");
console.log(`mode: ${dryRun ? "DRY-RUN (no network write)" : "REAL PUBLISH"}  pack manager: ${packManager}  registry: ${registry}`);
let anyFail = false;
for (const r of results) {
  console.log(`  ${r.ok ? "PASS" : "FAIL"}  ${r.name}${r.reason ? ` — ${r.reason}` : ""}`);
  if (!r.ok) anyFail = true;
}
process.exit(anyFail ? 1 : 0);
