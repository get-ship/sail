// Root vitest config — `bun run test` (or `vitest run --project=...`) runs the
// entire test surface across all per-package vitest configs. Each package's
// own `vitest.config.ts` is the source of truth for its environment, aliases,
// and shadow-DOM compile plugin; this file just glues them together as
// projects. To add a new testable workspace package, add a `projects` entry
// below — keep the path relative to the repo root.
//
// Rust (`cargo test`) and Playwright (`bunx playwright test`) are NOT part of
// `bun run test`; they have their own entry points and toolchains. This
// script covers the bun + vitest surface only.
import { defineConfig } from 'vitest/config'

const PROJECTS = [
  'packages/components/sail/code-editor',
  'packages/components/sail/mermaid',
  'packages/components/sail/chat-elements',
  'packages/components/sail/media',
  'packages/components/sail/primitives',
  'packages/components/sail/viewer',
  'packages/deck/host',
  'packages/deck/hono',
  'packages/deck/router',
  'packages/sail/dom',
  'packages/sail/runtime',
  'packages/sail/tokens',
  'packages/sail/test-setup',
  'packages/sail/sail-test',
  'packages/sail/studio-bridge',
  'packages/sail/studio-shell',
  'apps/studio',
  // `tests/integration` is intentionally omitted — its `vitest.config.ts`
  // has a pre-existing module-resolution issue (rolldown can't resolve the
  // relative TS import of `../packages/sail/test-setup/src/vitest.ts`).
  // Track re-inclusion in a follow-up; until then, run it directly with
  // `cd tests/integration && vitest run`.
]

export default defineConfig({
  test: {
    projects: PROJECTS,
    // Per-project configs own their own environment, aliases, plugins, and
    // include globs — don't override anything here that would mask them.
    passWithNoTests: true,
  },
})
