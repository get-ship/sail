#!/usr/bin/env node
// lint-config-paths — fail committed config that reaches OUTSIDE the repo or
// INTO build output. This is the filter for the failure-mode that the nest
// restructure surfaced by hand (e.g. a component tsconfig's `rootDirs` pointing
// at `.ship/sail-out` — a moving, build-output, cross-concern path that, when
// mispathed, silently resolved types to `any` and masked real errors).
//
// Rules (each a P0; they STACK):
//   absolute-path     — a value like /home/..., ~/..., C:\...
//   escapes-repo      — resolves outside the repo root (a cross-repo reach)
//   into-build-output — resolves into a generated/build dir (.ship .deck dist target gen out-tsc .next)
//   foreign-framework — (sui repo only) a `jsx`/react setting; SUI isn't JSX
//
// Scans: tsconfig*/jsconfig* (rootDirs, paths, references[].path, extends, jsx),
//        package.json (file:/link: deps), Cargo.toml (path = "..." deps).
//
// Usage: node lint-config-paths.mjs [repo-root]   (defaults to cwd)
// Exit 1 if any finding. Drop into pre-commit + CI per repo.

import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs'
import { resolve, dirname, relative } from 'node:path'

const REPO = resolve(process.argv[2] ?? '.')
const SKIP_DIRS = new Set(['node_modules', 'target', '.git', 'dist', '.deck', '.ship', '.artifacts', 'pkg'])
const BUILD_SEGMENTS = new Set(['.ship', '.deck', 'dist', 'target', 'gen', 'out-tsc', 'out', '.next'])
// sui framework repo? (marker: it ships the sui runtime). Gates the jsx fence —
// ship-level repos legitimately wrap React (e.g. TipTap), the framework repo does not.
const IS_SUI = existsSync(resolve(REPO, 'packages/sail/runtime'))

const findings = []
const flag = (file, line, rule, detail) =>
  findings.push({ file: relative(REPO, file), line, rule, detail })

function* walk(dir) {
  for (const e of readdirSync(dir)) {
    const p = resolve(dir, e)
    let st; try { st = statSync(p) } catch { continue }
    if (st.isDirectory()) { if (!SKIP_DIRS.has(e)) yield* walk(p) }
    else yield p
  }
}

// Line number of the first occurrence of `needle` in `text` (1-based, best-effort).
const lineOf = (text, needle) => {
  const i = text.indexOf(needle)
  return i < 0 ? 0 : text.slice(0, i).split('\n').length
}

function checkPath(file, text, value, label) {
  if (typeof value !== 'string' || !value) return
  const line = lineOf(text, value)
  if (/^([/~]|[A-Za-z]:[\\/])/.test(value)) return flag(file, line, 'absolute-path', `${label}: ${value}`)
  const abs = resolve(dirname(file), value)
  if (relative(REPO, abs).startsWith('..')) return flag(file, line, 'escapes-repo', `${label}: ${value}`)
  for (const seg of value.split(/[\\/]/)) if (BUILD_SEGMENTS.has(seg)) return flag(file, line, 'into-build-output', `${label}: ${value}`)
}

const stripJsonc = (s) => s.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^[ \t]*\/\/.*$/gm, '')

function checkTsconfig(file, text) {
  let json; try { json = JSON.parse(stripJsonc(text)) } catch { return }
  const co = json.compilerOptions ?? {}
  for (const v of co.rootDirs ?? []) checkPath(file, text, v, 'rootDirs')
  for (const arr of Object.values(co.paths ?? {})) for (const v of arr) checkPath(file, text, v, 'paths')
  for (const r of json.references ?? []) checkPath(file, text, r.path, 'references.path')
  if (json.extends) checkPath(file, text, json.extends, 'extends')
  if (IS_SUI && co.jsx) flag(file, lineOf(text, '"jsx"'), 'foreign-framework', `jsx: ${co.jsx} (SUI isn't JSX)`)
}

function checkPackageJson(file, text) {
  let json; try { json = JSON.parse(text) } catch { return }
  for (const k of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
    for (const [name, spec] of Object.entries(json[k] ?? {})) {
      const m = /^(file|link):(.+)$/.exec(String(spec))
      if (m) checkPath(file, text, m[2], `${k}.${name}`)
    }
  }
}

function checkCargoToml(file, text) {
  for (const m of text.matchAll(/(^|[\s{,])path\s*=\s*"([^"]+)"/gm)) checkPath(file, text, m[2], 'path dep')
}

for (const file of walk(REPO)) {
  const base = file.split('/').pop()
  const text = (() => { try { return readFileSync(file, 'utf8') } catch { return null } })()
  if (text == null) continue
  if (/^(tsconfig|jsconfig).*\.json$/.test(base)) checkTsconfig(file, text)
  else if (base === 'package.json') checkPackageJson(file, text)
  else if (base === 'Cargo.toml') checkCargoToml(file, text)
}

if (findings.length === 0) {
  console.log(`✓ lint-config-paths: clean (${relative(process.cwd(), REPO) || '.'})`)
  process.exit(0)
}
console.error(`✗ lint-config-paths: ${findings.length} finding(s) — committed config must not reach outside the repo or into build output:`)
for (const f of findings) console.error(`  ${f.file}:${f.line}  [${f.rule}]  ${f.detail}`)
console.error('\nFix at the source (emit the artifact where the consumer resolves it natively); do not add a path segment.')
process.exit(1)
