import { defineConfig } from 'vitest/config'

// The wasm surface is pure (compile/analyze return strings + plain objects),
// so no DOM env is needed — node keeps the smoke test fast. The pretest script
// builds pkg/ if a fresh checkout hasn't run `bun run wasm` yet.
export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'node',
  },
})
