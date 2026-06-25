// Setup for @ship/test-setup's own tests + the behavior-grade generated tests.
// This package runs under the happy-dom environment (see vitest.config.ts): its
// CSS parser tolerates the modern CSS components inject as scoped <style>, where
// jsdom 25 threw "Could not parse CSS stylesheet" on mount. happy-dom also ships
// ResizeObserver / matchMedia / PointerEvent / popover / dialog natively, so the
// polyfills below are feature-detected no-ops here — kept only as a backstop in
// case a consumer flips this package back to jsdom.
import { initJsdomPolyfills } from '../src/jsdom-polyfills'

initJsdomPolyfills()
