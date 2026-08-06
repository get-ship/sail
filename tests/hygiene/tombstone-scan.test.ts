/**
 * Living-contract tombstone scan (SPEC-SCRUB-1 WI3).
 *
 * Asserts the product tree does not reintroduce scrubbed dead-surface
 * machinery (named retirement refusals, scrub harnesses, empty-schema
 * fossil fields). This is NOT a tombstone of retired *messages* — it
 * asserts absence of the retired *machinery*.
 *
 * Living temptation refusals (class=/style=/querySelector) are intentionally
 * not listed: they must remain.
 */
import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')

const SKIP_DIRS = new Set([
  'node_modules',
  'target',
  'dist',
  'coverage',
  '.deck',
  '.artifacts',
  '.git',
  '.claude',
  '.grok',
  '.agents',
  '.sail-out',
  'out',
  'runs',
  'eval',
])

/** Dead machinery names scrubbed by SPEC-SCRUB-1 — must stay gone. */
const DEAD_MACHINERY: Array<{ name: string; re: RegExp; allow?: RegExp }> = [
  {
    name: 'state_directive_retired',
    re: /state_directive_retired/,
  },
  {
    name: 'reject_retired_host_selector',
    re: /reject_retired_host_selector/,
  },
  {
    name: 'reject_part_selector',
    re: /reject_part_selector\b/,
  },
  {
    name: 'sail-legacy-layout rule',
    re: /sail-legacy-layout/,
  },
  {
    name: 'legacy_layout module',
    re: /mod legacy_layout|pub mod legacy_layout|rules\/legacy_layout/,
  },
  {
    name: 'scrub harness path',
    re: /tests\/scrub\/retired-forms/,
  },
  {
    name: 'ship.edit-mode localStorage purge',
    re: /ship\.edit-mode\./,
    // annotation-mode keys remain; only the retired edit-mode purge string is banned
    allow: /annotation-mode/,
  },
  {
    name: 'deck.jsonc migration hard-error message',
    re: /deck\.jsonc is no longer supported/,
  },
  {
    name: 'empty schema data fossil field in emitters',
    re: /"data":\s*Value::Array\(Vec::new\(\)\)/,
  },
  {
    name: 'empty meta/mount clients fossil field in emitters',
    re: /clients:\s*\[\]\s*\}\)|"clients":\s*Value::Array\(Vec::new\(\)\)/,
  },
]

function* walkFiles(dir: string): Generator<string> {
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
    if (st.isDirectory()) {
      yield* walkFiles(p)
    } else if (st.isFile()) {
      yield p
    }
  }
}

function readText(path: string): string | null {
  try {
    return readFileSync(path, 'utf8')
  } catch {
    return null
  }
}

describe('tombstone scan: scrubbed dead machinery stays gone', () => {
  test('no reintroduced delete-now machinery in product tree', () => {
    const hits: string[] = []
    for (const file of walkFiles(REPO)) {
      const rel = relative(REPO, file)
      // This file itself names the dead strings.
      if (rel.startsWith('tests/hygiene/tombstone-scan')) continue
      if (rel === 'CHANGELOG.md' || rel === 'AGENTS.md') continue
      if (/\.(png|webp|jpg|jpeg|svg|woff2?|lock|wasm)$/i.test(rel)) continue
      const text = readText(file)
      if (!text) continue
      for (const dead of DEAD_MACHINERY) {
        if (!dead.re.test(text)) continue
        if (dead.allow && dead.allow.test(text) && !dead.re.test(text.replace(dead.allow, ''))) {
          // Only allow-pattern matches — not a hit for the dead name alone.
          continue
        }
        // For ship.edit-mode: require the banned prefix without going through allow.
        if (dead.name.startsWith('ship.edit-mode') && !/ship\.edit-mode\./.test(text)) continue
        hits.push(`${rel}: ${dead.name}`)
      }
    }
    expect(hits).toEqual([])
  })

  test('living temptation refusals still present', () => {
    // These MUST stay (foreign-idiom teaching). Absence would be a regression.
    const floor = readText(join(REPO, 'crates/sail/compiler/src/check/policy.rs')) ?? ''
    const foreign =
      readText(join(REPO, 'crates/sail/compiler/src/directive_manifest.rs')) ?? ''
    const noClass = readText(join(REPO, 'crates/sail/lint/src/rules/no_class.rs')) ?? ''
    const noStyle = readText(join(REPO, 'crates/sail/lint/src/rules/no_inline_style.rs')) ?? ''
    const ambient =
      readText(join(REPO, 'crates/sail/lint/src/rules/controller_no_ambient_authority.rs')) ??
      readText(join(REPO, 'crates/sail/lint/src/rules/no_ambient_dom.rs')) ??
      ''

    expect(noClass.includes('sail-no-class') || floor.includes('sail-no-class')).toBe(true)
    expect(
      noStyle.includes('sail-no-inline-style') || floor.includes('sail-no-inline-style'),
    ).toBe(true)
    expect(foreign.includes('querySelector') || ambient.includes('querySelector')).toBe(true)
  })
})
