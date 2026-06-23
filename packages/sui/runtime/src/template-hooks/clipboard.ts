// <use-clipboard> — copy text to clipboard on host-element click.
//
// Writes `text` to the clipboard when the host element receives a click
// event. Fires the optional `onCopy` callback after a successful write.
//
// Example:
//   <use-clipboard bind:text={agentUrl} bind:onCopy={markCopied} />
//
// The `text` binding may be a plain string or a signal accessor. The
// click listener attaches to `ctx.el` — place `<use-clipboard>` inside
// (or bound to) the element that should trigger the copy on click.

import type { TemplateHook, TemplateHookContext } from '../controller'

interface ClipboardAttrs extends Record<string, unknown> {
  /** Text to copy — plain string or a signal accessor `() => string`. */
  text: string | (() => string)
  /** Called after the clipboard write resolves. Optional. */
  onCopy?: () => void
}

export const useClipboardHook: TemplateHook<ClipboardAttrs, void> = {
  name: 'clipboard',
  attrs: { text: 'binding', onCopy: 'binding' },
  factory: (ctx: TemplateHookContext<ClipboardAttrs>) => {
    const { text, onCopy } = ctx.attrs
    if (text === undefined || text === null) return

    const handler = () => {
      const value = typeof text === 'function' ? text() : text
      navigator.clipboard.writeText(value).then(() => {
        onCopy?.()
      })
    }

    ctx.el.addEventListener('click', handler)
    ctx.cleanup(() => ctx.el.removeEventListener('click', handler))
  },
}
