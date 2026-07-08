// Vitest setup for @ship/sui-code-editor — prime the JSDOM polyfills the
// runtime expects (matchMedia, ResizeObserver, canvas.getContext). No
// lucide/icon cache to prime here (the editor is text-only).
import { initJsdomPolyfills } from '@ship/test-setup'
initJsdomPolyfills()
