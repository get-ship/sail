/**
 * Checks surface VM — createTable owns the findings inbox (grouping/sort/
 * filter/selection/expand). Host gets table identity via assignTableHost.
 * Detail chrome (why, sail-diff, exits) sits on selected leaf rows.
 */
import { createTable, type Table } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  APPLY_FIX_READY,
  applyFix,
  applyFixReview,
  applyFixToSource,
  applyRuleFixes,
  assignTableHost,
  buildExpandedMap,
  closeFixReview,
  findingIsMachineFixable,
  getChecksState,
  groupMetaFromRows,
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
  toggleCollapsedRule,
  toFindingRows,
  uiEmptyCheckedSummary,
  uiFileOptions,
  uiFixableCount,
  uiReviewAcceptedCount,
  uiRuleOptions,
  uiTableFindings,
  type AutoFixMode,
  type Finding,
} from '@ship/studio/checks'
import { navigate } from '@ship/deck/router'
import { getEditState, openDocument, requestSourceScroll } from '@ship/studio/edit'
import { getShellState } from '@ship/sail-studio-shell'
import { hrefForActivity } from '../shell/shell-page-vm'

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
  showDiff: boolean
  diffFrom: string
  diffTo: string
  whyText: string
  snippet: SnippetLineUi[]
  openSourceUrl: string
  key: string
}

type RuleGroupUi = {
  rule: string
  groupRowId: string
  countLabel: string
  errorBadge: string
  warningBadge: string
  infoBadge: string
  showErrorBadge: boolean
  showWarningBadge: boolean
  showInfoBadge: boolean
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

/** Flat row fed to createTable — engine groups by `rule`. */
export type FindingTableRow = {
  id: string
  severity: string
  severityRank: number
  rule: string
  fileSpan: string
  message: string
  path: string
}

const SEV_RANK: Record<string, number> = {
  error: 0,
  warning: 1,
  info: 2,
  hint: 3,
}

function severityTone(severity: string): string {
  if (severity === 'warning') return 'warning'
  if (severity === 'error') return 'destructive'
  return 'muted'
}

function severityLabel(severity: string): string {
  if (severity === 'error') return 'error'
  if (severity === 'warning') return 'warn'
  if (severity === 'info') return 'info'
  return severity || 'info'
}

function rowUi(
  f: Finding,
  selectedId: string | null,
  showSuggestDiff: boolean,
): FindingRowUi {
  const base = toFindingRows([f], selectedId)[0]!
  const snip = snippetForFinding(f)
  const machine = findingIsMachineFixable(f)
  const spanLine =
    snip.lines.find((l) => l.highlight)?.text ??
    snip.lines[0]?.text ??
    ''
  const suggest = f.suggest ?? ''
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
  const hasSuggest = !!suggest || (diffFrom.length > 0 && diffTo.length > 0)
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
    hasSuggest,
    showDiff: hasSuggest && showSuggestDiff,
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

/**
 * Walk engine `table.rows()` (group + leaf flatten) into UI rule groups.
 * This is the single list source — not a parallel store groupByRule path.
 */
function groupsFromTable(
  table: Table<FindingTableRow>,
  findingsById: Map<string, Finding>,
  selectedId: string | null,
  showSuggestDiff: boolean,
): RuleGroupUi[] {
  const out: RuleGroupUi[] = []
  let current: RuleGroupUi | null = null
  for (const row of table.rows()) {
    if (row.getIsGrouped()) {
      const rule = String(
        row.getGroupingValue('rule') ?? row.getValue('rule') ?? '',
      )
      const leaves = row.getLeafRows()
      let errorCount = 0
      let warningCount = 0
      let infoCount = 0
      let fixableCount = 0
      for (const leaf of leaves) {
        const sev = String(leaf.getValue('severity') ?? '')
        if (sev === 'error') errorCount++
        else if (sev === 'warning') warningCount++
        else infoCount++
        const f = findingsById.get(leaf.id)
        if (f && findingIsMachineFixable(f)) fixableCount++
      }
      current = {
        rule,
        groupRowId: row.id,
        countLabel: String(leaves.length),
        errorBadge: errorCount > 0 ? `${errorCount}e` : '',
        warningBadge: warningCount > 0 ? `${warningCount}w` : '',
        infoBadge: infoCount > 0 ? `${infoCount}i` : '',
        showErrorBadge: errorCount > 0,
        showWarningBadge: warningCount > 0,
        showInfoBadge: infoCount > 0,
        collapsed: !row.getIsExpanded(),
        open: row.getIsExpanded(),
        showApplyAll: fixableCount > 0 && APPLY_FIX_READY,
        applyAllLabel:
          fixableCount > 0 ? `Apply all (${fixableCount})` : 'Apply all',
        key: rule,
        rows: [],
      }
      out.push(current)
    } else if (current) {
      const f = findingsById.get(row.id)
      if (f) current.rows.push(rowUi(f, selectedId, showSuggestDiff))
    }
  }
  return out
}

export function createChecksSurface() {
  const ruleGroups = signal<RuleGroupUi[]>([])
  const hasFindings = signal(false)
  const emptyTitle = signal('Run checks')
  const emptyBody = signal(
    'Re-check to load compiler and layout findings for the open path.',
  )
  const emptyChecked = signal('Not checked yet')
  const errorCountLabel = signal('0 errors')
  const warningCountLabel = signal('0 warnings')
  const checksStatusNote = signal('All clear')
  const ruleOpts = signal<OptUi[]>([{ value: '', label: 'All rules', key: 'all' }])
  const fileOpts = signal<OptUi[]>([{ value: '', label: 'All files', key: 'all' }])
  const severityOpts = signal<OptUi[]>([
    { value: '', label: 'All severities', key: 'all' },
    { value: 'error', label: 'Errors', key: 'error' },
    { value: 'warning', label: 'Warnings', key: 'warning' },
    { value: 'info', label: 'Info', key: 'info' },
  ])
  const ruleFilterVal = signal('')
  const fileFilterVal = signal('')
  const severityFilterVal = signal('')
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
  /** Rules the user collapsed — never use expand-all `true` (bare toggle is no-op). */
  let collapsedRules = new Set<string>()
  const table: Table<FindingTableRow> = createTable<FindingTableRow>({
    data: dataSig,
    columns: [
      {
        id: 'severity',
        accessorKey: 'severity',
        header: 'Severity',
        enableGrouping: false,
        enableColumnFilter: true,
        enableSorting: true,
        // Stable severity order via numeric rank column companion.
        sortFn: (a, b) => {
          const ra = Number(a.original.severityRank ?? 99)
          const rb = Number(b.original.severityRank ?? 99)
          return ra - rb
        },
      },
      {
        id: 'rule',
        accessorKey: 'rule',
        header: 'Rule',
        enableGrouping: true,
        enableColumnFilter: true,
      },
      {
        id: 'fileSpan',
        accessorKey: 'fileSpan',
        header: 'Span',
        enableColumnFilter: true,
      },
      {
        id: 'message',
        accessorKey: 'message',
        header: 'Summary',
      },
      {
        id: 'severityRank',
        accessorKey: 'severityRank',
        header: 'Rank',
        enableHiding: true,
        enableColumnFilter: false,
      },
    ],
    getRowId: (r) => r.id,
    grouping: ['rule'],
    enableExpanding: true,
    // Explicit map only — never expand-all `true` (toggle bare becomes no-op).
    expanded: {},
    enableSorting: true,
    enableFilters: true,
    enableRowSelection: true,
    initialState: {
      sorting: [{ id: 'severity', desc: false }],
      columnVisibility: { severityRank: false },
    },
  })

  const bindHost = () => {
    assignTableHost('[data-findings-table-host]', table)
  }

  /** Apply full expanded map so each group is open unless user-collapsed. */
  const applyExpandedMap = () => {
    const groups = groupMetaFromRows(table.rows())
    table.setExpanded(buildExpandedMap(groups, collapsedRules))
  }

  const sync = () => {
    const cs = getChecksState()
    const mode = cs.autoFixMode
    const suggestOn = mode === 'suggest' || mode === 'apply'
    showSuggestDiff.set(suggestOn)

    // Ordered leaves for createTable (count-desc rule order → first-bucket groups).
    const ordered = uiTableFindings()
    const findingsById = new Map(ordered.map((f) => [f.id, f]))
    dataSig.set(
      ordered.map((f) => ({
        id: f.id,
        severity: String(f.severity),
        severityRank: SEV_RANK[String(f.severity)] ?? 99,
        rule: f.rule,
        fileSpan: f.fileSpan,
        message: f.message,
        path: f.path,
      })),
    )

    // Severity facet lives on the table engine (not a parallel list filter).
    const sev = severityFilterVal()
    if (sev) table.setColumnFilter('severity', sev)
    else table.setColumnFilter('severity', undefined)

    // Expand map after data (group headers exist even when collapsed).
    applyExpandedMap()

    // Selection identity on the engine.
    if (cs.selectedId && findingsById.has(cs.selectedId)) {
      table.setRowSelection({ mode: 'include', ids: new Set([cs.selectedId]) })
    } else {
      table.resetRowSelection()
    }

    // THE inbox: walk engine rows (group + leaves) — sole list source.
    const groups = groupsFromTable(table, findingsById, cs.selectedId, suggestOn)
    ruleGroups.set(groups)

    const leafCount = ordered.length
    // After severity facet, engine may hide rows — use group leaf totals.
    const facetVisible = groups.reduce((n, g) => n + Number(g.countLabel), 0)
    hasFindings.set(facetVisible > 0)
    hasChecksError.set(!!cs.error)
    checksError.set(cs.error ?? '')

    if (!cs.lastRunAt) {
      emptyTitle.set('Run checks')
      emptyBody.set(
        'Re-check to load compiler and layout findings for the open path.',
      )
      emptyChecked.set('Not checked yet')
    } else if (cs.findings.length > 0 && leafCount === 0) {
      emptyTitle.set('No findings match filters')
      emptyBody.set('Clear rule, file, or severity filters to see all findings.')
      emptyChecked.set('')
    } else if (facetVisible === 0 && leafCount > 0) {
      emptyTitle.set('No findings match filters')
      emptyBody.set('Clear rule, file, or severity filters to see all findings.')
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
    lastRunLabel.set(formatLastRun(cs.lastRunAt))
    isLoading.set(cs.loading)
    recheckLabel.set(cs.loading ? 'Checking…' : 'Re-check')

    // Bind engine identity to the host after DOM paint.
    queueMicrotask(bindHost)
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeChecks(() => sync())
    queueMicrotask(bindHost)
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

  const onSeverityFilter = (e: Event) => {
    const v = (e.target as HTMLSelectElement).value || ''
    severityFilterVal.set(v)
    if (v) table.setColumnFilter('severity', v)
    else table.setColumnFilter('severity', undefined)
    // Re-walk engine rows after facet change (no store emit).
    const cs = getChecksState()
    const ordered = uiTableFindings()
    const findingsById = new Map(ordered.map((f) => [f.id, f]))
    ruleGroups.set(
      groupsFromTable(
        table,
        findingsById,
        cs.selectedId,
        showSuggestDiff(),
      ),
    )
    const facetVisible = ruleGroups().reduce(
      (n, g) => n + Number(g.countLabel),
      0,
    )
    hasFindings.set(facetVisible > 0)
    if (facetVisible === 0 && ordered.length > 0) {
      emptyTitle.set('No findings match filters')
      emptyBody.set('Clear rule, file, or severity filters to see all findings.')
      emptyChecked.set('')
    }
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
    const next = getChecksState().selectedId === id ? null : id
    selectFinding(next)
    if (next) {
      table.setRowSelection({ mode: 'include', ids: new Set([next]) })
    } else {
      table.resetRowSelection()
    }
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
      // Primary: open the file in Studio Edit at the finding span.
      const f = getChecksState().findings.find((x) => x.id === row.dataset.findingId)
      if (f?.path) {
        void openDocument(f.path, {
          loadPreview: false,
          theme: getShellState().themeId,
          mode: getShellState().modeId,
        }).then(() => {
          requestSourceScroll(f.line, f.col)
          navigate(hrefForActivity('edit'))
        })
      } else {
        // Fallback: external editor deep link when path is unavailable.
        const url = openSourceUrlFor()
        if (url && typeof window !== 'undefined') {
          try {
            window.open(url, '_blank')
          } catch {
            /* */
          }
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
    const rule =
      btn?.dataset.ruleToggle ||
      (() => {
        const gid = btn?.dataset.groupRowId
        if (!gid) return ''
        const hit = table.rows().find((r) => r.id === gid && r.getIsGrouped())
        return hit
          ? String(hit.getGroupingValue('rule') ?? hit.getValue('rule') ?? '')
          : ''
      })()
    if (!rule) return
    e.stopPropagation?.()
    // Explicit map — never bare toggleExpanded under expand-all.
    collapsedRules = toggleCollapsedRule(collapsedRules, rule)
    applyExpandedMap()
    const cs = getChecksState()
    const ordered = uiTableFindings()
    const findingsById = new Map(ordered.map((f) => [f.id, f]))
    ruleGroups.set(
      groupsFromTable(
        table,
        findingsById,
        cs.selectedId,
        showSuggestDiff(),
      ),
    )
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
    severityOpts,
    ruleFilterVal,
    fileFilterVal,
    severityFilterVal,
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
    /** Exposed for host/tests — createTable identity bound to data-findings-table-host. */
    table,
    onRuleFilter,
    onFileFilter,
    onSeverityFilter,
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
