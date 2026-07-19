import { defineConfig } from 'vitest/config'

// test/setup.ts pre-loads the wasm module synchronously so every test that
// imports the browser LSP surfaces gets an already-initialized compiler.
export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'node',
    setupFiles: ['./test/setup.ts'],
  },
})
