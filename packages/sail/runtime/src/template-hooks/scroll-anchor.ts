// <use-scroll-anchor> — scroll the host element into view when the URL
// hash matches the declared `id`.
//
// Declares the host element as a named scroll target. On mount (and on
// every `hashchange` event), checks whether `window.location.hash`
// equals `#<id>`. When it does, calls `el.scrollIntoView()`.
//
// Example:
//   <section id="features">
//     <use-scroll-anchor id="features" />
//     ...
//   </section>
//
// The `id` attribute is a static string that matches the hash fragment
// without the leading `#`. Authors typically give `id` the same value
// as the parent element's `id` attribute.

import type { TemplateHook, TemplateHookContext } from '../controller'

interface ScrollAnchorAttrs extends Record<string, unknown> {
  /** Hash fragment to match (without the leading `#`). */
  id: string
}

export const useScrollAnchorHook: TemplateHook<ScrollAnchorAttrs, void> = {
  name: 'scroll-anchor',
  attrs: { id: 'literal' },
  factory: (ctx: TemplateHookContext<ScrollAnchorAttrs>) => {
    const { id } = ctx.attrs
    if (!id) return

    const { el } = ctx

    const check = () => {
      if (window.location.hash === `#${id}`) {
        el.scrollIntoView({ behavior: 'smooth' })
      }
    }

    window.addEventListener('hashchange', check)
    check()

    ctx.cleanup(() => window.removeEventListener('hashchange', check))
  },
}
