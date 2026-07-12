// Vitest setup for @ship/sui-band — prime the JSDOM polyfills the runtime
// expects (ResizeObserver + matchMedia stubs; the controller treats both as
// optional and falls back to the minimum seamless shape).
import { initJsdomPolyfills } from '@ship/test-setup'
initJsdomPolyfills()
