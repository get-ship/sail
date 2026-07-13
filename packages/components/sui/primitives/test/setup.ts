// Vitest setup for `@ship/sui-primitives` — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`; only the package-local
// glue stays here (the registry's `primeIconCache` to push into). The lucide
// JSON path is resolved cwd-independently by `@ship/test-setup`.

import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../../kit/src/sui-icon/registry'

primeLucideIcons(primeIconCache)
initJsdomPolyfills()
