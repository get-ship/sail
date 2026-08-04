/** Components inspect domain — surface picker, IR tree, instance panels (SailInspect). */
import { createTable } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  activeDoc,
  elementSailTreeNodes,
  getEditState,
  getOpsClient,
  subscribeEdit,
  type SailTreeNode,
} from '@ship/studio/edit'
import { COMPONENTS_CATALOG } from '../../components.gen'
import {
  extractTagName,
  isProjectComponentPath,
  projectComponentId,
  recordsFromCatalog,
} from '../components/catalog'
import type {
  IssueRowUi,
  LayoutBoxUi,
  StateRowUi,
  SurfaceOption,
} from '../components/types'
import type { ComponentsSession } from '../components-session/components-session'
import { stateCards } from '../edit/views'

/** Pack + project ids for "Show in library" honesty. */
function knownLibraryIds(): Set<string> {
  const ids = new Set(recordsFromCatalog(COMPONENTS_CATALOG).map((r) => r.id))
  for (const f of getEditState().files ?? []) {
    if (isProjectComponentPath(f.path)) ids.add(projectComponentId(f.path))
  }
  return ids
}

export type ComponentsInspect = ReturnType<typeof createComponentsInspect>

function flattenTree(ns: SailTreeNode[]): SailTreeNode[] {
  const out: SailTreeNode[] = []
  const walk = (list: SailTreeNode[]) => {
    for (const n of list) {
      out.push(n)
      if (n.children?.length) walk(n.children)
    }
  }
  walk(ns)
  return out
}

function surfaceLabel(path: string): string {
  const base = path.split('/').pop() || path
  if (!path.includes('/routes/')) return path.split('/').slice(-2).join('/')
  if (base === 'page.sail') {
    return path.replace(/^.*\/routes\//, '/').replace(/\/page\.sail$/, '') || '/'
  }
  return path.replace(/^.*\/routes\//, '')
}

export function createComponentsInspect(session: ComponentsSession) {
  const surfaces = signal<SurfaceOption[]>([])
  const selectedSurface = signal('')
  const hasSurface = signal(false)
  const isLoading = signal(false)
  const loadingLabel = signal('')
  const hasTree = signal(false)
  const treeFilter = signal('')

  const dataSig = signal<SailTreeNode[]>([])
  const tree = createTable<SailTreeNode>({
    data: dataSig,
    columns: [],
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    enableRowSelection: true,
    filterFromLeafRows: true,
    navMode: 'row',
    initialState: { expanded: true },
  })

  const hasNode = signal(false)
  const nodeTag = signal('')
  const nodeId = signal('')
  const nodeLayer = signal('')
  const nodePath = signal('')
  const nodeSpanLabel = signal('')
  const nodeLine = signal(0)
  const nodeCol = signal(0)
  const canOpenNode = signal(false)
  const mappedComponentId = signal('')
  const canShowInLibrary = signal(false)

  const issueRows = signal<IssueRowUi[]>([])
  const hasIssues = signal(false)
  const layoutRows = signal<LayoutBoxUi[]>([])
  const hasLayout = signal(false)
  const stateRows = signal<StateRowUi[]>([])
  const hasState = signal(false)
  const panelsNote = signal('')

  /** Tree/model load generation — stale settles no-op. */
  let loadGen = 0
  let selectedNodeId = ''

  const refreshSurfaces = () => {
    const es = getEditState()
    const opts: SurfaceOption[] = []
    const seen = new Set<string>()
    const add = (path: string, kind: 'route' | 'file' | 'open' = 'file') => {
      if (!path || seen.has(path) || !path.endsWith('.sail')) return
      seen.add(path)
      const label = surfaceLabel(path)
      opts.push({
        value: path,
        label: kind === 'open' ? `${label} · open` : label,
        key: path,
        kind: path.includes('/routes/') ? 'route' : kind,
      } as SurfaceOption & { kind?: string })
    }
    if (es.activePath) add(es.activePath, 'open')
    for (const t of es.docs.keys()) add(t, 'open')
    for (const f of es.files ?? []) {
      if (f.path.includes('/routes/') && f.path.endsWith('page.sail')) add(f.path, 'route')
    }
    for (const f of es.files ?? []) {
      if (!f.path.includes('/routes/')) add(f.path, 'file')
    }
    const cur = selectedSurface()
    const sliced = opts.slice(0, 80).map((o) => ({ ...o, selected: o.value === cur }))
    surfaces.set(sliced)
    if (!cur && sliced[0]) {
      selectedSurface.set(sliced[0].value)
      hasSurface.set(true)
      sliced[0].selected = true
    } else {
      hasSurface.set(!!selectedSurface())
    }
  }

  const onSurfacePick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[surface-path]') as HTMLElement | null
    const path = el?.dataset.surfacePath
    if (!path) return
    selectedSurface.set(path)
    hasSurface.set(true)
    refreshSurfaces()
    void loadTree(path)
  }

  const clearNode = () => {
    selectedNodeId = ''
    hasNode.set(false)
    nodeTag.set('')
    nodeId.set('')
    nodeLayer.set('')
    nodePath.set('')
    nodeSpanLabel.set('')
    nodeLine.set(0)
    nodeCol.set(0)
    canOpenNode.set(false)
    mappedComponentId.set('')
    canShowInLibrary.set(false)
    tree.resetRowSelection()
  }

  const publishNode = (node: SailTreeNode | null, path: string) => {
    if (!node) {
      clearNode()
      return
    }
    selectedNodeId = node.id
    hasNode.set(true)
    nodeTag.set(node.label || node.id)
    nodeId.set(node.id)
    nodeLayer.set(node.trailing || '')
    nodePath.set(path)
    const modelNode = activeDoc()?.tree.nodes.find((n) => n.id === node.id)
    const span = modelNode?.span
    nodeSpanLabel.set(span ? `L${span.line}:${span.col}` : '—')
    nodeLine.set(span?.line ?? 0)
    nodeCol.set(span?.col ?? 0)
    canOpenNode.set(!!path)
    const tag = extractTagName(modelNode?.tag, node.label || '')
    mappedComponentId.set(tag)
    canShowInLibrary.set(!!tag && knownLibraryIds().has(tag))
  }

  const publishPanelsFromDoc = () => {
    const doc = activeDoc()
    if (!doc || doc.path !== selectedSurface()) {
      issueRows.set([])
      hasIssues.set(false)
      layoutRows.set([])
      hasLayout.set(false)
      stateRows.set([])
      hasState.set(false)
      panelsNote.set(
        selectedSurface()
          ? 'Open this surface in Edit to load diagnostics, layout, and state — or wait for inspect model.'
          : 'Select a surface to inspect.',
      )
      return
    }
    const diags = doc.diagnostics ?? []
    const issues: IssueRowUi[] = diags.slice(0, 40).map((d, i) => {
      const sev = String((d as { severity?: string }).severity ?? 'info')
      const tone =
        sev === 'error' || sev === 'Error'
          ? 'destructive'
          : sev === 'warning' || sev === 'Warning'
            ? 'warning'
            : 'muted'
      const span = (d as { span?: { line?: number; col?: number } }).span
      return {
        id: String((d as { id?: string }).id ?? i),
        severity: sev,
        tone,
        message: String((d as { message?: string }).message ?? d),
        spanLabel: span?.line ? `L${span.line}:${span.col ?? 1}` : '—',
        key: `iss-${i}`,
      }
    })
    issueRows.set(issues)
    hasIssues.set(issues.length > 0)

    const boxes = doc.layout?.boxes ?? []
    const layouts: LayoutBoxUi[] = boxes.slice(0, 24).map((b, i) => ({
      id: b.id,
      tag: b.tag,
      dims: `${Math.round(b.w)}×${Math.round(b.h)} @ ${Math.round(b.x)},${Math.round(b.y)}`,
      flags: [b.collapsed ? 'collapsed' : '', b.overflow ? 'overflow' : '']
        .filter(Boolean)
        .join(' · ') || 'ok',
      key: `box-${b.id || i}`,
    }))
    layoutRows.set(layouts)
    hasLayout.set(layouts.length > 0)

    const cards = stateCards(doc.state?.bindings ?? [])
    const states: StateRowUi[] = cards.slice(0, 24).map((c) => ({
      id: c.id,
      name: c.name,
      kind: c.kind,
      tierLabel: c.tierLabel || c.ssrTier,
      tierTone:
        c.ssrTier === 'C' ? 'warning' : c.ssrTier === 'S' ? 'success' : 'muted',
      key: c.id,
    }))
    stateRows.set(states)
    hasState.set(states.length > 0)
    panelsNote.set(
      issues.length || layouts.length || states.length
        ? ''
        : 'No diagnostics, layout boxes, or state bindings on this surface.',
    )
  }

  const applyTreeFromDoc = (path: string) => {
    const es = getEditState()
    const doc = es.docs.get(path) ?? (es.activePath === path ? activeDoc() : null)
    if (!doc) {
      dataSig.set([])
      hasTree.set(false)
      clearNode()
      publishPanelsFromDoc()
      return
    }
    const nodes = elementSailTreeNodes(doc.tree.nodes, doc.tree.roots)
    dataSig.set(nodes)
    hasTree.set(nodes.length > 0)
    if (selectedNodeId) {
      const found = flattenTree(nodes).find((n) => n.id === selectedNodeId) ?? null
      publishNode(found, path)
      if (found) tree.setRowSelection({ mode: 'include', ids: new Set([found.id]) })
    }
    publishPanelsFromDoc()
  }

  const loadTree = async (path: string) => {
    if (!path) {
      // Invalidate in-flight load so prior path cannot re-apply (M3).
      loadGen += 1
      isLoading.set(false)
      loadingLabel.set('')
      selectedSurface.set('')
      hasSurface.set(false)
      dataSig.set([])
      hasTree.set(false)
      clearNode()
      return
    }
    const gen = ++loadGen
    isLoading.set(true)
    loadingLabel.set('Loading tree…')
    selectedSurface.set(path)
    hasSurface.set(true)

    try {
      // Prefer already-open edit doc (no dual ops)
      const es = getEditState()
      if (es.docs.has(path) || es.activePath === path) {
        if (gen !== loadGen) return
        applyTreeFromDoc(path)
        return
      }

      // Try SailInspect/Tree when ops present
      try {
        const ops = getOpsClient()
        if (ops?.sailInspectTree) {
          const ret = await ops.sailInspectTree({ path })
          if (gen !== loadGen) return
          const roots = (ret as { roots?: string[] }).roots
          const nodes = (ret as { nodes?: unknown[] }).nodes
          if (Array.isArray(nodes) && nodes.length && Array.isArray(roots)) {
            const byId = new Map<string, import('../edit/types').TreeNode>()
            for (const raw of nodes) {
              const o = raw as Record<string, unknown>
              const id = String(o.id ?? '')
              if (!id) continue
              const span = (o.span ?? {}) as Record<string, unknown>
              byId.set(id, {
                id,
                file: String(o.file ?? path),
                span: {
                  start: Number(span.start ?? 0),
                  end: Number(span.end ?? 0),
                  line: Number(span.line ?? 0),
                  col: Number(span.col ?? 0),
                  end_line: Number(span.end_line ?? 0),
                  end_col: Number(span.end_col ?? 0),
                },
                tag: String(o.tag ?? o.name ?? 'node'),
                layer: String(o.layer ?? ''),
                children: Array.isArray(o.children)
                  ? (o.children as unknown[]).map(String)
                  : [],
              })
            }
            const treeNodes = [...byId.values()]
            const sailNodes = elementSailTreeNodes(treeNodes, roots)
            dataSig.set(sailNodes)
            hasTree.set(sailNodes.length > 0)
            publishPanelsFromDoc()
            return
          }
        }
        if (ops?.sailInspectModel) {
          await ops.sailInspectModel({ path })
          if (gen !== loadGen) return
        }
      } catch {
        /* honest empty */
      }
      if (gen !== loadGen) return
      applyTreeFromDoc(path)
      if (!hasTree()) {
        panelsNote.set(
          'Inspect tree unavailable offline — open the surface in Edit, then return here.',
        )
      }
    } finally {
      if (gen === loadGen) {
        isLoading.set(false)
        loadingLabel.set('')
      }
    }
  }

  const onSurfaceChange = (e: Event) => {
    const v = (e.target as HTMLSelectElement).value
    void loadTree(v)
  }

  const onTreeSelect = (e: Event) => {
    const id = (e as CustomEvent<{ id?: string }>).detail?.id
    if (!id) return
    selectedNodeId = id
    publishNode(flattenTree(dataSig()).find((n) => n.id === id) ?? null, selectedSurface())
  }

  const onOpenNodeInEdit = () => {
    const path = nodePath() || selectedSurface()
    if (!path) {
      session.setNote('No surface path to open.', 'warning')
      return
    }
    void session.openInEdit(path, {
      line: nodeLine() || undefined,
      col: nodeCol() || undefined,
    })
  }

  const onShowInLibrary = () => {
    const id = mappedComponentId()
    if (!id || !knownLibraryIds().has(id)) {
      session.setNote(
        'Not in library — element skins live under Themes → Components.',
        'warning',
      )
      return
    }
    session.setSelectedId(id)
    session.applyMode('library')
  }

  const onTreeFilterInput = (e: Event) => {
    const q = (e.target as HTMLInputElement).value ?? ''
    treeFilter.set(q)
    tree.setGlobalFilter(q)
  }

  mountAmbient(() => {
    const unsub = session.onSync(() => {
      if (session.mode() === 'inspect') {
        refreshSurfaces()
        if (selectedSurface() && !hasTree() && !isLoading()) {
          void loadTree(selectedSurface())
        }
      }
    })
    const unsubEdit = subscribeEdit(() => {
      refreshSurfaces()
      if (session.mode() === 'inspect' && selectedSurface()) {
        applyTreeFromDoc(selectedSurface())
      }
    })
    refreshSurfaces()
    if (session.mode() === 'inspect' && selectedSurface()) {
      void loadTree(selectedSurface())
    }
    return () => {
      unsub()
      unsubEdit()
      loadGen += 1
    }
  })

  return {
    tree,
    surfaces,
    selectedSurface,
    hasSurface,
    isLoading,
    loadingLabel,
    hasTree,
    treeFilter,
    hasNode,
    nodeTag,
    nodeId,
    nodeLayer,
    nodePath,
    nodeSpanLabel,
    nodeLine,
    nodeCol,
    canOpenNode,
    mappedComponentId,
    canShowInLibrary,
    issueRows,
    hasIssues,
    layoutRows,
    hasLayout,
    stateRows,
    hasState,
    panelsNote,
    loadTree,
    onSurfaceChange,
    onSurfacePick,
    onTreeSelect,
    onOpenNodeInEdit,
    onShowInLibrary,
    onTreeFilterInput,
    refreshSurfaces,
    __loadGen: () => loadGen,
  }
}
