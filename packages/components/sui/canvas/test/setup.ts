// JSDOM polyfills for <sui-canvas>.
//
// Excalidraw is mocked in the test (see sui-canvas.test.ts), so the real
// React tree never renders here. ResizeObserver is still probed by the
// @ship/sail-runtime mount path, and jsdom doesn't ship it.

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}
