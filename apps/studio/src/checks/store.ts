import { postStudioMessage, getShellState, selectNode, setBadges } from '@ship/sail-studio-shell'
import {
  autoFixOnSave, bothViewportsRan, countSeverities, enrichNodeIds,
  filterFindings, fileOptions, nodeMapFromModelDiagnostics,
  parseCheckFindings, ruleOptions, sourceSnippet, statusNoteFor,
  toFindingRows, applyFixToSource,
} from './findings'
import { flattenRuleGroups, groupByRule, groupBySeverity } from './groups'
import {
  applyAcceptedReviews, buildFixReviewItems, countAccepted,
  isMachineFixable, machineFixableFindings,
} from './review'
import { commitAppliedSource, commitAppliedSources } from './commit'
import { defaultEditorSetting, editorDeepLink, parseEditorSetting } from './deeplink'
import type {
  AutoFixMode, ChecksOpsClient, ChecksSurfaceState,
  EditorSetting, Finding, FindingRow, FixAttribution,
} from './types'
import { APPLY_FIX_READY } from './types'
import { userError } from '../user-error'

type Listener = (s: ChecksSurfaceState) => void

let client: ChecksOpsClient | null = null
let state: ChecksSurfaceState = emptyState()
const listeners = new Set<Listener>()
/** Path last requested via loadFindings — used by recheck / apply re-run. */
let lastLoadPath: string | null = null
let lastLoadSource: string | undefined

function emptyState(): ChecksSurfaceState {
  return {
    projectRoot: '',
    findings: [],
    selectedId: null,
    ruleFilter: null,
    fileFilter: null,
    autoFixMode: 'off',
    editor: defaultEditorSetting('vscode'),
    layoutBothViewportsRan: false,
    errorCount: 0,
    warningCount: 0,
    statusNote: 'All clear',
    loading: false,
    error: null,
    sourceByPath: new Map(),
    autoFixAttributed: 'project setting',
    reviewOpen: false,
    reviewItems: [],
    reviewNote: '',
    lastRunAt: null,
    lastCheckedPath: null,
    lastLayoutChecked: false,
    attributions: [],
    collapsedRules: [],
  }
}

function emit(): void {
  for (const fn of listeners) fn(state)
}

function recomputeCounts(findings: Finding[]): void {
  const { errorCount, warningCount } = countSeverities(findings)
  state = {
    ...state,
    findings,
    errorCount,
    warningCount,
    statusNote: statusNoteFor(errorCount, warningCount),
  }
  setBadges(errorCount, getShellState().tasksBadge)
}

function patch(p: Partial<ChecksSurfaceState>): void {
  state = { ...state, ...p }
  emit()
}

export function getChecksState(): ChecksSurfaceState {
  return state
}

export function subscribeChecks(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

export function setChecksOpsClient(c: ChecksOpsClient | null): void {
  client = c
}

export function getChecksOpsClient(): ChecksOpsClient | null {
  return client
}

export function setProjectRoot(root: string): void {
  patch({ projectRoot: root })
}

export function setEditorSetting(raw: unknown): void {
  patch({ editor: parseEditorSetting(raw) })
}

export function setEditor(setting: EditorSetting): void {
  patch({ editor: setting })
}

export function setAutoFixMode(mode: AutoFixMode, attributed = 'project setting'): void {
  patch({ autoFixMode: mode, autoFixAttributed: attributed })
}

export function setRuleFilter(rule: string | null): void {
  patch({ ruleFilter: rule || null })
}

export function setFileFilter(file: string | null): void {
  patch({ fileFilter: file || null })
}

export function selectFinding(id: string | null): void {
  patch({ selectedId: id })
}

export function toggleRuleCollapsed(rule: string): void {
  const set = new Set(state.collapsedRules)
  if (set.has(rule)) set.delete(rule)
  else set.add(rule)
  patch({ collapsedRules: [...set] })
}

export function setRuleCollapsed(rule: string, collapsed: boolean): void {
  const set = new Set(state.collapsedRules)
  if (collapsed) set.add(rule)
  else set.delete(rule)
  patch({ collapsedRules: [...set] })
}

function pushAttributions(entries: FixAttribution[]): void {
  if (!entries.length) return
  state = {
    ...state,
    attributions: [...entries, ...state.attributions].slice(0, 100),
  }
}

function attributionFor(
  f: Finding,
  before: string,
  after: string,
  at = Date.now(),
): FixAttribution {
  return {
    id: `${f.id}|${at}`,
    rule: f.rule,
    path: f.path,
    findingId: f.id,
    before,
    after,
    at,
  }
}

export async function loadFindings(
  path: string,
  opts?: { source?: string },
): Promise<Finding[]> {
  lastLoadPath = path
  lastLoadSource = opts?.source
  if (!client) {
    // No ops client yet — empty findings until chrome boots the client.
    patch({ error: null, loading: false })
    return []
  }
  patch({ loading: true, error: null })
  try {
    const args = opts?.source ? { path, source: opts.source } : { path }
    const chk = await client.sailAuthoringCheck(args)
    let bothVp = false
    let layoutRan = false
    if (client.sailAuthoringLayoutCheck) {
      try {
        const layout = await client.sailAuthoringLayoutCheck(args)
        bothVp = bothViewportsRan(layout)
        layoutRan = true
      } catch {
        bothVp = false
        layoutRan = false
      }
    }
    let nodeByKey = new Map<string, string>()
    let sourceText = opts?.source ?? ''
    if (client.sailInspectModel) {
      try {
        const model = await client.sailInspectModel({ path })
        nodeByKey = nodeMapFromModelDiagnostics(model.diagnostics ?? [])
        if (!sourceText && typeof model.source === 'string') sourceText = model.source
      } catch {
        /* offline */
      }
    }
    if (!sourceText && client.readSource) {
      try {
        sourceText = await client.readSource(path)
      } catch {
        /* */
      }
    }
    // Prefer in-memory applied source so re-check sees fixes not yet on disk.
    const applied = lookupAppliedSource(path)
    if (applied != null && !opts?.source) sourceText = applied
    let findings = parseCheckFindings(chk.diagnostics ?? [], {
      defaultPath: path,
      bothViewports: bothVp,
      nodeByKey,
    })
    findings = enrichNodeIds(findings, nodeByKey)
    const sourceByPath = new Map(state.sourceByPath)
    if (sourceText) sourceByPath.set(path, sourceText)
    const keep =
      state.selectedId && findings.some((f) => f.id === state.selectedId)
        ? state.selectedId
        : findings[0]?.id ?? null
    state = {
      ...state,
      layoutBothViewportsRan: bothVp,
      sourceByPath,
      loading: false,
      error: null,
      selectedId: keep,
      lastRunAt: Date.now(),
      lastCheckedPath: path,
      lastLayoutChecked: layoutRan,
    }
    recomputeCounts(findings)
    emit()
    return findings
  } catch (e) {
    patch({
      loading: false,
      error: userError(e, 'Could not load checks — try again.'),
    })
    return []
  }
}

/** Re-run last check (or provided path). Used after apply + Re-check button. */
export async function recheckFindings(path?: string): Promise<Finding[]> {
  const p = path || lastLoadPath || state.lastCheckedPath
  if (!p) return []
  // Prefer applied buffer so re-check reflects fixes not yet on disk.
  const src =
    lookupAppliedSource(p) ??
    lastLoadSource ??
    state.sourceByPath.get(p)
  return loadFindings(p, src != null ? { source: src } : undefined)
}

function pickSelected(findings: Finding[], prefer: string | null): string | null {
  if (prefer && findings.some((f) => f.id === prefer)) return prefer
  return findings[0]?.id ?? null
}

function lookupAppliedSource(path: string): string | undefined {
  if (state.sourceByPath.has(path)) return state.sourceByPath.get(path)
  for (const [p, t] of state.sourceByPath) {
    if (p === path || p.endsWith(path) || path.endsWith(p)) return t
  }
  return undefined
}

export function visibleFindings(): Finding[] {
  return filterFindings(state.findings, state.ruleFilter, state.fileFilter)
}

export function uiFindingRows(): FindingRow[] {
  return toFindingRows(visibleFindings(), state.selectedId)
}

export function uiSelectedFinding(): Finding | null {
  if (!state.selectedId) return null
  return state.findings.find((f) => f.id === state.selectedId) ?? null
}

export function uiRuleOptions(): string[] {
  return ruleOptions(state.findings)
}

export function uiFileOptions(): string[] {
  return fileOptions(state.findings)
}

export function uiGroups() {
  return uiRuleGroups()
}

export function uiRuleGroups() {
  const groups = groupByRule(visibleFindings())
  const src = state.sourceByPath
  return groups.map((g) => ({
    ...g,
    fixableCount: machineFixableFindings(g.items, src).length,
    collapsed: state.collapsedRules.includes(g.rule),
  }))
}

/** Ordered leaf findings for createTable (rule count-desc, stable severity). */
export function uiTableFindings(): Finding[] {
  return flattenRuleGroups(groupByRule(visibleFindings()))
}

export function uiSeverityGroups() {
  return groupBySeverity(visibleFindings())
}

export function uiFixableCount(): number {
  return machineFixableFindings(visibleFindings(), state.sourceByPath).length
}

export function findingIsMachineFixable(finding: Finding): boolean {
  return isMachineFixable(state.sourceByPath.get(finding.path) ?? '', finding)
}

function resolvePath(path: string): string {
  if (path.startsWith('/')) return path
  const root = state.projectRoot.replace(/\/$/, '')
  return root ? `${root}/${path}` : path
}

export function snippetForFinding(finding: Finding) {
  return sourceSnippet(
    state.sourceByPath.get(finding.path) ?? '',
    finding.line,
    finding.col,
    1,
  )
}

export function openSourceUrlFor(finding?: Finding | null): string {
  const f = finding ?? uiSelectedFinding()
  if (!f) return ''
  return editorDeepLink(state.editor, resolvePath(f.path), f.line, f.col)
}

export function selectInApp(finding?: Finding | null): void {
  const f = finding ?? uiSelectedFinding()
  if (!f) return
  if (f.nodeId) selectNode(f.nodeId, 'surface')
  postStudioMessage({
    type: 'select',
    nodeId: f.nodeId ?? null,
    path: f.path,
    line: f.line,
    col: f.col,
  })
  patch({ selectedId: f.id })
}

export function applyFix(findingId?: string):
  | { ok: true; message: string; text?: string }
  | { ok: false; status: string; message: string; findingId: string } {
  const id = findingId ?? state.selectedId
  const f = state.findings.find((x) => x.id === id)
  if (!f) {
    return { ok: false, status: 'deferred', message: 'no finding selected', findingId: id ?? '' }
  }
  if (!APPLY_FIX_READY) {
    return {
      ok: false,
      status: 'deferred',
      message: 'apply fix unavailable',
      findingId: f.id,
    }
  }
  const src = state.sourceByPath.get(f.path) ?? ''
  const result = applyFixToSource(src, f)
  if (!result.ok || result.text === src) {
    return {
      ok: false,
      status: 'no_fix',
      message: result.message || 'no machine fix for this finding',
      findingId: f.id,
    }
  }
  const lines = src.split('\n')
  const before = (lines[f.line - 1] ?? src).trimEnd()
  const afterLines = result.text.split('\n')
  const after = (afterLines[f.line - 1] ?? result.text).trimEnd()
  const next = new Map(state.sourceByPath)
  next.set(f.path, result.text)
  const findings = state.findings.filter((x) => x.id !== f.id)
  const attr = attributionFor(f, before, after)
  state = {
    ...state,
    sourceByPath: next,
    findings,
    reviewNote: 'Applied 1 fix.',
    selectedId: pickSelected(findings, state.selectedId === f.id ? null : state.selectedId),
  }
  pushAttributions([attr])
  recomputeCounts(findings)
  emit()
  commitAppliedSource(f.path, result.text)
  // Re-run check so the table reflects NOW (fire-and-forget).
  void recheckFindings()
  return { ok: true, message: result.message, text: result.text }
}

/** Apply every machine fix for one rule (apply-all on group header). */
export function applyRuleFixes(rule: string): { applied: number } {
  const items = buildFixReviewItems(
    visibleFindings().filter((f) => f.rule === rule),
    state.sourceByPath,
  ).map((i) => ({ ...i, accepted: true }))
  if (!items.length) return { applied: 0 }
  const { sourceByPath, applied, appliedIds, changedPaths } = applyAcceptedReviews(
    items,
    state.findings,
    state.sourceByPath,
  )
  const idSet = new Set(appliedIds)
  const attrs: FixAttribution[] = []
  const at = Date.now()
  for (const item of items) {
    if (!idSet.has(item.findingId)) continue
    attrs.push({
      id: `${item.findingId}|${at}`,
      rule: item.rule,
      path: item.path,
      findingId: item.findingId,
      before: item.before,
      after: item.after,
      at,
    })
  }
  const findings = state.findings.filter((f) => !idSet.has(f.id))
  state = {
    ...state,
    sourceByPath,
    findings,
    reviewNote:
      applied > 0
        ? `Applied ${applied} fix${applied === 1 ? '' : 'es'} for ${rule}.`
        : 'No fixes applied.',
    selectedId: pickSelected(findings, state.selectedId),
  }
  pushAttributions(attrs)
  recomputeCounts(findings)
  emit()
  if (changedPaths.length) commitAppliedSources(sourceByPath, changedPaths)
  if (applied > 0) void recheckFindings()
  return { applied }
}

export function openFixReview(): void {
  const items = buildFixReviewItems(visibleFindings(), state.sourceByPath)
  patch({
    reviewOpen: true,
    reviewItems: items,
    reviewNote: items.length === 0 ? 'No automatic fixes available.' : '',
  })
}

export function closeFixReview(): void {
  patch({ reviewOpen: false })
}

export function setFixReviewAccepted(findingId: string, accepted: boolean): void {
  patch({
    reviewItems: state.reviewItems.map((i) =>
      i.findingId === findingId ? { ...i, accepted } : i,
    ),
  })
}

export function applyFixReview(): { applied: number } {
  const acceptedItems = state.reviewItems.filter((i) => i.accepted && i.machine)
  const { sourceByPath, applied, appliedIds, changedPaths } = applyAcceptedReviews(
    state.reviewItems,
    state.findings,
    state.sourceByPath,
  )
  const idSet = new Set(appliedIds)
  const at = Date.now()
  const attrs: FixAttribution[] = acceptedItems
    .filter((i) => idSet.has(i.findingId))
    .map((i) => ({
      id: `${i.findingId}|${at}`,
      rule: i.rule,
      path: i.path,
      findingId: i.findingId,
      before: i.before,
      after: i.after,
      at,
    }))
  const findings = state.findings.filter((f) => !idSet.has(f.id))
  state = {
    ...state,
    sourceByPath,
    findings,
    reviewOpen: false,
    reviewItems: [],
    reviewNote:
      applied > 0
        ? `Applied ${applied} fix${applied === 1 ? '' : 'es'}.`
        : 'No fixes applied — all skipped.',
    selectedId: pickSelected(findings, state.selectedId),
  }
  pushAttributions(attrs)
  recomputeCounts(findings)
  emit()
  if (changedPaths.length) commitAppliedSources(sourceByPath, changedPaths)
  if (applied > 0) void recheckFindings()
  return { applied }
}

export function uiReviewAcceptedCount(): number {
  return countAccepted(state.reviewItems)
}

export function sourceForSave(path: string, source: string): {
  text: string
  changed: boolean
  applied: number
} {
  const appliedText = lookupAppliedSource(path)
  if (appliedText != null && appliedText !== source) {
    return { text: appliedText, changed: true, applied: 1 }
  }
  const findings = state.findings.filter((f) => f.path === path || !f.path)
  return autoFixOnSave(source, findings, state.autoFixMode)
}

/** Composed empty-state body: what was checked, when. Null-run → honest idle. */
export function uiEmptyCheckedSummary(): string {
  if (!state.lastRunAt) return 'Not checked yet'
  const path = state.lastCheckedPath
  const when = new Date(state.lastRunAt).toLocaleTimeString()
  const parts: string[] = []
  if (path) {
    const base = path.split('/').pop() || path
    parts.push(`Checked ${base}`)
  } else {
    parts.push('Checked current path')
  }
  parts.push(state.lastLayoutChecked ? 'compiler + layout' : 'compiler')
  parts.push(`at ${when}`)
  if (state.autoFixMode !== 'off') {
    parts.push(`autofix ${state.autoFixMode} (${state.autoFixAttributed})`)
  }
  return parts.join(' · ')
}

export function __resetChecksForTests(): void {
  state = emptyState()
  client = null
  lastLoadPath = null
  lastLoadSource = undefined
  listeners.clear()
}

export {
  autoFixOnSave, bothViewportsRan, parseCheckFindings,
  editorDeepLink, parseEditorSetting, APPLY_FIX_READY,
}
