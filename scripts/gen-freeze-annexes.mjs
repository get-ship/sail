#!/usr/bin/env node
/**
 * Generate SAIL syntax freeze annexes A–E from live manifests.
 *
 * Sources (read-only; no cargo/npm required):
 *   A  apps/sail-site/src/language-surface.gen.ts  ELEMENTS.stdlib
 *      (+ stdlib/*.sail tag set for completeness check)
 *   B  packages/sail/tokens/src/token-registry.json
 *   C  language-surface.gen.ts PUBLIC_ACTIONS  (+ action_manifest PUBLIC_ACTION_NAMES)
 *   D  packages/sail/runtime/author-surface.json
 *   E  crates/sail/style/src/channel.rs + conditions.rs + language-surface MOTION
 *
 * Outputs under packages/sail/freeze-annexes/:
 *   a-l1-elements.json | b-tokens.json | c-public-actions.json
 *   d-runtime-exports.json | e-channel-vocabulary.json | index.json
 *   ANNEXES.md (operator-readable tables)
 *
 * Usage:
 *   node scripts/gen-freeze-annexes.mjs
 *   node scripts/gen-freeze-annexes.mjs --check   # drift (exit 1 if dirty)
 */
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs'
import { resolve, dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'
import { execSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO = resolve(__dirname, '..')
const OUT = join(REPO, 'packages/sail/freeze-annexes')
const CHECK = process.argv.includes('--check')

function read(rel) {
  return readFileSync(join(REPO, rel), 'utf8')
}

function readJson(rel) {
  return JSON.parse(read(rel))
}

function sha16(s) {
  return createHash('sha256').update(s).digest('hex').slice(0, 16)
}

function tipSha() {
  try {
    return execSync('git rev-parse HEAD', { cwd: REPO, encoding: 'utf8' }).trim()
  } catch {
    return 'unknown'
  }
}

/** Extract a top-level `export const NAME = …` JSON-ish value from a .gen.ts file. */
function extractExport(src, name) {
  const re = new RegExp(`export const ${name}[^=]*=\\s*`)
  const m = re.exec(src)
  if (!m) throw new Error(`export const ${name} not found`)
  let i = m.index + m[0].length
  // skip whitespace
  while (/\s/.test(src[i])) i++
  const start = i
  const open = src[i]
  if (open !== '{' && open !== '[') {
    // primitive
    const end = src.indexOf('\n', i)
    return JSON.parse(src.slice(i, end).replace(/,$/, '').trim())
  }
  const close = open === '{' ? '}' : ']'
  let depth = 0
  let inStr = false
  let strQ = ''
  let esc = false
  for (; i < src.length; i++) {
    const c = src[i]
    if (inStr) {
      if (esc) {
        esc = false
        continue
      }
      if (c === '\\') {
        esc = true
        continue
      }
      if (c === strQ) inStr = false
      continue
    }
    if (c === '"' || c === "'" || c === '`') {
      inStr = true
      strQ = c
      continue
    }
    if (c === open) depth++
    else if (c === close) {
      depth--
      if (depth === 0) {
        const raw = src.slice(start, i + 1)
        // TS object → JSON: only double-quoted keys already
        return JSON.parse(raw)
      }
    }
  }
  throw new Error(`unterminated export ${name}`)
}

/** Parse channel_for_setter arms from channel.rs → { setter: channel }. */
function parseChannelSetters(rs) {
  const start = rs.indexOf('pub fn channel_for_setter')
  if (start < 0) throw new Error('channel_for_setter not found')
  const body = rs.slice(start, rs.indexOf('_ => return None', start))
  const map = {}
  // Match arms are either:
  //   "a" | "b" => Channel::Box,
  //   "a" | "b" => { Channel::Box }
  // Walk each arm: collect quoted names until Channel::X.
  const armRe =
    /((?:"[A-Za-z][A-Za-z0-9]*"\s*\|\s*)*"[A-Za-z][A-Za-z0-9]*")\s*=>\s*(?:\{\s*)?Channel::(Box|Paint|Text|Animate)/g
  for (const m of body.matchAll(armRe)) {
    const ch = m[2].toLowerCase()
    for (const name of m[1].matchAll(/"([A-Za-z][A-Za-z0-9]*)"/g)) {
      map[name[1]] = ch
    }
  }
  return map
}

function parseStringArrayConst(rs, constName) {
  const re = new RegExp(`pub const ${constName}[^=]*=\\s*&\\[([\\s\\S]*?)\\];`)
  const m = rs.match(re)
  if (!m) return []
  return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1])
}

function parseConditionPrefixes(rs) {
  const m = rs.match(/pub const CONDITION_PREFIXES[\s\S]*?=\s*&\[([\s\S]*?)\];/)
  if (!m) return []
  const out = []
  for (const row of m[1].matchAll(/\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)/g)) {
    out.push({ key: row[1], media: row[2] })
  }
  return out
}

function parsePublicActionNames(rs) {
  const m = rs.match(
    /pub const PUBLIC_ACTION_NAMES[^=]*=\s*&\[([\s\S]*?)\];/,
  )
  if (!m) throw new Error('PUBLIC_ACTION_NAMES not found')
  return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1])
}

function parseSetterTable(rs) {
  // SetterSpec { name: "bg", … scale: Some("color") | None, …}
  const out = []
  // crude block walk
  const blocks = rs.split('SetterSpec {').slice(1)
  for (const b of blocks) {
    const name = b.match(/name:\s*"([^"]+)"/)
    if (!name) continue
    const scaleM = b.match(/scale:\s*Some\("([^"]+)"\)/)
    const scaleNone = /scale:\s*None/.test(b)
    const vocab = b.match(/vocab:\s*"([^"]+)"/)
    out.push({
      name: name[1],
      vocab: vocab ? vocab[1] : name[1],
      scale: scaleM ? scaleM[1] : scaleNone ? null : null,
    })
  }
  return out
}

// ── A. L1 elements ──────────────────────────────────────────────────────────
const langSurface = read('apps/sail-site/src/language-surface.gen.ts')
const elementsDoc = extractExport(langSurface, 'ELEMENTS')
const stdlibTags = readdirSync(join(REPO, 'crates/sail/compiler/stdlib'))
  .filter((f) => f.endsWith('.sail'))
  .map((f) => f.replace(/\.sail$/, ''))
  .sort()

const elementsA = {
  $comment:
    'Annex A — L1 element inventory. Generated from language-surface ELEMENTS.stdlib (prop-registry / l1_manifest) cross-checked against crates/sail/compiler/stdlib/*.sail.',
  source: {
    surface: 'apps/sail-site/src/language-surface.gen.ts#ELEMENTS',
    stdlibDir: 'crates/sail/compiler/stdlib',
  },
  count: elementsDoc.stdlib.length,
  reskinsCount: (elementsDoc.reskins || []).length,
  tags: elementsDoc.stdlib.map((e) => e.tag).sort(),
  elements: elementsDoc.stdlib
    .map((e) => ({
      tag: e.tag,
      description: (e.description || '').replace(/\s+/g, ' ').trim(),
    }))
    .sort((a, b) => a.tag.localeCompare(b.tag)),
  stdlibFileCount: stdlibTags.length,
  // File basenames may differ from runtime tags (e.g. anchor.sail → tag `a`).
  // SSOT for freeze is ELEMENTS.stdlib tags; file count is a completeness guard.
  note: 'stdlib filename may differ from tag (anchor.sail → a); tags list is normative.',
}

// ── B. Tokens ───────────────────────────────────────────────────────────────
const registry = readJson('packages/sail/tokens/src/token-registry.json')
const byType = {}
const byPathRoot = {}
for (const [name, tok] of Object.entries(registry.tokens || {})) {
  const ty = tok.type || 'unknown'
  byType[ty] = (byType[ty] || 0) + 1
  const root = (tok.path || '?').split('.')[0]
  byPathRoot[root] = (byPathRoot[root] || 0) + 1
}
const tokensB = {
  $comment:
    'Annex B — Token inventory. Generated from packages/sail/tokens/src/token-registry.json (DTCG SSOT; generated=true from model).',
  source: 'packages/sail/tokens/src/token-registry.json',
  version: registry.version,
  generated: registry.generated,
  modelSource: registry.source,
  count: registry.names.length,
  byType,
  byPathRoot,
  products: registry.products
    ? {
        channel: registry.products.channel,
        axis: registry.products.axis,
        layer: registry.products.layer,
        entryCount: Array.isArray(registry.products.entries)
          ? registry.products.entries.length
          : Object.keys(registry.products.entries || {}).length,
      }
    : null,
  aliasesCount: Object.keys(registry.aliases || {}).length,
  names: [...registry.names].sort(),
}

// ── C. Public actions ───────────────────────────────────────────────────────
const publicActionsSurface = extractExport(langSurface, 'PUBLIC_ACTIONS')
const actionManifestRs = read('crates/sail/compiler/src/action_manifest.rs')
const publicNamesManifest = parsePublicActionNames(actionManifestRs)
const actionsC = {
  $comment:
    'Annex C — Public action catalog (use:). Generated from language-surface PUBLIC_ACTIONS + action_manifest::PUBLIC_ACTION_NAMES.',
  source: {
    surface: 'apps/sail-site/src/language-surface.gen.ts#PUBLIC_ACTIONS',
    manifest: 'crates/sail/compiler/src/action_manifest.rs#PUBLIC_ACTION_NAMES',
  },
  count: publicActionsSurface.length,
  names: publicActionsSurface.map((a) => a.name).sort(),
  actions: publicActionsSurface
    .map((a) => ({
      name: a.name,
      param: a.param,
      summary: a.summary,
      public: a.public !== false,
    }))
    .sort((a, b) => a.name.localeCompare(b.name)),
  manifestNames: publicNamesManifest,
  manifestDrift: {
    surfaceOnly: publicActionsSurface
      .map((a) => a.name)
      .filter((n) => !publicNamesManifest.includes(n)),
    manifestOnly: publicNamesManifest.filter(
      (n) => !publicActionsSurface.some((a) => a.name === n),
    ),
  },
  sanctionedPreRcAdditions: [],
  note: 'Widget-private actions remain auto-imported for stdlib stamps but are not public / taught. focus + inView landed as public (FREEZE-HONESTY-1).',
}

// ── D. Runtime exports ──────────────────────────────────────────────────────
const authorSurface = readJson('packages/sail/runtime/author-surface.json')
const exportsByCat = {}
const exportList = []
for (const [name, cat] of Object.entries(authorSurface.exports || {})) {
  exportsByCat[cat] = exportsByCat[cat] || []
  exportsByCat[cat].push(name)
  exportList.push({ name, category: cat })
}
for (const k of Object.keys(exportsByCat)) exportsByCat[k].sort()
exportList.sort((a, b) => a.name.localeCompare(b.name))

const runtimeD = {
  $comment:
    'Annex D — Runtime public exports (tier B author surface). Generated from packages/sail/runtime/author-surface.json.',
  source: 'packages/sail/runtime/author-surface.json',
  specifier: authorSurface.specifier,
  entry: authorSurface.entry,
  count: exportList.length,
  categories: authorSurface.categories,
  countsByCategory: Object.fromEntries(
    Object.entries(exportsByCat).map(([k, v]) => [k, v.length]),
  ),
  exports: exportList,
  exportsByCategory: exportsByCat,
}

// ── E. Channel vocabulary ───────────────────────────────────────────────────
const channelRs = read('crates/sail/style/src/channel.rs')
const conditionsRs = read('crates/sail/style/src/conditions.rs')
const settersRs = read('crates/sail/style/src/setters.rs')
const setterChannel = parseChannelSetters(channelRs)
const am = channelRs.match(
  /fn is_animate_trigger[\s\S]*?matches!\(\s*\n?\s*name,\s*\n?\s*([\s\S]*?)\)\s*\n\s*\}/,
)
const animate = am ? [...am[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : []
if (!animate.length) throw new Error('failed to parse is_animate_trigger')
const textBearing = parseStringArrayConst(channelRs, 'TEXT_BEARING_ELEMENTS')
const conditions = parseConditionPrefixes(conditionsRs)
const setterTable = parseSetterTable(settersRs)
const motion = extractExport(langSurface, 'MOTION')

const byChannel = { box: [], paint: [], text: [], animate: [] }
for (const [name, ch] of Object.entries(setterChannel)) {
  if (!byChannel[ch]) byChannel[ch] = []
  byChannel[ch].push(name)
}
for (const k of Object.keys(byChannel)) byChannel[k].sort()

const channelE = {
  $comment:
    'Annex E — Channel vocabulary. Generated from style IR channel map (channel_for_setter), CONDITION_PREFIXES, animate triggers, MOTION presets.',
  source: {
    channelMap: 'crates/sail/style/src/channel.rs#channel_for_setter',
    setters: 'crates/sail/style/src/setters.rs#setters',
    conditions: 'crates/sail/style/src/conditions.rs#CONDITION_PREFIXES',
    motion: 'apps/sail-site/src/language-surface.gen.ts#MOTION',
  },
  channels: ['box', 'paint', 'text', 'animate'],
  setterCount: Object.keys(setterChannel).length,
  settersByChannel: byChannel,
  setterChannel,
  setterTable: setterTable.map((s) => ({
    name: s.name,
    vocab: s.vocab,
    scale: s.scale,
    channel: setterChannel[s.name] || null,
  })),
  animateTriggers: animate,
  mediaConditions: conditions,
  textBearingElements: textBearing,
  motionPresets: motion.presets.map((p) => ({ name: p.name, kind: p.kind })),
  motionProperties: motion.properties,
  motionTriggersFromSurface: motion.triggers,
}

// ── Index ───────────────────────────────────────────────────────────────────
const generatedAt = new Date().toISOString().slice(0, 10)
const tip = tipSha()
const index = {
  $comment: 'SAIL syntax freeze annex index — regenerate via node scripts/gen-freeze-annexes.mjs',
  generatedAt,
  tipSha: tip,
  annexes: {
    A: { file: 'a-l1-elements.json', title: 'L1 element inventory', count: elementsA.count },
    B: { file: 'b-tokens.json', title: 'Token inventory', count: tokensB.count },
    C: { file: 'c-public-actions.json', title: 'Public action catalog', count: actionsC.count },
    D: { file: 'd-runtime-exports.json', title: 'Runtime public exports', count: runtimeD.count },
    E: {
      file: 'e-channel-vocabulary.json',
      title: 'Channel vocabulary',
      count: channelE.setterCount,
    },
  },
  contentHash: null, // filled after write payload
}

function stableStringify(obj) {
  return JSON.stringify(obj, null, 2) + '\n'
}

const files = {
  'a-l1-elements.json': stableStringify(elementsA),
  'b-tokens.json': stableStringify(tokensB),
  'c-public-actions.json': stableStringify(actionsC),
  'd-runtime-exports.json': stableStringify(runtimeD),
  'e-channel-vocabulary.json': stableStringify(channelE),
}

// Hash of annex bodies only (exclude tipSha/date from drift of content)
index.contentHash = sha16(
  files['a-l1-elements.json'] +
    files['b-tokens.json'] +
    files['c-public-actions.json'] +
    files['d-runtime-exports.json'] +
    files['e-channel-vocabulary.json'],
)
files['index.json'] = stableStringify(index)

// ── Markdown tables ─────────────────────────────────────────────────────────
function mdTable(headers, rows) {
  const head = `| ${headers.join(' | ')} |`
  const sep = `| ${headers.map(() => '---').join(' | ')} |`
  const body = rows.map((r) => `| ${r.join(' | ')} |`).join('\n')
  return `${head}\n${sep}\n${body}\n`
}

let md = `# SAIL Syntax Freeze — Annexes A–E (generated)

> **Do not hand-edit.** Regenerate: \`node scripts/gen-freeze-annexes.mjs\`
> Tip: \`${tip}\` · Generated: ${generatedAt} · Content hash: \`${index.contentHash}\`

Normative freeze prose: mission scratch \`SAIL-SYNTAX-FREEZE.md\`. These files are the
machine-readable inventory the freeze annexes point at.

## Summary

${mdTable(
  ['Annex', 'Title', 'Count', 'File'],
  Object.entries(index.annexes).map(([k, v]) => [k, v.title, String(v.count), v.file]),
)}

## A. L1 element inventory (${elementsA.count})

Source: compiler element manifest / stdlib (\`language-surface.gen.ts\` ELEMENTS).

${mdTable(
  ['Tag', 'Description'],
  elementsA.elements.map((e) => [
    `\`${e.tag}\``,
    e.description.replace(/\|/g, '\\|').slice(0, 120),
  ]),
)}

Reskins: **${elementsA.reskinsCount}** (retired — empty). Stdlib files: ${elementsA.stdlibFileCount}.

## B. Token inventory (${tokensB.count})

Source: \`packages/sail/tokens/src/token-registry.json\`.

${mdTable(
  ['DTCG type', 'Count'],
  Object.entries(tokensB.byType)
    .sort((a, b) => b[1] - a[1])
    .map(([k, v]) => [k, String(v)]),
)}

${mdTable(
  ['Path root', 'Count'],
  Object.entries(tokensB.byPathRoot)
    .sort((a, b) => b[1] - a[1])
    .map(([k, v]) => [k, String(v)]),
)}

Product axis: \`${tokensB.products?.channel ?? 'n/a'}\` (entries: ${tokensB.products?.entryCount ?? 0}).
Aliases: ${tokensB.aliasesCount}.

Full name list: [\`b-tokens.json\`](./b-tokens.json) (\`names\` array).

## C. Public action catalog (${actionsC.count})

Source: \`action_manifest::PUBLIC_ACTION_NAMES\` / \`PUBLIC_ACTIONS\` surface.

${mdTable(
  ['Name', 'Param', 'Summary'],
  actionsC.actions.map((a) => [
    `\`${a.name}\``,
    (a.param || '').replace(/\|/g, '\\|'),
    (a.summary || '').replace(/\|/g, '\\|').slice(0, 100),
  ]),
)}

Sanctioned pre-RC additions (not yet in catalog): ${actionsC.sanctionedPreRcAdditions.map((n) => `\`${n}\``).join(', ')}.

Manifest drift surface↔PUBLIC_ACTION_NAMES: surfaceOnly=${JSON.stringify(actionsC.manifestDrift.surfaceOnly)}, manifestOnly=${JSON.stringify(actionsC.manifestDrift.manifestOnly)}.

## D. Runtime public exports (${runtimeD.count})

Source: \`packages/sail/runtime/author-surface.json\` (tier B).

${mdTable(
  ['Category', 'Count'],
  Object.entries(runtimeD.countsByCategory)
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([k, v]) => [k, String(v)]),
)}

${mdTable(
  ['Export', 'Category'],
  runtimeD.exports.map((e) => ['`' + e.name + '`', e.category]),
)}

## E. Channel vocabulary

Channels: **box · paint · text · animate**. Setters mapped: **${channelE.setterCount}**.

${mdTable(
  ['Channel', 'Setter count', 'Setters'],
  Object.entries(channelE.settersByChannel).map(([ch, names]) => [
    ch,
    String(names.length),
    names.map((n) => `\`${n}\``).join(', '),
  ]),
)}

### Animate triggers

${channelE.animateTriggers.map((t) => `\`${t}\``).join(', ')}

### Motion presets (keyframes catalog surface)

${mdTable(
  ['Preset', 'Kind'],
  channelE.motionPresets.map((p) => ['`' + p.name + '`', p.kind]),
)}

### \`when:\` media conditions

${mdTable(
  ['Key', 'Media'],
  channelE.mediaConditions.map((c) => ['`' + c.key + '`', c.media]),
)}

### Text-bearing hosts (text channel)

${channelE.textBearingElements.map((t) => `\`${t}\``).join(', ')}

---

Regenerate after any prop-registry / token-registry / action_manifest / author-surface / channel map change.
`

files['ANNEXES.md'] = md

// ── Write or check ──────────────────────────────────────────────────────────
mkdirSync(OUT, { recursive: true })

if (CHECK) {
  let dirty = false
  for (const [name, content] of Object.entries(files)) {
    const path = join(OUT, name)
    if (!existsSync(path)) {
      console.error(`missing ${name}`)
      dirty = true
      continue
    }
    const existing = readFileSync(path, 'utf8')
    // For index.json, ignore tipSha/generatedAt when comparing content — compare contentHash + annex meta
    if (name === 'index.json') {
      const a = JSON.parse(existing)
      const b = JSON.parse(content)
      if (a.contentHash !== b.contentHash) {
        console.error(`drift index contentHash: ${a.contentHash} → ${b.contentHash}`)
        dirty = true
      }
      continue
    }
    if (name === 'ANNEXES.md') {
      // compare without tip line volatility: strip tip/date lines
      const norm = (s) =>
        s
          .replace(/Tip: `[^`]+`/g, 'Tip: `<sha>`')
          .replace(/Generated: \d{4}-\d{2}-\d{2}/g, 'Generated: <date>')
      if (norm(existing) !== norm(content)) {
        console.error(`drift ${name}`)
        dirty = true
      }
      continue
    }
    if (existing !== content) {
      console.error(`drift ${name}`)
      dirty = true
    }
  }
  if (dirty) {
    console.error('freeze annexes out of date — run: node scripts/gen-freeze-annexes.mjs')
    process.exit(1)
  }
  console.log(`freeze annexes OK (hash ${index.contentHash})`)
  process.exit(0)
}

for (const [name, content] of Object.entries(files)) {
  writeFileSync(join(OUT, name), content)
  console.log('wrote', join('packages/sail/freeze-annexes', name))
}
console.log(`contentHash=${index.contentHash} tip=${tip}`)
console.log(
  `A=${elementsA.count} B=${tokensB.count} C=${actionsC.count} D=${runtimeD.count} E=${channelE.setterCount}`,
)
