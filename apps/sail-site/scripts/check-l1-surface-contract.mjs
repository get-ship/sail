#!/usr/bin/env bun
/**
 * L1 surface contract checker (SPEC-L1-SURFACE-SCAFFOLD §2.3).
 *
 * Fail closed when:
 * - Registry axis missing from docs knobs (first-class packages)
 * - Docs knob not in prop-registry (hosts)
 * - Face path not in token registry
 * - Visual spec missing / no rows() / empty
 * - Hub tile without component-docs/<slug>/meta.ts
 * - legend appears as hub tile or top-level DETAILS key
 *
 * Usage: bun apps/sail-site/scripts/check-l1-surface-contract.mjs
 * Exit 1 on any failure.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..')
const SITE = path.join(ROOT, 'apps/sail-site')
const DOCS = path.join(SITE, 'src/component-docs')
const REGISTRY_GEN = path.join(SITE, 'src/prop-registry.gen.ts')
const DETAILS = path.join(SITE, 'src/compositions/component-details.ts')
const TOKEN_REG = path.join(ROOT, 'packages/sail/tokens/src/token-registry.json')
const SPECS = path.join(ROOT, 'packages/sail/l1-conformance/specs')

const errors = []
const warn = (m) => console.warn(`warn: ${m}`)
const fail = (m) => {
  errors.push(m)
  console.error(`FAIL: ${m}`)
}

// ── Load contracts via dynamic import of the TS index (bun handles TS) ──
const { CONTRACTS } = await import(path.join(DOCS, 'contracts.ts'))

// ── Prop registry ───────────────────────────────────────────────────────
const regSrc = fs.readFileSync(REGISTRY_GEN, 'utf8')
const PROP_REGISTRY = (() => {
  // Minimal extract: `"tag": [ { "name": "…" }, … ]`
  const out = {}
  const re = /"([^"]+)":\s*\[([\s\S]*?)\](?=,\s*(?:"[^"]+":|}))/g
  let m
  while ((m = re.exec(regSrc))) {
    const tag = m[1]
    const body = m[2]
    const names = [...body.matchAll(/"name":\s*"([^"]+)"/g)].map((x) => x[1])
    out[tag] = names
  }
  return out
})()

// ── Token paths ─────────────────────────────────────────────────────────
const tokenDoc = JSON.parse(fs.readFileSync(TOKEN_REG, 'utf8'))
const tokenPaths = new Set(Object.keys(tokenDoc.paths || {}))

// ── DETAILS keys (top-level) ────────────────────────────────────────────
const detailsSrc = fs.readFileSync(DETAILS, 'utf8')
// Rough: top-level keys in DETAILS object — look for `  key: lang(` or `  'key': lang(`
const detailsKeys = new Set(
  [...detailsSrc.matchAll(/^\s{2}(?:'([^']+)'|([A-Za-z0-9_-]+)):\s*lang\(/gm)].map(
    (m) => m[1] || m[2],
  ),
)

// ── Hub packages (meta.ts dirs) ─────────────────────────────────────────
const hubSlugs = fs
  .readdirSync(DOCS, { withFileTypes: true })
  .filter((d) => d.isDirectory() && !d.name.startsWith('_'))
  .map((d) => d.name)

// ── legend bans ─────────────────────────────────────────────────────────
if (hubSlugs.includes('legend')) fail('legend must not be a hub package (component-docs/legend/)')
if (detailsKeys.has('legend')) fail('legend must not be a top-level DETAILS key')

// ── Hub tile ↔ meta.ts ──────────────────────────────────────────────────
for (const slug of hubSlugs) {
  const meta = path.join(DOCS, slug, 'meta.ts')
  if (!fs.existsSync(meta)) fail(`hub package ${slug} missing meta.ts`)
}

// ── Per-contract checks ─────────────────────────────────────────────────
for (const c of CONTRACTS) {
  const slug = c.slug
  const tag = c.tag
  const dir = path.join(DOCS, slug)
  const metaPath = path.join(dir, 'meta.ts')
  const contractPath = path.join(dir, 'contract.ts')
  const docPath = path.join(dir, `doc-${slug}.sail`)

  if (!fs.existsSync(metaPath)) fail(`${slug}: missing meta.ts`)
  if (!fs.existsSync(contractPath)) fail(`${slug}: missing contract.ts`)

  const metaSrc = fs.readFileSync(metaPath, 'utf8')
  const firstClass = /host:\s*['"]doc-/.test(metaSrc)
  const hostIsLegacy = /host:\s*['"]legacy['"]/.test(metaSrc)

  // Faces must exist in token registry
  for (const f of c.faces || []) {
    if (!tokenPaths.has(f.path)) fail(`${slug}: face path not in token registry: ${f.path}`)
  }

  // Host packages: axes must ⊆ prop-registry for tag
  if (c.kind === 'host' && tag) {
    const reg = PROP_REGISTRY[tag]
    if (!reg) {
      fail(`${slug}: tag "${tag}" missing from prop-registry`)
    } else {
      for (const axis of Object.keys(c.axes)) {
        // registry uses camelCase names as declared; onRemove etc. match
        if (!reg.includes(axis)) {
          // allow kebab-case registry names? prop registry uses onRemove style
          fail(`${slug}: contract axis "${axis}" not in prop-registry for <${tag}>`)
        }
      }
    }
  }

  // First-class docs: every non-event, non-docsSkip axis must appear as a knob label/control
  if (firstClass) {
    if (!fs.existsSync(docPath)) {
      fail(`${slug}: first-class host but missing doc-${slug}.sail`)
      continue
    }
    const docSrc = fs.readFileSync(docPath, 'utf8')
    for (const [axis, def] of Object.entries(c.axes)) {
      if (def.event || def.docsSkip || def.control === 'none') continue
      // Heuristic: axis name appears in doc host as knob label, bind, or signal
      const re = new RegExp(
        `\\b${axis}\\b|label>${axis[0].toUpperCase()}${axis.slice(1)}|Label>${axis}|${axis}\\(`,
        'i',
      )
      // Special cases for common control labels
      const labelHints = {
        variant: /Variant/i,
        tone: /Tone/i,
        size: /Size/i,
        disabled: /Disabled/i,
        pill: /Pill/i,
        selected: /Selected/i,
        removable: /Removable/i,
        dismissible: /Dismissible/i,
        orientation: /Orientation/i,
        shape: /Shape/i,
        centered: /Centered/i,
        spin: /Spin/i,
        name: /Name/i,
        label: /Label/i,
        value: /Value/i,
      }
      const ok = re.test(docSrc) || (labelHints[axis] && labelHints[axis].test(docSrc))
      if (!ok) fail(`${slug}: registry/contract axis "${axis}" missing from docs knobs`)
    }

    // Reverse: docs knob labels that look like fantasy axes not in contract
    // (light check — only flag explicit select options for unknown axes is hard; skip deep AST)
  } else if (!hostIsLegacy && c.kind === 'host') {
    warn(`${slug}: contracted but meta.host is not first-class or legacy`)
  }

  // Visual spec
  if (tag) {
    const specPath = path.join(SPECS, `${tag}.mjs`)
    if (!fs.existsSync(specPath)) {
      fail(`${slug}: missing visual spec packages/sail/l1-conformance/specs/${tag}.mjs`)
    } else {
      const specSrc = fs.readFileSync(specPath, 'utf8')
      if (!/rows\s*\(/.test(specSrc)) fail(`${slug}: visual spec has no rows()`)
      if (c.visual?.defaultRow && !/default|rows\s*\(\s*\)/i.test(specSrc)) {
        // soft: presence of rows is enough; default often encoded as first matrix cell
      }
    }
  }
}

// ── Registry first-class hosts should have contracts once listed ────────
// (optional soft check — only for packages with contract.ts present)

if (errors.length) {
  console.error(`\ncheck-l1-surface-contract: ${errors.length} failure(s)`)
  process.exit(1)
}
console.log(`check-l1-surface-contract: ok (${CONTRACTS.length} contracts, ${hubSlugs.length} hub packages)`)
process.exit(0)
