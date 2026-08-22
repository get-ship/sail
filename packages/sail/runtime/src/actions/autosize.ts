// autosize — grow a control to fit its content. `use:autosize`
//
// The simplest shape of the `use:` API: a pure action (no hook, no state) —
// just element behavior with teardown. This is the whole point of `use:` for
// the author: `<textarea use:autosize>` / `<input use:autosize>` and it grows;
// no ref, no effect, no resize listener written by hand.
//
// Textarea height crosses JS via `--autosize-h`. The framework sheet keys on
// `textarea[data-autosize]` and applies `height: var(--autosize-h, auto)` so
// the action can collapse to content (`--autosize-h: auto`) then publish the
// measured `scrollHeight` — never assigning a layout CSS property.
//
// Single-line text inputs use `field-sizing: content` on `[data-autosize]` in
// sail-base (width auto-fits the value). The action only stamps the data attr.
//
// Mount timing: actions attach while the template subtree is still detached
// (or before the first layout pass). `scrollHeight` is 0 then. Publishing
// `height: 0px` pins the field forever (min-height alone reads as a one-line
// control). Never write a zero measurement — leave `auto` so rows/min-height
// own the box, and re-measure on the next animation frames once connected.

import type { Action } from '../action'
import { writeCssVar } from '../bind'

/**
 * Make a `<textarea>` grow in height, or a single-line `<input>` grow in width,
 * to fit its content. Reflows on input and after mount layout settles.
 * `use:autosize` — no parameter.
 */
export const autosize: Action<void> = (node) => {
  const el = node as HTMLElement
  el.dataset.autosize = ''

  if (!(el instanceof HTMLTextAreaElement)) {
    // input (and other) hosts: width via sail-base `field-sizing: content`.
    return {
      destroy: () => {
        delete el.dataset.autosize
      },
    }
  }

  const reflow = () => {
    // Collapse via the skin's `height: var(--autosize-h, auto)` bridge so
    // scrollHeight reflects the full value, then publish the measurement.
    writeCssVar(el, '--autosize-h', 'auto')
    const sh = el.scrollHeight
    if (sh > 0) {
      writeCssVar(el, '--autosize-h', `${sh}px`)
    } else {
      // Detached / pre-layout: do not pin height:0 — rows + min-height keep
      // a multi-line empty composer readable until a real measure lands.
      writeCssVar(el, '--autosize-h', null)
    }
  }

  reflow()

  // Catch the connected layout pass(es). One rAF is not always enough when
  // the host is `display: contents` and the parent flex/grid sizes later.
  let raf = 0
  let disposed = false
  const schedule = () => {
    if (disposed || typeof requestAnimationFrame !== 'function') return
    raf = requestAnimationFrame(() => {
      reflow()
      raf = requestAnimationFrame(() => {
        reflow()
        raf = 0
      })
    })
  }
  schedule()

  el.addEventListener('input', reflow)
  return {
    destroy: () => {
      disposed = true
      if (raf) cancelAnimationFrame(raf)
      el.removeEventListener('input', reflow)
      writeCssVar(el, '--autosize-h', null)
      delete el.dataset.autosize
    },
  }
}
