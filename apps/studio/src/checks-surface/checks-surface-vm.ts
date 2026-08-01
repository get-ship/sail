/**
 * Checks surface VM — compiler findings inbox grouped by rule (createTable),
 * expandable why + sail-diff suggest, autofix off|suggest|apply, re-check.
 */
import { createTable, type Table } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  APPLY_FIX_READY,
  applyFix,
  applyFixReview,
  applyFixToSource,
  applyRuleFixes,
  closeFixReview,
  findingIsMachineFixable,
  getChecksState,
  loadFindings,
  openFixReview,
  openSourceUrlFor,
  recheckFindings,
  selectFinding,
  selectInApp,
  setAutoFixMode,
  setFileFilter,
  setFixReviewAccepted,
  setRuleFilter,
  snippetForFinding,
  subscribeChecks,
  toggleRuleCollapsed,
  toFindingRows,
  uiEmptyCheckedSummary,
  uiFileOptions,
  uiFixableCount,
  uiReviewAcceptedCount,
  uiRuleGroups,
  uiRuleOptions,
  uiTableFindings,
  type AutoFixMode,
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
  severityLabel: string
  rule: string
  message: string
  fileSpan: string
  selected: boolean
  expanded: boolean
  showBothViewports: boolean
  showFixChip: boolean
  showApply: boolean
  hasSuggest: boolean
  /** sail-diff `from` — original span line (or empty). */
  diffFrom: string
  /** sail-diff `to` — suggest / rewritten line. */
  diffTo: string
  whyText: string
  snippet: SnippetLineUi[]
  openSourceUrl: string
  key: string
}

type RuleGroupUi = {
  rule: string
  countLabel: string
  errorBadge: string
  warningBadge: string
  showErrorBadge: boolean
  showWarningBadge: boolean
  collapsed: boolean
  open: boolean
  showApplyAll: boolean
  applyAllLabel: string
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

type AutoFixOptUi = {
  mode: AutoFixMode
  label: string
  active: boolean
  key: string
}

type FindingTableRow = {
  id: string
  severity: string
  rule: string
  fileSpan: string
  message: string
}

function severityTone(severity: string): string {
  if (severity === 'warning') return 'warning'
  if (severity === 'error') return 'destructive'
  return 'muted'
}

function severityLabel(severity: string): string {
  if (severity === 'error') return 'error'
  if (severity === 'warning') return 'warn'
  return severity || 'info'
}

function rowUi(f: Finding, selectedId: string | null): FindingRowUi {
  const base = toFindingRows([f], selectedId)[0]!
  const snip = snippetForFinding(f)
  const machine = findingIsMachineFixable(f)
  const spanLine =
    snip.lines.find((l) => l.highlight)?.text ??
    snip.lines[0]?.text ??
    ''
  const suggest = f.suggest ?? ''
  // Suggest mode: true diff when we have a rewrite target; else empty.
  let diffFrom = ''
  let diffTo = ''
  if (machine) {
    const src = getChecksState().sourceByPath.get(f.path) ?? ''
    const lines = src.split('\n')
    diffFrom = (lines[f.line - 1] ?? spanLine).trimEnd()
    const applied = applyFixToSource(src, f)
    if (applied.ok && applied.text !== src) {
      diffTo = (applied.text.split('\n')[f.line - 1] ?? applied.text).trimEnd()
    } else if (suggest) {
      diffFrom = spanLine
      diffTo = suggest
    }
  } else if (suggest) {
    diffFrom = spanLine
    diffTo = suggest
  }
  return {
    id: f.id,
    severity: f.severity,
    severityTone: severityTone(f.severity),
    severityLabel: severityLabel(f.severity),
    rule: f.rule,
    message: f.message,
    fileSpan: f.fileSpan,
    selected: base.selected,
    expanded: base.selected,
    showBothViewports: base.showBothViewports,
    showFixChip: machine || base.showFixChip,
    showApply: machine && APPLY_FIX_READY,
    hasSuggest: !!suggest || (diffFrom.length > 0 && diffTo.length > 0),
    diffFrom,
    diffTo,
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

function formatLastRun(at: number | null): string {
  if (!at) return 'Not run yet'
  try {
    return `Last run ${new Date(at).toLocaleTimeString()}`
  } catch {
    return 'Last run —'
  }
}

function resolveCheckPath(): string {
  const cs = getChecksState()
  const shell = getShellState()
  const edit = getEditState()
  const root = (cs.projectRoot || '').replace(/\/$/, '')
  return (
    edit.activePath ||
    shell.sailPath ||
    cs.lastCheckedPath ||
    cs.findings[0]?.path ||
    (root ? `${root}/routes/page.sail` : 'apps/studio/routes/page.sail')
  )
}

export function createChecksSurface() {
  const ruleGroups = signal<RuleGroupUi[]>([])
  const hasFindings = signal(false)
  const emptyTitle = signal('No findings — project is clean')
  const emptyBody = signal('Run checks to see what the compiler and layout pass cover.')
  const emptyChecked = signal('')
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
  const autoFixOpts = signal<AutoFixOptUi[]>([
    { mode: 'off', label: 'Off', active: true, key: 'off' },
    { mode: 'suggest', label: 'Suggest', active: false, key: 'suggest' },
    { mode: 'apply', label: 'Apply', active: false, key: 'apply' },
  ])
  const autoFixModeLabel = signal('Autofix off')
  const lastRunLabel = signal('Not run yet')
  const isLoading = signal(false)
  const recheckLabel = signal('Re-check')
  const showSuggestDiff = signal(false)

  const dataSig = signal<FindingTableRow[]>([])
  const table: Table<FindingTableRow> = createTable<FindingTableRow>({
    data: dataSig,
    columns: [
      { id: 'severity', accessorKey: 'severity', header: 'Severity', enableGrouping: false },
      { id: 'rule', accessorKey: 'rule', header: 'Rule', enableGrouping: true },
      { id: 'fileSpan', accessorKey: 'fileSpan', header: 'Span' },
      { id: 'message', accessorKey: 'message', header: 'Summary' },
    ],
    getRowId: (r) => r.id,
    grouping: ['rule'],
    enableExpanding: true,
    expanded: true,
    enableSorting: true,
    enableRowSelection: true,
  })

  const sync = () => {
    const cs = getChecksState()
    const groups = uiRuleGroups()
    const tableRows = uiTableFindings().map((f) => ({
      id: f.id,
      severity: f.severity,
      rule: f.rule,
      fileSpan: f.fileSpan,
      message: f.message,
    }))
    dataSig.set(tableRows)
    // Drive selection into sail-table engine (identity parity with explorer).
    if (cs.selectedId) {
      table.setRowSelection({ mode: 'include', ids: new Set([cs.selectedId]) })
    } else {
      table.resetRowSelection()
    }

    ruleGroups.set(
      groups.map((g) => ({
        rule: g.rule,
        countLabel: String(g.count),
        errorBadge: g.errorCount > 0 ? `${g.errorCount}e` : '',
        warningBadge: g.warningCount > 0 ? `${g.warningCount}w` : '',
        showErrorBadge: g.errorCount > 0,
        showWarningBadge: g.warningCount > 0,
        collapsed: g.collapsed,
        open: !g.collapsed,
        showApplyAll: g.fixableCount > 0 && APPLY_FIX_READY,
        applyAllLabel:
          g.fixableCount > 0
            ? `Apply all (${g.fixableCount})`
            : 'Apply all',
        key: g.rule,
        rows: g.collapsed
          ? []
          : g.items.map((f) => rowUi(f, cs.selectedId)),
      })),
    )
    const visible = groups.reduce((n, g) => n + g.count, 0)
    hasFindings.set(visible > 0)
    hasChecksError.set(!!cs.error)
    checksError.set(cs.error ?? '')
    if (cs.findings.length > 0 && visible === 0) {
      emptyTitle.set('No findings match filters')
      emptyBody.set('Clear rule or file filters to see all findings.')
      emptyChecked.set('')
    } else {
      emptyTitle.set('No findings — project is clean')
      emptyBody.set("Here's what was checked.")
      emptyChecked.set(uiEmptyCheckedSummary())
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
        acceptLabel: i.accepted ? 'Skip this fix' : 'Accept this fix',
        key: i.findingId,
      })),
    )
    const acc = uiReviewAcceptedCount()
    applyFixesLabel.set(`Apply ${acc} fix${acc === 1 ? '' : 'es'}`)
    canApplyFixes.set(acc > 0)
    const mode = cs.autoFixMode
    autoFixOpts.set(
      (['off', 'suggest', 'apply'] as AutoFixMode[]).map((m) => ({
        mode: m,
        label: m === 'off' ? 'Off' : m === 'suggest' ? 'Suggest' : 'Apply',
        active: mode === m,
        key: m,
      })),
    )
    autoFixModeLabel.set(
      mode === 'off'
        ? 'Autofix off'
        : mode === 'suggest'
          ? 'Autofix suggest'
          : 'Autofix apply',
    )
    showSuggestDiff.set(mode === 'suggest' || mode === 'apply')
    lastRunLabel.set(formatLastRun(cs.lastRunAt))
    isLoading.set(cs.loading)
    recheckLabel.set(cs.loading ? 'Checking…' : 'Re-check')
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
    if ((e.target as Element).closest?.('[data-rule-toggle]')) return
    if ((e.target as Element).closest?.('[data-apply-rule]')) return
    const id = t.dataset.findingId
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

  const onRuleToggle = (e: Event) => {
    const btn = (e.target as Element | null)?.closest?.(
      '[data-rule-toggle]',
    ) as HTMLElement | null
    if (!btn?.dataset.ruleToggle) return
    e.stopPropagation?.()
    toggleRuleCollapsed(btn.dataset.ruleToggle)
    sync()
  }

  const onApplyRule = (e: Event) => {
    const btn = (e.target as Element | null)?.closest?.(
      '[data-apply-rule]',
    ) as HTMLElement | null
    if (!btn?.dataset.applyRule) return
    e.stopPropagation?.()
    applyRuleFixes(btn.dataset.applyRule)
    sync()
  }

  const onRetryChecks = () => {
    void loadFindings(resolveCheckPath()).then(() => sync())
  }

  const onRecheck = () => {
    void recheckFindings(resolveCheckPath()).then(() => sync())
  }

  const onAutoFixMode = (e: Event) => {
    const btn = (e.target as Element | null)?.closest?.(
      '[data-autofix-mode]',
    ) as HTMLElement | null
    if (!btn?.dataset.autofixMode) return
    const mode = btn.dataset.autofixMode as AutoFixMode
    if (mode !== 'off' && mode !== 'suggest' && mode !== 'apply') return
    setAutoFixMode(mode, 'studio checks')
    sync()
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
    ruleGroups,
    hasFindings,
    emptyTitle,
    emptyBody,
    emptyChecked,
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
    autoFixOpts,
    autoFixModeLabel,
    lastRunLabel,
    isLoading,
    recheckLabel,
    showSuggestDiff,
    table,
    onRuleFilter,
    onFileFilter,
    onFindingClick,
    onFindingAction,
    onRuleToggle,
    onApplyRule,
    onRetryChecks,
    onRecheck,
    onAutoFixMode,
    onOpenReview,
    onCloseReview,
    onReviewToggle,
    onApplyReview,
  }
}
