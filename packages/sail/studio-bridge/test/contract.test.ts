/**
 * Bridge contract: < 6 KB minified, no ops URLs.
 * Prefer baked deck asset when present; also check bun minify.
 */
import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'
import { describe, expect, it } from 'vitest'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const REPO = join(ROOT, '../../..')
const SRC = join(ROOT, 'src')
const OUT = join(ROOT, '.tmp-bridge.min.js')
const BAKED = join(REPO, 'crates/deck/deck-core/assets/sail-studio-bridge.js')

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) walk(p, out)
    else if (p.endsWith('.ts')) out.push(p)
  }
  return out
}

describe('studio-bridge contract', () => {
  const files = walk(SRC)
  const joined = files.map((f) => readFileSync(f, 'utf8')).join('\n')

  it('is under 6 KB (baked asset preferred; else bun minify)', () => {
    if (existsSync(BAKED)) {
      const source = readFileSync(BAKED, 'utf8')
      if (!source.includes('deck bootstrap stub')) {
        const bytes = Buffer.byteLength(source, 'utf8')
        // 6 KB: FAB unmount-while-open + standalone-studio gate are load-bearing.
        expect(bytes, `baked bridge ${bytes} B`).toBeLessThan(6 * 1024)
        return
      }
    }
    execSync(`bun build ${join(SRC, 'index.ts')} --minify --outfile=${OUT}`, {
      cwd: ROOT,
      stdio: 'pipe',
    })
    const bytes = statSync(OUT).size
    expect(bytes, `minified bridge ${bytes} B`).toBeLessThan(6 * 1024)
  })

  it('contains no ops URLs', () => {
    expect(joined).not.toMatch(/\/sail\.ops\.v1\//)
    expect(joined).not.toMatch(/sail\.ops\.v1/)
    expect(joined).not.toMatch(/\/_deck\/dev\//)
    if (existsSync(BAKED)) {
      const baked = readFileSync(BAKED, 'utf8')
      if (!baked.includes('deck bootstrap stub')) {
        expect(baked).not.toMatch(/sail\.ops\.v1/)
      }
    }
  })

  it('talks via postMessage for studio traffic', () => {
    expect(joined).toMatch(/postMessage/)
    expect(joined).toMatch(/sail-studio/)
  })
})
