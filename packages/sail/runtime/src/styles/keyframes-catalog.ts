// Closed product keyframe catalog — ambient CSS loops for L1 surfaces.
//
// SSOT for product @keyframes + --animate-* vars. Style.animate(name)
// lowers to `animation: var(--animate-{name})`; L1 skins (status-dot,
// skeleton, progress) reference the same vars. Not the fx: interaction
// catalog (crates/sail/lint/animation-presets.json) — infinite spinners /
// shimmers stay on CSS PE, not fx:loop.
//
// Closed ambient loops; materialised in sail-base.css.

export type KeyframeCatalogEntry = {
  /** Closed product name (`spin`, `shimmer`, …). */
  name: string
  /** CSS @keyframes identifier (`sail-spin`). */
  keyframe: string
  /** Body inside `@keyframes <keyframe> { … }`. */
  frames: string
  /** Full animation shorthand stored on `--animate-{name}`. */
  animation: string
  /** Optional reduced-motion override of the animation shorthand. */
  reducedMotion?: string
  /** When true, Style.animate(name) is a legal PE. */
  styleAnimate: boolean
  doc: string
}

/**
 * Closed product keyframe catalog (order = emit order).
 * Add a row here to ship a new ambient loop; do not hand-author @keyframes
 * in styles/index.ts skins.
 */
export const KEYFRAME_CATALOG: readonly KeyframeCatalogEntry[] = [
  {
    name: 'spin',
    keyframe: 'sail-spin',
    frames: 'to { transform: rotate(360deg); }',
    animation: 'sail-spin 0.7s linear infinite',
    reducedMotion: 'sail-spin 1.5s linear infinite',
    styleAnimate: true,
    doc: 'Continuous rotation — spinner rings and Style.animate(spin)',
  },
  {
    name: 'pulse',
    keyframe: 'sail-status-dot-pulse',
    frames:
      '0%, 100% { opacity: 1; transform: scale(1); } 50% { opacity: 0.82; transform: scale(0.92); }',
    animation: 'sail-status-dot-pulse 2.5s var(--ease-out) infinite',
    styleAnimate: false,
    doc: 'Status-dot attention pulse (span[dot][pulse])',
  },
  {
    name: 'shimmer',
    keyframe: 'sail-skeleton-shimmer',
    frames: '0% { background-position: 200% 0; } 100% { background-position: -200% 0; }',
    animation: 'sail-skeleton-shimmer 1.5s ease-in-out infinite',
    styleAnimate: false,
    doc: 'Skeleton placeholder sheen (span[skeleton])',
  },
  {
    name: 'progress-indeterminate',
    keyframe: 'sail-progress-indeterminate',
    frames: '0% { background-position: -40% 0; } 100% { background-position: 140% 0; }',
    animation: 'sail-progress-indeterminate 1.2s ease-in-out infinite',
    styleAnimate: false,
    doc: 'Indeterminate progress barber-pole (progress:not([value]))',
  },
]

/** Names legal for Style.animate(name). */
export const STYLE_ANIMATE_NAMES: readonly string[] = KEYFRAME_CATALOG.filter(
  (e) => e.styleAnimate,
).map((e) => e.name)

/**
 * Emit the ambient keyframes sheet: every catalog `@keyframes` + `:root`
 * `--animate-*` vars (with optional reduced-motion overrides). Always-on
 * for contracts; skins and Style PE reference the vars, never re-declare
 * keyframes.
 */
export function emitKeyframeCatalogCss(
  entries: readonly KeyframeCatalogEntry[] = KEYFRAME_CATALOG,
): string {
  let out = ''
  const rootVars: string[] = []
  const rmVars: string[] = []
  for (const e of entries) {
    out += `@keyframes ${e.keyframe} { ${e.frames} }\n`
    rootVars.push(`--animate-${e.name}: ${e.animation};`)
    if (e.reducedMotion) {
      rmVars.push(`--animate-${e.name}: ${e.reducedMotion};`)
    }
  }
  out += `:root { ${rootVars.join(' ')} }\n`
  if (rmVars.length > 0) {
    out += `@media (prefers-reduced-motion: reduce) { :root { ${rmVars.join(' ')} } }\n`
  }
  return out
}

/** Ambient sheet string (reference; sail-base.css is authoritative). */
export const AMBIENT_KEYFRAMES_CSS = emitKeyframeCatalogCss()
