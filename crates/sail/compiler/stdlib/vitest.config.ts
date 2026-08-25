import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '@ship/test-setup/vitest'

// Per-component behavior tests colocated with the L1 stdlib sources
// (`<tag>/<tag>.test.ts`). Same compile + mount harness the runtime/dom
// suites use: the native sail-compiler addon compiles each `.sail`, and the
// controller mounts into jsdom — no browser, no docs site, no matrix.
export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@ship/sail-runtime/template-hooks',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/template-hooks/index.ts'),
      },
      {
        find: '@ship/sail-runtime/actions',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/actions/index.ts'),
      },
      { find: '@ship/sail-dom/test-utils', replacement: resolve(__dirname, '../../../../packages/sail/dom/test/compile-sail.ts') },
      {
        find: '@ship/sail-runtime/motion',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/motion-bundle.ts'),
      },
      {
        find: '@ship/sail-runtime/route',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/route/index.ts'),
      },
      {
        find: '@ship/sail-runtime/query',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/query.ts'),
      },
      {
        find: '@ship/sail-runtime/virtual',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/virtual.ts'),
      },
      {
        find: '@ship/sail-runtime/internal',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/internal.ts'),
      },
      {
        find: '@ship/sail-runtime/query-mock',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/query-mock.ts'),
      },
      {
        find: '@ship/sail-runtime',
        replacement: resolve(__dirname, '../../../../packages/sail/runtime/src/index.ts'),
      },
      { find: '@ship/sail-dom', replacement: resolve(__dirname, '../../../../packages/sail/dom/src/index.ts') },
      { find: '@ship/tokens/token-registry.json', replacement: resolve(__dirname, '../../../../packages/sail/tokens/src/token-registry.json') },
      { find: '@ship/tokens/themes', replacement: resolve(__dirname, '../../../../packages/sail/tokens/src/themes.ts') },
      { find: '@ship/tokens', replacement: resolve(__dirname, '../../../../packages/sail/tokens/src/index.ts') },
    ],
    extensions: ['.mts', '.ts', '.tsx', '.mjs', '.js', '.jsx', '.json'],
  },
  test: {
    include: ['**/*.test.ts'],
    environment: 'jsdom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
