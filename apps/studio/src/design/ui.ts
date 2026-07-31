/**
 * Design surface view-models — pure projections of store state.
 */
import {
  colorSummary,
  getKnob,
  groupKnobs,
  humanLabel,
  isScaleFamilyGroup,
  listTokenGroups,
  type TokenKnob,
} from './token-groups'
import { getPathValue, listTokenRows } from './theme-doc'
import type { DesignDiffRow, DesignSurfaceState, InheritNode } from './types'

export type GroupNavRow = {
  id: string
  label: string
  count: string
  active: boolean
  key: string
}

export type TokenRowUi = {
  path: string
  label: string
  cssName: string
  value: string
  display: string
  dirty: boolean
  isColor: boolean
  isDim: boolean
  isText: boolean
  num: number
  unit: string
  min: number
  max: number
  step: number
  key: string
  /** Present when search spans all groups. */
  groupLabel: string
  groupId: string
}

function matchesQuery(k: TokenKnob, q: string): boolean {
  if (!q) return true
  return (
    k.path.toLowerCase().includes(q) ||
    k.label.toLowerCase().includes(q) ||
    k.cssName.toLowerCase().includes(q) ||
    k.base.toLowerCase().includes(q)
  )
}

function pathGroupId(path: string): string {
  return path.split('.')[0] || 'other'
}

/** Human-readable value for UI (colors → summary, never raw oklch primary). */
export function displayValue(value: string, path: string, cssName?: string): string {
  const knob = getKnob(path)
  const name = cssName ?? knob?.cssName ?? path
  const isColor =
    knob?.kind === 'color' ||
    value.startsWith('oklch') ||
    value.startsWith('#') ||
    value.startsWith('rgb') ||
    value.startsWith('color-mix')
  if (isColor) return colorSummary(value, name)
  if (!value) return '—'
  return value
}

export function displayPathLabel(path: string): string {
  const knob = getKnob(path)
  if (knob?.label) return knob.label
  if (path === '$extensions.ship.density') return 'Density'
  if (path.startsWith('$extensions.')) return humanPath(path.replace(/^\$extensions\./, ''))
  return humanLabel(path) || humanPath(path)
}

/** One human line for Changes panel + clipboard export. */
export function formatDiffLine(row: DesignDiffRow): string {
  const label = displayPathLabel(row.path)
  const from = displayValue(row.from, row.path)
  const to = displayValue(row.to, row.path)
  return `${label}: ${from} → ${to}`
}

export function uiTokenGroups(state: DesignSurfaceState): GroupNavRow[] {
  const q = state.tokenSearch.trim().toLowerCase()
  return listTokenGroups().map((g) => {
    let count = g.knobs.length
    if (q) {
      count = g.knobs.filter((k) => matchesQuery(k, q)).length
    }
    return {
      id: g.id,
      label: g.label,
      count: String(count),
      active: state.groupId === g.id,
      key: g.id,
    }
  })
}

function currentValue(
  knob: TokenKnob,
  state: DesignSurfaceState,
  saved: Record<string, unknown> | null,
): { value: string; dirty: boolean } {
  const fromDoc = getPathValue(state.themeDoc, knob.path)
  const fromSaved = getPathValue(saved, knob.path)
  const value = fromDoc ?? knob.base
  const dirty =
    fromDoc != null && fromSaved != null
      ? fromDoc !== fromSaved
      : fromDoc != null && fromDoc !== knob.base
  return { value, dirty }
}

function knobToRow(
  k: TokenKnob,
  state: DesignSurfaceState,
  saved: Record<string, unknown> | null,
  groupLabel: string,
  groupId: string,
): TokenRowUi | null {
  const { value, dirty } = currentValue(k, state, saved)
  if (state.overridesOnly && !dirty) return null
  const dim = (() => {
    const m = String(value).match(/^(-?\d+(?:\.\d+)?)/)
    return m ? Number(m[1]) : 0
  })()
  const isColor = k.kind === 'color'
  const isDim =
    k.kind === 'dimension' || k.kind === 'duration' || k.kind === 'number'
  return {
    path: k.path,
    label: k.label,
    cssName: k.cssName,
    value,
    display: isColor ? colorSummary(value, k.cssName) : value,
    dirty,
    isColor,
    isDim,
    isText: !isColor && !isDim,
    num: dim,
    unit: k.unit || '',
    min: k.min ?? 0,
    max: k.max ?? 64,
    step: k.step ?? 1,
    key: k.path,
    groupLabel,
    groupId,
  }
}

/**
 * Token rows for the editor.
 * Empty search → active group only.
 * Non-empty search → flat cross-group results (filter everything).
 */
export function uiGroupTokenRows(
  state: DesignSurfaceState,
  saved: Record<string, unknown> | null,
): TokenRowUi[] {
  const q = state.tokenSearch.trim().toLowerCase()
  const groups = listTokenGroups()
  const rows: TokenRowUi[] = []

  if (q) {
    for (const g of groups) {
      for (const k of g.knobs) {
        if (!matchesQuery(k, q)) continue
        const row = knobToRow(k, state, saved, g.label, g.id)
        if (row) rows.push(row)
      }
    }
    return rows
  }

  const active = groups.find((g) => g.id === state.groupId)
  const label = active?.label ?? state.groupId
  for (const k of groupKnobs(state.groupId)) {
    const row = knobToRow(k, state, saved, label, state.groupId)
    if (row) rows.push(row)
  }
  return rows
}

/** First group id with search hits (for auto-nav when needed). */
export function firstGroupWithSearchHits(state: DesignSurfaceState): string | null {
  const q = state.tokenSearch.trim().toLowerCase()
  if (!q) return null
  for (const g of listTokenGroups()) {
    if (g.knobs.some((k) => matchesQuery(k, q))) return g.id
  }
  return null
}

/** Advanced all-tokens list (search + overrides-only) with human labels. */
export function uiTokenRowsLabeled(
  state: DesignSurfaceState,
  saved: Record<string, unknown> | null,
): TokenRowUi[] {
  return listTokenRows(state.themeDoc, saved, {
    search: state.tokenSearch,
    overridesOnly: state.overridesOnly,
  }).map((r) => {
    const knob = getKnob(r.path)
    const isColor =
      knob?.kind === 'color' ||
      r.value.startsWith('oklch') ||
      r.value.startsWith('#') ||
      r.value.startsWith('rgb')
    const gid = pathGroupId(r.path)
    const gLabel =
      listTokenGroups().find((g) => g.id === gid)?.label ?? gid
    return {
      path: r.path,
      label: knob?.label ?? humanPath(r.path),
      cssName: knob?.cssName ?? r.path,
      value: r.value,
      display: displayValue(r.value, r.path, knob?.cssName),
      dirty: r.dirty,
      isColor,
      isDim: knob?.kind === 'dimension' || knob?.kind === 'duration',
      isText: !isColor && knob?.kind !== 'dimension' && knob?.kind !== 'duration',
      num: 0,
      unit: knob?.unit ?? '',
      min: knob?.min ?? 0,
      max: knob?.max ?? 64,
      step: knob?.step ?? 1,
      key: r.path,
      groupLabel: gLabel,
      groupId: gid,
    }
  })
}

function humanPath(path: string): string {
  return path
    .split('.')
    .filter((p) => p && p !== 'default' && !p.startsWith('$'))
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' · ')
}

export function uiRadiusSteps(state: DesignSurfaceState): {
  key: string
  valuePx: number
  raw: string
  label: string
}[] {
  return state.radius.steps.map((s) => ({
    key: s.key,
    valuePx: s.valuePx,
    raw: s.raw,
    label: s.key,
  }))
}

export function uiIconScaleSteps(state: DesignSurfaceState): {
  key: string
  valuePx: number
  raw: string
  label: string
}[] {
  return state.iconScale.steps.map((s) => ({
    key: s.key,
    valuePx: s.valuePx,
    raw: s.raw,
    label: s.key,
  }))
}

export function uiInheritance(state: DesignSurfaceState): InheritNode[] {
  return state.inheritance
}

export function uiThemeOptions(state: DesignSurfaceState): {
  id: string
  label: string
  catalog: boolean
}[] {
  const seen = new Set<string>()
  const out: { id: string; label: string; catalog: boolean }[] = []
  for (const t of [...state.projectThemes, ...state.catalogThemes]) {
    if (seen.has(t.id)) continue
    seen.add(t.id)
    out.push({
      id: t.id,
      label: t.catalog ? t.id : `${t.id} (yours)`,
      catalog: t.catalog,
    })
  }
  return out
}

export function uiHeader(state: DesignSurfaceState): {
  themeLabel: string
  forkLabel: string
  overridesLabel: string
  canSave: boolean
} {
  const forkLabel = state.forkedFrom
    ? `Based on ${state.forkedFrom}`
    : state.themePath
      ? 'Project theme'
      : 'Catalog'
  return {
    themeLabel: state.themeId,
    forkLabel,
    overridesLabel: `${state.overrideCount} change${state.overrideCount === 1 ? '' : 's'}`,
    canSave: !!state.themeDoc,
  }
}

export function uiActiveGroupMeta(state: DesignSurfaceState): {
  id: string
  label: string
  isScaleFamily: boolean
  isRadius: boolean
  isIcon: boolean
  isType: boolean
  isDensity: boolean
  isColor: boolean
} {
  const g = listTokenGroups().find((x) => x.id === state.groupId)
  const id = g?.id ?? state.groupId
  return {
    id,
    label: g?.label ?? id,
    isScaleFamily: isScaleFamilyGroup(id),
    isRadius: id === 'radius',
    isIcon: id === 'icon',
    isType: id === 'typography',
    isDensity: id === 'density',
    isColor: id === 'color',
  }
}
