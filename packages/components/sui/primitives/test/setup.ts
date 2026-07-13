// Vitest setup for `@ship/sui-primitives` — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`; only the package-local
// JSON path is resolved cwd-independently by `@ship/test-setup`.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
