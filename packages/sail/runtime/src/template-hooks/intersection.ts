// <use-intersection> — fire a callback when the host element enters the
// viewport.
//
// Attaches an `IntersectionObserver` to `ctx.el`. When the element
// becomes intersecting (i.e. enters the visible viewport), the `on`
// callback is invoked with the `IntersectionObserverEntry`. The
// observer is disconnected on unmount.
//
// Example:
//   <use-intersection bind:on={reveal} />
//
// In environments that do not support `IntersectionObserver` (jsdom,
// older browsers), the hook silently skips setup. Tests should install
// a stub on `globalThis.IntersectionObserver` before exercising the
// hook.

import type { TemplateHook, TemplateHookContext } from '../controller'

interface IntersectionAttrs extends Record<string, unknown> {
  /** Callback fired when the element enters the viewport. */
  on: (entry: IntersectionObserverEntry) => void
}

export const useIntersectionHook: TemplateHook<IntersectionAttrs, void> = {
  name: 'intersection',
  attrs: { on: 'binding' },
  factory: (ctx: TemplateHookContext<IntersectionAttrs>) => {
    const { on } = ctx.attrs
    if (!on || typeof IntersectionObserver === 'undefined') return

    const observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) on(entry)
      }
    })

    observer.observe(ctx.el)
    ctx.cleanup(() => observer.disconnect())
  },
}
