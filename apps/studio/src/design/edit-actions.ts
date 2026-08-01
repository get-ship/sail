/**
 * Design edit actions — density, scale families, type, icons, path writes.
 */
import {
  BRAND_FG_PATH,
  BRAND_PRIMARY_PATH,
  hexToOklchString,
  isHex,
  normHex,
  toPickerHex,
} from './color'
import {
  defaultIconFamily,
  defaultRadiusFamily,
  setScaleStep,
  validateMonotonic,
} from './scale'
import { specimenSource, SPECIMEN_VIEWPORT } from './specimen'
import {
  deletePath,
  densityOf,
  extractIconScale,
  extractRadius,
  patchIconScale,
  patchRadius,
  patchTypeRoles,
  setDensity,
  setPathValue,
} from './theme-doc'
import { getKnob } from './token-groups'
import type {
  BlastRadius,
  DensityId,
  DesignOpsClient,
  DesignSurfaceState,
  TypeRoles,
} from './types'
import { userError } from '../user-error'

export type EditCtx = {
  getClient: () => DesignOpsClient | null
  getState: () => DesignSurfaceState
  setState: (s: DesignSurfaceState) => void
  recomputeDerived: (doc: Record<string, unknown> | null) => void
  emit: () => void
  repaintSpecimen: () => Promise<void>
  refreshBlast: (token: string) => Promise<void>
  reflectCss: (cssName: string, value: string | null) => void
}

export async function setDesignDensityOn(
  ctx: EditCtx,
  density: DensityId,
): Promise<void> {
  const state = ctx.getState()
  if (!state.themeDoc) {
    ctx.setState({ ...state, density })
    ctx.emit()
    await ctx.repaintSpecimen()
    return
  }
  const doc = setDensity(state.themeDoc, density)
  ctx.recomputeDerived(doc)
  ctx.setState({ ...ctx.getState(), density, lastWriteError: null })
  ctx.emit()
  await ctx.repaintSpecimen()
}

export async function setRadiusStepOn(
  ctx: EditCtx,
  key: string,
  px: number,
): Promise<void> {
  const state = ctx.getState()
  const { family, error, clampNote } = setScaleStep(state.radius, key, px)
  if (error && !family.scaleTogether && !clampNote) {
    ctx.setState({
      ...state,
      lastWriteError: 'Radius steps must go from small to large',
      lastClampNote: null,
      radius: family,
    })
    ctx.emit()
    return
  }
  for (const step of family.steps) {
    ctx.reflectCss(`radius-${step.key}`, step.raw)
  }
  ctx.reflectCss('radius', family.steps.find((s) => s.key === 'lg')?.raw ?? null)
  if (!state.themeDoc) {
    ctx.setState({
      ...state,
      radius: family,
      lastWriteError: error && !clampNote ? error : null,
      lastClampNote: clampNote,
      statusNote: clampNote ?? state.statusNote,
    })
    ctx.emit()
    await ctx.repaintSpecimen()
    return
  }
  const doc = patchRadius(state.themeDoc, family, state.modeId)
  const monoErr = validateMonotonic(family.steps, family.ordered, family.path)
  if (monoErr && !family.scaleTogether && !clampNote) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: 'Radius steps must go from small to large',
      lastClampNote: null,
      radius: family,
    })
    ctx.emit()
    return
  }
  ctx.recomputeDerived(doc)
  ctx.setState({
    ...ctx.getState(),
    radius: {
      ...extractRadius(doc, state.modeId),
      scaleTogether: family.scaleTogether,
    },
    lastWriteError: null,
    lastClampNote: clampNote,
    statusNote: clampNote ?? `Updated radius · ${key}`,
  })
  ctx.emit()
  await ctx.repaintSpecimen()
  void ctx.refreshBlast(`radius-${key}`)
}

export async function overrideComponentRadiusOn(
  ctx: EditCtx,
  component: string,
  value: string | null,
): Promise<void> {
  const state = ctx.getState()
  if (!state.themeDoc) return
  const path = `component.${component}.radius`
  const doc =
    value == null
      ? deletePath(state.themeDoc, path, state.modeId)
      : setPathValue(state.themeDoc, path, value, 'dimension', state.modeId)
  ctx.recomputeDerived(doc)
  ctx.setState({ ...ctx.getState(), lastWriteError: null })
  ctx.emit()
  await ctx.repaintSpecimen()
}

export async function setTypeRoleOn(
  ctx: EditCtx,
  role: keyof TypeRoles,
  family: string,
): Promise<void> {
  const state = ctx.getState()
  if (!state.themeDoc) {
    ctx.setState({
      ...state,
      typeRoles: { ...state.typeRoles, [role]: family },
    })
    ctx.emit()
    return
  }
  const roles = { ...state.typeRoles, [role]: family }
  const doc = patchTypeRoles(state.themeDoc, roles, state.modeId)
  ctx.recomputeDerived(doc)
  ctx.setState({ ...ctx.getState(), typeRoles: roles, lastWriteError: null })
  ctx.emit()
  await ctx.repaintSpecimen()
}

export async function setIconScaleStepOn(
  ctx: EditCtx,
  key: string,
  px: number,
): Promise<void> {
  const state = ctx.getState()
  const { family, error, clampNote } = setScaleStep(state.iconScale, key, px)
  if (error && !family.scaleTogether && !clampNote) {
    ctx.setState({
      ...state,
      lastWriteError: error,
      lastClampNote: null,
      iconScale: family,
    })
    ctx.emit()
    return
  }
  for (const step of family.steps) {
    ctx.reflectCss(`icon-size-${step.key}`, step.raw)
  }
  if (!state.themeDoc) {
    ctx.setState({
      ...state,
      iconScale: family,
      lastWriteError: error && !clampNote ? error : null,
      lastClampNote: clampNote,
    })
    ctx.emit()
    return
  }
  const doc = patchIconScale(state.themeDoc, family, state.modeId)
  ctx.recomputeDerived(doc)
  ctx.setState({
    ...ctx.getState(),
    iconScale: {
      ...extractIconScale(doc, state.modeId),
      scaleTogether: family.scaleTogether,
    },
    lastWriteError: null,
    lastClampNote: clampNote,
  })
  ctx.emit()
}

/**
 * Brand color decision — hex from sail-color-picker → oklch write on primary.
 * Also co-writes primary-foreground for AA when L is extreme.
 * Ramp in the panel is preview-only (derived; not persisted as a scale).
 */
export async function setBrandColorOn(
  ctx: EditCtx,
  hexOrOklch: string,
): Promise<void> {
  const raw = String(hexOrOklch ?? '').trim()
  if (!raw) return
  const oklch = raw.startsWith('oklch')
    ? raw
    : (() => {
        try {
          return hexToOklchString(raw)
        } catch {
          return null
        }
      })()
  if (!oklch) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: 'That color is not valid',
    })
    ctx.emit()
    return
  }
  const hex = toPickerHex(oklch)
  ctx.reflectCss('primary', oklch)
  const state = ctx.getState()
  if (!state.themeDoc) {
    ctx.setState({
      ...state,
      statusNote: 'Preview brand color',
      lastWriteError: null,
    })
    ctx.emit()
    return
  }
  let doc = setPathValue(
    state.themeDoc,
    BRAND_PRIMARY_PATH,
    oklch,
    'color',
    state.modeId,
  )
  // Co-write label color so Primary buttons stay readable (soft auto).
  const L = (() => {
    const m = oklch.match(/^oklch\(\s*([\d.]+)/i)
    return m ? Number(m[1]) : 0.5
  })()
  const fg = L >= 0.55 ? 'oklch(0.18 0.02 48)' : 'oklch(0.98 0.01 90)'
  doc = setPathValue(doc, BRAND_FG_PATH, fg, 'color', state.modeId)
  ctx.reflectCss('primary-foreground', fg)
  ctx.recomputeDerived(doc)
  ctx.setState({
    ...ctx.getState(),
    lastWriteError: null,
    lastClampNote: null,
    statusNote: `Brand color · ${hex}`,
  })
  ctx.emit()
  await ctx.repaintSpecimen()
  void ctx.refreshBlast('primary')
}

/** Write a single registry token into the working theme doc + live specimen. */
export async function setTokenValueOn(
  ctx: EditCtx,
  path: string,
  value: string,
): Promise<void> {
  const knob = getKnob(path)
  const cssName = knob?.cssName ?? path
  const type = knob?.type ?? 'string'
  let writeValue = value
  // Color picker commits hex — store oklch (6-digit only).
  if (knob?.kind === 'color' || path.includes('color.')) {
    const h = normHex(value.trim())
    if (h && isHex(h)) writeValue = hexToOklchString(h)
  }
  ctx.reflectCss(cssName, writeValue)

  // Radius family steps — keep scale engine in sync.
  if (path.startsWith('radius.') && path !== 'radius.default') {
    const key = path.slice('radius.'.length)
    const n = Number(String(value).replace(/px$/, ''))
    if (Number.isFinite(n) && ['sm', 'md', 'lg', 'xl'].includes(key)) {
      await setRadiusStepOn(ctx, key, n)
      return
    }
  }

  if (path === BRAND_PRIMARY_PATH || path === 'color.primary.default') {
    await setBrandColorOn(ctx, writeValue)
    return
  }

  const state = ctx.getState()
  if (!state.themeDoc) {
    ctx.setState({
      ...state,
      statusNote: `Preview ${knob?.label ?? path}`,
    })
    ctx.emit()
    return
  }
  const doc = setPathValue(state.themeDoc, path, writeValue, type, state.modeId)
  ctx.recomputeDerived(doc)
  ctx.setState({
    ...ctx.getState(),
    lastWriteError: null,
    lastClampNote: null,
    statusNote: `Updated ${knob?.label ?? path}`,
  })
  ctx.emit()
  await ctx.repaintSpecimen()
}

export async function clearTokenValueOn(ctx: EditCtx, path: string): Promise<void> {
  const knob = getKnob(path)
  if (knob) ctx.reflectCss(knob.cssName, null)
  const state = ctx.getState()
  if (!state.themeDoc) return
  const doc = deletePath(state.themeDoc, path, state.modeId)
  ctx.recomputeDerived(doc)
  ctx.setState({ ...ctx.getState(), lastWriteError: null })
  ctx.emit()
  await ctx.repaintSpecimen()
}

export function previewInAppOn(ctx: EditCtx): {
  ok: boolean
  patchKeys: string[]
  message: string
} {
  const state = ctx.getState()
  const patch: Record<string, string> = {}
  for (const step of state.radius.steps) {
    patch[`--radius-${step.key}`] = step.raw
  }
  for (const step of state.iconScale.steps) {
    patch[`--icon-size-${step.key}`] = step.raw
  }
  const density = state.density
  const themeId = state.themeId

  if (typeof document !== 'undefined') {
    const root = document.documentElement
    if (themeId) root.setAttribute('data-theme', themeId)
    if (density === 'default') root.removeAttribute('data-density')
    else root.setAttribute('data-density', density)
    for (const [name, value] of Object.entries(patch)) {
      root.style.setProperty(name, value)
    }
  }

  if (typeof window !== 'undefined') {
    window.postMessage(
      {
        channel: 'sail-studio',
        type: 'theme-preview',
        themeId,
        density,
        patch,
      },
      '*',
    )
  }

  const message = `preview ${themeId} · density ${density} · ${Object.keys(patch).length} tokens`
  ctx.setState({ ...ctx.getState(), statusNote: message })
  ctx.emit()
  return { ok: true, patchKeys: Object.keys(patch), message }
}

export async function repaintSpecimenOn(ctx: EditCtx): Promise<void> {
  const client = ctx.getClient()
  const state = ctx.getState()
  if (!client?.sailRenderScreenshot) {
    ctx.setState({ ...state, specimenPng: null, specimenMs: null })
    ctx.emit()
    return
  }
  const source = specimenSource({
    density: state.density,
    typeRoles: state.typeRoles,
  })
  try {
    const t0 = Date.now()
    const r = await client.sailRenderScreenshot({
      source,
      theme: state.themeId,
      mode: state.modeId,
      viewport: SPECIMEN_VIEWPORT,
    })
    ctx.setState({
      ...ctx.getState(),
      specimenPng: r.imagePng ?? null,
      specimenMs: r.ms ?? Date.now() - t0,
    })
    ctx.emit()
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      specimenPng: null,
      specimenMs: null,
      error:
        ctx.getState().error ??
        userError(e, 'Could not update the preview — try again.'),
    })
    ctx.emit()
  }
}

export async function refreshBlastOn(
  ctx: EditCtx,
  token: string,
): Promise<void> {
  await refreshBlastAggregateOn(ctx, [token], 0)
}

/** Aggregate token-usage for dirty-path tokens (save-review blast). */
export async function refreshBlastAggregateOn(
  ctx: EditCtx,
  tokens: string[],
  dirtyCount = 0,
): Promise<void> {
  const client = ctx.getClient()
  const list = tokens.length > 0 ? tokens : ['primary']
  if (!client?.sailInspectTokenUsage) {
    const n = dirtyCount || list.length
    ctx.setState({
      ...ctx.getState(),
      blast: {
        token: list.join(','),
        components: 0,
        componentIds: [],
        sites: [],
        label:
          n > 0
            ? `${n} changed path${n === 1 ? '' : 's'} · usage not available offline`
            : 'Not used in this project yet',
      },
    })
    ctx.emit()
    return
  }
  try {
    const componentIds = new Set<string>()
    const sites: { file: string; span?: { line?: number; col?: number } }[] = []
    let components = 0
    for (const token of list.slice(0, 6)) {
      const r = await client.sailInspectTokenUsage({ token })
      components += r.components ?? 0
      for (const id of r.componentIds ?? []) componentIds.add(id)
      for (const s of r.sites ?? []) {
        const o = s as { file?: string; span?: { line?: number; col?: number } }
        sites.push({ file: o.file ?? '', span: o.span })
      }
    }
    const uniqueComponents = componentIds.size || components
    const surfaces = new Set(sites.map((s) => s.file).filter(Boolean)).size
    const pathNote =
      dirtyCount > 0
        ? `${dirtyCount} changed path${dirtyCount === 1 ? '' : 's'} · `
        : ''
    const blast: BlastRadius = {
      token: list.join(','),
      components: uniqueComponents,
      componentIds: [...componentIds],
      sites,
      label:
        uniqueComponents > 0
          ? `${pathNote}repaints ${uniqueComponents} component${uniqueComponents === 1 ? '' : 's'} across ${Math.max(1, surfaces)} surface${surfaces === 1 ? '' : 's'}`
          : `${pathNote}not used in this project yet`,
    }
    ctx.setState({ ...ctx.getState(), blast })
    ctx.emit()
  } catch {
    /* soft */
  }
}

export { defaultRadiusFamily, defaultIconFamily, densityOf }
