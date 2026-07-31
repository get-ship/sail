// L1 conformance runner. CI gate + dev loop, one entry.
//   node run.mjs            → render every spec, assert, write tracked shots (CI)
//   node run.mjs chip       → just <chip> (dev loop while iterating one element)
// Exit 1 if any spec's probe reports issues.

import { readdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { renderSpec, behaveSpec, closeBrowser, THEMES } from './harness.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = resolve(HERE, '__screenshots__')
const only = process.argv[2]
const THEME_N = THEMES.length

const specDir = resolve(HERE, 'specs')
const files = readdirSync(specDir)
  .filter((f) => f.endsWith('.mjs'))
  .filter((f) => !only || f === `${only}.mjs`)

if (!files.length) {
  console.error(only ? `no spec: specs/${only}.mjs` : 'no specs found')
  process.exit(2)
}

let failed = 0
for (const f of files) {
  const spec = (await import(resolve(specDir, f))).default
  const { count, bad } = await renderSpec(spec, OUT)
  // BEHAVIOR: drive the real `use:` action with click/keyboard (when declared).
  const beh = await behaveSpec(spec)
  const all = [...bad, ...beh.map((b) => ({ behavior: true, ...b }))]
  const ok = all.length === 0
  const behNote = spec.behavior ? ` · behavior ${beh.length ? `${beh.length} issue(s)` : 'ok'}` : ''
  console.log(
    `${ok ? '✓' : '✗'} ${spec.tag}  ${count} instances · ${bad.length} issue(s) · ${THEME_N} themes${behNote} → __screenshots__/${spec.tag}.png`,
  )
  if (!ok) { failed++; for (const b of all) console.log('    ', JSON.stringify(b)) }
}
await closeBrowser()
process.exit(failed ? 1 : 0)
