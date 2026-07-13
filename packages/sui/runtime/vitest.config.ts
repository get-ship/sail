import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS, suiSourceLoaderPlugin } from '@ship/test-setup/vitest'

export default defineConfig({
  // The integration tests compile real `.sui` fixtures and import the emitted
  // module, which imports its controller as the virtual `<file>.sui.controller.ts`.
  plugins: [suiSourceLoaderPlugin()],
  resolve: {
    alias: {
      // The integration test compiles real .set files; the emitted
      // JS imports `@ship/sui-runtime`. Point it at the local source so
      // dynamic-imported compiled modules can find the runtime.
      '@ship/sui-runtime/internal': resolve(__dirname, 'src/internal.ts'),
      '@ship/sui-runtime': resolve(__dirname, 'src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
