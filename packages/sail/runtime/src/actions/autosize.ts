// autosize — grow a <textarea> to fit its content. `use:autosize`
//
// The simplest shape of the `use:` API: a pure action (no hook, no state) —
// just element behavior with teardown. This is the whole point of `use:` for
// the author: `<textarea use:autosize>` and it grows; no ref, no effect, no
// resize listener written by hand.
//
// Height crosses JS via `--autosize-h`. The framework sheet keys on
// `[data-autosize]` and applies `height: var(--autosize-h, auto)` so the
// action can collapse to content (`--autosize-h: auto`) then publish the
// measured `scrollHeight` — never assigning a layout CSS property.

import type { Action } from '../action'

/**
 * Make a `<textarea>` grow to fit its content. Reflows on input and once on
 * mount. `use:autosize` — no parameter.
 */
export const autosize: Action<void> = (node) => {
  const el = node as HTMLTextAreaElement
  el.dataset.autosize = ''
  const reflow = () => {
    // Collapse via the skin's `height: var(--autosize-h, auto)` bridge so
    // scrollHeight reflects the full value, then publish the measurement.
    el.style.setProperty('--autosize-h', 'auto')
    el.style.setProperty('--autosize-h', `${el.scrollHeight}px`)
  }
  reflow()
  el.addEventListener('input', reflow)
  return {
    destroy: () => {
      el.removeEventListener('input', reflow)
      el.style.removeProperty('--autosize-h')
      delete el.dataset.autosize
    },
  }
}
