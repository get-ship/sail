import type { StudioLocalOps, StudioOpsMethods } from '@ship/sail-studio-client'

export type AutoFixMode = 'off' | 'suggest' | 'apply'

export type Severity = 'error' | 'warning' | string

/** One finding — compiler/lint or layout (taffy). */
export type Finding = {
  id: string
  severity: Severity
  rule: string
  /** One-line list message (first line of diagnostic). */
  message: string
  /** Verbatim diagnostic text (shown when a row expands). */
  why: string
  /** Extracted "suggest:" rewrite when present. */
  suggest: string | null
  /** Compiler provided a suggest trailer (not always machine-applicable). */
  hasFix: boolean
  path: string
  line: number
  col: number
  endLine: number
  endCol: number
  /** Display "file:line" */
  fileSpan: string
  /** Layout finding + both viewport passes actually ran. */
  bothViewports: boolean
  isLayout: boolean
  /** Span-keyed node for select-in-app when known. */
  nodeId: string | null
}

export type FindingRow = Finding & {
  selected: boolean
  showBothViewports: boolean
  showFixChip: boolean
  /** apply fix UI — true when a machine rewrite is available */
  showApply: boolean
}

export type RuleFilter = string | null
export type FileFilter = string | null

export type EditorSetting = {
  /** vscode | cursor | zed | custom */
  scheme: string
  /** Template with {path} {line} {col}; absolute path substituted. */
  template: string
}

/** One row in the Review fixes dialog (accept/skip). */
export type FixReviewItem = {
  findingId: string
  rule: string
  message: string
  fileSpan: string
  path: string
  line: number
  before: string
  after: string
  accepted: boolean
  machine: boolean
}

/** Recorded applied fix — rule, before/after, timestamp. */
export type FixAttribution = {
  id: string
  rule: string
  path: string
  findingId: string
  before: string
  after: string
  at: number
}

export type ChecksSurfaceState = {
  projectRoot: string
  findings: Finding[]
  selectedId: string | null
  ruleFilter: RuleFilter
  fileFilter: FileFilter
  autoFixMode: AutoFixMode
  editor: EditorSetting
  /** LayoutCheck reported ≥2 successful viewport passes. */
  layoutBothViewportsRan: boolean
  errorCount: number
  warningCount: number
  statusNote: string
  loading: boolean
  error: string | null
  /** Last source text loaded for why snippets. */
  sourceByPath: Map<string, string>
  /** Project setting attribution (e.g. "sail.jsonc"). */
  autoFixAttributed: string
  /** Review fixes dialog open. */
  reviewOpen: boolean
  /** Pending accept/skip rows for Review fixes. */
  reviewItems: FixReviewItem[]
  /** Last apply-review status line (user-facing). */
  reviewNote: string
  /** Last successful check run (ms epoch); null before first run. */
  lastRunAt: number | null
  /** Path last checked (for composed empty state). */
  lastCheckedPath: string | null
  /** Whether layout (taffy) check ran on last load. */
  lastLayoutChecked: boolean
  /** Applied fixes with attribution (newest first). */
  attributions: FixAttribution[]
  /** Collapsed rule ids in the findings table. */
  collapsedRules: string[]
}

/** Checks surface client — derived from generated ops (no hand-written shapes). */
export type ChecksOpsClient = Pick<StudioOpsMethods, 'sailAuthoringCheck'> &
  Partial<
    Pick<StudioOpsMethods, 'sailAuthoringLayoutCheck' | 'sailInspectModel'>
  > &
  Pick<StudioLocalOps, 'readSource'>

export const APPLY_FIX_READY = true
