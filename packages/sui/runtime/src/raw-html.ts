// Raw-HTML binding. Replaces the anchor element's innerHTML with the
// expression's value on each dependency change.
//
// **Author is responsible for sanitization.** This binding never
// escapes the value — that's the whole point — and is the only
// supported XSS surface in the runtime.
//
// Today only consumed by hand-written controllers (no compiler
// emission). The `{@html expr}` template syntax referenced in earlier
// drafts is NOT wired through the compiler. If the compiler grows a
// directive that emits this binding, it MUST refuse to compile when
// the source value is anything but a literal HTML string — never an
// interpolation that could carry attacker-controlled data.

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'
import { morph } from './morph'

/**
 * Replace `el.innerHTML` with the value of `get()` on every dependency
 * change. **Author is responsible for sanitization** — this binding
 * never escapes the value and is the only intended XSS surface in the
 * runtime.
 *
 * Pass extra signals via `deps` when the expression depends on values
 * that aren't read inside `get` (e.g. `get` is a pre-rendered string
 * but should still re-run when some toggle flips).
 *
 * If `el` carries the `morph` directive (`<element morph>…</element>`
 * in `.sui`), updates go through `morph()` instead of `innerHTML =`,
 * preserving DOM identity (focus, scroll, selection, custom-element
 * state) across re-renders.
 *
 * **Not** emitted by the compiler today — call this directly from a
 * controller when raw HTML is genuinely required. Future compiler
 * support (if added) must refuse interpolations that mix attacker-
 * controlled values into the rendered HTML.
 *
 * @param el Element whose `innerHTML` is rewritten (or morphed).
 * @param get Reactive read of the HTML string.
 * @param deps Extra signals to subscribe to (default `[]`).
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const html = signal('<em>hi</em>')
 * bindRawHtml(span, () => html())
 * ```
 *
 * @see bindText for safe, escaped interpolation; morph for the in-place patcher.
 */
export function bindRawHtml(
  el: Element,
  get: () => string,
  deps: Signal<unknown>[] = [],
): Disposer {
  return effect(() => {
    // touch deps for the dependency tracker
    for (const d of deps) d()
    const next = String(get() ?? '')
    if (el.hasAttribute('morph')) {
      morph(el, next)
    } else {
      el.innerHTML = next
    }
  })
}
