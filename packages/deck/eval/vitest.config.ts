import { defineConfig } from 'vitest/config'
// Relative import (not the `@ship/test-setup/vitest` specifier) because eval does
// not declare test-setup as a dependency — consistent with this config's other
// relative `../../sui/...` paths. The shared module's internal resolution is
// anchored to its own file location, so it works regardless of the import style.
import { shipWorkspaceAliases } from '../../sui/test-setup/src/vitest'

export default defineConfig({
  resolve: {
    // The eval's compile path pulls the runtime (including its SUBPATH exports —
    // `@ship/sail-runtime/route` et al.) through @ship/sail-dom/test-utils. Derive
    // the full @ship/* alias table from every package's `exports` map so those
    // subpaths resolve to source instead of the bare `@ship/sail-runtime` alias.
    alias: shipWorkspaceAliases(),
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    // Tests touch the shared `tasks/` directory tree (read-only) and a few
    // shared module-level constants. Running test files serially within
    // the package eliminates the rare cross-file race we observed under
    // heavy load when multiple packages ran in parallel.
    fileParallelism: false,
  },
})
