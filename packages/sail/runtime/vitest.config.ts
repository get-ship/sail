import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS, sailSourceLoaderPlugin } from '@ship/test-setup/vitest'

export default defineConfig({
  // The integration tests compile real `.sail` fixtures and import the emitted
  // module, which imports its controller as the virtual `<file>.sail.controller.ts`.
  plugins: [sailSourceLoaderPlugin()],
  resolve: {
    alias: {
      // The integration test compiles real .sail files; the emitted
      // JS imports `@ship/sail-runtime`. Point it at the local source so
      // dynamic-imported compiled modules can find the runtime.
      '@ship/sail-runtime/internal': resolve(__dirname, 'src/internal.ts'),
      '@ship/sail-runtime': resolve(__dirname, 'src/index.ts'),
      // The mount-route-bug repro fixture (test/fixtures/repro-route/page.sail)
      // interpolates text, so its compiled module emits a `hydrate()` that
      // imports `collectTextOpens`/`hydrateText` from `@ship/sail-dom` — the
      // existing integration fixtures don't hit this because none of them
      // interpolate text at the top level. Point it at the local source like
      // the runtime aliases above.
      '@ship/sail-dom': resolve(__dirname, '../dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
