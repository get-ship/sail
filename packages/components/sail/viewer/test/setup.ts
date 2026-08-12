// Vitest setup for `@ship/sail-components/viewers`. The viewing components compose
// `@ship/sail-components/primitives` (toggle-group, tabs, switch, dialog-backed surfaces,
// pointer-driven controls), so the shared Web-API shims from `@ship/test-setup`
// (ResizeObserver, matchMedia, Popover, Dialog, ToggleEvent, PointerEvent) must
// be installed before any specimen mounts. The viewer pack carries no icon
// registry of its own — `sail-icon`, when composed, primes its cache lazily — so
// only the polyfills are wired here (the terminal/chat-elements precedent).

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()

// happy-dom (this pack's environment) does not implement `attachInternals`,
// which every form-associated custom element calls in connectedCallback
// (sail-switch / sail-input / sail-toggle the knob panel composes).
// Stub it so those specimens connect; tests assert DOM/ARIA/event flow, not
// form-internals behaviour, so a no-op ElementInternals is sufficient.
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
