/**
 * JOB-STDLIB-1 — CSS lives in .sail / sail-base.css / stdlib/platform/*.css.
 * Rust string literals are not a stylesheet.
 */
import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')
const PLATFORM = join(REPO, 'crates/sail/compiler/stdlib/platform')
const MANIFEST = join(PLATFORM, 'MANIFEST.md')
const CRATES = join(REPO, 'crates')

const SKIP_DIRS = new Set([
  'node_modules',
  'target',
  'dist',
  'coverage',
  '.deck',
  '.artifacts',
  '.git',
  'tests',
])

const REASONS = new Set(['vendor-pseudo', 'native-anatomy', 'composite', 'vocab-gap'])

/**
 * Vendor / data-uri paint in Rust strings — the MAP loophole. Style-lowering
 * (`background:` fragments in setters) and lint teaching prose are not this.
 */
const CSS_IN_RUST = [/url\(\s*["']data:/, /::-webkit-/, /::-moz-/, /::backdrop/]

const TEACH =
  'CSS belongs in .sail, packages/sail/tokens/src/sail-base.css, or crates/sail/compiler/stdlib/platform/<element>.css — not Rust strings (JOB-STDLIB-1).'

const ALLOW_COLORS = new Set(['#888888', 'white', '#000', '#000000'])
const ALLOW_LENGTHS = new Set([
  '0',
  '0px',
  '1px',
  '2px',
  '2.5',
  '3px',
  '6px',
  '16px',
  '0.9em',
  '1rem',
  '16',
  '100vw',
  '100vh',
  '80vh',
])

function* walkRs(dir: string): Generator<string> {
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry)) continue
    const p = join(dir, entry)
    let st
    try {
      st = statSync(p)
    } catch {
      continue
    }
    if (st.isDirectory()) yield* walkRs(p)
    else if (st.isFile() && entry.endsWith('.rs')) yield p
  }
}

/** Extract "..." / r"..." / r#"..."# literals (best-effort, skips comments). */
function rustStringLiterals(src: string): string[] {
  const noBlock = src.replace(/\/\*[\s\S]*?\*\//g, '')
  const noLine = noBlock
    .split('\n')
    .map((l) => l.replace(/\/\/.*$/, ''))
    .join('\n')
  const out: string[] = []
  const rawHash = /r(#*)"([\s\S]*?)"\1/g
  let m: RegExpExecArray | null
  while ((m = rawHash.exec(noLine))) out.push(m[2] ?? '')
  const cooked = /(?<!r#*)"((?:\\.|[^"\\])*)"/g
  while ((m = cooked.exec(noLine))) {
    const start = m.index
    if (noLine.slice(Math.max(0, start - 3), start).includes('r#')) continue
    if (noLine[start - 1] === 'r') continue
    out.push((m[1] ?? '').replace(/\\[ntr"\\]/g, ' '))
  }
  return out
}

function platformCssFiles(): string[] {
  return readdirSync(PLATFORM)
    .filter((f) => f.endsWith('.css'))
    .sort()
}

function parseManifestRows(md: string): Array<{ file: string; reason: string }> {
  const rows: Array<{ file: string; reason: string }> = []
  for (const line of md.split('\n')) {
    if (!line.startsWith('| ') || line.startsWith('| file |') || /^\|[-| ]+$/.test(line)) continue
    const cells = line
      .split('|')
      .map((c) => c.trim())
      .filter(Boolean)
    if (cells.length < 3 || !cells[0].endsWith('.css')) continue
    rows.push({ file: cells[0], reason: cells[2] })
  }
  return rows
}

function declarationValues(css: string): string[] {
  const values: string[] = []
  const re = /[a-zA-Z-][\w-]*\s*:\s*([^;{}]+);/g
  let m: RegExpExecArray | null
  while ((m = re.exec(css))) values.push((m[1] ?? '').trim())
  return values
}

function hardcodedColors(value: string): string[] {
  const hits: string[] = []
  const hex = value.match(/#(?:[0-9a-fA-F]{3,8})/g) ?? []
  const encoded = value.match(/%23([0-9a-fA-F]{3,8})/g) ?? []
  hits.push(...hex)
  hits.push(...encoded.map((e) => `#${e.slice(3)}`))
  const named =
    value.match(
      /\b(?:white|black|red|blue|green|yellow|orange|purple|navy|teal|gray|grey|silver|maroon|olive|lime|aqua|fuchsia)\b/gi,
    ) ?? []
  hits.push(...named.map((n) => n.toLowerCase()))
  return hits.filter((c) => {
    const key = c.startsWith('#') ? (c.toLowerCase() === '#000000' ? '#000000' : c.toLowerCase() === '#000' ? '#000' : c.toLowerCase()) : c
    return !ALLOW_COLORS.has(key) && !ALLOW_COLORS.has(c) && !ALLOW_COLORS.has(c.toLowerCase())
  })
}

function hardcodedLengths(value: string): string[] {
  if (/var\(--/.test(value)) {
    // lengths inside calc/min that already ride tokens are fine
    const bare = value.match(/(?<![-\w])\d+(?:\.\d+)?(?:px|em|rem|vh|vw)\b/g) ?? []
    return bare.filter((l) => !ALLOW_LENGTHS.has(l))
  }
  const lens = value.match(/(?<![-\w])\d+(?:\.\d+)?(?:px|em|rem|vh|vw)\b/g) ?? []
  return lens.filter((l) => !ALLOW_LENGTHS.has(l))
}

describe('JOB-STDLIB-1 platform CSS', () => {
  test('no CSS declaration strings in production Rust', () => {
    const hits: string[] = []
    for (const file of walkRs(CRATES)) {
      const src = readFileSync(file, 'utf8')
      if (/\b(DescRule|DescPaint|CHECKBOX_MAP|append_descendant_skin)\b/.test(src)) {
        hits.push(`${relative(REPO, file)}: leftover DescRule/DescPaint/*_MAP/append_descendant_skin`)
      }
      for (const lit of rustStringLiterals(src)) {
        for (const pat of CSS_IN_RUST) {
          if (pat.test(lit)) {
            hits.push(`${relative(REPO, file)}: ${pat} — ${TEACH}`)
          }
        }
      }
    }
    expect(hits, hits.join('\n')).toEqual([])
  })

  test('every platform/*.css has a manifest row with a reason class', () => {
    const md = readFileSync(MANIFEST, 'utf8')
    const rows = parseManifestRows(md)
    const files = platformCssFiles()
    const rowFiles = rows.map((r) => r.file).sort()
    expect(rowFiles).toEqual(files)
    for (const row of rows) {
      expect(REASONS.has(row.reason), `${row.file} reason "${row.reason}"`).toBe(true)
    }
  })

  test('platform values are var(--*) or allowlisted constants', () => {
    const violations: string[] = []
    for (const file of platformCssFiles()) {
      const css = readFileSync(join(PLATFORM, file), 'utf8')
      for (const value of declarationValues(css)) {
        for (const c of hardcodedColors(value)) {
          violations.push(`${file}: color ${c} in "${value}"`)
        }
        for (const l of hardcodedLengths(value)) {
          violations.push(`${file}: length ${l} in "${value}"`)
        }
      }
    }
    expect(violations, violations.join('\n')).toEqual([])
  })

  test('served skins carry @element and @provenance headers', () => {
    const skins = readFileSync(join(REPO, 'packages/sail/tokens/src/element-skins.css'), 'utf8')
    expect(skins).toContain('/* @element checkbox */')
    expect(skins).toContain('/* @provenance platform/checkbox.css */')
    expect(skins).toContain('/* @provenance platform/select.css */')
  })
})
