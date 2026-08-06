#!/usr/bin/env bun
/**
 * SPEC-L1-1 WI2 — promote attribute-floor skeleton to L1 element (idempotent).
 *
 *   <span skeleton …>…</span>  →  <skeleton …>…</skeleton>
 *   querySelector('[skeleton]') / 'span[skeleton]' → '.sail-skeleton'
 *
 * Usage: bun scripts/l1-skeleton-codemod.mjs [--dry-run] [paths…]
 */
import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

const dry = process.argv.includes("--dry-run");
const roots = process.argv.slice(2).filter((a) => a !== "--dry-run");
const targets = roots.length ? roots : ["apps", "packages", "crates", "tests"];

async function* walk(dir) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const e of entries) {
    if (
      e.name === "node_modules" ||
      e.name === "target" ||
      e.name === "dist" ||
      e.name.startsWith(".")
    )
      continue;
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* walk(p);
    else if (
      e.name.endsWith(".sail") ||
      e.name.endsWith(".ts") ||
      e.name.endsWith(".mjs") ||
      e.name.endsWith(".md")
    )
      yield p;
  }
}

function rewrite(text) {
  let out = text;
  // Empty: <span skeleton attrs></span>
  out = out.replace(
    /<span\s+skeleton\b([^>]*)>\s*<\/span>/g,
    (_m, attrs) => `<skeleton${attrs}></skeleton>`,
  );
  // With body (e.g. style block): <span skeleton attrs>…</span>
  out = out.replace(
    /<span\s+skeleton\b([^>]*)>([\s\S]*?)<\/span>/g,
    (_m, attrs, body) => `<skeleton${attrs}>${body}</skeleton>`,
  );
  // Residual open tag only
  out = out.replace(/<span\s+skeleton\b/g, "<skeleton");
  // Selector probes
  out = out.replace(/document\.querySelector(All)?\(\s*['"]span\[skeleton\]['"]\s*\)/g, (_m, all) =>
    all ? `document.querySelectorAll('.sail-skeleton')` : `document.querySelector('.sail-skeleton')`,
  );
  out = out.replace(/document\.querySelector(All)?\(\s*['"]\[skeleton\]['"]\s*\)/g, (_m, all) =>
    all ? `document.querySelectorAll('.sail-skeleton')` : `document.querySelector('.sail-skeleton')`,
  );
  out = out.replace(/querySelector(All)?\(\s*['"]\[skeleton\]['"]\s*\)/g, (_m, all) =>
    all ? `querySelectorAll('.sail-skeleton')` : `querySelector('.sail-skeleton')`,
  );
  out = out.replace(/querySelector(All)?\(\s*['"]span\[skeleton\]['"]\s*\)/g, (_m, all) =>
    all ? `querySelectorAll('.sail-skeleton')` : `querySelector('.sail-skeleton')`,
  );
  return out;
}

let n = 0;
for (const root of targets) {
  for await (const p of walk(root)) {
    if (p.includes(".gen.ts") || p.includes(".deck.stage")) continue;
    // Skip this codemod script and the retired floor notes in scratch (not under roots).
    if (p.endsWith("l1-skeleton-codemod.mjs")) continue;
    let text = await readFile(p, "utf8");
    const next = rewrite(text);
    if (next !== text) {
      n++;
      console.log(dry ? `would change ${p}` : `changed ${p}`);
      if (!dry) await writeFile(p, next);
    }
  }
}
console.log(dry ? `dry-run: ${n} files` : `wrote ${n} files`);
