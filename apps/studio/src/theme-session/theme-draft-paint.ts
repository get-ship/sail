/**
 * Theme draft paint adapter — single Studio entry for live preview on hosts.
 *
 * Edit loop (dirty draft):
 *   RAM draftDoc → theme.applyScope(host, { doc, mode }) → host only
 *   No Generate, no HMR, no :root product CSSOM.
 *
 * Default applier is framework `theme.applyScope` (T1). Tests may inject
 * via setDraftPaintApplier. Without a doc, falls back to theme/mode attrs
 * for registered-table cold path only (never data-theme-preview).
 *
 * Wire:
 *   bindPreviewHost(el) once per specimen; store emit → scheduleDraftPaint.
 */
import { theme } from '@ship/sail-runtime'
import { getDesignState } from '@ship/studio/design'

export type DraftPaintOpts = {
  doc: Record<string, unknown> | null
  mode: 'light' | 'dark'
  themeId: string
  /** Registered catalog id for interim attr paint when no draft doc. */
  paintId: string
}

/**
 * Framework (or test) applier. May return a teardown for the host.
 * Default: theme.applyScope(host, { doc, mode }).
 */
export type DraftPaintApplier = (
  host: HTMLElement,
  opts: DraftPaintOpts,
) => void | (() => void)

const hosts = new Set<HTMLElement>()
const teardowns = new Map<HTMLElement, () => void>()

/** null = use default applyScope; function = inject; false sentinel not used */
let applier: DraftPaintApplier | null | undefined = undefined
/** Optional resolve of registered paint id when host has no paintId override. */
let paintIdResolver: ((themeId: string, forkedFrom: string | null) => string) | null =
  null
let raf = 0
let lastPaintKey = ''

function defaultApplyScope(host: HTMLElement, opts: DraftPaintOpts): () => void {
  if (!opts.doc) {
    // Cold path — attrs only (no draft tokens).
    applyRegisteredAttrs(host, opts)
    return () => {
      /* attrs may stay for template dual-bind */
    }
  }
  return theme.applyScope(host, { doc: opts.doc, mode: opts.mode })
}

export function setDraftPaintApplier(fn: DraftPaintApplier | null): void {
  applier = fn
  scheduleDraftPaint()
}

export function getDraftPaintApplier(): DraftPaintApplier | null {
  if (applier === undefined) return defaultApplyScope
  return applier
}

/** Optional paint-id resolve (registered tables) for interim attr paint. */
export function setDraftPaintIdResolver(
  fn: ((themeId: string, forkedFrom: string | null) => string) | null,
): void {
  paintIdResolver = fn
}

export function bindPreviewHost(host: HTMLElement | null | undefined): void {
  if (!host) return
  hosts.add(host)
  scheduleDraftPaint()
}

export function unbindPreviewHost(host: HTMLElement | null | undefined): void {
  if (!host) return
  hosts.delete(host)
  const td = teardowns.get(host)
  if (td) {
    try {
      td()
    } catch {
      /* host may be gone */
    }
    teardowns.delete(host)
  }
  host.removeAttribute('data-draft-paint')
}

export function teardownDraftPaint(): void {
  for (const host of [...hosts]) unbindPreviewHost(host)
  hosts.clear()
  teardowns.clear()
  lastPaintKey = ''
  if (raf) {
    cancelAnimationFrame(raf)
    raf = 0
  }
}

/** Re-apply after draft mutations (batched to rAF for continuous inputs). */
export function scheduleDraftPaint(): void {
  if (typeof requestAnimationFrame === 'undefined') {
    flushDraftPaint()
    return
  }
  if (raf) return
  raf = requestAnimationFrame(() => {
    raf = 0
    flushDraftPaint()
  })
}

/** Synchronous apply — tests + callers that need paint before next frame. */
export function flushDraftPaint(override?: Partial<DraftPaintOpts>): void {
  if (hosts.size === 0 && !override) return
  const ds = getDesignState()
  const themeId = override?.themeId ?? ds.themeId ?? ''
  const mode = (override?.mode ?? ds.modeId ?? 'light') as 'light' | 'dark'
  const doc = override?.doc !== undefined ? override.doc : ds.themeDoc
  const forkedFrom = ds.forkedFrom
  const paintId =
    override?.paintId ??
    (paintIdResolver ? paintIdResolver(themeId, forkedFrom) : themeId || 'studio')
  const opts: DraftPaintOpts = { doc, mode, themeId, paintId }
  const key = `${themeId}|${mode}|${paintId}|${doc ? JSON.stringify(doc).length : 0}|${ds.dirty}`
  // Always re-apply when dirty so draft mutations repaint even if length is stable.
  if (!ds.dirty && key === lastPaintKey && !override) return
  lastPaintKey = key

  for (const host of hosts) {
    applyOne(host, opts)
  }
}

function applyOne(host: HTMLElement, opts: DraftPaintOpts): void {
  const prev = teardowns.get(host)
  if (prev) {
    try {
      prev()
    } catch {
      /* */
    }
    teardowns.delete(host)
  }

  const fn = getDraftPaintApplier()
  if (fn) {
    host.setAttribute('data-draft-paint', opts.doc ? 'apply' : 'registered')
    try {
      const td = fn(host, opts)
      if (typeof td === 'function') teardowns.set(host, td)
    } catch (e) {
      host.setAttribute('data-draft-paint', 'error')
      console.warn('[theme-draft-paint] applier failed', e)
    }
    return
  }

  // Explicit null applier (tests): registered attrs only.
  host.setAttribute('data-draft-paint', 'registered')
  applyRegisteredAttrs(host, opts)
}

function applyRegisteredAttrs(host: HTMLElement, opts: DraftPaintOpts): void {
  const id = opts.paintId || opts.themeId || 'studio'
  // Author IR `theme` + mode — never invent data-theme-preview (T1 kill list).
  host.setAttribute('theme', id)
  host.setAttribute('data-theme', id)
  host.setAttribute('data-mode', opts.mode)
}

/** Test helper — bound host count. */
export function __draftPaintHostCountForTests(): number {
  return hosts.size
}

export function __resetDraftPaintForTests(): void {
  teardownDraftPaint()
  applier = undefined
  paintIdResolver = null
  lastPaintKey = ''
}
