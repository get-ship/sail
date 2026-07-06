// Drift lock: the materialized `packages/sui/tokens/src/contracts.css`
// (the asset build-time consumers read — the deck bundler's sui.css,
// non-JS hosts) must byte-match the runtime's LIGHT_CONTRACTS_CSS after
// its generated banner. Regenerate on mismatch:
//   bun packages/sui/runtime/scripts/emit-contracts-css.ts

import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { LIGHT_CONTRACTS_CSS } from '../src/styles/index'

describe('contracts.css asset', () => {
  it('matches the runtime constants (regenerate via emit-contracts-css.ts)', () => {
    const here = dirname(fileURLToPath(import.meta.url))
    const asset = readFileSync(
      resolve(here, '../../tokens/src/contracts.css'),
      'utf8',
    )
    const body = asset.slice(asset.indexOf('*/') + 3) // strip generated banner
    expect(body).toBe(LIGHT_CONTRACTS_CSS)
  })
})
