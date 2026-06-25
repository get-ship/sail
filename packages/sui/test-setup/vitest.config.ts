import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from './src/vitest'

// render.ts pulls the wasm-backed compile harness from @ship/sui-dom/test-utils
// plus the runtime. Mirror sui-dom's aliases so the same source resolves here.
// Order matters — most specific subpath first.
export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@ship/sui-dom/test-utils',
        replacement: resolve(__dirname, '../sui/dom/src/test-utils/compile-sui.ts'),
      },
      { find: '@ship/sui-dom', replacement: resolve(__dirname, '../sui/dom/src/index.ts') },
      {
        find: '@ship/sui-runtime/template-hooks',
        replacement: resolve(__dirname, '../sui/runtime/src/template-hooks/index.ts'),
      },
      {
        find: '@ship/deck-router',
        replacement: resolve(__dirname, '../deck/route/src/index.ts'),
      },
      {
        find: '@ship/sui-runtime/route',
        replacement: resolve(__dirname, '../sui/runtime/src/route/index.ts'),
      },
      { find: '@ship/sui-runtime', replacement: resolve(__dirname, '../sui/runtime/src/index.ts') },
      {
        find: '@ship/sui-compiler-wasm',
        replacement: resolve(__dirname, '../sui/compiler-wasm/pkg/sui_compiler.js'),
      },
    ],
    extensions: ['.mts', '.ts', '.tsx', '.mjs', '.js', '.jsx', '.json'],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'happy-dom',
    setupFiles: ['./test/setup.ts'],
    ...SHIP_VITEST_DEFAULTS,
    // Expose `global.gc` to the worker forks so the leak-budget retention signal
    // (WeakRef/FinalizationRegistry after a forced GC) is CONCLUSIVE here, not
    // inconclusive. Without this flag the component-leak gate still runs but its
    // closure/effect-retention check degrades to a documented no-op. In Vitest 4
    // `poolOptions` was removed — pool args are top-level `execArgv`.
    execArgv: ['--expose-gc'],
  },
})
