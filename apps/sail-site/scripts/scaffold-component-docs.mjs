#!/usr/bin/env bun
import fs from 'node:fs'
import path from 'node:path'

const root = 'apps/sail-site/src/component-docs'
fs.mkdirSync(path.join(root, '_chrome'), { recursive: true })

const cat = fs.readFileSync('apps/sail-site/src/compositions/component-catalog.ts', 'utf8')
const det = fs.readFileSync('apps/sail-site/src/compositions/component-details.ts', 'utf8')
const rows = [...cat.matchAll(/L\('([^']+)',\s*'([^']+)',\s*'([^']+)'\)/g)].map((m) => ({
  slug: m[1],
  tag: m[1],
  key: m[2],
  category: m[3],
}))

function blurbFor(slug) {
  const re = new RegExp(
    `(?:'${slug}'|${slug}):\\s*lang\\('[^']+',\\s*'[^']+',\\s*\\{[\\s\\S]*?blurb:\\s*\\n?\\s*'((?:\\\\'|[^'])*)'`,
  )
  const m = det.match(re)
  if (m) return m[1].replace(/\\n/g, ' ').replace(/\\'/g, "'")
  return `${slug} docs package`
}

function headingFor(slug) {
  const re = new RegExp(
    `(?:'${slug}'|${slug}):\\s*lang\\('[^']+',\\s*'[^']+',\\s*\\{[\\s\\S]*?heading:\\s*'((?:\\\\'|[^'])*)'`,
  )
  const m = det.match(re)
  return m ? m[1] : undefined
}

function kindFor(slug) {
  if (slug === 'checkbox') return 'alias'
  if (slug === 'dropzone') return 'action'
  if (slug === 'layout') return 'keywords'
  return 'host'
}

const HOSTS = new Set(['button', 'badge', 'fieldset'])
const owns = new Set([
  'button',
  'a',
  'badge',
  'separator',
  'alert',
  'chip',
  'spinner',
  'skeleton',
  'meter',
  'progress',
  'icon',
  'empty-state',
  'error-state',
  'fieldset',
  'label',
  'code-block',
  'switch',
  'radio',
  'ticker',
  'scroll-area',
  'slider',
  'textarea',
  'select',
  'input',
])

const metas = []
for (const r of rows) {
  if (r.slug === 'legend') continue
  const dir = path.join(root, r.slug)
  fs.mkdirSync(dir, { recursive: true })
  const kind = kindFor(r.slug)
  const blurb = blurbFor(r.slug)
  const heading = headingFor(r.slug)
  const host = `doc-${r.slug}`
  const stageOwnsKnobs = owns.has(r.slug)

  const ts = `import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: '${r.slug}',
  tag: '${r.tag}',
  key: '${r.key}',
  category: '${r.category}',
  kind: '${kind}',
  blurb: ${JSON.stringify(blurb)},
${heading ? `  heading: ${JSON.stringify(heading)},\n` : ''}  host: '${host}',
  stageOwnsKnobs: ${stageOwnsKnobs},
} as const satisfies DocMeta
`
  fs.writeFileSync(path.join(dir, 'meta.ts'), ts)
  metas.push({ ...r, kind, blurb, heading, host, stageOwnsKnobs })
}

fs.writeFileSync(
  path.join(root, 'types.ts'),
  `/** Docs package meta — SSOT for /components hub tiles. */
export type DocKind = 'host' | 'alias' | 'action' | 'keywords'

export type DocCategory =
  | 'inputs'
  | 'display'
  | 'feedback'
  | 'overlay'
  | 'navigation'
  | 'data'
  | 'layout'
  | 'effects'

export interface DocMeta {
  /** Route slug + hub id: /components/<slug> */
  slug: string
  /** Element tag when kind=host; may match slug. */
  tag?: string
  /** Scope key used by legacy stages / show.<key>(). */
  key: string
  category: DocCategory
  kind: DocKind
  blurb: string
  /** Override h1 (e.g. layout keywords). */
  heading?: string
  /**
   * Hyphenated CE tag for the docs host (`doc-<slug>`).
   * A host without a matching CE mount is the missing-host empty state.
   */
  host: string
  /** Stage owns configurator knobs (hide viewer variant strip). */
  stageOwnsKnobs?: boolean
}
`,
)

const imports = metas.map((m) => `import { meta as ${m.key}Meta } from './${m.slug}/meta'`).join('\n')
const list = metas.map((m) => `  ${m.key}Meta,`).join('\n')
fs.writeFileSync(
  path.join(root, 'registry.ts'),
  `// Static import map of component-docs/*/meta.ts.
// Hub tile exists iff a package meta is listed here (and the meta.ts file exists).
${imports}
import type { DocMeta } from './types'

export const DOC_PACKAGES: readonly DocMeta[] = [
${list}
]

export const DOC_BY_SLUG: Record<string, DocMeta> = Object.fromEntries(
  DOC_PACKAGES.map((m) => [m.slug, m]),
)

export function docMeta(slug: string): DocMeta | null {
  return DOC_BY_SLUG[slug] ?? null
}

/** First-class doc hosts (every package; missing CE → empty state). */
export const FIRST_CLASS_HOSTS = new Set(
  DOC_PACKAGES.map((m) => m.slug),
)
`,
)

// _chrome README as comment-only sail helper notes in ts
fs.writeFileSync(
  path.join(root, '_chrome/stage-styles.ts'),
  `/**
 * Shared configurator chrome for docs packages.
 * Layout SSOT lives in <sail-component-viewer> (specimen | knobs).
 * Geometry atoms for hosts still inlining the row. Width is space scale
 * (w(64)), never panel-max-w-*.
 */
export const STAGE_SPLIT = '{ box: w(full).minW(0).minH(64) }'
export const STAGE_CANVAS = '{ box: grow(1).basis(0).minW(0).minH(64).pad(8) }'
export const KNOB_COLUMN =
  '{ box: w(64).shrink(0).pad(5).minH(64), paint: borderLWidth(1).borderL(border-subtle) }'
export const KNOB_LABEL =
  '{ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase) }'
`,
)

console.log('packages', metas.length)
console.log(
  'first-class',
  metas.map((m) => m.slug),
)
