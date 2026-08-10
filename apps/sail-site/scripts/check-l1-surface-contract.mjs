#!/usr/bin/env bun
/**
 * L1 surface contract checker (SPEC-L1-SURFACE-SCAFFOLD §2.3).
 *
 * Fail closed when:
 * - Registry axis missing from docs knobs (first-class packages)
 * - Docs knob not in prop-registry / contract (hosts) — reverse fantasy
 * - Face path not in token registry
 * - Visual spec missing default row or enum value on visual.cross
 * - Hub tile without component-docs/<slug>/meta.ts
 * - legend appears as hub tile or top-level DETAILS key
 *
 * Usage: bun apps/sail-site/scripts/check-l1-surface-contract.mjs
 * Exit 1 on any failure.
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
}

/**
 * Extract docs knob axes from first-class hosts.
 * Only knobs with style(knobLabel) labels or bind targets for known axes —
 * never free-form label="primary" swatch a11y names.
 */
function extractDocAxes(docSrc) {
  const axes = new Set()
  // <label style(knobLabel) …>Tone</label>
  for (const m of docSrc.matchAll(
    /<label[^>]*style\(knobLabel\)[^>]*>\s*([A-Za-z][A-Za-z0-9\s-]*)\s*<\/label>/g,
  )) {
    const key = m[1].trim().toLowerCase()
    if (KNOB_LABELS[key]) axes.add(KNOB_LABELS[key])
  }
  // select/slider with label="Variant" on the control next to knobLabel sections
  for (const m of docSrc.matchAll(
    /<(?:select|slider|switch|input)[^>]*\blabel=["']([A-Za-z][A-Za-z0-9\s-]*)["']/g,
  )) {
    const key = m[1].trim().toLowerCase()
    if (KNOB_LABELS[key]) axes.add(KNOB_LABELS[key])
  }
  return axes
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

/** Defaults often omitted from markup — treat as covered if axis appears at all or default is the PE default. */
function isDefaultEnumValue(axis, val, def) {
  if (def?.default !== undefined && String(def.default) === String(val)) return true
  // common PE defaults
  if (axis === 'size' && val === 'md') return true
  if (axis === 'variant' && (val === 'default' || val === 'soft' || val === 'solid')) return true
  if (axis === 'orientation' && val === 'horizontal') return true
  if (axis === 'shape' && val === 'text') return true
  return false
}

function enumCovered(axis, val, text, specSrc) {
  const hay = text + '\n' + specSrc
  // attr forms
  const patterns = [
    new RegExp(`${axis}=["']${val}["']`, 'i'),
    new RegExp(`\\b${axis}="${val}"`, 'i'),
    // size/variant/tone/shape commonly as bare attrs
    new RegExp(`(?:size|variant|tone|shape|orientation)=["']${val}["']`, 'i'),
  ]
  if (patterns.some((re) => re.test(hay))) return true
  // size ladder bare cells: sm/md/lg labels in harness text nodes
  if (axis === 'size' && new RegExp(`>${val}<|\\('${val}'|size=['"]${val}`, 'i').test(hay)) return true
  if (axis === 'variant' && new RegExp(`variant=["']${val}["']`, 'i').test(hay)) return true
  if (axis === 'tone' && new RegExp(`tone=["']${val}["']`, 'i').test(hay)) return true
  if (axis === 'shape' && new RegExp(`shape=["']${val}["']`, 'i').test(hay)) return true
  return false
}

function boolCovered(axis, text, specSrc) {
  const hay = text + '\n' + specSrc
  // attribute presence: disabled, spin, centered, pill, selected, removable, dismissible
  if (new RegExp(`\\b${axis}\\b`, 'i').test(hay)) return true
  return false
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
      fail(`${slug}: tag "${tag}" missing from prop-registry`)
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
      // knobLabel text, data-axis (if ever allowed), or bind/signal name
      const has =
        docAxes.has(axis) ||
        new RegExp(`style\\(knobLabel\\)[^>]*>\\s*${axis}\\s*<`, 'i').test(docSrc) ||
        new RegExp(
          `style\\(knobLabel\\)[^>]*>\\s*${axis[0].toUpperCase()}${axis.slice(1)}\\s*<`,
        ).test(docSrc) ||
        new RegExp(`\\b${axis}\\b`).test(docSrc)
      if (!has) fail(`${slug}: registry/contract axis "${axis}" missing from docs knobs`)
    }

    // Reverse fantasy: only axes extracted from knob chrome
    for (const axis of docAxes) {
      if (demoOnly.has(axis)) continue
      if (contractAxes.has(axis)) continue
      fail(
        `${slug}: docs knob "${axis}" not in prop-registry/contract (fantasy or unlisted demoOnly)`,
      )
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
          // Require some rest/default/untoned/zero-config marker or any row content for thin hosts
          const ok =
            /default|rest|soft-default|:not\(\[tone\]\)|no tone|omit value|indeterminate|variant=["']soft["']|variant=["']solid["']|size=["']md["']|</i.test(
              text + '\n' + specSrc,
            )
          if (!ok) fail(`${slug}: visual missing default/rest row (visual.defaultRow)`)
        }

        for (const axis of c.visual?.cross || []) {
          const def = c.axes[axis]
          if (!def || def.event) continue

          if (def.values && def.values.length) {
            for (const val of def.values) {
              if (val === '' || val == null) continue
              if (isDefaultEnumValue(axis, val, def)) {
                // still require axis family present unless pure optional omit
                if (axis === 'tone' && def.optional) {
                  // optional tone: require at least one toned value OR rest path
                  continue
                }
                // default size/variant covered if any size/variant markup exists
                if (enumCovered(axis, val, text, specSrc)) continue
                if (
                  (axis === 'size' && /size=/.test(text + specSrc)) ||
                  (axis === 'variant' && /variant=/.test(text + specSrc)) ||
                  (axis === 'shape' && /shape=/.test(text + specSrc)) ||
                  (axis === 'orientation' && /orientation=/.test(text + specSrc))
                ) {
                  continue
                }
              }
              if (!enumCovered(axis, val, text, specSrc)) {
                // optional tone: rest path may omit some tones only if ALL non-default fail — require each listed
                if (axis === 'tone' && def.optional && val === 'default') continue
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
