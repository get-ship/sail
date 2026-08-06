#!/usr/bin/env bun
/**
 * BLOCKS-1 codemod: four blocks + arm syntax.
 *
 * Rewrites:
 *   as="item" on <for> → in-brace `{items as item}`
 *   sibling <else>/<else-if>/<case>/<default> → `{… => …}` arms
 *   while-loading + while-error + if kind==='ok' → <match>
 *
 * Usage: bun scripts/blocks1-codemod.mjs [--dry-run] [paths…]
 */
import { readdir, readFile, writeFile, stat } from "node:fs/promises";
import { join, extname } from "node:path";

const dry = process.argv.includes("--dry-run");
const roots = process.argv.slice(2).filter((a) => a !== "--dry-run");
const targets = roots.length
  ? roots
  : [
      "apps",
      "packages",
      "crates",
      "tests",
      "skills",
    ];

async function* walk(dir) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const e of entries) {
    if (e.name === "node_modules" || e.name === "target" || e.name === "dist" || e.name.startsWith("."))
      continue;
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* walk(p);
    else if (
      e.name.endsWith(".sail") ||
      e.name === "snippets.ts" ||
      (e.name.endsWith(".md") && !e.name.startsWith("RESULT-"))
    )
      yield p;
  }
}

function rewriteForAs(src) {
  // <for {ITER} as="VAR" …>  or as='VAR'
  // Also index="I" / index='I'
  return src.replace(
    /<for\s+\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}\s+as=(["'])(\w+)\2([^>]*)>/g,
    (_m, iter, _q, asName, rest) => {
      let r = rest;
      let idx = null;
      r = r.replace(/\s+index=(["'])(\w+)\1/, (_m2, _q2, i) => {
        idx = i;
        return "";
      });
      const binding = idx ? `${asName}, ${idx}` : asName;
      return `<for {${iter.trim()} as ${binding}}${r}>`;
    },
  );
}

/** Convert a sibling clause element to an arm brace. */
function clauseToArm(tag, openAttrs, body) {
  const t = tag.toLowerCase();
  if (t === "else") {
    return `{else => ${body.trim()}}`;
  }
  if (t === "default") {
    return `{else => ${body.trim()}}`;
  }
  if (t === "else-if") {
    // primary brace on else-if
    const m = openAttrs.match(/\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}/);
    const cond = m ? m[1].trim() : "";
    return `{else-if {${cond}} => ${body.trim()}}`;
  }
  if (t === "case") {
    const m = openAttrs.match(/\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}/);
    let pat = m ? m[1].trim() : "";
    // {'info'} or "info" → keep; bare ok etc.
    // Strip wrapping quotes form that is already expression.
    return `{${pat} => ${body.trim()}}`;
  }
  return null;
}

function rewriteSiblingClauses(src) {
  // Match <else-if …>…</else-if>, <else>…</else>, <case …>…</case>, <default>…</default>
  // Non-greedy body; allow nesting by iterative passes.
  let prev;
  let out = src;
  const tags = ["else-if", "case", "default", "else"];
  for (let pass = 0; pass < 20; pass++) {
    prev = out;
    for (const tag of tags) {
      const re = new RegExp(
        `<${tag}(\\s[^>]*)?>([\\s\\S]*?)<\\/${tag}>`,
        "gi",
      );
      out = out.replace(re, (_m, attrs, body) => {
        // Don't rewrite if body still contains same tag unclosed mess — body may have arms already
        const arm = clauseToArm(tag, attrs || "", body);
        return arm ?? _m;
      });
    }
    if (out === prev) break;
  }
  return out;
}

function rewriteWhileTrio(src) {
  // Pattern A: while-loading + while-error + if kind === 'ok' sharing same signal
  // Also tolerate while-error before loading, and as= placement.
  const re =
    /<while-loading\s+\{(\w+)\}\s*>([\s\S]*?)<\/while-loading>\s*<while-error(?:\s+as=(["'])(\w+)\3)?\s+\{(\w+)\}\s*(?:as=(["'])(\w+)\6)?\s*>([\s\S]*?)<\/while-error>\s*<if\s+\{\s*(\w+)\(\)\.kind\s*===\s*['"]ok['"]\s*\}\s*>([\s\S]*?)<\/if>/g;

  let out = src.replace(
    re,
    (
      _m,
      sigL,
      loadBody,
      _q1,
      as1,
      sigE,
      _q2,
      as2,
      okSig,
      okBody,
    ) => {
      if (sigL !== sigE || sigL !== okSig) return _m;
      const errVar = as1 || as2 || "err";
      // Prefer binding ok payload when body uses sig().value
      let body = okBody;
      const valueRe = new RegExp(`${sigL}\\(\\)\\.value`, "g");
      const usesValue = valueRe.test(body);
      let okArm;
      if (usesValue) {
        body = body.replace(valueRe, "value");
        okArm = `{ok as value => ${body.trim()}}`;
      } else {
        okArm = `{ok => ${body.trim()}}`;
      }
      return `<match {${sigL}()}>\n  {loading => ${loadBody.trim()}}\n  {error as ${errVar} => ${errBodyTrim(arguments)}}\n  ${okArm}\n</match>`;
    },
  );

  // Fix the broken replace above - redo properly
  out = src;
  out = out.replace(
    /<while-loading\s+\{(\w+)\}\s*>([\s\S]*?)<\/while-loading>\s*<while-error((?:\s+[^>]*)?)>([\s\S]*?)<\/while-error>\s*<if\s+\{\s*(\w+)\(\)\.kind\s*===\s*['"]ok['"]\s*\}\s*>([\s\S]*?)<\/if>/g,
    (_m, sigL, loadBody, errAttrs, errBody, okSig, okBody) => {
      const sigE = (errAttrs.match(/\{(\w+)\}/) || [])[1];
      if (!sigE || sigL !== sigE || sigL !== okSig) return _m;
      const asM = errAttrs.match(/as=(["'])(\w+)\1/);
      const errVar = asM ? asM[2] : "err";
      let body = okBody;
      const valueRe = new RegExp(`${sigL}\\(\\)\\.value`, "g");
      let okArm;
      if (valueRe.test(body)) {
        body = body.replace(valueRe, "value");
        okArm = `{ok as value => ${body.trim()}}`;
      } else {
        okArm = `{ok => ${body.trim()}}`;
      }
      return `<match {${sigL}()}>\n  {loading => ${loadBody.trim()}}\n  {error as ${errVar} => ${errBody.trim()}}\n  ${okArm}\n</match>`;
    },
  );

  // while-error before while-loading order
  out = out.replace(
    /<while-error((?:\s+[^>]*)?)>([\s\S]*?)<\/while-error>\s*<while-loading\s+\{(\w+)\}\s*>([\s\S]*?)<\/while-loading>\s*<if\s+\{\s*(\w+)\(\)\.kind\s*===\s*['"]ok['"]\s*\}\s*>([\s\S]*?)<\/if>/g,
    (_m, errAttrs, errBody, sigL, loadBody, okSig, okBody) => {
      const sigE = (errAttrs.match(/\{(\w+)\}/) || [])[1];
      if (!sigE || sigL !== sigE || sigL !== okSig) return _m;
      const asM = errAttrs.match(/as=(["'])(\w+)\1/);
      const errVar = asM ? asM[2] : "err";
      let body = okBody;
      const valueRe = new RegExp(`${sigL}\\(\\)\\.value`, "g");
      let okArm;
      if (valueRe.test(body)) {
        body = body.replace(valueRe, "value");
        okArm = `{ok as value => ${body.trim()}}`;
      } else {
        okArm = `{ok => ${body.trim()}}`;
      }
      return `<match {${sigL}()}>\n  {loading => ${loadBody.trim()}}\n  {error as ${errVar} => ${errBody.trim()}}\n  ${okArm}\n</match>`;
    },
  );

  // Lone while-loading
  out = out.replace(
    /<while-loading\s+\{(\w+)\}\s*>([\s\S]*?)<\/while-loading>/g,
    (_m, sig, body) =>
      `<match {${sig}()}>\n  {loading => ${body.trim()}}\n  {else => }\n</match>`,
  );

  // Lone while-error
  out = out.replace(
    /<while-error((?:\s+[^>]*)?)>([\s\S]*?)<\/while-error>/g,
    (_m, attrs, body) => {
      const sig = (attrs.match(/\{(\w+)\}/) || [])[1] || "sig";
      const asM = attrs.match(/as=(["'])(\w+)\1/);
      const errVar = asM ? asM[2] : "err";
      return `<match {${sig}()}>\n  {error as ${errVar} => ${body.trim()}}\n  {else => }\n</match>`;
    },
  );

  return out;
}

function errBodyTrim() {
  return "";
}

function rewriteFile(src) {
  let out = src;
  out = rewriteForAs(out);
  out = rewriteWhileTrio(out);
  out = rewriteSiblingClauses(out);
  // residual as= on while already gone; strip residual for as= on for if any missed
  out = out.replace(
    /<for\s+\{([^}]+)\}\s+as=(["'])(\w+)\2/g,
    "<for {$1 as $3}",
  );
  return out;
}

let scanned = 0;
let changed = 0;
for (const root of targets) {
  for await (const file of walk(root)) {
    // Prefer .sail; also docs/snippets that embed sail
    const text = await readFile(file, "utf8");
    if (
      !text.includes("<for") &&
      !text.includes("<while-") &&
      !text.includes("<else") &&
      !text.includes("<case") &&
      !text.includes("<default")
    ) {
      continue;
    }
    scanned++;
    const next = rewriteFile(text);
    if (next !== text) {
      changed++;
      if (dry) console.log("would rewrite", file);
      else {
        await writeFile(file, next);
        console.log("rewrote", file);
      }
    }
  }
}
console.error(`scanned=${scanned} changed=${changed} dry_run=${dry}`);
