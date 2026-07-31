/**
 * Checks surface VM — compiler findings inbox: severity groups, expand rows,
 * Review fixes dialog. Mirrors @ship/studio/checks; shell owns loadFindings.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  APPLY_FIX_READY,
  applyFix,
  applyFixReview,
  closeFixReview,
  findingIsMachineFixable,
  getChecksState,
  loadFindings,
  openFixReview,
  openSourceUrlFor,
  selectFinding,
  selectInApp,
  setFileFilter,
  setFixReviewAccepted,
  setRuleFilter,
  snippetForFinding,
  subscribeChecks,
  toFindingRows,
  uiFileOptions,
  uiFixableCount,
  uiReviewAcceptedCount,
  uiRuleOptions,
  uiSeverityGroups,
  type Finding,
} from '@ship/studio/checks'
import { getEditState } from '@ship/studio/edit'
import { getShellState } from '@ship/sail-studio-shell'

type SnippetLineUi = { n: number; text: string; highlight: boolean; key: string }
type OptUi = { value: string; label: string; key: string }

type FindingRowUi = {
  id: string
  severity: string
  severityTone: string
  rule: string
  message: string
  fileSpan: string
  selected: boolean
  expanded: boolean
  showBothViewports: boolean
  showFixChip: boolean
  showApply: boolean
  hasSuggest: boolean
  suggestText: string
  whyText: string
  snippet: SnippetLineUi[]
  openSourceUrl: string
  key: string
}

type SeverityGroupUi = {
  severity: string
  label: string
  tone: string
  countLabel: string
  key: string
  rows: FindingRowUi[]
}

type ReviewItemUi = {
  findingId: string
  rule: string
  message: string
  fileSpan: string
  before: string
  after: string
  accepted: boolean
  acceptLabel: string
  key: string
}

function severityTone(severity: string): string {
  if (severity === 'warning') return 'warning'
  if (severity === 'error') return 'destructive'
  return 'muted'
}

function rowUi(f: Finding, selectedId: string | null): FindingRowUi {
  const base = toFindingRows([f], selectedId)[0]!
  const snip = snippetForFinding(f)
  const machine = findingIsMachineFixable(f)
  return {
    id: f.id,
    severity: f.severity,
    severityTone: severityTone(f.severity),
    rule: f.rule,
    message: f.message,
    fileSpan: f.fileSpan,
    selected: base.selected,
    expanded: base.selected,
    showBothViewports: base.showBothViewports,
    showFixChip: machine || base.showFixChip,
    showApply: machine && APPLY_FIX_READY,
    hasSuggest: !!f.suggest,
    suggestText: f.suggest ?? '',
    whyText: f.why,
    snippet: snip.lines.map((l) => ({
      n: l.n,
      text: l.text,
      highlight: l.highlight,
      key: String(l.n),
    })),
    openSourceUrl: openSourceUrlFor(f),
    key: f.id,
  }
}

export function createChecksSurface() {
  const severityGroups = signal<SeverityGroupUi[]>([])
  const hasFindings = signal(false)
  const emptyTitle = signal('All clear')
  const emptyBody = signal(
    'No compiler findings for this path. Save or re-check after edits.',
  )
  const errorCountLabel = signal('0 errors')
  const warningCountLabel = signal('0 warnings')
  const checksStatusNote = signal('All clear')
  const ruleOpts = signal<OptUi[]>([{ value: '', label: 'All rules', key: 'all' }])
  const fileOpts = signal<OptUi[]>([{ value: '', label: 'All files', key: 'all' }])
  const ruleFilterVal = signal('')
  const fileFilterVal = signal('')
  const hasFixable = signal(false)
  const fixableCountLabel = signal('Review fixes')
  const reviewOpen = signal(false)
  const reviewItems = signal<ReviewItemUi[]>([])
  const applyFixesLabel = signal('Apply 0 fixes')
  const canApplyFixes = signal(false)
  const hasChecksError = signal(false)
  const checksError = signal('')
  const sync = () => {
    const cs = getChecksState()
    const groups = uiSeverityGroups()
    severityGroups.set(
      groups.map((g) => ({
        severity: g.severity,
        label: g.label,
        tone: g.tone,
        countLabel: String(g.count),
        key: g.severity,
        rows: g.items.map((f) => rowUi(f, cs.selectedId)),
      })),
    )
    const visible = groups.reduce((n, g) => n + g.items.length, 0)
    hasFindings.set(visible > 0)
    hasChecksError.set(!!cs.error)
    checksError.set(cs.error ?? '')
    // Empty-state is neutral only — errors use the error strip + retry.
    if (cs.findings.length > 0 && visible === 0) {
      emptyTitle.set('No findings match filters')
      emptyBody.set('Clear rule or file filters to see all findings.')
    } else {
      emptyTitle.set('No findings — project is clean')
      emptyBody.set('Save or re-check after edits to refresh compiler findings.')
    }
    errorCountLabel.set(
      `${cs.errorCount} error${cs.errorCount === 1 ? '' : 's'}`,
    )
    warningCountLabel.set(
      `${cs.warningCount} warning${cs.warningCount === 1 ? '' : 's'}`,
    )
    checksStatusNote.set(cs.reviewNote || cs.statusNote)
    ruleOpts.set([
      { value: '', label: 'All rules', key: 'all' },
      ...uiRuleOptions().map((r) => ({ value: r, label: r, key: r })),
    ])
    fileOpts.set([
      { value: '', label: 'All files', key: 'all' },
      ...uiFileOptions().map((p) => ({
        value: p,
        label: p.split('/').pop() ?? p,
        key: p,
      })),
    ])
    ruleFilterVal.set(cs.ruleFilter ?? '')
    fileFilterVal.set(cs.fileFilter ?? '')
    const fixN = uiFixableCount()
    hasFixable.set(fixN > 0)
    fixableCountLabel.set(
      fixN > 0 ? `Review fixes (${fixN})` : 'Review fixes',
    )
    reviewOpen.set(cs.reviewOpen)
    reviewItems.set(
      cs.reviewItems.map((i) => ({
        findingId: i.findingId,
        rule: i.rule,
        message: i.message,
        fileSpan: i.fileSpan,
        before: i.before,
        after: i.after,
        accepted: i.accepted,
        // Action label = what the click does (not current state).
        acceptLabel: i.accepted ? 'Skip this fix' : 'Accept this fix',
        key: i.findingId,
      })),
    )
    const acc = uiReviewAcceptedCount()
    applyFixesLabel.set(`Apply ${acc} fix${acc === 1 ? '' : 'es'}`)
    canApplyFixes.set(acc > 0)
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeChecks(() => sync())
    return () => unsub()
  })

  const onRuleFilter = (e: Event) => {
    setRuleFilter((e.target as HTMLSelectElement).value || null)
    sync()
  }

  const onFileFilter = (e: Event) => {
    setFileFilter((e.target as HTMLSelectElement).value || null)
    sync()
  }

  const onFindingClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.(
      '[data-finding-id]',
    ) as HTMLElement | null
    if (!t?.dataset.findingId) return
    if ((e.target as Element).closest?.('[data-finding-action]')) return
    const id = t.dataset.findingId
    // Re-click selected row collapses (toggle expand).
    selectFinding(getChecksState().selectedId === id ? null : id)
    sync()
  }

  const onFindingAction = (e: Event) => {
    const btn = (e.target as Element | null)?.closest?.(
      '[data-finding-action]',
    ) as HTMLElement | null
    const row = (e.target as Element | null)?.closest?.(
      '[data-finding-id]',
    ) as HTMLElement | null
    if (!btn?.dataset.findingAction || !row?.dataset.findingId) return
    e.stopPropagation?.()
    selectFinding(row.dataset.findingId)
    const action = btn.dataset.findingAction
    if (action === 'open-source') {
      const url = openSourceUrlFor()
      if (url && typeof window !== 'undefined') {
        try {
          window.open(url, '_blank')
        } catch {
          /* */
        }
      }
    } else if (action === 'select-in-app') {
      selectInApp()
    } else if (action === 'apply-fix') {
      applyFix(row.dataset.findingId)
    }
    sync()
  }

  const onRetryChecks = () => {
    const cs = getChecksState()
    const shell = getShellState()
    const edit = getEditState()
    const root = (cs.projectRoot || '').replace(/\/$/, '')
    const retryPath =
      edit.activePath ||
      shell.sailPath ||
      cs.findings[0]?.path ||
      (root ? `${root}/routes/page.sail` : 'apps/studio/routes/page.sail')
    void loadFindings(retryPath).then(() => sync())
  }

  const onOpenReview = () => {
    openFixReview()
    sync()
  }

  const onCloseReview = () => {
    closeFixReview()
    sync()
  }

  const onReviewToggle = (e: Event) => {
    const btn = (e.target as Element | null)?.closest?.(
      '[data-review-id]',
    ) as HTMLElement | null
    if (!btn?.dataset.reviewId) return
    const id = btn.dataset.reviewId
    const item = getChecksState().reviewItems.find((i) => i.findingId === id)
    if (!item) return
    setFixReviewAccepted(id, !item.accepted)
    sync()
  }

  const onApplyReview = () => {
    applyFixReview()
    sync()
  }

  return {
    severityGroups,
    hasFindings,
    emptyTitle,
    emptyBody,
    hasChecksError,
    checksError,
    errorCountLabel,
    warningCountLabel,
    checksStatusNote,
    ruleOpts,
    fileOpts,
    ruleFilterVal,
    fileFilterVal,
    hasFixable,
    fixableCountLabel,
    reviewOpen,
    reviewItems,
    applyFixesLabel,
    canApplyFixes,
    onRuleFilter,
    onFileFilter,
    onFindingClick,
    onFindingAction,
    onRetryChecks,
    onOpenReview,
    onCloseReview,
    onReviewToggle,
    onApplyReview,
  }
}
