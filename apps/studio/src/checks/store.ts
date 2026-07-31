import { getShellState, selectNode, setBadges } from '@ship/sail-studio-shell'
import { STUDIO_CHANNEL } from '@ship/sail-studio-bridge'
import {
  autoFixOnSave, bothViewportsRan, countSeverities, enrichNodeIds,
  filterFindings, fileOptions, groupByRule, nodeMapFromModelDiagnostics,
  parseCheckFindings, ruleOptions, sourceSnippet, statusNoteFor,
  toFindingRows, applyFixStub, applyFixToSource,
} from './findings'
import { groupBySeverity } from './groups'
import {
  applyAcceptedReviews, buildFixReviewItems, countAccepted,
  isMachineFixable, machineFixableFindings,
} from './review'
import { commitAppliedSource, commitAppliedSources } from './commit'
import { defaultEditorSetting, editorDeepLink, parseEditorSetting } from './deeplink'
import type {
  AutoFixMode, ChecksOpsClient, ChecksSurfaceState,
  EditorSetting, Finding, FindingRow,
} from './types'
import { APPLY_FIX_READY } from './types'
import { userError } from '../user-error'

type Listener = (s: ChecksSurfaceState) => void

let client: ChecksOpsClient | null = null
let state: ChecksSurfaceState = emptyState()
const listeners = new Set<Listener>()

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

export async function loadFindings(
  path: string,
  opts?: { source?: string },
): Promise<Finding[]> {
  if (!client) {
    // Ops not wired yet — honest empty, not a red error (chrome boots client next).
    patch({ error: null, loading: false })
    return []
  }
  patch({ loading: true, error: null })
  try {
    const args = opts?.source ? { path, source: opts.source } : { path }
    const chk = await client.sailAuthoringCheck(args)
    let bothVp = false
    if (client.sailAuthoringLayoutCheck) {
      try {
        bothVp = bothViewportsRan(await client.sailAuthoringLayoutCheck(args))
      } catch {
        bothVp = false
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
  return groupByRule(visibleFindings())
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
  if (typeof window !== 'undefined') {
    window.postMessage(
      {
        channel: STUDIO_CHANNEL,
        type: 'select',
        nodeId: f.nodeId ?? null,
        path: f.path,
        line: f.line,
        col: f.col,
      },
      '*',
    )
  }
  patch({ selectedId: f.id })
}

export function applyFix(findingId?: string):
  | ReturnType<typeof applyFixStub>
  | { ok: true; message: string; text?: string }
  | { ok: false; status: string; message: string; findingId: string } {
  const id = findingId ?? state.selectedId
  const f = state.findings.find((x) => x.id === id)
  if (!f) {
    return { ok: false, status: 'deferred', message: 'no finding selected', findingId: id ?? '' }
  }
  if (!APPLY_FIX_READY) return applyFixStub(f)
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
  const next = new Map(state.sourceByPath)
  next.set(f.path, result.text)
  const findings = state.findings.filter((x) => x.id !== f.id)
  state = {
    ...state,
    sourceByPath: next,
    findings,
    reviewNote: 'Applied 1 fix.',
    selectedId: pickSelected(findings, state.selectedId === f.id ? null : state.selectedId),
  }
  recomputeCounts(findings)
  emit()
  commitAppliedSource(f.path, result.text)
  return { ok: true, message: result.message, text: result.text }
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
  const { sourceByPath, applied, appliedIds, changedPaths } = applyAcceptedReviews(
    state.reviewItems,
    state.findings,
    state.sourceByPath,
  )
  const idSet = new Set(appliedIds)
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
  recomputeCounts(findings)
  emit()
  if (changedPaths.length) commitAppliedSources(sourceByPath, changedPaths)
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

export function __resetChecksForTests(): void {
  state = emptyState()
  client = null
  listeners.clear()
}

export {
  autoFixOnSave, bothViewportsRan, parseCheckFindings,
  editorDeepLink, parseEditorSetting, APPLY_FIX_READY,
}
