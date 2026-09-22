// <use-click-outside> — fire an action when the user clicks outside
// a target element.
//
// Used by every dropdown / modal / popover that closes on outside
// click. The target is passed via `bind:target={ref}`; the action is
// looked up by name from route scope (typically a controller `on.*`
// handler).
//
// Example:
//   <use-click-outside bind:target={dialogEl} action="close" />
//
// Listener attaches to `document` (capture phase, so handlers nested
// inside the target don't fire incorrectly).

import type { TemplateHook, TemplateHookContext } from '../controller'

interface ClickOutsideAttrs extends Record<string, unknown> {
  /** The element to consider "inside". Anything outside fires action. */
  target: HTMLElement
  /** Resolved action handler. */
  action: (event: MouseEvent) => void
}

export const useClickOutsideHook: TemplateHook<ClickOutsideAttrs, void> = {
  name: 'click-outside',
  attrs: { target: 'binding', action: 'binding' },
  factory: (ctx: TemplateHookContext<ClickOutsideAttrs>) => {
    const { target, action } = ctx.attrs
    if (!target || !action) return

    const handler = (event: MouseEvent) => {
      const t = event.target
      if (t instanceof Node && target.contains(t)) return
      action(event)
    }

    // Capture phase: outside-click should fire before any inside
    // bubble-phase listeners on the target itself.
    document.addEventListener('mousedown', handler, true)
    ctx.cleanup(() => document.removeEventListener('mousedown', handler, true))
  },
}
