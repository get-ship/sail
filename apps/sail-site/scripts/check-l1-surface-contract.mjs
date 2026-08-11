#!/usr/bin/env bun
/**
 * L1 surface contract checker (SPEC-L1-SURFACE-SCAFFOLD §2.3).
 *
 * Fail closed when:
 * - Registry axis missing from docs knobs (first-class packages)
 * - Docs knob not in prop-registry / contract (hosts) — reverse fantasy
 * - Docs select option values drift from contract.values
 * - Face path not in token registry
 * - Visual missing real default/rest row or visual.cross enum/bool
 * - First-class slug in DOC_CE_SLUGS without <doc-slug> mount in shell
 * - Hub tile without meta; legend hub/DETAILS ban
 *
 * Usage: bun apps/sail-site/scripts/check-l1-surface-contract.mjs
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..')
const SITE = path.join(ROOT, 'apps/sail-site')
const DOCS = path.join(SITE, 'src/component-docs')
const DETAILS = path.join(SITE, 'src/compositions/component-details.ts')
const TOKEN_REG = path.join(ROOT, 'packages/sail/tokens/src/token-registry.json')
const SPECS = path.join(ROOT, 'packages/sail/l1-conformance/specs')
const SHELL = path.join(SITE, 'routes/components/[tag]/page.sail')

const errors = []
const warn = (m) => console.warn(`warn: ${m}`)
const fail = (m) => {
  errors.push(m)
  console.error(`FAIL: ${m}`)
}

const { CONTRACTS } = await import(pathToFileURL(path.join(DOCS, 'contracts.ts')).href)
const { PROP_REGISTRY } = await import(
  pathToFileURL(path.join(SITE, 'src/prop-registry.gen.ts')).href
)

const tokenDoc = JSON.parse(fs.readFileSync(TOKEN_REG, 'utf8'))
const tokenPaths = new Set(Object.keys(tokenDoc.paths || {}))

const detailsSrc = fs.readFileSync(DETAILS, 'utf8')
const detailsKeys = new Set(
  [...detailsSrc.matchAll(/^\s{2}(?:'([^']+)'|([A-Za-z0-9_-]+)):\s*lang\(/gm)].map(
    (m) => m[1] || m[2],
  ),
)

const hubSlugs = fs
  .readdirSync(DOCS, { withFileTypes: true })
  .filter((d) => d.isDirectory() && !d.name.startsWith('_'))
  .map((d) => d.name)

if (hubSlugs.includes('legend')) fail('legend must not be a hub package (component-docs/legend/)')
if (detailsKeys.has('legend')) fail('legend must not be a top-level DETAILS key')

for (const slug of hubSlugs) {
  if (!fs.existsSync(path.join(DOCS, slug, 'meta.ts'))) fail(`hub package ${slug} missing meta.ts`)
}

// ── Shell DOC_CE_SLUGS ↔ <doc-*> mount pairing ──────────────────────────
const shellSrc = fs.existsSync(SHELL) ? fs.readFileSync(SHELL, 'utf8') : ''
const docCeSlugs = new Set()
const ceMatch = shellSrc.match(/DOC_CE_SLUGS\s*=\s*new Set\(\[([\s\S]*?)\]\)/)
if (ceMatch) {
  for (const m of ceMatch[1].matchAll(/['"]([a-z0-9-]+)['"]/g)) docCeSlugs.add(m[1])
}
for (const slug of docCeSlugs) {
  // Require a mount: <doc-{slug} or use{Slug} that eventually mounts it.
  // Prefer literal CE tag in template.
  const tagRe = new RegExp(`<doc-${slug}[\\s>]`)
  if (!tagRe.test(shellSrc)) {
    fail(`shell: DOC_CE_SLUGS has "${slug}" but no <doc-${slug}> mount in page.sail`)
  }
}
// First-class packages should be in DOC_CE_SLUGS once contracted
for (const c of CONTRACTS) {
  const metaPath = path.join(DOCS, c.slug, 'meta.ts')
  if (!fs.existsSync(metaPath)) continue
  const metaSrc = fs.readFileSync(metaPath, 'utf8')
  if (/host:\s*['"]doc-/.test(metaSrc) && !docCeSlugs.has(c.slug)) {
    fail(`shell: first-class "${c.slug}" missing from DOC_CE_SLUGS`)
  }
}

/** Known knob labels → axis names (docs configurator chrome only). */
const KNOB_LABELS = {
  variant: 'variant',
  tone: 'tone',
  size: 'size',
  disabled: 'disabled',
  pill: 'pill',
  selected: 'selected',
  removable: 'removable',
  dismissible: 'dismissible',
  orientation: 'orientation',
  shape: 'shape',
  centered: 'centered',
  spin: 'spin',
  name: 'name',
  label: 'label',
  value: 'value',
  indeterminate: 'indeterminate',
  checked: 'checked',
  invalid: 'invalid',
  required: 'required',
  min: 'min',
  max: 'max',
  step: 'step',
  type: 'type',
  placeholder: 'placeholder',
  rows: 'rows',
  multiple: 'multiple',
  lang: 'lang',
  code: 'code',
  message: 'message',
  connecting: 'connecting',
}

function extractDocAxes(docSrc) {
  const axes = new Set()
  for (const m of docSrc.matchAll(
    /<label[^>]*style\(knobLabel\)[^>]*>\s*([A-Za-z][A-Za-z0-9\s-]*)\s*<\/label>/g,
  )) {
    const key = m[1].trim().toLowerCase()
    if (KNOB_LABELS[key]) axes.add(KNOB_LABELS[key])
  }
  for (const m of docSrc.matchAll(
    /<(?:select|slider|switch|input)[^>]*\blabel=["']([A-Za-z][A-Za-z0-9\s-]*)["']/g,
  )) {
    const key = m[1].trim().toLowerCase()
    if (KNOB_LABELS[key]) axes.add(KNOB_LABELS[key])
  }
  return axes
}

/** Extract option value= sets under selects that look like axis knobs. */
function extractSelectOptionValues(docSrc) {
  const byAxis = {}
  // Split on select tags with label= matching a known axis
  const re =
    /<select([^>]*)>([\s\S]*?)<\/select>/gi
  let m
  while ((m = re.exec(docSrc))) {
    const attrs = m[1]
    const body = m[2]
    const labelM = attrs.match(/\blabel=["']([^"']+)["']/)
    const axis = labelM ? KNOB_LABELS[labelM[1].trim().toLowerCase()] : null
    if (!axis) continue
    const vals = [...body.matchAll(/<option[^>]*\bvalue=["']([^"']+)["']/g)].map((x) => x[1])
    if (vals.length) byAxis[axis] = vals
  }
  return byAxis
}

async function loadVisualRowsText(tag) {
  const specPath = path.join(SPECS, `${tag}.mjs`)
  if (!fs.existsSync(specPath)) return null
  try {
    const mod = await import(pathToFileURL(specPath).href + `?t=${Date.now()}`)
    const spec = mod.default
    if (!spec?.rows) return fs.readFileSync(specPath, 'utf8')
    const rows = spec.rows()
    return Array.isArray(rows) ? rows.join('\n') : String(rows)
  } catch {
    return fs.readFileSync(specPath, 'utf8')
  }
}

/** Only contract.default counts as the PE default (no soft/solid hardcodes). */
function isContractDefault(val, def) {
  if (def?.default === undefined || def.default === null) return false
  return String(def.default) === String(val)
}

function enumCovered(axis, val, text, specSrc) {
  const hay = text + '\n' + specSrc
  const patterns = [
    new RegExp(`${axis}=["']${val}["']`, 'i'),
    new RegExp(`(?:size|variant|tone|shape|orientation)=["']${val}["']`, 'i'),
  ]
  if (patterns.some((re) => re.test(hay))) return true
  if (axis === 'size' && new RegExp(`size=['"]${val}['"]`, 'i').test(hay)) return true
  return false
}

function familyPresent(axis, text, specSrc) {
  const hay = text + '\n' + specSrc
  if (axis === 'size') return /size=/.test(hay)
  if (axis === 'variant') return /variant=/.test(hay)
  if (axis === 'shape') return /shape=/.test(hay)
  if (axis === 'orientation') return /orientation=/.test(hay)
  if (axis === 'tone') return /tone=/.test(hay)
  return new RegExp(`\\b${axis}=`).test(hay)
}

function boolCovered(axis, text, specSrc) {
  return new RegExp(`\\b${axis}\\b`, 'i').test(text + '\n' + specSrc)
}

/**
 * Real default/rest row — no vacuous `<` match.
 * Requires explicit rest/default markers or untoned/zero-config host cell.
 */
function hasDefaultRestRow(text, specSrc, tag, c) {
  const hay = text + '\n' + specSrc
  // Explicit markers used by harnesses
  if (
    /soft-default|outline-default|ghost-default|soft-rest|rest-default|zero-config|no tone|omit value|indeterminate|l1-mark/i.test(
      hay,
    )
  ) {
    return true
  }
  // tone="default" rest path (badge matrix)
  if (/tone=["']default["']/.test(text)) return true
  // Optional tone: require at least one host cell without tone=
  if (c.axes?.tone?.optional) {
    // class="sail-{tag}..." without tone= on same open tag
    const openTags = text.match(/<[^>]+>/g) || []
    const untoned = openTags.some(
      (t) =>
        (t.includes(`sail-${tag}`) || new RegExp(`^<${tag}[\\s>]`).test(t)) &&
        !/\btone=/.test(t),
    )
    if (untoned) return true
    // chip()/helper forms with no tone in attrs string
    if (/variant=["'][^"']+["'](?![^"'<>]*tone=)/.test(text)) return true
  }
  // No tone axis: bare host is zero-config
  if (!c.axes?.tone) {
    if (new RegExp(`<(${tag}|[a-z-]+\\s[^>]*sail-${tag})`, 'i').test(text)) return true
  }
  // button default variant
  if (tag === 'button' && /variant=["']default["']/.test(text)) return true
  return false
}

/** Optional tone: rest path AND ≥1 toned cell required once. */
function optionalToneCoverageOk(text, specSrc, def) {
  const hay = text + '\n' + specSrc
  const hasRest =
    /soft-default|outline-default|ghost-default|:not\(\[tone\]\)|no tone|tone=["']default["']/i.test(
      hay,
    ) ||
    (text.match(/<[^>]+>/g) || []).some(
      (t) => (t.includes('sail-') || t.startsWith('<')) && !/\btone=/.test(t) && /class=|variant=/.test(t),
    )
  const tonedValues = (def.values || []).filter((v) => v && v !== 'default')
  const hasToned = tonedValues.some((v) => enumCovered('tone', v, text, specSrc))
  return hasRest && hasToned
}

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

  for (const f of c.faces || []) {
    if (!tokenPaths.has(f.path)) fail(`${slug}: face path not in token registry: ${f.path}`)
  }

  if (c.kind === 'host' && tag) {
    const reg = PROP_REGISTRY[tag]
    if (!reg) {
      // Zero-axis hosts with stdlib props:{} are omitted from prop-registry gen (kbd, prose).
      if (Object.keys(c.axes).length > 0) {
        fail(`${slug}: tag "${tag}" missing from prop-registry`)
      }
    } else {
      const regNames = reg.map((p) => p.name)
      for (const axis of Object.keys(c.axes)) {
        if (!regNames.includes(axis)) {
          fail(`${slug}: contract axis "${axis}" not in prop-registry for <${tag}>`)
        }
      }
    }
  }

  const demoOnly = new Set(c.demoOnly || [])
  const contractAxes = new Set(Object.keys(c.axes))
  const requiredDocAxes = Object.entries(c.axes)
    .filter(([, def]) => !def.event && !def.docsSkip && def.control !== 'none')
    .map(([name]) => name)

  if (firstClass) {
    if (!fs.existsSync(docPath)) {
      fail(`${slug}: first-class host but missing doc-${slug}.sail`)
      continue
    }
    const docSrc = fs.readFileSync(docPath, 'utf8')
    const docAxes = extractDocAxes(docSrc)

    for (const axis of requiredDocAxes) {
      const has =
        docAxes.has(axis) ||
        new RegExp(`style\\(knobLabel\\)[^>]*>\\s*${axis}\\s*<`, 'i').test(docSrc) ||
        new RegExp(
          `style\\(knobLabel\\)[^>]*>\\s*${axis[0].toUpperCase()}${axis.slice(1)}\\s*<`,
        ).test(docSrc) ||
        new RegExp(`\\b${axis}\\b`).test(docSrc)
      if (!has) fail(`${slug}: registry/contract axis "${axis}" missing from docs knobs`)
    }

    for (const axis of docAxes) {
      if (demoOnly.has(axis)) continue
      if (contractAxes.has(axis)) continue
      fail(
        `${slug}: docs knob "${axis}" not in prop-registry/contract (fantasy or unlisted demoOnly)`,
      )
    }

    // Option value sets vs contract.values (select knobs only)
    const optionSets = extractSelectOptionValues(docSrc)
    for (const [axis, opts] of Object.entries(optionSets)) {
      const def = c.axes[axis]
      if (!def?.values?.length) continue
      const expected = new Set(def.values.map(String))
      const got = new Set(opts.map(String))
      for (const v of expected) {
        if (!got.has(v)) fail(`${slug}: docs options for ${axis} missing contract value "${v}"`)
      }
      for (const v of got) {
        if (!expected.has(v)) fail(`${slug}: docs options for ${axis} has non-contract value "${v}"`)
      }
    }
  } else if (!hostIsLegacy && c.kind === 'host') {
    warn(`${slug}: contracted but meta.host is not first-class or legacy`)
  }

  if (tag) {
    const specPath = path.join(SPECS, `${tag}.mjs`)
    if (!fs.existsSync(specPath)) {
      fail(`${slug}: missing visual spec packages/sail/l1-conformance/specs/${tag}.mjs`)
    } else {
      const specSrc = fs.readFileSync(specPath, 'utf8')
      if (!/rows\s*\(/.test(specSrc)) fail(`${slug}: visual spec has no rows()`)

      const rowsText = await loadVisualRowsText(tag)
      if (!rowsText || !String(rowsText).trim()) {
        fail(`${slug}: visual rows() empty`)
      } else {
        const text = String(rowsText)

        if (c.visual?.defaultRow) {
          if (!hasDefaultRestRow(text, specSrc, tag, c)) {
            fail(`${slug}: visual missing default/rest row (visual.defaultRow)`)
          }
        }

        // Optional tone: one-time rest + ≥1 toned assert
        if (c.axes.tone?.optional && (c.visual?.cross || []).includes('tone')) {
          if (!optionalToneCoverageOk(text, specSrc, c.axes.tone)) {
            fail(`${slug}: visual optional tone needs rest path AND ≥1 toned value`)
          }
        }

        for (const axis of c.visual?.cross || []) {
          const def = c.axes[axis]
          if (!def || def.event) continue

          if (def.values && def.values.length) {
            for (const val of def.values) {
              if (val === '' || val == null) continue

              // Optional tone values are covered by optionalToneCoverageOk (not per-value default soft-pass)
              if (axis === 'tone' && def.optional) {
                // Still require each listed non-empty tone value to appear
                if (val === 'default') continue
                if (!enumCovered(axis, val, text, specSrc)) {
                  fail(`${slug}: visual missing enum value ${axis}=${val}`)
                }
                continue
              }

              if (isContractDefault(val, def)) {
                // Default may be omitted in markup if family present or explicitly covered
                if (enumCovered(axis, val, text, specSrc)) continue
                if (familyPresent(axis, text, specSrc)) continue
                // No family and no explicit — fail
                fail(`${slug}: visual missing default enum ${axis}=${val} (not omitted with family)`)
                continue
              }

              if (!enumCovered(axis, val, text, specSrc)) {
                fail(`${slug}: visual missing enum value ${axis}=${val}`)
              }
            }
          } else if (
            def.control === 'switch' ||
            def.default === false ||
            def.default === true ||
            def.docsControl === 'switch'
          ) {
            if (!boolCovered(axis, text, specSrc)) {
              fail(`${slug}: visual missing bool axis coverage for ${axis}`)
            }
          } else if (axis === 'value') {
            if (!/value=/.test(text) && !/value=/.test(specSrc)) {
              fail(`${slug}: visual missing value= coverage`)
            }
          }
        }
      }
    }
  }
}

if (errors.length) {
  console.error(`\ncheck-l1-surface-contract: ${errors.length} failure(s)`)
  process.exit(1)
}
console.log(
  `check-l1-surface-contract: ok (${CONTRACTS.length} contracts, ${hubSlugs.length} hub packages)`,
)
process.exit(0)
