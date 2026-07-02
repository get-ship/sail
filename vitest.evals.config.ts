// vitest.evals.config.ts — SPIKE config for the vitest-evals harness trial.
// Separate from the root vitest.config.ts on purpose: evals are not unit
// tests; they never run under `bun run test`. Run them explicitly:
//
//   bun x vitest run --config vitest.evals.config.ts
//
// The eval reporter renders judge scores in the terminal; the json reporter
// preserves the full normalized HarnessRun + scores in `meta` at
// .artifacts/vitest-evals/results.json (browsable via `bun x vitest-evals
// serve .artifacts/vitest-evals/results.json`).
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['evals/**/*.eval.ts'],
    environment: 'node',
    reporters: ['vitest-evals/reporter', 'json'],
    outputFile: { json: '.artifacts/vitest-evals/results.json' },
    // Recorded-run replay is cheap; live agent dispatch would need a much
    // larger timeout (or to stay outside vitest entirely — see the spike doc).
    testTimeout: 30_000,
  },
})
