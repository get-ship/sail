// CSSOM transport gate: `.style` member access is confined to the sanctioned
// transport + framework-owned seams. Every allowlist entry carries its
// rationale in-file. A new hit here means author code reached around
// bind()/generated writers to paint directly — route it through
// `writeCssVar` or a controller signal instead.
import { describe, expect, test } from 'bun:test'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const ROOT = 'packages/sail/runtime/src'

// File → rationale (each file's header carries the same doctrine).
const ALLOWED = [
  'bind.ts', // writeCssVar — THE transport; generated code imports it.
  'theme.ts', // theme preview broadcast (framework-owned, lint-exempt).
  'scroll-lock.ts', // body scroll lock seam (documented in-file).
  'server-dom.ts', // SSR string serialization — attribute text, not CSSOM.
  'motion.ts', // enter/exit initial-state stamping for play(…) configs.
]

function walk(dir: string): string[] {
  const out: string[] = []
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) out.push(...walk(p))
    else if (p.endsWith('.ts')) out.push(p)
  }
  return out
}

describe('CSSOM transport gate', () => {
  test('`.style` access lives only in the sanctioned transport files', () => {
    const offenders: string[] = []
    for (const file of walk(ROOT)) {
      const base = file.split('/').pop()!
      if (ALLOWED.includes(base)) continue
      const src = readFileSync(file, 'utf8')
      for (const [i, line] of src.split('\n').entries()) {
        if (/\.style\b/.test(line) && !line.trim().startsWith('//')) {
          offenders.push(`${file}:${i + 1}: ${line.trim()}`)
        }
      }
    }
    expect(offenders).toEqual([])
  })
})
