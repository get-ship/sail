#!/usr/bin/env bun
/**
 * SPEC-L1-1 structure codemod (idempotent).
 *
 *   collapsible → accordion
 *   (details/summary L1 usages must be rewritten to accordion by hand —
 *    freeform native <details>/<summary> HTML is still legal.)
 *
 * Usage: bun scripts/l1-1-codemod.mjs [--dry-run] [paths…]
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
    else if (e.name.endsWith(".sail") || e.name.endsWith(".ts") || e.name.endsWith(".mjs"))
      yield p;
  }
}

let n = 0;
for (const root of targets) {
  for await (const p of walk(root)) {
    if (p.includes(".gen.ts") || p.includes(".deck.stage")) continue;
    let text = await readFile(p, "utf8");
    const orig = text;
    text = text.replace(/<\/?collapsible\b/g, (m) => m.replace("collapsible", "accordion"));
    text = text.replace(/\bsail-collapsible\b/g, "sail-accordion");
    if (text !== orig) {
      n++;
      console.log(dry ? `would change ${p}` : `changed ${p}`);
      if (!dry) await writeFile(p, text);
    }
  }
}
console.log(dry ? `dry-run: ${n} files` : `wrote ${n} files`);
