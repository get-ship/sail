// <use-title> — set document.title at mount, restore on unmount.
//
// `text` is a literal string attribute on the element; the compiler
// emits it verbatim and the hook applies it on mount. The factory
// captures the previous title in cleanup so unmount is non-leaking.
//
// For a reactive title (one that tracks a signal), use `effect` in
// the controller — the hook is for lifecycle-bound side effects,
// not live bindings.
//
// Example:
//   <use-title text="SUI tutorial · step 19" />

import type { TemplateHook, TemplateHookContext } from '../controller'

interface TitleAttrs extends Record<string, unknown> {
  /** Document title applied on mount. */
  text: string
}

export const useTitleHook: TemplateHook<TitleAttrs, void> = {
  name: 'title',
  attrs: { text: 'literal' },
  factory: (ctx: TemplateHookContext<TitleAttrs>) => {
    const { text } = ctx.attrs
    if (text === undefined || text === null) return

    const prev = document.title
    document.title = text

    ctx.cleanup(() => {
      document.title = prev
    })
  },
}
