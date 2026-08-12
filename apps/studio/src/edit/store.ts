import type { SailHost } from '../vendor/runtime'
/**
 * Edit surface store — explorer + documents + view switcher + selection sync.
 * Consumes SailInspect / SailRender / SailAuthoring via injected OpsClient (T-I2).
 */

import { type ViewportId, clearSelection, getSelection, getShellState, openTab, selectNode, setModelRev, setSailPath, setTabDirty, subscribeSelection } from '../vendor/studio-shell'
import {
  elementRows,
  fileRows as _fileRows,
  groupHeaders,
  parseTreeNodes,
  pathToFileEntry,
} from './explorer'
import { hitTestRegion, parseRegions } from './region-hit'
import { userError } from '../user-error'
import {
  a11yRows,
  anatomyOfSelected,
  dataRows,
  layoutRows,
  parseBindings,
  parseCheckDiagnostics,
  parseDataKeys,
  parseLayoutBoxes,
  parseThreads,
  stateCards,
  structuredEditOps,
  childrenOfSelected,
  tokensOfSelected,
  applySourceScroll,
  applySourceEditorScroll,
} from './views'
import type {
  DocViewId,
  EditDocument,
  EditSurfaceState,
  ElementRow,
  FileEntry,
  FileRow,
  HoverTip,
  OpsClient,
  PreviewState,
  SourceState,
  ThreadRow,
} from './types'
import { DOC_VIEWS } from './types'

export { groupHeaders, DOC_VIEWS, applySourceScroll, applySourceEditorScroll }
export { fileRows as buildFileRows } from './explorer'

type Listener = (s: EditSurfaceState) => void

let client: OpsClient | null = null
let state: EditSurfaceState = emptyState()
const listeners = new Set<Listener>()
let unsubSelection: (() => void) | null = null

function emptySource(text = ''): SourceState {
  return {
    text,
    dirty: false,
    status: text ? 'Loaded' : '—',
    cursorSpan: '—',
    checks: [],
    checksLabel: '',
  }
}

function emptyState(): EditSurfaceState {
  return {
    projectRoot: '',
    filter: '',
    files: [],
    activePath: null,
    view: 'source',
    docs: new Map(),
    modelRev: 0,
    selectionNodeId: null,
    sourceScrollLine: null,
    sourceScrollCol: null,
    sourceScrollNonce: 0,
    hoverTip: { nodeId: '', label: '', visible: false },
    threads: [],
    autoFixMode: 'off',
    error: null,
    loading: false,
  }
}

function emit(): void {
  for (const fn of listeners) fn(state)
}

export function getEditState(): EditSurfaceState {
  return state
}

export function subscribeEdit(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

export function setOpsClient(c: OpsClient | null): void {
  client = c
}

export function getOpsClient(): OpsClient | null {
  return client
}

/** Wire shell selection → edit surface (and reverse via selectNode). */
export function bindEditSelection(): () => void {
  unsubSelection?.()
  unsubSelection = subscribeSelection((sel) => {
    if (state.selectionNodeId === sel.nodeId) return
    const span = spanForSelection(sel.nodeId)
    state = {
      ...state,
      selectionNodeId: sel.nodeId,
      sourceScrollLine: span.line,
      sourceScrollCol: span.col,
      sourceScrollNonce: state.sourceScrollNonce + 1,
    }
    emit()
  })
  return () => {
    unsubSelection?.()
    unsubSelection = null
  }
}

function spanForSelection(nodeId: string | null): {
  line: number | null
  col: number | null
} {
  if (!nodeId || !state.activePath) return { line: null, col: null }
  const doc = state.docs.get(state.activePath)
  if (!doc) return { line: null, col: null }
  const n = doc.tree.nodes.find((x) => x.id === nodeId)
  if (!n) return { line: null, col: null }
  return { line: n.span.line ?? null, col: n.span.col ?? 1 }
}

export function setProjectRoot(root: string): void {
  state = { ...state, projectRoot: root }
  emit()
}

export function setFilter(filter: string): void {
  state = { ...state, filter }
  emit()
}

export function setFiles(files: FileEntry[]): void {
  state = { ...state, files }
  emit()
}

export function seedFilesFromPaths(paths: string[]): void {
  setFiles(paths.map((p) => pathToFileEntry(p)))
}

export function setView(view: DocViewId): void {
  if (!DOC_VIEWS.includes(view)) return
  if (state.view === view) return
  const path = state.activePath
  if (path) {
    const doc = state.docs.get(path)
    if (doc) {
      const docs = new Map(state.docs)
      docs.set(path, { ...doc, view })
      state = { ...state, view, docs }
      emit()
      return
    }
  }
  state = { ...state, view }
  emit()
}

/**
 * Write a signal override (State view). Updates local doc values and posts to
 * the live preview iframe when mounted (kept mounted across views for bridge).
 */
export function writeSignalValue(name: string, value: string): void {
  const path = state.activePath
  if (!path || !name) return
  const doc = state.docs.get(path)
  if (!doc) return
  const docs = new Map(state.docs)
  docs.set(path, {
    ...doc,
    signalValues: { ...doc.signalValues, [name]: value },
  })
  state = { ...state, docs }
  emit()
  // Use the Window registered via setLivePreviewWindow (no ambient lookup).
  const win = livePreviewWindow
  if (!win) return
  // Prefer the registered window's origin (not ambient location) so postMessage
  // is scoped when the preview frame origin is known; fall back to '*'.
  let targetOrigin = '*'
  try {
    const o = (win as { location?: { origin?: string } }).location?.origin
    if (o && o !== 'null') targetOrigin = o
  } catch {
    targetOrigin = '*'
  }
  try {
    win.postMessage({ type: 'sail-studio-signal', name, value }, targetOrigin)
  } catch {
    /* cross-origin or missing */
  }
}

/** Register the live-preview contentWindow (template/controller owns the host). */
let livePreviewWindow: { postMessage(msg: unknown, origin: string): void; location?: { origin?: string } } | null = null
export function setLivePreviewWindow(w: { postMessage(msg: unknown, origin: string): void; location?: { origin?: string } } | null): void {
  livePreviewWindow = w
}

export function setAutoFixMode(mode: EditSurfaceState['autoFixMode']): void {
  state = { ...state, autoFixMode: mode }
  emit()
}

/** True for Sail sources that go through SailInspect. */
export function isSailSourcePath(path: string): boolean {
  return path.replace(/\\/g, '/').toLowerCase().endsWith('.sail')
}

function installOpenedDoc(
  path: string,
  doc: EditDocument,
  threads: ThreadRow[],
  gen: number,
): EditDocument | null {
  if (gen !== previewGen) return null
  const docs = new Map(state.docs)
  docs.set(path, doc)
  const rev = state.modelRev + 1
  state = {
    ...state,
    docs,
    activePath: path,
    view: doc.view,
    modelRev: rev,
    loading: false,
    error: null,
    threads,
  }
  setModelRev(rev)
  if (!state.files.some((f) => f.path === path)) {
    state = { ...state, files: [...state.files, pathToFileEntry(path)] }
  }
  emit()
  return state.docs.get(path) ?? doc
}

/**
 * Open a project path: .sail → SailInspect model (+ optional preview);
 * other text files → source-only (no inspect tree). Always selects the path.
 */
export async function openDocument(
  path: string,
  opts?: {
    viewport?: { width: number; height: number }
    theme?: string
    mode?: string
    loadPreview?: boolean
    sourceText?: string
  },
): Promise<EditDocument | null> {
  if (!client) {
    state = { ...state, error: 'Studio is not connected yet. Try reloading.' }
    emit()
    return null
  }
  // Own this open; supersedes in-flight refresh / prior open applies.
  const gen = ++previewGen
  state = { ...state, loading: true, error: null, activePath: path }
  emit()
  setSailPath(path)
  // Label is path-aware among open tabs (edit/page.sail vs bare page.sail).
  const openIds = [...getShellState().tabs.map((t) => t.id), path]
  const parts = path.replace(/\\/g, '/').split('/').filter(Boolean)
  const base = parts[parts.length - 1] ?? path
  const collisions = openIds.filter((id) => {
    const p = id.replace(/\\/g, '/')
    return (p.split('/').pop() || p) === base
  })
  const label =
    collisions.length > 1 && parts.length >= 2
      ? `${parts[parts.length - 2]}/${parts[parts.length - 1]}`
      : base
  openTab({ id: path, label, dirty: false })

  try {
    // Non-Sail: list in explorer, open as text when readSource works; otherwise
    // empty doc with a clear status (tree still selects the path).
    if (!isSailSourcePath(path)) {
      let sourceText = opts?.sourceText ?? ''
      if (!sourceText && client.readSource) {
        try {
          sourceText = await client.readSource(path)
        } catch {
          /* fall through */
        }
      }
      if (!sourceText) {
        sourceText = state.docs.get(path)?.source.text ?? ''
      }
      if (gen !== previewGen) return null
      const prior = state.docs.get(path)
      const docView = prior?.view ?? 'source'
      const hasText = sourceText.length > 0
      const source: SourceState = {
        text: sourceText,
        dirty: false,
        status: hasText
          ? 'Text file'
          : 'Non-Sail file — source not available here (open .sail or use CLI ops)',
        cursorSpan: '—',
        checks: [],
        checksLabel: '',
      }
      const doc: EditDocument = {
        path,
        label,
        modelVersion: prior?.modelVersion ?? 0,
        tree: { roots: [], nodes: [] },
        layout: { boxes: [] },
        state: { bindings: [] },
        data: { keys: [] },
        diagnostics: [],
        theme: prior?.theme,
        source,
        preview: emptyPreview(),
        view: docView,
        signalValues: prior?.signalValues ?? {},
      }
      return installOpenedDoc(path, doc, [], gen)
    }

    const model = await client.sailInspectModel({
      path,
      ...(opts?.viewport !== undefined ? { viewport: opts.viewport } : {}),
    })
    if (gen !== previewGen) return null

    const treeRaw = model.tree as { roots?: string[]; nodes?: unknown[] }
    const nodes = parseTreeNodes(treeRaw?.nodes ?? [])
    const roots = treeRaw?.roots ?? []
    const bindings = parseBindings(
      (model.state as { bindings?: unknown[] })?.bindings ?? [],
    )
    const boxes = parseLayoutBoxes(model.layout)
    const keys = parseDataKeys(model.data)

    // Source load order: explicit opt → Model.source → readSource → prior doc
    let sourceText = opts?.sourceText ?? ''
    if (!sourceText && typeof model.source === 'string' && model.source.length) {
      sourceText = model.source
    }
    if (!sourceText && client.readSource) {
      try {
        sourceText = await client.readSource(path)
      } catch {
        /* fall through */
      }
    }
    if (!sourceText) {
      sourceText = state.docs.get(path)?.source.text ?? ''
    }
    if (gen !== previewGen) return null

    let checks = parseCheckDiagnostics([])
    let checksLabel = ''
    if (client.sailAuthoringCheck) {
      try {
        const chk = await client.sailAuthoringCheck({ path })
        checks = parseCheckDiagnostics(chk.diagnostics ?? [])
        const n = checks.filter((c) => c.severity === 'error').length || checks.length
        checksLabel = n > 0 ? `${n} check${n === 1 ? '' : 's'}` : ''
      } catch {
        /* offline */
      }
    }

    const source: SourceState = {
      text: sourceText,
      dirty: false,
      status: sourceText ? 'Saved' : 'No source loaded',
      cursorSpan: '—',
      checks,
      checksLabel,
    }

    // Open-time preview is empty; load via refreshPreview after the doc is
    // installed so one gen-gated apply path owns screenshot settles.
    const preview: PreviewState = emptyPreview(
      opts?.loadPreview !== false ? { loading: true } : undefined,
    )

    let threads: ThreadRow[] = []
    if (client.sailTaskList) {
      try {
        const proj =
          path.split('/').slice(0, 2).join('/') || state.projectRoot || ''
        const tl = await client.sailTaskList(
          proj ? { project: proj } : {},
        )
        threads = parseThreads(tl.tasks ?? [], path)
      } catch {
        threads = []
      }
    }
    if (gen !== previewGen) return null

    const prior = state.docs.get(path)
    const docView = prior?.view ?? state.view
    const doc: EditDocument = {
      path,
      label,
      modelVersion: model.modelVersion,
      tree: { roots, nodes },
      layout: { boxes },
      state: { bindings },
      data: { keys },
      diagnostics: model.diagnostics ?? [],
      theme: model.theme,
      source,
      preview,
      view: docView,
      signalValues: prior?.signalValues ?? {},
    }

    const installed = installOpenedDoc(path, doc, threads, gen)
    if (!installed) return null

    // Re-check after install: a concurrent open may have taken gen/activePath.
    // Never call refreshPreview for a stolen activePath (it reads state.activePath).
    if (gen !== previewGen) return null

    if (opts?.loadPreview !== false) {
      await refreshPreview({
        ...(opts?.theme !== undefined ? { theme: opts.theme } : {}),
        ...(opts?.mode !== undefined ? { mode: opts.mode } : {}),
        ...(opts?.viewport !== undefined ? { viewport: opts.viewport } : {}),
      })
      // refreshPreview bumps gen; judge ownership by path, not open gen.
      if (state.activePath !== path) return null
    }
    return state.docs.get(path) ?? doc
  } catch (e) {
    if (gen !== previewGen) return null
    state = {
      ...state,
      loading: false,
      error: userError(e, 'Could not open that file — try again.'),
    }
    emit()
    return null
  }
}

/** Default cap so SailRender can never leave the surface on "Rendering…" forever. */
export const PREVIEW_TIMEOUT_MS = 25_000

let previewTimeoutMs = PREVIEW_TIMEOUT_MS
/**
 * Monotonic generation for preview loads. Each refreshPreview/openDocument bumps it;
 * only the settle whose gen is still current may write preview state (drops stale races).
 */
let previewGen = 0

/** Test-only: shorten (or restore) the preview hang timeout. */
export function __setPreviewTimeoutForTests(ms: number | null): void {
  previewTimeoutMs = ms == null ? PREVIEW_TIMEOUT_MS : ms
}

/** Test-only: current preview generation (for concurrent-refresh assertions). */
export function __previewGenForTests(): number {
  return previewGen
}

function emptyPreview(partial?: Partial<PreviewState>): PreviewState {
  return {
    imagePng: null,
    regions: [],
    dynamics: [],
    provenance: 'Audit snapshot',
    loading: false,
    error: null,
    ...partial,
  }
}

/**
 * Race a promise against a wall-clock timeout. Clears the timer in `finally`.
 * Does **not** cancel the underlying ops work (no AbortSignal on SailRender yet);
 * callers must use `previewGen` so late settles are ignored after supersede/retry.
 */
async function withTimeout<T>(
  promise: Promise<T>,
  ms: number,
  label: string,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_, reject) => {
        timer = setTimeout(
          () => reject(new Error(`${label} timed out after ${ms}ms`)),
          ms,
        )
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}

async function loadPreview(
  path: string,
  opts?: {
    theme?: string
    mode?: string
    viewport?: { width: number; height: number }
    timeoutMs?: number
  },
): Promise<PreviewState> {
  if (!client) {
    return emptyPreview({ error: 'Studio is not connected yet.' })
  }
  const timeoutMs = opts?.timeoutMs ?? previewTimeoutMs
  try {
    const vp = opts?.viewport ?? { width: 1440, height: 900 }
    const ret = await withTimeout(
      client.sailRenderScreenshot({
        path,
        theme: opts?.theme ?? 'ship',
        mode: opts?.mode ?? 'dark',
        viewport: `${vp.width}x${vp.height}`,
      }),
      timeoutMs,
      'Preview render',
    )
    return emptyPreview({
      imagePng: ret.imagePng,
      regions: parseRegions(ret.regions ?? []),
      dynamics: (ret.dynamics ?? []).map((d) => {
        const o = d as Record<string, unknown>
        return {
          nodeId: String(o.nodeId ?? o.id ?? ''),
          ...(o.reason != null ? { reason: String(o.reason) } : {}),
          ...(o.tag != null ? { tag: String(o.tag) } : {}),
        }
      }),
      loading: false,
      error: null,
    })
  } catch (e) {
    const timedOut =
      e instanceof Error && /timed out/i.test(e.message)
    return emptyPreview({
      loading: false,
      error: userError(
        e,
        timedOut
          ? 'Preview timed out — try again, or open a smaller file.'
          : 'Could not render the audit snapshot — try again.',
      ),
    })
  }
}

export type RefreshPreviewResult = { applied: boolean }

/**
 * Load/refresh audit screenshot for the active doc.
 * Returns `{ applied: true }` only when this generation still owns the settle
 * (stale concurrent refresh/open returns `{ applied: false }`).
 */
export async function refreshPreview(opts?: {
  theme?: string
  mode?: string
  viewport?: { width: number; height: number }
  timeoutMs?: number
}): Promise<RefreshPreviewResult> {
  const path = state.activePath
  if (!path) return { applied: false }
  const doc = state.docs.get(path)
  if (!doc) return { applied: false }
  // Bump gen so a concurrent/prior refresh cannot apply after this one starts
  // (retry and re-open supersede in-flight settles; ops may still run — see withTimeout).
  const gen = ++previewGen
  // Surface loading immediately so the UI never sticks on a silent hang.
  const docsLoading = new Map(state.docs)
  docsLoading.set(path, {
    ...doc,
    preview: {
      ...doc.preview,
      loading: true,
      error: null,
    },
  })
  state = { ...state, docs: docsLoading }
  emit()

  const preview = await loadPreview(path, opts)
  if (gen !== previewGen) {
    // Dropped settle: clear sticky loading only when focus moved away so a
    // newer same-path refresh (still loading) is not clobbered.
    if (state.activePath !== path) {
      const stale = state.docs.get(path)
      if (stale?.preview.loading) {
        const docs = new Map(state.docs)
        docs.set(path, {
          ...stale,
          preview: { ...stale.preview, loading: false },
        })
        state = { ...state, docs }
        emit()
      }
    }
    return { applied: false }
  }
  const latest = state.docs.get(path)
  if (!latest || state.activePath !== path) return { applied: false }
  const docs = new Map(state.docs)
  docs.set(path, { ...latest, preview })
  state = { ...state, docs }
  emit()
  return { applied: true }
}

/** Select node from explorer / preview / source — shared shell selection. */
export function selectEditNode(nodeId: string | null): void {
  selectNode(nodeId, nodeId ? 'surface' : 'clear')
  const span = spanForSelection(nodeId)
  state = {
    ...state,
    selectionNodeId: nodeId,
    sourceScrollLine: span.line,
    sourceScrollCol: span.col,
    sourceScrollNonce: state.sourceScrollNonce + 1,
  }
  emit()
}

/** Scroll source editor to a file span (Checks → Edit deep link). */
export function requestSourceScroll(line: number, col = 1): void {
  state = {
    ...state,
    sourceScrollLine: line,
    sourceScrollCol: col,
    sourceScrollNonce: state.sourceScrollNonce + 1,
  }
  emit()
}

/** Click on preview image → hit-test region → select. */
export function selectFromPreviewClick(
  x: number,
  y: number,
): string | null {
  const path = state.activePath
  if (!path) return null
  const doc = state.docs.get(path)
  if (!doc) return null
  const hit = hitTestRegion(doc.preview.regions, x, y)
  if (!hit) return null
  selectEditNode(hit.nodeId)
  return hit.nodeId
}

/** Hover region tooltip (tag + id) over preview image. Skips emit when unchanged. */
export function hoverPreviewAt(x: number, y: number): HoverTip {
  const path = state.activePath
  const empty: HoverTip = { nodeId: '', label: '', visible: false }
  const setTip = (tip: HoverTip): HoverTip => {
    const prev = state.hoverTip
    if (
      prev.visible === tip.visible &&
      prev.nodeId === tip.nodeId &&
      prev.label === tip.label
    ) {
      return prev
    }
    state = { ...state, hoverTip: tip }
    emit()
    return tip
  }
  if (!path) return setTip(empty)
  const doc = state.docs.get(path)
  if (!doc) return setTip(empty)
  const hit = hitTestRegion(doc.preview.regions, x, y)
  if (!hit) return setTip(empty)
  const node = doc.tree.nodes.find((n) => n.id === hit.nodeId)
  const tag = hit.tag || node?.tag || 'node'
  return setTip({
    nodeId: hit.nodeId,
    label: `<${tag}> · ${hit.nodeId.split('#').pop() ?? hit.nodeId}`,
    visible: true,
  })
}

export function clearHoverTip(): void {
  if (!state.hoverTip.visible) return
  state = {
    ...state,
    hoverTip: { nodeId: '', label: '', visible: false },
  }
  emit()
}

/**
 * Scroll source editor (sail-code-editor or textarea) to the selection span.
 * Returns true when the host was present and scroll applied.
 */
export function scrollSourceEditor(host: SailHost | null): boolean {
  if (!host || state.sourceScrollLine == null) return false
  return applySourceEditorScroll(
    host,
    state.sourceScrollLine,
    state.sourceScrollCol ?? 1,
  )
}

/**
 * Resolve the active source editor host (CodeMirror CE or textarea).
 * Returns the host registered via setSourceEditorHost only (no ambient lookup).
 */
let sourceEditorHost: SailHost | null = null
export function setSourceEditorHost(el: SailHost | null): void {
  sourceEditorHost = el
}
export function querySourceEditor(): SailHost | null {
  return sourceEditorHost
}

export function setSourceText(path: string, text: string): void {
  const doc = state.docs.get(path)
  if (!doc) return
  const docs = new Map(state.docs)
  docs.set(path, {
    ...doc,
    source: {
      ...doc.source,
      text,
      dirty: true,
      status: 'Unsaved',
    },
  })
  state = { ...state, docs }
  setTabDirty(path, true)
  emit()
}

export function setSourceTextFromEditor(text: string): void {
  if (!state.activePath) return
  setSourceText(state.activePath, text)
}

/**
 * Save active source via writeSource (local FS) or sail_edit dry path;
 * bump model rev; reload model. Returns new rev or null on failure.
 */
export async function saveActiveSource(): Promise<number | null> {
  const path = state.activePath
  if (!path || !client) return null
  const doc = state.docs.get(path)
  if (!doc) return null
  try {
    if (client.writeSource) {
      await client.writeSource(path, doc.source.text)
    } else {
      state = {
        ...state,
        error: 'Could not save — write is unavailable.',
      }
      emit()
      return null
    }
    // Reload model so tree/state match disk
    const next = await openDocument(path, {
      sourceText: doc.source.text,
      loadPreview: false,
    })
    if (!next) return null
    // openDocument already reloaded source/checks; just ensure dirty cleared
    setTabDirty(path, false)
    emit()
    return state.modelRev
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not save — try again.'),
    }
    emit()
    return null
  }
}

/** Inject source text without dirty (e.g. after fs read). */
export function seedSourceText(path: string, text: string): void {
  const doc = state.docs.get(path)
  if (!doc) {
    // Placeholder until openDocument
    const stub: EditDocument = {
      path,
      label: path.split('/').pop() ?? path,
      modelVersion: 0,
      tree: { roots: [], nodes: [] },
      layout: { boxes: [] },
      state: { bindings: [] },
      data: { keys: [] },
      diagnostics: [],
      theme: null,
      source: emptySource(text),
      preview: {
        imagePng: null,
        regions: [],
        dynamics: [],
        provenance: 'Audit snapshot',
        loading: false,
        error: null,
      },
      view: state.view,
      signalValues: {},
    }
    const docs = new Map(state.docs)
    docs.set(path, stub)
    state = { ...state, docs }
    emit()
    return
  }
  const docs = new Map(state.docs)
  docs.set(path, {
    ...doc,
    source: { ...doc.source, text, dirty: false, status: 'Saved' },
  })
  state = { ...state, docs }
  emit()
}

/** Apply model payload from watch / second view without full re-fetch. */
export function applyModelUpdate(
  path: string,
  payload: {
    tree?: { roots?: string[]; nodes?: unknown[] }
    state?: { bindings?: unknown[] }
    layout?: unknown
    data?: unknown
    modelVersion?: number
  },
): void {
  const doc = state.docs.get(path)
  if (!doc) return
  const docs = new Map(state.docs)
  const next: EditDocument = {
    ...doc,
    modelVersion: payload.modelVersion ?? doc.modelVersion,
    tree: payload.tree
      ? {
          roots: payload.tree.roots ?? doc.tree.roots,
          nodes: parseTreeNodes(payload.tree.nodes ?? []),
        }
      : doc.tree,
    state: payload.state
      ? { bindings: parseBindings(payload.state.bindings ?? []) }
      : doc.state,
    layout: payload.layout
      ? { boxes: parseLayoutBoxes(payload.layout) }
      : doc.layout,
    data: payload.data ? { keys: parseDataKeys(payload.data) } : doc.data,
  }
  docs.set(path, next)
  const rev = state.modelRev + 1
  state = { ...state, docs, modelRev: rev }
  setModelRev(rev)
  emit()
}

// ── Derived UI snapshots (for Sail controller binding) ─────────────────────

export function activeDoc(): EditDocument | null {
  return state.activePath ? state.docs.get(state.activePath) ?? null : null
}

export function uiFileRows(): FileRow[] {
  return _fileRows(state.files, state.filter, state.activePath)
}

export function uiElementRows(): ElementRow[] {
  const doc = activeDoc()
  if (!doc) return []
  return elementRows(doc.tree.nodes, doc.tree.roots, state.selectionNodeId)
}

export function uiStateCards() {
  const doc = activeDoc()
  const cards = stateCards(doc?.state.bindings ?? [])
  const overrides = doc?.signalValues ?? {}
  return cards.map((c) => ({
    ...c,
    value: overrides[c.name] ?? '',
  }))
}

/** JSON snapshot for sail-json-view (state signals + overrides). */
export function uiStateJson(): string {
  const doc = activeDoc()
  if (!doc) return '{}'
  const body: Record<string, unknown> = {}
  for (const b of doc.state.bindings) {
    body[b.name] = {
      kind: b.kind,
      ssr: b.ssr_tier,
      value: doc.signalValues[b.name] ?? null,
      readers: b.readers.length,
      writers: b.writers.length,
    }
  }
  return JSON.stringify(body, null, 2)
}

/** JSON snapshot for Data view keys. */
export function uiDataJson(): string {
  const keys = activeDoc()?.data.keys ?? []
  const body = Object.fromEntries(
    keys.map((k) => [
      k.key,
      { kind: k.kind, ssr_fetchable: k.ssr_fetchable },
    ]),
  )
  return JSON.stringify(body, null, 2)
}

export function uiLayoutRows(viewport?: ViewportId) {
  const doc = activeDoc()
  if (!doc) return []
  return layoutRows(
    doc.layout.boxes,
    doc.tree.nodes,
    state.selectionNodeId,
    viewport,
  )
}

export function uiDataRows() {
  return dataRows(activeDoc()?.data.keys ?? [])
}

export function uiA11yRows(themeId: string, modeId: string) {
  return a11yRows(activeDoc()?.tree.nodes ?? [], themeId, modeId)
}

export function uiChildren() {
  const doc = activeDoc()
  if (!doc) return []
  return childrenOfSelected(doc.tree.nodes, state.selectionNodeId)
}

export function uiAnatomy() {
  const doc = activeDoc()
  if (!doc) return []
  return anatomyOfSelected(doc.tree.nodes, state.selectionNodeId)
}

export function uiTokens() {
  const doc = activeDoc()
  if (!doc) return []
  return tokensOfSelected(
    doc.tree.nodes,
    state.selectionNodeId,
    doc.source.text,
  )
}

export function uiStructuredOps() {
  return structuredEditOps()
}

export function uiThreads(tasks?: unknown[]): ThreadRow[] {
  if (tasks) return parseThreads(tasks, state.activePath)
  return state.threads
}

export function uiCheckDiags() {
  return activeDoc()?.source.checks ?? []
}

export function viewFlags(view: DocViewId = state.view) {
  return {
    isPreview: view === 'preview',
    isSource: view === 'source',
    isState: view === 'state',
    isLayout: view === 'layout',
    isData: view === 'data',
    isA11y: view === 'a11y',
    isThreads: view === 'threads',
  }
}

export function __resetEditForTests(): void {
  state = emptyState()
  client = null
  previewTimeoutMs = PREVIEW_TIMEOUT_MS
  previewGen = 0
  unsubSelection?.()
  unsubSelection = null
  listeners.clear()
  clearSelection()
}

export { getSelection }
