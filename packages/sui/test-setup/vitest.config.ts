import { defineConfig } from 'vitest/config'
import { SHIP_VITEST_DEFAULTS, shipWorkspaceAliases } from './src/vitest'

// render.ts pulls the wasm-backed compile harness from @ship/sail-dom/test-utils
// plus the runtime — and the runtime's SUBPATH exports (`@ship/sail-runtime/route`
// et al.). `shipWorkspaceAliases()` derives the full @ship/* alias table from
// every package's `exports` map (subpaths sorted before bare names), so those
// subpaths resolve to their real source file instead of falling through to the
// bare `@ship/sail-runtime` alias.
export default defineConfig({
  resolve: {
    alias: shipWorkspaceAliases(),
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
