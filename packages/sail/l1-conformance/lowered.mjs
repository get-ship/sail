// The markup an element ACTUALLY lowers to.
//
// Specs used to hand-write their rows. That is how a stub ships: the spec tests
// the markup its author typed, so an element whose template is wrong — or was
// never built — still passes. The <theme-picker> was seven naked buttons for a
// week and the spec was green the whole time.
//
// This asks the compiler instead. The lowered markup is the `__tpl("…")` string
// in the emitted module, so we compile `<tag …></tag>` inside a host component
// and read it back.

import { execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..')
const COMPILER = join(ROOT, 'target/debug/sail-compiler')

const cache = new Map()

/**
 * Lowered HTML for `<tag attrs>children</tag>`, straight from the compiler.
 * Returns null if the compiler isn't built (so a spec can fall back).
 */
export function lowered(tag, { attrs = '', children = '' } = {}) {
  const key = `${tag}|${attrs}|${children}`
  if (cache.has(key)) return cache.get(key)

  let html = null
  try {
    const dir = mkdtempSync(join(tmpdir(), 'l1-lowered-'))
    const name = 'lw-host'
    writeFileSync(join(dir, `${name}.sui`), `<${tag}${attrs ? ' ' + attrs : ''}>${children}</${tag}>`)
    execFileSync(COMPILER, ['build', join(dir, `${name}.sui`), '--component-name', name], {
      cwd: ROOT,
      stdio: 'pipe',
    })
    const js = readFileSync(join(dir, 'dist', `${name}.compiled.js`), 'utf8')
    const m = js.match(/__tpl\("((?:[^"\\]|\\.)*)"\)/)
    if (m) {
      html = JSON.parse(`"${m[1]}"`)
      // The host wraps it in the Model-A root; hand back just the element.
      html = html.replace(/^<div class="sail-root">/, '').replace(/<\/div>$/, '')
    }
  } catch {
    html = null
  }
  cache.set(key, html)
  return html
}
