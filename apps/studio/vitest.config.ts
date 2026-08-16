import { defineConfig } from 'vitest/config'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  resolve: {
    alias: {
      '@ship/sail-studio-shell': join(
        root,
        '../../packages/sail/studio-shell/src/index.ts',
      ),
      '@ship/sail-studio-bridge': join(
        root,
        '../../packages/sail/studio-bridge/src/index.ts',
      ),
      '@ship/sail-studio-client': join(
        root,
        '../../packages/sail/studio-client/src/index.ts',
      ),
      '@ship/studio/edit': join(root, 'src/edit/index.ts'),
      '@ship/studio/checks': join(root, 'src/checks/index.ts'),
      // Deck remaps @ship/deck/router → package @ship/deck-router (src-direct).
      '@ship/deck/router': join(root, '../../packages/deck/router/src/index.ts'),
      '@ship/deck-router': join(root, '../../packages/deck/router/src/index.ts'),
    },
  },
  test: {
    environment: 'happy-dom',
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    // Default stays modest; long cases (batched sail check, openDocument) set per-test timeouts.
    testTimeout: 60_000,
    hookTimeout: 120_000,
  },
})
