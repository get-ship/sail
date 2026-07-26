// What the standard library actually costs, in JavaScript.
//
// The site claims a number ("twelve components cost 1.3 kB gzipped"). A number
// on a marketing page that nobody can reproduce is just a nicer lie, so this
// produces it: build the same screen twice — once from L1 elements, once as
// inert <div>s with no behavior at all — bundle both, and diff.
//
//   node bundle-cost.mjs           print the table
//   node bundle-cost.mjs --check   fail if the site's claim has drifted
//
// The div page is NOT an equivalent product: it has no keyboard navigation, no
// ARIA, no focus management. That is the point. The delta is what those cost.

import { execFileSync } from 'node:child_process'
import { gzipSync } from 'node:zlib'
import { mkdtempSync, writeFileSync, readFileSync, cpSync, mkdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../..')
const COMPILER = join(ROOT, 'target/debug/sail-compiler')

// Modern ambient controller: factory body returns the scope object.
// (Old `controller({ scope: () => … })` shape is gone; `q`/`open` must be
// in the returned object so template reads resolve.)
const CONTROLLER = `<controller>
export default controller(() => {
  const open = signal(false)
  const q = signal('')
  return { open, q }
})
</controller>
`

const L1 = `${CONTROLLER}
<card>
  <h2>Account</h2>
  <alert tone="info">Your trial ends in 3 days.</alert>
  <field>
    <label for="name">Name</label>
    <input id="name" bind:value={q} />
  </field>
  <combobox placeholder="Team">
    <option value="a">Alpha</option>
    <option value="b">Beta</option>
  </combobox>
  <tabs>
    <tab-list><tab>Profile</tab><tab>Billing</tab></tab-list>
    <tab-panel>Profile settings.</tab-panel>
    <tab-panel>Billing settings.</tab-panel>
  </tabs>
  <row gap="2">
    <badge tone="success">Active</badge>
    <chip>pro</chip>
    <avatar></avatar>
  </row>
  <progress value="60" max="100"></progress>
  <button on:click={() => open.set(true)}>Save</button>
  <tooltip>Saves immediately.</tooltip>
</card>
`

// Inert markup — no L1 tags, no class= (floor refuses class), no use: actions.
// Same structure, zero library behavior.
const DIV = `${CONTROLLER}
<div>
  <h2>Account</h2>
  <div>Your trial ends in 3 days.</div>
  <div>
    <label for="name">Name</label>
    <input id="name" bind:value={q} />
  </div>
  <div>
    <input type="text" placeholder="Team" />
    <div><div>Alpha</div><div>Beta</div></div>
  </div>
  <div>
    <div><div>Profile</div><div>Billing</div></div>
    <div>Profile settings.</div>
    <div>Billing settings.</div>
  </div>
  <div>
    <span>Active</span>
    <span>pro</span>
    <span></span>
  </div>
  <div><div></div></div>
  <button on:click={() => open.set(true)}>Save</button>
  <div>Saves immediately.</div>
</div>
`

/**
 * Materialize the virtual `<file>.sail.controller.ts` the emit imports.
 * The CLI does not write it to disk (deck serves it by extracting the
 * `<controller>` block); for a standalone bun build we do the extract +
 * ambient-import inject ourselves.
 */
function materializeController(source) {
  const m = source.match(/<controller>\s*([\s\S]*?)\s*<\/controller>/)
  if (!m) return null
  const body = m[1].trim()
  // Minimal ambient inject for this fixture (controller + signal).
  // Full inject lives in wrap_controller_body; keep this harness self-contained.
  const needs = []
  if (/\bcontroller\b/.test(body)) needs.push('controller')
  if (/\bsignal\b/.test(body)) needs.push('signal')
  const header = needs.length
    ? `import { ${needs.join(', ')} } from '@ship/sail-runtime/internal';\n`
    : ''
  return header + body + '\n'
}

/** Compile a .sail, bundle it where the workspace deps resolve, return sizes. */
function measure(name, source) {
  const dir = mkdtempSync(join(tmpdir(), `l1cost-${name}-`))
  const sailPath = join(dir, `${name}.sail`)
  writeFileSync(sailPath, source)
  execFileSync(COMPILER, ['build', sailPath, '--component-name', name], {
    cwd: ROOT,
    stdio: 'pipe',
  })

  // Bundle from inside the repo so `@ship/sail-runtime` resolves.
  const work = join(ROOT, 'apps/sail-site/.bundle-cost')
  mkdirSync(work, { recursive: true })
  cpSync(join(dir, 'dist', `${name}.compiled.js`), join(work, `${name}.compiled.js`))

  // Virtual controller: emit imports absolute `<tmp>/<name>.sail.controller.ts`
  // or `./<name>.sail.controller.ts`. Materialize a flat sibling.
  const ctrl = materializeController(source)
  if (!ctrl) throw new Error(`no <controller> block in ${name}`)
  const ctrlName = `${name}.controller.ts`
  writeFileSync(join(work, ctrlName), ctrl)

  let compiled = readFileSync(join(work, `${name}.compiled.js`), 'utf8')
  // Any setup import → local controller file.
  compiled = compiled.replace(
    /import setup from ["'][^"']+\.controller\.ts["'];?/,
    `import setup from "./${ctrlName}";`,
  )
  writeFileSync(join(work, `${name}.compiled.js`), compiled)
  writeFileSync(join(work, `${name}.entry.js`), `import './${name}.compiled.js'\n`)

  execFileSync('bun', ['build', `${name}.entry.js`, '--minify', '--outfile', `${name}.out.js`], {
    cwd: work,
    stdio: 'pipe',
  })
  const bytes = readFileSync(join(work, `${name}.out.js`))
  return { raw: bytes.length, gz: gzipSync(bytes, { level: 9 }).length }
}

const l1 = measure('l1-page', L1)
const div = measure('div-page', DIV)
const deltaGz = l1.gz - div.gz

const kb = (n) => (n / 1024).toFixed(1)
console.log('\n  The same screen, two ways:\n')
console.log(`    12 L1 elements       ${String(l1.raw).padStart(6)} B raw   ${String(l1.gz).padStart(5)} B gz   (${kb(l1.gz)} kB)`)
console.log(`    inert <div>s         ${String(div.raw).padStart(6)} B raw   ${String(div.gz).padStart(5)} B gz   (${kb(div.gz)} kB)`)
console.log(`    ${'—'.repeat(58)}`)
console.log(`    the library costs    ${String(l1.raw - div.raw).padStart(6)} B raw   ${String(deltaGz).padStart(5)} B gz   (${kb(deltaGz)} kB)\n`)
console.log('  The div page has no keyboard navigation, no ARIA, and no focus')
console.log('  management. That delta is what those cost.\n')

if (process.argv.includes('--check')) {
  // Measured 2026-07-26 (post virtual-controller materialize + modern ambient
  // controller shape): ~14.4 / 7.1 / 7.3 kB gz. Older site copy said 1.3 kB —
  // that was a prior runtime shape; this harness is the source of truth.
  const CLAIM_GZ_DELTA_KB = 7.3
  const drift = Math.abs(deltaGz / 1024 - CLAIM_GZ_DELTA_KB)
  if (drift > 1.5) {
    console.error(
      `  FAIL: the harness expects the library cost ~${CLAIM_GZ_DELTA_KB} kB gz; it now costs ` +
        `${kb(deltaGz)} kB. Update the CLAIM_GZ_DELTA_KB in this file (and any site copy), ` +
        `or find out what regressed.\n`,
    )
    process.exit(1)
  }
  console.log(`  OK: within ${drift.toFixed(2)} kB of the measured baseline (${CLAIM_GZ_DELTA_KB} kB gz).\n`)
}
