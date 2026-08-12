import type { SailHost } from '../vendor/runtime'
/**
 * Theme domain store — product UI is Themes only.
 * (Legacy path name `design`; writers shared by theme-session facets.)
 *
 * Draft model: one themeDoc in RAM; dirty ⇔ doc ≠ baseline (savedDoc);
 * Save = JSON write + SailTheme Generate mode=app. Live preview via
 * theme-session/theme-draft-paint (applyScope when available).
 */
import {
  clearTokenValueOn,
  formatUsedByLabel,
  overrideComponentRadiusOn,
  previewInAppOn,
  refreshBlastAggregateOn,
  repaintSpecimenOn,
  setBrandColorOn,
  setDesignDensityOn,
  setIconScaleStepOn,
  setRadiusStepOn,
  setRadiusFamilyOn,
  setTokenValueOn,
  setTypeRoleOn,
  type EditCtx,
} from './edit-actions'

export { formatUsedByLabel }
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
  deleteThemeOn,
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

/** Lazy import to avoid circular design ↔ theme-session at module init. */
function scheduleDraftPaint(): void {
  void import('../compositions/theme-session/theme-draft-paint')
    .then((m) => m.scheduleDraftPaint())
    .catch(() => {
      /* theme-session optional in pure design tests */
    })
}

type Listener = (s: DesignSurfaceState) => void

let client: DesignOpsClient | null = null
let state: DesignSurfaceState = bootState()
let savedDoc: Record<string, unknown> | null = null
const listeners = new Set<Listener>()
let previewRoot: SailHost | null = null
/** Monotonic gen for usage refresh — stale async settles must not overwrite. */
let blastGen = 0

/** Undo/redo stacks — JSON snapshots of themeDoc (project theme edits only). */
const historyPast: string[] = []
const historyFuture: string[] = []
const HISTORY_MAX = 40
let lastHistorySnap = ''

function snapshotDoc(doc: Record<string, unknown> | null): string {
  return doc ? JSON.stringify(doc) : ''
}

function pushHistoryBeforeWrite(): void {
  const snap = snapshotDoc(state.themeDoc)
  if (!snap || snap === lastHistorySnap) return
  historyPast.push(snap)
  if (historyPast.length > HISTORY_MAX) historyPast.shift()
  historyFuture.length = 0
  lastHistorySnap = snap
  state = {
    ...state,
    canUndo: historyPast.length > 0,
    canRedo: historyFuture.length > 0,
  }
}

function restoreSnapshot(snap: string): void {
  const doc = snap ? (JSON.parse(snap) as Record<string, unknown>) : null
  recomputeDerived(doc)
  lastHistorySnap = snap
  state = {
    ...state,
    canUndo: historyPast.length > 0,
    canRedo: historyFuture.length > 0,
  }
  emit()
}

function beginBlastRefresh(): () => boolean {
  const gen = ++blastGen
  return () => gen === blastGen
}

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
  scheduleDraftPaint()
}

function clearHistory(): void {
  historyPast.length = 0
  historyFuture.length = 0
  lastHistorySnap = ''
  state = {
    ...state,
    canUndo: false,
    canRedo: false,
  }
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

// API stability shim; live draft paint is scheduleDraftPaint / applyScope.
function reflectCss(_cssName: string, _value: string | null): void {
  /* no-op — paint via scheduleDraftPaint / applyScope */
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

/**
 * Drop unsaved draft edits — restore baseline (savedDoc), clear history.
 * Disk untouched. Used by dirty leave Discard.
 */
export function discardThemeEdits(): void {
  if (!savedDoc) {
    // No baseline — clear working doc.
    clearHistory()
    recomputeDerived(null)
    state = {
      ...state,
      dirty: false,
      saveReviewOpen: false,
      diffRows: [],
      statusNote: 'Discarded unsaved changes',
    }
    emit()
    return
  }
  clearHistory()
  const baseline = structuredClone(savedDoc)
  recomputeDerived(baseline)
  lastHistorySnap = snapshotDoc(baseline)
  state = {
    ...state,
    dirty: false,
    saveReviewOpen: false,
    statusNote: 'Discarded unsaved changes',
  }
  emit()
}

/**
 * Free in-memory draft after clean leave / delete.
 * Teardown draft paint hosts is session's job (theme-draft-paint).
 */
export function closeThemeSession(): void {
  clearHistory()
  savedDoc = null
  state = {
    ...state,
    themeId: '',
    themePath: null,
    themeDoc: null,
    forkedFrom: null,
    dirty: false,
    saveReviewOpen: false,
    diffRows: [],
    lastWriteError: null,
    statusNote: 'Theme closed',
  }
  emit()
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

export async function setSaveReviewOpen(open: boolean): Promise<void> {
  state = { ...state, saveReviewOpen: open }
  emit()
  if (open) {
    await refreshBlastForDirty()
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

/** Register the specimen panel root element (no CSSOM paint). */
export function setSpecimenPreviewRoot(el: SailHost | null): void {
  previewRoot = el
}

export async function loadThemes(opts?: { themeId?: string }): Promise<void> {
  await loadThemesOn(themeCtx(), opts)
}

export async function openTheme(themeId: string): Promise<void> {
  // One open theme: clear undo stack; baseline set by openThemeOn → savedDoc.
  clearHistory()
  await openThemeOn(themeCtx(), themeId)
  lastHistorySnap = snapshotDoc(state.themeDoc)
  state = {
    ...state,
    canUndo: false,
    canRedo: false,
    saveReviewOpen: false,
  }
  emit()
}

/**
 * Add catalog/source theme into the project (UI: "Add to project").
 * `forkTheme` kept as alias for existing call sites.
 */
export async function addThemeToProject(
  newId: string,
  sourceId?: string,
): Promise<boolean> {
  return forkThemeOn(themeCtx(), newId, sourceId)
}

export async function forkTheme(newId: string, sourceId?: string): Promise<boolean> {
  return addThemeToProject(newId, sourceId)
}

export async function saveTheme(): Promise<boolean> {
  const ok = await saveThemeOn(themeCtx())
  if (ok) {
    // Baseline matches draft; history can stay for undo of pre-save edits or clear.
    lastHistorySnap = snapshotDoc(state.themeDoc)
  }
  return ok
}

/** Delete a project theme file (catalog refused). Reloads theme list. */
export async function deleteTheme(themeId: string): Promise<boolean> {
  return deleteThemeOn(themeCtx(), themeId)
}

export async function setDesignDensity(density: DensityId): Promise<void> {
  await setDesignDensityOn(editCtx(), density)
}

export async function setRadiusStep(key: string, px: number): Promise<void> {
  pushHistoryBeforeWrite()
  await setRadiusStepOn(editCtx(), key, px)
}

export async function setRadiusFamily(
  steps: { key: string; px: number }[],
  scaleTogether = true,
): Promise<void> {
  pushHistoryBeforeWrite()
  await setRadiusFamilyOn(editCtx(), steps, scaleTogether)
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
  pushHistoryBeforeWrite()
  await setTokenValueOn(editCtx(), path, value)
}

export async function setBrandColor(hexOrOklch: string): Promise<void> {
  pushHistoryBeforeWrite()
  await setBrandColorOn(editCtx(), hexOrOklch)
}

export function undoThemeEdit(): boolean {
  if (!historyPast.length || !state.themeDoc) return false
  const cur = snapshotDoc(state.themeDoc)
  const prev = historyPast.pop()!
  historyFuture.push(cur)
  restoreSnapshot(prev)
  return true
}

export function redoThemeEdit(): boolean {
  if (!historyFuture.length) return false
  const cur = snapshotDoc(state.themeDoc)
  const next = historyFuture.pop()!
  if (cur) historyPast.push(cur)
  restoreSnapshot(next)
  return true
}

/** Leave-dialog Save: write without the Used-by review gate. */
export async function saveThemeForLeave(): Promise<boolean> {
  if (state.saveReviewOpen) {
    state = { ...state, saveReviewOpen: false }
  }
  return saveTheme()
}

export function canUndoTheme(): boolean {
  return historyPast.length > 0
}

export function canRedoTheme(): boolean {
  return historyFuture.length > 0
}

export async function clearTokenValue(path: string): Promise<void> {
  await clearTokenValueOn(editCtx(), path)
}

/** Save with optional Used-by review gate when dirty. */
export async function requestSaveTheme(): Promise<boolean> {
  if (!state.dirty) return saveTheme()
  if (!state.saveReviewOpen) {
    // Await aggregate so callers (toast / chrome) see save-review labels, not stale token labels.
    await setSaveReviewOpen(true)
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
  const isCurrent = beginBlastRefresh()
  await refreshBlastAggregateOn(editCtx(), [token], 0, isCurrent)
}

export async function refreshBlastAggregate(
  tokens: string[],
  dirtyCount = 0,
): Promise<void> {
  const isCurrent = beginBlastRefresh()
  await refreshBlastAggregateOn(editCtx(), tokens, dirtyCount, isCurrent)
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
  blastGen = 0
  historyPast.length = 0
  historyFuture.length = 0
  lastHistorySnap = ''
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
