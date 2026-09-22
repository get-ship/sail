// Vitest setup for `@ship/sail-components/chat` — JSDOM polyfills only.
//
// These are pure framework-tier primitives: no icon registry to prime,
// no SDK client to stub. The shared polyfills (ResizeObserver,
// matchMedia, Popover, Dialog, ToggleEvent, PointerEvent) cover the
// surfaces these components touch.
import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
