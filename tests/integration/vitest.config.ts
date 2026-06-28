// Standalone vitest config for the integration suite.
//
// Aliases sui runtime + dom to the sui repo's own source, and the
// contract SDK to ship-contracts. Relative paths so the config is
// portable across clones — earlier revisions hardcoded
// absolute author-machine paths which only worked on the original
// author's checkout.
//
// SHIP_CONTRACTS_DIR overrides the contract SDK's checkout path so CI
// can pin a different location (e.g. a sibling worktree) without
// touching this file.

// Resolve the sibling ship-contracts checkout relative to a discovered
// Ship umbrella root, not a hardcoded absolute path. Walks up from
// import.meta.url looking for a parent that contains a `ship-contracts`
// subdir (the same idiom helm's `scripts/_path.ts discoverShipRoot` uses).
// `SHIP_CONTRACTS_DIR` overrides for CI / containers / non-umbrella layouts.

import { existsSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { defineConfig } from 'vitest/config'
import { SHIP_VITEST_DEFAULTS } from '../packages/sui/test-setup/src/vitest.ts'

function discoverShipContractsDir(): string {
  let dir = new URL('.', import.meta.url).pathname
  for (let i = 0; i < 8; i++) {
    const parent = resolve(dir, '..')
    if (parent === dir) break
    if (existsSync(join(parent, 'ship-contracts'))) {
      return join(parent, 'ship-contracts')
    }
    dir = parent
  }
  return resolve(import.meta.dirname, '..', '..', 'ship-contracts')
}

const SHIP_CONTRACTS_DIR = process.env.SHIP_CONTRACTS_DIR ?? discoverShipContractsDir()
const SHIP_CONTRACTS_SDK_CLIENT = resolve(
  SHIP_CONTRACTS_DIR,
  'packages/sdk-client/src/index.ts',
)

export default defineConfig({
  resolve: {
    alias: {
      // `@ship/sui-*` resolves to this repo's source (sui owns the framework).
      '@ship/sui-runtime/template-hooks': resolve(
        __dirname,
        '../packages/sui/runtime/src/template-hooks/index.ts',
      ),
      '@ship/sui-runtime/dev': resolve(
        __dirname,
        '../packages/sui/runtime/src/dev.ts',
      ),
      '@ship/sui-runtime': resolve(
        __dirname,
        '../packages/sui/runtime/src/index.ts',
      ),
      '@ship/sui-dom/test-utils': resolve(
        __dirname,
        '../packages/sui/dom/src/test-utils/compile-sui.ts',
      ),
      '@ship/sui-dom': resolve(
        __dirname,
        '../packages/sui/dom/src/index.ts',
      ),
      // Contract SDK lives in ship-contracts.
      '@ship/sdk-client': SHIP_CONTRACTS_SDK_CLIENT,
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    include: [resolve(__dirname, '**/*.test.ts')],
    ...SHIP_VITEST_DEFAULTS,
  },
})
