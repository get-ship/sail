// Bakeoff — ReScript controller driving a compiled .sail render.
//
// Wires the real pinned-head compiler emit (`render/counter.js`, copied
// verbatim from /tmp/bakeoff-emit/counter) to the ReScript-compiled
// controller (`lib/es6/src/Setup.js`). The emit's virtual controller import
// is aliased to the bsc output; its runtime vocabulary maps to this repo's
// TS sources — the SAME compiled render + signal graph every other track
// uses, proving the controller contract is language-neutral.
//
// Run: cd tests/bakeoff/rescript && bunx vitest run --reporter=basic

import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'

const HERE = dirname(fileURLToPath(import.meta.url))
const RUNTIME_SRC = resolve(HERE, '../../../packages/sail/runtime/src')
const DOM_SRC = resolve(HERE, '../../../packages/sail/dom/src')
const TOKENS_SRC = resolve(HERE, '../../../packages/sail/tokens/src')

// Mirrors SHIP_VITEST_DEFAULTS (packages/sail/test-setup/src/vitest.ts)
// inlined: that module imports @ship/sail-dom/test-utils, which cannot be
// resolved while the config itself is loading in this standalone package.

export default defineConfig({
  resolve: {
    alias: {
      // The compiled render imports its controller by the compiler's
      // convention; serve our ReScript-compiled setup as that file.
      './counter.sail.controller.ts': resolve(HERE, 'lib/es6/src/Setup.js'),
      // The emit's runtime vocabulary → this repo's TS sources (the same
      // signal graph SailSignal.res binds).
      '@ship/sail-runtime/internal': resolve(RUNTIME_SRC, 'internal.ts'),
      '@ship/sail-runtime': resolve(RUNTIME_SRC, 'index.ts'),
      // Hydration helpers the emit's `hydrate()` references live in sail-dom.
      // runtime/index.ts re-exports theme surface from @ship/tokens.
      '@ship/tokens/themes': resolve(TOKENS_SRC, 'themes.ts'),
      '@ship/tokens/token-registry.json': resolve(TOKENS_SRC, 'token-registry.json'),
      '@ship/sail-dom': resolve(DOM_SRC, 'index.ts'),
      // SHIP_VITEST_DEFAULTS (below) imports this; resolve to TS source.
      '@ship/sail-dom/test-utils': resolve(DOM_SRC, 'test-utils.ts'),
    },
  },
  test: {
    include: [resolve(HERE, '**/*.test.ts')],
    environment: 'jsdom',
    // = SHIP_VITEST_DEFAULTS
    pool: 'forks',
    testTimeout: 10_000,
    hookTimeout: 10_000,
    passWithNoTests: false,
  },
})
