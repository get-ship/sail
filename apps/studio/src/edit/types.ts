
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
  attrs?: Record<string, string>
  control_flow?: { kind: string; expr?: string }
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
  unknown_join?: boolean
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
  expr?: string
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
  tag?: string
  label?: string
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
  controlFlow?: string
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
  severity: string
  message: string
  rule?: string
}

export type PreviewState = {
  imagePng: string | null
  regions: RenderRegion[]
  dynamics: { nodeId: string; reason?: string; tag?: string }[]
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

export type OpsClient = {
  sailInspectModel: (args: {
    path: string
    viewport?: { width: number; height: number }
  }) => Promise<{
    data: unknown
    diagnostics: unknown[]
    file: string
    source?: string
    layout: unknown
    modelVersion: number
    state: unknown
    theme: unknown
    tree: unknown
  }>
  sailInspectTree: (args: { path: string }) => Promise<{
    file: string
    modelVersion: number
    nodes: unknown[]
    roots: string[]
  }>
  sailInspectState: (args: { path: string }) => Promise<{
    bindings: unknown[]
    file: string
    modelVersion: number
  }>
  sailRenderScreenshot: (args: {
    path?: string
    source?: string
    theme?: string
    mode?: string
    viewport?: unknown
  }) => Promise<{
    dynamics: unknown[]
    imagePng: string
    ms: number
    regions: unknown[]
    theme: string
    viewport: unknown
  }>
    sailAuthoringEdit?: (args: {
    path: string
    ops: unknown[]
    source?: string
    autofix?: string
    actor?: string
    dryRun?: boolean
  }) => Promise<{
    applied: boolean
    modelRevision: string
    diagnostics: unknown[]
    autoFixed: unknown[]
    source?: string
    message?: string
  }>
  writeSource?: (path: string, content: string) => Promise<void>
  sailAuthoringListFiles?: (args: {
    project?: string
  }) => Promise<{ ok: boolean; project: string; files: string[] }>
  sailAuthoringCheck?: (args: {
    path?: string
    source?: string
  }) => Promise<{ count: number; diagnostics: unknown[]; ok: boolean }>
  sailAuthoringLayoutCheck?: (args: {
    path?: string
    source?: string
  }) => Promise<{
    diagnostics?: unknown[]
    viewports?: unknown[]
    count?: number
    ok?: boolean
    [key: string]: unknown
  }>
  sailTaskList?: (args: {
    project?: string
    status?: string
    actor?: string
    drift?: string
    open?: boolean
  }) => Promise<{ ok: boolean; tasks: unknown[] }>
  sailTaskGet?: (args: {
    project?: string
    id: string
  }) => Promise<{ ok: boolean; task: unknown }>
  sailTaskCreate?: (args: {
    project?: string
    title: string
    body?: string
    anchor?: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskComment?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
  sailTaskAddArtifact?: (args: {
    project?: string
    id: string
    filename?: string
    content_base64?: string
    mime?: string
    kind?: string
    label?: string
    region?: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; artifact_id?: string; path?: string; task?: unknown }>
  sailTaskAgentNote?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
  sailTaskThreadCreate?: (args: {
    project?: string
    title: string
    body?: string
    anchor: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskThreadReply?: (args: {
    project?: string
    id: string
    body: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; event?: unknown; task?: unknown }>
  sailTaskThreadList?: (args: {
    project?: string
    status?: string
    actor?: string
    drift?: string
    open?: boolean
  }) => Promise<{ ok: boolean; tasks: unknown[] }>
  sailTaskReanchor?: (args: {
    project?: string
    id: string
    anchor: Record<string, unknown>
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; task?: unknown }>
  sailTaskCommitIntent?: (args: {
    project?: string
    id: string
    actor?: { kind?: string; name?: string }
  }) => Promise<{ ok?: boolean; path?: string; bytes?: number; task?: unknown }>
  sailInspectTokenUsage?: (args: { token: string }) => Promise<{
    token?: string
    components?: number
    componentIds?: string[]
    sites?: unknown[]
  }>
  sailThemeList?: (args: { path: string }) => Promise<{
    files?: string[]
    ok?: boolean
    path?: string
  }>
  sailThemeRead?: (args: { path: string }) => Promise<{
    content: string
    ok?: boolean
    path?: string
  }>
  sailThemeWrite?: (args: {
    path: string
    content: string
  }) => Promise<{ ok?: boolean; path?: string; bytes?: number }>
  sailThemeFork?: (args: {
    source_path: string
    dest_path: string
    new_id?: string
    content?: string
  }) => Promise<{ ok?: boolean; dest_path?: string; path?: string }>
  sailThemeDelete?: (args: {
    path: string
  }) => Promise<{ ok?: boolean; path?: string }>
  /** SailTheme/Generate — mode=app after project theme Save. */
  sailThemeGenerate?: (args: {
    mode?: string
    project?: string
    workspace?: string
    model?: string
    out?: string
  }) => Promise<Record<string, unknown>>
  sailThemeGraph?: (args: { themeId?: string }) => Promise<Record<string, unknown>>
  sailThemeFontAdd?: (args: {
    family: string
    axes?: string
  }) => Promise<Record<string, unknown>>
  sailThemeFontSearch?: (args: {
    query: string
  }) => Promise<{ families?: string[] }>
  sailThemeEffectiveIds?: (args: {
    project?: string
  }) => Promise<{ all?: string[]; project?: string[]; catalog?: string[] }>
  /** Optional FS/ops read when Model.source is absent. */
  readSource?: (path: string) => Promise<string>
}
