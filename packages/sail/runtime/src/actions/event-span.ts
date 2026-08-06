import type { Action } from '../action'

/**
 * Rect-as-data — a positioned element's top/height/left/width.
 * Concrete runtime geometry from time math (calendar) or coordinates
 * (canvas / annotation / graph). All four are NUMBERS in px (or %); none
 * of them are tokens (there is no token for "3pm to 4pm", no token for
 * "node at (200, 300)"). The action writes each as a custom property; the
 * consuming template reads them through the matching sx atoms:
 *
 *     style="top-(--evt-top) h-(--evt-h) left-(--evt-left) w-(--evt-w)"
 *
 * The atoms do NOT resolve to `var(--space-N)` — the writer is a custom
 * property, the reader is `var(--<custom>)`. That is the ONLY way a
 * runtime-measured value reaches the cascade without going through token
 * space (which would be wrong: a value the user dragged is not a design
 * decision).
 *
 * The unit (px or %) is decided per-site by the action that writes — the
 * calendar lanes use % (lanes share a track), the canvas world uses px.
 */
export interface RectSpan {
  /** Top offset (px or %). null clears the property. */
  top: number | string | null
  /** Height (px or %). null clears the property. */
  height: number | string | null
  /** Left offset (px or %). null clears the property. */
  left: number | string | null
  /** Width (px or %). null clears the property. */
  width: number | string | null
}

const toValue = (v: number | string | null | undefined): string | null => {
  if (v == null) return null
  if (typeof v === 'number') return Number.isFinite(v) ? `${v}px` : null
  // String: caller-decorated — `%`, `px`, `calc(...)`, whatever. Empty
  // string clears (often the resting state when a row windowed out).
  return v === '' ? null : v
}

export const eventSpan: Action<RectSpan | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (rect: RectSpan | null | undefined): void => {
    if (!rect) {
      el.style.removeProperty('--evt-top')
      el.style.removeProperty('--evt-h')
      el.style.removeProperty('--evt-left')
      el.style.removeProperty('--evt-w')
      return
    }
    const set = (name: string, v: number | string | null | undefined): void => {
      const val = toValue(v)
      if (val == null) el.style.removeProperty(name)
      else el.style.setProperty(name, val)
    }
    set('--evt-top', rect.top)
    set('--evt-h', rect.height)
    set('--evt-left', rect.left)
    set('--evt-w', rect.width)
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}
