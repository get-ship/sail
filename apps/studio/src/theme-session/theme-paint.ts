/**
 * Theme paint helpers — mode tables, display names, location id.
 * Pure functions used by createThemeSession (no store side effects).
 */
import { THEMES } from '@ship/tokens/themes'
import { themeMeta } from '@ship/tokens/themes-meta'
import { appRouteBase } from '../shell/shell-page-vm'

export type ModeInfo = {
  modes: ('light' | 'dark')[]
  defaultMode: 'light' | 'dark'
  dual: boolean
  modesLabel: string
}

export type PaintResolve = {
  paintId: string
  /** User-visible warning when paint fell back to a safe default. */
  warning?: string
}

const SAFE_PAINT_FALLBACK = 'studio'

function titleCase(id: string): string {
  return (
    id
      .split(/[-_]/)
      .filter(Boolean)
      .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
      .join(' ') || id
  )
}

export function modesForPaintId(paintId: string): ModeInfo {
  const meta = themeMeta(paintId)
  const reg = THEMES.find((t) => t.id === paintId)
  const raw = (meta?.modes?.length ? meta.modes : reg?.modes?.length ? reg.modes : ['light', 'dark']) as string[]
  const modes = raw.filter((m): m is 'light' | 'dark' => m === 'light' || m === 'dark')
  const list = modes.length ? modes : (['light'] as ('light' | 'dark')[])
  const defRaw = meta?.defaultMode ?? reg?.defaultMode ?? list[0]
  const defaultMode: 'light' | 'dark' =
    defRaw === 'dark' && list.includes('dark')
      ? 'dark'
      : list.includes('light')
        ? 'light'
        : list[0]!
  const dual = list.includes('light') && list.includes('dark')
  const modesLabel = dual
    ? 'Light · Dark'
    : list[0] === 'dark'
      ? 'Dark only'
      : 'Light only'
  return { modes: list, defaultMode, dual, modesLabel }
}

/**
 * Registered catalog id for interim table paint (when applyScope has no draft doc).
 * Prefer: exact id → forkedFrom (store) → my- strip exact match → safe studio + warning.
 * Never substring-heuristic matches (silent wrong parent).
 */
export function resolvePaintId(
  id: string,
  forkedFrom?: string | null,
): PaintResolve {
  if (!id) {
    return {
      paintId: SAFE_PAINT_FALLBACK,
      warning: 'No theme id for paint preview; using studio table.',
    }
  }
  if (THEMES.some((t) => t.id === id)) return { paintId: id }
  if (forkedFrom && THEMES.some((t) => t.id === forkedFrom)) {
    return { paintId: forkedFrom }
  }
  // my-studio → studio; my-ship-ab12 → ship only when exact catalog id after strip.
  const stripped = id.replace(/^my-/, '').replace(/-theme-[a-z0-9]+$/i, '')
  if (stripped && THEMES.some((t) => t.id === stripped)) {
    return { paintId: stripped }
  }
  // my-{catalog}-{stamp} where catalog is exact THEMES id
  const stampStrip = id.replace(/^my-/, '').replace(/-[a-z0-9]{3,6}$/i, '')
  if (stampStrip && stampStrip !== stripped && THEMES.some((t) => t.id === stampStrip)) {
    return { paintId: stampStrip }
  }
  return {
    paintId: SAFE_PAINT_FALLBACK,
    warning: `No registered paint table for “${id}”; preview uses ${SAFE_PAINT_FALLBACK}.`,
  }
}

/** @deprecated Prefer resolvePaintId — returns paint id only (drops warning). */
export function paintIdFor(id: string, forkedFrom?: string | null): string {
  return resolvePaintId(id, forkedFrom).paintId
}

export function themeDisplayName(id: string): string {
  const reg = THEMES.find((t) => t.id === id)
  return reg?.label ?? titleCase(id)
}

/**
 * Theme id from location for any facet path:
 * /themes/:id, /themes/:id/color|shape|assets|components
 */
export function themeIdFromLocation(pathname = ''): string {
  const path = pathname
  const base = appRouteBase()
  const rest = base && path.startsWith(base) ? path.slice(base.length) : path
  const m = rest.match(/^\/themes\/([^/]+)(?:\/(?:color|shape|assets|components))?\/?$/)
  return m ? decodeURIComponent(m[1]!) : ''
}
