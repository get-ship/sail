import type { Action } from '../action'

/**
 * `use:liveBox={cssDecls}` — freeform specimen geometry (shape facet).
 * Param is semicolon-separated CSS declarations, e.g.
 *   `border-radius:12px;box-shadow:…;border-width:1px;background:oklch(…)`
 * Token-bound `style.radius(bind(…))` only accepts design tokens, so continuous
 * master drags cannot paint live without this path.
 */
export const liveBox: Action<string | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  let applied: string[] = []
  const clear = () => {
    for (const p of applied) el.style.removeProperty(p)
    applied = []
  }
  const apply = (css: string | null | undefined): void => {
    clear()
    if (css == null || css === '') return
    for (const part of String(css).split(';')) {
      const i = part.indexOf(':')
      if (i < 0) continue
      const prop = part.slice(0, i).trim()
      const val = part.slice(i + 1).trim()
      if (!prop || !val) continue
      el.style.setProperty(prop, val)
      applied.push(prop)
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
    destroy: () => clear(),
  }
}
