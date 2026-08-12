// Vitest setup for @ship/sail-components/mermaid — prime the JSDOM polyfills the
// runtime expects. Mermaid itself does not need a primed icon cache.
import { initJsdomPolyfills } from '@ship/test-setup'
initJsdomPolyfills()
