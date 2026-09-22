/**
 * Theme draft paint adapter — single Studio entry for live preview on hosts.
 *
 * Edit loop (dirty draft):
 *   RAM draftDoc → theme.applyScope(host, { doc, mode }) → host only
 *   No Generate, no HMR, no :root product CSSOM.
 *
 * Default applier is framework `theme.applyScope` (T1). Tests may inject
 * via setDraftPaintApplier. Without a doc, falls back to theme/mode attrs
 * for registered-table paint only (never theme).
 *
 * Wire:
 *   bindPreviewHost(el) once per specimen; store emit → scheduleDraftPaint.
 *
 * Host handle type is SailHost (Pure-safe). Attr writes go through studio-shell
 * Framework helpers so this composition never names DOM methods.
 */
import { getDesignState } from '../../design/index.ts'
import {
  hostRemoveAttribute,
  hostSetAttribute,
  scheduleFrame,
} from '@ship/sail-studio-shell'
import { theme, type SailHost } from '@ship/sail-runtime'

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
  host: SailHost,
  opts: DraftPaintOpts,
) => void | (() => void)

const hosts = new Set<SailHost>()
const teardowns = new Map<SailHost, () => void>()

/** null = use default applyScope; function = inject; false sentinel not used */
let applier: DraftPaintApplier | null | undefined = undefined
/** Optional resolve of registered paint id when host has no paintId override. */
let paintIdResolver: ((themeId: string, forkedFrom: string | null) => string) | null =
  null
let raf = 0
let lastPaintKey = ''

function defaultApplyScope(host: SailHost, opts: DraftPaintOpts): () => void {
  if (!opts.doc) {
    // Attr-only — no draft tokens.
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

export function bindPreviewHost(host: SailHost | null | undefined): void {
  if (!host) return
  hosts.add(host)
  scheduleDraftPaint()
}

export function unbindPreviewHost(host: SailHost | null | undefined): void {
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
  hostRemoveAttribute(host, 'draft-paint')
}

export function teardownDraftPaint(): void {
  for (const host of [...hosts]) unbindPreviewHost(host)
  hosts.clear()
  teardowns.clear()
  lastPaintKey = ''
  raf = 0
}

/** Re-apply after draft mutations (batched to rAF for continuous inputs). */
export function scheduleDraftPaint(): void {
  if (raf) return
  raf = 1
  scheduleFrame(() => {
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

function applyOne(host: SailHost, opts: DraftPaintOpts): void {
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
    hostSetAttribute(host, 'draft-paint', opts.doc ? 'apply' : 'registered')
    try {
      const td = fn(host, opts)
      if (typeof td === 'function') teardowns.set(host, td)
    } catch (e) {
      hostSetAttribute(host, 'draft-paint', 'error')
      console.warn('[theme-draft-paint] applier failed', e)
    }
    return
  }

  // Explicit null applier (tests): registered attrs only.
  hostSetAttribute(host, 'draft-paint', 'registered')
  applyRegisteredAttrs(host, opts)
}

function applyRegisteredAttrs(host: SailHost, opts: DraftPaintOpts): void {
  const id = opts.paintId || opts.themeId || 'studio'
  // Author IR `theme` + `mode` — paint authority for registered tables
  // (`[theme=id][mode=…]` in themes.css). Never data-*.
  hostSetAttribute(host, 'theme', id)
  hostRemoveAttribute(host, 'data-theme')
  hostRemoveAttribute(host, 'data-theme-preview')
  hostRemoveAttribute(host, 'data-mode')
  hostSetAttribute(host, 'mode', opts.mode)
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
