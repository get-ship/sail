// JSDOM polyfills for the chart components.
//
// Charts don't use icons, popovers, dialogs, xterm or the canvas paths
// that ui-components' setup compensates for — every chart is pure
// SVG + signals. ResizeObserver is the only Web API the chart bundle
// touches that jsdom doesn't ship, and it shows up only when a chart's
// host element wants to react to layout changes (currently none do via
// RO, but other code paths in @ship/sail-runtime probe for it).

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}
