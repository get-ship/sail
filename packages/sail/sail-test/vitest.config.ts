import { defineConfig } from 'vitest/config'
import { SHIP_VITEST_DEFAULTS, shipWorkspaceAliases } from '@ship/test-setup/vitest'

// Vitest/happy-dom is an adapter only — the public package has no runner identity.
export default defineConfig({
  resolve: {
    alias: shipWorkspaceAliases(),
    extensions: ['.mts', '.ts', '.tsx', '.mjs', '.js', '.jsx', '.json'],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'happy-dom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
