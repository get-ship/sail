import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '@ship/test-setup/vitest'

export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@ship/sail-runtime/template-hooks',
        replacement: resolve(__dirname, '../runtime/src/template-hooks/index.ts'),
      },
      {
        find: '@ship/deck-router',
        replacement: resolve(__dirname, '../deck/router/src/index.ts'),
      },
      {
        find: '@ship/sail-runtime/route',
        replacement: resolve(__dirname, '../runtime/src/route/index.ts'),
      },
      {
        find: '@ship/sail-runtime',
        replacement: resolve(__dirname, '../runtime/src/index.ts'),
      },
      { find: '@ship/sail-dom', replacement: resolve(__dirname, 'src/index.ts') },
      // No `@ship/sail-compiler-wasm` alias: the test helper
      // (src/test-utils/compile-sui.ts) now loads the native sail-compiler as a
      // Node-API `.node` addon (built by `cargo build -p sail-compiler-napi` /
      // `just sail-napi`) via process.dlopen — one cargo-current build feeds
      // both the daemon and this harness, so there is no wasm pkg to alias or
      // keep fresh (#255). The wasm surface stays browser-only.
    ],
    // sail-compiler emits `view.js` next to `view.sail` and the paired `view.ts`
    // (see scripts/build-bundles.mjs::compileSui). Tests target the controller
    // source — prefer `.ts` over the generated `.js` shadow.
    extensions: ['.mts', '.ts', '.tsx', '.mjs', '.js', '.jsx', '.json'],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
