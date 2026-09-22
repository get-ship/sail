/**
 * Color domain — ramp editor over the engine (derive_scale / check_theme).
 * Contrast and steps come from ops; this file does not invent ratios.
 */
import { signal } from '@ship/sail-runtime'
import {
  generateTheme,
  getDesignOpsClient,
  getDesignState,
  getPathValue,
  setAnchorColor,
  setDesignDensity,
  setDesignMode,
  setTokenValue,
  tryPickerHex,
} from '../../design/index.ts'
import {
  anchorsForMode,
  fulfilledDimensions,
} from '../../design/theme-doc'
import type { DensityId } from '../../design/types'
import type { ThemeSession } from '../theme-session/theme-session'

export const COLOR_ROLES = {
  primary: { path: 'color.primary.solid', label: 'Primary', displayPath: 'color.primary', role: 'pr', section: 'brand' },
  secondary: { path: 'color.secondary.solid', label: 'Secondary', displayPath: 'color.secondary', role: 'sc', section: 'brand' },
  accent: { path: 'color.accent.solid', label: 'Accent', displayPath: 'color.accent', role: 'ac', section: 'brand' },
  bg: { path: 'color.background', label: 'Background', displayPath: 'color.background', role: 'bg', section: 'surfaces' },
  surface: { path: 'color.surface', label: 'Surface', displayPath: 'color.surface', role: 'sf', section: 'surfaces' },
  fg: { path: 'color.foreground', label: 'Foreground', displayPath: 'color.foreground', role: 'fg', section: 'text' },
  muted: { path: 'color.muted.foreground', label: 'Muted', displayPath: 'color.muted', role: 'mu', section: 'text' },
  border: { path: 'color.border.default', label: 'Border', displayPath: 'color.border', role: 'bd', section: 'text' },
  ring: { path: 'color.ring.default', label: 'Ring', displayPath: 'color.ring', role: 'rg', section: 'text' },
  success: { path: 'color.success.solid', label: 'Success', displayPath: 'color.success', role: 'ok', section: 'semantic' },
  warning: { path: 'color.warning.solid', label: 'Warning', displayPath: 'color.warning', role: 'wn', section: 'semantic' },
  danger: { path: 'color.danger.solid', label: 'Danger', displayPath: 'color.danger', role: 'dn', section: 'semantic' },
  info: { path: 'color.info.solid', label: 'Info', displayPath: 'color.info', role: 'in', section: 'semantic' },
} as const

export type ColorRoleKey = keyof typeof COLOR_ROLES

export const RAMP_ROLES = [
  { role: 'primary', label: 'Primary' },
  { role: 'accent', label: 'Accent' },
  { role: 'secondary', label: 'Secondary' },
  { role: 'success', label: 'Success' },
  { role: 'warning', label: 'Warning' },
  { role: 'info', label: 'Info' },
  { role: 'danger', label: 'Danger' },
  { role: 'neutral', label: 'Neutral' },
] as const

export const RAMP_STEPS = ['solid', 'hover', 'subtle', 'border', 'text'] as const

const FALLBACKS: Record<ColorRoleKey, string> = {
  primary: '#3b82f6',
  secondary: '#64748b',
  accent: '#a855f7',
  bg: '#f8f7f6',
  surface: '#ffffff',
  fg: '#1c1917',
  muted: '#78716c',
  border: '#e7e5e4',
  ring: '#3b82f6',
  success: '#16a34a',
  warning: '#d97706',
  danger: '#dc2626',
  info: '#2563eb',
}

type ScaleMap = Record<string, Record<string, string>>
type ContrastPair = {
  key: string
  label: string
  ratio: string
  ok: boolean
  soft: boolean
}
type GauntletCell = { key: string; color: string; label: string }
type RampRow = {
  role: string
  label: string
  hex: string
  steps: { key: string; color: string; path: string }[]
}

function colorHex(
  doc: Record<string, unknown> | null,
  path: string,
  mode: 'light' | 'dark',
  fallback: string,
): string {
  const raw = getPathValue(doc, path, mode)
  if (!raw) return fallback
  return tryPickerHex(raw) ?? ''
}

function asRecord(v: unknown): Record<string, unknown> | null {
  if (!v || typeof v !== 'object' || Array.isArray(v)) return null
  return v as Record<string, unknown>
}

export function createColorEdit(session: ThemeSession) {
  const primaryHex = signal(FALLBACKS.primary)
  const secondaryHex = signal(FALLBACKS.secondary)
  const accentHex = signal(FALLBACKS.accent)
  const bgHex = signal(FALLBACKS.bg)
  const surfaceHex = signal(FALLBACKS.surface)
  const fgHex = signal(FALLBACKS.fg)
  const mutedHex = signal(FALLBACKS.muted)
  const borderHex = signal(FALLBACKS.border)
  const ringHex = signal(FALLBACKS.ring)
  const successHex = signal(FALLBACKS.success)
  const warningHex = signal(FALLBACKS.warning)
  const dangerHex = signal(FALLBACKS.danger)
  const infoHex = signal(FALLBACKS.info)

  const hexByKey: Record<ColorRoleKey, ReturnType<typeof signal<string>>> = {
    primary: primaryHex,
    secondary: secondaryHex,
    accent: accentHex,
    bg: bgHex,
    surface: surfaceHex,
    fg: fgHex,
    muted: mutedHex,
    border: borderHex,
    ring: ringHex,
    success: successHex,
    warning: warningHex,
    danger: dangerHex,
    info: infoHex,
  }

  const rampRows = signal<RampRow[]>([])
  const gauntletCells = signal<GauntletCell[]>([])
  const contrastPairs = signal<ContrastPair[]>([])
  const generateEnabled = signal(false)
  const generateBusy = signal(false)
  const showDensity = signal(false)
  const showBreakpoint = signal(false)
  const showMode = signal(false)
  const densityId = signal<DensityId>('default')
  const densityCompactOn = signal(false)
  const densitySpaciousOn = signal(false)
  const densityDefaultOn = signal(true)
  const breakpointId = signal('base')
  const breakpointRows = signal<{ key: string; label: string; on: boolean }[]>([])

  let deriveGen = 0

  const refreshEngine = () => {
    const ds = getDesignState()
    const doc = ds.themeDoc
    const mode = session.writeMode()
    const client = getDesignOpsClient()
    const gen = ++deriveGen
    if (!doc || !client?.sailThemeDeriveScale) {
      rampRows.set(rowsFromAnchors(doc, mode, {}))
      gauntletCells.set(cellsFromScales({}))
      contrastPairs.set([])
      generateEnabled.set(false)
      return
    }
    generateEnabled.set(true)
    void client
      .sailThemeDeriveScale({ theme: doc, mode })
      .then((out) => {
        if (gen !== deriveGen) return
        const scales = asRecord(out.scales) ?? {}
        const mapped: ScaleMap = {}
        for (const [role, steps] of Object.entries(scales)) {
          const stepObj = asRecord(steps)
          if (!stepObj) continue
          mapped[role] = {}
          for (const [step, css] of Object.entries(stepObj)) {
            if (typeof css === 'string') mapped[role]![step] = css
          }
        }
        rampRows.set(rowsFromAnchors(doc, mode, mapped))
        gauntletCells.set(cellsFromScales(mapped))
      })
      .catch(() => {
        if (gen !== deriveGen) return
        rampRows.set(rowsFromAnchors(doc, mode, {}))
        gauntletCells.set(cellsFromScales({}))
      })
    if (client.sailThemeCheckTheme) {
      void client
        .sailThemeCheckTheme({ theme: doc, derive: true, label: ds.themeId || 'theme' })
        .then((out) => {
          if (gen !== deriveGen) return
          contrastPairs.set(pairsFromEngine(out.contrastPairs))
        })
        .catch(() => {
          if (gen !== deriveGen) return
          contrastPairs.set([])
        })
    }
  }

  const syncDimensions = () => {
    const ds = getDesignState()
    const fulfilled = fulfilledDimensions(ds.themeDoc)
    const modes = fulfilled.modes.length
      ? fulfilled.modes
      : (['light', 'dark'] as ('light' | 'dark')[])
    showMode.set(modes.length > 1)
    showDensity.set(fulfilled.density)
    densityId.set(ds.density)
    densityCompactOn.set(ds.density === 'compact')
    densityDefaultOn.set(ds.density === 'default')
    densitySpaciousOn.set(ds.density === 'spacious')
    const bps = fulfilled.breakpoints
    showBreakpoint.set(bps.length > 0)
    if (bps.length && !bps.includes(breakpointId())) {
      breakpointId.set(bps[0]!)
    }
    breakpointRows.set(
      bps.map((id) => ({
        key: id,
        label: id,
        on: breakpointId() === id,
      })),
    )
  }

  const sync = () => {
    const ds = getDesignState()
    const doc = ds.themeDoc
    const mode = session.writeMode()
    for (const key of Object.keys(COLOR_ROLES) as ColorRoleKey[]) {
      const def = COLOR_ROLES[key]
      hexByKey[key].set(colorHex(doc, def.path, mode, FALLBACKS[key]))
    }
    syncDimensions()
    refreshEngine()
  }

  session.onSync(sync)
  sync()

  const onRoleColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const hex = String(detail?.value ?? '').trim()
    if (!hex) return
    const key = detail?.token as ColorRoleKey | null
    if (!key || !(key in COLOR_ROLES)) return
    const def = COLOR_ROLES[key]
    hexByKey[key].set(tryPickerHex(hex) || hex)
    const write = key === 'primary' ? setAnchorColor('primary', hex) : setTokenValue(def.path, hex)
    void write.then(() => session.sync())
  }

  const onAnchorColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const hex = String(detail?.value ?? '').trim()
    const role = String(detail?.token ?? '').trim()
    if (!hex || !role) return
    void setAnchorColor(role, hex).then(() => session.sync())
  }

  const onStepColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const hex = String(detail?.value ?? '').trim()
    const path = String(detail?.token ?? '').trim()
    if (!hex || !path.startsWith('color.')) return
    void setTokenValue(path, hex).then(() => session.sync())
  }

  const brandRows = () =>
    (['primary', 'secondary', 'accent'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      hex: hexByKey[key](),
    }))

  const surfaceRows = () =>
    (['bg', 'surface'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
    }))

  const textRows = () =>
    (['fg', 'muted', 'border', 'ring'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
    }))

  const semanticRows = () =>
    (['success', 'warning', 'danger', 'info'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
      sem: key,
    }))

  const seedDots = () => [
    { key: 's-primary', hex: primaryHex() },
    { key: 's-secondary', hex: secondaryHex() },
    { key: 's-accent', hex: accentHex() },
    { key: 's-bg', hex: bgHex() },
    { key: 's-fg', hex: fgHex() },
    { key: 's-muted', hex: mutedHex() },
    { key: 's-border', hex: borderHex() },
  ]

  const onSeedGenerate = () => {
    if (!session.guardEdit()) return
    if (generateBusy()) return
    generateBusy.set(true)
    void generateTheme()
      .then((ok) => {
        if (ok) {
          const note = getDesignState().statusNote
          session.setNote(note || 'Theme generated.', 'success')
        } else {
          session.setNote(
            getDesignState().lastWriteError || 'Generate refused by the contrast gate.',
            'danger',
          )
        }
        session.sync()
      })
      .finally(() => generateBusy.set(false))
  }

  const onDensity = (id: DensityId) => {
    if (!session.guardEdit()) return
    void setDesignDensity(id).then(() => session.sync())
  }

  const onBreakpoint = (id: string) => {
    breakpointId.set(id)
    syncDimensions()
  }

  const onPreviewMode = (mode: 'light' | 'dark') => {
    setDesignMode(mode)
    session.sync()
  }

  return {
    primaryHex,
    secondaryHex,
    accentHex,
    bgHex,
    surfaceHex,
    fgHex,
    mutedHex,
    borderHex,
    ringHex,
    successHex,
    warningHex,
    dangerHex,
    infoHex,
    rampRows,
    gauntletCells,
    contrastPairs,
    generateEnabled,
    generateBusy,
    showDensity,
    showBreakpoint,
    showMode,
    densityCompactOn,
    densityDefaultOn,
    densitySpaciousOn,
    breakpointRows,
    brandRows,
    surfaceRows,
    textRows,
    semanticRows,
    seedDots,
    onRoleColor,
    onAnchorColor,
    onStepColor,
    onSeedGenerate,
    onDensity,
    onBreakpoint,
    onPreviewMode,
  }
}

function rowsFromAnchors(
  doc: Record<string, unknown> | null,
  mode: 'light' | 'dark',
  scales: ScaleMap,
): RampRow[] {
  const anchors = anchorsForMode(doc, mode)
  return RAMP_ROLES.map(({ role, label }) => {
    const raw = anchors[role] ?? scales[role]?.solid ?? ''
    const hex = raw ? tryPickerHex(raw) || raw : ''
    const steps = RAMP_STEPS.map((step) => ({
      key: step,
      color: scales[role]?.[step] ?? '',
      path: `color.${role}.${step}`,
    }))
    return { role, label, hex, steps }
  }).filter((row) => row.hex || row.steps.some((s) => s.color))
}

function cellsFromScales(scales: ScaleMap): GauntletCell[] {
  const out: GauntletCell[] = []
  for (const { role } of RAMP_ROLES) {
    const steps = scales[role]
    if (!steps) continue
    for (const step of RAMP_STEPS) {
      const color = steps[step]
      if (!color) continue
      out.push({
        key: `${role}-${step}`,
        color,
        label: `${role} ${step}`,
      })
    }
  }
  return out
}

function pairsFromEngine(raw: unknown): ContrastPair[] {
  const arr = Array.isArray(raw) ? raw : []
  const out: ContrastPair[] = []
  for (const item of arr) {
    const obj = asRecord(item)
    if (!obj) continue
    const fg = typeof obj.fg === 'string' ? obj.fg : ''
    const bg = typeof obj.bg === 'string' ? obj.bg : ''
    if (!fg || !bg) continue
    const ratioNum = typeof obj.ratio === 'number' ? obj.ratio : Number(obj.ratio)
    const ratio = Number.isFinite(ratioNum) ? `${ratioNum.toFixed(2)}:1` : '—'
    out.push({
      key: `${obj.mode ?? 'mode'}-${fg}-${bg}`,
      label: `${fg} on ${bg}`,
      ratio,
      ok: obj.ok === true,
      soft: obj.soft === true,
    })
  }
  return out
}
