// Emit the light-DOM framework contracts as a real CSS asset —
// `packages/sui/tokens/src/contracts.css` — so build-time consumers (the
// deck bundler's `sui.css` aggregation, any non-JS host) read the SAME
// text the runtime injects. The runtime TS constants remain the authoring
// source; this script materializes them, and the drift test in
// `test/contracts-css.test.ts` fails the suite whenever the two diverge
// (regenerate by running this script).
//
//   bun packages/sui/runtime/scripts/emit-contracts-css.ts
//
// (The runtime's client-injection path `ensureLightContracts` is itself a
// migration-window mechanism: once every host serves the asset, injection
// detects the served sheet and no-ops; the injection path dies when the
// last JS-only host converts — complete-cutover doctrine.)

import { writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { LIGHT_CONTRACTS_CSS, layered } from '../src/styles/index'

const here = dirname(fileURLToPath(import.meta.url))
const out = resolve(here, '../../tokens/src/contracts.css')
const banner =
  '/* GENERATED from @ship/sui-runtime styles — do not hand-edit.\n' +
  ' * Regenerate: bun packages/sui/runtime/scripts/emit-contracts-css.ts\n' +
  ' * The light-DOM framework contracts: tone/size channels (attribute-\n' +
  ' * keyed), interactive-surface baseline, focus/disabled, L1 element\n' +
  ' * skins ride separately via the stdlib registry. */\n'
writeFileSync(out, banner + layered(LIGHT_CONTRACTS_CSS))
console.log(`wrote ${out} (${LIGHT_CONTRACTS_CSS.length} bytes, @layer sui)`)
