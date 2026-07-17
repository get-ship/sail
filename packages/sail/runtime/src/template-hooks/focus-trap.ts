// <use-focus-trap> — trap keyboard focus within a target element.
//
// Intercepts Tab / Shift+Tab to cycle within the container's focusable
// descendants. Restores the previously focused element on unmount.
//
// Example:
//   <use-focus-trap bind:target={dialogEl} />
//
// The target element (typically a modal or dialog container) is passed
// via `bind:target`. The listener attaches to `document` so it fires
// regardless of which child currently holds focus.

import type { TemplateHook, TemplateHookContext } from '../controller'

const FOCUSABLE =
  'a[href],area[href],input:not([disabled]),select:not([disabled]),' +
  'textarea:not([disabled]),button:not([disabled]),iframe,object,embed,' +
  '[tabindex]:not([tabindex="-1"]),[contenteditable]'

function focusables(el: HTMLElement): HTMLElement[] {
  return Array.from(el.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (e) => !e.closest('[hidden]') && e.getAttribute('aria-hidden') !== 'true',
  )
}

interface FocusTrapAttrs extends Record<string, unknown> {
  /** Container element whose descendants should receive trapped focus. */
  target: HTMLElement
}

export const useFocusTrapHook: TemplateHook<FocusTrapAttrs, void> = {
  name: 'focus-trap',
  attrs: { target: 'binding' },
  factory: (ctx: TemplateHookContext<FocusTrapAttrs>) => {
    const { target } = ctx.attrs
    if (!target) return

    const previously = document.activeElement as HTMLElement | null
    const all = focusables(target)
    if (all.length > 0) all[0]!.focus()

    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return
      const els = focusables(target)
      if (!els.length) {
        e.preventDefault()
        return
      }
      const first = els[0]!
      const last = els[els.length - 1]!
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault()
          last.focus()
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault()
          first.focus()
        }
      }
    }

    document.addEventListener('keydown', handler)
    ctx.cleanup(() => {
      document.removeEventListener('keydown', handler)
      previously?.focus()
    })
  },
}
