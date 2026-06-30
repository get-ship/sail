// Build-time generator for the /playground storybook: walks every
// @ship/sui-primitives source + its CEM into a single manifest the playground
// imports. Each entry = { tag, title, source, cem }. The viewer compiles
// `source` in-browser (WASM) and drives its knobs off `cem`.
//
//   node scripts/gen-manifest.mjs
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const primSrc = resolve(here, '../../../packages/components/sui/primitives/src')
const primDist = resolve(here, '../../../packages/components/sui/primitives/dist')

const titleize = (tag) =>
  tag.replace(/^sui-/, '').replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())

const entries = []
for (const dir of readdirSync(primSrc, { withFileTypes: true })) {
  if (!dir.isDirectory() || !dir.name.startsWith('sui-')) continue
  const tag = dir.name
  const suiFile = resolve(primSrc, tag, `${tag}.sui`)
  const cemFile = resolve(primDist, `${tag}.custom-elements.json`)
  if (!existsSync(suiFile)) continue
  const source = readFileSync(suiFile, 'utf8')
  let cem = null
  if (existsSync(cemFile)) {
    try {
      cem = JSON.parse(readFileSync(cemFile, 'utf8'))
    } catch {
      cem = null
    }
  }
  entries.push({ tag, title: titleize(tag), source, cem })
}

entries.sort((a, b) => a.tag.localeCompare(b.tag))
const out = resolve(here, '../src/playground-manifest.json')
writeFileSync(out, JSON.stringify(entries, null, 0))
console.log(`playground-manifest: ${entries.length} primitives (${entries.filter((e) => e.cem).length} with CEM) → ${out}`)
