// Vitest setup for `@ship/sail-components/primitives` — JSDOM polyfills.
// No glyph priming: `<icon name=…>` glyphs are compiler pack-inline.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
