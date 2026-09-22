import { defineConfig } from 'vitest/config'
import { SHIP_VITEST_DEFAULTS } from '@ship/test-setup/vitest'

// Pure JS + CSS parsing — no DOM, no compiler, no jsdom. happy-dom
// keeps `document` available if any token helper reads it, but the
// assertion surface is just the file contents + a few parse calls.
export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'happy-dom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
