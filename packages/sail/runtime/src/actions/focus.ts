// focus — focus the host when mounted / when the param is truthy.
// `use:focus` · `use:focus={shouldFocus()}`
//
// Agents and authors reach for autofocus constantly. Native `autofocus` is
// one-shot and race-prone; this action re-applies when the reactive param
// becomes truthy and no-ops when falsey. Parameter-less `use:focus` focuses
// once on mount.

import type { Action } from '../action'

/**
 * Focus the host element. With a boolean param, focus whenever the param is
 * truthy (including the first mount). Without a param, focus once on mount.
 *
 * `use:focus` · `use:focus={open()}`.
 */
export const focus: Action<boolean | null | undefined | void> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (want: boolean | null | undefined | void): void => {
    // void/undefined (param-less use:focus) → focus once.
    if (want === false || want === null) return
    // Defer so layout/paint settle before moving keyboard focus.
    requestAnimationFrame(() => {
      try {
        el.focus({ preventScroll: true })
      } catch {
        el.focus()
      }
    })
  }
  apply(initial)
  return {
    update: (want) => apply(want),
  }
}
