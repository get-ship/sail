/**
 * Design surface store — theme workbench state + actions.
 */
import {
  clearTokenValueOn,
  overrideComponentRadiusOn,
  previewInAppOn,
  refreshBlastAggregateOn,
  refreshBlastOn,
  repaintSpecimenOn,
  setBrandColorOn,
  setDesignDensityOn,
  setIconScaleStepOn,
  setRadiusStepOn,
  setTokenValueOn,
  setTypeRoleOn,
  type EditCtx,
} from './edit-actions'
import { defaultIconFamily, defaultRadiusFamily } from './scale'
import { makeStoreUi } from './store-ui'
import {
  countOverrides,
  densityOf,
  diffThemeDocs,
  extractIconScale,
  extractRadius,
  extractTypeRoles,
  radiusInheritance,
} from './theme-doc'
import {
  emptyDesignState,
  forkThemeOn,
  loadThemesOn,
  openThemeOn,
  saveThemeOn,
  setDesignProjectRootOn,
  type ThemeIoCtx,
} from './theme-io'
import { DEFAULT_GROUP_ID } from './token-groups'
import type {
  DecisionId,
  DensityId,
  DesignOpsClient,
  DesignSurfaceState,
  TypeRoles,
} from './types'
import { DECISION_IDS } from './types'

type Listener = (s: DesignSurfaceState) => void

let client: DesignOpsClient | null = null
let state: DesignSurfaceState = bootState()
let savedDoc: Record<string, unknown> | null = null
const listeners = new Set<Listener>()
let previewRoot: HTMLElement | null = null

function bootState(): DesignSurfaceState {
  const s = emptyDesignState()
  return {
    ...s,
    radius: defaultRadiusFamily(true),
    iconScale: defaultIconFamily(true),
    inheritance: radiusInheritance(null),
    groupId: DEFAULT_GROUP_ID,
    statusNote: 'Pick a decision or token group',
  }
}

function emit(): void {
  for (const fn of listeners) fn(state)
}

function recomputeDerived(doc: Record<string, unknown> | null): void {
  const mode = state.modeId
  const radius = extractRadius(doc, mode)
  radius.scaleTogether = state.radius.scaleTogether
  const iconScale = extractIconScale(doc, mode)
  iconScale.scaleTogether = state.iconScale.scaleTogether
  state = {
    ...state,
    themeDoc: doc,
    density: densityOf(doc),
    radius,
    iconScale,
    inheritance: radiusInheritance(doc, mode),
    typeRoles: extractTypeRoles(doc, mode),
    overrideCount: countOverrides(doc, savedDoc),
    diffRows: diffThemeDocs(savedDoc, doc),
    dirty: !!doc && JSON.stringify(doc) !== JSON.stringify(savedDoc),
  }
}

function themeCtx(): ThemeIoCtx {
  return {
    getClient: () => client,
    getState: () => state,
    setState: (s) => {
      state = s
    },
    getSavedDoc: () => savedDoc,
    setSavedDoc: (d) => {
      savedDoc = d
    },
    recomputeDerived,
    emit,
    repaintSpecimen: () => repaintSpecimen(),
    refreshBlast: (t) => refreshBlast(t),
  }
}

function editCtx(): EditCtx {
  return {
    getClient: () => client,
    getState: () => state,
    setState: (s) => {
      state = s
    },
    recomputeDerived,
    emit,
    repaintSpecimen: () => repaintSpecimen(),
    refreshBlast: (t) => refreshBlast(t),
    reflectCss,
  }
}

function reflectCss(cssName: string, value: string | null): void {
  if (!previewRoot) return
  if (value == null) previewRoot.style.removeProperty(`--${cssName}`)
  else previewRoot.style.setProperty(`--${cssName}`, value)
}

const ui = makeStoreUi({
  getState: () => state,
  getSaved: () => savedDoc,
  getClient: () => client,
  setState: (s) => {
    state = s
  },
  emit,
  setTypeRole: (role, family) => setTypeRole(role, family),
})

export function getDesignState(): DesignSurfaceState {
  return state
}

export function subscribeDesign(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

export function setDesignOpsClient(c: DesignOpsClient | null): void {
  client = c
}

export function getDesignOpsClient(): DesignOpsClient | null {
  return client
}

export function setDesignProjectRoot(root: string): void {
  setDesignProjectRootOn(themeCtx(), root)
}

export function setDesignMode(modeId: 'light' | 'dark'): void {
  if (state.modeId === modeId) return
  state = { ...state, modeId }
  // Re-extract scale families + type roles for the active mode.
  recomputeDerived(state.themeDoc)
  emit()
  void repaintSpecimen()
}

export function setTokenGroup(id: string): void {
  const decisionId = (DECISION_IDS as string[]).includes(id)
    ? (id as DecisionId)
    : state.decisionId
  state = { ...state, groupId: id, decisionId }
  emit()
  if (id === 'radius') void refreshBlast('radius-lg')
  if (id === 'color') void refreshBlast('primary')
}

export function setDecision(id: DecisionId): void {
  state = { ...state, decisionId: id, groupId: id }
  emit()
  if (id === 'radius') void refreshBlast('radius-lg')
  if (id === 'color') void refreshBlast('primary')
  if (id === 'icon') void refreshBlast('icon-md')
}

export function setSaveReviewOpen(open: boolean): void {
  state = { ...state, saveReviewOpen: open }
  emit()
  if (open) {
    void refreshBlastForDirty()
  }
}

/** Map dirty DTCG paths → usage tokens for blast aggregation. */
export function blastTokensFromDiffs(
  rows: { path: string }[],
): string[] {
  const tokens = new Set<string>()
  for (const r of rows) {
    const p = r.path
    if (p.includes('radius')) tokens.add('radius-lg')
    else if (p.includes('icon')) tokens.add('icon-md')
    else if (p.includes('primary') || p.includes('color.')) tokens.add('primary')
    else if (p.includes('typography') || p.includes('font')) tokens.add('font-body')
    else if (p.includes('density')) tokens.add('space-4')
  }
  if (tokens.size === 0) tokens.add('primary')
  return [...tokens]
}

export async function refreshBlastForDirty(): Promise<void> {
  const tokens = blastTokensFromDiffs(state.diffRows)
  await refreshBlastAggregate(tokens, state.diffRows.length)
}

export function setScaleTogether(on: boolean): void {
  state = { ...state, radius: { ...state.radius, scaleTogether: on } }
  emit()
}

export function setTokenSearch(q: string): void {
  state = { ...state, tokenSearch: q }
  emit()
}

export function setOverridesOnly(on: boolean): void {
  state = { ...state, overridesOnly: on }
  emit()
}

export function setIconQuery(q: string): void {
  state = { ...state, icons: { ...state.icons, query: q } }
  emit()
}

export function setIconPack(pack: string): void {
  state = { ...state, icons: { ...state.icons, pack } }
  emit()
}

export function setInspectOpen(open: boolean): void {
  state = { ...state, inspectOpen: open }
  emit()
}

export function setIconScaleTogether(on: boolean): void {
  state = { ...state, iconScale: { ...state.iconScale, scaleTogether: on } }
  emit()
}

/** Specimen panel root — live CSS var paint (never documentElement). */
export function setSpecimenPreviewRoot(el: HTMLElement | null): void {
  previewRoot = el
  if (!el) return
  for (const step of state.radius.steps) {
    el.style.setProperty(`--radius-${step.key}`, step.raw)
  }
  const lg = state.radius.steps.find((s) => s.key === 'lg')
  if (lg) el.style.setProperty('--radius', lg.raw)
  for (const step of state.iconScale.steps) {
    el.style.setProperty(`--icon-size-${step.key}`, step.raw)
  }
}

export async function loadThemes(opts?: { themeId?: string }): Promise<void> {
  await loadThemesOn(themeCtx(), opts)
}

export async function openTheme(themeId: string): Promise<void> {
  await openThemeOn(themeCtx(), themeId)
}

export async function forkTheme(newId: string, sourceId?: string): Promise<boolean> {
  return forkThemeOn(themeCtx(), newId, sourceId)
}

export async function saveTheme(): Promise<boolean> {
  return saveThemeOn(themeCtx())
}

export async function setDesignDensity(density: DensityId): Promise<void> {
  await setDesignDensityOn(editCtx(), density)
}

export async function setRadiusStep(key: string, px: number): Promise<void> {
  await setRadiusStepOn(editCtx(), key, px)
}

export async function overrideComponentRadius(
  component: string,
  value: string | null,
): Promise<void> {
  await overrideComponentRadiusOn(editCtx(), component, value)
}

export async function resetInheritance(nodeId: string): Promise<void> {
  const node = state.inheritance.find((n) => n.id === nodeId)
  if (!node?.canReset) return
  await overrideComponentRadius(nodeId, null)
}

export async function setComponentOverride(
  nodeId: string,
  value: string,
): Promise<void> {
  await overrideComponentRadius(nodeId, value)
}

export async function setTypeRole(
  role: keyof TypeRoles,
  family: string,
): Promise<void> {
  await setTypeRoleOn(editCtx(), role, family)
}

export async function setTokenValue(path: string, value: string): Promise<void> {
  await setTokenValueOn(editCtx(), path, value)
}

export async function setBrandColor(hexOrOklch: string): Promise<void> {
  await setBrandColorOn(editCtx(), hexOrOklch)
}

export async function clearTokenValue(path: string): Promise<void> {
  await clearTokenValueOn(editCtx(), path)
}

/** Save with optional blast-radius review gate when dirty. */
export async function requestSaveTheme(): Promise<boolean> {
  if (!state.dirty) return saveTheme()
  if (!state.saveReviewOpen) {
    setSaveReviewOpen(true)
    return false
  }
  const ok = await saveTheme()
  if (ok) state = { ...state, saveReviewOpen: false }
  return ok
}

export const searchFonts = ui.searchFonts
export const addFontFamily = ui.addFontFamily
export const loadIconGlyphs = ui.loadIconGlyphs
export const filteredIconGlyphs = ui.filteredIconGlyphs
export const exportDiffText = ui.exportDiffText
export const uiGroups = ui.uiGroups
export const uiActiveGroup = ui.uiActiveGroup
export const uiGroupTokens = ui.uiGroupTokens
export const uiRadiusSteps = ui.uiRadiusSteps
export const uiInheritance = ui.uiInheritance
export const uiIconScaleSteps = ui.uiIconScaleSteps
export const uiThemeOptions = ui.uiThemeOptions
export const uiHeader = ui.uiHeader
export const uiDecisions = ui.uiDecisions
export const uiBrand = ui.uiBrand
export const uiDensityLabel = ui.uiDensityLabel
export const uiTokenRows = ui.uiTokenRows

export async function refreshBlast(token: string): Promise<void> {
  await refreshBlastOn(editCtx(), token)
}

export async function refreshBlastAggregate(
  tokens: string[],
  dirtyCount = 0,
): Promise<void> {
  await refreshBlastAggregateOn(editCtx(), tokens, dirtyCount)
}

export function previewInApp(): {
  ok: boolean
  patchKeys: string[]
  message: string
} {
  return previewInAppOn(editCtx())
}

export async function setIconScaleStep(key: string, px: number): Promise<void> {
  await setIconScaleStepOn(editCtx(), key, px)
}

export async function repaintSpecimen(): Promise<void> {
  await repaintSpecimenOn(editCtx())
}

export function __resetDesignForTests(): void {
  state = bootState()
  savedDoc = null
  client = null
  previewRoot = null
}

export function __setWorkingDocForTests(doc: Record<string, unknown> | null): void {
  savedDoc = doc ? structuredClone(doc) : null
  recomputeDerived(doc)
  emit()
}

/** Test-only: project theme path so saveTheme skips fork. */
export function __setThemePathForTests(path: string | null): void {
  state = { ...state, themePath: path }
  emit()
}

export function __getSavedDocForTests(): Record<string, unknown> | null {
  return savedDoc
}
