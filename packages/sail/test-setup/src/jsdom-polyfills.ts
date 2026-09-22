// JSDOM-25 polyfills shared by every component-level vitest suite. Vitest's
// jsdom environment is intentionally minimal — these stubs suppress uncaught
// errors without simulating real behavior. Tests only assert DOM structure,
// ARIA, and event flow; layout, paint, and the overlay APIs are out of scope.
//
// Each polyfill is feature-detected so this file is safe to call from any
// runtime: missing APIs get stubbed, present APIs stay unchanged. Calling
// `initJsdomPolyfills` more than once is idempotent.

export interface JsdomPolyfillOpts {
  /** Wrap `requestAnimationFrame` to swallow throws from third-party init —
   *  e.g. xterm.js's WebGL addon throws "WebGL2 not supported null" inside
   *  rAF callbacks under jsdom. Off by default; on only where a consumer
   *  loads such a library. */
  suppressRafErrors?: boolean
}

export function initJsdomPolyfills(opts: JsdomPolyfillOpts = {}): void {
  installResizeObserver()
  installMatchMedia()
  installCanvasGetContext()
  installToggleEvent()
  installPointerEvent()
  installCssEscape()
  installPopoverApi()
  installPopoverDisplay()
  installDialogMethods()
}

function installCssEscape(): void {
  if (typeof CSS !== 'undefined' && typeof CSS.escape === 'function') return
  ;(globalThis as { CSS: { escape?: (s: string) => string } }).CSS ??= {}
  ;(globalThis as { CSS: { escape: (s: string) => string } }).CSS.escape = (s: string) =>
    s.replace(/[^a-zA-Z0-9_\u00A0-\uFFFF-]/g, (c) => `\\${c}`)
}

function installPopoverDisplay(): void {
  // jsdom doesn't apply UA display rules for [popover]. The show/hidePopover
  // polyfills dispatch events but don't change rendering. Set inline styles
  // so tests that check visibility (offsetHeight > 0) work.
  if (typeof HTMLElement === 'undefined') return
  const proto = HTMLElement.prototype
  if ('__sailPopoverDisplay' in proto) return
  Object.defineProperty(proto, '__sailPopoverDisplay', { value: true, writable: true, configurable: true })

  const origShow = proto.showPopover
  Object.defineProperty(proto, 'showPopover', {
    value(this: HTMLElement) {
      origShow?.call(this)
      this.style.display = ''
    },
    writable: true, configurable: true,
  })
  const origHide = proto.hidePopover
  Object.defineProperty(proto, 'hidePopover', {
    value(this: HTMLElement) {
      origHide?.call(this)
      this.style.display = 'none'
    },
    writable: true, configurable: true,
  })
}

function installResizeObserver(): void {
  if (typeof ResizeObserver !== 'undefined') return
  global.ResizeObserver = class ResizeObserver {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
}

function installMatchMedia(): void {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'undefined') return
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: (_query: string) => ({
      matches: false,
      media: _query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  })
}

function installCanvasGetContext(): void {
  if (typeof HTMLCanvasElement === 'undefined') return
  // JSDOM's getContext throws "Not implemented". Override to return null so
  // canvas/WebGL consumers (xterm, charts) skip render cleanly.
  HTMLCanvasElement.prototype.getContext = (() => null) as unknown as typeof HTMLCanvasElement.prototype.getContext
}

function installToggleEvent(): void {
  if (typeof ToggleEvent !== 'undefined') return
  globalThis.ToggleEvent = class ToggleEvent extends Event {
    newState: string
    oldState: string
    constructor(type: string, init: EventInit & { newState?: string; oldState?: string } = {}) {
      super(type, init)
      this.newState = init.newState ?? ''
      this.oldState = init.oldState ?? ''
    }
  }
}

function installPointerEvent(): void {
  if (typeof PointerEvent !== 'undefined') return
  // No MouseEvent means no DOM at all (a `@vitest-environment node` file
  // sharing a jsdom-suite setup) — nothing dispatches pointer events there,
  // and subclassing would throw at setup. Skip, per this file's "safe to
  // call from any runtime" contract.
  if (typeof MouseEvent === 'undefined') return
  // JSDOM 25 ships MouseEvent but not PointerEvent. Anchor-position primitives
  // (sail-hover-card, sail-tooltip) dispatch synthetic pointerenter/leave; the
  // shape (button, clientX, …) inherits from MouseEvent so subclassing works
  // without a real pointer-device emulation.
  ;(globalThis as { PointerEvent: unknown }).PointerEvent = class PointerEvent extends MouseEvent {
    pointerId: number
    pointerType: string
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init)
      this.pointerId = init.pointerId ?? 0
      this.pointerType = init.pointerType ?? 'mouse'
    }
  }
}

function installPopoverApi(): void {
  if (typeof HTMLElement === 'undefined' || 'showPopover' in HTMLElement.prototype) return
  const _openPopovers = new WeakSet<HTMLElement>()

  Object.defineProperty(HTMLElement.prototype, 'showPopover', {
    value(this: HTMLElement) {
      if (_openPopovers.has(this)) return
      _openPopovers.add(this)
      const ev = new (globalThis as unknown as Record<string, typeof ToggleEvent>)['ToggleEvent']!(
        'toggle', { newState: 'open', oldState: 'closed', bubbles: false },
      )
      this.dispatchEvent(ev)
    },
    writable: true, configurable: true,
  })

  Object.defineProperty(HTMLElement.prototype, 'hidePopover', {
    value(this: HTMLElement) {
      if (!_openPopovers.has(this)) return
      _openPopovers.delete(this)
      const ev = new (globalThis as unknown as Record<string, typeof ToggleEvent>)['ToggleEvent']!(
        'toggle', { newState: 'closed', oldState: 'open', bubbles: false },
      )
      this.dispatchEvent(ev)
    },
    writable: true, configurable: true,
  })

  Object.defineProperty(HTMLElement.prototype, 'togglePopover', {
    value(this: HTMLElement) {
      if (_openPopovers.has(this)) (this as HTMLElement & { hidePopover(): void }).hidePopover()
      else (this as HTMLElement & { showPopover(): void }).showPopover()
    },
    writable: true, configurable: true,
  })
}

function installDialogMethods(): void {
  if (typeof HTMLElement === 'undefined' || typeof HTMLDialogElement === 'undefined') return
  const dlgProto = HTMLDialogElement.prototype
  if (typeof dlgProto.showModal === 'function') return

  Object.defineProperty(dlgProto, 'showModal', {
    value(this: HTMLDialogElement) { this.setAttribute('open', '') },
    writable: true, configurable: true,
  })
  Object.defineProperty(dlgProto, 'show', {
    value(this: HTMLDialogElement) { this.setAttribute('open', '') },
    writable: true, configurable: true,
  })
  Object.defineProperty(dlgProto, 'close', {
    value(this: HTMLDialogElement) {
      this.removeAttribute('open')
      this.dispatchEvent(new Event('close', { bubbles: false }))
    },
    writable: true, configurable: true,
  })
}

function installRafErrorSuppression(): void {
  if (typeof requestAnimationFrame === 'undefined') return
  const _origRaf = globalThis.requestAnimationFrame.bind(globalThis)
  globalThis.requestAnimationFrame = (cb: FrameRequestCallback): number =>
    _origRaf((...args: Parameters<FrameRequestCallback>) => {
      try { cb(...args) } catch { /* suppress third-party init errors in JSDOM */ }
    })
}
