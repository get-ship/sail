import type { StudioLocalOps, StudioOpsMethods } from '../vendor/studio-client'

export type DocViewId =
  | 'preview'
  | 'source'
  | 'state'
  | 'layout'
  | 'data'
  | 'a11y'
  | 'threads'

export const DOC_VIEWS: DocViewId[] = [
  'preview',
  'source',
  'state',
  'layout',
  'data',
  'a11y',
  'threads',
]

export type ModelSpan = {
  start: number
  end: number
  line: number
  col: number
  end_line: number
  end_col: number
}

export type TreeNode = {
  id: string
  file: string
  span: ModelSpan
  tag: string
  layer: string
  attrs?: Record<string, string> | undefined
  control_flow?: { kind: string; expr?: string | undefined } | undefined
  children: string[]
}

export type LayoutBox = {
  id: string
  file: string
  span: ModelSpan
  viewport: { width: number; height: number }
  x: number
  y: number
  w: number
  h: number
  collapsed: boolean
  overflow: boolean
  tag: string
  unknown_join?: boolean | undefined
}

export type StateBinding = {
  id: string
  file: string
  span: ModelSpan
  name: string
  kind: string
  source: string
  ssr_tier: string
  readers: StateEdge[]
  writers: StateEdge[]
}

export type StateEdge = {
  id: string
  file: string
  span: ModelSpan
  kind: string
  expr?: string | undefined
}

export type DataKey = {
  id: string
  file: string
  span: ModelSpan
  kind: string
  key: string
  ssr_fetchable: boolean
}

export type RenderRegion = {
  nodeId: string
  x: number
  y: number
  w: number
  h: number
  tag?: string | undefined
  label?: string | undefined
}

export type FileEntry = {
  /** Workspace-relative path */
  path: string
  /** Display name (basename) */
  name: string
  /** Group: routes | src | packs | other */
  group: string
  /** Check error count badge (0 = none) */
  checks: number
}

export type ElementRow = {
  id: string
  tag: string
  layer: string
  layerBadge: string
  depth: number
  label: string
  selected: boolean
  spanLine: number
  controlFlow?: string | undefined
}

export type FileRow = {
  path: string
  name: string
  group: string
  checks: number
  checksLabel: string
  hasChecks: boolean
  selected: boolean
  indent: number
}

export type StateCard = {
  id: string
  name: string
  kind: string
  /** Raw model code S|I|C (tests + data attrs). */
  ssrTier: string
  /** Human badge label for the debug rail. */
  tierLabel: string
  tierWhy: string
  spanLabel: string
  readersLabel: string
  writersLabel: string
  readers: string[]
  writers: string[]
  /** Local write override (State view). */
  value?: string
}

export type LayoutRow = {
  id: string
  tag: string
  depth: number
  line: string
  selected: boolean
  collapsed: boolean
  overflow: boolean
}

export type DataRow = {
  id: string
  key: string
  kind: string
  ssrFetchable: boolean
  freshness: string
}

export type A11yRow = {
  id: string
  check: string
  status: 'pass' | 'warn' | 'na'
  detail: string
}

export type ThreadRow = {
  id: string
  title: string
  status: string
  nodeId: string | null
}

export type AnatomyRow = {
  id: string
  kind: 'part' | 'prop' | 'state' | 'meta'
  label: string
  detail: string
}

export type TokenSwatch = {
  id: string
  token: string
}

export type HoverTip = {
  nodeId: string
  label: string
  visible: boolean
}

export type CheckDiag = {
  line: number
  /** 1-based column when the checker provides a span. */
  column?: number | undefined
  endLine?: number | undefined
  endColumn?: number | undefined
  severity: string
  message: string
  rule?: string | undefined
}

export type PreviewState = {
  imagePng: string | null
  regions: RenderRegion[]
  dynamics: { nodeId: string; reason?: string | undefined; tag?: string | undefined }[]
  provenance: string
  loading: boolean
  error: string | null
}

export type SourceState = {
  text: string
  dirty: boolean
  status: string
  cursorSpan: string
  /** Gutter-style checks from SailAuthoring/Check */
  checks: CheckDiag[]
  checksLabel: string
}

export type StructuredEditOp = {
  id: string
  label: string
  disabled: boolean
}

export type EditDocument = {
  path: string
  label: string
  modelVersion: number
  tree: { roots: string[]; nodes: TreeNode[] }
  layout: { boxes: LayoutBox[] }
  state: { bindings: StateBinding[] }
  data: { keys: DataKey[] }
  diagnostics: unknown[]
  theme: unknown
  source: SourceState
  preview: PreviewState
  /** Per-doc view switcher (Preview·Source·State·Layout·Data·A11y·Threads). */
  view: DocViewId
  /** Local signal write overrides (name → text). Preview may consume via bridge. */
  signalValues: Record<string, string>
}

export type EditSurfaceState = {
  projectRoot: string
  filter: string
  files: FileEntry[]
  activePath: string | null
  view: DocViewId
  docs: Map<string, EditDocument>
  /** Local rev counter (bumps on save + model load). */
  modelRev: number
  selectionNodeId: string | null
  sourceScrollLine: number | null
  sourceScrollCol: number | null
  /** Bumped when selection should force textarea scroll/caret. */
  sourceScrollNonce: number
  hoverTip: HoverTip
  threads: ThreadRow[]
  autoFixMode: 'off' | 'suggest' | 'apply'
  error: string | null
  loading: boolean
}

/**
 * Edit shell ops client — Pick of generated methods + local write/read helpers.
 * Deleting a method from METHOD_CONTRACTS removes it from StudioOpsMethods and
 * fails the studio typecheck + contract-drift tests.
 */
export type OpsClient = Pick<
  StudioOpsMethods,
  | 'sailInspectModel'
  | 'sailInspectTree'
  | 'sailInspectState'
  | 'sailRenderScreenshot'
  // Required so one createOpsClient() satisfies Checks + Design surfaces
  // under exactOptionalPropertyTypes (shell-page-vm injection).
  | 'sailAuthoringCheck'
  | 'sailThemeList'
  | 'sailThemeRead'
  | 'sailThemeWrite'
  | 'sailThemeFork'
> &
  Partial<
    Pick<
      StudioOpsMethods,
      | 'sailAuthoringEdit'
      | 'sailAuthoringListFiles'
      | 'sailAuthoringLayoutCheck'
      | 'sailTaskList'
      | 'sailTaskGet'
      | 'sailTaskCreate'
      | 'sailTaskComment'
      | 'sailTaskAddArtifact'
      | 'sailTaskAgentNote'
      | 'sailTaskThreadCreate'
      | 'sailTaskThreadReply'
      | 'sailTaskThreadList'
      | 'sailTaskReanchor'
      | 'sailTaskCommitIntent'
      | 'sailInspectTokenUsage'
      | 'sailThemeDelete'
      | 'sailThemeGenerate'
      | 'sailThemeGraph'
      | 'sailThemeFontAdd'
      | 'sailThemeFontSearch'
      | 'sailThemeEffectiveIds'
    >
  > &
  StudioLocalOps
