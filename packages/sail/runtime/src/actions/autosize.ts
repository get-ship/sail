// autosize — grow a <textarea> to fit its content. `use:autosize`
//
// The simplest shape of the `use:` API: a pure action (no hook, no state) —
// just element behavior with teardown. This is the whole point of `use:` for
// the author: `<textarea use:autosize>` and it grows; no ref, no effect, no
// resize listener written by hand.

import type { Action } from '../action'

/**
 * Make a `<textarea>` grow to fit its content. Reflows on input and once on
 * mount. `use:autosize` — no parameter.
 */
export const autosize: Action<void> = (node) => {
  const el = node as HTMLTextAreaElement
  const reflow = () => {
    el.style.height = 'auto'
    el.style.height = `${el.scrollHeight}px`
  }
  reflow()
  el.addEventListener('input', reflow)
  return {
    destroy: () => el.removeEventListener('input', reflow),
  }
}
