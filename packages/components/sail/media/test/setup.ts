// Vitest setup for `@ship/sail-media`. The renderers compose
// `@ship/sail-primitives` (sail-table, sail-empty-state, sail-pagination,
// sail-icon), so the shared Web-API shims from `@ship/test-setup`
// (ResizeObserver, matchMedia, Popover, Dialog, ToggleEvent, PointerEvent) must
// be installed before any renderer mounts. This pack carries no icon registry of
// its own — `sail-icon`, when composed, primes its cache lazily — so only the
// polyfills are wired here.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()

// happy-dom (this pack's environment) does not implement `attachInternals`,
// which form-associated custom elements call in connectedCallback (sail-input,
// the composed sail-table renders). Stub it so those specimens
// connect; tests assert DOM/ARIA/event flow, not form-internals behaviour, so a
// no-op ElementInternals is sufficient.
const proto = HTMLElement.prototype as HTMLElement & { attachInternals?: () => unknown }
if (typeof proto.attachInternals !== 'function') {
  proto.attachInternals = function attachInternals(): unknown {
    return {
      setFormValue() {},
      setValidity() {},
      checkValidity() {
        return true
      },
      reportValidity() {
        return true
      },
      get form() {
        return null
      },
      get labels() {
        return [] as unknown as NodeList
      },
      get shadowRoot() {
        return (this as unknown as { shadowRoot: ShadowRoot | null }).shadowRoot ?? null
      },
    }
  }
}
