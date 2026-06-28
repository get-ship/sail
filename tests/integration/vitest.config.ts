// Standalone vitest config for the integration suite.
//
// Aliases sui runtime + dom to the sui repo's own source, and the
// contract SDK to ship-contracts. Relative paths so the config is
// portable across clones — the previous absolute
// `/home/madvib/dev/Ship/sui/...` aliases only worked on the original
// author's machine.
//
// SHIP_CONTRACTS_DIR overrides the contract SDK's checkout path so CI
// can pin a different location (e.g. a sibling worktree) without
// touching this file.

import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '../packages/sui/test-setup/src/vitest.ts'

const SHIP_CONTRACTS_SDK_CLIENT = resolve(
  process.env.SHIP_CONTRACTS_DIR ?? '/home/madvib/dev/Ship/ship-contracts',
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
