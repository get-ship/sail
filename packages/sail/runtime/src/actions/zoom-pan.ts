import type { Action } from '../action'

/**
 * `use:zoomPan={state}` — the live transform of a zoomed+panned image
 * (sail-media-image). `state` carries the cursor-relative translate and
 * scale; the action composes the CSS `transform` string and writes it as
 * a custom property. The consuming element reads it via
 *     style="transform-(--zoom-tx)"
 *
 * The values are user input (wheel zoom, drag pan) or measured (the
 * rect-clamped pan bounds), so they are NOT tokens — there is no
 * `var(--zoom-scale-7)` and there should not be. The seam is exactly
 * the seam described in LANGUAGE-FLOOR.md: the property stays in sx,
 * the number crosses JS.
 */
export interface ZoomPanState {
  x: number
  y: number
  scale: number
}

export const zoomPan: Action<ZoomPanState | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const compose = (s: ZoomPanState): string => {
    const x = Number.isFinite(s.x) ? s.x : 0
    const y = Number.isFinite(s.y) ? s.y : 0
    const sc = Number.isFinite(s.scale) ? s.scale : 1
    return `translate(${x}px, ${y}px) scale(${sc})`
  }
  const apply = (state: ZoomPanState | null | undefined): void => {
    if (!state) {
      el.style.removeProperty('--zoom-tx')
      return
    }
    el.style.setProperty('--zoom-tx', compose(state))
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}

/**
 * `use:maxWidth={px}` — runtime max-width (e.g. a viewer stage sized to
 * the host viewport minus chrome). A value the user resized to, not a
 * design decision. Writes `--max-w`; the consuming template uses
 *     style="max-w-(--max-w)"
 */
export const maxWidth: Action<number | string | null | undefined> = (
  node,
  initial,
) => {
  const el = node as HTMLElement
  const toValue = (v: number | string | null | undefined): string | null => {
    if (v == null) return null
    if (typeof v === 'number') return Number.isFinite(v) ? `${v}px` : null
    return v === '' ? null : v
  }
  const apply = (v: number | string | null | undefined): void => {
    const val = toValue(v)
    if (val == null) el.style.removeProperty('--max-w')
    else el.style.setProperty('--max-w', val)
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}

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
