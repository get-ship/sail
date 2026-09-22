import { inputValue, type UiEvent } from '../ui-event'
/**
 * Shape domain — radius / elev / border / glass. Session owns open/mode/save/path.
 * Live specimen via freeform px/shadow/fill bind (not product CSS-var state).
 *
 * Radius master + recipes write mock ramp ratios sm/md/lg/xl = 0.5/1/1.5/2 × md
 * (absolute steps; not proportional-from-previous scaleTogether).
 */
import { signal } from '@ship/sail-runtime'
import { getDesignState, getPathValue, parseDimensionPx, setRadiusFamily, setTokenValue } from '../../design/index.ts'
import type { ThemeSession } from '../theme-session/theme-session'
import {
  ELEV_STEPS,
  RADIUS_KEYS,
  RECIPE_IDS,
  RECIPES,
  SHADOW_FALLBACKS,
  elevFromCardRest,
  type ElevKey,
  type RadiusKey,
  type RecipeId,
} from './theme-shape-recipes'

export { ELEV_STEPS, RADIUS_KEYS, RECIPE_IDS, type ElevKey, type RadiusKey, type RecipeId }

const BORDER_WIDTH_PATH = 'color.border.width'
const CARD_BACKDROP_PATH = 'filter.backdrop'
const CARD_REST_PATH = 'shadow.card-rest'

/** Mock SSOT ramp relative to md: sm / md / lg / xl. */
export const RADIUS_RAMP = { sm: 0.5, md: 1, lg: 1.5, xl: 2 } as const

export function formatPxLabel(n: number): string {
  const r = Math.round(n * 10) / 10
  return `${r}`
}

/** Absolute mock family from master (md) px. Soft master 10 → 5 / 10 / 15 / 20. */
export function mockRadiusFamily(masterMd: number): {
  sm: number
  md: number
  lg: number
  xl: number
} {
  const md = Math.max(0, masterMd)
  return {
    sm: Math.max(0, Math.round(md * RADIUS_RAMP.sm * 10) / 10),
    md: Math.round(md * 10) / 10,
    lg: Math.round(md * RADIUS_RAMP.lg * 10) / 10,
    xl: Math.round(md * RADIUS_RAMP.xl * 10) / 10,
  }
}

/**
 * Apply alpha to an oklch/color string for frosted card fill.
 * Glass catalog uses e.g. `oklch(1 0 0 / 55%)` on the live specimen fill.
 */
export function withColorAlpha(raw: string | null | undefined, alpha: number): string {
  const a = Math.max(0.1, Math.min(1, alpha))
  const pct = Math.round(a * 100)
  const src = (raw ?? '').trim()
  if (/^oklch\(/i.test(src)) {
    const m = src.match(/^oklch\(\s*([^/)]+?)(?:\s*\/\s*[^)]+)?\s*\)$/i)
    if (m) return `oklch(${m[1]!.trim()} / ${pct}%)`
  }
  // Opaque / unknown → light frosted plate (live freeform + write fallback).
  return `oklch(1 0 0 / ${pct}%)`
}

/** Strip alpha → opaque oklch components when leaving glass. */
export function stripColorAlpha(raw: string | null | undefined): string {
  const src = (raw ?? '').trim()
  if (/^oklch\(/i.test(src)) {
    const m = src.match(/^oklch\(\s*([^/)]+?)(?:\s*\/\s*[^)]+)?\s*\)$/i)
    if (m) return `oklch(${m[1]!.trim()})`
  }
  return src || 'oklch(1 0 0)'
}

function parseBlurPx(raw: string | null): number {
  if (!raw || raw === 'none') return 0
  const m = raw.match(/blur\(\s*([0-9.]+)\s*px\s*\)/i)
  if (m) return Number(m[1]) || 0
  const n = parseDimensionPx(raw)
  return n ?? 0
}

export function createShapeEdit(session: ThemeSession) {
  const recipeId = signal<RecipeId>('soft')
  const recipeLabel = signal('soft')
  const focusStep = signal<RadiusKey>('md')
  const masterPx = signal(10)
  const masterLabel = signal('10px')
  const elevKey = signal<ElevKey>('md')
  const elevLabel = signal('md')
  const borderW = signal(1)
  const borderWLabel = signal('1px')
  const surfaceOp = signal(0.72)
  const surfaceOpPct = signal(72)
  const surfaceOpLabel = signal('72%')
  const surfaceBlur = signal(14)
  const surfaceBlurLabel = signal('14px')
  const showGlass = signal(false)
  const radiusReadout = signal('sm 4 · md 6 · lg 10 · xl 14')
  const radiusSm = signal(4)
  const radiusMd = signal(6)
  const radiusLg = signal(10)
  const radiusXl = signal(14)
  const radiusSmLabel = signal('4')
  const radiusMdLabel = signal('6')
  const radiusLgLabel = signal('10')
  const radiusXlLabel = signal('14')
  const frostFill = derived<string>(() =>
    showGlass() ? withColorAlpha(cardBaseColor, surfaceOp()) : stripColorAlpha(cardBaseColor) || 'oklch(1 0 0)',
  )
  const statRadius = signal('6px')
  const statShadow = signal('md')
  const statBorder = signal('1px')
  const statRecipe = signal('soft · scaled family')

  /** Freeform fill — translucent when glass, solid otherwise. */
  /** Packed freeform CSS for use:liveBox (specimen + corner/ladder chips). */
  /** Footer mono line — radius.md + elev step (mock canvas-foot). */
  const liveCssLine = signal('--radius-md: 6px; --shadow-card-rest: md')

  let applyingRecipe = false
  let elevLocal: ElevKey | null = null
  let recipeLocal: RecipeId | null = null
  /** Base card color without alpha (for glass α dials). */
  const cardBaseColor = 'oklch(1 0 0)'

  const markCustom = () => {
    if (applyingRecipe) return
    recipeLocal = 'custom'
    recipeId.set('custom')
    recipeLabel.set('custom')
    statRecipe.set('custom · free dials')
  }

  const shadowValueForElev = (
    doc: Record<string, unknown> | null,
    mode: 'light' | 'dark',
    elev: ElevKey,
  ): string => {
    if (elev === 'none') return 'none'
    const fromDoc = getPathValue(doc, `shadow.${elev}`, mode)
    return fromDoc ?? SHADOW_FALLBACKS[elev]
  }

  const applyLocalRadius = (fam: { sm: number; md: number; lg: number; xl: number }) => {
    radiusSm.set(fam.sm)
    radiusMd.set(fam.md)
    radiusLg.set(fam.lg)
    radiusXl.set(fam.xl)
    radiusSmLabel.set(formatPxLabel(fam.sm))
    radiusMdLabel.set(formatPxLabel(fam.md))
    radiusLgLabel.set(formatPxLabel(fam.lg))
    radiusXlLabel.set(formatPxLabel(fam.xl))
    masterPx.set(fam.md)
    masterLabel.set(`${formatPxLabel(fam.md)}px`)
    radiusReadout.set(
      `sm ${formatPxLabel(fam.sm)} · md ${formatPxLabel(fam.md)} · lg ${formatPxLabel(fam.lg)} · xl ${formatPxLabel(fam.xl)}`,
    )
    statRadius.set(`${formatPxLabel(fam.md)}px`)
  }

  const pushLiveShape = () => {
    liveCssLine.set(`--radius-md: ${formatPxLabel(radiusMd())}px; --shadow-card-rest: ${elevKey()}`)
  }

  /**
   * Write absolute mock ramp (0.5/1/1.5/2 × md). Temporarily disables
   * scaleTogether so steps are not re-scaled from the previous family.
   */
  const writeRadiusFamily = async (masterMd: number) => {
    const fam = mockRadiusFamily(masterMd)
    applyLocalRadius(fam)
    pushLiveShape()
    // One store write — sequential setRadiusStep re-clamped mid-ladder (glitchy master drag).
    await setRadiusFamily(
      [
        { key: 'sm', px: fam.sm },
        { key: 'md', px: fam.md },
        { key: 'lg', px: fam.lg },
        { key: 'xl', px: fam.xl },
      ],
      true,
    )
  }

  const writeElev = async (elev: ElevKey) => {
    elevLocal = elev
    elevKey.set(elev)
    elevLabel.set(elev)
    statShadow.set(elev)
    const ds = getDesignState()
    const val = shadowValueForElev(ds.themeDoc, ds.modeId, elev)
    await setTokenValue(CARD_REST_PATH, val)
  }

  const writeBorderW = async (px: number) => {
    const clamped = Math.max(0, Math.min(3, px))
    borderW.set(clamped)
    borderWLabel.set(`${formatPxLabel(clamped)}px`)
    statBorder.set(`${formatPxLabel(clamped)}px`)
    await setTokenValue(BORDER_WIDTH_PATH, `${clamped}px`)
  }

  const writeGlass = async (op: number, blur: number) => {
    const o = Math.max(0.1, Math.min(1, op))
    const b = Math.max(0, Math.min(28, blur))
    surfaceOp.set(o)
    surfaceOpPct.set(Math.round(o * 100))
    surfaceOpLabel.set(`${Math.round(o * 100)}%`)
    surfaceBlur.set(b)
    surfaceBlurLabel.set(`${b}px`)
    showGlass.set(true)
    const fill = withColorAlpha(cardBaseColor, o)
    // Backdrop blur persists; frost fill stays live-specimen freeform.
    await setTokenValue(CARD_BACKDROP_PATH, b > 0 ? `blur(${b}px)` : 'none')
  }

  const clearGlass = async () => {
    surfaceOp.set(1)
    surfaceOpPct.set(100)
    surfaceOpLabel.set('100%')
    surfaceBlur.set(0)
    surfaceBlurLabel.set('0px')
    showGlass.set(false)
    const solid = stripColorAlpha(cardBaseColor)
    await setTokenValue(CARD_BACKDROP_PATH, 'none')
  }

  const sync = () => {
    const ds = getDesignState()
    const doc = ds.themeDoc
    const writeMode = session.writeMode()

    const fam = ds.radius
    const byKey = new Map(fam.steps.map((s) => [s.key, s]))
    const sm = byKey.get('sm')?.valuePx ?? 4
    const md = byKey.get('md')?.valuePx ?? 6
    const lg = byKey.get('lg')?.valuePx ?? 10
    const xl = byKey.get('xl')?.valuePx ?? 14
    radiusSm.set(sm)
    radiusMd.set(md)
    radiusLg.set(lg)
    radiusXl.set(xl)
    radiusSmLabel.set(formatPxLabel(sm))
    radiusMdLabel.set(formatPxLabel(md))
    radiusLgLabel.set(formatPxLabel(lg))
    radiusXlLabel.set(formatPxLabel(xl))
    masterPx.set(md)
    masterLabel.set(`${formatPxLabel(md)}px`)
    radiusReadout.set(
      `sm ${formatPxLabel(sm)} · md ${formatPxLabel(md)} · lg ${formatPxLabel(lg)} · xl ${formatPxLabel(xl)}`,
    )
    statRadius.set(`${formatPxLabel(md)}px`)

    const bwRaw = getPathValue(doc, BORDER_WIDTH_PATH, writeMode)
    const bw = bwRaw != null ? (parseDimensionPx(bwRaw) ?? 1) : borderW()
    borderW.set(bw)
    borderWLabel.set(`${formatPxLabel(bw)}px`)
    statBorder.set(`${formatPxLabel(bw)}px`)

    const cardRest = getPathValue(doc, CARD_REST_PATH, writeMode)
    const elev = elevLocal ?? elevFromCardRest(cardRest)
    elevKey.set(elev)
    elevLabel.set(elev)
    statShadow.set(elev)

    const blurRaw = getPathValue(doc, CARD_BACKDROP_PATH, writeMode)
    if (blurRaw != null) {
      const b = parseBlurPx(blurRaw)
      surfaceBlur.set(b)
      surfaceBlurLabel.set(`${b}px`)
    }

    const rid = recipeLocal ?? recipeId()
    recipeId.set(rid)
    recipeLabel.set(rid)
    const glassOn =
      rid === 'glass' || (rid === 'custom' && (surfaceBlur() > 0 || surfaceOp() < 0.999))
    showGlass.set(glassOn)
    if (rid === 'custom') statRecipe.set('custom · free dials')
    else statRecipe.set(`${rid} · scaled family`)

    pushLiveShape()
  }

  session.onSync(sync)
  sync()

  const recipes = () =>
    RECIPE_IDS.map((id) => ({
      key: id,
      id,
      label: id.charAt(0).toUpperCase() + id.slice(1),
      on: recipeId() === id,
    }))

  const elevChips = () =>
    ELEV_STEPS.map((s) => ({
      key: s.key,
      elev: s.key,
      label: s.label,
      hint: s.hint,
      on: elevKey() === s.key,
    }))

  /** Closed-over recipe id — no dataset bus. */
  const onRecipeClick = (id: Exclude<RecipeId, 'custom'>) => {
    if (!session.guardEdit()) return
    if (!id || !(id in RECIPES)) return
    const r = RECIPES[id]
    applyingRecipe = true
    recipeLocal = id
    recipeId.set(id)
    recipeLabel.set(id)
    elevLocal = r.elev
    void (async () => {
      // Absolute mock ramp (Soft master 10 → 5/10/15/20), not proportional-from-prev.
      await writeRadiusFamily(r.master)
      await writeElev(r.elev)
      await writeBorderW(r.borderW)
      if (r.glass) {
        await writeGlass(r.surfaceOp, r.surfaceBlur)
      } else {
        await clearGlass()
      }
      applyingRecipe = false
      showGlass.set(r.glass)
      statRecipe.set(`${id} · scaled family`)
      session.sync()
    })()
  }

  const onRadiusChip = (key: RadiusKey) => {
    if (!key || !RADIUS_KEYS.includes(key)) return
    focusStep.set(key)
  }

  const onMasterInput = (e: UiEvent) => {
    if (!session.guardEdit()) return
    const n = Number(inputValue(e))
    if (!Number.isFinite(n)) return
    markCustom()
    // Absolute mock ramp — local + store stay aligned (no optimistic flash/snap).
    void writeRadiusFamily(n).then(() => session.sync())
  }

  const onElevClick = (key: ElevKey) => {
    if (!session.guardEdit()) return
    if (!key || !ELEV_STEPS.some((s) => s.key === key)) return
    markCustom()
    void writeElev(key).then(() => session.sync())
  }

  const onBorderWidth = (e: UiEvent) => {
    if (!session.guardEdit()) return
    const n = Number(inputValue(e))
    if (!Number.isFinite(n)) return
    markCustom()
    void writeBorderW(n).then(() => session.sync())
  }

  const onSurfaceOp = (e: UiEvent) => {
    if (!session.guardEdit()) return
    const pct = Number(inputValue(e))
    if (!Number.isFinite(pct)) return
    markCustom()
    void writeGlass(pct / 100, surfaceBlur()).then(() => {
      showGlass.set(true)
      session.sync()
    })
  }

  const onSurfaceBlur = (e: UiEvent) => {
    if (!session.guardEdit()) return
    const n = Number(inputValue(e))
    if (!Number.isFinite(n)) return
    markCustom()
    void writeGlass(surfaceOp(), n).then(() => {
      showGlass.set(true)
      session.sync()
    })
  }

  return {
    recipeId,
    recipeLabel,
    focusStep,
    masterPx,
    masterLabel,
    elevKey,
    elevLabel,
    borderW,
    borderWLabel,
    surfaceOp,
    surfaceOpPct,
    surfaceOpLabel,
    surfaceBlur,
    surfaceBlurLabel,
    showGlass,
    radiusReadout,
    radiusSmLabel,
    radiusMdLabel,
    radiusLgLabel,
    radiusLg,
    radiusXlLabel,
    statRadius,
    statShadow,
    statBorder,
    statRecipe,
    frostFill,
    liveCssLine,
    recipes,
    elevChips,
    onRecipeClick,
    onRadiusChip,
    onMasterInput,
    onElevClick,
    onBorderWidth,
    onSurfaceOp,
    onSurfaceBlur,
  }
}
