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

const CONTROLLER = `<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller({
  scope: () => ({ open: signal(false), q: signal('') }),
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

const DIV = `${CONTROLLER}
<div class="card">
  <h2>Account</h2>
  <div class="alert">Your trial ends in 3 days.</div>
  <div class="field">
    <label for="name">Name</label>
    <input id="name" bind:value={q} />
  </div>
  <div class="combobox">
    <input type="text" placeholder="Team" />
    <div class="listbox"><div class="opt">Alpha</div><div class="opt">Beta</div></div>
  </div>
  <div class="tabs">
    <div class="tablist"><div class="tab">Profile</div><div class="tab">Billing</div></div>
    <div class="panel">Profile settings.</div>
    <div class="panel">Billing settings.</div>
  </div>
  <div class="row">
    <span class="badge">Active</span>
    <span class="chip">pro</span>
    <span class="avatar"></span>
  </div>
  <div class="progress"><div class="bar"></div></div>
  <button class="btn" on:click={() => open.set(true)}>Save</button>
  <div class="tooltip">Saves immediately.</div>
</div>
`

/** Compile a .sui, bundle it where the workspace deps resolve, return sizes. */
function measure(name, source) {
  const dir = mkdtempSync(join(tmpdir(), `l1cost-${name}-`))
  writeFileSync(join(dir, `${name}.sui`), source)
  execFileSync(COMPILER, ['build', join(dir, `${name}.sui`), '--component-name', name], {
    cwd: ROOT,
    stdio: 'pipe',
  })

  // Bundle from inside the repo so `@ship/sail-runtime` resolves.
  const work = join(ROOT, 'apps/sail-site/.bundle-cost')
  mkdirSync(work, { recursive: true })
  cpSync(join(dir, 'dist', `${name}.compiled.js`), join(work, `${name}.compiled.js`))
  cpSync(join(dir, `${name}.controller.ts`), join(work, `${name}.controller.ts`))
  // The emit points at `../<name>.controller.ts`; flatten it.
  const compiled = readFileSync(join(work, `${name}.compiled.js`), 'utf8')
    .replace(`../${name}.controller.ts`, `./${name}.controller.ts`)
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
  // The site says 8.7 / 7.4 / 1.3 kB. Allow drift, fail on a different story.
  const CLAIM_GZ_DELTA_KB = 1.3
  const drift = Math.abs(deltaGz / 1024 - CLAIM_GZ_DELTA_KB)
  if (drift > 0.5) {
    console.error(
      `  FAIL: the site claims the library costs ~${CLAIM_GZ_DELTA_KB} kB gz; it now costs ` +
        `${kb(deltaGz)} kB. Update /why and /elements, or find out what regressed.\n`,
    )
    process.exit(1)
  }
  console.log(`  OK: within ${drift.toFixed(2)} kB of the number the site publishes.\n`)
}
