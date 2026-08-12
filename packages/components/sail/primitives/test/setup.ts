// Vitest setup for `@ship/sail-components/primitives` — JSDOM polyfills + glyph priming
// for the runtime's `use:icon` cache (no network in jsdom). The lucide JSON
// path is resolved cwd-independently by `@ship/test-setup`.

import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '@ship/sail-runtime'

initJsdomPolyfills()
primeLucideIcons((name, svg) => primeIconCache({ [name]: svg }))
